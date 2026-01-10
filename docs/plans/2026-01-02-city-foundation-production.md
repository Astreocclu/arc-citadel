# City Foundation & Production Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a city-builder foundation with buildings, construction via worker-ticks, resource management, and production chains.

**Architecture:**
- New `src/city/` module containing BuildingArchetype (SoA), construction logic, and production recipes
- Extend existing Task system with building targets
- Add `building_skills: Vec<f32>` to species archetypes
- Worker contribution formula: `base_rate × (0.5 + skill × 0.5) × (1.0 - fatigue × 0.4)`
- Dual-layer support: Entity-scale buildings with polity-level aggregation queries

**Tech Stack:** Rust, SoA pattern, TOML for recipes, integration with existing tick system

---

## Task 1: Create City Module Structure

**Files:**
- Create: `src/city/mod.rs`
- Create: `src/city/building.rs`
- Modify: `src/lib.rs` (add `pub mod city;`)

**Step 1: Write the failing test**

Create `src/city/building.rs`:

```rust
//! Building archetype with SoA layout

use crate::core::types::{EntityId, Vec2, Tick};
use serde::{Deserialize, Serialize};

/// Type of building
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingType {
    House,
    Farm,
    Workshop,
    Granary,
    Wall,
    Gate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_type_exists() {
        let bt = BuildingType::House;
        assert_eq!(bt, BuildingType::House);
    }
}
```

**Step 2: Create module file**

Create `src/city/mod.rs`:

```rust
//! City layer - buildings, construction, and production

pub mod building;

pub use building::{BuildingType};
```

**Step 3: Add to lib.rs**

Modify `src/lib.rs` - add after other module declarations:

```rust
pub mod city;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib city::building::tests::test_building_type_exists`
Expected: PASS

**Step 5: Commit**

```bash
git add src/city/mod.rs src/city/building.rs src/lib.rs
git commit -m "feat(city): add city module with BuildingType enum"
```

---

## Task 2: Implement BuildingState and Core Properties

**Files:**
- Modify: `src/city/building.rs`

**Step 1: Write the failing test**

Add to `src/city/building.rs`:

```rust
/// Current state of a building
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingState {
    /// Construction site - not yet usable
    UnderConstruction,
    /// Fully operational
    Complete,
    /// Needs repair before use
    Damaged,
}

impl BuildingType {
    /// Base work required to construct this building type
    pub fn work_required(&self) -> f32 {
        match self {
            BuildingType::House => 100.0,
            BuildingType::Farm => 150.0,
            BuildingType::Workshop => 200.0,
            BuildingType::Granary => 120.0,
            BuildingType::Wall => 80.0,
            BuildingType::Gate => 60.0,
        }
    }

    /// Maximum workers that can contribute effectively
    pub fn max_workers(&self) -> u32 {
        match self {
            BuildingType::House => 3,
            BuildingType::Farm => 5,
            BuildingType::Workshop => 4,
            BuildingType::Granary => 4,
            BuildingType::Wall => 6,
            BuildingType::Gate => 4,
        }
    }

    /// Size of the building (width, height in units)
    pub fn size(&self) -> (f32, f32) {
        match self {
            BuildingType::House => (2.0, 2.0),
            BuildingType::Farm => (4.0, 4.0),
            BuildingType::Workshop => (3.0, 3.0),
            BuildingType::Granary => (3.0, 3.0),
            BuildingType::Wall => (1.0, 1.0),
            BuildingType::Gate => (2.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_type_exists() {
        let bt = BuildingType::House;
        assert_eq!(bt, BuildingType::House);
    }

    #[test]
    fn test_building_work_required() {
        assert_eq!(BuildingType::House.work_required(), 100.0);
        assert_eq!(BuildingType::Farm.work_required(), 150.0);
        assert!(BuildingType::Workshop.work_required() > BuildingType::House.work_required());
    }

    #[test]
    fn test_building_max_workers() {
        assert_eq!(BuildingType::House.max_workers(), 3);
        assert!(BuildingType::Farm.max_workers() >= BuildingType::House.max_workers());
    }

    #[test]
    fn test_building_state() {
        let state = BuildingState::UnderConstruction;
        assert_ne!(state, BuildingState::Complete);
    }
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test --lib city::building::tests`
Expected: 4 tests PASS

**Step 3: Commit**

```bash
git add src/city/building.rs
git commit -m "feat(city): add BuildingState and BuildingType properties"
```

---

## Task 3: Implement BuildingArchetype (SoA)

**Files:**
- Modify: `src/city/building.rs`

**Step 1: Write the failing test**

Add to `src/city/building.rs`:

```rust
/// Unique building identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BuildingId(pub u32);

impl BuildingId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Structure of Arrays for building entities
#[derive(Debug, Clone, Default)]
pub struct BuildingArchetype {
    /// Unique identifiers
    pub ids: Vec<BuildingId>,
    /// Type of each building
    pub building_types: Vec<BuildingType>,
    /// Current state
    pub states: Vec<BuildingState>,
    /// Position in world
    pub positions: Vec<Vec2>,
    /// Construction progress (0.0 to work_required)
    pub construction_progress: Vec<f32>,
    /// Currently assigned worker count
    pub assigned_workers: Vec<u32>,
    /// Owning polity (optional)
    pub polity_ids: Vec<Option<u32>>,
    /// Tick when construction started
    pub started_ticks: Vec<Tick>,
    /// Tick when completed (0 if not complete)
    pub completed_ticks: Vec<Tick>,
}

impl BuildingArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn count(&self) -> usize {
        self.ids.len()
    }

    /// Spawn a new building (starts as construction site)
    pub fn spawn(
        &mut self,
        id: BuildingId,
        building_type: BuildingType,
        position: Vec2,
        tick: Tick,
    ) -> usize {
        let index = self.ids.len();
        self.ids.push(id);
        self.building_types.push(building_type);
        self.states.push(BuildingState::UnderConstruction);
        self.positions.push(position);
        self.construction_progress.push(0.0);
        self.assigned_workers.push(0);
        self.polity_ids.push(None);
        self.started_ticks.push(tick);
        self.completed_ticks.push(0);
        index
    }

    pub fn index_of(&self, id: BuildingId) -> Option<usize> {
        self.ids.iter().position(|&b| b == id)
    }

    /// Iterate over buildings under construction
    pub fn iter_under_construction(&self) -> impl Iterator<Item = usize> + '_ {
        self.states.iter()
            .enumerate()
            .filter(|(_, state)| **state == BuildingState::UnderConstruction)
            .map(|(i, _)| i)
    }

    /// Iterate over completed buildings
    pub fn iter_complete(&self) -> impl Iterator<Item = usize> + '_ {
        self.states.iter()
            .enumerate()
            .filter(|(_, state)| **state == BuildingState::Complete)
            .map(|(i, _)| i)
    }
}
```

Add tests:

