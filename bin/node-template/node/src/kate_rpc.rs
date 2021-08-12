use std::sync::Arc;
use jsonrpc_core::{Result, Error as RpcError, ErrorCode};
use jsonrpc_derive::rpc;
use lru::LruCache;
use sp_blockchain::HeaderBackend;
use sc_client_api::BlockBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::{generic::BlockId, traits::{Block as BlockT}};
use sp_runtime::traits::{NumberFor, Header, ApplicationId};
use jsonrpc_core::futures::{
	Sink, Future,
	future::result,
};
use futures::{StreamExt as _, compat::Compat};
use futures::future::{ready, FutureExt, TryFutureExt};
use sp_rpc::number::NumberOrHex;
use std::sync::RwLock;
use codec::{Encode, Decode};
use frame_system::limits::BlockLength;
use sp_core::{Bytes, storage::well_known_keys};
use kate_rpc_runtime_api::KateParamsGetter;
use frame_benchmarking::frame_support::weights::DispatchClass;
use kate::com::{BlockDimensions, XtsLayout};
use jsonrpc_pubsub::{typed::Subscriber, SubscriptionId, manager::SubscriptionManager};
use sp_transaction_pool::{
	TransactionPool, InPoolTransaction, TransactionStatus, TransactionSource,
	BlockHash, TxHash, TransactionFor, error::IntoPoolError,
};
use log::warn;
use sc_rpc_api::author::{ error::Error as RpcApiError, error::Result as RpcApiResult };

#[rpc]
pub trait KateApi<Hash, BlockHash> {
	type Metadata;
	type Block;

	#[rpc(name = "kate_queryProof")]
	fn query_proof(
		&self,
		block_number: NumberOrHex,
		cells: Vec<kate::com::Cell>,
	) -> Result<Vec<u8>>;

	#[rpc(name = "kate_blockLength")]
	fn query_block_length(
		&self,
	) -> Result<BlockLength>;
}

pub struct Kate<TxPool, Client, BlockHash> {
	client: Arc<Client>,
	block_ext_cache: RwLock<LruCache<BlockHash, (Vec<dusk_plonk::prelude::BlsScalar>, BlockDimensions)>>,
	tx_pool: Arc<TxPool>,
	subscriptions: SubscriptionManager,
}

impl<TxPool, Client, BlockHash> Kate<TxPool, Client, BlockHash>
	where BlockHash: Eq + std::hash::Hash
{
	pub fn new(
		client: Arc<Client>,
		tx_pool: Arc<TxPool>,
		subscriptions: SubscriptionManager,
	) -> Self {
		Self {
			client,
			subscriptions,
			tx_pool,
			block_ext_cache: RwLock::new(LruCache::new(2048)) // 524288 bytes per block, ~1Gb max size
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

const TX_SOURCE: TransactionSource = TransactionSource::External;

impl<TxPool, Client> KateApi<
	TxHash<TxPool>, BlockHash<TxPool>
> for Kate<TxPool, Client, BlockHash<TxPool>> where
	TxPool: TransactionPool + Sync + Send + 'static,
	<TxPool::Block as BlockT>::Extrinsic: ApplicationId,
	Client: Send + Sync + 'static,
	Client: HeaderBackend<TxPool::Block> + ProvideRuntimeApi<TxPool::Block> + BlockBackend<TxPool::Block>,
	Client::Api: KateParamsGetter<TxPool::Block>,
{
	type Metadata = sc_rpc_api::Metadata;
	type Block = TxPool::Block;

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

		let block_num = <NumberFor<Self::Block>>::from(block_num);
		let block = self.client.block(&BlockId::number(block_num)).unwrap();
		let mut block_ext_cache = self.block_ext_cache.write().unwrap();

		if !block.is_none() {
			let best_hash = BlockId::hash(self.client.info().best_hash);
			let block_length: BlockLength = self.client.runtime_api().get_block_length(&best_hash).map_err(|e| RpcError {
				code: ErrorCode::ServerError(9876),
				message: "Something wrong".into(),
				data: Some(format!("{:?}", e).into()),
			}).unwrap();

			let block = block.unwrap();
			let block_hash = block.block.header().hash();
			if !block_ext_cache.contains(&block_hash) {
				// build block data extension and cache it
				let xts_by_id: Vec<(u32, Vec<u8>)> = block.block.extrinsics().into_iter().map(|e|{
					(e.app_id(), e.encode())
				}).collect();

				let (_, block, block_dims) = kate::com::flatten_and_pad_block(
					block_length.rows as usize,
					block_length.cols as usize,
					block_length.chunk_size as usize,
					&xts_by_id,
					block.block.header().parent_hash().as_ref()
				);

				let data = kate::com::extend_data_matrix(
					block_dims,
					&block
				);
				block_ext_cache.put(block_hash, (data, block_dims));
			}

			let (ext_data, block_dims) = block_ext_cache.get(&block_hash).unwrap();
			let kc_public_params = self.client.runtime_api().get_public_params(&best_hash).map_err(|e| RpcError {
				code: ErrorCode::ServerError(9876),
				message: "Something wrong".into(),
				data: Some(format!("{:?}", e).into()),
			}).unwrap();

			let proof = kate::com::build_proof(
				&kc_public_params,
				*block_dims,
				&ext_data,
				cells
			);

			return Ok(proof.unwrap());
		}

		Err(RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "".into(),
			data: None
		})
	}

	fn query_block_length(&self) -> Result<BlockLength> {
		Ok(self.client.runtime_api().get_block_length(&BlockId::hash(self.client.info().best_hash)).map_err(|e| RpcError {
			code: ErrorCode::ServerError(9877),
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		}).unwrap())
	}
}
