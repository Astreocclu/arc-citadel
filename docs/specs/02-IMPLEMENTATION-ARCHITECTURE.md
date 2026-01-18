# 02-IMPLEMENTATION-ARCHITECTURE
> Technical stack, module structure, and architectural decisions

## Overview

Arc Citadel is built as a distributed system with a Rust simulation core, PostgreSQL persistence, async LLM integration, and cache-efficient entity storage using Structure of Arrays (SoA) layout.

---

## Technical Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Simulation Core** | Rust 2021 | Game logic, tick execution, entity management |
| **Data Layout** | Structure of Arrays (SoA) | Cache-efficient entity iteration |
| **ECS Pattern** | Custom Archetype-based | Entity management without OOP overhead |
| **Persistence** | PostgreSQL | World state, entity data, cross-session storage |
| **Async Runtime** | Tokio | Non-blocking I/O, LLM calls, network |
| **LLM Integration** | OpenAI API | Natural language parsing |
| **Spatial Index** | Sparse Hash Grid | O(1) neighbor queries |
| **Visualization** | egui | Real-time simulation visualization |

---

## Module Structure

```
src/
├── main.rs                 # Entry point, game loop
├── lib.rs                  # Library exports
│
├── core/                   # Foundation types
│   ├── mod.rs
│   ├── types.rs            # EntityId, Tick, Vec2, etc.
│   ├── error.rs            # Error types, Result aliases
│   ├── astronomy.rs        # Seasons, moons, eclipses
│   └── config.rs           # SimulationConfig, tunable constants
│
├── ecs/                    # Entity Component System
│   ├── mod.rs
│   ├── world.rs            # Central World struct
│   ├── entity.rs           # EntityId management
│   └── registry.rs         # Entity lookup
│
├── spatial/                # Spatial queries
│   ├── mod.rs
│   ├── sparse_hash.rs      # SparseHashGrid implementation
│   ├── grid.rs             # Grid<T> for terrain/maps
│   └── queries.rs          # Radius, line-of-sight queries
│
├── entity/                 # Entity components
│   ├── mod.rs
│   ├── needs.rs            # Universal needs system
│   ├── thoughts.rs         # ThoughtBuffer, decay
│   ├── tasks.rs            # TaskQueue, Task enum
│   ├── body.rs             # BodyState, wounds
│   ├── equipment.rs        # Weapon/armor slots (planned)
│   └── species/
│       ├── mod.rs
│       ├── human.rs        # HumanArchetype, HumanValues
│       └── traits.rs       # Species trait definitions
│
├── genetics/               # Genome and phenotype (planned)
│   ├── mod.rs
│   ├── genome.rs           # DNA representation
│   ├── phenotype.rs        # Physical traits
│   └── values.rs           # Value derivation
│
├── simulation/             # Core game loop
│   ├── mod.rs
│   ├── tick.rs             # Main tick function
│   ├── perception.rs       # What entities notice
│   ├── thought_gen.rs      # Perception → Thought
│   ├── action_select.rs    # Thought → Action
│   └── production.rs       # Building production tick
│
├── actions/                # Action definitions
│   ├── mod.rs
│   ├── catalog.rs          # ActionId enum, properties
│   └── execution/          # Per-action execution logic
│       ├── mod.rs
│       ├── movement.rs
│       ├── social.rs
│       └── work.rs
│
├── combat/                 # Combat resolution
│   ├── mod.rs
│   ├── weapons.rs          # Weapon types, materials
│   ├── armor.rs            # Armor, penetration
│   ├── resolution.rs       # resolve_attack()
│   ├── wounds.rs           # Wound effects
│   └── morale.rs           # Combat morale
│
├── city/                   # City/stronghold layer (planned)
│   ├── mod.rs
│   ├── building.rs         # Building types
│   ├── construction.rs     # Construction queue
│   ├── zone.rs             # Resource zones
│   ├── production.rs       # Production chains
│   └── stockpile.rs        # Storage
│
├── campaign/               # Strategic map layer
│   ├── mod.rs
│   ├── map.rs              # Campaign hex map
│   ├── location.rs         # Location types
│   └── polity.rs           # Factions, settlements
│
├── battle/                 # Tactical combat layer
│   ├── mod.rs
│   ├── battle_map.rs       # Terrain, elevation
│   ├── courier.rs          # Order delays
│   ├── formation.rs        # Formation types
│   └── execution.rs        # Battle tick
│
├── llm/                    # LLM integration
│   ├── mod.rs
│   ├── client.rs           # API client
│   ├── parser.rs           # Natural language → ParsedIntent
│   └── context.rs          # Context building for prompts
│
└── ui/                     # User interface
    ├── mod.rs
    └── egui_app.rs         # egui visualization
```

---

## Architectural Patterns

### Structure of Arrays (SoA)

Entities are stored in parallel arrays for cache efficiency:

```rust
pub struct HumanArchetype {
    // Each field is a parallel array indexed by internal index
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
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
    /// Get internal index for an entity
    pub fn index_of(&self, id: EntityId) -> Option<usize> {
        self.ids.iter().position(|&i| i == id)
    }

    /// Iterate over all living entities
    pub fn iter_alive(&self) -> impl Iterator<Item = usize> + '_ {
        self.alive.iter()
            .enumerate()
            .filter(|(_, &alive)| alive)
            .map(|(idx, _)| idx)
    }
}
```

**Benefits**:
- Cache-efficient iteration (all positions together, all needs together)
- No virtual dispatch overhead
- Easy SIMD optimization potential
- Clear data ownership

**Trade-offs**:
- Index management complexity
- Entity lookup O(n) without secondary index
- Per-species boilerplate

### Archetype-Based ECS

Instead of component-per-entity storage, entities are grouped by archetype (species):

```rust
pub struct World {
    // Each species is a separate archetype
    pub humans: HumanArchetype,
    pub dwarves: DwarfArchetype,  // Future
    pub elves: ElfArchetype,       // Future

    // Non-entity data
    pub spatial_index: SparseHashGrid,
    pub current_tick: Tick,
    pub calendar: Calendar,
    pub buildings: Vec<Building>,  // Planned
}
```

**Species Dispatch**:
```rust
impl World {
    pub fn get_position(&self, id: EntityId) -> Option<Vec2> {
        // Dispatch based on entity ID prefix or lookup table
        if let Some(idx) = self.humans.index_of(id) {
            return Some(self.humans.positions[idx]);
        }
        // ... other species
        None
    }
}
```

### Sparse Hash Grid

O(1) spatial queries via hash-based cell lookup:

```rust
pub struct SparseHashGrid {
    cells: HashMap<(i32, i32), Vec<EntityId>>,
    cell_size: f32,  // Default: 10.0 units
}

impl SparseHashGrid {
    pub fn cell_key(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }

    pub fn query_radius(&self, center: Vec2, radius: f32) -> Vec<EntityId> {
        let mut results = Vec::new();
        let cell_radius = (radius / self.cell_size).ceil() as i32;
        let center_cell = self.cell_key(center);

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                let key = (center_cell.0 + dx, center_cell.1 + dy);
                if let Some(entities) = self.cells.get(&key) {
                    results.extend(entities.iter().cloned());
                }
            }
        }
        results
    }
}
```

---

## Data Flow

### Tick Execution

```rust
pub fn tick(world: &mut World) {
    // Phase 1: Needs update
    for idx in world.humans.iter_alive() {
        world.humans.needs[idx].decay();
    }

    // Phase 2: Perception
    let spatial = SparseHashGrid::from_world(world);
    let perceptions = run_perception(world, &spatial);

    // Phase 3: Thought generation
    for (entity_id, perceptions) in perceptions {
        generate_thoughts(world, entity_id, &perceptions);
    }

    // Phase 4: Thought decay
    for idx in world.humans.iter_alive() {
        world.humans.thoughts[idx].decay();
    }

    // Phase 5: Action selection (idle entities only)
    for idx in world.humans.iter_alive() {
        if world.humans.task_queues[idx].is_empty() {
            let action = select_action(world, idx);
            world.humans.task_queues[idx].push(action);
        }
    }

    // Phase 6: Task execution
    for idx in world.humans.iter_alive() {
        execute_task(world, idx);
    }

    // Phase 7: Advance time
    world.current_tick = world.current_tick.next();
}
```

### LLM Command Flow

```
Player Input ──┬──► LLM Service ──► ParsedIntent ──► World Mutation
               │        │
               │        ▼
               │   Context Builder
               │   (entity names, locations, state)
               │
               └──► Fallback Parser (if LLM unavailable)
```

---

## Persistence Strategy

### PostgreSQL Schema (Planned)

```sql
-- Core entity storage
CREATE TABLE entities (
    id BIGINT PRIMARY KEY,
    species VARCHAR(32) NOT NULL,
    name VARCHAR(128),
    created_at TIMESTAMP DEFAULT NOW()
);

-- Per-species data tables
CREATE TABLE human_data (
    entity_id BIGINT PRIMARY KEY REFERENCES entities(id),
    position_x FLOAT NOT NULL,
    position_y FLOAT NOT NULL,
    strength FLOAT NOT NULL,
    -- ... other fields
    values_json JSONB NOT NULL,
    needs_json JSONB NOT NULL
);

-- World state
CREATE TABLE world_state (
    id SERIAL PRIMARY KEY,
    tick BIGINT NOT NULL,
    calendar_json JSONB NOT NULL,
    saved_at TIMESTAMP DEFAULT NOW()
);
```

### Save/Load Pattern