```rust
    #[test]
    fn test_building_archetype_spawn() {
        let mut arch = BuildingArchetype::new();
        assert_eq!(arch.count(), 0);

        let id = BuildingId::new(1);
        let idx = arch.spawn(id, BuildingType::House, Vec2::new(10.0, 20.0), 100);

        assert_eq!(arch.count(), 1);
        assert_eq!(idx, 0);
        assert_eq!(arch.building_types[0], BuildingType::House);
        assert_eq!(arch.states[0], BuildingState::UnderConstruction);
        assert_eq!(arch.construction_progress[0], 0.0);
    }

    #[test]
    fn test_building_archetype_index_of() {
        let mut arch = BuildingArchetype::new();
        let id1 = BuildingId::new(1);
        let id2 = BuildingId::new(2);

        arch.spawn(id1, BuildingType::House, Vec2::new(0.0, 0.0), 0);
        arch.spawn(id2, BuildingType::Farm, Vec2::new(10.0, 0.0), 0);

        assert_eq!(arch.index_of(id1), Some(0));
        assert_eq!(arch.index_of(id2), Some(1));
        assert_eq!(arch.index_of(BuildingId::new(99)), None);
    }

    #[test]
    fn test_building_archetype_iter_under_construction() {
        let mut arch = BuildingArchetype::new();
        arch.spawn(BuildingId::new(1), BuildingType::House, Vec2::new(0.0, 0.0), 0);
        arch.spawn(BuildingId::new(2), BuildingType::Farm, Vec2::new(10.0, 0.0), 0);

        // Both are under construction
        let under_construction: Vec<_> = arch.iter_under_construction().collect();
        assert_eq!(under_construction.len(), 2);

        // Complete one
        arch.states[0] = BuildingState::Complete;

        let under_construction: Vec<_> = arch.iter_under_construction().collect();
        assert_eq!(under_construction.len(), 1);
        assert_eq!(under_construction[0], 1);
    }
```

**Step 2: Run tests to verify they pass**

Run: `cargo test --lib city::building::tests`
Expected: 7 tests PASS

**Step 3: Commit**

```bash
git add src/city/building.rs
git commit -m "feat(city): add BuildingArchetype with SoA layout"
```

---

## Task 4: Add BuildingArchetype to World

**Files:**
- Modify: `src/ecs/world.rs`
- Modify: `src/city/mod.rs`

**Step 1: Update city module exports**

In `src/city/mod.rs`, update:

```rust
//! City layer - buildings, construction, and production

pub mod building;

pub use building::{BuildingType, BuildingState, BuildingId, BuildingArchetype};
```

**Step 2: Add to World**

In `src/ecs/world.rs`, add import:

```rust
use crate::city::building::{BuildingArchetype, BuildingId, BuildingType};
```

Add field to `World` struct:

```rust
pub struct World {
    pub current_tick: u64,
    entity_registry: AHashMap<EntityId, (Species, usize)>,
    pub humans: HumanArchetype,
    pub orcs: OrcArchetype,
    next_indices: AHashMap<Species, usize>,
    pub food_zones: Vec<FoodZone>,
    next_food_zone_id: u32,
    pub resource_zones: Vec<ResourceZone>,
    pub astronomy: AstronomicalState,
    pub buildings: BuildingArchetype,  // NEW
    next_building_id: u32,              // NEW
}
```

Update `World::new()`:

```rust
    pub fn new() -> Self {
        let mut next_indices = AHashMap::new();
        next_indices.insert(Species::Human, 0);
        next_indices.insert(Species::Dwarf, 0);
        next_indices.insert(Species::Elf, 0);
        next_indices.insert(Species::Orc, 0);

        Self {
            current_tick: 0,
            entity_registry: AHashMap::new(),
            humans: HumanArchetype::new(),
            orcs: OrcArchetype::new(),
            next_indices,
            food_zones: Vec::new(),
            next_food_zone_id: 0,
            resource_zones: Vec::new(),
            astronomy: AstronomicalState::default(),
            buildings: BuildingArchetype::new(),  // NEW
            next_building_id: 0,                   // NEW
        }
    }
```

Add spawn method:

```rust
    pub fn spawn_building(&mut self, building_type: BuildingType, position: Vec2) -> BuildingId {
        let id = BuildingId::new(self.next_building_id);
        self.next_building_id += 1;
        self.buildings.spawn(id, building_type, position, self.current_tick);
        id
    }
```

**Step 3: Write test**

Add to `src/ecs/world.rs` tests:

```rust
    #[test]
    fn test_world_has_buildings() {
        let mut world = World::new();
        assert_eq!(world.buildings.count(), 0);

        use crate::city::building::BuildingType;
        let id = world.spawn_building(BuildingType::House, Vec2::new(50.0, 50.0));

        assert_eq!(world.buildings.count(), 1);
        assert_eq!(world.buildings.index_of(id), Some(0));
    }
```

**Step 4: Run tests**

Run: `cargo test --lib ecs::world::tests::test_world_has_buildings`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ecs/world.rs src/city/mod.rs
git commit -m "feat(city): integrate BuildingArchetype into World"
```

---

## Task 5: Add Building Skills to Human Archetype

**Files:**
- Modify: `src/entity/species/human.rs`

**Step 1: Add building_skills field**

In `src/entity/species/human.rs`, add to `HumanArchetype`:

```rust
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
    pub social_memories: Vec<SocialMemory>,
    pub event_buffers: Vec<EventBuffer>,
    pub building_skills: Vec<f32>,  // NEW: 0.0 to 1.0
}
```

Update `HumanArchetype::new()`:

```rust
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
            social_memories: Vec::new(),
            event_buffers: Vec::new(),
            building_skills: Vec::new(),  // NEW
        }
    }
```

Update `HumanArchetype::spawn()`:

```rust
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
        self.social_memories.push(SocialMemory::new());
        self.event_buffers.push(EventBuffer::default());
        // Random starting skill 0.0-0.3
        self.building_skills.push(rand::random::<f32>() * 0.3);
    }
```

Add import at top:

```rust
use rand;
```

**Step 2: Write test**

Add to tests in `src/entity/species/human.rs`:

```rust
    #[test]
    fn test_human_has_building_skill() {
        let mut archetype = HumanArchetype::new();
        let id = EntityId::new();
        archetype.spawn(id, "Builder Bob".into(), 0);

        assert_eq!(archetype.building_skills.len(), 1);
        assert!(archetype.building_skills[0] >= 0.0);
        assert!(archetype.building_skills[0] <= 0.3);
    }
```

**Step 3: Run tests**

Run: `cargo test --lib entity::species::human::tests`
Expected: 2 tests PASS

**Step 4: Commit**

```bash
git add src/entity/species/human.rs
git commit -m "feat(entity): add building_skills to HumanArchetype"
```

---

## Task 6: Add Building Target to Task

**Files:**
- Modify: `src/entity/tasks.rs`

**Step 1: Add building target field**

In `src/entity/tasks.rs`, add import:

```rust
use crate::city::building::BuildingId;
```

Add field to `Task` struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub action: ActionId,
    pub target_position: Option<Vec2>,
    pub target_entity: Option<EntityId>,
    pub target_building: Option<BuildingId>,  // NEW
    pub priority: TaskPriority,
    pub created_tick: Tick,
    pub progress: f32,
    pub source: TaskSource,
}
```

