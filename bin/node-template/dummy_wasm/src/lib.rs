#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit="256"]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

#[cfg(not(feature = "std"))]
pub use sp_core::to_substrate_wasm_fn_return_value;

use sp_std::prelude::*;
// use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	ApplyExtrinsicResult, generic, create_runtime_str, impl_opaque_keys, MultiSignature,
	transaction_validity::{TransactionValidity, TransactionSource},
};
use sp_runtime::traits::{
	BlakeTwo256, Block as BlockT, AccountIdLookup, Verify, IdentifyAccount, NumberFor,
};
use sp_api::impl_runtime_apis;
// use sp_consensus_aura::sr25519::AuthorityId as AuraId;
// use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
// use pallet_grandpa::fg_primitives;
use sp_version::RuntimeVersion;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
//
// // A few exports that help ease life for downstream crates.
// #[cfg(any(feature = "std", test))]
// pub use sp_runtime::BuildStorage;
// pub use pallet_timestamp::Call as TimestampCall;
// pub use pallet_balances::Call as BalancesCall;
pub use sp_runtime::{Permill, Perbill};
pub use frame_support::{
	construct_runtime,
	parameter_types,
	StorageValue,
	traits::{KeyOwnerProofSystem, Randomness},
	weights::{
		Weight, IdentityFee,
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
	},
};
// use pallet_transaction_payment::CurrencyAdapter;
//
//
/// An index to a block.
pub type BlockNumber = u32;
//
// /// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;
//
// /// Some way of identifying an account on the chain. We intentionally make it equivalent
// /// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
//
// /// The type for looking up accounts. We don't expect more than 4 billion of them, but you
// /// never know...
pub type AccountIndex = u32;
//
// /// Balance of an account.
pub type Balance = u128;
//
// /// Index of a transaction in the chain.
pub type Index = u32;
//
// /// A hash of some data used by the chain.
pub type Hash = sp_core::H256;
//
// /// Digest item type.
// pub type DigestItem = generic::DigestItem<Hash>;
//
// /// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
// /// the specifics of the runtime. They can then be made to be agnostic over specific formats
// /// of data like extrinsics, allowing for them to continue syncing the network through upgrades
// /// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	//
	// impl_opaque_keys! {
	// 	pub struct SessionKeys {
	// 		pub aura: Aura,
	// 		pub grandpa: Grandpa,
	// 	}
	// }
}

pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("node-template"),
	impl_name: create_runtime_str!("node-template"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
};

/// This determines the average expected block time that we are targetting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
// pub const MILLISECS_PER_BLOCK: u64 = 6000;
//
// pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
//
// // Time is measured by number of blocks.
// pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
// pub const HOURS: BlockNumber = MINUTES * 60;
// pub const DAYS: BlockNumber = HOURS * 24;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(100);
//
parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const BlockHashCount: BlockNumber = 2400;
	// We allow for 2 seconds of compute with a 6 second average block time.
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
		::with_sensible_defaults(2 * WEIGHT_PER_SECOND, NORMAL_DISPATCH_RATIO);
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(kate::config::MAX_BLOCK_SIZE as u32, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = ();
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = BlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
}
//
// impl pallet_aura::Config for Runtime {
// 	type AuthorityId = AuraId;
// }
//
// impl pallet_grandpa::Config for Runtime {
// 	type Event = Event;
// 	type Call = Call;
//
// 	type KeyOwnerProofSystem = ();
//
// 	type KeyOwnerProof =
// 	<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
//
// 	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
// 		KeyTypeId,
// 		GrandpaId,
// 	)>>::IdentificationTuple;
//
// 	type HandleEquivocation = ();
//
// 	type WeightInfo = ();
// }
//
// parameter_types! {
// 	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
// }
//
// impl pallet_timestamp::Config for Runtime {
// 	/// A timestamp: milliseconds since the unix epoch.
// 	type Moment = u64;
// 	type OnTimestampSet = Aura;
// 	type MinimumPeriod = MinimumPeriod;
// 	type WeightInfo = ();
// }
//
// parameter_types! {
// 	pub const ExistentialDeposit: u128 = 500;
// 	pub const MaxLocks: u32 = 50;
// }
//
// impl pallet_balances::Config for Runtime {
// 	type MaxLocks = MaxLocks;
// 	/// The type for recording an account's balance.
// 	type Balance = Balance;
// 	/// The ubiquitous event type.
// 	type Event = Event;
// 	type DustRemoval = ();
// 	type ExistentialDeposit = ExistentialDeposit;
// 	type AccountStore = System;
// 	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
// }
//
// parameter_types! {
// 	pub const TransactionByteFee: Balance = 1;
// }
//
// impl pallet_transaction_payment::Config for Runtime {
// 	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
// 	type TransactionByteFee = TransactionByteFee;
// 	type WeightToFee = IdentityFee<Balance>;
// 	type FeeMultiplierUpdate = ();
// }
//
// impl pallet_sudo::Config for Runtime {
// 	type Event = Event;
// 	type Call = Call;
// }

/// Configure the pallet template in pallets/template.
// impl template::Config for Runtime {
// 	type Event = Event;
// }

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		// RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
		// Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
		// Aura: pallet_aura::{Module, Config<T>, Inherent},
		// Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event},
		// Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		// TransactionPayment: pallet_transaction_payment::{Module, Storage},
		// Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},
		// // Include the custom logic from the template pallet in the runtime.
		// TemplateModule: template::{Module, Call, Storage, Event<T>},
	}
);

/// Block type as expected by this runtime.

pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
// pub type SignedExtra = (
// 	frame_system::CheckSpecVersion<Runtime>,
// );
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		// fn execute_block(block: Block) {
		//
		// }
		//
		// fn initialize_block(header: &<Block as BlockT>::Header) {
		//
		// }
	}
}
