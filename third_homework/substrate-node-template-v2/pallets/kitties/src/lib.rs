#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

mod migrations;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use frame_system::offchain::{
	AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, SignedPayload, Signer,
	SigningTypes,
};
use sp_runtime::{
	transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
	RuntimeDebug,
};

use sp_core::crypto::KeyTypeId;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"btc!");
pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};
	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::inherent::Vec;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::{BlockNumberFor, *};

	use frame_support::traits::{Currency, ExistenceRequirement, Randomness, StorageVersion};
	use frame_support::PalletId;
	use sp_io::hashing::blake2_128;

	use serde::{Deserialize, Deserializer};
	use sp_runtime::offchain::storage::StorageValueRef;
	use sp_runtime::offchain::{http, Duration};
	use sp_runtime::traits::AccountIdConversion;

	pub type KittyId = u32;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(
		Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
	)]
	// pub struct Kitty(pub [u8; 16]);
	pub struct Kitty {
		pub dna: [u8; 16],
		pub name: [u8; 8],
	}

	#[derive(Deserialize, Encode, Decode, Clone, PartialEq, Eq, scale_info::TypeInfo)]
	struct BtcInfo {
		mins: u8,
		#[serde(deserialize_with = "de_string_to_bytes")]
		price: Vec<u8>,
	}

	pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s: &str = Deserialize::deserialize(de)?;
		Ok(s.as_bytes().to_vec())
	}

	use core::{convert::TryInto, fmt};
	impl fmt::Debug for BtcInfo {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write!(
				f,
				"{{ mins: {}, price: {} }}",
				&self.mins,
				sp_std::str::from_utf8(&self.price).map_err(|_| fmt::Error)?,
			)
		}
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct Payload<Public> {
		price: Vec<u8>,
		public: Public,
	}

	impl<T: SigningTypes> SignedPayload<T> for Payload<T::Public> {
		fn public(&self) -> T::Public {
			self.public.clone()
		}
	}

	#[derive(Debug, Deserialize, Encode, Decode, Default)]
	struct IndexingData(Vec<u8>, [u8; 8]);

	const ONCHAIN_TX_KEY: &[u8] = b"ocw-kitties::storage::tx";

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type Currency: Currency<Self::AccountId>;
		#[pallet::constant]
		type KittyPrice: Get<BalanceOf<Self>>;
		type PalletId: Get<PalletId>;
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type NextKittyId<T: Config> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_on_sale)]
	pub type KittyOnSale<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, ()>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_parents)]
	pub type KittyParents<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId)>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		KittyCreated {
			who: T::AccountId,
			kitty_id: KittyId,
			kitty: Kitty,
		},
		KittyBred {
			who: T::AccountId,
			kitty_id: KittyId,
			kitty: Kitty,
		},
		KittyTransferred {
			who: T::AccountId,
			recipient: T::AccountId,
			kitty_id: KittyId,
		},
		KittyOnSale {
			who: T::AccountId,
			kitty_id: KittyId,
		},
		KittyBought {
			who: T::AccountId,
			kitty_id: KittyId,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		SameKittyId,
		NotOwner,
		AlreadyOnSale,
		NoOwner,
		AlreadyOwned,
		NotOnSale,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_runtime_upgrade() -> Weight {
			migrations::v2::migrate::<T>()
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			let key = Self::derived_key(block_number);
			let oci_mem = StorageValueRef::persistent(&key);

			if let Ok(Some(data)) = oci_mem.get::<IndexingData>() {
				log::info!("OCW ==> off-chain key {:?}", key);
				log::info!("OCW ==> off-chain indexing data: {:?}, {:?}", data.0, data.1);
			}

			match Self::fetch_btc_info() {
				Ok(info) => {
					log::info!("OCW ==> BTC Info: {:?}", info);

					// Retrieve the signer to sign the payload
					let signer = Signer::<T, T::AuthorityId>::any_account();
					if let Some((_, res)) = signer.send_unsigned_transaction(
						// this line is to prepare and return payload
						|acct| Payload { price: info.price.clone(), public: acct.public.clone() },
						|payload, signature| Call::unsigned_extrinsic_with_signed_payload {
							payload,
							signature,
						},
					) {
						match res {
							Ok(()) => {
								log::info!(
									"OCW ==> unsigned tx with signed payload successfully sent."
								);
							},
							Err(()) => {
								log::error!(
									"OCW ==> sending unsigned tx with signed payload failed."
								);
							},
						};
					} else {
						// The case of `None`: no account is available for sending
						log::error!("OCW ==> No local account available");
					}
				},
				Err(e) => {
					log::info!("OCW ==> Error while fetch BTC info! {:?}", e);
				},
			}
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn create(origin: OriginFor<T>, name: [u8; 8]) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			let kitty_id = Self::get_next_id()?;
			let kitty = Kitty { dna: Self::random_value(&who), name };

			let price = T::KittyPrice::get();
			T::Currency::transfer(
				&who,
				&Self::get_account_id(),
				price,
				ExistenceRequirement::KeepAlive,
			)?;
			// T::Currency::reserve(&who, price)?;

			Kitties::<T>::insert(kitty_id, kitty);
			KittyOwner::<T>::insert(kitty_id, &who);

			let key = Self::derived_key(frame_system::Pallet::<T>::block_number());
			let data = IndexingData(b"submit_name".to_vec(), name);
			sp_io::offchain_index::set(&key, &data.encode());

			// Emit an event.
			Self::deposit_event(Event::KittyCreated { who, kitty_id, kitty });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: KittyId,
			kitty_id_2: KittyId,
			name: [u8; 8],
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);

			ensure!(Kitties::<T>::contains_key(kitty_id_1), Error::<T>::InvalidKittyId);
			ensure!(Kitties::<T>::contains_key(kitty_id_2), Error::<T>::InvalidKittyId);

			let kitty_id = Self::get_next_id()?;

			let _kitty_1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
			let _kitty_2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

			let _selector = Self::random_value(&who);
			let dna = [0u8; 16];
			// for i in 0..kitty_1.0.len() {
			// 	data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & !selector[i]);
			// }

			let kitty = Kitty { dna, name };

			let price = T::KittyPrice::get();
			T::Currency::transfer(
				&who,
				&Self::get_account_id(),
				price,
				ExistenceRequirement::KeepAlive,
			)?;
			// T::Currency::reserve(&who, price)?;

			Kitties::<T>::insert(kitty_id, kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			KittyParents::<T>::insert(kitty_id, (kitty_id_1, kitty_id_2));

			// Emit an event.
			Self::deposit_event(Event::KittyBred { who, kitty_id, kitty });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			kitty_id: KittyId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Kitties::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);

			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			ensure!(owner == who, Error::<T>::NotOwner);

			KittyOwner::<T>::insert(kitty_id, &recipient);

			// Emit an event.
			Self::deposit_event(Event::KittyTransferred { who, recipient, kitty_id });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn sale(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::kitties(kitty_id).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;

			ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);
			ensure!(Self::kitty_on_sale(kitty_id).is_none(), Error::<T>::AlreadyOnSale);

			KittyOnSale::<T>::insert(kitty_id, ());
			Self::deposit_event(Event::KittyOnSale { who, kitty_id });

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn buy(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::kitties(kitty_id).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;

			let owner =
				Self::kitty_owner(kitty_id).ok_or::<DispatchError>(Error::<T>::NoOwner.into())?;
			ensure!(owner != who, Error::<T>::AlreadyOwned);
			ensure!(Self::kitty_on_sale(kitty_id).is_some(), Error::<T>::NotOnSale);

			let price = T::KittyPrice::get();
			// T::Currency::reserve(&who, price)?;
			// T::Currency::unreserve(&owner, price);
			T::Currency::transfer(&who, &owner, price, ExistenceRequirement::KeepAlive)?;

			KittyOwner::<T>::insert(kitty_id, &who);
			KittyOnSale::<T>::remove(kitty_id);

			Self::deposit_event(Event::KittyBought { who, kitty_id });

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(0)]
		pub fn unsigned_extrinsic_with_signed_payload(
			origin: OriginFor<T>,
			payload: Payload<T::Public>,
			_signature: T::Signature,
		) -> DispatchResult {
			ensure_none(origin)?;

			log::info!("OCW ==> in call unsigned_extrinsic_with_signed_payload: {:?}", payload);
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_next_id() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|next_id| -> Result<KittyId, DispatchError> {
				let current_id = *next_id;
				*next_id = next_id
					.checked_add(1)
					.ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
				Ok(current_id)
			})
		}

		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);

			payload.using_encoded(blake2_128)
		}

		fn get_account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}

	impl<T: Config> Pallet<T> {
		fn derived_key(block_number: T::BlockNumber) -> Vec<u8> {
			block_number.using_encoded(|encoded_bn| {
				ONCHAIN_TX_KEY
					.clone()
					.into_iter()
					.chain(b"/".into_iter())
					.chain(encoded_bn)
					.copied()
					.collect::<Vec<u8>>()
			})
		}
	}

	impl<T: Config> Pallet<T> {
		fn fetch_btc_info() -> Result<BtcInfo, http::Error> {
			// prepare for send request
			let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(8_000));
			let request =
				http::Request::get("https://data.binance.com/api/v3/avgPrice?symbol=BTCUSDT");
			let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;
			let response =
				pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
			if response.code != 200 {
				log::warn!("Unexpected status code: {}", response.code);
				return Err(http::Error::Unknown);
			}

			let body = response.body().collect::<Vec<u8>>();

			let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
				log::warn!("No UTF8 body");
				http::Error::Unknown
			})?;

			// parse the response str
			let btc_info: BtcInfo =
				serde_json::from_str(body_str).map_err(|_| http::Error::Unknown)?;

			Ok(btc_info)
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			const UNSIGNED_TXS_PRIORITY: u64 = 100;
			let valid_tx = |provide| {
				ValidTransaction::with_tag_prefix("my-pallet")
					.priority(UNSIGNED_TXS_PRIORITY) // please define `UNSIGNED_TXS_PRIORITY` before this line
					.and_provides([&provide])
					.longevity(3)
					.propagate(true)
					.build()
			};

			// match call {
			// 	Call::submit_data_unsigned { key: _ } => valid_tx(b"my_unsigned_tx".to_vec()),
			// 	_ => InvalidTransaction::Call.into(),
			// }

			match call {
				Call::unsigned_extrinsic_with_signed_payload { ref payload, ref signature } => {
					if !SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone()) {
						return InvalidTransaction::BadProof.into();
					}
					valid_tx(b"unsigned_extrinsic_with_signed_payload".to_vec())
				},
				_ => InvalidTransaction::Call.into(),
			}
		}
	}
}
