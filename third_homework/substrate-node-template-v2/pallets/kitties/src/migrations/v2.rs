use frame_support::pallet_prelude::*;
use frame_support::storage::StoragePrefixedMap;
use frame_support::traits::GetStorageVersion;
use frame_support::weights::Weight;

use frame_support::migration::storage_key_iter;
use frame_support::Blake2_128Concat;

use crate::{Config, Kitties, Kitty, KittyId, Pallet};

#[derive(
	Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
)]
pub struct OldKitty(pub [u8; 16]);

#[derive(
	Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
)]
pub struct V1Kitty {
	pub dna: [u8; 16],
	pub name: [u8; 4],
}

pub fn migrate<T: Config>() -> Weight {
	let on_chain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::current_storage_version();

	if current_version > 1 {
		return Weight::zero();
	}

	if on_chain_version == 0 {
		return v0_to_v2::<T>();
	}

	if on_chain_version == 1 {
		return v1_to_v2::<T>();
	}

	Weight::zero()
}

pub fn v0_to_v2<T: Config>() -> Weight {
	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();

	for (index, kitty) in
		storage_key_iter::<KittyId, OldKitty, Blake2_128Concat>(module, item).drain()
	{
		let new_kitty = Kitty { dna: kitty.0, name: *b"abcd0000" };
		Kitties::<T>::insert(index, new_kitty);
	}

	Weight::zero()
}

pub fn v1_to_v2<T: Config>() -> Weight {
	let module = Kitties::<T>::module_prefix();
	let item = Kitties::<T>::storage_prefix();

	for (index, kitty) in
		storage_key_iter::<KittyId, V1Kitty, Blake2_128Concat>(module, item).drain()
	{
		let name: [u8; 8] = [kitty.name, kitty.name].concat().try_into().unwrap();

		let new_kitty = Kitty { dna: kitty.dna, name };
		Kitties::<T>::insert(index, new_kitty);
	}

	Weight::zero()
}
