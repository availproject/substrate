#![cfg_attr(not(feature = "std"), no_std)]

pub mod config {
	pub const NUM_BLOBS: usize = 128;
	pub const NUM_CHUNKS_IN_BLOB: usize = 256;
	/// in bytes
	pub const CHUNK_SIZE: usize = 64;

	pub const MAX_BLOCK_SIZE: usize = NUM_BLOBS * NUM_CHUNKS_IN_BLOB * CHUNK_SIZE;

	pub const SCALAR_SIZE_WIDE: usize = 64;
	pub const SCALAR_SIZE: usize = 32;
	pub const EXTENSION_FACTOR: usize = 2;
	pub const PROVER_KEY_SIZE: usize = 48;
	pub const PROOF_SIZE: usize = 48;
	pub const MAX_PROOFS_REQUEST: usize = 30;
}

#[cfg(feature = "std")]
pub mod com;
