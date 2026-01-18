# Playability Features Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform the running simulation into a playable game where players issue commands via LLM, see results in the GPU renderer, and manage settlement economy.

**Architecture:**
- Two-phase command pipeline: ParsedIntent → IntentResolver (resolve subjects + locations) → TaskCreator → Vec<Task>
- egui overlay on existing wgpu renderer for UI panels
- Combat adapter pattern bridging tick.rs ↔ combat/resolution.rs
- Fix building math by removing double skill penalty

**Tech Stack:** Rust, wgpu, egui, winit, existing combat/production modules

---

## Task 1: Fix Building Progress Math

**Files:**
- Modify: `src/simulation/tick.rs:1119-1122`
- Test: Run existing tests `cargo test test_build_action`

**Step 1: Understand the bug**

The building tests fail because skill is double-penalized:
1. `calculate_worker_contribution(skill, fatigue)` applies: `BASE_RATE * (0.5 + skill * 0.5)`
2. Then line 1122 multiplies by `effective_skill` from skill_check (~0.28)

Result: 1.0 × 0.75 × 1.0 × 0.28 ≈ 0.21/tick instead of expected ~1.0/tick

**Step 2: Run failing tests to confirm**

Run: `cargo test test_build_action -- --nocapture`
Expected: 3 failures showing progress ~10 instead of ~50

**Step 3: Apply the fix**

In `src/simulation/tick.rs`, replace lines 1119-1122:

```rust
// BEFORE (around line 1119-1122):
// Calculate contribution with skill modifier
let base_contribution =
    calculate_worker_contribution(building_skill, fatigue);
let contribution = base_contribution * effective_skill;

// AFTER:
// Calculate contribution - skill already factored into base calculation
let contribution =
    calculate_worker_contribution(building_skill, fatigue);
```

**Step 4: Run tests to verify fix**

Run: `cargo test test_build_action`
Expected: 3 PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "fix(building): remove double skill penalty in construction progress

calculate_worker_contribution already applies skill modifier (0.5 + skill * 0.5).
Multiplying by effective_skill from skill_check was double-penalizing.

Fixes: test_build_action_with_building_target
Fixes: test_build_action_progress_reflects_construction
Fixes: test_build_action_improves_skill_on_completion"
```

---

## Task 2: Create Command Module Structure

**Files:**
- Create: `src/command/mod.rs`
- Create: `src/command/resolver.rs`
- Create: `src/command/executor.rs`
- Modify: `src/lib.rs`

**Step 1: Create module directory and mod.rs**

Create `src/command/mod.rs`:

```rust
//! Command execution pipeline
//!
//! Converts LLM ParsedIntent into executable Tasks:
//! ParsedIntent → IntentResolver → IntentResolution → TaskCreator → Vec<Task>

pub mod executor;
pub mod resolver;

pub use executor::CommandExecutor;
pub use resolver::{IntentResolution, IntentResolver, SubjectMatch};
```

**Step 2: Create resolver.rs with subject resolution**

Create `src/command/resolver.rs`:

```rust
//! Intent resolution - converts ParsedIntent subjects/locations to concrete entities/positions

use crate::core::types::{EntityId, Vec2};
use crate::ecs::world::World;
use crate::llm::parser::ParsedIntent;

/// Result of resolving an intent's subjects and location
#[derive(Debug, Clone)]
pub struct IntentResolution {
    /// Entities that matched the subject criteria
    pub subjects: Vec<SubjectMatch>,
    /// Resolved location (if specified)
    pub location: Option<Vec2>,
    /// Target entity (for "attack that orc" style commands)
    pub target_entity: Option<EntityId>,
    /// Any ambiguity or issues encountered
    pub notes: Vec<String>,
}

/// A matched subject with confidence
#[derive(Debug, Clone)]
pub struct SubjectMatch {
    pub entity_id: EntityId,
    pub name: String,
    pub match_reason: MatchReason,
}

#[derive(Debug, Clone)]
pub enum MatchReason {
    ExactName,
    PartialName,
    Qualification { skill: String, level: f32 },
    Everyone,
}

/// Resolves ParsedIntent subjects and locations to concrete entities/positions
pub struct IntentResolver<'a> {
    world: &'a World,
}

