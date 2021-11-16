
//! Autogenerated weights for pallet_babe
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-11-16, STEPS: [20, ], REPEAT: 10, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// ./target/release/node-template
// benchmark
// --chain
// dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// pallet_babe
// --extrinsic
// *
// --steps
// 20
// --repeat
// 10
// --raw
// --output
// ./frame/babe/src/weights.rs


#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_babe.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_babe::WeightInfo for WeightInfo<T> {
	fn check_equivocation_proof(x: u32, ) -> Weight {
		(147_333_000 as Weight)
			// Standard Error: 298_000
			.saturating_add((13_333_000 as Weight).saturating_mul(x as Weight))
	}
}
