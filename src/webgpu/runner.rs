//! WebGPU benchmark execution

use crate::config::BenchmarkConfig;
use crate::results::BenchmarkResult;
use crate::{Backend, BenchmarkError, Operation};
use std::collections::HashMap;
use std::time::Instant;
use wgpu::util::DeviceExt;

use super::{WebGpuContext, WebGpuPipeline};

/// Benchmark runner for WebGPU
pub struct WebGpuRunner {
    ctx: WebGpuContext,
    shaders: HashMap<Operation, String>,
}

impl WebGpuRunner {
    pub fn new() -> Result<Self, BenchmarkError> {
        let ctx = WebGpuContext::new()?;
        let shaders = Self::load_shaders();
        Ok(Self { ctx, shaders })
    }

    pub fn device_name(&self) -> String {
        self.ctx.device_name()
    }

    /// Load all WGSL shaders
    fn load_shaders() -> HashMap<Operation, String> {
        let mut shaders = HashMap::new();

        // Include shaders at compile time
        shaders.insert(
            Operation::U32Add,
            include_str!("../../shaders/wgsl/bench_u32_add.wgsl").to_string(),
        );
        shaders.insert(
            Operation::U64AddEmulated,
            include_str!("../../shaders/wgsl/bench_u64_add.wgsl").to_string(),
        );
        shaders.insert(
            Operation::FieldMul,
            include_str!("../../shaders/wgsl/bench_field_mul.wgsl").to_string(),
        );
        shaders.insert(
            Operation::FieldAdd,
            include_str!("../../shaders/wgsl/bench_field_add.wgsl").to_string(),
        );
        shaders.insert(
            Operation::U256Add,
            include_str!("../../shaders/wgsl/bench_u256_add.wgsl").to_string(),
        );
        shaders.insert(
            Operation::MersenneFieldAdd,
            include_str!("../../shaders/wgsl/bench_mersenne_field_add.wgsl").to_string(),
        );
        shaders.insert(
            Operation::MersenneFieldMul,
            include_str!("../../shaders/wgsl/bench_mersenne_field_mul.wgsl").to_string(),
        );

        shaders
    }

    /// Run a benchmark with the given configuration
    pub fn run_benchmark(
        &self,
        operation: Operation,
        config: &BenchmarkConfig,
    ) -> Result<BenchmarkResult, BenchmarkError> {
        // Get shader source
        let shader_source = self.shaders.get(&operation).ok_or_else(|| {
            BenchmarkError::ShaderCompilation(format!(
                "No shader found for operation: {}",
                operation.name()
            ))
        })?;

        // Create pipeline
        let entry_point = operation_to_entry_point(operation);
        let pipeline = WebGpuPipeline::new(
            &self.ctx.device,
            shader_source,
            entry_point,
            config.workgroup_size,
        )?;

        // Create buffers
        let total_threads = config.total_threads() as usize;
        let input_buffer = self.create_input_buffer(config.seed);
        let output_buffer = self.create_output_buffer(total_threads);
        let params_buffer = self.create_params_buffer(config);

        // Create bind group
        let bind_group = self.ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Benchmark Bind Group"),
            layout: &pipeline.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        // Warmup runs
        for _ in 0..config.warmup_iterations {
            self.dispatch(&pipeline, &bind_group, config);
        }

        // Timed runs
        let mut timings = Vec::with_capacity(config.measurement_iterations as usize);

        for _ in 0..config.measurement_iterations {
            let start = Instant::now();
            self.dispatch(&pipeline, &bind_group, config);
            timings.push(start.elapsed());
        }

        // Create result
        Ok(BenchmarkResult::from_timings(
            Backend::WebGPU,
            operation,
            config.workgroup_size,
            config.total_threads(),
            config.ops_per_thread,
            &timings,
            None, // WebGPU doesn't expose GPU clock
        ))
    }

    /// Create input buffer with random data
    fn create_input_buffer(&self, seed: u32) -> wgpu::Buffer {
        let data: Vec<u32> = (0..16u32)
            .map(|i| seed.wrapping_add(i).wrapping_mul(0x9E3779B9))
            .collect();

        self.ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Input Buffer"),
                contents: bytemuck::cast_slice(&data),
                usage: wgpu::BufferUsages::STORAGE,
            })
    }

    /// Create output buffer
    fn create_output_buffer(&self, count: usize) -> wgpu::Buffer {
        self.ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: (count * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    }

    /// Create parameters buffer
    fn create_params_buffer(&self, config: &BenchmarkConfig) -> wgpu::Buffer {
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
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

        self.ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Params Buffer"),
                contents: bytemuck::cast_slice(&[params]),
                usage: wgpu::BufferUsages::UNIFORM,
            })
    }

    /// Dispatch the compute shader
    fn dispatch(&self, pipeline: &WebGpuPipeline, bind_group: &wgpu::BindGroup, config: &BenchmarkConfig) {
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Benchmark Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Benchmark Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&pipeline.pipeline);
            compute_pass.set_bind_group(0, bind_group, &[]);
            compute_pass.dispatch_workgroups(config.num_workgroups, 1, 1);
        }

        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        self.ctx.device.poll(wgpu::Maintain::Wait);
    }
}

/// Map operation to WGSL entry point name
fn operation_to_entry_point(operation: Operation) -> &'static str {
    match operation {
        Operation::U32Add => "bench_u32_add",
        Operation::U64AddNative => "bench_u64_add", // Not available in WebGPU
        Operation::U64AddEmulated => "bench_u64_add",
        Operation::FieldMul => "bench_field_mul",
        Operation::FieldAdd => "bench_field_add",
        Operation::U256Add => "bench_u256_add",
        Operation::MersenneFieldAdd => "bench_mersenne_field_add",
        Operation::MersenneFieldMul => "bench_mersenne_field_mul",
    }
}
