//! Instance data for GPU instancing.

use bytemuck::{Pod, Zeroable};

/// GPU instance data for shapes. 24 bytes, tightly packed.
/// Each instance represents one shape to render.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ShapeInstance {
    /// World position (x, y).
    pub position: [f32; 2], // 8 bytes
    /// Rotation in radians.
    pub rotation: f32, // 4 bytes
    /// Uniform scale factor.
    pub scale: f32, // 4 bytes
    /// Packed RGBA color (see Color::to_u32).
    pub color: u32, // 4 bytes
    /// Shape type (0=Circle, 1=Rectangle, 2=Triangle, 3=Hexagon).
    pub shape_type: u32, // 4 bytes
}

impl ShapeInstance {
    /// Create a new shape instance.
    pub fn new(position: [f32; 2], rotation: f32, scale: f32, color: u32, shape_type: u32) -> Self {
        Self {
            position,
            rotation,
            scale,
            color,
            shape_type,
        }
    }

    /// Vertex buffer layout descriptor for instancing.
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ShapeInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // rotation
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
                // scale
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
                // color
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32,
                },
                // shape_type
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_size() {
        assert_eq!(std::mem::size_of::<ShapeInstance>(), 24);
    }

    #[test]
    fn test_instance_alignment() {
        assert_eq!(std::mem::align_of::<ShapeInstance>(), 4);
    }
}