```rust
impl World {
    pub async fn save(&self, pool: &PgPool) -> Result<()> {
        // Transaction for consistency
        let mut tx = pool.begin().await?;

        // Save world state
        sqlx::query("INSERT INTO world_state (tick, calendar_json) VALUES ($1, $2)")
            .bind(self.current_tick.0 as i64)
            .bind(serde_json::to_value(&self.calendar)?)
            .execute(&mut *tx)
            .await?;

        // Save entities by species
        for idx in 0..self.humans.ids.len() {
            // ... save each human
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn load(pool: &PgPool) -> Result<Self> {
        // Load most recent world state
        let state = sqlx::query_as("SELECT * FROM world_state ORDER BY saved_at DESC LIMIT 1")
            .fetch_one(pool)
            .await?;

        // Load entities
        // ...

        Ok(world)
    }
}
```

---

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ArcError {
    #[error("Entity {0} not found")]
    EntityNotFound(EntityId),

    #[error("Invalid action {0:?} for entity {1}")]
    InvalidAction(ActionId, EntityId),

    #[error("LLM error: {0}")]
    LlmError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type ArcResult<T> = Result<T, ArcError>;
```

### Graceful Degradation

```rust
// LLM calls degrade gracefully
impl LlmClient {
    pub async fn parse_command(&self, input: &str) -> ParsedIntent {
        match self.call_api(input).await {
            Ok(intent) => intent,
            Err(e) => {
                log::warn!("LLM unavailable: {e}, using fallback parser");
                FallbackParser::parse(input)
            }
        }
    }
}
```

---

## Performance Considerations

### Target Metrics

| Metric | Target | Notes |
|--------|--------|-------|
| Tick time | < 16ms | 60 TPS capability |
| Entity count | 1000+ | Per settlement |
| Spatial query | < 1ms | Radius queries |
| Memory per entity | < 1KB | SoA efficiency |

### Optimization Strategies

1. **SoA Layout**: Parallel arrays for cache-efficient iteration
2. **Sparse Hash Grid**: O(1) spatial queries
3. **Lazy Evaluation**: Compute perception only for entities that need it
4. **Batch LLM Calls**: Aggregate commands when possible
5. **Incremental Spatial Update**: Don't rebuild entire grid each tick

### Profiling Points

```rust
// Key functions to profile
tick()                    // Full tick time
run_perception()          // O(n²) potential
select_action()           // Per-entity decision
execute_task()            // Task progress
SparseHashGrid::rebuild() // Grid reconstruction
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn needs_decay_over_time() {
        let mut needs = Needs::default();
        needs.decay();
        assert!(needs.hunger > 0.0);
    }

    #[test]
    fn spatial_query_finds_nearby() {
        let mut grid = SparseHashGrid::new(10.0);
        let entity = EntityId(1);
        grid.insert(entity, Vec2::new(5.0, 5.0));
        let results = grid.query_radius(Vec2::ZERO, 10.0);
        assert!(results.contains(&entity));
    }
}
```

### Integration Tests

```rust
#[test]
fn entity_perceives_nearby_entities() {
    let mut world = World::new();
    let observer = spawn_human(&mut world, Vec2::new(0.0, 0.0));
    let target = spawn_human(&mut world, Vec2::new(5.0, 0.0));

    tick(&mut world);

    let thoughts = &world.humans.thoughts[world.humans.index_of(observer).unwrap()];
    assert!(thoughts.iter().any(|t| matches!(t.source, ThoughtSource::Perception(_))));
}
```

### Property Tests

```rust
#[quickcheck]
fn needs_stay_bounded(initial: f32, ticks: u8) -> bool {
    let mut needs = Needs::with_hunger(initial.clamp(0.0, 1.0));
    for _ in 0..ticks {
        needs.decay();
    }
    needs.hunger >= 0.0 && needs.hunger <= 1.0
}
```

---

## Configuration

### SimulationConfig

```rust
pub struct SimulationConfig {
    // Tick settings
    pub tick_duration_ms: u64,        // Real-time tick length
    pub max_entities: usize,          // Hard limit

    // Needs
    pub need_decay_rate: f32,         // Per-tick decay
    pub need_satisfaction_mult: f32,  // Action satisfaction

    // Perception
    pub perception_range: f32,        // Base awareness range
    pub grid_cell_size: f32,          // Spatial hash cell size

    // Thoughts
    pub max_thoughts: usize,          // Buffer capacity
    pub thought_decay_rate: f32,      // Per-tick decay

    // Combat
    pub penetration_curve_steepness: f32,
    pub fatigue_combat_penalty: f32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            tick_duration_ms: 16,
            max_entities: 10000,
            need_decay_rate: 0.001,
            need_satisfaction_mult: 1.0,
            perception_range: 20.0,
            grid_cell_size: 10.0,
            max_thoughts: 20,
            thought_decay_rate: 0.05,
            penetration_curve_steepness: 10.0,
            fatigue_combat_penalty: 0.4,
        }
    }
}
```

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [01-GAME-DESIGN-DOCUMENT](01-GAME-DESIGN-DOCUMENT.md) | Design requirements |
| [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md) | Entity implementation details |
| [10-PERFORMANCE-ARCHITECTURE-SPEC](10-PERFORMANCE-ARCHITECTURE-SPEC.md) | Optimization details |
| [09-GAP-ANALYSIS](09-GAP-ANALYSIS.md) | Implementation status |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
