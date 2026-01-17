//! Live simulation renderer - shape-based MVP
//!
//! Connects wgpu renderer to running simulation.
//! Controls:
//!   WASD / Arrow keys: Pan camera
//!   +/-: Zoom in/out
//!   Mouse wheel: Zoom
//!   Space: Pause/resume simulation
//!   Enter: Focus command input
//!   Escape: Quit / Cancel command
//!
//! Commands (type in command input):
//!   move <name> <x> <y>  - Move entity to position
//!   gather <name>        - Have entity gather food
//!   rest <name>          - Have entity rest
//!   attack <name> <target> - Have entity attack target
//!   spawn <name>         - Spawn a new human
//!   spawn_orc <name>     - Spawn a hostile orc
//!   save <filename>      - Save game state
//!   load <filename>      - Load game state

use std::fs::File;
use std::io::{BufReader, BufWriter};
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

use arc_citadel::actions::catalog::ActionId;
use arc_citadel::core::types::{EntityId, Vec2 as SimVec2};
use arc_citadel::ecs::world::{Abundance, World};
use arc_citadel::entity::tasks::{Task, TaskPriority, TaskSource};
use arc_citadel::renderer::{CameraState, Color, RenderEntity, RenderState, Renderer, ShapeType};
use arc_citadel::simulation::tick::run_simulation_tick;
use arc_citadel::simulation::SimulationEvent;
use arc_citadel::ui::{GameUI, LogCategory};

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

    // Spawn hostile orcs at edges of the map
    const ORC_COUNT: usize = 10;
    for i in 0..ORC_COUNT {
        let id = world.spawn_orc(format!("Orc_{}", i));

        // Position orcs at the edges of the map
        rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let edge = rng_seed % 4;
        rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let along_edge = (rng_seed % 1000) as f32 / 1000.0 * WORLD_SIZE;

        let (x, y) = match edge {
            0 => (along_edge, 5.0),                      // Top edge
            1 => (along_edge, WORLD_SIZE - 5.0),         // Bottom edge
            2 => (5.0, along_edge),                      // Left edge
            _ => (WORLD_SIZE - 5.0, along_edge),         // Right edge
        };

        if let Some(idx) = world.orcs.index_of(id) {
            world.orcs.positions[idx] = SimVec2::new(x, y);
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

    // Battle state for win/lose tracking
    let mut battle_state = BattleState::new(&world);

    // Mouse position tracking for entity selection
    let mut mouse_pos: Option<(f32, f32)> = None;

    // Pending command to execute (from egui)
    let mut pending_command: Option<String> = None;

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
                                        PhysicalKey::Code(KeyCode::Enter) => {
                                            // Focus command input
                                            game_ui.command_focused = true;
                                        }
                                        PhysicalKey::Code(KeyCode::Escape) => {
                                            // Clear command input or quit
                                            if !game_ui.command_input.is_empty() {
                                                game_ui.command_input.clear();
                                            } else {
                                                elwt.exit();
                                            }
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
                        // Execute pending command if any
                        if let Some(cmd) = pending_command.take() {
                            let result = execute_command(&mut world, &cmd, sim_ticks);
                            game_ui.log(sim_ticks, format!("Command: {}", cmd), LogCategory::System);
                            game_ui.log(sim_ticks, result, LogCategory::System);
                            // Reset battle state if we loaded a new game
                            if cmd.starts_with("load ") {
                                battle_state = BattleState::new(&world);
                            }
                        }

                        // Run simulation tick if not paused
                        if !paused {
                            let events = run_simulation_tick(&mut world);
                            sim_ticks += 1;

                            // Update battle state
                            battle_state.update(&world);

                            // Log simulation events to the action log
                            for event in events {
                                let (msg, category) = match event {
                                    SimulationEvent::TaskStarted { entity_name, action, .. } => {
                                        (format!("{} started {:?}", entity_name, action), LogCategory::Action)
                                    }
                                    SimulationEvent::TaskCompleted { entity_name, action } => {
                                        (format!("{} completed {:?}", entity_name, action), LogCategory::Action)
                                    }
                                    SimulationEvent::CombatHit { attacker, defender } => {
                                        (format!("{} hit {}", attacker, defender), LogCategory::Combat)
                                    }
                                    SimulationEvent::ProductionComplete { recipe, .. } => {
                                        (format!("Produced: {}", recipe), LogCategory::Production)
                                    }
                                    // Detailed logging events - skip in live UI (too verbose)
                                    SimulationEvent::PerceptionUpdate { .. }
                                    | SimulationEvent::ThoughtGenerated { .. }
                                    | SimulationEvent::SocialMemoryUpdate { .. }
                                    | SimulationEvent::DispositionChange { .. } => continue,
                                };
                                game_ui.log(sim_ticks, msg, category);
                            }
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
                        draw_ui(&egui_ctx, &mut game_ui, &world, &battle_state);

                        // Check if command was submitted (Enter pressed in text field)
                        if egui_ctx.input(|i| i.key_pressed(egui::Key::Enter)) && !game_ui.command_input.is_empty() {
                            pending_command = Some(game_ui.command_input.clone());
                            game_ui.command_input.clear();
                        }

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
fn draw_ui(ctx: &egui::Context, ui: &mut GameUI, world: &World, battle_state: &BattleState) {
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

    // Status bar (top) with command input
    egui::TopBottomPanel::top("status_bar").show(ctx, |panel| {
        panel.horizontal(|h| {
            h.label(format!("Tick: {}", world.current_tick));
            h.separator();
            h.label(format!("Humans: {}", world.humans.count()));
            h.separator();
            h.label(format!("Orcs: {}", world.orcs.count()));
            h.separator();

            // Battle status
            match &battle_state.outcome {
                BattleOutcome::InProgress => {
                    h.label("Battle: In Progress");
                }
                BattleOutcome::Victory => {
                    h.colored_label(egui::Color32::GREEN, "VICTORY!");
                }
                BattleOutcome::Defeat => {
                    h.colored_label(egui::Color32::RED, "DEFEAT!");
                }
                BattleOutcome::Draw => {
                    h.colored_label(egui::Color32::YELLOW, "DRAW");
                }
            }
        });

        // Command input
        panel.horizontal(|h| {
            h.label("Command:");
            let response = h.text_edit_singleline(&mut ui.command_input);
            if ui.command_focused {
                response.request_focus();
                ui.command_focused = false;
            }
        });
    });
}

/// Battle state for win/lose conditions
#[derive(Debug, Clone)]
struct BattleState {
    outcome: BattleOutcome,
    humans_alive_at_start: usize,
    orcs_alive_at_start: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum BattleOutcome {
    InProgress,
    Victory,
    Defeat,
    Draw,
}

impl BattleState {
    fn new(world: &World) -> Self {
        Self {
            outcome: BattleOutcome::InProgress,
            humans_alive_at_start: world.humans.count(),
            orcs_alive_at_start: world.orcs.count(),
        }
    }

    fn update(&mut self, world: &World) {
        if self.outcome != BattleOutcome::InProgress {
            return; // Already decided
        }

        let humans_alive = world.humans.count();
        let orcs_alive = world.orcs.count();

        // Check win/lose conditions
        if orcs_alive == 0 && self.orcs_alive_at_start > 0 {
            self.outcome = BattleOutcome::Victory;
        } else if humans_alive == 0 && self.humans_alive_at_start > 0 {
            self.outcome = BattleOutcome::Defeat;
        } else if humans_alive == 0 && orcs_alive == 0 {
            self.outcome = BattleOutcome::Draw;
        }
    }
}

/// Parse and execute a local command (without LLM)
fn execute_command(world: &mut World, command: &str, tick: u64) -> String {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    if parts.is_empty() {
        return "Empty command".to_string();
    }

    match parts[0].to_lowercase().as_str() {
        "move" => {
            if parts.len() < 4 {
                return "Usage: move <name> <x> <y>".to_string();
            }
            let name = parts[1];
            let x: f32 = match parts[2].parse() {
                Ok(v) => v,
                Err(_) => return "Invalid x coordinate".to_string(),
            };
            let y: f32 = match parts[3].parse() {
                Ok(v) => v,
                Err(_) => return "Invalid y coordinate".to_string(),
            };

            if let Some(entity_id) = find_entity_by_name(world, name) {
                if let Some(idx) = world.humans.index_of(entity_id) {
                    let mut task = Task::new(ActionId::MoveTo, TaskPriority::High, tick);
                    task.source = TaskSource::PlayerCommand;
                    task.target_position = Some(SimVec2::new(x, y));
                    world.humans.task_queues[idx].push(task);
                    return format!("{} moving to ({}, {})", name, x, y);
                }
            }
            format!("Entity '{}' not found", name)
        }

        "gather" => {
            if parts.len() < 2 {
                return "Usage: gather <name>".to_string();
            }
            let name = parts[1];

            if let Some(entity_id) = find_entity_by_name(world, name) {
                if let Some(idx) = world.humans.index_of(entity_id) {
                    let mut task = Task::new(ActionId::Gather, TaskPriority::High, tick);
                    task.source = TaskSource::PlayerCommand;
                    // Target the nearest food zone
                    if let Some(zone) = world.food_zones.first() {
                        task.target_position = Some(zone.position);
                    }
                    world.humans.task_queues[idx].push(task);
                    return format!("{} gathering food", name);
                }
            }
            format!("Entity '{}' not found", name)
        }

        "rest" => {
            if parts.len() < 2 {
                return "Usage: rest <name>".to_string();
            }
            let name = parts[1];

            if let Some(entity_id) = find_entity_by_name(world, name) {
                if let Some(idx) = world.humans.index_of(entity_id) {
                    let mut task = Task::new(ActionId::Rest, TaskPriority::High, tick);
                    task.source = TaskSource::PlayerCommand;
                    world.humans.task_queues[idx].push(task);
                    return format!("{} resting", name);
                }
            }
            format!("Entity '{}' not found", name)
        }

        "attack" => {
            if parts.len() < 3 {
                return "Usage: attack <name> <target>".to_string();
            }
            let name = parts[1];
            let target_name = parts[2];

            let attacker_id = find_entity_by_name(world, name);
            let target_id = find_entity_by_name(world, target_name);

            match (attacker_id, target_id) {
                (Some(attacker), Some(target)) => {
                    if let Some(idx) = world.humans.index_of(attacker) {
                        let mut task = Task::new(ActionId::Attack, TaskPriority::Critical, tick);
                        task.source = TaskSource::PlayerCommand;
                        task.target_entity = Some(target);
                        world.humans.task_queues[idx].push(task);
                        return format!("{} attacking {}", name, target_name);
                    }
                    format!("{} cannot attack", name)
                }
                (None, _) => format!("Attacker '{}' not found", name),
                (_, None) => format!("Target '{}' not found", target_name),
            }
        }

        "save" => {
            if parts.len() < 2 {
                return "Usage: save <filename>".to_string();
            }
            let filename = parts[1];
            match save_game(world, filename) {
                Ok(_) => format!("Game saved to {}", filename),
                Err(e) => format!("Save failed: {}", e),
            }
        }

        "load" => {
            if parts.len() < 2 {
                return "Usage: load <filename>".to_string();
            }
            let filename = parts[1];
            match load_game(filename) {
                Ok(loaded_world) => {
                    *world = loaded_world;
                    format!("Game loaded from {}", filename)
                }
                Err(e) => format!("Load failed: {}", e),
            }
        }

        "spawn" => {
            if parts.len() < 2 {
                return "Usage: spawn <name>".to_string();
            }
            let name = parts[1];
            let id = world.spawn_human(name.to_string());
            // Random position
            let x = (tick as f32 * 1.618) % WORLD_SIZE;
            let y = ((tick as f32 + 1000.0) * 2.718) % WORLD_SIZE;
            if let Some(idx) = world.humans.index_of(id) {
                world.humans.positions[idx] = SimVec2::new(x, y);
            }
            format!("Spawned {} at ({:.0}, {:.0})", name, x, y)
        }

        "spawn_orc" => {
            if parts.len() < 2 {
                return "Usage: spawn_orc <name>".to_string();
            }
            let name = parts[1];
            let id = world.spawn_orc(name.to_string());
            // Position at edge
            let edge = tick % 4;
            let along = (tick as f32 * 1.618) % WORLD_SIZE;
            let (x, y) = match edge {
                0 => (along, 5.0),
                1 => (along, WORLD_SIZE - 5.0),
                2 => (5.0, along),
                _ => (WORLD_SIZE - 5.0, along),
            };
            if let Some(idx) = world.orcs.index_of(id) {
                world.orcs.positions[idx] = SimVec2::new(x, y);
            }
            format!("Spawned orc {} at ({:.0}, {:.0})", name, x, y)
        }

        "help" => {
            "Commands: move, gather, rest, attack, spawn, spawn_orc, save, load, help".to_string()
        }

        _ => format!("Unknown command: '{}'. Type 'help' for commands.", parts[0]),
    }
}

/// Find an entity by name (searches humans and orcs)
fn find_entity_by_name(world: &World, name: &str) -> Option<EntityId> {
    let name_lower = name.to_lowercase();

    // Search humans
    for i in world.humans.iter_living() {
        if world.humans.names[i].to_lowercase().contains(&name_lower) {
            return Some(world.humans.ids[i]);
        }
    }

    // Search orcs
    for i in world.orcs.iter_living() {
        if world.orcs.names[i].to_lowercase().contains(&name_lower) {
            return Some(world.orcs.ids[i]);
        }
    }

    None
}

/// Simplified save state for MVP
#[derive(serde::Serialize, serde::Deserialize)]
struct SaveState {
    tick: u64,
    humans: Vec<HumanSaveData>,
    orcs: Vec<OrcSaveData>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct HumanSaveData {
    name: String,
    position: (f32, f32),
    needs: NeedsSaveData,
    fatigue: f32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct OrcSaveData {
    name: String,
    position: (f32, f32),
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NeedsSaveData {
    food: f32,
    rest: f32,
    safety: f32,
    social: f32,
    purpose: f32,
}

/// Save game state to file
fn save_game(world: &World, filename: &str) -> std::io::Result<()> {
    let mut humans = Vec::new();
    for i in world.humans.iter_living() {
        let needs = &world.humans.needs[i];
        humans.push(HumanSaveData {
            name: world.humans.names[i].clone(),
            position: (world.humans.positions[i].x, world.humans.positions[i].y),
            needs: NeedsSaveData {
                food: needs.food,
                rest: needs.rest,
                safety: needs.safety,
                social: needs.social,
                purpose: needs.purpose,
            },
            fatigue: world.humans.body_states[i].fatigue,
        });
    }

    let mut orcs = Vec::new();
    for i in world.orcs.iter_living() {
        orcs.push(OrcSaveData {
            name: world.orcs.names[i].clone(),
            position: (world.orcs.positions[i].x, world.orcs.positions[i].y),
        });
    }

    let state = SaveState {
        tick: world.current_tick,
        humans,
        orcs,
    };

    let file = File::create(filename)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &state).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })
}

/// Load game state from file
fn load_game(filename: &str) -> std::io::Result<World> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let state: SaveState = serde_json::from_reader(reader).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })?;

    let mut world = World::new();

    // Add food zone at center (like initial setup)
    world.add_food_zone(
        SimVec2::new(WORLD_SIZE / 2.0, WORLD_SIZE / 2.0),
        30.0,
        Abundance::Unlimited,
    );

    // Restore humans
    for human in state.humans {
        let id = world.spawn_human(human.name);
        if let Some(idx) = world.humans.index_of(id) {
            world.humans.positions[idx] = SimVec2::new(human.position.0, human.position.1);
            world.humans.needs[idx].food = human.needs.food;
            world.humans.needs[idx].rest = human.needs.rest;
            world.humans.needs[idx].safety = human.needs.safety;
            world.humans.needs[idx].social = human.needs.social;
            world.humans.needs[idx].purpose = human.needs.purpose;
            world.humans.body_states[idx].fatigue = human.fatigue;
        }
    }

    // Restore orcs
    for orc in state.orcs {
        let id = world.spawn_orc(orc.name);
        if let Some(idx) = world.orcs.index_of(id) {
            world.orcs.positions[idx] = SimVec2::new(orc.position.0, orc.position.1);
        }
    }

    Ok(world)
}