impl<'a> IntentResolver<'a> {
    pub fn new(world: &'a World) -> Self {
        Self { world }
    }

    /// Resolve a parsed intent to concrete entities and positions
    pub fn resolve(&self, intent: &ParsedIntent) -> IntentResolution {
        let subjects = self.resolve_subjects(&intent.subjects);
        let location = self.resolve_location(&intent.location);
        let target_entity = self.resolve_target(&intent.target);

        IntentResolution {
            subjects,
            location,
            target_entity,
            notes: Vec::new(),
        }
    }

    fn resolve_subjects(&self, subjects: &Option<Vec<String>>) -> Vec<SubjectMatch> {
        let Some(subject_specs) = subjects else {
            return Vec::new();
        };

        let mut matches = Vec::new();

        for spec in subject_specs {
            let spec_lower = spec.to_lowercase();

            // Check for "everyone" / "all"
            if spec_lower == "everyone" || spec_lower == "all" {
                for (i, name) in self.world.humans.names.iter().enumerate() {
                    if self.world.humans.alive[i] {
                        matches.push(SubjectMatch {
                            entity_id: self.world.humans.ids[i],
                            name: name.clone(),
                            match_reason: MatchReason::Everyone,
                        });
                    }
                }
                continue;
            }

            // Check for qualification patterns
            if let Some(qual_match) = self.resolve_qualification(&spec_lower) {
                matches.extend(qual_match);
                continue;
            }

            // Try exact name match
            if let Some(m) = self.find_by_name(&spec) {
                matches.push(m);
            }
        }

        matches
    }

    fn find_by_name(&self, name: &str) -> Option<SubjectMatch> {
        let name_lower = name.to_lowercase();

        for (i, entity_name) in self.world.humans.names.iter().enumerate() {
            if !self.world.humans.alive[i] {
                continue;
            }

            if entity_name.to_lowercase() == name_lower {
                return Some(SubjectMatch {
                    entity_id: self.world.humans.ids[i],
                    name: entity_name.clone(),
                    match_reason: MatchReason::ExactName,
                });
            }

            // Partial match (first name)
            if entity_name.to_lowercase().starts_with(&name_lower) {
                return Some(SubjectMatch {
                    entity_id: self.world.humans.ids[i],
                    name: entity_name.clone(),
                    match_reason: MatchReason::PartialName,
                });
            }
        }

        None
    }

    fn resolve_qualification(&self, spec: &str) -> Option<Vec<SubjectMatch>> {
        // Pattern: "a qualified builder", "the best builder", "a skilled fighter"
        let patterns = [
            ("builder", "building_skills"),
            ("fighter", "combat"),
            ("soldier", "combat"),
        ];

        for (keyword, skill_type) in patterns {
            if spec.contains(keyword) {
                return Some(self.find_by_skill(skill_type, 0.5)); // min skill 0.5
            }
        }

        None
    }

    fn find_by_skill(&self, skill_type: &str, min_level: f32) -> Vec<SubjectMatch> {
        let mut matches = Vec::new();

        for i in 0..self.world.humans.ids.len() {
            if !self.world.humans.alive[i] {
                continue;
            }

            let skill_level = match skill_type {
                "building_skills" => self.world.humans.building_skills[i],
                // Add more skill types as needed
                _ => 0.0,
            };

            if skill_level >= min_level {
                matches.push(SubjectMatch {
                    entity_id: self.world.humans.ids[i],
                    name: self.world.humans.names[i].clone(),
                    match_reason: MatchReason::Qualification {
                        skill: skill_type.to_string(),
                        level: skill_level,
                    },
                });
            }
        }

        // Sort by skill level descending
        matches.sort_by(|a, b| {
            let a_level = match &a.match_reason {
                MatchReason::Qualification { level, .. } => *level,
                _ => 0.0,
            };
            let b_level = match &b.match_reason {
                MatchReason::Qualification { level, .. } => *level,
                _ => 0.0,
            };
            b_level.partial_cmp(&a_level).unwrap_or(std::cmp::Ordering::Equal)
        });

        matches
    }