Update `Task::new()`:

```rust
    pub fn new(action: ActionId, priority: TaskPriority, tick: Tick) -> Self {
        Self {
            action,
            target_position: None,
            target_entity: None,
            target_building: None,  // NEW
            priority,
            created_tick: tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }
    }
```

Add builder method:

```rust
    pub fn with_building(mut self, building: BuildingId) -> Self {
        self.target_building = Some(building);
        self
    }
```

**Step 2: Write test**

Add to tests:

```rust
    #[test]
    fn test_task_with_building_target() {
        use crate::city::building::BuildingId;
        use crate::actions::catalog::ActionId;

        let task = Task::new(ActionId::Build, TaskPriority::Normal, 0)
            .with_building(BuildingId::new(42));

        assert_eq!(task.action, ActionId::Build);
        assert_eq!(task.target_building, Some(BuildingId::new(42)));
    }
```

**Step 3: Run tests**

Run: `cargo test --lib entity::tasks`
Expected: PASS

**Step 4: Commit**

```bash
git add src/entity/tasks.rs
git commit -m "feat(task): add target_building field for construction tasks"
```

---

## Task 7: Implement Construction System

**Files:**
- Create: `src/city/construction.rs`
- Modify: `src/city/mod.rs`

**Step 1: Create construction module**

Create `src/city/construction.rs`:

```rust
//! Construction system - handles building progress from worker contributions

use crate::city::building::{BuildingArchetype, BuildingState, BuildingType};

/// Result of a worker contribution
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContributionResult {
    /// Work contributed, building still under construction
    InProgress { contributed: f32 },
    /// Work contributed, building is now complete
    Completed { contributed: f32 },
    /// Building is already complete
    AlreadyComplete,
    /// Building not found
    NotFound,
}

/// Calculate worker contribution with diminishing returns
///
/// Formula: base_rate × sqrt(workers)
/// - 1 worker = 1.0
/// - 2 workers = 1.41
/// - 3 workers = 1.73
/// - 4 workers = 2.0
/// - 5 workers = 2.24
pub fn calculate_team_contribution(worker_count: u32, max_workers: u32) -> f32 {
    let effective = worker_count.min(max_workers) as f32;
    effective.sqrt()
}

/// Calculate individual worker contribution per tick
///
/// Formula: base_rate × (0.5 + skill × 0.5) × (1.0 - fatigue × 0.4)
pub fn calculate_worker_contribution(
    building_skill: f32,
    fatigue: f32,
) -> f32 {
    const BASE_RATE: f32 = 1.0;
    let skill_multiplier = 0.5 + building_skill * 0.5;  // 0.5 to 1.0
    let fatigue_penalty = 1.0 - fatigue.clamp(0.0, 1.0) * 0.4;  // 0.6 to 1.0
    BASE_RATE * skill_multiplier * fatigue_penalty
}

/// Apply construction work to a building
pub fn apply_construction_work(
    buildings: &mut BuildingArchetype,
    building_idx: usize,
    work_amount: f32,
    current_tick: u64,
) -> ContributionResult {
    if building_idx >= buildings.count() {
        return ContributionResult::NotFound;
    }

    if buildings.states[building_idx] == BuildingState::Complete {
        return ContributionResult::AlreadyComplete;
    }

    let work_required = buildings.building_types[building_idx].work_required();
    buildings.construction_progress[building_idx] += work_amount;

    if buildings.construction_progress[building_idx] >= work_required {
        buildings.states[building_idx] = BuildingState::Complete;
        buildings.completed_ticks[building_idx] = current_tick;
        ContributionResult::Completed { contributed: work_amount }
    } else {
        ContributionResult::InProgress { contributed: work_amount }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::city::building::BuildingId;
    use crate::core::types::Vec2;

    #[test]
    fn test_team_contribution_diminishing_returns() {
        // 1 worker = 1.0
        assert!((calculate_team_contribution(1, 5) - 1.0).abs() < 0.01);
        // 2 workers = ~1.41
        assert!((calculate_team_contribution(2, 5) - 1.414).abs() < 0.01);
        // 4 workers = 2.0
        assert!((calculate_team_contribution(4, 5) - 2.0).abs() < 0.01);
        // Over max is capped
        assert!((calculate_team_contribution(10, 5) - calculate_team_contribution(5, 5)).abs() < 0.01);
    }

    #[test]
    fn test_worker_contribution_formula() {
        // No skill, no fatigue
        let contrib = calculate_worker_contribution(0.0, 0.0);
        assert!((contrib - 0.5).abs() < 0.01);

        // Max skill, no fatigue
        let contrib = calculate_worker_contribution(1.0, 0.0);
        assert!((contrib - 1.0).abs() < 0.01);

        // Max skill, max fatigue
        let contrib = calculate_worker_contribution(1.0, 1.0);
        assert!((contrib - 0.6).abs() < 0.01);

        // Mid skill, mid fatigue
        let contrib = calculate_worker_contribution(0.5, 0.5);
        assert!((contrib - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_apply_construction_work() {
        let mut buildings = BuildingArchetype::new();
        let id = BuildingId::new(1);
        buildings.spawn(id, BuildingType::Wall, Vec2::new(0.0, 0.0), 0);

        // Wall requires 80.0 work
        let result = apply_construction_work(&mut buildings, 0, 40.0, 100);
        assert!(matches!(result, ContributionResult::InProgress { .. }));
        assert!((buildings.construction_progress[0] - 40.0).abs() < 0.01);

        // Complete it
        let result = apply_construction_work(&mut buildings, 0, 50.0, 200);
        assert!(matches!(result, ContributionResult::Completed { .. }));
        assert_eq!(buildings.states[0], BuildingState::Complete);
        assert_eq!(buildings.completed_ticks[0], 200);
    }

    #[test]
    fn test_apply_construction_already_complete() {
        let mut buildings = BuildingArchetype::new();
        let id = BuildingId::new(1);
        buildings.spawn(id, BuildingType::Wall, Vec2::new(0.0, 0.0), 0);
        buildings.states[0] = BuildingState::Complete;

        let result = apply_construction_work(&mut buildings, 0, 10.0, 100);
        assert_eq!(result, ContributionResult::AlreadyComplete);
    }
}
```

**Step 2: Update module exports**

In `src/city/mod.rs`:

```rust
//! City layer - buildings, construction, and production

pub mod building;
pub mod construction;

pub use building::{BuildingType, BuildingState, BuildingId, BuildingArchetype};
pub use construction::{
    calculate_team_contribution,
    calculate_worker_contribution,
    apply_construction_work,
    ContributionResult,
};
```

**Step 3: Run tests**

Run: `cargo test --lib city::construction::tests`
Expected: 4 tests PASS

