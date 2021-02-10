use std::sync::Arc;
use jsonrpc_core::{Result, Error as RpcError, ErrorCode};
use jsonrpc_derive::rpc;
use lru::LruCache;

use sp_core::storage::well_known_keys;
use sc_client_api::BlockBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::{generic::BlockId, traits::{Block as BlockT}};
use sp_runtime::traits::{NumberFor, Header};
use sp_rpc::number::NumberOrHex;
use std::sync::RwLock;
use codec::{Encode};

#[rpc]
pub trait KateRpcApi<C, B> {
	#[rpc(name = "kate_queryProof")]
	fn query_proof(
		&self,
		num_or_hex: NumberOrHex,
		cells: Vec<kate::com::Cell>,
	) -> Result<Vec<u8>>;
}

pub struct KateRpc<Client, Block: BlockT> {
	client: Arc<Client>,
	cache: RwLock<LruCache<Block::Hash, Vec<u8>>>,
}

impl<Client, Block> KateRpc<Client, Block> where Block: BlockT {
	pub fn new(client: Arc<Client>) -> Self {
		Self {
			client,
			cache: RwLock::new(LruCache::new(333)) // 3145728 bytes per proof, ~1Gb max size
		}
	}
}

/// Error type of this RPC api.
pub enum Error {
	/// The transaction was not decodable.
	DecodeError,
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

impl<Client, Block> KateRpcApi<Client, Block> for KateRpc<Client, Block> where
	Block: BlockT,
	Client: Send + Sync + 'static + ProvideRuntimeApi<Block> + BlockBackend<Block>,
{
	fn query_proof(
		&self,
		num_or_hex: NumberOrHex,
		cells: Vec<kate::com::Cell>,
	) -> Result<Vec<u8>> {
		use std::convert::TryInto;
		let block_num: u32 = num_or_hex.try_into().map_err(|_| RpcError {
			code: ErrorCode::ServerError(Error::DecodeError.into()),
			message: format!(
				"`{:?}` > u32::max_value(), the max block number is u32.",
				num_or_hex
			).into(),
			data: None,
		})?;
		let block_num = <NumberFor<Block>>::from(block_num);

		let block = self.client.block(&BlockId::number(block_num)).unwrap();
		// let mut cache = self.cache.write().unwrap();

		if !block.is_none() {
			let block = block.unwrap();
			// let block_hash = block.block.header().hash();
			// if !cache.contains(&block_hash) {
			// 	// let serializer = Serializer::
			// 	// build proof and cache it
			// 	let data: Vec<Vec<u8>> = block.block.extrinsics().into_iter().map(|e|{
			// 		e.encode()
			// 	}).collect();
			// 	let kc_public_params: Vec<u8> = sp_io::storage::get(well_known_keys::KATE_PUBLIC_PARAMS)
			// 		.unwrap_or_default();
			// 	let proof = kate::com::build_proof(&kc_public_params, &data, cells);
			// 	cache.put(block_hash, proof);
			// }

			let data: Vec<Vec<u8>> = block.block.extrinsics().into_iter().map(|e|{
				e.encode()
			}).collect();
			let kc_public_params: Vec<u8> = sp_io::storage::get(well_known_keys::KATE_PUBLIC_PARAMS)
				.unwrap_or_default();
			let proof = kate::com::build_proof(&kc_public_params, &data, cells);

			return Ok(proof.unwrap());
		}

		Err(RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "".into(),
			data: None
		})
	}
}