    fn resolve_location(&self, location: &Option<String>) -> Option<Vec2> {
        let loc = location.as_ref()?;
        let loc_lower = loc.to_lowercase();

        // Named locations (expand as needed)
        if loc_lower.contains("center") || loc_lower.contains("middle") {
            return Some(Vec2::new(100.0, 100.0));
        }
        if loc_lower.contains("east") {
            return Some(Vec2::new(180.0, 100.0));
        }
        if loc_lower.contains("west") {
            return Some(Vec2::new(20.0, 100.0));
        }
        if loc_lower.contains("north") {
            return Some(Vec2::new(100.0, 180.0));
        }
        if loc_lower.contains("south") {
            return Some(Vec2::new(100.0, 20.0));
        }

        // TODO: Parse coordinates like "50, 100"

        None
    }

    fn resolve_target(&self, target: &Option<String>) -> Option<EntityId> {
        // For now, no entity targeting by description
        // Would need spatial queries for "that orc" or "nearest enemy"
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_by_name() {
        let mut world = World::new();
        let marcus_id = world.spawn_human("Marcus".into());

        let resolver = IntentResolver::new(&world);
        let matches = resolver.resolve_subjects(&Some(vec!["Marcus".to_string()]));

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].entity_id, marcus_id);
    }

    #[test]
    fn test_resolve_everyone() {
        let mut world = World::new();
        world.spawn_human("Alice".into());
        world.spawn_human("Bob".into());

        let resolver = IntentResolver::new(&world);
        let matches = resolver.resolve_subjects(&Some(vec!["everyone".to_string()]));

        assert_eq!(matches.len(), 2);
    }
}
```

**Step 3: Create executor.rs with task creation**

Create `src/command/executor.rs`:

```rust
//! Command execution - converts resolved intents to tasks

use crate::actions::catalog::ActionId;
use crate::city::building::BuildingType;
use crate::command::resolver::{IntentResolution, IntentResolver};
use crate::core::types::{EntityId, Tick, Vec2};
use crate::ecs::world::World;
use crate::entity::tasks::{Task, TaskPriority, TaskSource};
use crate::llm::parser::{IntentAction, IntentPriority, ParsedIntent};

/// Executes commands by creating tasks for entities
pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute a parsed intent, returning created tasks
    pub fn execute(world: &mut World, intent: &ParsedIntent, tick: Tick) -> ExecutionResult {
        let resolver = IntentResolver::new(world);
        let resolution = resolver.resolve(intent);

        if resolution.subjects.is_empty() && needs_subjects(&intent.action) {
            return ExecutionResult {
                tasks_created: 0,
                assigned_to: Vec::new(),
                error: Some("No matching entities found for command".to_string()),
            };
        }

        let priority = convert_priority(intent.priority);
        let mut tasks_created = 0;
        let mut assigned_to = Vec::new();

        for subject in &resolution.subjects {
            if let Some(task) = create_task(intent, &resolution, subject.entity_id, priority, tick) {
                if let Some(idx) = world.humans.index_of(subject.entity_id) {
                    world.humans.task_queues[idx].push(task);
                    tasks_created += 1;
                    assigned_to.push((subject.entity_id, subject.name.clone()));
                }
            }
        }

        ExecutionResult {
            tasks_created,
            assigned_to,
            error: None,
        }
    }
}

/// Result of executing a command
#[derive(Debug)]
pub struct ExecutionResult {
    pub tasks_created: usize,
    pub assigned_to: Vec<(EntityId, String)>,
    pub error: Option<String>,
}

fn needs_subjects(action: &IntentAction) -> bool {
    !matches!(action, IntentAction::Query)
}

fn convert_priority(priority: IntentPriority) -> TaskPriority {
    match priority {
        IntentPriority::Critical => TaskPriority::Critical,
        IntentPriority::High => TaskPriority::High,
        IntentPriority::Normal => TaskPriority::Normal,
        IntentPriority::Low => TaskPriority::Low,
    }
}

