// There should be no data races happening because not a single operation will be executed at the same time.
// Example: We cannot run block import before we actually download the block from our peers.
//
// Importing the block is the last action that we are doing so that's the moment where we are going to send
// the telemetry data.

use std::{
	sync::Mutex,
	time::{SystemTime, SystemTimeError, UNIX_EPOCH},
};

#[derive(Debug, Default, Clone)]
pub struct BlockMetrics {
	pub proposal_end_timestamp: Option<(u64, u128)>, // (block number, timestamp in ms)
	pub proposal_time: Option<(u64, u128)>,          // (block number, time in ms)
	pub new_sync_target_timestamp: Option<(u64, u128)>, // (block number, timestamp in ms)
	pub accepted_block_timestamp: Option<(u64, u128)>, // (block number, timestamp in ms)
	pub import_block_start_timestamp: Option<(u64, u128)>, // (block number, timestamp in ms).
	pub import_block_end_timestamp: Option<(u64, u128)>, // (block number, timestamp in ms).
}

pub static BLOCK_METRICS: Mutex<BlockMetrics> = Mutex::new(BlockMetrics::new());
impl BlockMetrics {
	pub const fn new() -> Self {
		Self {
			proposal_end_timestamp: None,
			proposal_time: None,
			new_sync_target_timestamp: None,
			accepted_block_timestamp: None,
			import_block_start_timestamp: None,
			import_block_end_timestamp: None,
		}
	}

	pub fn observe_proposal_end_timestamp(block_number: u64) {
		let Ok(timestamp) = Self::get_current_timestamp_in_ms() else {
			return;
		};

		let Ok(mut metrics) = BLOCK_METRICS.lock() else {
			return;
		};

		metrics.proposal_end_timestamp = Some((block_number, timestamp));
	}

	pub fn observe_proposal_time(block_number: u64, time: u128) {
		let Ok(mut metrics) = BLOCK_METRICS.lock() else {
			return;
		};

		metrics.proposal_time = Some((block_number, time));
	}

	pub fn observe_new_sync_target_timestamp(block_number: u64) {
		let Ok(timestamp) = Self::get_current_timestamp_in_ms() else {
			return;
		};
		let Ok(mut metrics) = BLOCK_METRICS.lock() else {
			return;
		};

		metrics.new_sync_target_timestamp = Some((block_number, timestamp));
	}

	pub fn observe_accepted_block_timestamp(block_number: u64) {
		let Ok(timestamp) = Self::get_current_timestamp_in_ms() else {
			return;
		};
		let Ok(mut metrics) = BLOCK_METRICS.lock() else {
			return;
		};

		metrics.accepted_block_timestamp = Some((block_number, timestamp));
	}

	pub fn observe_import_block_start_timestamp(block_number: u64) {
		let Ok(timestamp) = Self::get_current_timestamp_in_ms() else {
			return;
		};
		let Ok(mut metrics) = BLOCK_METRICS.lock() else {
			return;
		};

		metrics.import_block_start_timestamp = Some((block_number, timestamp));
	}

	pub fn observe_import_block_end_timestamp(block_number: u64) {
		let Ok(timestamp) = Self::get_current_timestamp_in_ms() else {
			return;
		};
		let Ok(mut metrics) = BLOCK_METRICS.lock() else {
			return;
		};

		metrics.import_block_end_timestamp = Some((block_number, timestamp));
	}

	pub fn take() -> BlockMetrics {
		let Ok(mut metrics) = BLOCK_METRICS.lock() else {
			return BlockMetrics::new();
		};

		let val = metrics.clone();
		*metrics = BlockMetrics::new();

		val
	}

	pub fn to_block_metrics_telemetry(self) -> Option<BlockMetricsTelemetry> {
		let mut proposal_timestamps = None;
		if let (Some(end), Some(time)) = (&self.proposal_end_timestamp, &self.proposal_time) {
			if end.0 == time.0 {
				proposal_timestamps = Some(((end.1 - time.1) as u64, end.1 as u64, end.0));
			}
		}

		let mut sync_block_timestamps = None;
		if let (Some(start), Some(end)) =
			(&self.new_sync_target_timestamp, &self.accepted_block_timestamp)
		{
			if start.0 == end.0 {
				sync_block_timestamps = Some((start.1 as u64, end.1 as u64, start.0));
			}
		}

		let mut import_block_timestamps = None;
		if let (Some(start), Some(end)) =
			(&self.import_block_start_timestamp, &self.import_block_end_timestamp)
		{
			if start.0 == end.0 {
				import_block_timestamps = Some((start.1 as u64, end.1 as u64, start.0));
			}
		}

		if proposal_timestamps.is_none()
			&& sync_block_timestamps.is_none()
			&& import_block_timestamps.is_none()
		{
			return None;
		}

		Some(BlockMetricsTelemetry {
			proposal_timestamps,
			sync_block_timestamps,
			import_block_timestamps,
		})
	}

	fn get_current_timestamp_in_ms() -> Result<u128, SystemTimeError> {
		let start = SystemTime::now();
		start.duration_since(UNIX_EPOCH).map(|f| f.as_millis())
	}
}

#[derive(Debug)]
pub struct BlockMetricsTelemetry {
	pub proposal_timestamps: Option<(u64, u64, u64)>, // (timestamp in ms (start, end, block_number))
	pub sync_block_timestamps: Option<(u64, u64, u64)>, // (timestamp in ms (start, end, block_number))
	pub import_block_timestamps: Option<(u64, u64, u64)>, // (timestamp in ms (start, end, block_number))
}