**Step 4: Commit**

```bash
git add src/city/construction.rs src/city/mod.rs
git commit -m "feat(city): add construction system with worker contribution formulas"
```

---

## Task 8: Extend ResourceType for City Resources

**Files:**
- Modify: `src/simulation/resource_zone.rs`

**Step 1: Add new resource types**

In `src/simulation/resource_zone.rs`, update `ResourceType`:

```rust
/// Type of resource available in a zone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Wood,
    Stone,
    Ore,
    Iron,   // NEW: Processed from Ore
    Cloth,  // NEW: From workshops
    Food,   // NEW: Explicit food resource
}

impl ResourceType {
    /// Whether this resource requires processing (can't be gathered directly)
    pub fn requires_processing(&self) -> bool {
        matches!(self, ResourceType::Iron | ResourceType::Cloth)
    }
}
```

**Step 2: Write test**

Add test:

```rust
    #[test]
    fn test_resource_type_processing() {
        assert!(!ResourceType::Wood.requires_processing());
        assert!(!ResourceType::Stone.requires_processing());
        assert!(!ResourceType::Ore.requires_processing());
        assert!(ResourceType::Iron.requires_processing());
        assert!(ResourceType::Cloth.requires_processing());
    }
```

**Step 3: Run tests**

Run: `cargo test --lib simulation::resource_zone::tests`
Expected: 5 tests PASS

**Step 4: Commit**

```bash
git add src/simulation/resource_zone.rs
git commit -m "feat(resource): add Iron, Cloth, Food resource types"
```

---

## Task 9: Add Material Requirements to Buildings

**Files:**
- Modify: `src/city/building.rs`

**Step 1: Add material requirements**

In `src/city/building.rs`, add import:

```rust
use crate::simulation::resource_zone::ResourceType;
```

Add method to `BuildingType`:

```rust
impl BuildingType {
    // ... existing methods ...

    /// Materials required to construct this building
    pub fn required_materials(&self) -> Vec<(ResourceType, u32)> {
        match self {
            BuildingType::House => vec![
                (ResourceType::Wood, 20),
                (ResourceType::Stone, 10),
            ],
            BuildingType::Farm => vec![
                (ResourceType::Wood, 30),
            ],
            BuildingType::Workshop => vec![
                (ResourceType::Wood, 40),
                (ResourceType::Stone, 20),
                (ResourceType::Iron, 5),
            ],
            BuildingType::Granary => vec![
                (ResourceType::Wood, 50),
                (ResourceType::Stone, 30),
            ],
            BuildingType::Wall => vec![
                (ResourceType::Stone, 25),
            ],
            BuildingType::Gate => vec![
                (ResourceType::Wood, 15),
                (ResourceType::Iron, 10),
            ],
        }
    }
}
```

**Step 2: Write test**

Add test:

```rust
    #[test]
    fn test_building_required_materials() {
        use crate::simulation::resource_zone::ResourceType;

        let materials = BuildingType::House.required_materials();
        assert_eq!(materials.len(), 2);
        assert!(materials.contains(&(ResourceType::Wood, 20)));
        assert!(materials.contains(&(ResourceType::Stone, 10)));

        // Workshop needs iron
        let workshop_mats = BuildingType::Workshop.required_materials();
        assert!(workshop_mats.iter().any(|(r, _)| *r == ResourceType::Iron));
    }
```

**Step 3: Run tests**

Run: `cargo test --lib city::building::tests`
Expected: PASS

**Step 4: Commit**

```bash
git add src/city/building.rs
git commit -m "feat(city): add material requirements to BuildingType"
```

---

## Task 10: Create Stockpile System

**Files:**
- Create: `src/city/stockpile.rs`
- Modify: `src/city/mod.rs`

**Step 1: Create stockpile module**

Create `src/city/stockpile.rs`:

```rust
//! Stockpile - settlement-level resource storage

use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use crate::simulation::resource_zone::ResourceType;

/// A stockpile holding resources for a settlement
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stockpile {
    /// Resources stored: type -> (current, capacity)
    resources: AHashMap<ResourceType, (u32, u32)>,
}

impl Stockpile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set capacity for a resource type
    pub fn set_capacity(&mut self, resource: ResourceType, capacity: u32) {
        let entry = self.resources.entry(resource).or_insert((0, 0));
        entry.1 = capacity;
    }

    /// Get current amount of a resource
    pub fn get(&self, resource: ResourceType) -> u32 {
        self.resources.get(&resource).map(|(c, _)| *c).unwrap_or(0)
    }

    /// Get capacity for a resource
    pub fn capacity(&self, resource: ResourceType) -> u32 {
        self.resources.get(&resource).map(|(_, cap)| *cap).unwrap_or(0)
    }

    /// Try to add resources, returns amount actually added
    pub fn add(&mut self, resource: ResourceType, amount: u32) -> u32 {
        let entry = self.resources.entry(resource).or_insert((0, 100)); // Default capacity 100
        let space = entry.1.saturating_sub(entry.0);
        let added = amount.min(space);
        entry.0 += added;
        added
    }

    /// Try to remove resources, returns amount actually removed
    pub fn remove(&mut self, resource: ResourceType, amount: u32) -> u32 {
        if let Some(entry) = self.resources.get_mut(&resource) {
            let removed = amount.min(entry.0);
            entry.0 -= removed;
            removed
        } else {
            0
        }
    }

    /// Check if stockpile has enough of all required materials
    pub fn has_materials(&self, requirements: &[(ResourceType, u32)]) -> bool {
        requirements.iter().all(|(res, amount)| self.get(*res) >= *amount)
    }

    /// Consume materials for construction, returns true if successful
    pub fn consume_materials(&mut self, requirements: &[(ResourceType, u32)]) -> bool {
        if !self.has_materials(requirements) {
            return false;
        }
        for (res, amount) in requirements {
            self.remove(*res, *amount);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stockpile_add_remove() {
        let mut stockpile = Stockpile::new();
        stockpile.set_capacity(ResourceType::Wood, 50);

        assert_eq!(stockpile.add(ResourceType::Wood, 30), 30);
        assert_eq!(stockpile.get(ResourceType::Wood), 30);

        // Can't exceed capacity
        assert_eq!(stockpile.add(ResourceType::Wood, 30), 20);
        assert_eq!(stockpile.get(ResourceType::Wood), 50);

        // Remove
        assert_eq!(stockpile.remove(ResourceType::Wood, 20), 20);
        assert_eq!(stockpile.get(ResourceType::Wood), 30);
    }

    #[test]
    fn test_stockpile_has_materials() {
        let mut stockpile = Stockpile::new();
        stockpile.add(ResourceType::Wood, 50);
        stockpile.add(ResourceType::Stone, 30);

        let requirements = vec![
            (ResourceType::Wood, 20),
            (ResourceType::Stone, 10),
        ];
        assert!(stockpile.has_materials(&requirements));

        let too_much = vec![
            (ResourceType::Wood, 100),
        ];
        assert!(!stockpile.has_materials(&too_much));
    }

    #[test]
    fn test_stockpile_consume_materials() {
        let mut stockpile = Stockpile::new();
        stockpile.add(ResourceType::Wood, 50);
        stockpile.add(ResourceType::Stone, 30);

        let requirements = vec![
            (ResourceType::Wood, 20),
            (ResourceType::Stone, 10),
        ];

        assert!(stockpile.consume_materials(&requirements));
        assert_eq!(stockpile.get(ResourceType::Wood), 30);
        assert_eq!(stockpile.get(ResourceType::Stone), 20);
    }
}
```