fn create_task(
    intent: &ParsedIntent,
    resolution: &IntentResolution,
    entity_id: EntityId,
    priority: TaskPriority,
    tick: Tick,
) -> Option<Task> {
    let action_id = match intent.action {
        IntentAction::Build => ActionId::Build,
        IntentAction::Craft => ActionId::Craft,
        IntentAction::Combat => ActionId::Attack,
        IntentAction::Gather => ActionId::Gather,
        IntentAction::Move => ActionId::MoveTo,
        IntentAction::Rest => ActionId::Rest,
        IntentAction::Social => ActionId::TalkTo,
        IntentAction::Assign | IntentAction::Query | IntentAction::Unknown => return None,
    };

    let mut task = Task::new(action_id, priority, tick);
    task.source = TaskSource::PlayerCommand;

    // Apply location if resolved
    if let Some(pos) = resolution.location {
        task.target_position = Some(pos);
    }

    // Apply target entity if resolved
    if let Some(target) = resolution.target_entity {
        task.target_entity = Some(target);
    }

    Some(task)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_move_command() {
        let mut world = World::new();
        let marcus_id = world.spawn_human("Marcus".into());

        let intent = ParsedIntent {
            action: IntentAction::Move,
            target: None,
            location: Some("east".to_string()),
            subjects: Some(vec!["Marcus".to_string()]),
            priority: IntentPriority::Normal,
            ambiguous_concepts: Vec::new(),
            confidence: 0.9,
        };

        let result = CommandExecutor::execute(&mut world, &intent, 0);

        assert_eq!(result.tasks_created, 1);
        assert!(result.error.is_none());

        let idx = world.humans.index_of(marcus_id).unwrap();
        let task = world.humans.task_queues[idx].current().unwrap();
        assert_eq!(task.action, ActionId::MoveTo);
        assert!(task.target_position.is_some());
    }
}
```

**Step 4: Register module in lib.rs**

Add to `src/lib.rs`:

```rust
pub mod command;
```

**Step 5: Run tests**

Run: `cargo test command::`
Expected: PASS

**Step 6: Commit**

```bash
git add src/command src/lib.rs
git commit -m "feat(command): add intent resolution and task creation pipeline

- IntentResolver: resolves subjects by name, qualification, or 'everyone'
- CommandExecutor: converts ParsedIntent to Tasks for entities
- Supports location resolution (cardinal directions)
- Foundation for player command execution"
```

---

## Task 3: Integrate Command Pipeline into Main

**Files:**
- Modify: `src/main.rs`

**Step 1: Read current main.rs structure**

Examine the REPL loop to understand where to integrate.

**Step 2: Add command execution to LLM parse branch**

In `src/main.rs`, find the LLM command handling section and update:

```rust
// Add import at top
use arc_citadel::command::CommandExecutor;

// In the main loop, after LLM parsing:
// Find the section that handles non-builtin commands (the else branch)
// Replace the "just print intent" behavior with actual execution:

// BEFORE (somewhere in main.rs):
// println!("Parsed: {:?}", intent);

// AFTER:
match &intent.action {
    IntentAction::Query => {
        // Handle queries (status, info) without creating tasks
        println!("Query: {:?}", intent.target);
    }
    _ => {
        // Execute command
        let result = CommandExecutor::execute(&mut world, &intent, world.current_tick);

        if let Some(error) = &result.error {
            println!("Command failed: {}", error);
        } else if result.tasks_created > 0 {
            println!("Assigned {} task(s) to:", result.tasks_created);
            for (_, name) in &result.assigned_to {
                println!("  - {}", name);
            }
        } else {
            println!("No tasks created");
        }
    }
}
```

**Step 3: Test manually**

Run: `cargo run`
Input: `have Marcus rest`
Expected: "Assigned 1 task(s) to: Marcus"

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat(main): wire command executor to LLM parsing

Player commands now create actual tasks for entities.
'have Marcus rest' creates Rest task for Marcus."
```

---

## Task 4: Add egui Dependencies

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add egui crates**

Add to `Cargo.toml` under `[dependencies]`:

```toml
# UI overlay
egui = "0.28"
egui-wgpu = "0.28"
egui-winit = "0.28"
```

**Step 2: Verify build**

Run: `cargo build`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "deps: add egui for UI overlay

egui, egui-wgpu, egui-winit for GPU-accelerated UI panels"
```

---

## Task 5: Create UI State Module

**Files:**
- Create: `src/ui/state.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Create UI state structure**

Create `src/ui/state.rs`:

