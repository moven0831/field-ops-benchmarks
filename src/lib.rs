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
    U32Add,
    U64AddNative,
    U64AddEmulated,
    FieldMul,
    FieldAdd,
    MersenneFieldAdd,
    MersenneFieldMul,
}

impl Operation {
    pub fn name(&self) -> &'static str {
        match self {
            Operation::U32Add => "u32_add",
            Operation::U64AddNative => "u64_add_native",
            Operation::U64AddEmulated => "u64_add_emulated",
            Operation::FieldMul => "field_mul",
            Operation::FieldAdd => "field_add",
            Operation::MersenneFieldAdd => "mersenne_field_add",
            Operation::MersenneFieldMul => "mersenne_field_mul",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Operation::U32Add => "Native u32 addition",
            Operation::U64AddNative => "Native 64-bit addition (Metal only)",
            Operation::U64AddEmulated => "u64 addition via u32 pairs with carry (WebGPU only)",
            Operation::FieldMul => "BN254 Montgomery field multiplication",
            Operation::FieldAdd => "BN254 field addition",
            Operation::MersenneFieldAdd => "Mersenne (2^31-1) field addition",
            Operation::MersenneFieldMul => "Mersenne (2^31-1) field multiplication",
        }
    }

    /// Returns true if this operation requires native u64 support
    pub fn requires_native_u64(&self) -> bool {
        matches!(self, Operation::U64AddNative)
    }

    /// Returns calibrated ops_per_thread for fast execution (~3-5 seconds per operation)
    pub fn calibrated_ops_per_thread(&self) -> u32 {
        match self {
            Operation::U32Add => 1_000,
            Operation::U64AddNative => 1_000,
            Operation::U64AddEmulated => 500,
            Operation::FieldMul => 20,
            Operation::FieldAdd => 20,
            Operation::MersenneFieldAdd => 500,
            Operation::MersenneFieldMul => 200,
        }
    }

    /// Returns true if this operation is only for backends without native u64
    pub fn is_emulation_only(&self) -> bool {
        matches!(self, Operation::U64AddEmulated)
    }

    pub fn all() -> Vec<Operation> {
        vec![
            Operation::U32Add,
            Operation::U64AddNative,
            Operation::U64AddEmulated,
            Operation::FieldMul,
            Operation::FieldAdd,
            Operation::MersenneFieldAdd,
            Operation::MersenneFieldMul,
        ]
    }

    /// Returns operations available for a specific backend
    pub fn available_for(backend: Backend) -> Vec<Operation> {
        Self::all()
            .into_iter()
            .filter(|op| {
                // u64_add_native only available on backends with native u64 support
                if op.requires_native_u64() {
                    backend.has_native_u64()
                }
                // u64_add_emulated only needed for backends without native u64
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
