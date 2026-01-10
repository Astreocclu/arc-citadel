# 2D Renderer Proof-of-Concept Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a macroquad-based 2D renderer as a separate binary that visualizes simulation entities as colored rectangles with pan/zoom camera controls.

**Architecture:** Separate binary (`src/bin/renderer.rs`) consumes read-only snapshots from `World`. A `src/render/` module provides `RenderEntity` extraction and camera logic. Zero-allocation rendering after initial warmup via buffer reuse.

**Tech Stack:** macroquad 0.4, existing arc-citadel library

---

## Task 1: Add macroquad Dependency

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add macroquad to dependencies**

Edit `Cargo.toml` to add macroquad after the existing dependencies:

```toml
macroquad = "0.4"
```

Add it after line 25 (after `rayon`).

**Step 2: Verify dependency resolves**

Run: `cargo check`
Expected: Compiles successfully (may download dependencies)

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add macroquad dependency for 2D renderer"
```

---

## Task 2: Create Render Module Structure

**Files:**
- Create: `src/render/mod.rs`
- Modify: `src/lib.rs`

**Step 1: Create the render module directory and mod.rs**

Create `src/render/mod.rs`:

```rust
//! 2D Rendering system for Arc Citadel
//!
//! Provides visual representation of simulation state.
//! This module is READ-ONLY - it never modifies simulation state.

pub mod camera;
pub mod colors;

use crate::core::types::{Species, Vec2};
use crate::entity::body::BodyState;
use crate::ecs::world::World;

/// Lightweight snapshot of an entity for rendering
#[derive(Debug, Clone)]
pub struct RenderEntity {
    pub position: Vec2,
    pub species: Species,
    pub health: f32,
    pub fatigue: f32,
}

/// Collects all renderable entities from the world into a reusable buffer.
/// Call this once per frame, passing the same buffer to avoid allocations.
pub fn collect_render_entities(world: &World, buffer: &mut Vec<RenderEntity>) {
    buffer.clear();

    // Collect living humans
    for i in 0..world.humans.ids.len() {
        if world.humans.alive[i] {
            let body = &world.humans.body_states[i];
            buffer.push(RenderEntity {
                position: world.humans.positions[i],
                species: Species::Human,
                health: body.overall_health,
                fatigue: body.fatigue,
            });
        }
    }

    // Collect living orcs
    for i in 0..world.orcs.ids.len() {
        if world.orcs.alive[i] {
            let body = &world.orcs.body_states[i];
            buffer.push(RenderEntity {
                position: world.orcs.positions[i],
                species: Species::Orc,
                health: body.overall_health,
                fatigue: body.fatigue,
            });
        }
    }
}
```

**Step 2: Export render module from lib.rs**

Add to `src/lib.rs` after line 18 (after `pub mod city;`):

```rust
pub mod render;
```

**Step 3: Verify module compiles**

Run: `cargo check`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/render/mod.rs src/lib.rs
git commit -m "feat(render): add render module with RenderEntity and collection"
```

---

## Task 3: Create Camera Module

**Files:**
- Create: `src/render/camera.rs`

**Step 1: Write camera implementation**

Create `src/render/camera.rs`:

```rust
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
```

**Step 2: Verify camera module compiles**

Run: `cargo test --lib render::camera`
Expected: 3 tests pass

**Step 3: Commit**

```bash
git add src/render/camera.rs
git commit -m "feat(render): add camera with pan, zoom, and coordinate transforms"
```

---

## Task 4: Create Colors Module

**Files:**
- Create: `src/render/colors.rs`

**Step 1: Write species color mapping**

Create `src/render/colors.rs`:

```rust
//! Color definitions for species and visual states

use crate::core::types::Species;

/// RGBA color (0.0 to 1.0 per channel)
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Darken color by a factor (0.0 = black, 1.0 = unchanged)
    pub fn darken(&self, factor: f32) -> Self {
        Self {
            r: self.r * factor,
            g: self.g * factor,
            b: self.b * factor,
            a: self.a,
        }
    }
}

/// Background color for the renderer
pub const BACKGROUND: Color = Color::new(0.1, 0.1, 0.12, 1.0);

/// Get the base color for a species
pub fn species_color(species: Species) -> Color {
    match species {
        // Playable races
        Species::Human => Color::new(0.2, 0.6, 0.9, 1.0),      // Blue
        Species::Dwarf => Color::new(0.8, 0.5, 0.2, 1.0),      // Brown/orange
        Species::Elf => Color::new(0.3, 0.9, 0.5, 1.0),        // Light green
        Species::Orc => Color::new(0.1, 0.7, 0.2, 1.0),        // Dark green

        // Humanoid monsters
        Species::Kobold => Color::new(0.6, 0.4, 0.2, 1.0),     // Tan
        Species::Gnoll => Color::new(0.7, 0.5, 0.3, 1.0),      // Yellowish brown
        Species::Lizardfolk => Color::new(0.2, 0.5, 0.3, 1.0), // Swamp green
        Species::Hobgoblin => Color::new(0.8, 0.4, 0.1, 1.0),  // Orange-red
        Species::Ogre => Color::new(0.5, 0.4, 0.3, 1.0),       // Muddy brown
        Species::Goblin => Color::new(0.4, 0.6, 0.2, 1.0),     // Yellow-green

        // Mythical humanoids
        Species::Harpy => Color::new(0.8, 0.7, 0.5, 1.0),      // Feather tan
        Species::Centaur => Color::new(0.6, 0.4, 0.3, 1.0),    // Chestnut
        Species::Minotaur => Color::new(0.5, 0.3, 0.2, 1.0),   // Dark brown
        Species::Satyr => Color::new(0.7, 0.5, 0.4, 1.0),      // Russet

        // Nature spirits
        Species::Dryad => Color::new(0.4, 0.7, 0.3, 1.0),      // Forest green
        Species::Fey => Color::new(0.7, 0.5, 0.9, 1.0),        // Purple/violet

        // Large monsters
        Species::Troll => Color::new(0.3, 0.5, 0.3, 1.0),      // Moss green
        Species::StoneGiants => Color::new(0.5, 0.5, 0.5, 1.0),// Gray

        // Magical/elemental
        Species::AbyssalDemons => Color::new(0.8, 0.1, 0.1, 1.0), // Blood red
        Species::Elemental => Color::new(0.9, 0.6, 0.2, 1.0),  // Fiery orange
        Species::Golem => Color::new(0.4, 0.4, 0.5, 1.0),      // Stone gray

        // Aquatic
        Species::Merfolk => Color::new(0.2, 0.6, 0.8, 1.0),    // Ocean blue
        Species::Naga => Color::new(0.3, 0.7, 0.6, 1.0),       // Teal

        // Undead
        Species::Revenant => Color::new(0.4, 0.4, 0.5, 1.0),   // Ashen gray
        Species::Vampire => Color::new(0.6, 0.1, 0.2, 1.0),    // Dark crimson

        // Lycanthropes
        Species::Lupine => Color::new(0.5, 0.4, 0.3, 1.0),     // Fur brown
    }
}

/// Modulate color based on entity health (lower health = more red tint)
pub fn health_tint(base: Color, health: f32) -> Color {
    let health_clamped = health.clamp(0.0, 1.0);
    // Interpolate toward red as health decreases
    Color {
        r: base.r + (1.0 - base.r) * (1.0 - health_clamped) * 0.5,
        g: base.g * health_clamped,
        b: base.b * health_clamped,
        a: base.a,
    }
}

/// Modulate color based on fatigue (higher fatigue = darker)
pub fn fatigue_tint(base: Color, fatigue: f32) -> Color {
    let fatigue_clamped = fatigue.clamp(0.0, 1.0);
    let brightness = 1.0 - fatigue_clamped * 0.4; // Max 40% darkening
    base.darken(brightness)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_species_colors_unique() {
        // Ensure at least a few key species have distinct colors
        let human = species_color(Species::Human);
        let orc = species_color(Species::Orc);
        let dwarf = species_color(Species::Dwarf);

        // Colors should be different (not exactly equal)
        assert!(human.r != orc.r || human.g != orc.g || human.b != orc.b);
        assert!(human.r != dwarf.r || human.g != dwarf.g || human.b != dwarf.b);
    }

    #[test]
    fn test_health_tint_full_health() {
        let base = Color::new(0.5, 0.5, 0.5, 1.0);
        let tinted = health_tint(base, 1.0);
        // Full health should be close to original
        assert!((tinted.r - base.r).abs() < 0.01);
    }

    #[test]
    fn test_fatigue_darkens() {
        let base = Color::new(1.0, 1.0, 1.0, 1.0);
        let tired = fatigue_tint(base, 1.0);
        // Max fatigue should darken by 40%
        assert!((tired.r - 0.6).abs() < 0.01);
    }
}
```