**Step 2: Update module exports**

In `src/city/mod.rs`:

```rust
//! City layer - buildings, construction, and production

pub mod building;
pub mod construction;
pub mod stockpile;

pub use building::{BuildingType, BuildingState, BuildingId, BuildingArchetype};
pub use construction::{
    calculate_team_contribution,
    calculate_worker_contribution,
    apply_construction_work,
    ContributionResult,
};
pub use stockpile::Stockpile;
```

**Step 3: Run tests**

Run: `cargo test --lib city::stockpile::tests`
Expected: 3 tests PASS

**Step 4: Commit**

```bash
git add src/city/stockpile.rs src/city/mod.rs
git commit -m "feat(city): add Stockpile for settlement resource storage"
```

---

## Task 11: Create Production Recipe System

**Files:**
- Create: `src/city/recipe.rs`
- Modify: `src/city/mod.rs`

**Step 1: Create recipe module**

Create `src/city/recipe.rs`:

```rust
//! Production recipes - define what buildings produce

use serde::{Deserialize, Serialize};
use crate::simulation::resource_zone::ResourceType;
use crate::city::building::BuildingType;

/// A production recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Building type that can execute this recipe
    pub building_type: BuildingType,
    /// Input resources consumed
    pub inputs: Vec<(ResourceType, u32)>,
    /// Output resources produced
    pub outputs: Vec<(ResourceType, u32)>,
    /// Ticks to complete one production cycle
    pub duration_ticks: u32,
    /// Workers needed for full speed
    pub workers_needed: u32,
}

impl Recipe {
    /// Calculate production rate based on worker count
    /// Returns multiplier (0.0 to 1.0+)
    pub fn production_rate(&self, workers: u32) -> f32 {
        if self.workers_needed == 0 {
            return 1.0;
        }
        (workers as f32 / self.workers_needed as f32).min(1.2)
    }
}

/// Catalog of all available recipes
#[derive(Debug, Clone, Default)]
pub struct RecipeCatalog {
    recipes: Vec<Recipe>,
}

impl RecipeCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load default recipes
    pub fn with_defaults() -> Self {
        let mut catalog = Self::new();

        // Farm produces food
        catalog.add(Recipe {
            id: "farm_food".into(),
            name: "Grow Food".into(),
            building_type: BuildingType::Farm,
            inputs: vec![],
            outputs: vec![(ResourceType::Food, 5)],
            duration_ticks: 100,
            workers_needed: 2,
        });

        // Workshop: ore -> iron
        catalog.add(Recipe {
            id: "smelt_iron".into(),
            name: "Smelt Iron".into(),
            building_type: BuildingType::Workshop,
            inputs: vec![(ResourceType::Ore, 3)],
            outputs: vec![(ResourceType::Iron, 1)],
            duration_ticks: 50,
            workers_needed: 1,
        });

        // Workshop: wool -> cloth (using Food as proxy for wool)
        catalog.add(Recipe {
            id: "weave_cloth".into(),
            name: "Weave Cloth".into(),
            building_type: BuildingType::Workshop,
            inputs: vec![(ResourceType::Wood, 2)],  // Using wood as fiber proxy
            outputs: vec![(ResourceType::Cloth, 1)],
            duration_ticks: 40,
            workers_needed: 1,
        });

        catalog
    }

    pub fn add(&mut self, recipe: Recipe) {
        self.recipes.push(recipe);
    }

    pub fn get(&self, id: &str) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.id == id)
    }

    pub fn for_building(&self, building_type: BuildingType) -> impl Iterator<Item = &Recipe> {
        self.recipes.iter().filter(move |r| r.building_type == building_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_production_rate() {
        let recipe = Recipe {
            id: "test".into(),
            name: "Test".into(),
            building_type: BuildingType::Farm,
            inputs: vec![],
            outputs: vec![(ResourceType::Food, 1)],
            duration_ticks: 100,
            workers_needed: 2,
        };

        // 0 workers = 0 rate
        assert!((recipe.production_rate(0) - 0.0).abs() < 0.01);
        // 1 worker = 50% rate
        assert!((recipe.production_rate(1) - 0.5).abs() < 0.01);
        // 2 workers = 100% rate
        assert!((recipe.production_rate(2) - 1.0).abs() < 0.01);
        // 3 workers = 120% rate (capped)
        assert!((recipe.production_rate(3) - 1.2).abs() < 0.01);
    }

    #[test]
    fn test_recipe_catalog_defaults() {
        let catalog = RecipeCatalog::with_defaults();

        let farm_food = catalog.get("farm_food");
        assert!(farm_food.is_some());
        assert_eq!(farm_food.unwrap().building_type, BuildingType::Farm);

        let smelt = catalog.get("smelt_iron");
        assert!(smelt.is_some());
    }

    #[test]
    fn test_recipe_catalog_for_building() {
        let catalog = RecipeCatalog::with_defaults();

        let farm_recipes: Vec<_> = catalog.for_building(BuildingType::Farm).collect();
        assert_eq!(farm_recipes.len(), 1);

        let workshop_recipes: Vec<_> = catalog.for_building(BuildingType::Workshop).collect();
        assert_eq!(workshop_recipes.len(), 2);
    }
}
```

**Step 2: Update module exports**

In `src/city/mod.rs`:

```rust
//! City layer - buildings, construction, and production

pub mod building;
pub mod construction;
pub mod stockpile;
pub mod recipe;

pub use building::{BuildingType, BuildingState, BuildingId, BuildingArchetype};
pub use construction::{
    calculate_team_contribution,
    calculate_worker_contribution,
    apply_construction_work,
    ContributionResult,
};
pub use stockpile::Stockpile;
pub use recipe::{Recipe, RecipeCatalog};
```

**Step 3: Run tests**

Run: `cargo test --lib city::recipe::tests`
Expected: 3 tests PASS

**Step 4: Commit**

```bash
git add src/city/recipe.rs src/city/mod.rs
git commit -m "feat(city): add production recipe system"
```

---

## Task 12: Add Production State to Buildings

**Files:**
- Modify: `src/city/building.rs`

**Step 1: Add production tracking**

In `src/city/building.rs`, add new fields to `BuildingArchetype`:

