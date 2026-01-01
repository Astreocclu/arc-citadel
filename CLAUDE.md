# Arc Citadel - AI Agent Onboarding

> Deep simulation strategy game where entity behavior emerges naturally from values, needs, and thoughts. Natural language commands flow through LLM parsing into structured game actions.

## Quick Start

```bash
# Build the project
cargo build

# Run the simulation
cargo run

# Run all tests
cargo test

# Run specific module tests
cargo test --lib simulation::
```

## Project Vision

Arc Citadel creates **emergent gameplay** through layered systems:
- Entities perceive their environment filtered by their values
- Perceptions generate thoughts weighted by personality
- Thoughts compete for attention based on needs
- Actions emerge from the interaction of all these systems

The magic happens in the **interactions between systems**, not in scripted behaviors.

## Core Design Principles

### 1. Property Interaction

Properties affect each other through meaningful relationships:

```rust
// Strength, weapon weight, and fatigue interact to produce damage
fn calculate_damage(strength: f32, weapon_weight: f32, fatigue: f32) -> f32 {
    let base_force = strength * weapon_weight;
    let fatigue_reduction = 1.0 - (fatigue * 0.5);
    base_force * fatigue_reduction
}
```

### 2. Type-Safe Species Values

Each species has its own value vocabulary as a distinct type:

```rust
// Humans value these concepts
pub struct HumanValues {
    pub honor: f32,      // Social standing, keeping one's word
    pub beauty: f32,     // Aesthetic appreciation
    pub comfort: f32,    // Physical ease and safety
    pub ambition: f32,   // Drive for advancement
    pub loyalty: f32,    // Group attachment
    pub love: f32,       // Individual attachment
    pub justice: f32,    // Fairness and equity
    pub curiosity: f32,  // Desire to explore and learn
    pub safety: f32,     // Self-preservation
    pub piety: f32,      // Spiritual devotion
}

// Dwarves (future) will have their own vocabulary
pub struct DwarfValues {
    pub tradition: f32,
    pub craftsmanship: f32,
    // ... different concepts entirely
}
```

### 3. LLM as Command Parser

The LLM translates player intent into structured game commands:

```rust
// Player types natural language
"Have Marcus build a defensive wall near the eastern gate"

// LLM parses to structured intent
ParsedIntent {
    action: IntentAction::Build,
    target: Some("defensive wall"),
    location: Some("eastern gate"),
    subjects: Some(vec!["Marcus"]),
    priority: IntentPriority::Normal,
}

// Game systems execute the intent
```

### 4. Emergent Behavior

Behavior emerges from the interaction of values, needs, and context:

```rust
// Values filter what entities notice in perception
// Needs weight which thoughts feel urgent
// Context determines available actions
// The combination produces unique behavior for each entity
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        SIMULATION TICK                               │
│                                                                      │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐      │
│  │Perception│───▶│ Thought  │───▶│  Action  │───▶│   Task   │      │
│  │  System  │    │Generation│    │Selection │    │Execution │      │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘      │
│       │               │               │               │             │
│       ▼               ▼               ▼               ▼             │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐      │
│  │ Spatial  │    │  Entity  │    │  Entity  │    │   ECS    │      │
│  │  Grid    │    │ Thoughts │    │   Needs  │    │  World   │      │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                      PLAYER COMMAND FLOW                             │
│                                                                      │
│  Player Input ──▶ LLM Parser ──▶ ParsedIntent ──▶ Game Systems      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Module Map

| Module | Purpose | Key Entry Points |
|--------|---------|------------------|
| `core/` | Foundation types and errors | `types.rs`, `error.rs` |
| `ecs/` | Entity Component System | `world.rs` |
| `spatial/` | Spatial queries and grids | `sparse_hash.rs`, `grid.rs` |
| `entity/` | Entity components | `needs.rs`, `thoughts.rs`, `tasks.rs` |
| `genetics/` | Genome and phenotype | `genome.rs`, `values.rs` |
| `simulation/` | Core game loop | `tick.rs`, `perception.rs`, `action_select.rs` |
| `actions/` | Action definitions | `catalog.rs` |
| `combat/` | Combat resolution | `resolution.rs`, `wounds.rs` |
| `llm/` | Natural language parsing | `client.rs`, `parser.rs`, `context.rs` |
| `campaign/` | Strategic map layer | `map.rs`, `location.rs` |
| `battle/` | Tactical combat | `battle_map.rs`, `execution.rs` |
| `ui/` | Terminal interface | `terminal.rs`, `display.rs` |

## Key Data Flows

### Simulation Tick
```
tick.rs
  → update_needs()      : Needs decay over time
  → run_perception()    : Build spatial index, query neighbors
  → generate_thoughts() : Create thoughts from perceptions
  → decay_thoughts()    : Fade old thoughts
  → select_actions()    : Choose actions for idle entities
  → execute_tasks()     : Progress current tasks
  → world.tick()        : Advance simulation time
