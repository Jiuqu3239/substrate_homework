#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;
pub use pallet::*;
pub use weights::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::inherent::Vec;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		#[pallet::constant]
		type MaxClaimLength: Get<u32>;
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxClaimLength>,
		(T::AccountId, T::BlockNumber),
	>;

	// Pallets use events to inform users when important changes are made.
	// Event documentation should end with an array that provides descriptive names for parameters.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when a claim has been created.
		ClaimCreated(T::AccountId, Vec<u8>),
		/// Event emitted when a claim is revoked by the owner.
		ClaimRevoked(T::AccountId, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The claim already exists.
		ProofAlreadyExist,
		/// The claim data too long.
		ClaimTooLong,
		/// The claim does not exist, so it cannot be revoked.
		ClaimNotExist,
		/// The claim is owned by another account, so caller can't revoke it.
		NotClaimOwner,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allow users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::base())]
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let sender = ensure_signed(origin)?;

			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			// Verify that the specified claim has not already been stored.
			ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ProofAlreadyExist);

			// Get the block number from the FRAME System pallet.
			let current_block = <frame_system::Pallet<T>>::block_number();

			// Store the claim with the sender and block number.
			Proofs::<T>::insert(&bounded_claim, (sender.clone(), current_block));

			// Emit an event that the claim was created.
			Self::deposit_event(Event::ClaimCreated(sender, claim));

			Ok(().into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::base())]
		pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let sender = ensure_signed(origin)?;

			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			// Get owner of the claim, if none return an error.
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::NotClaimOwner)?;

			// Verify that sender of the current call is the claim owner.
			ensure!(sender == owner, Error::<T>::NotClaimOwner);

			// Remove claim from storage.
			Proofs::<T>::remove(&bounded_claim);

			// Emit an event that the claim was erased.
			Self::deposit_event(Event::ClaimRevoked(sender, claim));
			Ok(().into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::base())]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>,
			dest: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;

			// Get owner of the claim, if none return an error.
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::NotClaimOwner)?;

			// Verify that sender of the current call is the claim owner.
			ensure!(sender == owner, Error::<T>::NotClaimOwner);

			// Get the block number from the FRAME System pallet.
			let current_block = <frame_system::Pallet<T>>::block_number();

			// Store the claim with the sender and block number.
			Proofs::<T>::insert(&bounded_claim, (dest, current_block));

			Ok(().into())
		}
	}
}