```rust
/// Structure of Arrays for building entities
#[derive(Debug, Clone, Default)]
pub struct BuildingArchetype {
    // ... existing fields ...

    /// Active recipe ID (None if not producing)
    pub active_recipes: Vec<Option<String>>,
    /// Production progress (0.0 to 1.0)
    pub production_progress: Vec<f32>,
    /// Workers assigned to production
    pub production_workers: Vec<u32>,
}
```

Update `BuildingArchetype::new()` to initialize new `Vec`s, and update `spawn()`:

```rust
    pub fn spawn(
        &mut self,
        id: BuildingId,
        building_type: BuildingType,
        position: Vec2,
        tick: Tick,
    ) -> usize {
        let index = self.ids.len();
        self.ids.push(id);
        self.building_types.push(building_type);
        self.states.push(BuildingState::UnderConstruction);
        self.positions.push(position);
        self.construction_progress.push(0.0);
        self.assigned_workers.push(0);
        self.polity_ids.push(None);
        self.started_ticks.push(tick);
        self.completed_ticks.push(0);
        // Production fields
        self.active_recipes.push(None);
        self.production_progress.push(0.0);
        self.production_workers.push(0);
        index
    }
```

**Step 2: Add production methods**

```rust
impl BuildingArchetype {
    // ... existing methods ...

    /// Start production of a recipe
    pub fn start_production(&mut self, index: usize, recipe_id: String) -> bool {
        if index >= self.count() {
            return false;
        }
        if self.states[index] != BuildingState::Complete {
            return false;
        }
        self.active_recipes[index] = Some(recipe_id);
        self.production_progress[index] = 0.0;
        true
    }

    /// Advance production by given amount, returns true if cycle completed
    pub fn advance_production(&mut self, index: usize, amount: f32) -> bool {
        if index >= self.count() || self.active_recipes[index].is_none() {
            return false;
        }
        self.production_progress[index] += amount;
        if self.production_progress[index] >= 1.0 {
            self.production_progress[index] = 0.0; // Reset for next cycle
            true
        } else {
            false
        }
    }

    /// Iterate over buildings that are producing
    pub fn iter_producing(&self) -> impl Iterator<Item = usize> + '_ {
        self.active_recipes.iter()
            .enumerate()
            .filter(|(i, recipe)| {
                recipe.is_some() && self.states[*i] == BuildingState::Complete
            })
            .map(|(i, _)| i)
    }
}
```

**Step 3: Write test**

Add test:

```rust
    #[test]
    fn test_building_production() {
        let mut arch = BuildingArchetype::new();
        let id = BuildingId::new(1);
        arch.spawn(id, BuildingType::Farm, Vec2::new(0.0, 0.0), 0);

        // Can't produce while under construction
        assert!(!arch.start_production(0, "farm_food".into()));

        // Complete it
        arch.states[0] = BuildingState::Complete;
        assert!(arch.start_production(0, "farm_food".into()));
        assert_eq!(arch.active_recipes[0], Some("farm_food".into()));

        // Advance production
        assert!(!arch.advance_production(0, 0.5)); // Not complete
        assert!(arch.advance_production(0, 0.6)); // Complete (0.5 + 0.6 > 1.0)
        assert!((arch.production_progress[0] - 0.0).abs() < 0.01); // Reset
    }
```

**Step 4: Run tests**

Run: `cargo test --lib city::building::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add src/city/building.rs
git commit -m "feat(city): add production state to BuildingArchetype"
```

---

## Task 13: Integrate Construction into Tick System

**Files:**
- Modify: `src/simulation/tick.rs`

**Step 1: Add import**

Add at top of `src/simulation/tick.rs`:

```rust
use crate::city::construction::{calculate_worker_contribution, apply_construction_work, ContributionResult};
use crate::city::building::BuildingId;
```

**Step 2: Update Build action execution**

Find the section handling `ActionId::Build` (around line 814) and replace:

```rust
                        ActionId::Build => {
                            // Check if task has a building target
                            if let Some(building_id) = task.target_building {
                                if let Some(building_idx) = world.buildings.index_of(building_id) {
                                    // Calculate contribution based on skill and fatigue
                                    let skill = world.humans.building_skills[i];
                                    let fatigue = world.humans.body_states[i].fatigue;
                                    let contribution = calculate_worker_contribution(skill, fatigue);

                                    let result = apply_construction_work(
                                        &mut world.buildings,
                                        building_idx,
                                        contribution,
                                        world.current_tick,
                                    );

                                    match result {
                                        ContributionResult::Completed { .. } => {
                                            // Skill improvement on completion
                                            world.humans.building_skills[i] =
                                                (world.humans.building_skills[i] + 0.01).min(1.0);
                                            true // Task complete
                                        }
                                        ContributionResult::InProgress { .. } => {
                                            task.progress = world.buildings.construction_progress[building_idx]
                                                / world.buildings.building_types[building_idx].work_required();
                                            false // Still working
                                        }
                                        ContributionResult::AlreadyComplete | ContributionResult::NotFound => {
                                            true // Task complete (nothing to do)
                                        }
                                    }
                                } else {
                                    true // Building not found, complete task
                                }
                            } else {
                                // Legacy behavior: no building target
                                let duration = task.action.base_duration();
                                let progress_rate = match duration {
                                    0 => 0.1,
                                    1..=60 => 0.05,
                                    _ => 0.02,
                                };
                                task.progress += progress_rate;
                                duration > 0 && task.progress >= 1.0
                            }
                        }
```

**Step 3: Write integration test**

Add test in `src/simulation/tick.rs` tests:

```rust
    #[test]
    fn test_build_action_with_building_target() {
        use crate::city::building::{BuildingType, BuildingState};
        use crate::entity::tasks::{Task, TaskPriority};
        use crate::actions::catalog::ActionId;

        let mut world = World::new();
        let entity = world.spawn_human("Builder".into());

        // Spawn a building
        let building_id = world.spawn_building(BuildingType::Wall, Vec2::new(50.0, 50.0));

        // Get entity index
        let idx = world.humans.index_of(entity).unwrap();

        // Set position near building
        world.humans.positions[idx] = Vec2::new(50.0, 50.0);

        // Set high building skill for faster progress
        world.humans.building_skills[idx] = 1.0;

        // Create build task
        let task = Task::new(ActionId::Build, TaskPriority::Normal, 0)
            .with_building(building_id);
        world.humans.task_queues[idx].push(task);

        // Run many ticks to complete construction
        // Wall needs 80 work, max contribution per tick = 1.0 (skill 1.0, no fatigue)
        for _ in 0..100 {
            run_simulation_tick(&mut world);
        }

        // Building should be complete
        let building_idx = world.buildings.index_of(building_id).unwrap();
        assert_eq!(world.buildings.states[building_idx], BuildingState::Complete);
    }
```

**Step 4: Run tests**

Run: `cargo test --lib simulation::tick::tests::test_build_action_with_building_target`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat(tick): integrate construction system with Build action"
```

---

## Task 14: Add Building Perception

**Files:**
- Modify: `src/simulation/perception.rs`

**Step 1: Add building to perception**

Add field to `Perception` struct:

```rust
use crate::city::building::BuildingId;

