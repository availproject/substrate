#![cfg_attr(not(feature = "std"), no_std)]

use dusk_plonk::commitment_scheme::kzg10;
use dusk_plonk::fft::{EvaluationDomain,Evaluations};
use bls12_381::{G1Affine,G1Projective,Scalar};
use std::ops::MulAssign;
use frame_support::debug;
use std::vec;
use std::iter;
use log::{info};
use frame_support::traits::Len;
use std::convert::TryInto;

pub mod config {
	pub const NUM_BLOBS: usize = 32;
	pub const NUM_CHUNKS_IN_BLOB: usize = 64;

	/// in bytes
	pub const CHUNK_SIZE: usize = 64;
}

#[inline]
fn bitreverse(mut n: u32, l: u32) -> u32 {
	let mut r = 0;
	for _ in 0..l {
		r = (r << 1) | (n & 1);
		n >>= 1;
	}
	r
}

fn serial_fft(a: &mut [G1Projective], omega: Scalar, log_n: u32) {
	let n = a.len() as u32;
	assert_eq!(n, 1 << log_n);

	for k in 0..n {
		let rk = bitreverse(k, log_n);
		if k < rk {
			a.swap(rk as usize, k as usize);
		}
	}

	let mut m = 1;
	for _ in 0..log_n {
		let w_m = omega.pow(&[(n / (2 * m)) as u64, 0, 0, 0]);

		let mut k = 0;
		while k < n {
			let mut w = Scalar::one();
			for j in 0..m {
				let mut t = a[(k + j + m) as usize];
				t = t * &w;
				let mut tmp = a[(k + j) as usize];
				tmp = tmp - &t;
				a[(k + j + m) as usize] = tmp;
				a[(k + j) as usize] += &t;
				w.mul_assign(&w_m);
			}

			k += 2 * m;
		}

		m *= 2;
	}
}

// function to perform FFT/IFFT on commitments on the given evaluation domain- depending on inverse flag
fn group_fft (a: Vec<kzg10::Commitment>, eval_dom: EvaluationDomain, inverse: bool) -> Vec<kzg10::Commitment>{
	// Convert Commitments to G1Affine elements
	let mut commits = Vec::new();
	for i in 0..a.len() {
		commits.push(G1Projective::from( G1Affine::from_uncompressed(&a[i].0.to_uncompressed()).unwrap()));
	}
	commits.resize(eval_dom.size(), G1Projective::identity());

	if inverse {
		// Perform IFFT
		serial_fft(&mut commits, Scalar::from_bytes(&eval_dom.group_gen_inv.to_bytes()).unwrap(), eval_dom.log_size_of_group);
		commits.iter_mut().for_each(|val| *val *= Scalar::from_bytes(&eval_dom.size_inv.to_bytes()).unwrap());
	}
	else {
		// Perform FFT
		serial_fft(&mut commits, Scalar::from_bytes(&eval_dom.group_gen.to_bytes()).unwrap(), eval_dom.log_size_of_group);
	}

	// let mut affine_from_projective = vec![G1Affine::identity(); eval_dom.size()];
	// G1Projective::batch_normalize(&commits, &mut affine_from_projective);

	let mut modified_commits = Vec::new();
	// for i in 0..commits.len() {
	// 	modified_commits.push(kzg10::Commitment::from_affine(dusk_plonk::bls12_381::G1Affine::from_uncompressed(&affine_from_projective[i].to_uncompressed()).unwrap()));
	// }
	modified_commits
}

fn flatten_and_pad_block(extrinsics: &Vec<Vec<u8>>) -> Vec<u8> {
	let max_block_size = config::NUM_BLOBS*config::NUM_CHUNKS_IN_BLOB*config::CHUNK_SIZE;
	let mut block:Vec<u8> = extrinsics.clone().into_iter().flatten().collect::<Vec<u8>>(); // TODO probably can be done more efficiently

	if block.len() < max_block_size {
		let more_elems = max_block_size - block.len();
		block.reserve_exact(more_elems);
		for i in 0..more_elems {
			block.push((i % 256) as u8);
		}
	} else if block.len() > max_block_size {
		panic!("block is too big, must not happen!");
	}

	block
}

