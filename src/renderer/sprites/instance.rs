//! GPU sprite instance data.

use bytemuck::{Pod, Zeroable};

/// GPU sprite instance. 32 bytes, 16-byte aligned.
#[repr(C, align(16))]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SpriteInstance {
    /// World position (x, y).
    pub position: [f32; 2],
    /// Packed UV offset (16-bit u, 16-bit v normalized to 0-65535).
    pub uv_offset: u32,
    /// Packed UV size (16-bit width, 16-bit height normalized).
    pub uv_size: u32,
    /// Color tint as RGBA8 packed.
    pub color_tint: u32,
    /// Packed transform: rotation (16-bit) + scale (8-bit) + flags (8-bit).
    pub transform_flags: u32,
    /// Padding for 16-byte alignment.
    pub _padding: [u32; 2],
}

impl SpriteInstance {
    /// Create a new sprite instance.
    pub fn new(
        position: [f32; 2],
        uv_offset: [f32; 2],
        uv_size: [f32; 2],
        color: [u8; 4],
        rotation: f32,
        scale: f32,
        flip_x: bool,
        flip_y: bool,
    ) -> Self {
        // Pack UV offset (normalized 0-1 to 0-65535)
        let u_off = (uv_offset[0].clamp(0.0, 1.0) * 65535.0) as u32;
        let v_off = (uv_offset[1].clamp(0.0, 1.0) * 65535.0) as u32;
        let uv_offset_packed = u_off | (v_off << 16);

        // Pack UV size
        let u_size = (uv_size[0].clamp(0.0, 1.0) * 65535.0) as u32;
        let v_size = (uv_size[1].clamp(0.0, 1.0) * 65535.0) as u32;
        let uv_size_packed = u_size | (v_size << 16);

        // Pack color as RGBA8
        let color_tint = (color[0] as u32) << 24
            | (color[1] as u32) << 16
            | (color[2] as u32) << 8
            | color[3] as u32;

        // Pack transform: rotation (16-bit) + scale (8-bit) + flags (8-bit)
        // Rotation: 0-65535 maps to 0-TAU radians
        let rotation_packed = ((rotation / std::f32::consts::TAU).fract().abs() * 65535.0) as u32;
        // Scale: 0-255 maps to 0-25.5 (scale factor in tenths)
        let scale_packed = ((scale / 25.5).clamp(0.0, 1.0) * 255.0) as u32;
        // Flags: bit 0 = flip_x, bit 1 = flip_y
        let flags = (flip_x as u32) | ((flip_y as u32) << 1);
        let transform_flags = (rotation_packed << 16) | (scale_packed << 8) | flags;

        Self {
            position,
            uv_offset: uv_offset_packed,
            uv_size: uv_size_packed,
            color_tint,
            transform_flags,
            _padding: [0; 2],
        }
    }

    /// Create a simple sprite instance without rotation or flip.
    pub fn simple(position: [f32; 2], uv_offset: [f32; 2], uv_size: [f32; 2], scale: f32) -> Self {
        Self::new(
            position,
            uv_offset,
            uv_size,
            [255, 255, 255, 255],
            0.0,
            scale,
            false,
            false,
        )
    }

    /// Vertex buffer layout descriptor.
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: 32, // Fixed 32 bytes
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // position: vec2<f32>
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // uv_offset: u32
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Uint32,
                },
                // uv_size: u32
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
                // color_tint: u32
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32,
                },
                // transform_flags: u32
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
    fn test_sprite_instance_size() {
        assert_eq!(std::mem::size_of::<SpriteInstance>(), 32);
    }

    #[test]
    fn test_sprite_instance_alignment() {
        assert_eq!(std::mem::align_of::<SpriteInstance>(), 16);
    }

    #[test]
    fn test_sprite_instance_packing() {
        let sprite = SpriteInstance::new(
            [100.0, 200.0],
            [0.25, 0.5],
            [0.125, 0.125],
            [255, 128, 64, 255],
            std::f32::consts::FRAC_PI_2,
            2.0,
            true,
            false,
        );

        assert_eq!(sprite.position, [100.0, 200.0]);
        assert!(sprite.color_tint != 0);
        assert!(sprite.transform_flags & 1 == 1); // flip_x set
        assert!(sprite.transform_flags & 2 == 0); // flip_y not set
    }

    #[test]
    fn test_sprite_instance_simple() {
        let sprite = SpriteInstance::simple([50.0, 100.0], [0.0, 0.0], [0.5, 0.5], 1.0);

        assert_eq!(sprite.position, [50.0, 100.0]);
        // Color should be white (0xFFFFFFFF)
        assert_eq!(sprite.color_tint, 0xFFFFFFFF);
        // No flip flags should be set
        assert_eq!(sprite.transform_flags & 0x3, 0);
    }

    #[test]
    fn test_sprite_instance_uv_packing() {
        let sprite = SpriteInstance::new(
            [0.0, 0.0],
            [0.5, 0.25],
            [0.125, 0.0625],
            [255, 255, 255, 255],
            0.0,
            1.0,
            false,
            false,
        );

        // UV offset: 0.5 -> 32767, 0.25 -> 16383 (approximately)
        let u_off = sprite.uv_offset & 0xFFFF;
        let v_off = (sprite.uv_offset >> 16) & 0xFFFF;
        // Should be approximately half and quarter of 65535
        assert!(u_off > 32000 && u_off < 33000); // ~32767
        assert!(v_off > 16000 && v_off < 17000); // ~16383
    }
}
