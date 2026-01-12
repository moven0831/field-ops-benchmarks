//! WebGPU device and queue management

use crate::BenchmarkError;
use wgpu::{Adapter, Device, Instance, Queue};

/// WebGPU context
pub struct WebGpuContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl WebGpuContext {
    /// Create a new WebGPU context
    pub fn new() -> Result<Self, BenchmarkError> {
        pollster::block_on(Self::new_async())
    }

    async fn new_async() -> Result<Self, BenchmarkError> {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(BenchmarkError::NoDevice)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Benchmark Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .map_err(|_| BenchmarkError::NoDevice)?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }

    /// Get device name
    pub fn device_name(&self) -> String {
        let info = self.adapter.get_info();
        format!("{} ({})", info.name, info.backend.to_str())
    }

    /// Check if timestamp queries are supported
    pub fn supports_timestamp_queries(&self) -> bool {
        self.adapter
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY)
    }
}
