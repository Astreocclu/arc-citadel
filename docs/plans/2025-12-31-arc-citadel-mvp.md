# Arc Citadel MVP Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a working Rust simulation game with emergent entity behavior (perception→thought→action loop), terminal UI, and LLM command parsing.

**Architecture:** Custom ECS with Structure-of-Arrays (SoA) data layout. Species-specific archetypes (MVP: humans only). Sparse hash grid for spatial queries. Async LLM client via tokio. Behavior emerges from values/needs/thoughts - NO scripted events, NO percentage modifiers.

**Tech Stack:** Rust 2021, tokio (async), serde/serde_json (serialization), reqwest (HTTP), uuid, ahash, thiserror, tracing, crossterm/ratatui (terminal UI)

---

## Critical Design Constraints

Before implementing ANY code, internalize these rules:

1. **NO PERCENTAGE MODIFIERS** - Properties interact, outcomes emerge
2. **Species values are DIFFERENT TYPES** - HumanValues and DwarfValues are incompatible structs
3. **LLM parses commands ONLY** - Never controls entity behavior
4. **Emergence over scripting** - No `if coward then flee` hardcoding

---

## Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs` (placeholder)
- Create: `src/lib.rs` (placeholder)
- Create: `.gitignore`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "arc-citadel"
version = "0.1.0"
edition = "2021"
authors = ["Arc Citadel Team"]
description = "Deep simulation strategy game with natural language commands"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rand = "0.8"
rand_chacha = "0.3"
ordered-float = "4.2"
uuid = { version = "1.6", features = ["v4", "serde"] }
reqwest = { version = "0.11", features = ["json"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
crossterm = "0.27"
ratatui = "0.25"
ahash = "0.8"
thiserror = "1.0"
derive_more = "0.99"

[dev-dependencies]
criterion = "0.5"
proptest = "1.4"

[profile.release]
lto = true
codegen-units = 1
opt-level = 3

[[bench]]
name = "simulation_bench"
harness = false
```

**Step 2: Create placeholder main.rs**

```rust
fn main() {
    println!("Arc Citadel - scaffolding complete");
}
```

**Step 3: Create placeholder lib.rs**

```rust
//! Arc Citadel - Deep Simulation Strategy Game
```

**Step 4: Create .gitignore**

```
/target
Cargo.lock
.env
*.log
```

**Step 5: Verify build**

Run: `cd /home/astre/arc-citadel && cargo build`
Expected: Compiles successfully

**Step 6: Initialize git and commit**

```bash
cd /home/astre/arc-citadel
git init
git add .
git commit -m "chore: initial project scaffolding"
```

---

## Task 2: Directory Structure

**Files:**
- Create all module directories

**Step 1: Create all directories**

```bash
cd /home/astre/arc-citadel
mkdir -p src/{core,ecs,spatial,entity/species,genetics,simulation,actions,combat,llm,campaign,battle,ui,data}
mkdir -p data/species
mkdir -p tests
```

**Step 2: Create all mod.rs files**

Create `src/core/mod.rs`:
```rust
pub mod types;
pub mod error;
pub mod config;
```

Create `src/ecs/mod.rs`:
```rust
pub mod world;
```

Create `src/spatial/mod.rs`:
```rust
pub mod grid;
pub mod sparse_hash;
pub mod flow_field;
```

Create `src/entity/mod.rs`:
```rust
pub mod identity;
pub mod body;
pub mod needs;
pub mod thoughts;
pub mod relationships;
pub mod tasks;
pub mod species;
```

Create `src/entity/species/mod.rs`:
```rust
pub mod human;
```

Create `src/genetics/mod.rs`:
```rust
pub mod genome;
pub mod phenotype;
pub mod personality;
pub mod values;
```

Create `src/simulation/mod.rs`:
```rust
pub mod perception;
pub mod thought_gen;
pub mod action_select;
pub mod action_execute;
pub mod tick;
```

Create `src/actions/mod.rs`:
```rust
pub mod catalog;
pub mod movement;
pub mod survival;
pub mod work;
pub mod social;
pub mod combat;
```

Create `src/combat/mod.rs`:
```rust
pub mod resolution;
pub mod weapons;
pub mod armor;
pub mod wounds;
pub mod morale;
```

Create `src/llm/mod.rs`:
```rust
pub mod client;
pub mod parser;
pub mod context;
pub mod species_interpret;
pub mod prompts;
```

Create `src/campaign/mod.rs`:
```rust
pub mod map;
pub mod location;
pub mod route;
pub mod weather;
pub mod supply;
```

Create `src/battle/mod.rs`:
```rust
pub mod battle_map;
pub mod planning;
pub mod execution;
pub mod courier;
pub mod resolution;
```

Create `src/ui/mod.rs`:
```rust
pub mod terminal;
pub mod input;
pub mod display;
```

Create `src/data/mod.rs`:
```rust
// Data loading utilities
```

**Step 3: Update src/lib.rs**

```rust
//! Arc Citadel - Deep Simulation Strategy Game

pub mod core;
pub mod ecs;
pub mod spatial;
pub mod entity;
pub mod genetics;
pub mod simulation;
pub mod actions;
pub mod combat;
pub mod llm;
pub mod campaign;
pub mod battle;
pub mod ui;
pub mod data;
```

**Step 4: Commit**

```bash
git add .
git commit -m "chore: create module directory structure"
```

---

## Task 3: Core Types

**Files:**
- Create: `src/core/types.rs`
- Create: `src/core/error.rs`
- Create: `src/core/config.rs` (stub)

**Step 1: Create types.rs**

```rust
//! Core type definitions used throughout the codebase

use uuid::Uuid;

/// Unique identifier for entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

/// Game tick counter (simulation time unit)
pub type Tick = u64;

/// Location identifier for campaign map
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocationId(pub u32);

/// Flow field identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlowFieldId(pub u32);

/// Species enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Species {
    Human,
    Dwarf,
    Elf,
}

/// 2D position
#[derive(Debug, Clone, Copy, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0001 {
            Self { x: self.x / len, y: self.y / len }
        } else {
            Self::default()
        }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self { x: self.x * rhs, y: self.y * rhs }
    }
}
```

**Step 2: Create error.rs**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArcError {
    #[error("Entity not found: {0:?}")]
    EntityNotFound(crate::core::types::EntityId),

    #[error("Component not found for entity: {0}")]
    ComponentNotFound(String),

    #[error("Invalid action: {0}")]
    InvalidAction(String),

    #[error("LLM error: {0}")]
    LlmError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Navigation error: {0}")]
    NavigationError(String),
}

pub type Result<T> = std::result::Result<T, ArcError>;
```

**Step 3: Create config.rs stub**

```rust
//! Configuration loading - stub for MVP
```

**Step 4: Verify build**

