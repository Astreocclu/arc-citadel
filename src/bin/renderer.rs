//! wgpu renderer test binary.
//!
//! Renders 10,000 shapes with camera controls.
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
use arc_citadel::renderer::{CameraState, Color, RenderEntity, RenderState, Renderer, ShapeType};

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Arc Citadel wgpu renderer");

    // Create event loop and window
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Arc Citadel - wgpu Renderer")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)
            .expect("Failed to create window"),
    );

    // Create renderer
    let mut renderer = pollster::block_on(Renderer::new(window.clone()));

    // Create test entities - 10,000 shapes in a grid
    let grid_size = 100;
    let spacing = 10.0;
    let mut entities = Vec::with_capacity(grid_size * grid_size);

    for i in 0..grid_size {
        for j in 0..grid_size {
            let shape = match (i + j) % 4 {
                0 => ShapeType::Circle,
                1 => ShapeType::Rectangle,
                2 => ShapeType::Triangle,
                _ => ShapeType::Hexagon,
            };

            // Color gradient based on position
            let color = Color::rgba(
                i as f32 / grid_size as f32,
                j as f32 / grid_size as f32,
                0.5,
                1.0,
            );

            entities.push(RenderEntity {
                id: EntityId::new(),
                position: Vec2::new(i as f32 * spacing, j as f32 * spacing),
                facing: 0.0,
                shape,
                color,
                scale: 3.0,
                z_order: 0,
            });
        }
    }

    // Center camera on the grid
    let center = Vec2::new(
        (grid_size as f32 * spacing) / 2.0,
        (grid_size as f32 * spacing) / 2.0,
    );

    let mut camera = CameraState {
        center,
        zoom: 1.0,
        viewport_size: Vec2::new(1280.0, 720.0),
    };

    // FPS tracking
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
                            PhysicalKey::Code(KeyCode::KeyW) | PhysicalKey::Code(KeyCode::ArrowUp) => {
                                camera.pan(Vec2::new(0.0, pan_speed));
                            }
                            PhysicalKey::Code(KeyCode::KeyS) | PhysicalKey::Code(KeyCode::ArrowDown) => {
                                camera.pan(Vec2::new(0.0, -pan_speed));
                            }
                            PhysicalKey::Code(KeyCode::KeyA) | PhysicalKey::Code(KeyCode::ArrowLeft) => {
                                camera.pan(Vec2::new(-pan_speed, 0.0));
                            }
                            PhysicalKey::Code(KeyCode::KeyD) | PhysicalKey::Code(KeyCode::ArrowRight) => {
                                camera.pan(Vec2::new(pan_speed, 0.0));
                            }
                            PhysicalKey::Code(KeyCode::Equal) | PhysicalKey::Code(KeyCode::NumpadAdd) => {
                                camera.zoom_by(0.9); // Zoom in
                            }
                            PhysicalKey::Code(KeyCode::Minus) | PhysicalKey::Code(KeyCode::NumpadSubtract) => {
                                camera.zoom_by(1.1); // Zoom out
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
                            if y > 0.0 { 0.9 } else { 1.1 }
                        }
                        MouseScrollDelta::PixelDelta(pos) => {
                            if pos.y > 0.0 { 0.95 } else { 1.05 }
                        }
                    };
                    camera.zoom_by(zoom_factor);
                }

                WindowEvent::RedrawRequested => {
                    let state = RenderState {
                        tick: frame_count,
                        entities: entities.clone(),
                        sprites: vec![],
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
                        frame_count = 0;
                        last_fps_time = Instant::now();

                        let metrics = renderer.metrics();
                        window.set_title(&format!(
                            "Arc Citadel - {} entities | {:.1} FPS ({:.2}ms) | {} draws | {} uploads",
                            entities.len(),
                            metrics.fps(),
                            metrics.avg_frame_time_ms(),
                            metrics.draw_calls,
                            metrics.buffer_uploads
                        ));
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
