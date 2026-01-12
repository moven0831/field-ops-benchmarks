//! WebGPU backend (cross-platform)

mod device;
mod pipeline;
mod runner;

pub use device::WebGpuContext;
pub use pipeline::WebGpuPipeline;
pub use runner::WebGpuRunner;
