//! Metal benchmark execution

use crate::config::BenchmarkConfig;
use crate::results::BenchmarkResult;
use crate::{Backend, BenchmarkError, Operation};
use metal::{Buffer, MTLResourceOptions, MTLSize};
use std::time::Instant;

use super::{MetalContext, MetalPipeline};

/// Benchmark runner for Metal
pub struct MetalRunner {
    ctx: MetalContext,
}

impl MetalRunner {
    pub fn new() -> Result<Self, BenchmarkError> {
        let ctx = MetalContext::new()?;
        Ok(Self { ctx })
    }

    pub fn device_name(&self) -> String {
        self.ctx.device_name()
    }

    /// Load metallib from embedded bytes
    pub fn load_library_data(&mut self, data: &[u8]) -> Result<(), BenchmarkError> {
        self.ctx.load_library_data(data)
    }

    /// Run a benchmark with the given configuration
    pub fn run_benchmark(
        &self,
        operation: Operation,
        config: &BenchmarkConfig,
    ) -> Result<BenchmarkResult, BenchmarkError> {
        // Get the kernel function name for this operation
        let function_name = operation_to_function_name(operation);

        // Check if we have a library loaded
        let library = self.ctx.library.as_ref().ok_or_else(|| {
            BenchmarkError::ShaderCompilation("No shader library loaded".to_string())
        })?;

        // Create the compute pipeline
        let pipeline = MetalPipeline::new(
            &self.ctx.device,
            library,
            &function_name,
            config.workgroup_size,
        )?;

        // Create buffers
        let total_threads = config.total_threads() as usize;
        let input_buffer = self.create_input_buffer(total_threads, config.seed)?;
        let output_buffer = self.create_output_buffer(total_threads)?;
        let params_buffer = self.create_params_buffer(config)?;

        // Warmup runs
        for _ in 0..config.warmup_iterations {
            self.dispatch(
                &pipeline,
                &input_buffer,
                &output_buffer,
                &params_buffer,
                config,
            )?;
        }

        // Timed runs
        let mut timings = Vec::with_capacity(config.measurement_iterations as usize);

        for _ in 0..config.measurement_iterations {
            let start = Instant::now();
            self.dispatch(
                &pipeline,
                &input_buffer,
                &output_buffer,
                &params_buffer,
                config,
            )?;
            timings.push(start.elapsed());
        }

        // Create result
        Ok(BenchmarkResult::from_timings(
            Backend::Metal,
            operation,
            config.workgroup_size,
            config.total_threads(),
            config.ops_per_thread,
            &timings,
            Some(1.5), // TODO: Detect actual GPU clock
        ))
    }

    /// Create input buffer with random data
    fn create_input_buffer(&self, count: usize, seed: u32) -> Result<Buffer, BenchmarkError> {
        let data: Vec<u32> = (0..count)
            .map(|i| seed.wrapping_add(i as u32).wrapping_mul(0x9E3779B9))
            .collect();

        let buffer = self.ctx.device.new_buffer_with_data(
            data.as_ptr() as *const _,
            (data.len() * std::mem::size_of::<u32>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );

        Ok(buffer)
    }

    /// Create output buffer
    fn create_output_buffer(&self, count: usize) -> Result<Buffer, BenchmarkError> {
        let buffer = self.ctx.device.new_buffer(
            (count * std::mem::size_of::<u32>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );

        Ok(buffer)
    }

    /// Create parameters buffer
    fn create_params_buffer(&self, config: &BenchmarkConfig) -> Result<Buffer, BenchmarkError> {
        #[repr(C)]
        struct BenchParams {
            iterations: u32,
            seed: u32,
            _pad0: u32,
            _pad1: u32,
        }

        let params = BenchParams {
            iterations: config.ops_per_thread,
            seed: config.seed,
            _pad0: 0,
            _pad1: 0,
        };

        let buffer = self.ctx.device.new_buffer_with_data(
            &params as *const _ as *const _,
            std::mem::size_of::<BenchParams>() as u64,
            MTLResourceOptions::StorageModeShared,
        );

        Ok(buffer)
    }

    /// Dispatch the compute kernel
    fn dispatch(
        &self,
        pipeline: &MetalPipeline,
        input_buffer: &Buffer,
        output_buffer: &Buffer,
        params_buffer: &Buffer,
        config: &BenchmarkConfig,
    ) -> Result<(), BenchmarkError> {
        let command_buffer = self.ctx.command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();

        encoder.set_compute_pipeline_state(&pipeline.pipeline_state);
        encoder.set_buffer(0, Some(input_buffer), 0);
        encoder.set_buffer(1, Some(output_buffer), 0);
        encoder.set_buffer(2, Some(params_buffer), 0);

        let threadgroups = MTLSize::new(config.num_workgroups as u64, 1, 1);
        let threads_per_threadgroup = pipeline.threads_per_threadgroup;

        encoder.dispatch_thread_groups(threadgroups, threads_per_threadgroup);
        encoder.end_encoding();

        command_buffer.commit();
        command_buffer.wait_until_completed();

        Ok(())
    }
}

/// Map operation to Metal kernel function name
fn operation_to_function_name(operation: Operation) -> String {
    match operation {
        Operation::U32Add => "bench_u32_add".to_string(),
        Operation::U64AddNative => "bench_u64_add".to_string(),
        Operation::U64AddEmulated => "bench_u64_add".to_string(),
        Operation::FieldMul => "bench_field_mul".to_string(),
        Operation::FieldAdd => "bench_field_add".to_string(),
        Operation::U256Add => "bench_u256_add".to_string(),
    }
}
