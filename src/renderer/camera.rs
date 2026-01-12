//! Camera system with coordinate transformations.

use super::state::CameraState;
use glam::{Mat4, Vec2};

impl CameraState {
    /// Create a new camera centered at origin.
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            center: Vec2::ZERO,
            zoom: 1.0,
            viewport_size: Vec2::new(viewport_width, viewport_height),
        }
    }

    /// Convert world coordinates to screen coordinates.
    /// Screen origin is top-left, Y increases downward.
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let relative = world_pos - self.center;
        let scaled = relative / self.zoom;
        Vec2::new(
            self.viewport_size.x / 2.0 + scaled.x,
            self.viewport_size.y / 2.0 - scaled.y, // Y-flip for screen coords
        )
    }

    /// Convert screen coordinates to world coordinates.
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let centered = Vec2::new(
            screen_pos.x - self.viewport_size.x / 2.0,
            self.viewport_size.y / 2.0 - screen_pos.y, // Y-flip
        );
        self.center + centered * self.zoom
    }

    /// Generate orthographic view-projection matrix for GPU.
    /// This matrix transforms world coordinates to clip space (-1 to 1).
    pub fn view_projection_matrix(&self) -> Mat4 {
        let half_width = self.viewport_size.x * self.zoom / 2.0;
        let half_height = self.viewport_size.y * self.zoom / 2.0;

        Mat4::orthographic_rh(
            self.center.x - half_width,  // left
            self.center.x + half_width,  // right
            self.center.y - half_height, // bottom
            self.center.y + half_height, // top
            -1000.0,                     // near
            1000.0,                      // far
        )
    }

    /// Pan camera by delta in world units.
    pub fn pan(&mut self, delta: Vec2) {
        self.center += delta;
    }

    /// Set camera center position.
    pub fn set_center(&mut self, center: Vec2) {
        self.center = center;
    }

    /// Zoom camera by factor, optionally centering on a screen position.
    /// factor < 1.0 zooms in, factor > 1.0 zooms out.
    pub fn zoom_by(&mut self, factor: f32) {
        self.zoom *= factor;
        self.zoom = self.zoom.clamp(0.1, 100.0);
    }

    /// Zoom toward a specific screen position (e.g., mouse cursor).
    /// This keeps the world point under the cursor fixed.
    pub fn zoom_toward(&mut self, screen_pos: Vec2, factor: f32) {
        let world_before = self.screen_to_world(screen_pos);
        self.zoom *= factor;
        self.zoom = self.zoom.clamp(0.1, 100.0);
        let world_after = self.screen_to_world(screen_pos);
        self.center += world_before - world_after;
    }

    /// Update viewport size (call on window resize).
    pub fn set_viewport_size(&mut self, width: f32, height: f32) {
        self.viewport_size = Vec2::new(width, height);
    }

    /// Get the visible world bounds (min, max).
    pub fn visible_bounds(&self) -> (Vec2, Vec2) {
        let half_width = self.viewport_size.x * self.zoom / 2.0;
        let half_height = self.viewport_size.y * self.zoom / 2.0;
        (
            Vec2::new(self.center.x - half_width, self.center.y - half_height),
            Vec2::new(self.center.x + half_width, self.center.y + half_height),
        )
    }

    /// Check if a world point is visible on screen.
    pub fn is_visible(&self, world_pos: Vec2) -> bool {
        let (min, max) = self.visible_bounds();
        world_pos.x >= min.x && world_pos.x <= max.x && world_pos.y >= min.y && world_pos.y <= max.y
    }

    /// Check if a world point with radius is visible (for culling).
    pub fn is_visible_with_radius(&self, world_pos: Vec2, radius: f32) -> bool {
        let (min, max) = self.visible_bounds();
        world_pos.x + radius >= min.x
            && world_pos.x - radius <= max.x
            && world_pos.y + radius >= min.y
            && world_pos.y - radius <= max.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_screen_roundtrip() {
        let camera = CameraState {
            center: Vec2::new(100.0, 200.0),
            zoom: 2.0,
            viewport_size: Vec2::new(800.0, 600.0),
        };

        let world = Vec2::new(150.0, 250.0);
        let screen = camera.world_to_screen(world);
        let back = camera.screen_to_world(screen);

        assert!((world - back).length() < 0.001);
    }

    #[test]
    fn test_center_maps_to_screen_center() {
        let camera = CameraState {
            center: Vec2::new(50.0, 75.0),
            zoom: 1.0,
            viewport_size: Vec2::new(800.0, 600.0),
        };

        let screen = camera.world_to_screen(camera.center);
        assert!((screen.x - 400.0).abs() < 0.001);
        assert!((screen.y - 300.0).abs() < 0.001);
    }

    #[test]
    fn test_zoom_affects_visible_bounds() {
        let mut camera = CameraState::new(800.0, 600.0);

        let (min1, max1) = camera.visible_bounds();
        let width1 = max1.x - min1.x;

        camera.zoom_by(2.0); // Zoom out
        let (min2, max2) = camera.visible_bounds();
        let width2 = max2.x - min2.x;

        assert!((width2 - width1 * 2.0).abs() < 0.001);
    }

    #[test]
    fn test_zoom_toward_keeps_point_fixed() {
        let mut camera = CameraState::new(800.0, 600.0);
        let screen_pos = Vec2::new(200.0, 150.0);

        let world_before = camera.screen_to_world(screen_pos);
        camera.zoom_toward(screen_pos, 0.5); // Zoom in
        let world_after = camera.screen_to_world(screen_pos);

        assert!((world_before - world_after).length() < 0.001);
    }

    #[test]
    fn test_visibility_check() {
        let camera = CameraState {
            center: Vec2::ZERO,
            zoom: 1.0,
            viewport_size: Vec2::new(100.0, 100.0),
        };

        assert!(camera.is_visible(Vec2::ZERO));
        assert!(camera.is_visible(Vec2::new(40.0, 40.0)));
        assert!(!camera.is_visible(Vec2::new(100.0, 0.0)));
    }
}
