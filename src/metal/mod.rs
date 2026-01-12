//! Metal GPU backend for macOS/iOS

mod device;
mod pipeline;
mod runner;

pub use device::MetalContext;
pub use pipeline::MetalPipeline;
pub use runner::MetalRunner;
