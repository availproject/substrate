// There should be no data races happening because not a single operation will be executed at the same time.
// Example: We cannot run block import before we actually download the block from our peers.
//
// Importing the block is the last action that we are doing so that's the moment where we are going to send
// the telemetry data.

use std::{
	sync::OnceLock,
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

pub const BLOCK_METRICS: OnceLock<BlockMetrics> = OnceLock::new();
impl BlockMetrics {
	pub fn observe_proposal_end_timestamp(block_number: u64) {
		let Ok(timestamp) = Self::get_current_timestamp_in_ms() else {
			return;
		};

		let mut metrics = BLOCK_METRICS.get_or_init(|| BlockMetrics::default()).clone();
		metrics.proposal_end_timestamp = Some((block_number, timestamp));
		_ = BLOCK_METRICS.set(metrics);
	}

	pub fn observe_proposal_time(block_number: u64, time: u128) {
		let mut metrics = BLOCK_METRICS.get_or_init(|| BlockMetrics::default()).clone();
		metrics.proposal_time = Some((block_number, time));
		_ = BLOCK_METRICS.set(metrics);
	}

	pub fn observe_new_sync_target_timestamp(block_number: u64) {
		let Ok(timestamp) = Self::get_current_timestamp_in_ms() else {
			return;
		};

		let mut metrics = BLOCK_METRICS.get_or_init(|| BlockMetrics::default()).clone();
		metrics.new_sync_target_timestamp = Some((block_number, timestamp));
		_ = BLOCK_METRICS.set(metrics);
	}

	pub fn observe_accepted_block_timestamp(block_number: u64) {
		let Ok(timestamp) = Self::get_current_timestamp_in_ms() else {
			return;
		};

		let mut metrics = BLOCK_METRICS.get_or_init(|| BlockMetrics::default()).clone();
		metrics.accepted_block_timestamp = Some((block_number, timestamp));
		_ = BLOCK_METRICS.set(metrics);
	}

	pub fn observe_import_block_start_timestamp(block_number: u64) {
		let Ok(timestamp) = Self::get_current_timestamp_in_ms() else {
			return;
		};

		let mut metrics: BlockMetrics =
			BLOCK_METRICS.get_or_init(|| BlockMetrics::default()).clone();
		metrics.import_block_start_timestamp = Some((block_number, timestamp));
		_ = BLOCK_METRICS.set(metrics);
	}

	pub fn observe_import_block_end_timestamp(block_number: u64) {
		let Ok(timestamp) = Self::get_current_timestamp_in_ms() else {
			return;
		};

		let mut metrics = BLOCK_METRICS.get_or_init(|| BlockMetrics::default()).clone();
		metrics.import_block_end_timestamp = Some((block_number, timestamp));
		_ = BLOCK_METRICS.set(metrics);
	}

	pub fn take() -> BlockMetrics {
		let metrics = BLOCK_METRICS.get_or_init(|| BlockMetrics::default()).clone();
		_ = BLOCK_METRICS.set(BlockMetrics::default());

		metrics
	}

	fn get_current_timestamp_in_ms() -> Result<u128, SystemTimeError> {
		let start = SystemTime::now();
		start.duration_since(UNIX_EPOCH).map(|f| f.as_millis())
	}
}

impl TryFrom<BlockMetrics> for BlockMetricsTelemetry {
	type Error = ();

	fn try_from(value: BlockMetrics) -> Result<Self, Self::Error> {
		let mut proposal_timestamps = None;
		if let (Some(end), Some(time)) = (&value.proposal_end_timestamp, &value.proposal_time) {
			if end.0 == time.0 {
				proposal_timestamps = Some((end.1 - time.1, end.1, end.0));
			}
		}

		let mut sync_block_start_timestamps = None;
		if let (Some(start), Some(end)) =
			(&value.new_sync_target_timestamp, &value.accepted_block_timestamp)
		{
			if start.0 == end.0 {
				sync_block_start_timestamps = Some((start.1, end.1, start.0));
			}
		}

		let mut import_block_timestamps = None;
		if let (Some(start), Some(end)) =
			(&value.import_block_start_timestamp, &value.import_block_end_timestamp)
		{
			if start.0 == end.0 {
				import_block_timestamps = Some((start.1, end.1, start.0));
			}
		}

		if proposal_timestamps.is_none()
			&& sync_block_start_timestamps.is_none()
			&& import_block_timestamps.is_none()
		{
			return Err(());
		}

		Ok(Self { proposal_timestamps, sync_block_start_timestamps, import_block_timestamps })
	}
}

pub struct BlockMetricsTelemetry {
	pub proposal_timestamps: Option<(u128, u128, u64)>, // (timestamp in ms (start, end, block_number))
	pub sync_block_start_timestamps: Option<(u128, u128, u64)>, // (timestamp in ms (start, end, block_number))
	pub import_block_timestamps: Option<(u128, u128, u64)>, // (timestamp in ms (start, end, block_number))
}