pub fn build_kc(public_params_data: &Vec<u8>, extrinsics: &Vec<Vec<u8>>) -> Vec<u8> {
	let no_of_blobs = config::NUM_BLOBS;
	let no_of_chunks_in_blob = config::NUM_CHUNKS_IN_BLOB;
	let no_of_bytes_in_chunk = config::CHUNK_SIZE;

	let public_params = kzg10::PublicParameters::from_bytes(public_params_data.as_slice()).unwrap();
	let (prover_key, verifier_key) = public_params.trim(no_of_chunks_in_blob).unwrap();
	let row_eval_domain = EvaluationDomain::new(no_of_chunks_in_blob).unwrap();
	let column_eval_domain = EvaluationDomain::new(no_of_blobs*2).unwrap();

	// Generate all the x-axis points of the domain on which all the column polynomials reside
	let mut col_dom_x_pts = Vec::new();
	let mut pt_it = column_eval_domain.elements();
	for _ in 0..column_eval_domain.size() {
		col_dom_x_pts.push(pt_it.next().unwrap());
	}

	// Generate all the x-axis points of the domain on which all the row polynomials reside
	let mut row_dom_x_pts = Vec::new();
	let mut pt_it = row_eval_domain.elements();
	for _ in 0..row_eval_domain.size() {
		row_dom_x_pts.push(pt_it.next().unwrap());
	}

	// Generate polynomials representing data blobs - each having 64 elements
	// Generate commitment to each data blob - C1, .. , Cn
	let mut block = flatten_and_pad_block(extrinsics);
	let mut blob_polynomials = Vec::new();
	let mut blob_elements = Vec::new();
	let mut blob_commits = Vec::new();
	for i in 0..no_of_blobs {
		let mut chunk = block[i*no_of_chunks_in_blob*no_of_bytes_in_chunk..(i+1)*no_of_chunks_in_blob*no_of_bytes_in_chunk].chunks_exact(64);
		let mut chunk_elements = Vec::new();
		for _ in 0..no_of_chunks_in_blob {
			// from_bytes_wide expects [u8;64]
			chunk_elements.push(dusk_plonk::prelude::BlsScalar::from_bytes_wide(chunk.next().unwrap().try_into().expect("slice with incorrect length")));
		}
		blob_elements.push(chunk_elements.clone());
		blob_polynomials.push(Evaluations::from_vec_and_domain(chunk_elements, row_eval_domain).interpolate());
		// blob_polynomials.push(Polynomial::from_coefficients_vec(chunk_elements));
		blob_commits.push(prover_key.commit(&blob_polynomials[i]).unwrap());
	}

	// PROOFS

	// Take commitments C1 to Cn and extend to C{n+1} to C{2n}
	// Ignoring the sampling before extending commitment set
	// let blob_commits = group_fft(group_fft(blob_commits, column_eval_domain, true), column_eval_domain, false);

	// The blob/block producer can cache the proof for each chunk to save redundant work
	// This is the worst case approach. In practice, the proof is computed and cached when a chunk is queried the first time
	// let mut proofs = Vec::new();
	// for i in 0..no_of_blobs {
	// 	let mut proof_row = Vec::new();
	// 	for j in 0..no_of_chunks_in_blob {
	// 		proof_row.push(prover_key.open_single(&blob_polynomials[i], &blob_elements[i][j], &row_dom_x_pts[j]).unwrap())
	// 	}
	// 	proofs.push(proof_row);
	// }

	// Serialize and flatten
	let bytes_commitments = blob_commits
		.iter()
		.map(|it| { it.to_bytes() })
		.collect::<Vec<[u8; 48]>>();

	let mut result_bytes: Vec<u8> = Vec::new();
	result_bytes.reserve_exact(48 * bytes_commitments.len());

	bytes_commitments.iter().for_each(|it| {
		result_bytes.extend_from_slice(it);
	});

	result_bytes
}
