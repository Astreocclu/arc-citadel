# ECS Module

> Custom Entity Component System with Structure-of-Arrays (SoA) data layout. Lightweight, purpose-built for Arc Citadel.

## Module Structure

```
ecs/
├── mod.rs      # Module exports
└── world.rs    # World struct and entity management
```

## Design Philosophy

Arc Citadel uses a custom ECS for these benefits:
- **Species-specific archetypes** with SoA layout for cache efficiency
- **Simple iteration** without complex query systems
- **Direct field access** without runtime type lookups
- **Full control** over entity ID generation and lifecycle

## World Structure

```rust
pub struct World {
    pub current_tick: u64,
    entity_registry: AHashMap<EntityId, (Species, usize)>,
    pub humans: HumanArchetype,
    next_indices: AHashMap<Species, usize>,
}
```

| Field | Purpose |
|-------|---------|
| `current_tick` | Simulation time counter |
| `entity_registry` | Maps EntityId → (Species, index) |
| `humans` | SoA storage for human entities |
| `next_indices` | Next available index per species |

## Key Operations

### Creating the World
```rust
let mut world = World::new();
```

### Spawning Entities
```rust
let entity_id = world.spawn_human("Marcus".into());
// Entity created at next available index
// Returns unique EntityId
```

### Looking Up Entities
```rust
if let Some((species, index)) = world.get_entity_info(entity_id) {
    match species {
        Species::Human => {
            let name = &world.humans.names[index];
            let needs = &world.humans.needs[index];
        }
        _ => {}
    }
}
```

### Iterating Living Entities
```rust
for i in world.humans.iter_living() {
    let name = &world.humans.names[i];
    let needs = &mut world.humans.needs[i];
    // Process entity at index i
}
```

### Advancing Time
```rust
world.tick();  // Increments current_tick
```

## Structure of Arrays (SoA) Pattern

### Why SoA?

When iterating over a single component type (e.g., all positions), SoA keeps data contiguous in memory:

```rust
// SoA layout - cache-friendly iteration
pub struct HumanArchetype {
    pub ids: Vec<EntityId>,      // [id0, id1, id2, ...]
    pub positions: Vec<Vec2>,    // [pos0, pos1, pos2, ...]
    pub needs: Vec<Needs>,       // [needs0, needs1, needs2, ...]
    // Each array is contiguous
}

// Iterating positions loads only position data
for i in 0..archetype.positions.len() {
    let pos = &archetype.positions[i];
    // Cache stays hot - only position data loaded
}
```

### Human Archetype Example

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
}
```

### Index-Based Access

All arrays are parallel - same index refers to same entity:

```rust
let idx = world.humans.index_of(entity_id)?;

// All these refer to the same entity
let name = &world.humans.names[idx];
let pos = &world.humans.positions[idx];
let needs = &world.humans.needs[idx];
let values = &world.humans.values[idx];
```

## Adding a New Species

When implementing dwarves, elves, or other species:

### 1. Create Species-Specific Types

```rust
// src/entity/species/dwarf.rs
pub struct DwarfValues {
    pub tradition: f32,
    pub craftsmanship: f32,
    pub clan_honor: f32,
    pub stone_affinity: f32,
    // ... dwarf-specific value vocabulary
}

pub struct DwarfArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub birth_ticks: Vec<Tick>,
    pub positions: Vec<Vec2>,
    pub values: Vec<DwarfValues>,  // Different type than HumanValues
    // ... parallel arrays for all components
}
```

### 2. Add to World

```rust
pub struct World {
    pub humans: HumanArchetype,
    pub dwarves: DwarfArchetype,  // Add new archetype
    // ...
}

impl World {
    pub fn spawn_dwarf(&mut self, name: String) -> EntityId {
        let entity_id = EntityId::new();
        let index = *self.next_indices.get(&Species::Dwarf).unwrap();

        self.dwarves.spawn(entity_id, name, self.current_tick);
        self.entity_registry.insert(entity_id, (Species::Dwarf, index));
        *self.next_indices.get_mut(&Species::Dwarf).unwrap() += 1;

        entity_id
    }
}
```

### 3. Update Systems

Systems that process all entities need species-aware iteration:

```rust
// In tick.rs
fn update_needs(world: &mut World) {
    // Process humans
    for i in world.humans.iter_living() {
        world.humans.needs[i].decay(dt, is_active);
    }

    // Process dwarves
    for i in world.dwarves.iter_living() {
        world.dwarves.needs[i].decay(dt, is_active);
    }
}
```

## Best Practices

### Accessing Components
```rust
// Get index first, then access multiple components
let idx = world.humans.index_of(entity_id)?;
let needs = &world.humans.needs[idx];
let values = &world.humans.values[idx];
let thoughts = &mut world.humans.thoughts[idx];
```

### Modifying Components
```rust
let idx = world.humans.index_of(entity_id)?;
world.humans.needs[idx].satisfy(NeedType::Food, 0.5);
world.humans.thoughts[idx].add(new_thought);
world.humans.task_queues[idx].push(new_task);
```

### Iterating Safely
```rust
// Use iter_living() to skip dead entities
for i in world.humans.iter_living() {
    // Only processes entities where alive[i] == true
}
```

### Entity Count
```rust
let total = world.entity_count();  // All species
let humans = world.humans.count(); // Just humans
```

## Testing

```bash
cargo test --lib ecs::
```

### Key Tests
- `test_world_creation` - World initializes with zero entities
- `test_spawn_human` - Spawning increases entity count, returns valid ID
