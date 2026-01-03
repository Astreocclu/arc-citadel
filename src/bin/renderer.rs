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