Run: `cargo build`
Expected: Compiles (will have unused warnings, that's fine)

**Step 5: Commit**

```bash
git add .
git commit -m "feat(core): add types, error handling"
```

---

## Task 4: Spatial Systems

**Files:**
- Create: `src/spatial/grid.rs`
- Create: `src/spatial/sparse_hash.rs`
- Create: `src/spatial/flow_field.rs` (stub)

**Step 1: Create grid.rs**

```rust
//! Generic grid for spatial data

use crate::core::types::Vec2;

/// Generic 2D grid with configurable cell size
#[derive(Debug, Clone)]
pub struct Grid<T: Clone + Default> {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub origin: Vec2,
    data: Vec<T>,
}

impl<T: Clone + Default> Grid<T> {
    pub fn new(width: usize, height: usize, cell_size: f32, origin: Vec2) -> Self {
        Self {
            width,
            height,
            cell_size,
            origin,
            data: vec![T::default(); width * height],
        }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x < self.width && y < self.height {
            Some(&self.data[y * self.width + x])
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x < self.width && y < self.height {
            Some(&mut self.data[y * self.width + x])
        } else {
            None
        }
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, value: T) {
        if x < self.width && y < self.height {
            self.data[y * self.width + x] = value;
        }
    }

    /// Convert world position to cell coordinates
    #[inline]
    pub fn world_to_cell(&self, pos: Vec2) -> (usize, usize) {
        let x = ((pos.x - self.origin.x) / self.cell_size).floor() as i32;
        let y = ((pos.y - self.origin.y) / self.cell_size).floor() as i32;
        (
            x.max(0).min(self.width as i32 - 1) as usize,
            y.max(0).min(self.height as i32 - 1) as usize,
        )
    }

    /// Sample grid at world position
    pub fn sample(&self, pos: Vec2) -> Option<&T> {
        let (x, y) = self.world_to_cell(pos);
        self.get(x, y)
    }

    /// Cell center in world coordinates
    pub fn cell_center(&self, x: usize, y: usize) -> Vec2 {
        Vec2::new(
            self.origin.x + (x as f32 + 0.5) * self.cell_size,
            self.origin.y + (y as f32 + 0.5) * self.cell_size,
        )
    }
}
```

**Step 2: Create sparse_hash.rs**

```rust
//! Sparse hash grid for efficient spatial queries

use ahash::AHashMap;
use crate::core::types::{EntityId, Vec2};

/// Sparse hash grid for O(1) neighbor queries
pub struct SparseHashGrid {
    cell_size: f32,
    cells: AHashMap<(i32, i32), Vec<EntityId>>,
}

impl SparseHashGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: AHashMap::new(),
        }
    }

    #[inline]
    fn cell_coord(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn insert(&mut self, entity: EntityId, pos: Vec2) {
        let coord = self.cell_coord(pos);
        self.cells.entry(coord).or_default().push(entity);
    }

    pub fn remove(&mut self, entity: EntityId, pos: Vec2) {
        let coord = self.cell_coord(pos);
        if let Some(cell) = self.cells.get_mut(&coord) {
            cell.retain(|&e| e != entity);
        }
    }

    /// Query all entities in neighboring cells (3x3 neighborhood)
    pub fn query_neighbors(&self, pos: Vec2) -> impl Iterator<Item = EntityId> + '_ {
        let (cx, cy) = self.cell_coord(pos);

        (-1..=1).flat_map(move |dx| {
            (-1..=1).flat_map(move |dy| {
                self.cells.get(&(cx + dx, cy + dy))
                    .into_iter()
                    .flatten()
                    .copied()
            })
        })
    }

    /// Query entities within radius
    pub fn query_radius(&self, center: Vec2, radius: f32, positions: &[Vec2]) -> Vec<EntityId> {
        let radius_sq = radius * radius;
        self.query_neighbors(center)
            .filter(|&entity| {
                let idx = entity.0.as_u128() as usize % positions.len();
                positions.get(idx)
                    .map(|pos| center.distance(pos) <= radius)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Rebuild grid from positions
    pub fn rebuild<'a>(&mut self, entities: impl Iterator<Item = (EntityId, Vec2)>) {
        self.clear();
        for (entity, pos) in entities {
            self.insert(entity, pos);
        }
    }
}
```

**Step 3: Create flow_field.rs stub**

```rust
//! Flow field navigation - stub for MVP
```

**Step 4: Verify build**

Run: `cargo build`
Expected: Compiles

**Step 5: Commit**

```bash
git add .
git commit -m "feat(spatial): add grid and sparse hash grid"
```

---

## Task 5: Entity Components - Body and Needs

**Files:**
- Create: `src/entity/identity.rs` (stub)
- Create: `src/entity/body.rs`
- Create: `src/entity/needs.rs`
- Create: `src/entity/relationships.rs` (stub)

**Step 1: Create identity.rs stub**

```rust
//! Entity identity - stub for MVP
```

**Step 2: Create body.rs**

```rust
//! Physical body simulation

use serde::{Deserialize, Serialize};

/// Physical state of an entity's body
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BodyState {
    /// 0.0 = fresh, 1.0 = exhausted
    pub fatigue: f32,
    /// 0.0 = fed, 1.0 = starving
    pub hunger: f32,
    /// 0.0 = none, 1.0 = incapacitating
    pub pain: f32,
    /// Computed from wounds
    pub overall_health: f32,
}

impl BodyState {
    pub fn new() -> Self {
        Self {
            fatigue: 0.0,
            hunger: 0.0,
            pain: 0.0,
            overall_health: 1.0,
        }
    }

    /// Check if entity can act
    pub fn can_act(&self) -> bool {
        self.overall_health > 0.1 && self.fatigue < 0.95 && self.pain < 0.9
    }

    /// Check if entity can move
    pub fn can_move(&self) -> bool {
        self.can_act() && self.fatigue < 0.9
    }

    /// Apply fatigue from activity
    pub fn add_fatigue(&mut self, amount: f32) {
        self.fatigue = (self.fatigue + amount).min(1.0);
    }

    /// Recover fatigue from rest
    pub fn recover_fatigue(&mut self, amount: f32) {
        self.fatigue = (self.fatigue - amount).max(0.0);
    }
}

/// Individual wound on a body part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wound {
    pub body_part: BodyPart,
    pub wound_type: WoundType,
    pub severity: f32,
    pub infected: bool,
    pub tick_received: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyPart {
    Head,
    Torso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WoundType {
    Cut,
    Pierce,
    Blunt,
    Burn,
}

/// Collection of wounds on an entity
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Wounds {
    pub wounds: Vec<Wound>,
}

impl Wounds {
    pub fn new() -> Self {
        Self { wounds: Vec::new() }
    }

    pub fn add(&mut self, wound: Wound) {
        self.wounds.push(wound);
    }

    /// Calculate overall health from wounds
    pub fn calculate_health(&self) -> f32 {
        if self.wounds.is_empty() {
            return 1.0;
        }

        let total_severity: f32 = self.wounds.iter()
            .map(|w| w.severity)
            .sum();

        (1.0 - total_severity).max(0.0)
    }

    /// Check if any limb prevents movement
    pub fn can_walk(&self) -> bool {
        !self.wounds.iter().any(|w| {
            matches!(w.body_part, BodyPart::LeftLeg | BodyPart::RightLeg)
                && w.severity > 0.5
        })
    }
}
```

**Step 3: Create needs.rs**

```rust
//! Universal needs that drive entity behavior

use serde::{Deserialize, Serialize};

/// Universal needs shared by all species
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Needs {
    /// 0.0 = fully rested, 1.0 = desperate for rest
    pub rest: f32,
    /// 0.0 = fed, 1.0 = starving
    pub food: f32,
    /// 0.0 = safe, 1.0 = in mortal danger
    pub safety: f32,
    /// 0.0 = socially satisfied, 1.0 = lonely
    pub social: f32,
    /// 0.0 = has purpose, 1.0 = aimless
    pub purpose: f32,
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            rest: 0.2,
            food: 0.2,
            safety: 0.1,
            social: 0.3,
            purpose: 0.3,
        }
    }
}

impl Needs {
    /// Get most pressing need
    pub fn most_pressing(&self) -> (NeedType, f32) {
        let needs = [
            (NeedType::Rest, self.rest),
            (NeedType::Food, self.food),
            (NeedType::Safety, self.safety),
            (NeedType::Social, self.social),
            (NeedType::Purpose, self.purpose),
        ];
        needs.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }

    /// Check if any need is critical (> 0.8)
    pub fn has_critical(&self) -> Option<NeedType> {
        if self.safety > 0.8 { return Some(NeedType::Safety); }
        if self.food > 0.8 { return Some(NeedType::Food); }
        if self.rest > 0.8 { return Some(NeedType::Rest); }
        None
    }

    /// Decay needs over time (called each tick)
    pub fn decay(&mut self, dt: f32, is_active: bool) {
        let activity_mult = if is_active { 1.5 } else { 1.0 };

        self.rest += 0.001 * dt * activity_mult;
        self.food += 0.0005 * dt;
        self.social += 0.0003 * dt;
        self.purpose += 0.0002 * dt;

        self.safety = (self.safety - 0.01 * dt).max(0.0);

        self.rest = self.rest.min(1.0);
        self.food = self.food.min(1.0);
        self.social = self.social.min(1.0);
        self.purpose = self.purpose.min(1.0);
    }

    /// Satisfy a need
    pub fn satisfy(&mut self, need: NeedType, amount: f32) {
        match need {
            NeedType::Rest => self.rest = (self.rest - amount).max(0.0),
            NeedType::Food => self.food = (self.food - amount).max(0.0),
            NeedType::Safety => self.safety = (self.safety - amount).max(0.0),
            NeedType::Social => self.social = (self.social - amount).max(0.0),
            NeedType::Purpose => self.purpose = (self.purpose - amount).max(0.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeedType {
    Rest,
    Food,
    Safety,
    Social,
    Purpose,
}
```

**Step 4: Create relationships.rs stub**

```rust
//! Relationship tracking - stub for MVP
```

**Step 5: Verify build and commit**

```bash
cargo build
git add .
git commit -m "feat(entity): add body state and needs systems"
```

---

## Task 6: Thoughts and Tasks

**Files:**
- Create: `src/entity/thoughts.rs`
- Create: `src/entity/tasks.rs`

**Step 1: Create thoughts.rs**

```rust
//! Thought generation and management

use crate::core::types::EntityId;
use serde::{Deserialize, Serialize};

/// A thought is a reaction to a perceived event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    pub valence: Valence,
    pub intensity: f32,
    pub concept_category: String,
    pub cause_description: String,
    pub cause_type: CauseType,
    pub cause_entity: Option<EntityId>,
    pub created_tick: u64,
    pub decay_rate: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Valence {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CauseType {
    Object,
    Entity,
    Action,
    Need,
    Event,
}

impl Thought {
    pub fn new(
        valence: Valence,
        intensity: f32,
        concept: impl Into<String>,
        description: impl Into<String>,
        cause_type: CauseType,
        tick: u64,
    ) -> Self {
        Self {
            valence,
            intensity,
            concept_category: concept.into(),
            cause_description: description.into(),
            cause_type,
            cause_entity: None,
            created_tick: tick,
            decay_rate: 0.01,
        }
    }

    pub fn decay(&mut self) {
        self.intensity = (self.intensity - self.decay_rate).max(0.0);
    }

    pub fn is_faded(&self) -> bool {
        self.intensity < 0.1
    }
}

/// Buffer of active thoughts
#[derive(Debug, Clone, Default)]
pub struct ThoughtBuffer {
    thoughts: Vec<Thought>,
    max_thoughts: usize,
}

impl ThoughtBuffer {
    pub fn new() -> Self {
        Self {
            thoughts: Vec::new(),
            max_thoughts: 20,
        }
    }

    pub fn add(&mut self, thought: Thought) {
        if self.thoughts.len() >= self.max_thoughts {
            if let Some(pos) = self.thoughts
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.intensity.partial_cmp(&b.intensity).unwrap())
                .map(|(i, _)| i)
            {
                if self.thoughts[pos].intensity < thought.intensity {
                    self.thoughts.remove(pos);
                } else {
                    return;
                }
            }
        }
        self.thoughts.push(thought);
    }

    pub fn decay_all(&mut self) {
        for thought in &mut self.thoughts {
            thought.decay();
        }
        self.thoughts.retain(|t| !t.is_faded());
    }

    pub fn strongest(&self) -> Option<&Thought> {
        self.thoughts.iter()
            .max_by(|a, b| a.intensity.partial_cmp(&b.intensity).unwrap())
    }

    pub fn about_entity(&self, entity: EntityId) -> impl Iterator<Item = &Thought> {
        self.thoughts.iter()
            .filter(move |t| t.cause_entity == Some(entity))
    }

    pub fn positive(&self) -> impl Iterator<Item = &Thought> {
        self.thoughts.iter().filter(|t| t.valence == Valence::Positive)
    }

    pub fn negative(&self) -> impl Iterator<Item = &Thought> {
        self.thoughts.iter().filter(|t| t.valence == Valence::Negative)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Thought> {
        self.thoughts.iter()
    }
}
```

**Step 2: Create tasks.rs**

```rust
//! Task queue and execution

use crate::core::types::{EntityId, Vec2, Tick};
use crate::actions::catalog::ActionId;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A task is an action with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub action: ActionId,
    pub target_position: Option<Vec2>,
    pub target_entity: Option<EntityId>,
    pub priority: TaskPriority,
    pub created_tick: Tick,
    pub progress: f32,
    pub source: TaskSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Critical,
    High,
    Normal,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskSource {
    PlayerCommand,
    Autonomous,
    Reaction,
}

impl Task {
    pub fn new(action: ActionId, priority: TaskPriority, tick: Tick) -> Self {
        Self {
            action,
            target_position: None,
            target_entity: None,
            priority,
            created_tick: tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }
    }

    pub fn with_position(mut self, pos: Vec2) -> Self {
        self.target_position = Some(pos);
        self
    }

    pub fn with_entity(mut self, entity: EntityId) -> Self {
        self.target_entity = Some(entity);
        self
    }

    pub fn from_player(mut self) -> Self {
        self.source = TaskSource::PlayerCommand;
        self
    }
}

/// Queue of tasks for an entity
#[derive(Debug, Clone, Default)]
pub struct TaskQueue {
    current: Option<Task>,
    queued: VecDeque<Task>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            current: None,
            queued: VecDeque::new(),
        }
    }

    pub fn current(&self) -> Option<&Task> {
        self.current.as_ref()
    }

    pub fn current_mut(&mut self) -> Option<&mut Task> {
        self.current.as_mut()
    }

    pub fn push(&mut self, task: Task) {
        let pos = self.queued.iter()
            .position(|t| task.priority as u8 > t.priority as u8)
            .unwrap_or(self.queued.len());
        self.queued.insert(pos, task);

        if self.current.is_none() {
            self.current = self.queued.pop_front();
        }
    }

    pub fn complete_current(&mut self) {
        self.current = self.queued.pop_front();
    }

    pub fn cancel_current(&mut self) {
        self.current = self.queued.pop_front();
    }

    pub fn clear(&mut self) {
        self.current = None;
        self.queued.clear();
    }

    pub fn is_idle(&self) -> bool {
        self.current.is_none() && self.queued.is_empty()
    }
}
```

**Step 3: Verify build and commit**

```bash
cargo build
git add .
git commit -m "feat(entity): add thoughts and task queue"
```

---

## Task 7: Action Catalog

**Files:**
- Create: `src/actions/catalog.rs`
- Create: `src/actions/movement.rs` (stub)
- Create: `src/actions/survival.rs` (stub)
- Create: `src/actions/work.rs` (stub)
- Create: `src/actions/social.rs` (stub)
- Create: `src/actions/combat.rs` (stub)

**Step 1: Create catalog.rs**

```rust
//! Action definitions and catalog

use serde::{Deserialize, Serialize};
use crate::entity::needs::NeedType;

/// Unique action identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionId {
    MoveTo,
    Follow,
    Flee,
    Rest,
    Eat,
    SeekSafety,
    Build,
    Craft,
    Gather,
    Repair,
    TalkTo,
    Help,
    Trade,
    Attack,
    Defend,
    Charge,
    HoldPosition,
    IdleWander,
    IdleObserve,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCategory {
    Movement,
    Survival,
    Work,
    Social,
    Combat,
    Idle,
}

impl ActionId {
    pub fn category(&self) -> ActionCategory {
        match self {
            ActionId::MoveTo | ActionId::Follow | ActionId::Flee => ActionCategory::Movement,
            ActionId::Rest | ActionId::Eat | ActionId::SeekSafety => ActionCategory::Survival,
            ActionId::Build | ActionId::Craft | ActionId::Gather | ActionId::Repair => ActionCategory::Work,
            ActionId::TalkTo | ActionId::Help | ActionId::Trade => ActionCategory::Social,
            ActionId::Attack | ActionId::Defend | ActionId::Charge | ActionId::HoldPosition => ActionCategory::Combat,
            ActionId::IdleWander | ActionId::IdleObserve => ActionCategory::Idle,
        }
    }

    pub fn satisfies_needs(&self) -> Vec<(NeedType, f32)> {
        match self {
            ActionId::Rest => vec![(NeedType::Rest, 0.3)],
            ActionId::Eat => vec![(NeedType::Food, 0.5)],
            ActionId::SeekSafety | ActionId::Flee => vec![(NeedType::Safety, 0.3)],
            ActionId::TalkTo | ActionId::Help => vec![(NeedType::Social, 0.3)],
            ActionId::Build | ActionId::Craft | ActionId::Gather => vec![(NeedType::Purpose, 0.3)],
            _ => vec![],
        }
    }

    pub fn is_interruptible(&self) -> bool {
        match self {
            ActionId::Attack | ActionId::Charge => false,
            _ => true,
        }
    }

    pub fn base_duration(&self) -> u32 {
        match self {
            ActionId::Attack | ActionId::Defend => 1,
            ActionId::TalkTo => 60,
            ActionId::Rest => 600,
            ActionId::Eat => 30,
            ActionId::Build => 3600,
            ActionId::Craft => 1800,
            _ => 0,
        }
    }
}

pub struct ActionAvailability {
    pub available: bool,
    pub reason: Option<String>,
}

impl ActionAvailability {
    pub fn yes() -> Self {
        Self { available: true, reason: None }
    }

    pub fn no(reason: impl Into<String>) -> Self {
        Self { available: false, reason: Some(reason.into()) }
    }
}
```

**Step 2: Create stub files**

Create `src/actions/movement.rs`:
```rust
//! Movement actions - stub for MVP
```

Create `src/actions/survival.rs`:
```rust
//! Survival actions - stub for MVP
```

Create `src/actions/work.rs`:
```rust
//! Work actions - stub for MVP
```

Create `src/actions/social.rs`:
```rust
//! Social actions - stub for MVP
```

Create `src/actions/combat.rs`:
```rust
//! Combat actions - stub for MVP
```

**Step 3: Verify build and commit**

```bash
cargo build
git add .
git commit -m "feat(actions): add action catalog"
```

---

## Task 8: Human Archetype (SoA)

**Files:**
- Create: `src/entity/species/human.rs`

**Step 1: Create human.rs**

```rust
//! Human-specific archetype with SoA layout

use crate::core::types::{EntityId, Vec2, Tick};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::thoughts::ThoughtBuffer;
use crate::entity::tasks::TaskQueue;

/// Human-specific value vocabulary
#[derive(Debug, Clone, Default)]
pub struct HumanValues {
    pub honor: f32,
    pub beauty: f32,
    pub comfort: f32,
    pub ambition: f32,
    pub loyalty: f32,
    pub love: f32,
    pub justice: f32,
    pub curiosity: f32,
    pub safety: f32,
    pub piety: f32,
}

impl HumanValues {
    pub fn dominant(&self) -> (&'static str, f32) {
        let values = [
            ("honor", self.honor),
            ("beauty", self.beauty),
            ("comfort", self.comfort),
            ("ambition", self.ambition),
            ("loyalty", self.loyalty),
            ("love", self.love),
            ("justice", self.justice),
            ("curiosity", self.curiosity),
            ("safety", self.safety),
            ("piety", self.piety),
        ];
        values.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }
}

/// Structure of Arrays for human entities
pub struct HumanArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub birth_ticks: Vec<Tick>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<HumanValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
}

impl HumanArchetype {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            names: Vec::new(),
            birth_ticks: Vec::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
            body_states: Vec::new(),
            needs: Vec::new(),
            thoughts: Vec::new(),
            values: Vec::new(),
            task_queues: Vec::new(),
            alive: Vec::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.ids.len()
    }

    pub fn spawn(&mut self, id: EntityId, name: String, tick: Tick) {
        self.ids.push(id);
        self.names.push(name);
        self.birth_ticks.push(tick);
        self.positions.push(Vec2::default());
        self.velocities.push(Vec2::default());
        self.body_states.push(BodyState::default());
        self.needs.push(Needs::default());
        self.thoughts.push(ThoughtBuffer::new());
        self.values.push(HumanValues::default());
        self.task_queues.push(TaskQueue::new());
        self.alive.push(true);
    }

    pub fn index_of(&self, id: EntityId) -> Option<usize> {
        self.ids.iter().position(|&e| e == id)
    }

    pub fn iter_living(&self) -> impl Iterator<Item = usize> + '_ {
        self.alive.iter()
            .enumerate()
            .filter(|(_, &alive)| alive)
            .map(|(i, _)| i)
    }
}

impl Default for HumanArchetype {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Verify build and commit**

```bash
cargo build
git add .
git commit -m "feat(entity): add human archetype with SoA layout"
```

---

## Task 9: ECS World

**Files:**
- Create: `src/ecs/world.rs`

**Step 1: Create world.rs**

```rust
//! ECS World - manages all entities and their components

use ahash::AHashMap;
use crate::core::types::{EntityId, Species};
use crate::entity::species::human::HumanArchetype;

/// The game world containing all entities
pub struct World {
    pub current_tick: u64,
    entity_registry: AHashMap<EntityId, (Species, usize)>,
    pub humans: HumanArchetype,
    next_indices: AHashMap<Species, usize>,
}

impl World {
    pub fn new() -> Self {
        let mut next_indices = AHashMap::new();
        next_indices.insert(Species::Human, 0);
        next_indices.insert(Species::Dwarf, 0);
        next_indices.insert(Species::Elf, 0);

        Self {
            current_tick: 0,
            entity_registry: AHashMap::new(),
            humans: HumanArchetype::new(),
            next_indices,
        }
    }

    pub fn spawn_human(&mut self, name: String) -> EntityId {
        let entity_id = EntityId::new();
        let index = *self.next_indices.get(&Species::Human).unwrap();

        self.humans.spawn(entity_id, name, self.current_tick);

        self.entity_registry.insert(entity_id, (Species::Human, index));
        *self.next_indices.get_mut(&Species::Human).unwrap() += 1;

        entity_id
    }

    pub fn get_entity_info(&self, entity_id: EntityId) -> Option<(Species, usize)> {
        self.entity_registry.get(&entity_id).copied()
    }

    pub fn entity_count(&self) -> usize {
        self.humans.count()
    }

    pub fn tick(&mut self) {
        self.current_tick += 1;
    }

    pub fn human_entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entity_registry
            .iter()
            .filter(|(_, (species, _))| *species == Species::Human)
            .map(|(id, _)| *id)
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Verify build and commit**

```bash
cargo build
git add .
git commit -m "feat(ecs): add world with entity management"
```

---

## Task 10: Perception System

**Files:**
- Create: `src/simulation/perception.rs`
- Create: `src/simulation/thought_gen.rs` (stub)
- Create: `src/simulation/action_execute.rs` (stub)

**Step 1: Create perception.rs**

```rust
//! Perception system - what entities notice based on their values

use crate::core::types::{EntityId, Vec2};
use crate::entity::species::human::HumanValues;
use crate::spatial::sparse_hash::SparseHashGrid;

#[derive(Debug, Clone)]
pub struct Perception {
    pub observer: EntityId,
    pub perceived_entities: Vec<PerceivedEntity>,
    pub perceived_objects: Vec<PerceivedObject>,
    pub perceived_events: Vec<PerceivedEvent>,
}

#[derive(Debug, Clone)]
pub struct PerceivedEntity {
    pub entity: EntityId,
    pub distance: f32,
    pub relationship: RelationshipType,
    pub threat_level: f32,
    pub notable_features: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipType {
    Unknown,
    Ally,
    Neutral,
    Hostile,
}

#[derive(Debug, Clone)]
pub struct PerceivedObject {
    pub object_type: String,
    pub position: Vec2,
    pub properties: Vec<ObjectProperty>,
}

#[derive(Debug, Clone)]
pub struct ObjectProperty {
    pub name: String,
    pub value: PropertyValue,
}

#[derive(Debug, Clone)]
pub enum PropertyValue {
    Quality(f32),
    Material(String),
    Condition(f32),
    Aesthetic(f32),
}

#[derive(Debug, Clone)]
pub struct PerceivedEvent {
    pub event_type: String,
    pub participants: Vec<EntityId>,
    pub significance: f32,
}

pub struct PerceptionRanges {
    pub visual_base: f32,
    pub audio_base: f32,
}

impl Default for PerceptionRanges {
    fn default() -> Self {
        Self {
            visual_base: 50.0,
            audio_base: 20.0,
        }
    }
}

pub fn effective_visual_range(
    base: f32,
    fatigue: f32,
    terrain_modifier: f32,
    light_level: f32,
) -> f32 {
    let fatigue_mod = if fatigue > 0.7 { 0.8 } else { 1.0 };
    base * terrain_modifier * light_level * fatigue_mod
}

pub fn filter_perception_human(
    raw_perception: &[PerceivedObject],
    values: &HumanValues,
) -> Vec<PerceivedObject> {
    raw_perception.iter()
        .filter(|obj| {
            if obj.properties.iter().any(|p| p.name == "threat") {
                return true;
            }

            for prop in &obj.properties {
                match &prop.name[..] {
                    "aesthetic" if values.beauty > 0.5 => return true,
                    "quality" if values.beauty > 0.5 || values.ambition > 0.5 => return true,
                    "social_status" if values.honor > 0.5 || values.ambition > 0.5 => return true,
                    "comfort" if values.comfort > 0.5 => return true,
                    "sacred" if values.piety > 0.5 => return true,
                    _ => {}
                }
            }

            false
        })
        .cloned()
        .collect()
}

pub fn perception_system(
    spatial_grid: &SparseHashGrid,
    positions: &[Vec2],
    entity_ids: &[EntityId],
    perception_range: f32,
) -> Vec<Perception> {
    entity_ids.iter().enumerate().map(|(i, &observer_id)| {
        let observer_pos = positions[i];

        let nearby: Vec<_> = spatial_grid.query_neighbors(observer_pos)
            .filter(|&e| e != observer_id)
            .collect();

        let perceived_entities: Vec<_> = nearby.iter()
            .filter_map(|&entity| {
                let entity_idx = entity_ids.iter().position(|&e| e == entity)?;
                let entity_pos = positions[entity_idx];
                let distance = observer_pos.distance(&entity_pos);

                if distance <= perception_range {
                    Some(PerceivedEntity {
                        entity,
                        distance,
                        relationship: RelationshipType::Unknown,
                        threat_level: 0.0,
                        notable_features: vec![],
                    })
                } else {
                    None
                }
            })
            .collect();

        Perception {
            observer: observer_id,
            perceived_entities,
            perceived_objects: vec![],
            perceived_events: vec![],
        }
    }).collect()
}
```

**Step 2: Create stubs**

Create `src/simulation/thought_gen.rs`:
```rust
//! Thought generation from perception - stub for MVP
```

Create `src/simulation/action_execute.rs`:
```rust
//! Action execution - stub for MVP
```

**Step 3: Verify build and commit**

```bash
cargo build
git add .
git commit -m "feat(simulation): add perception system"
```

---

## Task 11: Action Selection

**Files:**
- Create: `src/simulation/action_select.rs`

**Step 1: Create action_select.rs**

```rust
//! Action selection algorithm - the heart of autonomous behavior

use crate::actions::catalog::ActionId;
use crate::entity::needs::{Needs, NeedType};
use crate::entity::thoughts::ThoughtBuffer;
use crate::entity::body::BodyState;
use crate::entity::species::human::HumanValues;
use crate::entity::tasks::{Task, TaskPriority, TaskSource};
use crate::core::types::Tick;

pub struct SelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a HumanValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
}

pub fn select_action_human(ctx: &SelectionContext) -> Option<Task> {
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response(critical, ctx);
    }

    if ctx.has_current_task {
        return None;
    }

    if let Some(task) = check_value_impulses(ctx) {
        return Some(task);
    }

    if let Some(task) = address_moderate_need(ctx) {
        return Some(task);
    }

    Some(select_idle_action(ctx))
}

fn select_critical_response(need: NeedType, ctx: &SelectionContext) -> Option<Task> {
    let action = match need {
        NeedType::Safety if ctx.threat_nearby => ActionId::Flee,
        NeedType::Safety => ActionId::SeekSafety,
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Food => ActionId::SeekSafety,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Rest => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task {
        action,
        target_position: None,
        target_entity: None,
        priority: TaskPriority::Critical,
        created_tick: ctx.current_tick,
        progress: 0.0,
        source: TaskSource::Reaction,
    })
}

fn check_value_impulses(ctx: &SelectionContext) -> Option<Task> {
    if let Some(thought) = ctx.thoughts.strongest() {
        if thought.intensity > 0.7 {
            if ctx.values.justice > 0.7 && thought.concept_category == "injustice" {
                return Some(Task::new(ActionId::Help, TaskPriority::High, ctx.current_tick));
            }
        }
    }

    None
}

fn address_moderate_need(ctx: &SelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.6 {
        return None;
    }

    let action = match need_type {
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::IdleObserve,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

fn select_idle_action(ctx: &SelectionContext) -> Task {
    let action = if ctx.values.curiosity > ctx.values.social {
        ActionId::IdleObserve
    } else if ctx.entity_nearby {
        ActionId::TalkTo
    } else {
        ActionId::IdleWander
    };

    Task::new(action, TaskPriority::Low, ctx.current_tick)
}
```

**Step 2: Verify build and commit**

```bash
cargo build
git add .
git commit -m "feat(simulation): add action selection system"
```

---

## Task 12: Simulation Tick

**Files:**
- Create: `src/simulation/tick.rs`

**Step 1: Create tick.rs**

```rust
//! Tick system - orchestrates simulation updates

use crate::ecs::world::World;
use crate::spatial::sparse_hash::SparseHashGrid;
use crate::simulation::perception::{perception_system, RelationshipType};
use crate::simulation::action_select::{select_action_human, SelectionContext};
use crate::entity::thoughts::{Thought, Valence, CauseType};
use crate::entity::needs::NeedType;

pub fn run_simulation_tick(world: &mut World) {
    update_needs(world);
    let perceptions = run_perception(world);
    generate_thoughts(world, &perceptions);
    decay_thoughts(world);
    select_actions(world);
    execute_tasks(world);
    world.tick();
}

fn update_needs(world: &mut World) {
    let dt = 1.0;
    for i in world.humans.iter_living() {
        let is_active = world.humans.task_queues[i].current().is_some();
        world.humans.needs[i].decay(dt, is_active);
    }
}

fn run_perception(world: &World) -> Vec<crate::simulation::perception::Perception> {
    let mut grid = SparseHashGrid::new(10.0);

    let positions: Vec<_> = world.humans.positions.iter().cloned().collect();
    let ids: Vec<_> = world.humans.ids.iter().cloned().collect();

    grid.rebuild(ids.iter().cloned().zip(positions.iter().cloned()));

    perception_system(&grid, &positions, &ids, 50.0)
}

fn generate_thoughts(world: &mut World, perceptions: &[crate::simulation::perception::Perception]) {
    for perception in perceptions {
        let Some(idx) = world.humans.index_of(perception.observer) else { continue };
        let values = &world.humans.values[idx];

        for perceived in &perception.perceived_entities {
            if perceived.threat_level > 0.5 {
                let thought = Thought::new(
                    Valence::Negative,
                    perceived.threat_level,
                    if values.safety > 0.5 { "fear" } else { "concern" },
                    "threatening entity nearby",
                    CauseType::Entity,
                    world.current_tick,
                );
                world.humans.thoughts[idx].add(thought);

                world.humans.needs[idx].safety =
                    (world.humans.needs[idx].safety + perceived.threat_level * 0.3).min(1.0);
            }

            if perceived.relationship == RelationshipType::Ally {
                world.humans.needs[idx].satisfy(NeedType::Social, 0.1);
            }
        }
    }
}

fn decay_thoughts(world: &mut World) {
    for i in world.humans.iter_living() {
        world.humans.thoughts[i].decay_all();
    }
}

fn select_actions(world: &mut World) {
    for i in world.humans.iter_living() {
        if world.humans.task_queues[i].current().is_some() {
            continue;
        }

        let ctx = SelectionContext {
            body: &world.humans.body_states[i],
            needs: &world.humans.needs[i],
            thoughts: &world.humans.thoughts[i],
            values: &world.humans.values[i],
            has_current_task: false,
            threat_nearby: world.humans.needs[i].safety > 0.5,
            food_available: true,
            safe_location: world.humans.needs[i].safety < 0.3,
            entity_nearby: true,
            current_tick: world.current_tick,
        };

        if let Some(task) = select_action_human(&ctx) {
            world.humans.task_queues[i].push(task);
        }
    }
}

fn execute_tasks(world: &mut World) {
    for i in world.humans.iter_living() {
        if let Some(task) = world.humans.task_queues[i].current_mut() {
            task.progress += 0.01;

            for (need, amount) in task.action.satisfies_needs() {
                world.humans.needs[i].satisfy(need, amount * 0.01);
            }

            let duration = task.action.base_duration();
            if duration > 0 && task.progress >= 1.0 {
                world.humans.task_queues[i].complete_current();
            }
        }
    }
}
```

**Step 2: Verify build and commit**

```bash
cargo build
git add .
git commit -m "feat(simulation): add tick system orchestration"
```

---

## Task 13: LLM Client

**Files:**
- Create: `src/llm/client.rs`
- Create: `src/llm/species_interpret.rs` (stub)
- Create: `src/llm/prompts.rs` (stub)

**Step 1: Create client.rs**

```rust
//! Async LLM client for command parsing

use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::core::error::{ArcError, Result};

pub struct LlmClient {
    client: Client,
    api_key: String,
    api_url: String,
    model: String,
}

impl LlmClient {
    pub fn new(api_key: String, api_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            api_url,
            model,
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("LLM_API_KEY")
            .map_err(|_| ArcError::LlmError("LLM_API_KEY not set".into()))?;
        let api_url = std::env::var("LLM_API_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com/v1/messages".into());
        let model = std::env::var("LLM_MODEL")
            .unwrap_or_else(|_| "claude-3-haiku-20240307".into());

        Ok(Self::new(api_key, api_url, model))
    }

    pub async fn complete(&self, system: &str, user: &str) -> Result<String> {
        let request = CompletionRequest {
            model: self.model.clone(),
            max_tokens: 1024,
            system: system.into(),
            messages: vec![Message {
                role: "user".into(),
                content: user.into(),
            }],
        };

        let response = self.client
            .post(&self.api_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ArcError::LlmError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ArcError::LlmError(format!("API error: {}", error_text)));
        }

        let completion: CompletionResponse = response.json().await
            .map_err(|e| ArcError::LlmError(e.to_string()))?;

        completion.content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| ArcError::LlmError("Empty response".into()))
    }
}

#[derive(Serialize)]
struct CompletionRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct CompletionResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}
```

**Step 2: Create stubs**

Create `src/llm/species_interpret.rs`:
```rust
//! Species-specific interpretation - stub for MVP
```

Create `src/llm/prompts.rs`:
```rust
//! Prompt templates - stub for MVP
```

**Step 3: Verify build and commit**

```bash
cargo build
git add .
git commit -m "feat(llm): add async LLM client"
```

---

## Task 14: LLM Parser and Context

**Files:**
- Create: `src/llm/parser.rs`
- Create: `src/llm/context.rs`

**Step 1: Create parser.rs**

```rust
//! Parse natural language commands into structured intents

use serde::{Deserialize, Serialize};
use crate::llm::client::LlmClient;
use crate::llm::context::GameContext;
use crate::core::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedIntent {
    pub action: IntentAction,
    pub target: Option<String>,
    pub location: Option<String>,
    pub subjects: Option<Vec<String>>,
    pub priority: IntentPriority,
    pub ambiguous_concepts: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentAction {
    Build,
    Craft,
    Assign,
    Combat,
    Gather,
    Move,
    Query,
    Social,
    Rest,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentPriority {
    Critical,
    High,
    Normal,
    Low,
}

impl Default for IntentPriority {
    fn default() -> Self {
        Self::Normal
    }
}

pub async fn parse_command(
    client: &LlmClient,
    input: &str,
    context: &GameContext,
) -> Result<ParsedIntent> {
    let system_prompt = PARSE_SYSTEM_PROMPT;
    let user_prompt = format!(
        "CONTEXT:\n{}\n\nPLAYER INPUT:\n{}\n\nParse this command into JSON:",
        context.summary(),
        input
    );

    let response = client.complete(system_prompt, &user_prompt).await?;
    let json_str = extract_json(&response)?;

    let intent: ParsedIntent = serde_json::from_str(json_str)
        .map_err(|e| crate::core::error::ArcError::LlmError(
            format!("Failed to parse intent: {} - Response: {}", e, response)
        ))?;

    Ok(intent)
}

fn extract_json(response: &str) -> Result<&str> {
    let start = response.find('{')
        .ok_or_else(|| crate::core::error::ArcError::LlmError("No JSON found".into()))?;
    let end = response.rfind('}')
        .ok_or_else(|| crate::core::error::ArcError::LlmError("No JSON found".into()))?;
    Ok(&response[start..=end])
}

const PARSE_SYSTEM_PROMPT: &str = r#"You are parsing player commands for a medieval simulation game.
Convert natural language orders into structured JSON.

AVAILABLE ACTIONS:
- BUILD: Construct structures
- CRAFT: Create items
- ASSIGN: Move personnel to roles/locations
- COMBAT: Engage enemies or prepare defenses
- GATHER: Collect resources
- MOVE: Travel to location
- QUERY: Ask about game state (not an action)
- SOCIAL: Interact with characters
- REST: Rest and recover

OUTPUT FORMAT (JSON only, no explanation):
{
  "action": "ACTION_TYPE",
  "target": "what to build/craft/attack/etc or null",
  "location": "where (if applicable) or null",
  "subjects": ["who should do this"] or null if unspecified,
  "priority": "CRITICAL|HIGH|NORMAL|LOW",
  "ambiguous_concepts": ["terms that might be interpreted differently by non-humans"],
  "confidence": 0.0-1.0
}

Examples:
"build a wall" -> {"action": "BUILD", "target": "wall", "location": null, "subjects": null, "priority": "NORMAL", "ambiguous_concepts": [], "confidence": 0.9}
"have Marcus guard the east" -> {"action": "ASSIGN", "target": "guard duty", "location": "east", "subjects": ["Marcus"], "priority": "NORMAL", "ambiguous_concepts": [], "confidence": 0.85}
"make it beautiful" -> {"action": "CRAFT", "target": null, "location": null, "subjects": null, "priority": "NORMAL", "ambiguous_concepts": ["beautiful"], "confidence": 0.6}
"#;
```

**Step 2: Create context.rs**

```rust
//! Gather game context for LLM prompts

use crate::ecs::world::World;
use crate::core::types::Species;

pub struct GameContext {
    pub location_name: String,
    pub entity_count: usize,
    pub available_resources: Vec<String>,
    pub recent_events: Vec<String>,
    pub named_entities: Vec<NamedEntity>,
    pub threats: Vec<String>,
}

pub struct NamedEntity {
    pub name: String,
    pub species: Species,
    pub role: String,
    pub status: String,
}

impl GameContext {
    pub fn from_world(world: &World) -> Self {
        let named_entities: Vec<_> = world.humans.iter_living()
            .take(10)
            .map(|i| NamedEntity {
                name: world.humans.names[i].clone(),
                species: Species::Human,
                role: "worker".into(),
                status: if world.humans.body_states[i].can_act() {
                    "healthy"
                } else {
                    "incapacitated"
                }.into(),
            })
            .collect();

        Self {
            location_name: "Main Camp".into(),
            entity_count: world.entity_count(),
            available_resources: vec!["wood".into(), "stone".into()],
            recent_events: vec![],
            named_entities,
            threats: vec![],
        }
    }

    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("Location: {}\n", self.location_name));
        s.push_str(&format!("Population: {}\n", self.entity_count));

        if !self.named_entities.is_empty() {
            s.push_str("Key Personnel:\n");
            for entity in &self.named_entities {
                s.push_str(&format!("- {} ({:?}, {}, {})\n",
                    entity.name, entity.species, entity.role, entity.status));
            }
        }

        if !self.available_resources.is_empty() {
            s.push_str(&format!("Resources: {}\n", self.available_resources.join(", ")));
        }

        if !self.threats.is_empty() {
            s.push_str(&format!("Threats: {}\n", self.threats.join(", ")));
        }

        s
    }
}
```

**Step 3: Verify build and commit**

```bash
cargo build
git add .
git commit -m "feat(llm): add command parser and context"
```

---

## Task 15: Stub Files for Remaining Modules

**Files:**
- Create all remaining stub files

**Step 1: Create genetics stubs**

Create `src/genetics/genome.rs`:
```rust
//! Genome - stub for MVP
```

Create `src/genetics/phenotype.rs`:
```rust
//! Phenotype expression - stub for MVP
```

Create `src/genetics/personality.rs`:
```rust
//! Personality traits - stub for MVP
```

Create `src/genetics/values.rs`:
```rust
//! Value calculation - stub for MVP
```

**Step 2: Create combat stubs**

Create `src/combat/resolution.rs`:
```rust
//! Combat resolution - stub for MVP
```

Create `src/combat/weapons.rs`:
```rust
//! Weapon properties - stub for MVP
```

Create `src/combat/armor.rs`:
```rust
//! Armor properties - stub for MVP
```

Create `src/combat/wounds.rs`:
```rust
//! Wound system - stub for MVP
```

Create `src/combat/morale.rs`:
```rust
//! Morale system - stub for MVP
```

**Step 3: Create campaign stubs**

Create `src/campaign/map.rs`:
```rust
//! Campaign map - stub for MVP
```

Create `src/campaign/location.rs`:
```rust
//! Location - stub for MVP
```

Create `src/campaign/route.rs`:
```rust
//! Routes - stub for MVP
```

Create `src/campaign/weather.rs`:
```rust
//! Weather system - stub for MVP
```

Create `src/campaign/supply.rs`:
```rust
//! Supply system - stub for MVP
```

**Step 4: Create battle stubs**

Create `src/battle/battle_map.rs`:
```rust
//! Battle map - stub for MVP
```

Create `src/battle/planning.rs`:
```rust
//! Battle planning - stub for MVP
```

Create `src/battle/execution.rs`:
```rust
//! Battle execution - stub for MVP
```

Create `src/battle/courier.rs`:
```rust
//! Courier system - stub for MVP
```

Create `src/battle/resolution.rs`:
```rust
//! Battle resolution - stub for MVP
```

**Step 5: Create UI stubs**

Create `src/ui/terminal.rs`:
```rust
//! Terminal UI - stub for MVP
```

Create `src/ui/input.rs`:
```rust
//! Input handling - stub for MVP
```

Create `src/ui/display.rs`:
```rust
//! Display rendering - stub for MVP
```

**Step 6: Verify build and commit**

```bash
cargo build
git add .
git commit -m "chore: add stub files for remaining modules"
```

---

## Task 16: Main Entry Point

**Files:**
- Update: `src/main.rs`

**Step 1: Replace main.rs with full implementation**

```rust
//! Arc Citadel - Entry Point

use arc_citadel::core::error::Result;
use arc_citadel::ecs::world::World;
use arc_citadel::llm::client::LlmClient;
use arc_citadel::llm::parser::parse_command;
use arc_citadel::llm::context::GameContext;
use arc_citadel::simulation::tick::run_simulation_tick;

use std::io::{self, Write};
use tokio::runtime::Runtime;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("arc_citadel=debug")
        .init();

    tracing::info!("Arc Citadel starting...");

    let rt = Runtime::new()?;
    let mut world = World::new();

    spawn_initial_population(&mut world);

    let llm_client = LlmClient::from_env().ok();
    if llm_client.is_none() {
        tracing::warn!("LLM_API_KEY not set - running without natural language commands");
    }

    println!("\n=== ARC CITADEL ===");
    println!("Type commands or 'quit' to exit.\n");

    loop {
        display_status(&world);

        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "quit" || input == "q" {
            break;
        }

        if input == "tick" || input == "t" {
            run_simulation_tick(&mut world);
            println!("Tick {} complete.", world.current_tick);
            continue;
        }

        if input.starts_with("spawn ") {
            let name = input.strip_prefix("spawn ").unwrap();
            let id = world.spawn_human(name.into());
            println!("Spawned {} (ID: {:?})", name, id);
            continue;
        }

        if let Some(ref client) = llm_client {
            let context = GameContext::from_world(&world);

            match rt.block_on(parse_command(client, input, &context)) {
                Ok(intent) => {
                    println!("\nParsed intent: {:?}", intent);
                    println!("Action: {:?}", intent.action);
                    if let Some(target) = &intent.target {
                        println!("Target: {}", target);
                    }
                    if !intent.ambiguous_concepts.is_empty() {
                        println!("Ambiguous concepts: {:?}", intent.ambiguous_concepts);
                    }
                }
                Err(e) => {
                    println!("Could not parse command: {}", e);
                }
            }
        } else {
            println!("Commands: tick/t, spawn <name>, quit/q");
        }
    }

    println!("Goodbye!");
    Ok(())
}

fn spawn_initial_population(world: &mut World) {
    let names = ["Marcus", "Elena", "Thomas", "Sarah", "William"];
    for name in names {
        world.spawn_human(name.into());
    }
    tracing::info!("Spawned {} initial humans", names.len());
}

fn display_status(world: &World) {
    println!("\n--- Status (Tick {}) ---", world.current_tick);
    println!("Population: {}", world.entity_count());

    for i in world.humans.iter_living().take(5) {
        let name = &world.humans.names[i];
        let body = &world.humans.body_states[i];
        let needs = &world.humans.needs[i];
        let (top_need, level) = needs.most_pressing();

        println!("  {} - Fatigue: {:.0}%, Top need: {:?} ({:.0}%)",
            name,
            body.fatigue * 100.0,
            top_need,
            level * 100.0
        );
    }
    println!();
}
```

**Step 2: Test the full binary**

```bash
cargo build
cargo run
```

Expected: Game starts, shows status, accepts commands (tick, spawn, quit)

**Step 3: Commit**

```bash
git add .
git commit -m "feat: complete main entry point with game loop"
```

---

## Task 17: Data Files

**Files:**
- Create: `data/species/human.json`

**Step 1: Create human.json**

```json
{
  "species": "Human",
  "value_vocabulary": [
    {"name": "honor", "description": "Social standing, keeping word"},
    {"name": "beauty", "description": "Aesthetic appreciation"},
    {"name": "comfort", "description": "Physical ease"},
    {"name": "ambition", "description": "Desire for advancement"},
    {"name": "loyalty", "description": "Attachment to group"},
    {"name": "love", "description": "Attachment to individuals"},
    {"name": "justice", "description": "Fairness"},
    {"name": "curiosity", "description": "Desire to know"},
    {"name": "safety", "description": "Self-preservation"},
    {"name": "piety", "description": "Religious devotion"}
  ],
  "idle_behaviors": ["socialize", "observe", "wander"],
  "perception_filters": {
    "high_beauty": ["aesthetic", "quality", "appearance"],
    "high_honor": ["social_status", "reputation"],
    "high_piety": ["sacred", "ritual"]
  }
}
```

**Step 2: Commit**

```bash
git add .
git commit -m "feat(data): add human species definition"
```

---

## Task 18: Final Integration Test

**Files:**
- Create: `tests/emergence_tests.rs`

**Step 1: Create basic emergence test**

```rust
//! Test that behavior emerges from values

use arc_citadel::ecs::world::World;
use arc_citadel::simulation::tick::run_simulation_tick;

#[test]
fn test_world_creation() {
    let world = World::new();
    assert_eq!(world.entity_count(), 0);
    assert_eq!(world.current_tick, 0);
}

#[test]
fn test_spawn_human() {
    let mut world = World::new();
    let id = world.spawn_human("Test".into());
    assert_eq!(world.entity_count(), 1);
    assert!(world.humans.index_of(id).is_some());
}

#[test]
fn test_needs_decay() {
    let mut world = World::new();
    world.spawn_human("Test".into());

    let initial_rest = world.humans.needs[0].rest;

    for _ in 0..10 {
        run_simulation_tick(&mut world);
    }

    assert!(world.humans.needs[0].rest > initial_rest);
    assert_eq!(world.current_tick, 10);
}

#[test]
fn test_different_values_different_behavior() {
    let mut world = World::new();

    let cautious_id = world.spawn_human("Cautious Carl".into());
    let brave_id = world.spawn_human("Brave Bob".into());

    if let Some(idx) = world.humans.index_of(cautious_id) {
        world.humans.values[idx].safety = 0.9;
        world.humans.values[idx].curiosity = 0.1;
    }
    if let Some(idx) = world.humans.index_of(brave_id) {
        world.humans.values[idx].safety = 0.2;
        world.humans.values[idx].curiosity = 0.9;
    }

    // Run simulation
    for _ in 0..10 {
        run_simulation_tick(&mut world);
    }

    // Both should have selected actions based on values
    // (This validates the system runs without crashing with different values)
    assert_eq!(world.entity_count(), 2);
}
```

**Step 2: Run tests**

```bash
cargo test
```

Expected: All tests pass

**Step 3: Final commit**

```bash
git add .
git commit -m "test: add emergence tests"
```

---

## Verification Checklist

After completing all tasks, verify:

1. **`cargo build` succeeds** - No compilation errors
2. **`cargo test` passes** - All tests green
3. **`cargo run` works** - Game loop runs, shows entities
4. **Commands work**:
   - `tick` - advances simulation
   - `spawn Name` - creates entity
   - `quit` - exits cleanly
5. **Needs decay** - Run multiple ticks, see needs increase
6. **Values differ** - Entities with different values select different actions

---

## Summary

**Total Tasks:** 18
**Estimated Time:** 2-3 hours for implementation
**Key Files:** ~45 source files created
**Core Systems:**
- ECS World with SoA HumanArchetype
- Needs system with decay
- Perception system with spatial queries
- Action selection from needs/values
- LLM command parsing (optional)
- Terminal REPL interface

**What's NOT included (post-MVP):**
- Dwarf/Elf species
- Combat resolution
- Campaign map
- Battle system
- Persistence
- Graphics
