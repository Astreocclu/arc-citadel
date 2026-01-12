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

use egui_wgpu::ScreenDescriptor;
use egui_winit::State as EguiWinitState;
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
use arc_citadel::ui::GameUI;

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

    // Create egui context and state
    let egui_ctx = egui::Context::default();
    let mut egui_winit_state = EguiWinitState::new(
        egui_ctx.clone(),
        egui::ViewportId::ROOT,
        &window,
        None,
        None,
    );

    // Create egui renderer for wgpu
    let mut egui_renderer = egui_wgpu::Renderer::new(
        renderer.device(),
        renderer.surface_format(),
        None,
        1,
    );

    // Game UI state
    let mut game_ui = GameUI::new();

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

    // Mouse position tracking for entity selection
    let mut mouse_pos: Option<(f32, f32)> = None;

    // Run event loop
    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent { ref event, .. } => {
                    // Let egui handle events first
                    let egui_consumed = egui_winit_state.on_window_event(&window, event).consumed;

                    // Handle game input only if egui didn't consume it
                    if !egui_consumed {
                        match event {
                            WindowEvent::CloseRequested => {
                                elwt.exit();
                            }

                            WindowEvent::Resized(size) => {
                                renderer.resize(size.width, size.height);
                                camera.set_viewport_size(size.width as f32, size.height as f32);
                            }

                            WindowEvent::KeyboardInput {
                                event: key_event, ..
                            } => {
                                if key_event.state == ElementState::Pressed {
                                    let pan_speed = 20.0 * camera.zoom;
                                    match key_event.physical_key {
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
                                        if *y > 0.0 {
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

                            WindowEvent::CursorMoved { position, .. } => {
                                mouse_pos = Some((position.x as f32, position.y as f32));
                            }

                            WindowEvent::MouseInput {
                                state: ElementState::Pressed,
                                button: winit::event::MouseButton::Left,
                                ..
                            } => {
                                if let Some((mx, my)) = mouse_pos {
                                    // Convert screen to world coordinates
                                    let world_pos = camera.screen_to_world(Vec2::new(mx, my));

                                    // Find entity near click (within 10 units, scaled by zoom)
                                    let click_radius = 10.0 * camera.zoom;

                                    let mut closest: Option<(arc_citadel::core::types::EntityId, f32)> = None;
                                    for i in 0..world.humans.ids.len() {
                                        if !world.humans.alive[i] {
                                            continue;
                                        }
                                        let pos = world.humans.positions[i];
                                        let dist = ((pos.x - world_pos.x).powi(2) + (pos.y - world_pos.y).powi(2)).sqrt();
                                        if dist < click_radius {
                                            if closest.is_none() || dist < closest.unwrap().1 {
                                                closest = Some((world.humans.ids[i], dist));
                                            }
                                        }
                                    }

                                    if let Some((entity_id, _)) = closest {
                                        game_ui.toggle_select(entity_id);
                                    } else {
                                        game_ui.deselect();
                                    }
                                }
                            }

                            _ => {}
                        }
                    }

                    // Always handle RedrawRequested regardless of egui consumption
                    if let WindowEvent::RedrawRequested = event {
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

                            // Check if this entity is selected
                            let is_selected = game_ui.selected_entity == Some(id);

                            // Color: yellow highlight if selected, otherwise by need level
                            let color = if is_selected {
                                Color::rgba(1.0, 1.0, 0.0, 1.0) // Yellow highlight
                            } else {
                                // Color by need level (redder = more urgent needs)
                                let needs = &world.humans.needs[i];
                                let urgency = (needs.food + needs.rest + needs.social) / 3.0;
                                Color::rgba(0.3 + urgency * 0.7, 0.7 - urgency * 0.5, 0.3, 1.0)
                            };

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

                        // Begin egui frame
                        let raw_input = egui_winit_state.take_egui_input(&window);
                        egui_ctx.begin_frame(raw_input);

                        // Draw UI
                        draw_ui(&egui_ctx, &mut game_ui, &world);

                        // End egui frame and get paint data
                        let full_output = egui_ctx.end_frame();
                        egui_winit_state.handle_platform_output(&window, full_output.platform_output);
                        let paint_jobs = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

                        // Get screen size for egui rendering
                        let (width, height) = renderer.size();
                        let screen_descriptor = ScreenDescriptor {
                            size_in_pixels: [width, height],
                            pixels_per_point: full_output.pixels_per_point,
                        };

                        // Render with egui callback
                        let render_result = renderer.render_with_egui(&state, |device, queue, encoder, view| {
                            // Update egui textures
                            for (id, image_delta) in &full_output.textures_delta.set {
                                egui_renderer.update_texture(device, queue, *id, image_delta);
                            }

                            // Upload egui buffers
                            egui_renderer.update_buffers(
                                device,
                                queue,
                                encoder,
                                &paint_jobs,
                                &screen_descriptor,
                            );

                            // Render egui
                            {
                                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("egui Render Pass"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Load, // Load to preserve game content
                                            store: wgpu::StoreOp::Store,
                                        },
                                    })],
                                    depth_stencil_attachment: None,
                                    timestamp_writes: None,
                                    occlusion_query_set: None,
                                });
                                egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
                            }

                            // Free textures marked for removal
                            for id in &full_output.textures_delta.free {
                                egui_renderer.free_texture(id);
                            }
                        });

                        match render_result {
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
                }

                Event::AboutToWait => {
                    window.request_redraw();
                }

                _ => {}
            }
        })
        .expect("Event loop error");
}