**Step 2: Verify colors module compiles and tests pass**

Run: `cargo test --lib render::colors`
Expected: 3 tests pass

**Step 3: Commit**

```bash
git add src/render/colors.rs
git commit -m "feat(render): add species color mapping with health/fatigue tints"
```

---

## Task 5: Create Renderer Binary

**Files:**
- Create: `src/bin/renderer.rs`

**Step 1: Write the main renderer binary**

Create `src/bin/renderer.rs`:

```rust
//! Arc Citadel 2D Renderer
//!
//! Visualizes simulation state in real-time.
//! Controls:
//! - WASD / Arrow keys: Pan camera
//! - Mouse wheel: Zoom in/out
//! - Escape: Quit

use macroquad::prelude::*;

use arc_citadel::core::types::Vec2 as SimVec2;
use arc_citadel::ecs::world::{World, Abundance};
use arc_citadel::render::{collect_render_entities, RenderEntity};
use arc_citadel::render::camera::Camera;
use arc_citadel::render::colors::{self, species_color, health_tint, fatigue_tint, Color as RenderColor};
use arc_citadel::simulation::tick::run_simulation_tick;

/// Pan speed in world units per second
const PAN_SPEED: f32 = 100.0;
/// Zoom speed multiplier per scroll tick
const ZOOM_SPEED: f32 = 0.1;
/// Base entity size in pixels at zoom 1.0
const ENTITY_BASE_SIZE: f32 = 8.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Arc Citadel Renderer".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Initialize world with test entities
    let mut world = setup_test_world();

    // Initialize camera
    let mut camera = Camera::new(screen_width(), screen_height());

    // Reusable buffer for render entities (zero allocation after first frame)
    let mut render_buffer: Vec<RenderEntity> = Vec::with_capacity(1000);

    // Initial camera centering
    collect_render_entities(&world, &mut render_buffer);
    let positions: Vec<SimVec2> = render_buffer.iter().map(|e| e.position).collect();
    camera.center_on_entities(&positions);
    camera.update_bounds_from_entities(&positions, 50.0);

    let mut first_frame = true;

    loop {
        // Handle input
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        handle_camera_input(&mut camera);

        // Update viewport size on resize
        camera.viewport_size = (screen_width(), screen_height());

        // Run simulation tick
        run_simulation_tick(&mut world);

        // Collect entities for rendering
        collect_render_entities(&world, &mut render_buffer);

        // Update camera bounds periodically (not every frame for performance)
        if world.current_tick % 60 == 0 || first_frame {
            let positions: Vec<SimVec2> = render_buffer.iter().map(|e| e.position).collect();
            camera.update_bounds_from_entities(&positions, 50.0);
            first_frame = false;
        }

        // Render
        clear_background(to_macroquad_color(colors::BACKGROUND));

        // Draw entities
        for entity in &render_buffer {
            draw_entity(entity, &camera);
        }

        // Draw UI overlay
        draw_ui(&world, render_buffer.len());

        next_frame().await;
    }
}

fn handle_camera_input(camera: &mut Camera) {
    let dt = get_frame_time();
    let pan_amount = PAN_SPEED * dt / camera.zoom;

    // Pan with WASD or arrow keys
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        camera.pan(0.0, -pan_amount);
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        camera.pan(0.0, pan_amount);
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        camera.pan(-pan_amount, 0.0);
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        camera.pan(pan_amount, 0.0);
    }

    // Zoom with mouse wheel
    let (_, scroll_y) = mouse_wheel();
    if scroll_y != 0.0 {
        camera.adjust_zoom(scroll_y * ZOOM_SPEED);
    }
}

fn draw_entity(entity: &RenderEntity, camera: &Camera) {
    let (screen_x, screen_y) = camera.world_to_screen(entity.position);

    // Calculate size based on health (smaller when wounded)
    let health_size_mod = 0.5 + entity.health * 0.5;
    let size = ENTITY_BASE_SIZE * camera.zoom * health_size_mod;

    // Calculate color with health and fatigue modulation
    let base_color = species_color(entity.species);
    let health_color = health_tint(base_color, entity.health);
    let final_color = fatigue_tint(health_color, entity.fatigue);

    draw_rectangle(
        screen_x - size / 2.0,
        screen_y - size / 2.0,
        size,
        size,
        to_macroquad_color(final_color),
    );
}

fn draw_ui(world: &World, entity_count: usize) {
    // FPS counter
    draw_text(
        &format!("FPS: {}", get_fps()),
        10.0,
        20.0,
        20.0,
        WHITE,
    );

    // Entity count
    draw_text(
        &format!("Entities: {}", entity_count),
        10.0,
        40.0,
        20.0,
        WHITE,
    );

    // Simulation tick
    draw_text(
        &format!("Tick: {}", world.current_tick),
        10.0,
        60.0,
        20.0,
        WHITE,
    );

    // Controls hint
    draw_text(
        "WASD/Arrows: Pan | Scroll: Zoom | ESC: Quit",
        10.0,
        screen_height() - 10.0,
        16.0,
        GRAY,
    );
}

/// Convert our Color to macroquad's Color
fn to_macroquad_color(c: RenderColor) -> macroquad::color::Color {
    macroquad::color::Color::new(c.r, c.g, c.b, c.a)
}

/// Create a test world with entities for visualization
fn setup_test_world() -> World {
    let mut world = World::new();
    let mut rng_seed = 42u64;

    // Spawn 500 humans spread across a 200x200 world
    for i in 0..500 {
        world.spawn_human(format!("Human_{}", i));

        // Grid layout with some randomness
        let base_x = (i % 25) as f32 * 8.0;
        let base_y = (i / 25) as f32 * 8.0;
        let jitter_x = pseudo_random(&mut rng_seed) * 4.0 - 2.0;
        let jitter_y = pseudo_random(&mut rng_seed) * 4.0 - 2.0;
        world.humans.positions[i] = SimVec2::new(base_x + jitter_x, base_y + jitter_y);

        // Varied health and fatigue for visual testing
        world.humans.body_states[i].overall_health = 0.3 + pseudo_random(&mut rng_seed) * 0.7;
        world.humans.body_states[i].fatigue = pseudo_random(&mut rng_seed) * 0.5;
    }

    // Add some food zones
    world.add_food_zone(SimVec2::new(50.0, 50.0), 20.0, Abundance::Unlimited);
    world.add_food_zone(SimVec2::new(150.0, 100.0), 15.0, Abundance::Unlimited);

    world
}

/// Simple pseudo-random number generator (0.0 to 1.0)
fn pseudo_random(seed: &mut u64) -> f32 {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    ((*seed >> 16) & 0x7fff) as f32 / 32767.0
}
```