```rust
//! UI state management for live simulation

use crate::core::types::EntityId;
use std::collections::VecDeque;

/// Maximum action log entries to keep
const MAX_LOG_ENTRIES: usize = 50;

/// Game UI state
#[derive(Debug, Default)]
pub struct GameUI {
    /// Currently selected entity (if any)
    pub selected_entity: Option<EntityId>,
    /// Action log entries
    pub action_log: VecDeque<LogEntry>,
    /// Whether to show entity panel
    pub show_entity_panel: bool,
    /// Whether to show action log
    pub show_action_log: bool,
    /// Command input buffer
    pub command_input: String,
    /// Whether command input is focused
    pub command_focused: bool,
}

/// An entry in the action log
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub tick: u64,
    pub message: String,
    pub category: LogCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogCategory {
    Action,
    Combat,
    Production,
    System,
}

impl GameUI {
    pub fn new() -> Self {
        Self {
            selected_entity: None,
            action_log: VecDeque::with_capacity(MAX_LOG_ENTRIES),
            show_entity_panel: true,
            show_action_log: true,
            command_input: String::new(),
            command_focused: false,
        }
    }

    /// Add an entry to the action log
    pub fn log(&mut self, tick: u64, message: String, category: LogCategory) {
        if self.action_log.len() >= MAX_LOG_ENTRIES {
            self.action_log.pop_front();
        }
        self.action_log.push_back(LogEntry {
            tick,
            message,
            category,
        });
    }

    /// Select an entity by ID
    pub fn select(&mut self, entity_id: EntityId) {
        self.selected_entity = Some(entity_id);
    }

    /// Clear selection
    pub fn deselect(&mut self) {
        self.selected_entity = None;
    }

    /// Toggle selection
    pub fn toggle_select(&mut self, entity_id: EntityId) {
        if self.selected_entity == Some(entity_id) {
            self.deselect();
        } else {
            self.select(entity_id);
        }
    }
}
```

**Step 2: Update mod.rs**

Replace `src/ui/mod.rs`:

```rust
//! UI module - egui-based overlay for live simulation

pub mod state;

pub use state::{GameUI, LogCategory, LogEntry};
```

**Step 3: Run tests**

Run: `cargo test ui::`
Expected: PASS (or no tests yet)

**Step 4: Commit**

```bash
git add src/ui/
git commit -m "feat(ui): add GameUI state management

- Entity selection tracking
- Action log with categories
- Command input buffer
- Panel visibility toggles"
```

---

## Task 6: Integrate egui into live_sim.rs

**Files:**
- Modify: `src/bin/live_sim.rs`

**Step 1: Add egui setup**

Add imports at top of `src/bin/live_sim.rs`:

```rust
use egui_wgpu::ScreenDescriptor;
use egui_winit::State as EguiWinitState;
use arc_citadel::ui::{GameUI, LogCategory};
```

**Step 2: Initialize egui in main()**

After renderer creation, add:

```rust
// Create egui context and state
let egui_ctx = egui::Context::default();
let mut egui_winit_state = EguiWinitState::new(
    egui_ctx.clone(),
    egui::ViewportId::ROOT,
    &window,
    None,
    None,
    None,
);

// Create egui renderer for wgpu
let mut egui_renderer = egui_wgpu::Renderer::new(
    renderer.device(),
    renderer.surface_format(),
    None,
    1,
    false,
);

// Game UI state
let mut game_ui = GameUI::new();
```

**Step 3: Handle egui input in event loop**

In the event handling section, add egui input processing:

```rust
Event::WindowEvent { event, .. } => {
    // Let egui handle events first
    let egui_consumed = egui_winit_state.on_window_event(&window, &event).consumed;

    if !egui_consumed {
        // Handle game input only if egui didn't consume it
        match event {
            // ... existing game input handling
        }
    }
}
```

**Step 4: Add egui rendering in frame**

After building render state, before presenting:

```rust
// Begin egui frame
let egui_input = egui_winit_state.take_egui_input(&window);
egui_ctx.begin_frame(egui_input);

// Draw UI panels
draw_ui(&egui_ctx, &mut game_ui, &world);

// End egui frame
let egui_output = egui_ctx.end_frame();
egui_winit_state.handle_platform_output(&window, egui_output.platform_output);

// Render egui
let egui_primitives = egui_ctx.tessellate(egui_output.shapes, egui_output.pixels_per_point);
let screen_descriptor = ScreenDescriptor {
    size_in_pixels: [size.width, size.height],
    pixels_per_point: window.scale_factor() as f32,
};

// Upload egui textures
for (id, delta) in &egui_output.textures_delta.set {
    egui_renderer.update_texture(renderer.device(), renderer.queue(), *id, delta);
}

// Render egui to command encoder (integrate with your render pass)
egui_renderer.render(
    &mut encoder,
    &view,
    &egui_primitives,
    &screen_descriptor,
);

// Free textures
for id in &egui_output.textures_delta.free {
    egui_renderer.free_texture(id);
}
```

**Step 5: Create UI drawing function**

Add at end of file:

```rust
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
```

**Step 6: Test**

Run: `cargo run --bin live_sim`
Expected: Window shows with egui panels overlaid

**Step 7: Commit**

```bash
git add src/bin/live_sim.rs
git commit -m "feat(live_sim): integrate egui UI overlay

- Entity panel shows selected entity details
- Action log panel shows recent events
- Status bar shows tick count and entity count
- egui handles input before game controls"
```

---

## Task 7: Add Entity Selection via Mouse Click

**Files:**
- Modify: `src/bin/live_sim.rs`

**Step 1: Track mouse position**

Add mouse position tracking:

```rust
// Add to state variables
let mut mouse_pos: Option<(f32, f32)> = None;

// In event handling:
WindowEvent::CursorMoved { position, .. } => {
    mouse_pos = Some((position.x as f32, position.y as f32));
}
```

**Step 2: Handle mouse click for selection**

Add click handling:

```rust
WindowEvent::MouseInput { state: ElementState::Pressed, button: winit::event::MouseButton::Left, .. } => {
    if !egui_consumed {
        if let Some((mx, my)) = mouse_pos {
            // Convert screen to world coordinates
            let world_pos = camera.screen_to_world(glam::Vec2::new(mx, my));

            // Find entity near click (within 10 units)
            let click_radius = 10.0 / camera.zoom;

            let mut closest: Option<(EntityId, f32)> = None;
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
}
```

**Step 3: Highlight selected entity in rendering**

In render state building, mark selected entity:

```rust
// When building render entities:
let is_selected = game_ui.selected_entity == Some(world.humans.ids[i]);
let color = if is_selected {
    Color::new(1.0, 1.0, 0.0, 1.0) // Yellow highlight
} else {
    // ... existing color logic
};
```

**Step 4: Test**

Run: `cargo run --bin live_sim`
Action: Click on entity
Expected: Entity highlights yellow, panel shows details

**Step 5: Commit**

```bash
git add src/bin/live_sim.rs
git commit -m "feat(live_sim): add entity selection via mouse click

- Click entity to select (yellow highlight)
- Click elsewhere to deselect
- Entity panel updates with selected entity info"
```

---

## Task 8: Wire Combat Resolution

**Files:**
- Create: `src/combat/adapter.rs`
- Modify: `src/combat/mod.rs`
- Modify: `src/simulation/tick.rs`

**Step 1: Create combat adapter**

Create `src/combat/adapter.rs`:

```rust
//! Combat adapter - bridges tick.rs entity data to combat resolution

use crate::combat::resolution::{Combatant, ExchangeResult, resolve_exchange};
use crate::combat::{ArmorProperties, CombatSkill, CombatStance, WeaponProperties};
use crate::core::types::EntityId;
use crate::ecs::world::World;

/// Adapter to resolve combat between entities using their actual stats
pub struct CombatAdapter<'a> {
    world: &'a World,
}

impl<'a> CombatAdapter<'a> {
    pub fn new(world: &'a World) -> Self {
        Self { world }
    }

    /// Resolve an attack between two human entities
    pub fn resolve_attack(
        &self,
        attacker_idx: usize,
        defender_id: EntityId,
        skill_modifier: f32,
    ) -> Option<CombatResult> {
        // Find defender
        let defender_idx = self.world.humans.index_of(defender_id)?;

        // Build combatants from entity data
        let attacker = self.build_combatant(attacker_idx, CombatStance::Pressing, skill_modifier);
        let defender = self.build_combatant(defender_idx, CombatStance::Neutral, 1.0);

        let exchange = resolve_exchange(&attacker, &defender);

        Some(CombatResult {
            attacker_idx,
            defender_idx,
            exchange,
        })
    }

    fn build_combatant(&self, idx: usize, stance: CombatStance, skill_mod: f32) -> Combatant {
        // For MVP: use default weapon/armor, derive skill from building_skills as proxy
        // TODO: Add actual weapon/armor components to entities

        let base_skill = self.world.humans.building_skills[idx]; // Temporary proxy
        let effective_skill = (base_skill * skill_mod).clamp(0.0, 1.0);

        Combatant {
            weapon: WeaponProperties::fists(), // Default: unarmed
            armor: ArmorProperties::none(),    // Default: unarmored
            stance,
            skill: CombatSkill::from_level(effective_skill),
        }
    }
}

/// Result of combat resolution
pub struct CombatResult {
    pub attacker_idx: usize,
    pub defender_idx: usize,
    pub exchange: ExchangeResult,
}

impl CombatResult {
    /// Did the attacker successfully hit?
    pub fn attacker_success(&self) -> bool {
        self.exchange.defender_hit
    }
}
```

