# Simulation Module

> The beating heart of Arc Citadel. Orchestrates the perception → thought → action → execution loop each tick.

## Module Structure

```
simulation/
├── mod.rs                  # Module exports
├── tick.rs                 # Orchestrate all systems each tick (4405 LOC)
├── action_select.rs        # Choose actions based on needs/values (6121 LOC)
├── perception.rs           # What entities notice in environment
├── thought_gen.rs          # Generate thoughts from perceptions (stub)
├── action_execute.rs       # Execute chosen actions (stub)
├── consumption.rs          # Resource consumption logic
├── expectation_formation.rs # Pattern learning from observations
├── housing.rs              # Housing assignment and capacity
├── population.rs           # Population dynamics
├── resource_zone.rs        # Resource zone management
├── rule_eval.rs            # Rule evaluation for actions
├── value_dynamics.rs       # Value changes over time
└── violation_detection.rs  # Detect behavioral violations (601 LOC)
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

The `tick.rs` file (4405 lines) orchestrates systems:

```rust
pub fn run_simulation_tick(world: &mut World) -> Vec<SimulationEvent> {
    update_needs(world);           // 1. Needs decay over time
    let perceptions = perception_system(world);  // 2. Build spatial index, query
    generate_thoughts(world, &perceptions);   // 3. React to perceptions
    decay_thoughts(world);         // 4. Fade old thoughts
    select_actions(world);         // 5. Choose actions for idle entities
    execute_tasks(world);          // 6. Progress current tasks
    // Combat resolution wired in at line ~2280
    world.tick();                  // 7. Advance time
}
```

## Key Components

### Action Selection (`action_select.rs` - 6121 LOC)

The largest file in the codebase. Implements action selection for all species.

**Key exports:**
```rust
pub use action_select::select_action_with_rules;
```

**Selection priority:**
1. **Critical needs** (> 0.65) trigger immediate response
2. **Value-driven impulses** from strong thoughts matching values
3. **Moderate needs** (> 0.5) addressed when idle
4. **Idle actions** based on personality (wander, observe, socialize)

### Perception System (`perception.rs`)

Determines what each entity notices based on spatial proximity and values.

### Expectation Formation (`expectation_formation.rs`)

Pattern learning from observations:
```rust
pub use expectation_formation::{
    infer_patterns_from_action,
    process_observations,
    record_observation,
};
```

### Violation Detection (`violation_detection.rs` - 601 LOC)

Detects when entity behavior violates expected patterns:
```rust
pub use violation_detection::{
    check_pattern_violation,
    check_violations,
    process_violations,
    ViolationType,
};
```

### Resource Zones (`resource_zone.rs`)

Manages resource availability:
```rust
pub use resource_zone::{ResourceType, ResourceZone};
```

### Value Dynamics (`value_dynamics.rs`)

Applies value changes over time:
```rust
pub use value_dynamics::{apply_event, apply_tick_dynamics};
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

### With `combat/` module
- `resolve_exchange()` called from tick.rs (line ~2280)
- Applies wounds and fatigue to combatants

## Critical Implementation Details

### Task Execution and Need Satisfaction

Actions satisfy needs at **0.05x per tick**, not the nominal amount:

```rust
// ActionId::Eat says it satisfies food by 0.5
// But actual satisfaction per tick is: 0.5 × 0.05 = 0.025

// Over a 20-tick action duration:
// Total satisfaction: 0.025 × 20 = 0.5
```

### Task Completion Rules

| Duration | Progress Rate | Behavior |
|----------|---------------|----------|
| 0 (continuous) | 0.1/tick | **Never completes** - cancelled/replaced only |
| 1-60 (quick) | 0.05/tick | Completes in ~20 ticks |
| >60 (long) | 0.02/tick | Completes in ~50 ticks |

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
