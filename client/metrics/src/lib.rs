// The canonical usage is to call MetricActions::observe_metric() when a metric needs to be recorded.
// The metric will be stored inside a vector and won't be send to the telemetry until it is requested.
//
// MetricActions::send_telemetry() sends all stored metrics to the telemetry endpoints and resets the
// storage.
//
// For partial recordings, use MetricActions::observe_metric_partial(). This allows to capture
// a metric in different points in the system without passing around the start and end timestamp.
//
// In order for the telemetry to work, MetricActions::subscribe_telemetry() needs to be called
// with a valid telemetry handle.

use sc_telemetry::{telemetry, TelemetryHandle, SUBSTRATE_INFO};
use std::{
	sync::Mutex,
	time::{SystemTime, SystemTimeError, UNIX_EPOCH},
};

use serde::Serialize;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum MetricKind {
	PROPOSAL = 0,
	SYNC = 1,
	IMPORT = 2,
}

impl MetricKind {
	pub fn from_u8(value: u8) -> Option<Self> {
		match value {
			0 => Some(Self::PROPOSAL),
			1 => Some(Self::SYNC),
			2 => Some(Self::IMPORT),
			_ => None,
		}
	}

	pub fn to_u8(self) -> u8 {
		self as u8
	}
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct MetricDetail {
	kind: u8,
	block_number: u64,
	start_timestamp: u64,
	end_timestamp: u64,
}

impl MetricDetail {
	pub fn new(
		kind: MetricKind,
		block_number: u64,
		start_timestamp: u64,
		end_timestamp: u64,
	) -> Self {
		Self { kind: kind.to_u8(), block_number, start_timestamp, end_timestamp }
	}
}

static STORED_METRICS: Mutex<Vec<MetricDetail>> = Mutex::new(Vec::new());
static TELEMETRY_HANDLE: Mutex<Option<TelemetryHandle>> = Mutex::new(None);

pub struct MetricActions;

impl MetricActions {
	pub fn subscribe_telemetry(handle: Option<TelemetryHandle>) {
		let Ok(mut lock) = TELEMETRY_HANDLE.lock() else {
			return;
		};

		*lock = handle;
	}

	pub fn observe_metric_option(
		kind: MetricKind,
		block_number: Option<u64>,
		start_timestamp: Option<u128>,
		end_timestamp: Option<u128>,
	) {
		if let (Some(block_number), Some(start_timestamp), Some(end_timestamp)) =
			(block_number, start_timestamp, end_timestamp)
		{
			Self::observe_metric(kind, block_number, start_timestamp, end_timestamp);
		}
	}

	pub fn observe_metric(
		kind: MetricKind,
		block_number: u64,
		start_timestamp: u128,
		end_timestamp: u128,
	) {
		let Ok(mut lock) = STORED_METRICS.lock() else {
			return;
		};

		lock.push(MetricDetail::new(
			kind,
			block_number,
			start_timestamp as u64,
			end_timestamp as u64,
		));
	}

	pub fn observe_metric_partial(
		kind: MetricKind,
		block_number: Option<u64>,
		timestamp: Option<u128>,
		is_start: bool,
	) {
		let Some(block_number) = block_number else {
			return;
		};
		let Some(timestamp) = timestamp else {
			return;
		};
		let timestamp = timestamp as u64;

		let Ok(mut lock) = STORED_METRICS.lock() else {
			return;
		};

		let item = lock
			.iter_mut()
			.find(|i| i.block_number == block_number && i.kind == kind.to_u8());
		if let Some(item) = item {
			if is_start {
				item.start_timestamp = timestamp;
			} else {
				item.end_timestamp = timestamp;
			}
		} else {
			let metric = if is_start {
				MetricDetail::new(kind, block_number, timestamp, 0)
			} else {
				MetricDetail::new(kind, block_number, 0, timestamp)
			};
			lock.push(metric)
		}
	}

	pub fn send_telemetry() {
		let Ok(mut metrics_lock) = STORED_METRICS.lock() else {
			return;
		};
		let Ok(telemetry) = TELEMETRY_HANDLE.lock() else {
			return;
		};

		let metrics: Vec<MetricDetail> = std::mem::take(&mut metrics_lock);
		telemetry!(
			telemetry;
			SUBSTRATE_INFO;
			"block.metrics";
			"metrics" => metrics,
		);
	}

	pub fn get_current_timestamp_in_ms() -> Result<u128, SystemTimeError> {
		let start = SystemTime::now();
		start.duration_since(UNIX_EPOCH).map(|f| f.as_millis())
	}
}