**Step 2: Add CombatSkill::from_level**

In `src/combat/mod.rs`, ensure this method exists (or add it):

```rust
impl CombatSkill {
    pub fn from_level(level: f32) -> Self {
        // Convert 0-1 level to skill struct
        Self {
            level: if level >= 0.9 {
                SkillLevel::Master
            } else if level >= 0.7 {
                SkillLevel::Veteran
            } else if level >= 0.4 {
                SkillLevel::Trained
            } else {
                SkillLevel::Novice
            },
            // ... other fields with defaults
        }
    }
}
```

**Step 3: Update combat/mod.rs exports**

```rust
pub mod adapter;
pub use adapter::{CombatAdapter, CombatResult};
```

**Step 4: Replace combat stubs in tick.rs**

In `src/simulation/tick.rs`, replace the Attack stub (around line 1314):

```rust
// BEFORE:
let success = true; // TODO: actual combat resolution

// AFTER:
let success = if let Some(target_id) = task.target_entity {
    let adapter = CombatAdapter::new(world);
    if let Some(result) = adapter.resolve_attack(i, target_id, skill_result.skill_modifier) {
        // Apply wounds to defender
        if let Some(wound) = &result.exchange.defender_wound {
            if wound.severity != WoundSeverity::None {
                // Apply fatigue/damage to defender
                let defender_idx = result.defender_idx;
                world.humans.body_states[defender_idx].fatigue =
                    (world.humans.body_states[defender_idx].fatigue + 0.1).min(1.0);
            }
        }
        result.attacker_success()
    } else {
        false // Target not found
    }
} else {
    false // No target
};
```

Apply similar pattern to Defend, Charge, HoldPosition stubs.

**Step 5: Add imports to tick.rs**

```rust
use crate::combat::adapter::CombatAdapter;
use crate::combat::WoundSeverity;
```

**Step 6: Test**

Run: `cargo test combat::`
Run: `cargo test tick::tests`
Expected: PASS

**Step 7: Commit**

```bash
git add src/combat/adapter.rs src/combat/mod.rs src/simulation/tick.rs
git commit -m "feat(combat): wire resolution system into tick execution

- CombatAdapter bridges entity data to Combatant structs
- Attack action now resolves via resolve_exchange()
- Wounds apply fatigue to defender
- Defend/Charge/HoldPosition use same pattern"
```

---

## Task 9: Enable Production Tick

**Files:**
- Modify: `src/simulation/tick.rs`

**Step 1: Add imports**

```rust
use crate::city::production::tick_production;
use crate::city::recipe::RecipeCatalog;
```

**Step 2: Add recipe catalog to World or pass as parameter**

For MVP, create catalog inline in tick:

```rust
// In run_simulation_tick, replace the TODO comment (around line 69-73):

// BEFORE:
// TODO: Run production tick for buildings
// This requires a stockpile per settlement - for MVP, use a global stockpile

// AFTER:
// Run production tick for buildings
let recipes = RecipeCatalog::default(); // TODO: Load from config
let production_results = tick_production(&mut world.buildings, &recipes, &mut world.stockpile);

// Log production completions (optional)
for result in production_results {
    tracing::debug!(
        "Production complete: building {} produced {}",
        result.building_idx,
        result.recipe_id
    );
}
```

