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
use sp_core::storage::well_known_keys;
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

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	trait Store for Module<T: Config> as DataAvailability {
		KeyToValue: map hasher(blake2_128_concat) (T::AccountId, Vec<u8>) => Vec<u8>;
		BlockLengthProposalID: u32;
		BlockLengthProposal: BlockLength;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
		/// Event emitted when a data has been submitted. [who, key, value]
        DataSubmitted(AccountId, Vec<u8>, Vec<u8>),
        /// Event emitted when block length proposal has been submitted. [who, proposal_id, rows, cols, chunk_size, ratio_percent]
        BlockLengthProposalSubmitted(AccountId, u32, u32, u32, u32, u32),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		/// The data has already been submitted.
		KeyAlreadyExists,
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
				ceil(value.len() as f64 / 200_000 as f64) as u64
			) as Weight,
			DispatchClass::Normal,
			Pays::Yes
		)]
        fn submit_data(origin, key: Vec<u8>, value: Vec<u8>) {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // Verify that the given data has not already been submitted.
            ensure!(!KeyToValue::<T>::contains_key((&sender, &key)), Error::<T>::KeyAlreadyExists);

            // Store the data with the sender and block number.
            KeyToValue::<T>::insert((&sender, &key), &value);

            // Emit an event that the claim was created.
            Self::deposit_event(RawEvent::DataSubmitted(sender, key, value));
		}

		// #[weight = 10_000]
		// fn vote_block_length_proposal(origin, proposal_id: u32) {
		// 	let sender = ensure_signed(origin)?;
		// }

		#[weight = 10_000]
		fn submit_block_length_proposal(origin, rows: u32, cols: u32, chunk_size: u32, ratio_percent: u32)  {
			let sender = ensure_signed(origin)?;

			ensure!(rows <= 1024, Error::<T>::BlockDimensionsOutOfBounds);
			ensure!(rows >= 32, Error::<T>::BlockDimensionsTooSmall);

			ensure!(cols <= 256, Error::<T>::BlockDimensionsOutOfBounds);
			ensure!(cols >= 32, Error::<T>::BlockDimensionsTooSmall);

			ensure!(chunk_size <= 256, Error::<T>::ChunkSizeOutOfBounds);
			ensure!(chunk_size >= 32, Error::<T>::ChunkSizeToSmall);

			ensure!(ratio_percent >= 50, Error::<T>::RatioTooSmall);
			ensure!(ratio_percent <= 100, Error::<T>::RatioOutOfBounds);

			let block_length = BlockLength::with_normal_ratio(rows, cols, chunk_size, Perbill::from_percent(ratio_percent));
			sp_io::storage::set(well_known_keys::BLOCK_LENGTH, &block_length.encode());

			// let proposalId = BlockLengthProposalID::get() + 1;
			// BlockLengthProposalID::put(proposalId);
			// BlockLengthProposal::put(BlockLength::with_normal_ratio(rows, cols, chunk_size, Perbill::from_percent(ratio_percent)));

			// Self::deposit_event(RawEvent::BlockLengthProposalSubmitted(sender, proposalId, rows, cols, chunk_size, ratio_percent));
		}
	}
}