**Step 2: Verify binary compiles**

Run: `cargo build --bin renderer`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/bin/renderer.rs
git commit -m "feat(render): add renderer binary with entity visualization"
```

---

## Task 6: Test the Renderer

**Files:**
- None (manual testing)

**Step 1: Run the renderer**

Run: `cargo run --bin renderer`
Expected:
- Window opens titled "Arc Citadel Renderer"
- 500 blue squares visible (humans)
- FPS counter shows ~60
- Entity count shows 500

**Step 2: Test camera controls**

Actions:
1. Press W/S/A/D - camera should pan
2. Scroll mouse wheel - camera should zoom in/out
3. Press Escape - window should close

Expected: All controls work smoothly

**Step 3: Verify health/fatigue visuals**

Observation:
- Some entities should appear darker (fatigued)
- Some entities should appear more red-tinted (low health)
- Some entities should be smaller (low health)

**Step 4: Commit test confirmation (optional)**

No code changes, but can document in README if desired.

---

## Task 7: Add Orc Visualization Test

**Files:**
- Modify: `src/bin/renderer.rs`

**Step 1: Add orcs to test world**

In `setup_test_world()`, add after the human spawning loop (around line 140):

```rust
    // Spawn 100 orcs in a separate cluster
    for i in 0..100 {
        world.spawn_orc(format!("Orc_{}", i));

        // Cluster in bottom-right area
        let base_x = 150.0 + (i % 10) as f32 * 6.0;
        let base_y = 150.0 + (i / 10) as f32 * 6.0;
        let jitter_x = pseudo_random(&mut rng_seed) * 3.0 - 1.5;
        let jitter_y = pseudo_random(&mut rng_seed) * 3.0 - 1.5;
        world.orcs.positions[i] = SimVec2::new(base_x + jitter_x, base_y + jitter_y);

        // Orcs are generally healthier but more tired
        world.orcs.body_states[i].overall_health = 0.7 + pseudo_random(&mut rng_seed) * 0.3;
        world.orcs.body_states[i].fatigue = pseudo_random(&mut rng_seed) * 0.8;
    }
