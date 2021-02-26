use dusk_plonk::commitment_scheme::kzg10;
use dusk_plonk::fft::{EvaluationDomain,Evaluations};
use bls12_381::{G1Affine,G1Projective,Scalar};
use std::ops::MulAssign;
use frame_support::debug;
use std::{vec, thread};
use std::time::{Instant};
use std::iter;
use log::{info};
use std::convert::{TryInto, TryFrom};
use serde::{Serialize, Deserialize};
use rand::{SeedableRng, rngs::StdRng, Rng};
use super::*;
use dusk_plonk::prelude::BlsScalar;

#[derive(Serialize, Deserialize)]
pub struct Cell {
	pub row: u32,
	pub col: u32,
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

fn flatten_and_pad_block(extrinsics: &Vec<Vec<u8>>, header_hash: &[u8]) -> Vec<u8> {
	let max_block_size = config::MAX_BLOCK_SIZE;
	let mut block:Vec<u8> = extrinsics.clone().into_iter().flatten().collect::<Vec<u8>>(); // TODO probably can be done more efficiently

	if block.len() < max_block_size {
		let more_elems = max_block_size - block.len();
		block.reserve_exact(more_elems);
		let mut rng:StdRng = rand::SeedableRng::from_seed(<[u8; 32]>::try_from(header_hash).unwrap());
		let mut byte_index = 0;
		for i in 0..more_elems {
			// pseudo random values
			block.push(rng.gen::<u8>());
			byte_index += 1;
		}

		info!(target: "system", "flatten_and_pad_block last rng.gen::<u8>(): {:#?} {:#?} {:#?}", rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>());
	} else if block.len() > max_block_size {
		panic!("block is too big, must not happen!");
	}

	block
}

/// build extended data matrix, by columns
fn extend_data_matrix(block: &Vec<u8>) -> Vec<dusk_plonk::prelude::BlsScalar> {
	let rows_num = config::NUM_BLOBS;
	let extended_rows_num = rows_num * config::EXTENSION_FACTOR;
	let cols_num = config::NUM_CHUNKS_IN_BLOB;

	let mut chunk_elements = Vec::new();

	// prepare extended size
	chunk_elements.resize(extended_rows_num * cols_num, BlsScalar::zero());

	// force vector of desired size
	// unsafe {
	// 	chunk_elements.set_len(extended_rows_num * cols_num);
	// }

	// generate column by column and pack into extended array of scalars
	let chunk_bytes_offset = rows_num * config::CHUNK_SIZE;
	let mut offset = 0;
	for i in 0..cols_num {
		let mut chunk = block[i * chunk_bytes_offset..(i+1) * chunk_bytes_offset].chunks_exact(config::SCALAR_SIZE_WIDE);
		for _ in 0..rows_num {
			// from_bytes_wide expects [u8;64]
			chunk_elements[offset] = BlsScalar::from_bytes_wide(chunk.next().unwrap().try_into().expect("slice with incorrect length"));
			offset += 1;
		}

		// offset extra rows
		offset += rows_num;
	}

	info!(
		target: "system",
		"extend_data_matrix {:#?}",
		chunk_elements[0],
	);

	let copy = chunk_elements[0];

	// extend data matrix, column by column
	let extended_column_eval_domain = EvaluationDomain::new(extended_rows_num).unwrap();
	let column_eval_domain = EvaluationDomain::new(rows_num).unwrap();

	for i in 0..cols_num {
		let mut original_column = &mut chunk_elements[i * extended_rows_num..(i+1) * extended_rows_num - extended_rows_num / config::EXTENSION_FACTOR];
		column_eval_domain.ifft_slice(original_column);

		let mut extended_column = &mut chunk_elements[i * extended_rows_num..(i+1) * extended_rows_num];
		extended_column_eval_domain.fft_slice(extended_column);
	}

	assert_eq!(copy, chunk_elements[0]);

	info!(
		target: "system",
		"extend_data_matrix 2 {:#?}",
		chunk_elements[0],
	);

	chunk_elements
}

//TODO cache extended data matrix
//TODO explore faster Variable Base Multi Scalar Multiplication
pub fn build_proof(public_params_data: &Vec<u8>, extrinsics: &Vec<Vec<u8>>, cells: Vec<Cell>, header_hash: &[u8]) -> Option<Vec<u8>> {
	let rows_num = config::NUM_BLOBS;
	let extended_rows_num = rows_num * config::EXTENSION_FACTOR;
	let cols_num = config::NUM_CHUNKS_IN_BLOB;

	if cells.len() > config::MAX_PROOFS_REQUEST {
		()
	}

	let block = flatten_and_pad_block(extrinsics, header_hash);
	let ext_data_matrix = extend_data_matrix(&block);

	let public_params = kzg10::PublicParameters::from_bytes(public_params_data.as_slice()).unwrap();
	let (prover_key, verifier_key) = public_params.trim(cols_num).unwrap();

	// Generate all the x-axis points of the domain on which all the row polynomials reside
	let row_eval_domain = EvaluationDomain::new(cols_num).unwrap();
	let mut row_dom_x_pts = Vec::with_capacity(row_eval_domain.size());
	row_dom_x_pts.extend(row_eval_domain.elements());

	let mut result_bytes: Vec<u8> = Vec::new();
	let serialized_proof_size = config::SCALAR_SIZE + config::PROOF_SIZE;
	result_bytes.reserve_exact(serialized_proof_size * cells.len());
	unsafe {
		result_bytes.set_len(serialized_proof_size * cells.len());
	}

	let prover_key = &prover_key;
	let ext_data_matrix = &ext_data_matrix;
	let row_dom_x_pts = &row_dom_x_pts;

	info!(
		target: "system",
		"Number of CPU cores: {:#?}",
		num_cpus::get()
	);
	// generate proof only for requested cells
	let total_start= Instant::now();
	Iterator::enumerate(cells.iter()).for_each(|(index, cell)| {
		let row_index = cell.row as usize;
		let col_index = cell.col as usize;

		if row_index < extended_rows_num && col_index < cols_num {
			// construct polynomial per extended matrix row
			let mut row = Vec::with_capacity(cols_num);

			for j in 0..cols_num {
				row.push(ext_data_matrix[row_index + j * extended_rows_num]);
			}

			let polynomial = Evaluations::from_vec_and_domain(row, row_eval_domain).interpolate();
			let witness = prover_key.compute_single_witness(&polynomial, &row_dom_x_pts[col_index]);
			let commitment_to_witness = prover_key.commit(&witness).unwrap();
			let evaluated_point = ext_data_matrix[row_index + col_index * extended_rows_num];

			info!(
				target: "system",
				"commitment_to_witness {:#?}\n evaluated_point {:#?}\n polynomial: {:#?} {:?} {:?}",
				commitment_to_witness,
				evaluated_point,
				prover_key.commit(&polynomial).unwrap(),
				row_index,
				col_index,
			);

			unsafe {
				std::ptr::copy(
					commitment_to_witness.to_bytes().as_ptr(),
					result_bytes.as_mut_ptr().add(index * serialized_proof_size),
					config::PROOF_SIZE
				);

				std::ptr::copy(
					evaluated_point.to_bytes().as_ptr(),
					result_bytes.as_mut_ptr().add(index * serialized_proof_size + config::PROOF_SIZE),
					config::SCALAR_SIZE
				);
			}
		}
	});

	info!(
		target: "system",
		"Time to build 1 row of proofs {:?}",
		total_start.elapsed()
	);

	Some(result_bytes)
}

pub fn build_commitments(public_params_data: &Vec<u8>, extrinsics: &Vec<Vec<u8>>, header_hash: &[u8]) -> Vec<u8> {
	let rows_num = config::NUM_BLOBS;
	let extended_rows_num = rows_num * config::EXTENSION_FACTOR;
	let cols_num = config::NUM_CHUNKS_IN_BLOB;

	let start= Instant::now();

	// generate data matrix first
	let block = flatten_and_pad_block(extrinsics, header_hash);
	let ext_data_matrix = extend_data_matrix(&block);

	info!(
		target: "system",
		"Time to prepare {:?}",
		start.elapsed()
	);

	// construct commitments in parallel
	let public_params = kzg10::PublicParameters::from_bytes(public_params_data.as_slice()).unwrap();
	let (prover_key, _) = public_params.trim(cols_num).unwrap();
	let row_eval_domain = EvaluationDomain::new(cols_num).unwrap();

	let mut result_bytes: Vec<u8> = Vec::new();
	result_bytes.reserve_exact(config::PROVER_KEY_SIZE * extended_rows_num);
	unsafe {
		result_bytes.set_len(config::PROVER_KEY_SIZE * extended_rows_num);
	}

	info!(
		target: "system",
		"Number of CPU cores: {:#?}",
		num_cpus::get()
	);

	let start = Instant::now();
	for i in 0..extended_rows_num {
		let mut row = Vec::with_capacity(cols_num);

		for j in 0..cols_num {
			row.push(ext_data_matrix[i + j * extended_rows_num]);
		}

		let polynomial = Evaluations::from_vec_and_domain(row, row_eval_domain).interpolate();
		let key_bytes = &prover_key.commit(&polynomial).unwrap().to_bytes();

		unsafe {
			std::ptr::copy(
				key_bytes.as_ptr(),
				result_bytes.as_mut_ptr().add(i * config::PROVER_KEY_SIZE),
				config::PROVER_KEY_SIZE
			);
		}
	}

	info!(
		target: "system",
		"Time to build a commitment {:?}",
		start.elapsed()
	);

	result_bytes
}