```

### Entity Decision Making
```
Perception (what entity notices)
  → filtered by Values (what matters to this entity)
  → generates Thoughts (emotional/cognitive reactions)
  → weighted by Needs (what feels urgent)
  → produces Action (behavioral response)
```

### Player Commands
```
Natural language input
  → LLM parsing (llm/parser.rs)
  → Structured ParsedIntent
  → Converted to Task
  → Added to entity's TaskQueue
  → Executed by simulation systems
```

## Critical File Paths

Start here when exploring the codebase:

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point, game loop |
| `src/ecs/world.rs` | Central world state |
| `src/simulation/tick.rs` | System orchestration |
| `src/simulation/action_select.rs` | Decision logic |
| `src/entity/species/human.rs` | Human archetype (SoA pattern) |
| `src/entity/needs.rs` | Universal needs system |
| `src/entity/thoughts.rs` | Thought buffer |
| `src/llm/parser.rs` | Command parsing |

## Structure of Arrays (SoA) Pattern

Entities use SoA layout for cache-efficient iteration:

```rust
pub struct HumanArchetype {
    // Each field is a parallel array
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

// Access by index
let idx = archetype.index_of(entity_id)?;
let needs = &archetype.needs[idx];
let values = &archetype.values[idx];
```

## Common Development Tasks

### Adding a New Action

1. Add variant to `ActionId` in `src/actions/catalog.rs`:
```rust
pub enum ActionId {
    // ... existing actions
    MyNewAction,  // Add here
}
```

2. Implement category and properties:
```rust
impl ActionId {
    pub fn category(&self) -> ActionCategory {
        match self {
            Self::MyNewAction => ActionCategory::Work,
            // ...
        }
    }

