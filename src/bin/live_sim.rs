//! Live simulation renderer - shape-based MVP
//!
//! Connects wgpu renderer to running simulation.
//! Controls:
//!   WASD / Arrow keys: Pan camera
//!   +/-: Zoom in/out
//!   Mouse wheel: Zoom
//!   Space: Pause/resume simulation
//!   Escape: Quit

use std::sync::Arc;
use std::time::Instant;

use glam::Vec2;
use winit::{
    event::{ElementState, Event, MouseScrollDelta, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use arc_citadel::core::types::Vec2 as SimVec2;
use arc_citadel::ecs::world::{Abundance, World};
use arc_citadel::renderer::{CameraState, Color, RenderEntity, RenderState, Renderer, ShapeType};
use arc_citadel::simulation::tick::run_simulation_tick;

/// Convert simulation Vec2 to renderer Vec2
fn to_render_pos(v: SimVec2) -> Vec2 {
    Vec2::new(v.x, v.y)
}

const ENTITY_COUNT: usize = 50;
const WORLD_SIZE: f32 = 200.0;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Arc Citadel Live Simulation");

    // Create event loop and window
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Arc Citadel - Live Simulation")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)
            .expect("Failed to create window"),
    );

    // Create renderer
    let mut renderer = pollster::block_on(Renderer::new(window.clone()));

    // Create simulation world
    let mut world = World::new();

    // Add a food zone at center
    world.add_food_zone(
        SimVec2::new(WORLD_SIZE / 2.0, WORLD_SIZE / 2.0),
        30.0,
        Abundance::Unlimited,
    );

    // Spawn entities at random positions
    let mut rng_seed: u64 = 12345;
    for i in 0..ENTITY_COUNT {
        let id = world.spawn_human(format!("Human_{}", i));

        // Simple LCG random for positions
        rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let x = (rng_seed % 1000) as f32 / 1000.0 * WORLD_SIZE;
        rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let y = (rng_seed % 1000) as f32 / 1000.0 * WORLD_SIZE;

        if let Some(idx) = world.humans.index_of(id) {
            world.humans.positions[idx] = SimVec2::new(x, y);
        }
    }

    // Camera centered on world
    let mut camera = CameraState {
        center: Vec2::new(WORLD_SIZE / 2.0, WORLD_SIZE / 2.0),
        zoom: 2.0,
        viewport_size: Vec2::new(1280.0, 720.0),
    };

    // Simulation state
    let mut paused = false;
    let mut frame_count: u64 = 0;
    let mut last_fps_time = Instant::now();
    let mut sim_ticks: u64 = 0;

    // Run event loop
    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }

                    WindowEvent::Resized(size) => {
                        renderer.resize(size.width, size.height);
                        camera.set_viewport_size(size.width as f32, size.height as f32);
                    }

                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == ElementState::Pressed {
                            let pan_speed = 20.0 * camera.zoom;
                            match event.physical_key {
                                PhysicalKey::Code(KeyCode::KeyW)
                                | PhysicalKey::Code(KeyCode::ArrowUp) => {
                                    camera.pan(Vec2::new(0.0, pan_speed));
                                }
                                PhysicalKey::Code(KeyCode::KeyS)
                                | PhysicalKey::Code(KeyCode::ArrowDown) => {
                                    camera.pan(Vec2::new(0.0, -pan_speed));
                                }
                                PhysicalKey::Code(KeyCode::KeyA)
                                | PhysicalKey::Code(KeyCode::ArrowLeft) => {
                                    camera.pan(Vec2::new(-pan_speed, 0.0));
                                }
                                PhysicalKey::Code(KeyCode::KeyD)
                                | PhysicalKey::Code(KeyCode::ArrowRight) => {
                                    camera.pan(Vec2::new(pan_speed, 0.0));
                                }
                                PhysicalKey::Code(KeyCode::Equal)
                                | PhysicalKey::Code(KeyCode::NumpadAdd) => {
                                    camera.zoom_by(0.9);
                                }
                                PhysicalKey::Code(KeyCode::Minus)
                                | PhysicalKey::Code(KeyCode::NumpadSubtract) => {
                                    camera.zoom_by(1.1);
                                }
                                PhysicalKey::Code(KeyCode::Space) => {
                                    paused = !paused;
                                    tracing::info!(
                                        "Simulation {}",
                                        if paused { "PAUSED" } else { "RUNNING" }
                                    );
                                }
                                PhysicalKey::Code(KeyCode::Escape) => {
                                    elwt.exit();
                                }
                                _ => {}
                            }
                        }
                    }

                    WindowEvent::MouseWheel { delta, .. } => {
                        let zoom_factor = match delta {
                            MouseScrollDelta::LineDelta(_, y) => {
                                if y > 0.0 {
                                    0.9
                                } else {
                                    1.1
                                }
                            }
                            MouseScrollDelta::PixelDelta(pos) => {
                                if pos.y > 0.0 {
                                    0.95
                                } else {
                                    1.05
                                }
                            }
                        };
                        camera.zoom_by(zoom_factor);
                    }

                    WindowEvent::RedrawRequested => {
                        // Run simulation tick if not paused
                        if !paused {
                            run_simulation_tick(&mut world);
                            sim_ticks += 1;
                        }

                        // Extract entities for rendering
                        let mut entities =
                            Vec::with_capacity(world.humans.count() + world.food_zones.len() + 1);

                        // Render food zones as green hexagons
                        for zone in &world.food_zones {
                            entities.push(RenderEntity {
                                id: arc_citadel::core::types::EntityId::new(),
                                position: to_render_pos(zone.position),
                                facing: 0.0,
                                shape: ShapeType::Hexagon,
                                color: Color::rgba(0.2, 0.6, 0.2, 0.5),
                                scale: zone.radius,
                                z_order: 0,
                            });
                        }

                        // Render humans
                        for i in world.humans.iter_living() {
                            let pos = world.humans.positions[i];
                            let id = world.humans.ids[i];

                            // Color by need level (redder = more urgent needs)
                            let needs = &world.humans.needs[i];
                            let urgency = (needs.food + needs.rest + needs.social) / 3.0;
                            let color =
                                Color::rgba(0.3 + urgency * 0.7, 0.7 - urgency * 0.5, 0.3, 1.0);

                            entities.push(RenderEntity {
                                id,
                                position: to_render_pos(pos),
                                facing: 0.0,
                                shape: ShapeType::Circle,
                                color,
                                scale: 3.0,
                                z_order: 1,
                            });
                        }

                        // Render orcs as red triangles
                        for i in world.orcs.iter_living() {
                            let pos = world.orcs.positions[i];
                            let id = world.orcs.ids[i];

                            entities.push(RenderEntity {
                                id,
                                position: to_render_pos(pos),
                                facing: 0.0,
                                shape: ShapeType::Triangle,
                                color: Color::rgba(0.8, 0.2, 0.2, 1.0),
                                scale: 4.0,
                                z_order: 1,
                            });
                        }

                        let state = RenderState {
                            tick: frame_count,
                            entities,
                            sprites: vec![], // No sprites yet - using shapes for entities
                            camera,
                        };

                        match renderer.render(&state) {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => {
                                let (w, h) = renderer.size();
                                renderer.resize(w, h);
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                tracing::error!("Out of GPU memory!");
                                elwt.exit();
                            }
                            Err(e) => {
                                tracing::warn!("Render error: {:?}", e);
                            }
                        }

                        // Update title with metrics
                        frame_count += 1;
                        let elapsed = last_fps_time.elapsed().as_secs_f32();
                        if elapsed >= 1.0 {
                            let metrics = renderer.metrics();
                            let status = if paused { "PAUSED" } else { "RUNNING" };
                            window.set_title(&format!(
                                "Arc Citadel [{status}] | {} entities | tick {} | {:.1} FPS",
                                world.humans.count(),
                                sim_ticks,
                                metrics.fps(),
                            ));
                            frame_count = 0;
                            last_fps_time = Instant::now();
                        }
                    }

                    _ => {}
                },

                Event::AboutToWait => {
                    window.request_redraw();
                }

                _ => {}
            }
        })
        .expect("Event loop error");
}