**Step 3: Ensure RecipeCatalog has default**

Check `src/city/recipe.rs` - if no Default impl, add:

```rust
impl Default for RecipeCatalog {
    fn default() -> Self {
        Self::new() // Or basic recipes
    }
}
```

**Step 4: Test**

Run: `cargo test production::`
Run: `cargo build`
Expected: Compiles and tests pass

**Step 5: Commit**

```bash
git add src/simulation/tick.rs src/city/recipe.rs
git commit -m "feat(production): enable production tick in simulation

Production system now runs each tick:
- Buildings with active recipes advance progress
- Completed recipes consume inputs, produce outputs
- Results logged for debugging"
```

---

## Task 10: Add Action Logging to UI

**Files:**
- Modify: `src/simulation/tick.rs`
- Modify: `src/bin/live_sim.rs`

**Step 1: Create action event system**

Add to `src/simulation/tick.rs` or create `src/simulation/events.rs`:

```rust
/// Events generated during simulation tick
#[derive(Debug, Clone)]
pub enum SimulationEvent {
    TaskStarted { entity_name: String, action: ActionId },
    TaskCompleted { entity_name: String, action: ActionId },
    CombatHit { attacker: String, defender: String },
    ProductionComplete { building_idx: usize, recipe: String },
}
```

**Step 2: Collect events during tick**

Modify `run_simulation_tick` to return events:

```rust
pub fn run_simulation_tick(world: &mut World) -> Vec<SimulationEvent> {
    let mut events = Vec::new();

    // ... existing tick logic ...

    // When task starts/completes, push event:
    events.push(SimulationEvent::TaskStarted {
        entity_name: world.humans.names[i].clone(),
        action: task.action,
    });

    events
}
```

**Step 3: Feed events to UI in live_sim.rs**

```rust
// In main loop:
if !paused {
    let events = run_simulation_tick(&mut world);

    for event in events {
        let msg = match event {
            SimulationEvent::TaskStarted { entity_name, action } => {
                format!("{} started {:?}", entity_name, action)
            }
            SimulationEvent::TaskCompleted { entity_name, action } => {
                format!("{} completed {:?}", entity_name, action)
            }
            SimulationEvent::CombatHit { attacker, defender } => {
                format!("{} hit {}", attacker, defender)
            }
            SimulationEvent::ProductionComplete { recipe, .. } => {
                format!("Produced: {}", recipe)
            }
        };
        game_ui.log(world.current_tick, msg, LogCategory::Action);
    }
}
```

**Step 4: Test**

Run: `cargo run --bin live_sim`
Expected: Action log shows events as simulation runs

**Step 5: Commit**

```bash
git add src/simulation/tick.rs src/simulation/events.rs src/bin/live_sim.rs
git commit -m "feat(ui): add action logging from simulation events

Simulation tick now returns events:
- TaskStarted, TaskCompleted
- CombatHit
- ProductionComplete

Events displayed in action log panel"
```

---

## Verification Checklist

After completing all tasks:

1. **Building tests pass:**
   ```bash
   cargo test test_build_action
   # Expected: 3 PASS
   ```

2. **Full test suite passes:**
   ```bash
   cargo test
   # Expected: All PASS
   ```

3. **Live sim runs with UI:**
   ```bash
   cargo run --bin live_sim
   # Expected: Window with entities, UI panels, action log
   ```

4. **Commands work in main:**
   ```bash
   cargo run
   # Input: "have Marcus rest"
   # Expected: Task assigned
   ```

5. **Combat resolves:**
   ```bash
   # In live_sim, spawn orcs and watch combat
   # Expected: Attacks apply fatigue to targets
   ```

---

## Summary

| Task | Feature | Files Changed |
|------|---------|---------------|
| 1 | Fix building math | tick.rs |
| 2-3 | Command pipeline | command/*, main.rs |
| 4-7 | egui UI overlay | Cargo.toml, ui/*, live_sim.rs |
| 8 | Combat integration | combat/adapter.rs, tick.rs |
| 9 | Production system | tick.rs |
| 10 | Action logging | tick.rs, live_sim.rs |