/// Draw the game UI using egui
fn draw_ui(ctx: &egui::Context, ui: &mut GameUI, world: &World) {
    // Entity panel (right side)
    if ui.show_entity_panel {
        egui::SidePanel::right("entity_panel")
            .default_width(250.0)
            .show(ctx, |panel| {
                panel.heading("Entity");

                if let Some(entity_id) = ui.selected_entity {
                    if let Some(idx) = world.humans.index_of(entity_id) {
                        panel.label(format!("Name: {}", world.humans.names[idx]));
                        panel.label(format!(
                            "Fatigue: {:.0}%",
                            world.humans.body_states[idx].fatigue * 100.0
                        ));

                        panel.separator();
                        panel.label("Needs:");
                        let needs = &world.humans.needs[idx];
                        panel.label(format!("  Food: {:.0}%", needs.food * 100.0));
                        panel.label(format!("  Rest: {:.0}%", needs.rest * 100.0));
                        panel.label(format!("  Safety: {:.0}%", needs.safety * 100.0));
                        panel.label(format!("  Social: {:.0}%", needs.social * 100.0));
                        panel.label(format!("  Purpose: {:.0}%", needs.purpose * 100.0));

                        panel.separator();
                        if let Some(task) = world.humans.task_queues[idx].current() {
                            panel.label(format!("Task: {:?}", task.action));
                        } else {
                            panel.label("Task: Idle");
                        }
                    }
                } else {
                    panel.label("Click an entity to select");
                }
            });
    }

    // Action log (bottom)
    if ui.show_action_log {
        egui::TopBottomPanel::bottom("action_log")
            .default_height(120.0)
            .show(ctx, |panel| {
                panel.heading("Action Log");
                egui::ScrollArea::vertical().show(panel, |scroll| {
                    for entry in ui.action_log.iter().rev().take(10) {
                        scroll.label(format!("[{}] {}", entry.tick, entry.message));
                    }
                });
            });
    }

    // Tick counter (top)
    egui::TopBottomPanel::top("status_bar").show(ctx, |panel| {
        panel.horizontal(|h| {
            h.label(format!("Tick: {}", world.current_tick));
            h.separator();
            h.label(format!("Entities: {}", world.humans.count()));
        });
    });
}
