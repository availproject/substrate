// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Benchmarks for the BABE Pallet.

use super::*;
use frame_benchmarking::benchmarks;
use hex_literal::hex;

type Header = sp_runtime::generic::Header<u64, sp_runtime::traits::BlakeTwo256>;

benchmarks! {
	check_equivocation_proof {
		let x in 0 .. 1;

		// NOTE: generated with the test below `test_generate_equivocation_report_blob`.
		// the output is not deterministic since keys are generated randomly (and therefore
		// signature content changes). it should not affect the benchmark.
		// with the current benchmark setup it is not possible to generate this programatically
		// from the benchmark setup.
		const EQUIVOCATION_PROOF_BLOB: [u8; 620] = hex!(
			"def12e42f3e487e9b14095aa8d5cc16a33491f1b50dadcf8811d1480f3fa86270b000000000000008526a
			9c21fa1aca2510754974532fe6efe00e3c9262d4bf738c32e545f010f2b28876f7ff1e2c8a7f391c6cd623
			13d35ed837a57c95b1b5e138984c24317f1d80c03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c
			082f29dcf4c11131481019188715cbf61d78e6ffc93ab30d9af35f00142f0462504dbfc128bb13af56b8fa
			f2a66aee08af074f536ffedcb2a52ed9188715cbf61d78e6ffc93ab30d9af35f00142f0462504dbfc128bb
			13af56b8faf2a66aee08af074f536ffedcb2a52ed010004000806424142453402000000000b00000000000
			00005424142450101d040a99d39a9704f43d6fe4eb6c4391bad218b7fe2d7cc838e2d2806ded932240dec4
			886ebab981a01c79aeb17b093326b3d693ad66ffc2c3034b6cf3839c5878526a9c21fa1aca251075497453
			2fe6efe00e3c9262d4bf738c32e545f010f2b28876f7ff1e2c8a7f391c6cd62313d35ed837a57c95b1b5e1
			38984c24317f1d80c03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c11131481019
			188715cbf61d78e6ffc93ab30d9af35f00142f0462504dbfc128bb13af56b8faf2a66aee08af074f536ffe
			dcb2a52ed9188715cbf61d78e6ffc93ab30d9af35f00142f0462504dbfc128bb13af56b8faf2a66aee08af
			074f536ffedcb2a52ed010004000806424142453402000000000b00000000000000054241424501010c704
			63cc03b63f3bfb30c0efba9149478a4aa1c550ea22021b5a9ce35dd170cc891de2d8fece48a6025c041b91
			1bd9fa69873da9456cc1cecfeb6fb9d182a8f");

		let equivocation_proof1: sp_consensus_babe::EquivocationProof<Header> =
			Decode::decode(&mut &EQUIVOCATION_PROOF_BLOB[..]).unwrap();

		let equivocation_proof2 = equivocation_proof1.clone();
	}: {
		sp_consensus_babe::check_equivocation_proof::<Header>(equivocation_proof1);
	} verify {
		assert!(sp_consensus_babe::check_equivocation_proof::<Header>(equivocation_proof2));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::*;
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		new_test_ext(3).execute_with(|| {
			assert_ok!(test_benchmark_check_equivocation_proof::<Test>());
		})
	}

	#[test]
	fn test_generate_equivocation_report_blob() {
		let (pairs, mut ext) = new_test_ext_with_pairs(3);

		let offending_authority_index = 0;
		let offending_authority_pair = &pairs[0];

		ext.execute_with(|| {
			start_era(1);

			let equivocation_proof = generate_equivocation_proof(
				offending_authority_index,
				offending_authority_pair,
				CurrentSlot::get() + 1,
			);

			println!("equivocation_proof: {:?}", equivocation_proof);
			let encoded = equivocation_proof.encode();
			let hex_encoded = hex::encode(encoded); 
			println!("equivocation_proof.encode(): {}", hex_encoded);
		});
	}
}
