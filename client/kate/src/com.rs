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

use super::*;

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
	let max_block_size = config::MAX_BLOCK_SIZE;
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
	let rows_num = config::NUM_BLOBS;
	let extended_rows_num = rows_num * config::EXTENSION_FACTOR;
	let cols_num = config::NUM_CHUNKS_IN_BLOB;
	let no_of_bytes_in_chunk = config::CHUNK_SIZE;

	// generate data matrix first
	let mut block = flatten_and_pad_block(extrinsics);
	let mut chunk_elements = Vec::new();

	// prepare extended size
	chunk_elements.reserve_exact(extended_rows_num * cols_num);

	// force vector of desired size
	unsafe {
		chunk_elements.set_len(extended_rows_num * cols_num);
	}

	// generate column by column and pack into extended array of scalars
	let chunk_bytes_offset = rows_num * no_of_bytes_in_chunk;
	let mut offset = 0;
	for i in 0..cols_num {
		let mut chunk = block[i * chunk_bytes_offset..(i+1) * chunk_bytes_offset].chunks_exact(config::SCALAR_SIZE);
		for _ in 0..rows_num {
			// from_bytes_wide expects [u8;64]
			chunk_elements[offset] = dusk_plonk::prelude::BlsScalar::from_bytes_wide(chunk.next().unwrap().try_into().expect("slice with incorrect length"));
			offset += 1;
		}

		// offset extra rows
		offset += rows_num;
	}

	// extend data matrix, column by column
	let column_eval_domain = EvaluationDomain::new(extended_rows_num).unwrap();
	info!(
		target: "system",
		"SIZE {:#?}",
		column_eval_domain.size()
	);

	for i in 0..cols_num {
		let mut slice = &mut chunk_elements[i * extended_rows_num..(i+1) * extended_rows_num];

		// slice.into_iter().for_each(|it| {
		// 	info!(
		// 		target: "system",
		// 		"BEFORE {:#?}",
		// 		it
		// 	);
		// });

		let mut v = slice.to_vec();

		column_eval_domain.ifft_in_place(&mut v);
		column_eval_domain.fft_in_place(&mut v);

		// v.into_iter().for_each(|it| {
		// 	info!(
		// 		target: "system",
		// 		"AFTER {:#?}",
		// 		it
		// 	);
		// });

		break;
	}

	// construct commitments
	let public_params = kzg10::PublicParameters::from_bytes(public_params_data.as_slice()).unwrap();
	let (prover_key, verifier_key) = public_params.trim(cols_num).unwrap();
	let row_eval_domain = EvaluationDomain::new(cols_num).unwrap();
	let mut result_bytes: Vec<u8> = Vec::new();
	result_bytes.reserve_exact(48 * extended_rows_num);

	for i in 0..extended_rows_num {
		let mut row = Vec::new();

		for j in 0..cols_num {
			row.push(chunk_elements[i + j * extended_rows_num]);
		}

		let polynomial = Evaluations::from_vec_and_domain(row, row_eval_domain).interpolate();
		result_bytes.extend_from_slice(&prover_key.commit(&polynomial).unwrap().to_bytes());
	}

	// // Generate all the x-axis points of the domain on which all the column polynomials reside
	// let mut col_dom_x_pts = Vec::with_capacity(column_eval_domain.size());
	// col_dom_x_pts.extend(column_eval_domain.elements());
	//
	// // Generate all the x-axis points of the domain on which all the row polynomials reside
	// let mut row_dom_x_pts = Vec::with_capacity(row_eval_domain.size());
	// row_dom_x_pts.extend(row_eval_domain.elements());

	result_bytes
}
