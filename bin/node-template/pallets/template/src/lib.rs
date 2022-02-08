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
	traits::{ Get },
	ensure,
	StorageMap,
	weights::{DispatchClass, Pays, Weight},
};
use codec::{Encode};
use frame_system::{ ensure_signed, limits::BlockLength };
use sp_std::vec::Vec;
use sp_core::{storage::well_known_keys, Bytes};
use sp_runtime::Perbill;
use libm::ceil;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

pub const BLOCK_CHUNK_SIZE: u32 = 32;
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(90);

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	trait Store for Module<T: Config> as DataAvailability {
		BlockLengthProposalID: u32;
		BlockLengthProposal: BlockLength;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
		/// Event is emitted when a data has been submitted. [who, app id]
        DataSubmitted(AccountId, u32),
        /// Event is emitted when block length proposal has been submitted. [who, rows, cols]
        BlockLengthProposalSubmitted(AccountId, u32, u32),
        /// Event is emitted when application id is created by providing string key [who, app key, app id]
        ApplicationKeyCreated(AccountId, Vec<u8>, u32),
        /// Event is emitted when application data submitter is added to the list of allowed [who, submitter]
        DataSubmitterAttached(AccountId, AccountId),
        /// Event is emitted when application data [who, submitter]
        DataSubmitterRemoved(AccountId, AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		/// Application key already exist.
		ApplicationKeyExists,
		/// The queried key does not exist
		KeyDoesNotExist,
		/// Block normal ratio is greater than 100%
		RatioOutOfBounds,
		RatioTooSmall,
		BlockDimensionsOutOfBounds,
		BlockDimensionsTooSmall,
		ChunkSizeOutOfBounds,
		ChunkSizeToSmall,
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
		/// Weight = ceil(data_size / 200000)*db_write_cost + 1 db_read_cost
		/// db_write_cost is calculated @ 200,000 items
		#[weight = (
			T::DbWeight::get().reads_writes(
				1,
				ceil(data.len() as f64 / 200_000 as f64) as u64
			) as Weight,
			DispatchClass::Normal,
			Pays::Yes
		)]
        fn submit_data(origin, data: Vec<u8>) {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // Verify that the given data has not already been submitted.
            // ensure!(!KeyToValue::<T>::contains_key((&sender, &key)), Error::<T>::KeyAlreadyExists);

            // Store the data with the sender and block number.
            // KeyToValue::<T>::insert((&sender, &key), &value);

            // Emit an event that the claim was created.
            // Self::deposit_event(RawEvent::DataSubmitted(sender, key));
		}

		// #[weight = 10_000]
		// fn vote_block_length_proposal(origin, proposal_id: u32) {
		// 	let sender = ensure_signed(origin)?;
		// }
		#[weight = 70_000_000]
		fn create_application_key(origin, key: Vec<u8>) {
			let sender = ensure_signed(origin)?;

			ensure!(<frame_system::Module<T>>::get_application_id(&key) == 0, Error::<T>::ApplicationKeyExists);
			let key_id = <frame_system::Module<T>>::create_application_key(&key);

			Self::deposit_event(RawEvent::ApplicationKeyCreated(sender, key, key_id));
		}

		// #[weight = 70_000_000]
		// fn attach_submitter(origin, submitter: AccountId, key: Vec<u8>) {
		// 	let sender = ensure_signed(origin)?;
		//
		// 	Self::deposit_event(RawEvent::ApplicationKeyCreated(sender, key, key_id));
		// }
		//
		// #[weight = 70_000_000]
		// fn remove_submitter(origin, submitter: AccountId, key: Vec<u8>) {
		// 	let sender = ensure_signed(origin)?;
		//
		// 	Self::deposit_event(RawEvent::ApplicationKeyCreated(sender, key, key_id));
		// }

		#[weight = 10_000]
		fn submit_block_length_proposal(origin, rows: u32, cols: u32) {
			let sender = ensure_signed(origin)?;

			ensure!(rows <= 1024, Error::<T>::BlockDimensionsOutOfBounds);
			ensure!(rows >= 32, Error::<T>::BlockDimensionsTooSmall);

			ensure!(cols <= 256, Error::<T>::BlockDimensionsOutOfBounds);
			ensure!(cols >= 32, Error::<T>::BlockDimensionsTooSmall);

			let block_length = BlockLength::with_normal_ratio(rows, cols, BLOCK_CHUNK_SIZE, NORMAL_DISPATCH_RATIO);
			sp_io::storage::set(well_known_keys::BLOCK_LENGTH, &block_length.encode());

			let proposalId = BlockLengthProposalID::get() + 1;
			BlockLengthProposalID::put(proposalId);

			Self::deposit_event(RawEvent::BlockLengthProposalSubmitted(sender, rows, cols));
		}
	}
}
