//! Camera system for 2D rendering
//!
//! Handles viewport positioning, zoom, and coordinate transforms.

use crate::core::types::Vec2;

/// Scale factor: 1 simulation unit = PIXELS_PER_UNIT pixels at zoom 1.0
pub const PIXELS_PER_UNIT: f32 = 4.0;

/// Camera configuration
pub struct Camera {
    /// Center position in world coordinates
    pub position: Vec2,
    /// Zoom level (1.0 = normal, 2.0 = 2x magnification)
    pub zoom: f32,
    /// Viewport size in pixels
    pub viewport_size: (f32, f32),
    /// Optional world bounds for clamping
    pub world_bounds: Option<WorldBounds>,
}

/// Axis-aligned bounding box for world limits
#[derive(Debug, Clone, Copy)]
pub struct WorldBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl Camera {
    /// Create a new camera centered at origin
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            zoom: 1.0,
            viewport_size: (viewport_width, viewport_height),
            world_bounds: None,
        }
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> (f32, f32) {
        let relative = Vec2::new(
            world_pos.x - self.position.x,
            world_pos.y - self.position.y,
        );
        let scaled_x = relative.x * self.zoom * PIXELS_PER_UNIT;
        let scaled_y = relative.y * self.zoom * PIXELS_PER_UNIT;
        let screen_x = scaled_x + self.viewport_size.0 / 2.0;
        let screen_y = scaled_y + self.viewport_size.1 / 2.0;
        (screen_x, screen_y)
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_x: f32, screen_y: f32) -> Vec2 {
        let centered_x = screen_x - self.viewport_size.0 / 2.0;
        let centered_y = screen_y - self.viewport_size.1 / 2.0;
        let world_x = centered_x / (self.zoom * PIXELS_PER_UNIT) + self.position.x;
        let world_y = centered_y / (self.zoom * PIXELS_PER_UNIT) + self.position.y;
        Vec2::new(world_x, world_y)
    }

    /// Pan the camera by a delta in world units
    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.position.x += dx;
        self.position.y += dy;
        self.clamp_to_bounds();
    }

    /// Adjust zoom level, clamped to [0.1, 10.0]
    pub fn adjust_zoom(&mut self, delta: f32) {
        self.zoom = (self.zoom * (1.0 + delta)).clamp(0.1, 10.0);
    }

    /// Update world bounds based on entity positions
    pub fn update_bounds_from_entities(&mut self, positions: &[Vec2], padding: f32) {
        if positions.is_empty() {
            self.world_bounds = None;
            return;
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for pos in positions {
            min_x = min_x.min(pos.x);
            min_y = min_y.min(pos.y);
            max_x = max_x.max(pos.x);
            max_y = max_y.max(pos.y);
        }

        self.world_bounds = Some(WorldBounds {
            min_x: min_x - padding,
            min_y: min_y - padding,
            max_x: max_x + padding,
            max_y: max_y + padding,
        });
    }

    /// Center camera on the midpoint of all entities
    pub fn center_on_entities(&mut self, positions: &[Vec2]) {
        if positions.is_empty() {
            return;
        }

        let sum_x: f32 = positions.iter().map(|p| p.x).sum();
        let sum_y: f32 = positions.iter().map(|p| p.y).sum();
        let count = positions.len() as f32;

        self.position = Vec2::new(sum_x / count, sum_y / count);
    }

    /// Clamp camera position to world bounds if set
    fn clamp_to_bounds(&mut self) {
        if let Some(bounds) = self.world_bounds {
            let half_view_x = self.viewport_size.0 / (2.0 * self.zoom * PIXELS_PER_UNIT);
            let half_view_y = self.viewport_size.1 / (2.0 * self.zoom * PIXELS_PER_UNIT);

            self.position.x = self.position.x.clamp(
                bounds.min_x + half_view_x,
                bounds.max_x - half_view_x,
            );
            self.position.y = self.position.y.clamp(
                bounds.min_y + half_view_y,
                bounds.max_y - half_view_y,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_screen_center() {
        let camera = Camera::new(800.0, 600.0);
        // Camera at origin, entity at origin -> should be at screen center
        let (sx, sy) = camera.world_to_screen(Vec2::new(0.0, 0.0));
        assert_eq!(sx, 400.0);
        assert_eq!(sy, 300.0);
    }

    #[test]
    fn test_world_to_screen_offset() {
        let camera = Camera::new(800.0, 600.0);
        // Entity at (10, 0) in world -> 10 * 4 = 40 pixels right of center
        let (sx, sy) = camera.world_to_screen(Vec2::new(10.0, 0.0));
        assert_eq!(sx, 440.0);
        assert_eq!(sy, 300.0);
    }

    #[test]
    fn test_zoom_clamp() {
        let mut camera = Camera::new(800.0, 600.0);
        camera.adjust_zoom(100.0); // Try to zoom way in
        assert!(camera.zoom <= 10.0);
        camera.adjust_zoom(-100.0); // Try to zoom way out
        assert!(camera.zoom >= 0.1);
    }
}
