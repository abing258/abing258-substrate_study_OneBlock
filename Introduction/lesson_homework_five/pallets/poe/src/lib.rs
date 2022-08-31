#![cfg_attr(not(feature = "std"), no_std)]

/// a module forproof of existence
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*
	};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	/// 模块配置接口
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn proofs)]
	pub type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		(T::AccountId, T::BlockNumber)
	>;

	#[pallet::event]
	///#[pallet::matedate(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
		ClaimRevoked(T::AccountId, Vec<u8>),
		ClaimedTransfered(T::AccountId,T::AccountId,Vec<u8>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ClaimNotExist,
		NotClaimOwner,
		NoSuchProof,
		NotProofOwner,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		///创建存证可调用函数  origin表示发送方 claim存证的hash值
		pub fn create_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>
		) -> DispatchResultWithPostInfo {
			///校验是否是一个签名的交易并获取sender
			let sender = ensure_signed(origin)?;
			ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);

			Proofs::<T>::insert(
				&claim,
				(sender.clone(), frame_system::Pallet::<T>::block_number())
			);

			Self::deposit_event(Event::ClaimCreated(sender, claim));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn revoke_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;
			ensure!(owner == sender, Error::<T>::NotClaimOwner);
			Proofs::<T>::remove(&claim);
			Self::deposit_event(Event::ClaimRevoked(sender, claim));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn transfer_claim(origin: OriginFor<T>, claim: Vec<u8>, dest: T::AccountId) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// 检测存证文件是否存在
			ensure!(Proofs::<T>::contains_key(&claim), Error::<T>::NoSuchProof);

			let (owner, _) = Proofs::<T>::get(&claim).expect("All proofs must have an owner!");

			ensure!(owner == sender, Error::<T>::NotProofOwner);

			Proofs::<T>::insert(&claim, (dest.clone(), frame_system::Pallet::<T>::block_number()));
			// 发送事件，声明权证转移
			Self::deposit_event(Event::ClaimedTransfered(sender,dest,claim));

			Ok(().into())
		}
	}
}
