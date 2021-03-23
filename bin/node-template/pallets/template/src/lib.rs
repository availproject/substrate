#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{
	decl_module,
	decl_storage,
	decl_event,
	decl_error,
	dispatch,
	traits::Get,
	ensure,
	StorageMap,
};

use frame_system::ensure_signed;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	trait Store for Module<T: Config> as DataAvailability {
		HashToBytes: map hasher(blake2_128_concat) (T::AccountId, Vec<u8>) => Vec<u8>;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
		/// Event emitted when a data has been submitted. [who, key, value]
        DataSubmitted(AccountId, Vec<u8>, Vec<u8>),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		/// The data has already been submitted.
		KeyAlreadyExists,
		/// The queried key does not exist
		KeyDoesNotExist,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// Allow a user to submit new data. 
        #[weight = 10_000] 		// TODO: Make weight = f(data size)
        fn submit_data(origin, key: Vec<u8>, value: Vec<u8>) {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // Verify that the given data has not already been submitted.
            ensure!(!HashToBytes::<T>::contains_key((&sender, &key)), Error::<T>::KeyAlreadyExists);

            // Store the data with the sender and block number.
            HashToBytes::<T>::insert((&sender, &key), &value);

            // Emit an event that the claim was created.
            Self::deposit_event(RawEvent::DataSubmitted(sender, key, value));
		}
	}
}
