#[cfg(feature = "std")]
use std::fmt;

#[cfg(feature = "std")]
use serde::{ Serialize, Deserialize };
use crate::codec::{Codec, Encode, Decode, EncodeLike, Error, Input};
use sp_std::prelude::*;
use sp_core::RuntimeDebug;
use crate::traits::{Block as BlockT};
use parity_util_mem::{MallocSizeOf};

#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
pub struct DataLookup {
	/// size of the look up
	pub size: u32,
	/// sorted vector of tuples(key, start index)
	pub index: Vec<(u32, u32)>
}

impl DataLookup
{
	pub fn construct(
		extrinsics: &Vec<(u32, u32)> // app_id, tx len
	) -> Self {
		let mut index = Vec::new();
		// transactions are order by application id
		// skip transactions with 0 application id - it's not a data txs
		let mut size = 0;
		let mut prev_app_id = 0;
		for xt in extrinsics {
			let app_id = xt.0;
			let data_len = xt.1;

			if app_id != 0 && prev_app_id != app_id {
				index.push((app_id, size as u32));
			}

			size += data_len;
			prev_app_id = app_id;
		}

		DataLookup {
			size,
			index,
		}
	}
}