    pub fn satisfies_needs(&self) -> Vec<(NeedType, f32)> {
        match self {
            Self::MyNewAction => vec![(NeedType::Purpose, 0.2)],
            // ...
        }
    }
}
```

3. Add execution logic in appropriate action module

### Adding a New Species

1. Create `src/entity/species/dwarf.rs`:
```rust
pub struct DwarfValues {
    pub tradition: f32,
    pub craftsmanship: f32,
    pub clan_honor: f32,
    // ... species-specific values
}

pub struct DwarfArchetype {
    pub ids: Vec<EntityId>,
    pub values: Vec<DwarfValues>,
    // ... parallel arrays for all components
}
```

2. Add to `src/ecs/world.rs`:
```rust
pub struct World {
    pub humans: HumanArchetype,
    pub dwarves: DwarfArchetype,  // Add new archetype
    // ...
}
```

3. Implement spawn method and species-specific perception filtering

### Modifying Entity Behavior

Tune behavior by adjusting how systems interact:

1. **Perception filtering** (`src/simulation/perception.rs`): What entities notice based on values
2. **Thought generation** (`src/simulation/thought_gen.rs`): How perceptions become thoughts
3. **Action weighting** (`src/simulation/action_select.rs`): How needs and values influence choice

## Testing

```bash
# Run all tests
cargo test

# Run with output visible
cargo test -- --nocapture

# Run specific test
cargo test test_needs_decay

# Run module tests
cargo test --lib entity::
cargo test --lib simulation::
```

### Key Test Files
- `tests/emergence_tests.rs` - Integration tests for emergent behavior
- Module-specific unit tests in each `*.rs` file

## Performance Considerations

- **Spatial queries**: `SparseHashGrid` with 10.0 cell size for O(1) neighbor lookups
- **Entity iteration**: SoA layout keeps cache hot during system updates
- **Thought management**: Buffer capped at 20 thoughts, auto-eviction of weakest
- **LLM calls**: Async via tokio, with graceful fallback when unavailable

## Implementation Status

| Module | Status | Notes |
|--------|--------|-------|
| core/ | Complete | Types, errors |
| ecs/ | Complete | World, entity management |
| spatial/ | Complete | Grid, sparse hash |
| entity/ | Complete | Needs, thoughts, tasks, human archetype |
| simulation/ | Complete | Core loop, perception, action selection |
| actions/ | Partial | Catalog complete, execution stubs |
| llm/ | Complete | Client, parser, context |
| combat/ | Stub | Planned |
| campaign/ | Stub | Planned |
| battle/ | Stub | Planned |
| ui/ | Stub | Planned |
| genetics/ | Stub | Planned |

## Important Implementation Details

These are critical details that affect how systems behave. Missing these can cause subtle bugs.

### Need Satisfaction Multiplier

Actions don't satisfy needs immediately. There's a **0.05x multiplier** per tick:

```rust
// Action says it satisfies food by 0.5
// Actual satisfaction per tick: 0.5 × 0.05 = 0.025

// Over a 20-tick action:
// Total satisfaction: 0.025 × 20 = 0.5
```

This creates meaningful time investment. An entity can't just "eat once" and be full.

### Safety Need is Asymmetric

Unlike other needs that only increase, safety **decreases** when no threats are present:

```rust
// Other needs: increase over time (0.0 → 1.0)
self.rest += 0.001;
self.food += 0.0005;

// Safety: decreases over time (1.0 → 0.0)
self.safety = (self.safety - 0.01).max(0.0);
```

This prevents entities from being permanently scared after a threat is gone.

### Spatial Query Requirements

When using `query_radius`, you must provide an entity-to-index map:

```rust
// Build the map ONCE before querying
let id_to_idx: AHashMap<EntityId, usize> = entity_ids
    .iter()
    .enumerate()
    .map(|(i, &id)| (id, i))
    .collect();

// Then query safely
grid.query_radius(center, radius, positions, &id_to_idx);
```

### Task Priority Ordering

`TaskPriority` uses explicit numeric values for safe ordering:

```rust
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}
```

Higher values = higher priority. The `TaskQueue` relies on this ordering.

### Configuration Reference

All magic numbers are documented in `src/core/config.rs`:

| Parameter | Default | Effect |
|-----------|---------|--------|
| `grid_cell_size` | 10.0 | Spatial query performance |
| `perception_range` | 50.0 | How far entities see |
| `critical_need_threshold` | 0.8 | When needs trigger immediate action |
| `satisfaction_multiplier` | 0.05 | How fast actions satisfy needs |
| `thought_decay_rate` | 0.01 | How fast thoughts fade (100 ticks to 0) |
| `parallel_threshold` | 1000 | When to use parallel processing |

See `SimulationConfig` for the complete list with documentation.

## Implementation Plan Reference

See `docs/plans/2025-12-31-arc-citadel-mvp.md` for detailed implementation tasks.

---

*This documentation is designed for AI agents. Each module has its own README.md with detailed implementation guidance.*
