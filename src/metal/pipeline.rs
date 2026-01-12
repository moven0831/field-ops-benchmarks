//! Metal compute pipeline management

use crate::BenchmarkError;
use metal::{ComputePipelineState, Device, Library, MTLSize};

/// Metal compute pipeline for a benchmark kernel
pub struct MetalPipeline {
    pub pipeline_state: ComputePipelineState,
    pub function_name: String,
    pub threads_per_threadgroup: MTLSize,
}

impl MetalPipeline {
    /// Create a new pipeline from a library and function name
    pub fn new(
        device: &Device,
        library: &Library,
        function_name: &str,
        workgroup_size: u32,
    ) -> Result<Self, BenchmarkError> {
        let function = library.get_function(function_name, None).map_err(|_| {
            BenchmarkError::ShaderCompilation(format!(
                "Function '{}' not found in library",
                function_name
            ))
        })?;

        let pipeline_state = device
            .new_compute_pipeline_state_with_function(&function)
            .map_err(|e| BenchmarkError::PipelineCreation(format!("{:?}", e)))?;

        let threads_per_threadgroup = MTLSize::new(workgroup_size as u64, 1, 1);

        Ok(Self {
            pipeline_state,
            function_name: function_name.to_string(),
            threads_per_threadgroup,
        })
    }

    /// Get the maximum threads per threadgroup for this pipeline
    pub fn max_threads_per_threadgroup(&self) -> u64 {
        self.pipeline_state.max_total_threads_per_threadgroup()
    }
}
