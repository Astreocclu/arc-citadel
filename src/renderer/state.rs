//! Render state types - frozen snapshots for rendering.

use glam::Vec2;
use crate::core::types::EntityId;

/// Frozen snapshot of simulation state for rendering.
/// Immutable once created - no references back to simulation.
#[derive(Clone)]
pub struct RenderState {
    pub tick: u64,
    pub entities: Vec<RenderEntity>,      // Shape-based entities
    pub sprites: Vec<SpriteEntity>,       // Textured sprites
    pub camera: CameraState,
}

/// Sprite render data (for textured entities).
#[derive(Clone, Copy)]
pub struct SpriteEntity {
    pub id: EntityId,
    pub position: Vec2,
    pub uv_rect: [f32; 4],  // [u, v, width, height] normalized
    pub color: [u8; 4],     // RGBA tint
    pub rotation: f32,
    pub scale: f32,
    pub flip_x: bool,
    pub flip_y: bool,
    pub z_order: i32,
}

impl SpriteEntity {
    /// Create a simple sprite using full texture.
    pub fn simple(id: EntityId, position: Vec2, scale: f32) -> Self {
        Self {
            id,
            position,
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            color: [255, 255, 255, 255],
            rotation: 0.0,
            scale,
            flip_x: false,
            flip_y: false,
            z_order: 0,
        }
    }
}

/// Minimal render data per entity.
#[derive(Clone, Copy)]
pub struct RenderEntity {
    pub id: EntityId,
    pub position: Vec2,
    pub facing: f32,
    pub shape: ShapeType,
    pub color: Color,
    pub scale: f32,
    pub z_order: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u32)]
pub enum ShapeType {
    Circle = 0,
    Rectangle = 1,
    Triangle = 2,
    Hexagon = 3,
}

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Pack color into u32 (RGBA8 format).
    pub fn to_u32(&self) -> u32 {
        let r = (self.r.clamp(0.0, 1.0) * 255.0) as u32;
        let g = (self.g.clamp(0.0, 1.0) * 255.0) as u32;
        let b = (self.b.clamp(0.0, 1.0) * 255.0) as u32;
        let a = (self.a.clamp(0.0, 1.0) * 255.0) as u32;
        (r << 24) | (g << 16) | (b << 8) | a
    }

    /// Unpack color from u32 (RGBA8 format).
    pub fn from_u32(packed: u32) -> Self {
        Self {
            r: ((packed >> 24) & 0xFF) as f32 / 255.0,
            g: ((packed >> 16) & 0xFF) as f32 / 255.0,
            b: ((packed >> 8) & 0xFF) as f32 / 255.0,
            a: (packed & 0xFF) as f32 / 255.0,
        }
    }

    // Common colors
    pub const WHITE: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::rgba(0.0, 0.0, 0.0, 1.0);
    pub const RED: Color = Color::rgba(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Color = Color::rgba(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Color = Color::rgba(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: Color = Color::rgba(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: Color = Color::rgba(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: Color = Color::rgba(1.0, 0.0, 1.0, 1.0);
}

#[derive(Clone, Copy, Debug)]
pub struct CameraState {
    pub center: Vec2,
    pub zoom: f32,           // World units per screen pixel (lower = zoomed in)
    pub viewport_size: Vec2, // Screen dimensions in pixels
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            zoom: 1.0,
            viewport_size: Vec2::new(1280.0, 720.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_u32() {
        let white = Color::rgba(1.0, 1.0, 1.0, 1.0);
        assert_eq!(white.to_u32(), 0xFFFFFFFF);

        let red = Color::rgba(1.0, 0.0, 0.0, 1.0);
        assert_eq!(red.to_u32(), 0xFF0000FF);

        let transparent_blue = Color::rgba(0.0, 0.0, 1.0, 0.5);
        let packed = transparent_blue.to_u32();
        assert_eq!(packed >> 24, 0);           // R = 0
        assert_eq!((packed >> 16) & 0xFF, 0);  // G = 0
        assert_eq!((packed >> 8) & 0xFF, 255); // B = 255
        assert!((packed & 0xFF) > 100 && (packed & 0xFF) < 140); // A ≈ 127
    }

    #[test]
    fn test_color_roundtrip() {
        let original = Color::rgba(0.5, 0.25, 0.75, 1.0);
        let packed = original.to_u32();
        let unpacked = Color::from_u32(packed);

        // Allow for quantization error (1/255 ≈ 0.004)
        assert!((original.r - unpacked.r).abs() < 0.01);
        assert!((original.g - unpacked.g).abs() < 0.01);
        assert!((original.b - unpacked.b).abs() < 0.01);
        assert!((original.a - unpacked.a).abs() < 0.01);
    }
}
