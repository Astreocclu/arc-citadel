//! GPU abstractions for wgpu.

pub mod buffers;
pub mod context;
pub mod pipeline;
pub mod texture;

pub use buffers::{BatchedInstances, ShapeBuffers, ShapeGeometry};
pub use context::GpuContext;
pub use pipeline::ShapePipeline;
pub use texture::{Texture, TextureError};
