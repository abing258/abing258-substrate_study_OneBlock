#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::weights;
/// a module for proof of existence
pub use pallet::*;
pub use weight::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weight;

#[frame_support::pallet]
pub mod pallet {
	pub use frame_support::pallet_prelude::*;
	pub use frame_system::pallet_prelude::*;
	pub use sp_std::prelude::*;
	use super::WeightInfo;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	/// 模块配置接口
	#[pallet::config]
	pub trait Config: frame_system::Config {

		/// The maximum length of chain that can be added
		#[pallet::constant]
		/// 定义存证的最大长度，超过了会导致链上到状态爆炸
		type MaxClaimLength: Get<u32>;
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		// 把runtime定义的系统的类型设置在当前模块，满足的条件，可以从当前模块转移过去，同时是系统模块的Event类型
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		///设置权重值
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	//定义自己所需的存储项所需的宏
	#[pallet::generate_store(pub(super) trait Store)]
	//定义模块所需的结构体
	pub struct Pallet<T>(_);

	#[pallet::storage]
	//#[pallet::getter(fn proofs)]
	//存储项
	pub type Proofs<T: Config> = StorageMap<
		_,
		// hash算法，用来将存储项存储到底层数据库的时候，对其位置进行计算（密码安全）
		Blake2_128Concat,
		// 新版本使用BoundedVec而不是Vec，BoundedVec长度受限的集合类型
		BoundedVec<u8, T::MaxClaimLength>,
		(T::AccountId, T::BlockNumber)
	>;

	#[pallet::event]
	//generate_deposit 生成了一个帮助方法 deposit_event
	//deposit_event 方便调用生成事件的宏
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		//创建时触发
		ClaimCreated(T::AccountId, Vec<u8>),
		//吊销时触发
		ClaimRevoked(T::AccountId, Vec<u8>),
		ClaimTransferred(T::AccountId, T::AccountId, Vec<u8>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ClaimTooLong,
		ClaimNotExist,
		NotClaimOwner,
	}

	#[pallet::hooks]
	//定义保留函数
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::create_claim(claim.len() as u32))]
		/// 创建存证可调用函数  origin表示发送方 claim存证的hash值
		pub fn create_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>
		) -> DispatchResultWithPostInfo {
			// 校验是否是一个签名的交易并获取sender
			let sender = ensure_signed(origin)?;
			// 校验claim长度
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).map_err(|_| Error::<T>::ClaimTooLong)?;
			// 确保不存在
			ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ProofAlreadyExist);
			// 存储
			Proofs::<T>::insert(
				&bounded_claim,
				(sender.clone(), frame_system::Pallet::<T>::block_number())
			);
			// 发送一个成功的事件
			Self::deposit_event(Event::ClaimCreated(sender, claim));

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::revoke_claim(claim.len() as u32))]
		pub fn revoke_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			// 校验claim长度
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).map_err(|_| Error::<T>::ClaimTooLong)?;
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;
			ensure!(owner == sender, Error::<T>::NotClaimOwner);
			Proofs::<T>::remove(&bounded_claim);
			Self::deposit_event(Event::ClaimRevoked(sender, claim));

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::transfer_claim(claim.len() as u32))]
		pub fn transfer_claim(origin: OriginFor<T>, claim: Vec<u8>, dest: T::AccountId) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			// 校验claim长度
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone()).map_err(|_| Error::<T>::ClaimTooLong)?;

			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExist)?;
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			Proofs::<T>::insert(&bounded_claim, (dest.clone(), frame_system::Pallet::<T>::block_number()));
			// 发送事件，声明权证转移
			Self::deposit_event(Event::ClaimTransferred(sender,dest,claim));

			Ok(().into())
		}
	}
}