pub struct Perception {
    pub observer: EntityId,
    pub perceived_entities: Vec<PerceivedEntity>,
    pub nearest_food_zone: Option<(Vec2, f32)>,
    pub nearest_building_site: Option<(BuildingId, Vec2, f32)>,  // NEW
}
```

Update default/new implementations to include the new field.

**Step 2: Add building perception function**

```rust
/// Find nearest building under construction within range
pub fn find_nearest_building_site(
    observer_pos: Vec2,
    range: f32,
    buildings: &crate::city::building::BuildingArchetype,
) -> Option<(BuildingId, Vec2, f32)> {
    use crate::city::building::BuildingState;

    let mut nearest: Option<(BuildingId, Vec2, f32)> = None;

    for i in buildings.iter_under_construction() {
        let pos = buildings.positions[i];
        let distance = observer_pos.distance(&pos);

        if distance <= range {
            if nearest.is_none() || distance < nearest.unwrap().2 {
                nearest = Some((buildings.ids[i], pos, distance));
            }
        }
    }

    nearest
}
```

**Step 3: Integrate into run_perception**

In `src/simulation/tick.rs`, update `run_perception` to populate `nearest_building_site`:

```rust
    // Populate nearest_building_site for each perception
    for (i, perception) in perceptions.iter_mut().enumerate() {
        perception.nearest_building_site = find_nearest_building_site(
            positions[i],
            perception_ranges[i],
            &world.buildings,
        );
    }
```

**Step 4: Write test**

```rust
    #[test]
    fn test_find_nearest_building_site() {
        use crate::city::building::{BuildingArchetype, BuildingType, BuildingId};

        let mut buildings = BuildingArchetype::new();
        buildings.spawn(BuildingId::new(1), BuildingType::House, Vec2::new(10.0, 0.0), 0);
        buildings.spawn(BuildingId::new(2), BuildingType::Farm, Vec2::new(5.0, 0.0), 0);

        let observer = Vec2::new(0.0, 0.0);
        let result = find_nearest_building_site(observer, 50.0, &buildings);

        assert!(result.is_some());
        let (id, _, dist) = result.unwrap();
        assert_eq!(id, BuildingId::new(2)); // Farm is closer
        assert!((dist - 5.0).abs() < 0.01);
    }
```

**Step 5: Run tests**

Run: `cargo test --lib simulation::perception`
Expected: PASS

**Step 6: Commit**

```bash
git add src/simulation/perception.rs src/simulation/tick.rs
git commit -m "feat(perception): add building site awareness"
```

---

## Task 15: Purpose-Driven Work Seeking in Action Selection

**Files:**
- Modify: `src/simulation/action_select.rs`

**Step 1: Add building work preference**

In `src/simulation/action_select.rs`, find the `select_action_human` function and add logic to prefer building work when purpose need is high:

```rust
/// Check if entity should seek building work based on purpose need
fn should_seek_building_work(
    needs: &Needs,
    building_skill: f32,
    nearest_building_site: Option<(BuildingId, Vec2, f32)>,
) -> Option<(ActionId, BuildingId, Vec2)> {
    // Only seek work if purpose need is high (> 0.6)
    if needs.purpose < 0.6 {
        return None;
    }

    // Need a building site within range
    let (building_id, pos, _distance) = nearest_building_site?;

    // Higher skill = more likely to seek building work
    let skill_weight = 0.5 + building_skill * 0.5;
    let purpose_weight = (needs.purpose - 0.6) * 2.5; // 0.0 to 1.0

    if skill_weight * purpose_weight > 0.3 {
        Some((ActionId::Build, building_id, pos))
    } else {
        None
    }
}
```

**Step 2: Integrate into selection**

In `select_action_human`, add check before idle actions:

```rust
    // Check for purpose-driven building work
    if let Some((action, building_id, target_pos)) = should_seek_building_work(
        needs,
        context.building_skill,
        context.nearest_building_site,
    ) {
        let task = Task::new(action, TaskPriority::Normal, context.current_tick)
            .with_building(building_id)
            .with_position(target_pos);
        return Some(task);
    }
```

**Step 3: Update SelectionContext**

Add new fields to `SelectionContext`:

```rust
pub struct SelectionContext<'a> {
    // ... existing fields ...
    pub building_skill: f32,
    pub nearest_building_site: Option<(BuildingId, Vec2, f32)>,
}
```

**Step 4: Update caller in tick.rs**

Update the call site that creates `SelectionContext` to include the new fields.

**Step 5: Write test**

```rust
    #[test]
    fn test_purpose_driven_building_work() {
        use crate::city::building::BuildingId;

        let mut needs = Needs::default();
        needs.purpose = 0.8; // High purpose need

        let building_site = Some((BuildingId::new(1), Vec2::new(10.0, 10.0), 5.0));

        // High skill, high purpose = should seek work
        let result = should_seek_building_work(&needs, 0.8, building_site);
        assert!(result.is_some());

        // Low purpose = should not seek work
        needs.purpose = 0.3;
        let result = should_seek_building_work(&needs, 0.8, building_site);
        assert!(result.is_none());
    }
```

**Step 6: Run tests**

Run: `cargo test --lib simulation::action_select`
Expected: PASS

**Step 7: Commit**

```bash
git add src/simulation/action_select.rs src/simulation/tick.rs
git commit -m "feat(action): add purpose-driven building work seeking"
```

---

## Task 16: Add Production Tick System

**Files:**
- Create: `src/city/production.rs`
- Modify: `src/city/mod.rs`
- Modify: `src/simulation/tick.rs`

**Step 1: Create production system**

Create `src/city/production.rs`:

```rust
//! Production system - processes building production each tick

use crate::city::building::BuildingArchetype;
use crate::city::recipe::RecipeCatalog;
use crate::city::stockpile::Stockpile;

/// Result of production tick
#[derive(Debug, Clone)]
pub struct ProductionResult {
    pub building_idx: usize,
    pub recipe_id: String,
    pub cycles_completed: u32,
}

