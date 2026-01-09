//! GPU abstractions for wgpu.

pub mod buffers;
pub mod context;
pub mod pipeline;
pub mod sprite_pipeline;
pub mod texture;

pub use buffers::{BatchedInstances, ShapeBuffers, ShapeGeometry};
pub use context::GpuContext;
pub use pipeline::ShapePipeline;
pub use sprite_pipeline::SpritePipeline;
pub use texture::{Texture, TextureError};
