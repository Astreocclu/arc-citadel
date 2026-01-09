//! Sprite rendering test binary.
//!
//! Creates a 10x10 grid of sprites to test sprite rendering.
//! Controls:
//!   WASD / Arrow keys: Pan camera
//!   +/-: Zoom in/out
//!   Mouse wheel: Zoom
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

use arc_citadel::core::types::EntityId;
use arc_citadel::renderer::{CameraState, RenderState, Renderer, SpriteEntity};

const GRID_SIZE: usize = 10;
const SPRITE_SPACING: f32 = 20.0;
const SPRITE_SCALE: f32 = 8.0;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Sprite Test");

    // Create event loop and window
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Arc Citadel - Sprite Test")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)
            .expect("Failed to create window"),
    );

    // Create renderer
    let mut renderer = pollster::block_on(Renderer::new(window.clone()));

    // Create test sprites in a 10x10 grid
    let sprites: Vec<SpriteEntity> = (0..GRID_SIZE)
        .flat_map(|i| {
            (0..GRID_SIZE).map(move |j| SpriteEntity {
                id: EntityId::new(),
                position: Vec2::new(i as f32 * SPRITE_SPACING, j as f32 * SPRITE_SPACING),
                uv_rect: [0.0, 0.0, 1.0, 1.0],
                color: [(i * 25) as u8, (j * 25) as u8, 128, 255],
                rotation: (i + j) as f32 * 0.3,
                scale: SPRITE_SCALE,
                flip_x: i % 2 == 0,
                flip_y: j % 2 == 0,
                z_order: 0,
            })
        })
        .collect();

    tracing::info!("Created {} sprites in {}x{} grid", sprites.len(), GRID_SIZE, GRID_SIZE);

    // Calculate grid center for camera
    let grid_center = Vec2::new(
        (GRID_SIZE as f32 - 1.0) * SPRITE_SPACING / 2.0,
        (GRID_SIZE as f32 - 1.0) * SPRITE_SPACING / 2.0,
    );

    // Camera centered on grid
    let mut camera = CameraState {
        center: grid_center,
        zoom: 1.0,
        viewport_size: Vec2::new(1280.0, 720.0),
    };

    // Frame metrics
    let mut frame_count: u64 = 0;
    let mut last_fps_time = Instant::now();

    // Run event loop
    event_loop.run(move |event, elwt| {
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
                    // Build render state with sprites only (no shape entities)
                    let state = RenderState {
                        tick: frame_count,
                        entities: Vec::new(),
                        sprites: sprites.clone(),
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

                    // Update title with FPS
                    frame_count += 1;
                    let elapsed = last_fps_time.elapsed().as_secs_f32();
                    if elapsed >= 1.0 {
                        let metrics = renderer.metrics();
                        window.set_title(&format!(
                            "Arc Citadel - Sprite Test | {} sprites | {:.1} FPS | zoom {:.2}",
                            sprites.len(),
                            metrics.fps(),
                            camera.zoom,
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
    }).expect("Event loop error");
}
