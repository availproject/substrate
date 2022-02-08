#![cfg_attr(not(feature = "std"), no_std)]

pub mod config {
	pub const SCALAR_SIZE_WIDE: usize = 64;
	pub const SCALAR_SIZE: usize = 32;
	pub const CHUNK_SIZE: usize = 31;
	pub const EXTENSION_FACTOR: usize = 2;
	pub const PROVER_KEY_SIZE: usize = 48;
	pub const PROOF_SIZE: usize = 48;
	pub const MAX_PROOFS_REQUEST: usize = 30;
	pub const MINIMUM_BLOCK_SIZE: usize = 256;
        pub const MAX_BLOCK_ROWS: u32  = if cfg!(feature = "extended-columns") { 128 } else { 256 };
        pub const MAX_BLOCK_COLUMNS: u32  = if cfg!(feature = "extended-columns") { 512 } else { 256 };
}

#[cfg(feature = "std")]
pub mod com;
