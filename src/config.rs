use crate::{Backend, Operation};

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of operations per thread (used when auto_calibrate is false)
    pub ops_per_thread: u32,

    /// Workgroup size for GPU dispatch
    pub workgroup_size: u32,

    /// Number of workgroups to dispatch
    pub num_workgroups: u32,

    /// Number of warmup iterations (not timed)
    pub warmup_iterations: u32,

    /// Number of measurement iterations
    pub measurement_iterations: u32,

    /// Random seed for input data
    pub seed: u32,

    /// Use operation-specific ops_per_thread for faster completion
    pub auto_calibrate: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            ops_per_thread: 100,
            workgroup_size: 64,
            num_workgroups: 1024,
            warmup_iterations: 3,
            measurement_iterations: 10,
            seed: 0x12345678,
            auto_calibrate: true,
        }
    }
}

impl BenchmarkConfig {
    /// Create a new config with the given workgroup size
    pub fn with_workgroup_size(mut self, size: u32) -> Self {
        self.workgroup_size = size;
        self
    }

    /// Create a new config with the given ops per thread
    pub fn with_ops_per_thread(mut self, ops: u32) -> Self {
        self.ops_per_thread = ops;
        self
    }

    /// Create a new config with the given measurement iterations
    pub fn with_iterations(mut self, iterations: u32) -> Self {
        self.measurement_iterations = iterations;
        self
    }

    /// Enable or disable auto-calibration
    pub fn with_auto_calibrate(mut self, enabled: bool) -> Self {
        self.auto_calibrate = enabled;
        self
    }

    /// Get operation-specific config (uses calibrated ops_per_thread if auto_calibrate is true)
    pub fn for_operation(&self, op: Operation) -> Self {
        if self.auto_calibrate {
            Self {
                ops_per_thread: op.calibrated_ops_per_thread(),
                ..self.clone()
            }
        } else {
            self.clone()
        }
    }

    /// Total number of threads
    pub fn total_threads(&self) -> u64 {
        self.workgroup_size as u64 * self.num_workgroups as u64
    }

    /// Total number of operations
    pub fn total_operations(&self) -> u64 {
        self.total_threads() * self.ops_per_thread as u64
    }
}

/// A benchmark run specification
#[derive(Debug, Clone)]
pub struct BenchmarkRun {
    pub backend: Backend,
    pub operation: Operation,
    pub config: BenchmarkConfig,
}

impl BenchmarkRun {
    pub fn new(backend: Backend, operation: Operation) -> Self {
        Self {
            backend,
            operation,
            config: BenchmarkConfig::default(),
        }
    }

    pub fn with_config(mut self, config: BenchmarkConfig) -> Self {
        self.config = config;
        self
    }
}

/// Available workgroup sizes
pub const WORKGROUP_SIZES: [u32; 3] = [64, 128, 256];
