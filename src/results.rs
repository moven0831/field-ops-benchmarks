use crate::{Backend, Operation};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Result of a single benchmark run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Backend used
    pub backend: String,

    /// Operation benchmarked
    pub operation: String,

    /// Workgroup size used
    pub workgroup_size: u32,

    /// Total threads executed
    pub total_threads: u64,

    /// Operations per thread
    pub ops_per_thread: u32,

    /// Total operations executed
    pub total_operations: u64,

    /// Timing statistics (in nanoseconds)
    pub min_ns: u64,
    pub max_ns: u64,
    pub mean_ns: f64,
    pub std_dev_ns: f64,

    /// Derived metrics
    pub gops_per_second: f64,
}

impl BenchmarkResult {
    /// Create a new result from timing measurements
    pub fn from_timings(
        backend: Backend,
        operation: Operation,
        workgroup_size: u32,
        total_threads: u64,
        ops_per_thread: u32,
        timings: &[Duration],
    ) -> Self {
        let timings_ns: Vec<u64> = timings.iter().map(|d| d.as_nanos() as u64).collect();

        let min_ns = *timings_ns.iter().min().unwrap_or(&0);
        let max_ns = *timings_ns.iter().max().unwrap_or(&0);
        let sum: u64 = timings_ns.iter().sum();
        let mean_ns = sum as f64 / timings_ns.len().max(1) as f64;

        let variance: f64 = timings_ns
            .iter()
            .map(|&t| (t as f64 - mean_ns).powi(2))
            .sum::<f64>()
            / timings_ns.len().max(1) as f64;
        let std_dev_ns = variance.sqrt();

        let total_operations = total_threads * ops_per_thread as u64;

        // Calculate GOP/s using minimum time (best case)
        let gops_per_second = if min_ns > 0 {
            (total_operations as f64) / (min_ns as f64 / 1e9) / 1e9
        } else {
            0.0
        };

        Self {
            backend: backend.name().to_string(),
            operation: operation.name().to_string(),
            workgroup_size,
            total_threads,
            ops_per_thread,
            total_operations,
            min_ns,
            max_ns,
            mean_ns,
            std_dev_ns,
            gops_per_second,
        }
    }

    /// Get minimum time in milliseconds
    pub fn min_ms(&self) -> f64 {
        self.min_ns as f64 / 1e6
    }

    /// Get mean time in milliseconds
    pub fn mean_ms(&self) -> f64 {
        self.mean_ns / 1e6
    }
}

/// Collection of benchmark results with analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    /// Device information
    pub device_name: String,
    pub device_vendor: String,

    /// All benchmark results
    pub results: Vec<BenchmarkResult>,

    /// Timestamp of the report
    pub timestamp: String,
}

impl BenchmarkReport {
    pub fn new(device_name: String, device_vendor: String) -> Self {
        Self {
            device_name,
            device_vendor,
            results: Vec::new(),
            timestamp: chrono_lite_timestamp(),
        }
    }

    pub fn add_result(&mut self, result: BenchmarkResult) {
        self.results.push(result);
    }

    /// Calculate overhead of emulated vs native u64 addition
    pub fn u64_overhead(&self) -> Option<f64> {
        let native = self
            .results
            .iter()
            .find(|r| r.operation == "u64_add_native")?;
        let emulated = self
            .results
            .iter()
            .find(|r| r.operation == "u64_add_emulated")?;

        if native.gops_per_second > 0.0 {
            Some(emulated.min_ms() / native.min_ms())
        } else {
            None
        }
    }
}

/// Simple timestamp without chrono dependency
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}
