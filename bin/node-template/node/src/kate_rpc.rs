use std::sync::Arc;
use jsonrpc_core::{Result, Error as RpcError, ErrorCode};
use jsonrpc_derive::rpc;
use lru::LruCache;
use sp_blockchain::HeaderBackend;
use sc_client_api::BlockBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::{generic::BlockId, traits::{Block as BlockT}};
use sp_runtime::traits::{NumberFor, Header};
use sp_rpc::number::NumberOrHex;
use std::sync::RwLock;
use codec::{Encode};
use kate_rpc_runtime_api::KateParamsGetter;

#[rpc]
pub trait KateApi {
	#[rpc(name = "kate_queryProof")]
	fn query_proof(
		&self,
		block_number: NumberOrHex,
		cells: Vec<kate::com::Cell>,
	) -> Result<Vec<u8>>;
}

pub struct Kate<Client, Block: BlockT> {
	client: Arc<Client>,
	cache: RwLock<LruCache<Block::Hash, Vec<u8>>>,
}

impl<Client, Block> Kate<Client, Block> where Block: BlockT {
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

impl<Client, Block> KateApi for Kate<Client, Block> where
	Block: BlockT,
	Client: Send + Sync + 'static,
	Client: HeaderBackend<Block> + ProvideRuntimeApi<Block> + BlockBackend<Block>,
	Client::Api: KateParamsGetter<Block>,
{
	//TODO allocate static thread pool, just for RPC related work, to free up resources, for the block producing processes.
	fn query_proof(
		&self,
		block_number: NumberOrHex,
		cells: Vec<kate::com::Cell>,
	) -> Result<Vec<u8>> {
		use std::convert::TryInto;
		let block_num: u32 = block_number.try_into().map_err(|_| RpcError {
			code: ErrorCode::ServerError(Error::DecodeError.into()),
			message: format!(
				"`{:?}` > u32::max_value(), the max block number is u32.",
				block_number
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

			let kc_public_params = self.client.runtime_api().get_public_params(&BlockId::hash(self.client.info().best_hash)).map_err(|e| RpcError {
				code: ErrorCode::ServerError(9876),
				message: "Something wrong".into(),
				data: Some(format!("{:?}", e).into()),
			}).unwrap();
			log::info!(
				target: "system",
				"RPC block.header.hash {:#?}",
				block.block.header().parent_hash()
			);
			let proof = kate::com::build_proof(&kc_public_params, &data, cells, block.block.header().parent_hash().as_ref());

			return Ok(proof.unwrap());
		}

		Err(RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "".into(),
			data: None
		})
	}
}
