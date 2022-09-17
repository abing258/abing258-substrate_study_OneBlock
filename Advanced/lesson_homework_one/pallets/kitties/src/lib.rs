#![cfg_attr(not(feature = "std"), no_std)]

/// 方便让别的模块调用
pub use pallet::*;

/// 必须引入以下两个宏，才能对kitties模块进行单元测试
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::traits::{Randomness, ReservableCurrency};
	use frame_support::{pallet_prelude::*, traits::Currency};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32Bit, Bounded, CheckedAdd};
	use sp_io::hashing::blake2_128;


	#[pallet::type_value]
	pub fn GetDefaultValue<T: Config>() -> T::KittyIndex {
		0_u8.into()
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]);

	/// 定义账号余额
	/// 参考：substrate/frame/nicks/src/lib.rs中的定义
	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	/// 模块配置接口
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		// 定义KittyIndex类型，要求实现执行的trait
		// Paramter 表示可以用于函数参数传递
		// AtLeast32Bit 表示转换为u32不会造成数据丢失
		// Default 表示有默认值
		// Copy 表示实现Copy方法
		// Bounded 表示包含上界和下界
		// 以后开发遇到在Runtime中定义无符号整型，可以直接复制套用
		type KittyIndex: Parameter + AtLeast32Bit + Default + Copy + Bounded + MaxEncodedLen;

		/// 引入资产类型，以便支持质押
		/// 参考：substrate/frame/treasury/src/lib.rs中的定义
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		// 定义常量时，必须带上以下宏
		#[pallet::constant]
		// 获取Runtime中Kitties pallet定义的质押金额常量
		// 在创建Kitty前需要做质押，避免反复恶意创建
		type KittyPrice: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type MaxKittyIndex: Get<u32>;
	}

	#[pallet::pallet]
	//定义自己所需的存储项所需的宏
	#[pallet::generate_store(pub(super) trait Store)]
	//定义模块所需的结构体
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> = StorageValue<_, T::KittyIndex, ValueQuery, GetDefaultValue<T>>;  // KittyIndex移到Runtime后，KittyIndex改为T::KittyIndex

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn owner_kitties)]
	pub type OwnerKitties<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<T::KittyIndex, T::MaxKittyIndex>, ValueQuery>;

	#[pallet::event]
	//generate_deposit 生成了一个帮助方法 deposit_event
	//deposit_event 方便调用生成事件的宏
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, T::KittyIndex, Kitty),
		KittyBreed(T::AccountId, T::KittyIndex, Kitty),
		KittyTransfer(T::AccountId, T::KittyIndex, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NotEnoughBalance,
		KittyIdOverflow,
		OwnTooManyKitties,
		SameKittyId,
		NotExistKittyId,
		NotOwner,
	}

	#[pallet::hooks]
	//定义保留函数
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// 创建kitty
		#[pallet::weight(10_000)]
		#[frame_support::transactional]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			// 校验是否是一个签名的交易并获取sender
			let sender = ensure_signed(origin)?;

			let kitty_price = T::KittyPrice::get();
			ensure!(T::Currency::can_reserve(&sender, kitty_price), Error::<T>::NotEnoughBalance);

			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::KittyIdOverflow)?;
			let random = Self::random_value(&sender);
			let kitty = Kitty(random);

			T::Currency::reserve(&sender, kitty_price)?;

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &sender);
			let next_kitty_id = kitty_id
				.checked_add(&(T::KittyIndex::from(1_u8)))
				.ok_or(Error::<T>::KittyIdOverflow)
				.unwrap();
			NextKittyId::<T>::set(next_kitty_id);

			OwnerKitties::<T>::try_mutate(&sender, | ref mut kitties| {
				kitties.try_push(kitty_id).map_err(|_| Error::<T>::OwnTooManyKitties)?;
				Ok::<(), DispatchError>(())
			})?;

			// 发送一个成功的事件
			Self::deposit_event(Event::KittyCreated(sender, kitty_id, kitty));
			Ok({})
		}

		/// 孵化kitty
		#[pallet::weight(10_000)]
		#[frame_support::transactional]
		pub fn breed(origin: OriginFor<T>, kitty_id_one: T::KittyIndex, kitty_id_two: T::KittyIndex) -> DispatchResult {
			// 校验是否是一个签名的交易并获取sender
			let sender = ensure_signed(origin)?;
			let kitty_price = T::KittyPrice::get();
			ensure!(T::Currency::can_reserve(&sender, kitty_price), Error::<T>::NotEnoughBalance);

			ensure!(kitty_id_one != kitty_id_two, Error::<T>::SameKittyId);
			let kitty_one = Self::get_kitty(kitty_id_one).map_err(|_| Error::<T>::NotExistKittyId)?;
			let kitty_two = Self::get_kitty(kitty_id_two).map_err(|_| Error::<T>::NotExistKittyId)?;

			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::KittyIdOverflow)?;
			let random = Self::random_value(&sender);

			let mut kitty_data = [0u8; 16];
			for i in 0..kitty_one.0.len() {
				kitty_data[i] = (kitty_one.0[i] & random[i]) | (kitty_two.0[i] & !random[i]);
			}

			let kitty = Kitty(kitty_data);

			T::Currency::reserve(&sender, kitty_price)?;

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &sender);
			let next_kitty_id = kitty_id
				.checked_add(&(T::KittyIndex::from(1_u8)))
				.ok_or(Error::<T>::KittyIdOverflow)
				.unwrap();
			NextKittyId::<T>::set(next_kitty_id);

			// OwnerKitties::<T>::try_mutate(&sender, |ref mut kitties| {
			// 	let index = kitties.iter().position(|&r| r == kitty_id).unwrap();
			// 	kitties.remove(index);
			// 	Ok::<(), DispatchError>(())
			// })?;

			OwnerKitties::<T>::try_mutate(&sender, |ref mut kitties| {
				kitties.try_push(kitty_id).map_err(|_| Error::<T>::OwnTooManyKitties)?;
				Ok::<(), DispatchError>(())
			})?;

			// 发送一个成功的事件
			Self::deposit_event(Event::KittyBreed(sender, kitty_id, kitty));
			Ok({})
		}

		/// 转移kitty
		#[pallet::weight(10_000)]
		#[frame_support::transactional]
		pub fn transfer(origin: OriginFor<T>, kitty_id: T::KittyIndex, new_owner: T::AccountId) -> DispatchResult {
			// 校验是否是一个签名的交易并获取sender
			let sender = ensure_signed(origin)?;
			let kitty_price = T::KittyPrice::get();
			ensure!(T::Currency::can_reserve(&new_owner, kitty_price), Error::<T>::NotEnoughBalance);

			Self::get_kitty(kitty_id).map_err(|_| Error::<T>::NotExistKittyId)?;
			ensure!(Self::kitty_owner(kitty_id) == Some(sender.clone()), Error::<T>::NotOwner);

			T::Currency::unreserve(&sender, kitty_price);
			T::Currency::reserve(&new_owner, kitty_price)?;
			KittyOwner::<T>::insert(kitty_id, &new_owner);

			OwnerKitties::<T>::try_mutate(&new_owner, |ref mut kitties| {
				kitties.try_push(kitty_id).map_err(|_| Error::<T>::OwnTooManyKitties)?;
				Ok::<(), DispatchError>(())
			})?;

			// 发送一个成功的事件
			Self::deposit_event(Event::KittyTransfer(sender, kitty_id, new_owner));
			Ok({})
		}
	}

	impl<T: Config> Pallet<T> {
		/// get a random 256
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet::<T>>::extrinsic_index(),
				);
			payload.using_encoded(blake2_128)
		}

		/// get next id
		fn get_next_id() -> Result<T::KittyIndex, ()> {
			let kitty_id = Self::next_kitty_id();
			match kitty_id {
				_ if T::KittyIndex::max_value() <= kitty_id => Err(()),
				val => Ok(val),
			}
		}

		/// get kitty by kitty_id
		fn get_kitty(kitty_id: T::KittyIndex) -> Result<Kitty, ()> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty),
				None => Err({}),
			}
		}
	}
}

