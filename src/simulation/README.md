# Simulation Module

> The beating heart of Arc Citadel. Orchestrates the perception → thought → action → execution loop each tick.

## Module Structure

```
simulation/
├── mod.rs              # Module exports
├── perception.rs       # What entities notice in their environment
├── thought_gen.rs      # Generate thoughts from perceptions
├── action_select.rs    # Choose actions based on thoughts and needs
├── action_execute.rs   # Execute chosen actions
└── tick.rs            # Orchestrate all systems each tick
```

## Architecture Position

```
                         simulation/
                              │
              ┌───────────────┼───────────────┐
              │               │               │
              ▼               ▼               ▼
        perception.rs   action_select.rs   tick.rs
              │               │               │
              ▼               ▼               ▼
         spatial/          entity/          ecs/
        (queries)      (needs, thoughts)   (world)
```

## System Execution Order

The `tick.rs` file orchestrates systems in this sequence:

```rust
pub fn run_simulation_tick(world: &mut World) {
    update_needs(world);           // 1. Needs decay over time
    let perceptions = run_perception(world);  // 2. Build spatial index, query
    generate_thoughts(world, &perceptions);   // 3. React to perceptions
    decay_thoughts(world);         // 4. Fade old thoughts
    select_actions(world);         // 5. Choose actions for idle entities
    execute_tasks(world);          // 6. Progress current tasks
    world.tick();                  // 7. Advance time
}
```

## Key Components

### Perception System (`perception.rs`)

Determines what each entity notices based on spatial proximity:

```rust
pub struct Perception {
    pub observer: EntityId,
    pub perceived_entities: Vec<PerceivedEntity>,
    pub perceived_objects: Vec<PerceivedObject>,
    pub perceived_events: Vec<PerceivedEvent>,
}

pub struct PerceivedEntity {
    pub entity: EntityId,
    pub distance: f32,
    pub relationship: RelationshipType,
    pub threat_level: f32,
    pub notable_features: Vec<String>,
}
```

**Key function:**
```rust
pub fn perception_system(
    spatial_grid: &SparseHashGrid,
    positions: &[Vec2],
    entity_ids: &[EntityId],
    perception_range: f32,
) -> Vec<Perception>
```

### Action Selection (`action_select.rs`)

Chooses actions based on needs, thoughts, and values:

```rust
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
```

**Selection priority:**
1. **Critical needs** (> 0.8) trigger immediate response
2. **Value impulses** from strong thoughts matching values
3. **Moderate needs** (> 0.6) addressed when idle
4. **Idle actions** based on personality (wander, observe, socialize)

```rust
pub fn select_action_human(ctx: &SelectionContext) -> Option<Task> {
    // Critical needs first
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response(critical, ctx);
    }

    // Already busy? Don't interrupt
    if ctx.has_current_task {
        return None;
    }

    // Value-driven impulses
    if let Some(task) = check_value_impulses(ctx) {
        return Some(task);
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need(ctx) {
        return Some(task);
    }

    // Default to idle behavior
    Some(select_idle_action(ctx))
}
```

## Data Flow

```
World State
    │
    ▼
┌─────────────────┐
│ SparseHashGrid  │ ← Rebuilt from entity positions
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Perception    │ ← Query neighbors within range
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Thought Buffer  │ ← Generate thoughts from perceptions
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Action Selection│ ← Weight by needs + values
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Task Queue    │ ← Push selected task
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Task Execution  │ ← Progress and satisfy needs
└─────────────────┘
```

## Integration Points

### With `entity/` module
- **Reads:** `needs`, `thoughts`, `values`, `body_states`, `task_queues`
- **Writes:** `needs` (satisfaction), `thoughts` (add/decay), `task_queues` (push tasks)

### With `spatial/` module
- Uses `SparseHashGrid` for neighbor queries
- Rebuilds grid each tick from entity positions

### With `ecs/` module
- Receives `&mut World` for all operations
- Iterates via `world.humans.iter_living()`

## Best Practices

### Building Context for Selection
```rust
let ctx = SelectionContext {
    body: &world.humans.body_states[i],
    needs: &world.humans.needs[i],
    thoughts: &world.humans.thoughts[i],
    values: &world.humans.values[i],
    has_current_task: world.humans.task_queues[i].current().is_some(),
    threat_nearby: world.humans.needs[i].safety > 0.5,
    food_available: true,  // TODO: Check actual food sources
    safe_location: world.humans.needs[i].safety < 0.3,
    entity_nearby: !perceptions[i].perceived_entities.is_empty(),
    current_tick: world.current_tick,
};
```

### Generating Thoughts from Perception
```rust
for perceived in &perception.perceived_entities {
    if perceived.threat_level > 0.5 {
        let thought = Thought::new(
            Valence::Negative,
            perceived.threat_level,
            "concern",  // Concept category
            "threatening entity nearby",
            CauseType::Entity,
            world.current_tick,
        );
        world.humans.thoughts[idx].add(thought);
    }
}
```

## Extension Points

### Adding New Perception Types
1. Add struct to `perception.rs`
2. Update `perception_system()` to detect new type
3. Update `generate_thoughts()` to react appropriately

### Adding New Selection Logic
1. Add helper function in `action_select.rs`
2. Call from appropriate priority level in `select_action_human()`
3. Ensure critical needs always take precedence

### Species-Specific Perception
Values filter what entities notice:
```rust
pub fn filter_perception_human(
    raw_perception: &[PerceivedObject],
    values: &HumanValues,
) -> Vec<PerceivedObject> {
    raw_perception.iter()
        .filter(|obj| {
            // High beauty value → notice aesthetic properties
            if values.beauty > 0.5 {
                if obj.has_property("aesthetic") { return true; }
            }
            // High honor value → notice social status
            if values.honor > 0.5 {
                if obj.has_property("social_status") { return true; }
            }
            // Always notice threats
            obj.has_property("threat")
        })
        .cloned()
        .collect()
}
```

## Critical Implementation Details

### Task Execution and Need Satisfaction

Actions satisfy needs at **0.05x per tick**, not the nominal amount:

```rust
// ActionId::Eat says it satisfies food by 0.5
// But actual satisfaction per tick is: 0.5 × 0.05 = 0.025

// Over a 20-tick action duration:
// Total satisfaction: 0.025 × 20 = 0.5
```

This creates meaningful time investment - entities can't instantly satisfy needs.

### Task Completion Rules

| Duration | Progress Rate | Behavior |
|----------|---------------|----------|
| 0 (continuous) | 0.1/tick | **Never completes** - cancelled/replaced only |
| 1-60 (quick) | 0.05/tick | Completes in ~20 ticks |
| >60 (long) | 0.02/tick | Completes in ~50 ticks |

Continuous actions (IdleWander, IdleObserve) run until interrupted by a higher-priority task.

### Configuration

All magic numbers are documented in `core::config::SimulationConfig`.

## Testing

```bash
# Run simulation tests
cargo test --lib simulation::

# Key tests
cargo test test_needs_decay
cargo test test_different_values_different_behavior
```

### Test Coverage
- `test_needs_decay` - Verify needs increase over ticks
- `test_different_values_different_behavior` - Different values produce different actions
