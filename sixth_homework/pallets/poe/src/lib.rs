#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;


#[frame_support::pallet]
pub mod pallet {
	use super::*;
///	pub use weights::WeightInfo;
	pub use frame_support::inherent::Vec;
	pub use frame_support::pallet_prelude::*;
	pub use frame_system::{pallet_prelude::*};
	pub use sp_std::prelude::*;
	use super::WeightInfo;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The maximum length of claim that can be added.
		#[pallet::constant] // Declaring variable type as constant.
		type  MaxClaimLength: Get<u32>; // Currently only accepting u32 type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;

	}


	#[pallet::pallet]
	pub struct Pallet<T>(_);

	//存储项
	#[pallet::storage]
	#[pallet::getter(fn Proofs)]
	pub type Proofs<T: Config> = StorageMap<
	_,
	Blake2_128Concat,
	BoundedVec<u8, T::MaxClaimLength>, // 不能直接使用vec 类型，需要使用长度受限的vec 类型
	(T::AccountId, T::BlockNumber),
	>;


	/// Defining an event.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
	//	ClaimCreated(T::AccountId, Vec<u8>), 
	//	ClaimRevoked(T::AccountId, Vec<u8>), 
	ClaimCreated(T::AccountId, BoundedVec<u8, T::MaxClaimLength>), 
	ClaimRevoked(T::AccountId, BoundedVec<u8, T::MaxClaimLength>),
	ClaimTransferred {
		sender: T::AccountId,
		recipient: T::AccountId,
		claim: BoundedVec<u8, T::MaxClaimLength>,
	},
	}

	/// Error message.
	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExist,
		ClaimTooLong,
		ClaimNotExist,
		NotClaimOwner,
	    TransferToOwner,
	}

	/// 保留函数
	#[pallet::hooks]
	impl <T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// 可调用函数
	#[pallet::call]
	impl <T: Config> Pallet<T> {
		///#[pallet::weight(0)] // Specifying weight 0. claim created.
		#[pallet::weight(T::WeightInfo::create_claim(claim.len() as u32))]
		pub fn create_claim(origin: OriginFor<T>, claim: BoundedVec<u8, T::MaxClaimLength>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);

			Proofs::<T>::insert(
				&claim,
				(sender.clone(), frame_system::Pallet::<T>::block_number()),
			);

			Self::deposit_event(Event::ClaimCreated(sender, claim));

			Ok(().into())
		}

		//claim revoke
		///#[pallet::weight(0)]
		#[pallet::weight(T::WeightInfo::revoke_claim(claim.len() as u32))]
		pub fn revoke_claim(origin: OriginFor<T>, claim: BoundedVec<u8, T::MaxClaimLength>) ->DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;
			ensure!(owner == sender, Error::<T>::NotClaimOwner);

			Proofs::<T>::remove(&claim);

			Self::deposit_event(Event::ClaimRevoked(sender, claim));

			Ok(().into())
		}

		///#[pallet::weight(0)]
		#[pallet::weight(T::WeightInfo::transfer_claim(claim.len() as u32))]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			claim: BoundedVec<u8, T::MaxClaimLength>
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
// Did it exceed the maximum length

			let (owner, _block_number) =
				Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;
			ensure!(signer == owner, Error::<T>::NotClaimOwner);
			ensure!(signer != recipient, Error::<T>::TransferToOwner);

			Proofs::<T>::insert(
				&claim,
				(recipient.clone(), frame_system::Pallet::<T>::block_number()),
			);
			Self::deposit_event(Event::ClaimTransferred { sender: signer, recipient, claim  });
			Ok(())
		}
	}

	
}