```

**Step 2: Verify both species render**

Run: `cargo run --bin renderer`
Expected:
- Blue squares (humans) in main area
- Green squares (orcs) in bottom-right cluster
- Entity count shows 600

**Step 3: Commit**

```bash
git add src/bin/renderer.rs
git commit -m "feat(render): add orc entities to test visualization"
```

---

## Task 8: Run All Tests

**Files:**
- None

**Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests pass, including new render module tests

**Step 2: Verify no regressions**

Check that existing tests still pass (emergence tests, city tests, etc.)

---

## Success Criteria Checklist

After completing all tasks, verify:

- [x] `cargo build --bin renderer` succeeds
- [x] `cargo run --bin renderer` displays entities at 60fps
- [x] WASD pans camera smoothly
- [x] Mouse scroll zooms in/out (clamped 0.1x to 10x)
- [x] Camera clamps to world bounds
- [x] Entities colored by species (blue humans, green orcs)
- [x] Entity size modulated by health
- [x] Entity color modulated by health (red tint) and fatigue (darker)
- [x] `cargo test` passes all tests

---

## Files Created/Modified Summary

| File | Action | Purpose |
|------|--------|---------|
| `Cargo.toml` | Modified | Add macroquad dependency |
| `src/lib.rs` | Modified | Export render module |
| `src/render/mod.rs` | Created | RenderEntity, collect_render_entities() |
| `src/render/camera.rs` | Created | Camera with pan/zoom/transforms |
| `src/render/colors.rs` | Created | Species colors, health/fatigue tints |
| `src/bin/renderer.rs` | Created | Main renderer binary |

---

## Next Steps (Future Tasks)

1. **Sprite rendering** - Replace rectangles with actual sprites
2. **Animation system** - Add animation state machine
3. **Juice effects** - Screen shake, hit flash, particles
4. **Terrain rendering** - Draw food zones, resource zones
5. **Selection system** - Click to select entities
6. **Performance optimization** - Entity culling, spatial batching