/// Process production for all active buildings
pub fn tick_production(
    buildings: &mut BuildingArchetype,
    recipes: &RecipeCatalog,
    stockpile: &mut Stockpile,
) -> Vec<ProductionResult> {
    let mut results = Vec::new();

    for i in buildings.iter_producing().collect::<Vec<_>>() {
        if let Some(recipe_id) = &buildings.active_recipes[i].clone() {
            if let Some(recipe) = recipes.get(recipe_id) {
                // Check if we have inputs
                if !stockpile.has_materials(&recipe.inputs) {
                    continue; // Can't produce without materials
                }

                // Calculate progress this tick
                let workers = buildings.production_workers[i];
                let rate = recipe.production_rate(workers);
                let progress = rate / recipe.duration_ticks as f32;

                if buildings.advance_production(i, progress) {
                    // Production cycle complete!
                    // Consume inputs
                    stockpile.consume_materials(&recipe.inputs);

                    // Produce outputs
                    for (resource, amount) in &recipe.outputs {
                        stockpile.add(*resource, *amount);
                    }

                    results.push(ProductionResult {
                        building_idx: i,
                        recipe_id: recipe_id.clone(),
                        cycles_completed: 1,
                    });
                }
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::city::building::{BuildingType, BuildingId, BuildingState};
    use crate::core::types::Vec2;
    use crate::simulation::resource_zone::ResourceType;

    #[test]
    fn test_tick_production() {
        let mut buildings = BuildingArchetype::new();
        let id = BuildingId::new(1);
        buildings.spawn(id, BuildingType::Farm, Vec2::new(0.0, 0.0), 0);
        buildings.states[0] = BuildingState::Complete;
        buildings.start_production(0, "farm_food".into());
        buildings.production_workers[0] = 2;

        let recipes = RecipeCatalog::with_defaults();
        let mut stockpile = Stockpile::new();

        // Run enough ticks to complete one cycle
        // farm_food: 100 ticks at rate 1.0 (2 workers)
        for _ in 0..100 {
            tick_production(&mut buildings, &recipes, &mut stockpile);
        }

        // Should have produced food
        assert!(stockpile.get(ResourceType::Food) > 0);
    }
}
```

**Step 2: Update module exports**

In `src/city/mod.rs`:

```rust
pub mod production;
pub use production::{tick_production, ProductionResult};
```

**Step 3: Integrate into main tick**

In `src/simulation/tick.rs`, add to `run_simulation_tick`:

```rust
    // Run production for buildings
    // Note: This requires a stockpile per settlement - for MVP, use a global stockpile
    // TODO: Per-settlement stockpiles
```

**Step 4: Run tests**

Run: `cargo test --lib city::production::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add src/city/production.rs src/city/mod.rs
git commit -m "feat(city): add production tick system"
```

---

## Task 17: End-to-End Integration Test

**Files:**
- Create: `tests/city_integration.rs`

**Step 1: Create integration test**

Create `tests/city_integration.rs`:

```rust
//! Integration tests for city layer

use arc_citadel::ecs::world::World;
use arc_citadel::city::building::{BuildingType, BuildingState};
use arc_citadel::entity::tasks::{Task, TaskPriority};
use arc_citadel::actions::catalog::ActionId;
use arc_citadel::simulation::tick::run_simulation_tick;
use arc_citadel::core::types::Vec2;

#[test]
fn test_complete_construction_workflow() {
    let mut world = World::new();

    // Spawn builder with high skill
    let builder = world.spawn_human("Mason".into());
    let idx = world.humans.index_of(builder).unwrap();
    world.humans.building_skills[idx] = 0.8;
    world.humans.positions[idx] = Vec2::new(50.0, 50.0);

    // Spawn a wall to build (80 work required)
    let building_id = world.spawn_building(BuildingType::Wall, Vec2::new(50.0, 50.0));

    // Assign build task
    let task = Task::new(ActionId::Build, TaskPriority::Normal, 0)
        .with_building(building_id);
    world.humans.task_queues[idx].push(task);

    // Track initial skill
    let initial_skill = world.humans.building_skills[idx];

    // Run until building complete (max 200 ticks)
    let mut completed = false;
    for tick in 0..200 {
        run_simulation_tick(&mut world);

        let building_idx = world.buildings.index_of(building_id).unwrap();
        if world.buildings.states[building_idx] == BuildingState::Complete {
            completed = true;
            println!("Building completed at tick {}", tick);
            break;
        }
    }

    assert!(completed, "Building should complete within 200 ticks");

    // Skill should have improved
    let final_skill = world.humans.building_skills[idx];
    assert!(final_skill > initial_skill, "Skill should improve after completing building");
}

#[test]
fn test_multiple_builders_faster() {
    let mut world1 = World::new();
    let mut world2 = World::new();

    // World 1: Single builder
    let builder1 = world1.spawn_human("Solo".into());
    let idx1 = world1.humans.index_of(builder1).unwrap();
    world1.humans.building_skills[idx1] = 0.5;
    world1.humans.positions[idx1] = Vec2::new(50.0, 50.0);
    let building1 = world1.spawn_building(BuildingType::House, Vec2::new(50.0, 50.0));
    world1.humans.task_queues[idx1].push(
        Task::new(ActionId::Build, TaskPriority::Normal, 0).with_building(building1)
    );

    // World 2: Two builders
    let builder2a = world2.spawn_human("Duo1".into());
    let builder2b = world2.spawn_human("Duo2".into());
    let idx2a = world2.humans.index_of(builder2a).unwrap();
    let idx2b = world2.humans.index_of(builder2b).unwrap();
    world2.humans.building_skills[idx2a] = 0.5;
    world2.humans.building_skills[idx2b] = 0.5;
    world2.humans.positions[idx2a] = Vec2::new(50.0, 50.0);
    world2.humans.positions[idx2b] = Vec2::new(50.0, 50.0);
    let building2 = world2.spawn_building(BuildingType::House, Vec2::new(50.0, 50.0));
    world2.humans.task_queues[idx2a].push(
        Task::new(ActionId::Build, TaskPriority::Normal, 0).with_building(building2)
    );
    world2.humans.task_queues[idx2b].push(
        Task::new(ActionId::Build, TaskPriority::Normal, 0).with_building(building2)
    );

    // Run both until complete
    let mut ticks1 = 0;
    let mut ticks2 = 0;

    for tick in 0..500 {
        if world1.buildings.states[0] != BuildingState::Complete {
            run_simulation_tick(&mut world1);
            ticks1 = tick + 1;
        }
        if world2.buildings.states[0] != BuildingState::Complete {
            run_simulation_tick(&mut world2);
            ticks2 = tick + 1;
        }
        if world1.buildings.states[0] == BuildingState::Complete &&
           world2.buildings.states[0] == BuildingState::Complete {
            break;
        }
    }

    println!("Single builder: {} ticks, Two builders: {} ticks", ticks1, ticks2);
    assert!(ticks2 < ticks1, "Two builders should complete faster than one");
}
```

**Step 2: Run integration tests**

Run: `cargo test --test city_integration`
Expected: 2 tests PASS

**Step 3: Commit**

```bash
git add tests/city_integration.rs
git commit -m "test: add city layer integration tests"
```

---

## Summary

This plan implements the city foundation with:

1. **Building System**: `BuildingArchetype` with SoA layout, 6 building types, construction states
2. **Construction**: Worker contribution formula with diminishing returns, skill-based progress
3. **Resources**: Extended `ResourceType` with 6 types, `Stockpile` for storage
4. **Production**: Recipe system, building production cycles
5. **Integration**: Buildings in World, construction in tick, purpose-driven work seeking

**Post-MVP Features (documented as TODOs):**
- Full Skills struct with 6 skills
- Authority priority levels
- Quality system for buildings
- Material waste based on skill
- Guild emergence

---

**Plan complete and saved to `docs/plans/2026-01-02-city-foundation-production.md`. Two execution options:**

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**
