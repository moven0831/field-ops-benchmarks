pub mod config;
pub mod reporter;
pub mod results;
pub mod tui;

#[cfg(feature = "metal")]
pub mod metal;

#[cfg(feature = "webgpu")]
pub mod webgpu;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BenchmarkError {
    #[error("No GPU device found")]
    NoDevice,

    #[error("Backend not available: {0}")]
    BackendNotAvailable(String),

    #[error("Shader compilation failed: {0}")]
    ShaderCompilation(String),

    #[error("Pipeline creation failed: {0}")]
    PipelineCreation(String),

    #[error("Buffer creation failed: {0}")]
    BufferCreation(String),

    #[error("Execution failed: {0}")]
    Execution(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Available GPU backends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Backend {
    Metal,
    WebGPU,
}

impl Backend {
    pub fn name(&self) -> &'static str {
        match self {
            Backend::Metal => "Metal",
            Backend::WebGPU => "WebGPU",
        }
    }

    /// Returns true if this backend has native u64 support
    pub fn has_native_u64(&self) -> bool {
        matches!(self, Backend::Metal)
    }

    pub fn is_available(&self) -> bool {
        match self {
            #[cfg(feature = "metal")]
            Backend::Metal => cfg!(target_os = "macos"),
            #[cfg(not(feature = "metal"))]
            Backend::Metal => false,

            #[cfg(feature = "webgpu")]
            Backend::WebGPU => true,
            #[cfg(not(feature = "webgpu"))]
            Backend::WebGPU => false,
        }
    }

    pub fn all() -> Vec<Backend> {
        vec![Backend::Metal, Backend::WebGPU]
    }

    pub fn available() -> Vec<Backend> {
        Self::all()
            .into_iter()
            .filter(|b| b.is_available())
            .collect()
    }
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Benchmark operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    U32Baseline,
    U64Native,
    U64Emulated,
    BigIntMul,
    FieldMul,
    FieldAdd,
    FieldSub,
}

impl Operation {
    pub fn name(&self) -> &'static str {
        match self {
            Operation::U32Baseline => "u32_baseline",
            Operation::U64Native => "u64_native",
            Operation::U64Emulated => "u64_emulated",
            Operation::BigIntMul => "bigint_mul",
            Operation::FieldMul => "field_mul",
            Operation::FieldAdd => "field_add",
            Operation::FieldSub => "field_sub",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Operation::U32Baseline => "Native u32 multiply-add",
            Operation::U64Native => "Native 64-bit operations (Metal only)",
            Operation::U64Emulated => "u64 via u32 pairs with carry (WebGPU only)",
            Operation::BigIntMul => "BigInt256 multiplication (16x16-bit limbs)",
            Operation::FieldMul => "BN254 Montgomery field multiplication",
            Operation::FieldAdd => "BN254 field addition",
            Operation::FieldSub => "BN254 field subtraction",
        }
    }

    /// Returns true if this operation requires native u64 support
    pub fn requires_native_u64(&self) -> bool {
        matches!(self, Operation::U64Native)
    }

    /// Returns calibrated ops_per_thread for fast execution (~3-5 seconds per operation)
    pub fn calibrated_ops_per_thread(&self) -> u32 {
        match self {
            Operation::U32Baseline => 1_000,
            Operation::U64Native => 1_000,
            Operation::U64Emulated => 500,
            Operation::BigIntMul => 25,
            Operation::FieldMul => 20,
            Operation::FieldAdd => 20,
            Operation::FieldSub => 20,
        }
    }

    /// Returns true if this operation is only for backends without native u64
    pub fn is_emulation_only(&self) -> bool {
        matches!(self, Operation::U64Emulated)
    }

    pub fn all() -> Vec<Operation> {
        vec![
            Operation::U32Baseline,
            Operation::U64Native,
            Operation::U64Emulated,
            Operation::BigIntMul,
            Operation::FieldMul,
            Operation::FieldAdd,
            Operation::FieldSub,
        ]
    }

    /// Returns operations available for a specific backend
    pub fn available_for(backend: Backend) -> Vec<Operation> {
        Self::all()
            .into_iter()
            .filter(|op| {
                // u64_native only available on backends with native u64 support
                if op.requires_native_u64() {
                    backend.has_native_u64()
                }
                // u64_emulated only needed for backends without native u64
                else if op.is_emulation_only() {
                    !backend.has_native_u64()
                } else {
                    true
                }
            })
            .collect()
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
