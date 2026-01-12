//! Metal device and queue management

use crate::BenchmarkError;
use metal::{CommandQueue, Device, Library};
use std::path::Path;

/// Metal GPU context
pub struct MetalContext {
    pub device: Device,
    pub command_queue: CommandQueue,
    pub library: Option<Library>,
}

impl MetalContext {
    /// Create a new Metal context with the default GPU
    pub fn new() -> Result<Self, BenchmarkError> {
        let device = Device::system_default().ok_or(BenchmarkError::NoDevice)?;

        let command_queue = device.new_command_queue();

        Ok(Self {
            device,
            command_queue,
            library: None,
        })
    }

    /// Load a metallib from the given path
    pub fn load_library(&mut self, path: &Path) -> Result<(), BenchmarkError> {
        let library = self
            .device
            .new_library_with_file(path)
            .map_err(|e| BenchmarkError::ShaderCompilation(format!("{:?}", e)))?;

        self.library = Some(library);
        Ok(())
    }

    /// Load a metallib from embedded bytes
    pub fn load_library_data(&mut self, data: &[u8]) -> Result<(), BenchmarkError> {
        let library = self
            .device
            .new_library_with_data(data)
            .map_err(|e| BenchmarkError::ShaderCompilation(format!("{:?}", e)))?;

        self.library = Some(library);
        Ok(())
    }

    /// Get device name
    pub fn device_name(&self) -> String {
        self.device.name().to_string()
    }

    /// Check if the device supports native 64-bit integers
    pub fn supports_native_u64(&self) -> bool {
        // All Apple Silicon GPUs support native 64-bit integers
        // Intel GPUs on older Macs may not
        true // Simplified for now
    }
}
