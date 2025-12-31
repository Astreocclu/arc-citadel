# Arc Citadel Documentation System - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create comprehensive, agent-friendly documentation that enables AI agents (Claude, etc.) to autonomously understand and modify the Arc Citadel codebase.

**Output:**
- `CLAUDE.md` at project root - central agent onboarding document
- `src/<module>/README.md` for each of 12 modules - co-located implementation guides
- Updated architecture understanding for agents

---

## Critical Design Principles

Before writing ANY documentation, internalize these rules:

1. **Agent-First** - Documentation is primarily for AI agents, not humans
2. **Copy-Paste Ready** - Code examples must be valid Rust that compiles
3. **Constraints Over Features** - Emphasize what NOT to do more than what to do
4. **Link to Source** - Reference exact file paths with line numbers
5. **Test the Docs** - Every pattern shown must match actual codebase

---

## Task 1: Create CLAUDE.md Foundation

**File:** `CLAUDE.md` (project root)

**Step 1: Create the file with core structure**

```markdown
# Arc Citadel - AI Agent Onboarding

> Deep simulation strategy game with emergent entity behavior. Natural language commands parsed by LLM, but entity behavior emerges from values/needs/thoughts - never controlled by LLM.

## Quick Start for Agents

```bash
# Build
cargo build

# Run with simulation
cargo run

# Run tests
cargo test

# Run specific module tests
cargo test --lib entity::
```

## Absolute Constraints (Non-Negotiable)

These rules CANNOT be violated under any circumstances:

### 1. NO Percentage Modifiers
```rust
// FORBIDDEN - percentage bonuses
fn damage(base: f32, bonus_percent: f32) -> f32 {
    base * (1.0 + bonus_percent)  // NEVER DO THIS
}

// CORRECT - property interaction
fn damage(strength: f32, weapon_weight: f32, fatigue: f32) -> f32 {
    strength * weapon_weight * (1.0 - fatigue * 0.5)  // Properties interact
}
```

### 2. Species Values Are Different Types
```rust
// FORBIDDEN - generic values
struct Values {
    species: String,
    values: HashMap<String, f32>,  // NEVER DO THIS
}

// CORRECT - type-safe species values
struct HumanValues { honor: f32, beauty: f32, comfort: f32, ... }
struct DwarfValues { tradition: f32, craftsmanship: f32, ... }
```

### 3. LLM Parses Commands Only
```rust
// FORBIDDEN - LLM controls behavior
async fn decide_action(llm: &LlmClient, entity: &Entity) -> Action {
    llm.complete("What should this entity do?").await  // NEVER DO THIS
}

// CORRECT - LLM parses player input only
async fn parse_command(llm: &LlmClient, player_input: &str) -> ParsedIntent {
    // LLM converts natural language to structured command
}
```

### 4. Emergence Over Scripting
```rust
// FORBIDDEN - hardcoded behavior
if entity.values.safety > 0.8 {
    return Action::Flee;  // NEVER hardcode species behavior
}

// CORRECT - values influence thought generation, thoughts influence action selection
// Behavior emerges from the interaction of values + needs + perception + context
```

## Architecture Overview

```
                    ┌─────────────────────────────────────────────────────────┐
                    │                    SIMULATION TICK                       │
                    └─────────────────────────────────────────────────────────┘
                                              │
                    ┌─────────────────────────┼─────────────────────────┐
                    │                         │                         │
                    ▼                         ▼                         ▼
            ┌───────────────┐        ┌───────────────┐        ┌───────────────┐
            │  Perception   │        │   Thought     │        │    Action     │
            │    System     │───────▶│  Generation   │───────▶│   Selection   │
            └───────────────┘        └───────────────┘        └───────────────┘
                    │                         │                         │
                    │                         │                         │
                    ▼                         ▼                         ▼
            ┌───────────────┐        ┌───────────────┐        ┌───────────────┐
            │ SparseHashGrid│        │ ThoughtBuffer │        │  TaskQueue    │
            │ (spatial)     │        │ (entity)      │        │  (entity)     │
            └───────────────┘        └───────────────┘        └───────────────┘
```

## Module Map

| Module | Purpose | Key Files |
|--------|---------|-----------|
| `core/` | Types, errors, config | `types.rs`, `error.rs` |
| `ecs/` | Entity Component System | `world.rs` |
| `spatial/` | Spatial queries | `sparse_hash.rs`, `grid.rs` |
| `entity/` | Entity components | `needs.rs`, `thoughts.rs`, `tasks.rs` |
| `genetics/` | Genome/phenotype | `genome.rs`, `values.rs` |
| `simulation/` | Core loop | `tick.rs`, `perception.rs`, `action_select.rs` |
| `actions/` | Action catalog | `catalog.rs` |
| `combat/` | Combat resolution | `resolution.rs`, `wounds.rs` |
| `llm/` | LLM integration | `client.rs`, `parser.rs`, `context.rs` |
| `campaign/` | Strategic map | `map.rs`, `location.rs` |
| `battle/` | Tactical combat | `battle_map.rs`, `execution.rs` |
| `ui/` | Terminal interface | `terminal.rs`, `display.rs` |

## Data Flow

### Player Command Flow
```
Player Input → llm/parser.rs → ParsedIntent → actions/catalog.rs → Task → entity/tasks.rs
```

### Simulation Tick Flow
```
tick.rs → perception.rs → thought_gen.rs → action_select.rs → action_execute.rs → world.rs
```

### Entity Decision Flow
```
Perception → filtered by Values → Thoughts → weighted by Needs → Action → execution
```

## Critical File Paths

These are the most important files for understanding the system:

- **Entry Point:** `src/main.rs:1` - Game loop and initialization
- **World State:** `src/ecs/world.rs:1` - All entities stored here
- **Simulation Loop:** `src/simulation/tick.rs:1` - Orchestrates systems
- **Action Selection:** `src/simulation/action_select.rs:1` - Core decision logic
- **Human Archetype:** `src/entity/species/human.rs:1` - SoA data layout example
- **LLM Boundary:** `src/llm/parser.rs:1` - Where LLM interaction happens

## Implementation Plan Reference

See `docs/plans/2025-12-31-arc-citadel-mvp.md` for detailed implementation tasks.

## Testing Commands

```bash
# All tests
cargo test

# Specific module
cargo test --lib simulation::

# With output
cargo test -- --nocapture

# Single test
cargo test test_needs_decay
```

## Common Tasks

### Adding a New Action

1. Add variant to `ActionId` enum in `src/actions/catalog.rs`
2. Implement `satisfies_needs()` mapping
3. Add execution logic in relevant action file
4. Update `src/simulation/action_execute.rs` if needed

### Adding a New Species

1. Create `src/entity/species/<species>.rs`
2. Define `<Species>Values` struct with unique value vocabulary
3. Create `<Species>Archetype` with SoA layout
4. Update `src/ecs/world.rs` to include new archetype
5. Add perception filter in `src/simulation/perception.rs`

### Modifying Entity Behavior

**DO NOT** add hardcoded behavior. Instead:
1. Modify values in `src/entity/species/<species>.rs`
2. Adjust perception filtering in `src/simulation/perception.rs`
3. Tune action weights in `src/simulation/action_select.rs`

## Anti-Patterns to Avoid

| Anti-Pattern | Why It's Wrong | Correct Approach |
|--------------|----------------|------------------|
| `if species == Human { ... }` | Hardcoded behavior | Use trait dispatch on species type |
| `base * (1.0 + bonus)` | Percentage modifier | Property interaction |
| `llm.decide_entity_action()` | LLM controls behavior | LLM only parses commands |
| `HashMap<String, f32>` for values | Type-unsafe | Species-specific value structs |
| `Arc<Mutex<World>>` | Shared mutable state | Pass `&mut World` through systems |

## Performance Notes

- Spatial queries use `SparseHashGrid` with 10.0 cell size
- Entity iteration uses Structure-of-Arrays (SoA) layout
- Thoughts decay and are pruned to max 20 per entity
- LLM calls are async via tokio runtime
```

**Step 2: Verify the file was created**

Run: `cat CLAUDE.md | head -50`
Expected: Shows the header and quick start sections

**Step 3: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add CLAUDE.md for AI agent onboarding"
```

---

## Task 2: Create simulation/ README

**File:** `src/simulation/README.md`

**Step 1: Create the simulation module documentation**

```markdown
# Simulation Module

> The beating heart of Arc Citadel. Orchestrates the perception → thought → action → execution loop.

## Files

| File | Purpose | Lines |
|------|---------|-------|
| `mod.rs` | Module exports | ~10 |
| `perception.rs` | What entities notice | ~100 |
| `thought_gen.rs` | Generate thoughts from perception | stub |
| `action_select.rs` | Choose action from thoughts/needs | ~100 |
| `action_execute.rs` | Execute chosen actions | stub |
| `tick.rs` | Orchestrate all systems | ~100 |

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
    spatial/         entity/          ecs/
    (queries)        (needs,          (world)
                     thoughts)
```

## System Execution Order

The `tick.rs` file orchestrates systems in this order:

1. **update_needs** - Decay needs over time
2. **run_perception** - Build spatial grid, query neighbors
3. **generate_thoughts** - Create thoughts from perceptions
4. **decay_thoughts** - Remove faded thoughts
5. **select_actions** - Choose actions for idle entities
6. **execute_tasks** - Progress current tasks
7. **world.tick()** - Increment tick counter

## Key Data Structures

### Perception (perception.rs:10)

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

### SelectionContext (action_select.rs:12)

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

## Critical Functions

### perception_system (perception.rs)

```rust
pub fn perception_system(
    spatial_grid: &SparseHashGrid,
    positions: &[Vec2],
    entity_ids: &[EntityId],
    perception_range: f32,
) -> Vec<Perception>
```

Builds `Perception` for each entity by:
1. Querying `SparseHashGrid` for neighbors
2. Filtering by perception range
3. Setting relationship type (currently defaults to Unknown)

### select_action_human (action_select.rs)

```rust
pub fn select_action_human(ctx: &SelectionContext) -> Option<Task>
```

Selection priority:
1. **Critical needs** (safety > 0.8, food > 0.8, rest > 0.8) → immediate response
2. **Value impulses** (strong thought + matching value) → value-driven action
3. **Moderate needs** (most pressing need > 0.6) → address need
4. **Idle** → wander, observe, or socialize based on values

### run_simulation_tick (tick.rs)

```rust
pub fn run_simulation_tick(world: &mut World) {
    update_needs(world);
    let perceptions = run_perception(world);
    generate_thoughts(world, &perceptions);
    decay_thoughts(world);
    select_actions(world);
    execute_tasks(world);
    world.tick();
}
```

## Integration Points

### With entity/ module
- Reads: `needs`, `thoughts`, `values`, `body_states`, `task_queues`
- Writes: `needs` (satisfaction), `thoughts` (add/decay), `task_queues` (push tasks)

### With spatial/ module
- Uses `SparseHashGrid` for neighbor queries
- Rebuilds grid each tick from entity positions

### With ecs/ module
- Receives `&mut World` for all operations
- Iterates via `world.humans.iter_living()`

## Constraints

### DO
```rust
// Use context structs for action selection
let ctx = SelectionContext {
    body: &world.humans.body_states[i],
    needs: &world.humans.needs[i],
    // ...
};
if let Some(task) = select_action_human(&ctx) { ... }
```

### DON'T
```rust
// Don't hardcode species behavior
if is_human && safety > 0.8 {
    return Action::Flee;  // WRONG - emergent, not hardcoded
}

// Don't skip perception for action selection
let action = select_without_perception(entity);  // WRONG - perception filters first
```

## Extension Points

### Adding New Perception Types
1. Add variant to `PerceivedObject` or create new struct
2. Update `perception_system` to detect new type
3. Update `generate_thoughts` to react to new perception

### Adding New Action Selection Logic
1. Add new function in `action_select.rs`
2. Call from `select_actions` in `tick.rs`
3. Ensure it respects critical need priority

## Testing

```bash
# Run simulation tests
cargo test --lib simulation::

# Test specific function
cargo test test_needs_decay
```

### Key Tests in tests/emergence_tests.rs
- `test_needs_decay` - Needs increase over ticks
- `test_different_values_different_behavior` - Values affect action selection
```

**Step 2: Commit**

```bash
git add src/simulation/README.md
git commit -m "docs(simulation): add module README"
```

---

## Task 3: Create ecs/ README

**File:** `src/ecs/README.md`

**Step 1: Create the ECS module documentation**

```markdown
# ECS Module

> Custom Entity Component System with Structure-of-Arrays (SoA) data layout. Lightweight, no external ECS crate.

## Files

| File | Purpose |
|------|---------|
| `mod.rs` | Module exports |
| `world.rs` | World struct, entity management |

## Design Philosophy

**Why custom ECS instead of bevy_ecs/specs/legion?**
1. Species-specific archetypes need SoA layout for cache efficiency
2. Simple iteration patterns without complex queries
3. Direct field access without runtime type lookups
4. Full control over entity ID generation

## Core Types

### World (world.rs)

```rust
pub struct World {
    pub current_tick: u64,
    entity_registry: AHashMap<EntityId, (Species, usize)>,
    pub humans: HumanArchetype,
    next_indices: AHashMap<Species, usize>,
}
```

**Fields:**
- `current_tick` - Simulation time counter
- `entity_registry` - Maps EntityId → (Species, index in archetype)
- `humans` - SoA storage for all human entities
- `next_indices` - Next available index per species

### Key Methods

```rust
impl World {
    // Create new world
    pub fn new() -> Self;

    // Spawn human entity, returns its ID
    pub fn spawn_human(&mut self, name: String) -> EntityId;

    // Get (Species, index) for entity
    pub fn get_entity_info(&self, entity_id: EntityId) -> Option<(Species, usize)>;

    // Total entity count
    pub fn entity_count(&self) -> usize;

    // Advance tick counter
    pub fn tick(&mut self);

    // Iterator over human entity IDs
    pub fn human_entities(&self) -> impl Iterator<Item = EntityId> + '_;
}
```

## Structure of Arrays (SoA) Pattern

### Why SoA?

**Array of Structs (AoS) - BAD for cache:**
```rust
struct Entity { position: Vec2, velocity: Vec2, health: f32 }
let entities: Vec<Entity>;
// Iterating positions loads velocity and health too - cache waste
```

**Structure of Arrays (SoA) - GOOD for cache:**
```rust
struct Archetype {
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    healths: Vec<f32>,
}
// Iterating positions loads only positions - cache efficient
```

### HumanArchetype Example (entity/species/human.rs)

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

## Common Operations

### Spawning an Entity

```rust
let mut world = World::new();
let entity_id = world.spawn_human("Marcus".into());
// entity_id now valid, entity at index 0 in humans archetype
```

### Iterating Living Entities

```rust
for i in world.humans.iter_living() {
    let name = &world.humans.names[i];
    let needs = &world.humans.needs[i];
    // Process entity at index i
}
```

### Looking Up Entity by ID

```rust
if let Some((species, index)) = world.get_entity_info(entity_id) {
    match species {
        Species::Human => {
            let name = &world.humans.names[index];
        }
        _ => {}
    }
}
```

### Modifying Entity Data

```rust
let idx = world.humans.index_of(entity_id).unwrap();
world.humans.needs[idx].satisfy(NeedType::Food, 0.5);
world.humans.thoughts[idx].add(thought);
```

## Constraints

### DO
```rust
// Access components by index
let idx = world.humans.index_of(id)?;
let pos = world.humans.positions[idx];

// Iterate with iter_living()
for i in world.humans.iter_living() { ... }

// Use species-specific archetypes
world.humans.spawn(...);  // Human
world.dwarves.spawn(...); // Dwarf (future)
```

### DON'T
```rust
// Don't use generic component storage
world.get_component::<Position>(entity);  // WRONG - no runtime type lookup

// Don't iterate dead entities
for i in 0..world.humans.ids.len() { ... }  // WRONG - includes dead

// Don't mix species in one archetype
world.entities.push(human_or_dwarf);  // WRONG - separate archetypes
```

## Future: Adding New Species

When adding dwarves/elves:

1. Create `src/entity/species/dwarf.rs`:
```rust
pub struct DwarfValues { tradition: f32, craftsmanship: f32, ... }
pub struct DwarfArchetype { ids: Vec<EntityId>, values: Vec<DwarfValues>, ... }
```

2. Add to World:
```rust
pub struct World {
    pub humans: HumanArchetype,
    pub dwarves: DwarfArchetype,  // ADD
    // ...
}
```

3. Update `spawn_dwarf`:
```rust
pub fn spawn_dwarf(&mut self, name: String) -> EntityId { ... }
```

## Testing

```bash
cargo test --lib ecs::
```

### Key Tests
- `test_world_creation` - World initializes empty
- `test_spawn_human` - Entity count increases, ID valid
```

**Step 2: Commit**

```bash
git add src/ecs/README.md
git commit -m "docs(ecs): add module README"
```

---

## Task 4: Create entity/ README

**File:** `src/entity/README.md`

**Step 1: Create the entity module documentation**

```markdown
# Entity Module

> Components that make up living entities: identity, body, needs, thoughts, tasks, relationships.

## Files

| File | Purpose | Status |
|------|---------|--------|
| `mod.rs` | Module exports | Complete |
| `identity.rs` | Name, biography | Stub |
| `body.rs` | Health, fatigue, wounds | Complete |
| `needs.rs` | Universal needs system | Complete |
| `thoughts.rs` | Thought generation/decay | Complete |
| `tasks.rs` | Task queue management | Complete |
| `relationships.rs` | Entity relationships | Stub |
| `species/mod.rs` | Species exports | Complete |
| `species/human.rs` | Human archetype + values | Complete |

## Core Components

### Needs (needs.rs)

Universal needs shared by ALL species:

```rust
pub struct Needs {
    pub rest: f32,    // 0.0 = rested, 1.0 = exhausted
    pub food: f32,    // 0.0 = fed, 1.0 = starving
    pub safety: f32,  // 0.0 = safe, 1.0 = terrified
    pub social: f32,  // 0.0 = fulfilled, 1.0 = lonely
    pub purpose: f32, // 0.0 = purposeful, 1.0 = aimless
}
```

**Key Methods:**
- `most_pressing() -> (NeedType, f32)` - Highest need
- `has_critical() -> Option<NeedType>` - Need > 0.8
- `decay(dt, is_active)` - Increase needs over time
- `satisfy(need, amount)` - Decrease a need

### BodyState (body.rs)

Physical condition:

```rust
pub struct BodyState {
    pub fatigue: f32,       // 0.0-1.0
    pub hunger: f32,        // 0.0-1.0
    pub pain: f32,          // 0.0-1.0
    pub overall_health: f32, // Computed from wounds
}
```

**Key Methods:**
- `can_act() -> bool` - health > 0.1, fatigue < 0.95, pain < 0.9
- `can_move() -> bool` - can_act() AND fatigue < 0.9
- `add_fatigue(amount)` / `recover_fatigue(amount)`

### Thoughts (thoughts.rs)

Reaction to perceived events:

```rust
pub struct Thought {
    pub valence: Valence,           // Positive/Negative
    pub intensity: f32,             // 0.0-1.0
    pub concept_category: String,   // e.g., "fear", "curiosity"
    pub cause_description: String,  // What caused it
    pub cause_type: CauseType,      // Object/Entity/Action/Need/Event
    pub cause_entity: Option<EntityId>,
    pub created_tick: u64,
    pub decay_rate: f32,
}
```

**ThoughtBuffer** holds up to 20 thoughts, auto-evicts weakest.

### Tasks (tasks.rs)

Action queue:

```rust
pub struct Task {
    pub action: ActionId,
    pub target_position: Option<Vec2>,
    pub target_entity: Option<EntityId>,
    pub priority: TaskPriority,  // Critical/High/Normal/Low
    pub created_tick: Tick,
    pub progress: f32,           // 0.0-1.0
    pub source: TaskSource,      // PlayerCommand/Autonomous/Reaction
}
```

**TaskQueue** maintains current task + priority queue.

### HumanValues (species/human.rs)

Human-specific value vocabulary:

```rust
pub struct HumanValues {
    pub honor: f32,     // Social standing, keeping word
    pub beauty: f32,    // Aesthetic appreciation
    pub comfort: f32,   // Physical ease
    pub ambition: f32,  // Desire for advancement
    pub loyalty: f32,   // Attachment to group
    pub love: f32,      // Attachment to individuals
    pub justice: f32,   // Fairness
    pub curiosity: f32, // Desire to know
    pub safety: f32,    // Self-preservation
    pub piety: f32,     // Religious devotion
}
```

## Species-Specific vs Universal

| Component | Scope | Why |
|-----------|-------|-----|
| Needs | Universal | All species need food/rest/safety |
| BodyState | Universal | All species have bodies |
| Thoughts | Universal | All species think |
| Tasks | Universal | All species perform actions |
| **Values** | **Species-specific** | Humans value honor, dwarves value tradition |

## Constraints

### DO
```rust
// Use universal needs for all species
entity.needs.satisfy(NeedType::Food, 0.5);

// Use species-specific values
let (dominant, strength) = human_values.dominant();

// Let thoughts decay naturally
thought_buffer.decay_all();
```

### DON'T
```rust
// Don't use generic values
let courage = entity.values.get("courage");  // WRONG

// Don't skip the thought system
entity.action = Action::Flee;  // WRONG - thoughts mediate perception→action

// Don't hardcode need thresholds outside action_select
if entity.needs.food > 0.8 { eat(); }  // WRONG - selection logic belongs in simulation/
```

## Data Flow

```
Perception
    │
    ▼ (filtered by species values)
Thoughts
    │
    ▼ (weighted by needs + values)
Task Selection
    │
    ▼
TaskQueue
    │
    ▼
Action Execution
```

## Testing

```bash
cargo test --lib entity::
```
```

**Step 2: Commit**

```bash
git add src/entity/README.md
git commit -m "docs(entity): add module README"
```

---

## Task 5: Create llm/ README

**File:** `src/llm/README.md`

**Step 1: Create the LLM module documentation**

```markdown
# LLM Module

> Natural language command parsing. **LLM ONLY parses player commands** - it NEVER controls entity behavior.

## Files

| File | Purpose | Status |
|------|---------|--------|
| `mod.rs` | Module exports | Complete |
| `client.rs` | Async HTTP client | Complete |
| `parser.rs` | Parse commands to intents | Complete |
| `context.rs` | Build game context for prompts | Complete |
| `species_interpret.rs` | Species-specific prompting | Stub |
| `prompts.rs` | Prompt templates | Stub |

## THE BOUNDARY

```
┌─────────────────────────────────────────────────────────────────────┐
│                        LLM BOUNDARY                                  │
│                                                                      │
│   Player Input ──▶ LLM ──▶ ParsedIntent ──▶ Game Systems            │
│                                                                      │
│   ✓ "build a wall"        → BUILD intent                            │
│   ✓ "have Marcus guard"   → ASSIGN intent                           │
│   ✓ "make it beautiful"   → CRAFT intent + ambiguous_concepts       │
│                                                                      │
│   ✗ Entity behavior                                                  │
│   ✗ NPC decision-making                                              │
│   ✗ Autonomous thought generation                                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## LlmClient (client.rs)

Async HTTP client for Claude API:

```rust
pub struct LlmClient {
    client: Client,      // reqwest
    api_key: String,
    api_url: String,
    model: String,
}

impl LlmClient {
    // From environment variables
    pub fn from_env() -> Result<Self>;

    // Send completion request
    pub async fn complete(&self, system: &str, user: &str) -> Result<String>;
}
```

**Environment Variables:**
- `LLM_API_KEY` (required)
- `LLM_API_URL` (default: https://api.anthropic.com/v1/messages)
- `LLM_MODEL` (default: claude-3-haiku-20240307)

## ParsedIntent (parser.rs)

Structured output from command parsing:

```rust
pub struct ParsedIntent {
    pub action: IntentAction,        // BUILD, CRAFT, ASSIGN, COMBAT, etc.
    pub target: Option<String>,      // What to build/craft/attack
    pub location: Option<String>,    // Where
    pub subjects: Option<Vec<String>>, // Who should do it
    pub priority: IntentPriority,    // CRITICAL/HIGH/NORMAL/LOW
    pub ambiguous_concepts: Vec<String>, // Terms needing species interpretation
    pub confidence: f32,             // 0.0-1.0
}
```

**IntentAction variants:**
- `Build`, `Craft`, `Assign`, `Combat`, `Gather`, `Move`, `Query`, `Social`, `Rest`, `Unknown`

## GameContext (context.rs)

Assembles world state for LLM prompts:

```rust
pub struct GameContext {
    pub location_name: String,
    pub entity_count: usize,
    pub available_resources: Vec<String>,
    pub recent_events: Vec<String>,
    pub named_entities: Vec<NamedEntity>,
    pub threats: Vec<String>,
}

impl GameContext {
    // Build from world state
    pub fn from_world(world: &World) -> Self;

    // Format for prompt
    pub fn summary(&self) -> String;
}
```

## Usage Pattern

```rust
// In main.rs
let llm_client = LlmClient::from_env().ok();

if let Some(ref client) = llm_client {
    let context = GameContext::from_world(&world);

    match parse_command(client, player_input, &context).await {
        Ok(intent) => {
            // Convert intent to game action
            match intent.action {
                IntentAction::Build => { /* queue build task */ }
                IntentAction::Assign => { /* assign entity */ }
                // ...
            }
        }
        Err(e) => println!("Could not parse: {}", e),
    }
}
```

## Constraints

### DO
```rust
// Use LLM only for player commands
let intent = parse_command(client, player_input, context).await?;

// Handle ambiguous concepts
if !intent.ambiguous_concepts.is_empty() {
    // May need species-specific interpretation
}

// Graceful fallback when LLM unavailable
if llm_client.is_none() {
    println!("Commands: tick, spawn <name>, quit");
}
```

### DON'T
```rust
// NEVER use LLM for entity decisions
async fn entity_think(llm: &LlmClient, entity: &Entity) -> Thought {
    llm.complete("What is this entity thinking?").await  // FORBIDDEN
}

// NEVER bypass the ParsedIntent structure
let raw_action = llm.complete("What action?").await;
execute_raw(raw_action);  // FORBIDDEN - must parse to structured intent

// NEVER trust LLM output without validation
let intent: ParsedIntent = serde_json::from_str(&llm_response)?;
// Always validate intent.action is a known variant
```

## Error Handling

```rust
// LLM errors should not crash the game
match client.complete(system, user).await {
    Ok(response) => parse_response(response),
    Err(ArcError::LlmError(e)) => {
        tracing::warn!("LLM error: {}", e);
        // Fall back to simple command parsing or ignore
    }
}
```

## Testing

```bash
# LLM tests require API key
LLM_API_KEY=sk-... cargo test --lib llm::
```

**Mock client for unit tests:**
```rust
// TODO: Implement MockLlmClient for testing
```
```

**Step 2: Commit**

```bash
git add src/llm/README.md
git commit -m "docs(llm): add module README with boundary rules"
```

---

## Task 6: Create core/ README

**File:** `src/core/README.md`

**Step 1: Create the core module documentation**

```markdown
# Core Module

> Foundation types, errors, and configuration shared across all modules.

## Files

| File | Purpose |
|------|---------|
| `mod.rs` | Module exports |
| `types.rs` | Core type definitions |
| `error.rs` | Error types and Result alias |
| `config.rs` | Configuration (stub) |

## Core Types (types.rs)

### EntityId

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub Uuid);

impl EntityId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}
```

### Vec2

```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self;
    pub fn distance(&self, other: &Self) -> f32;
    pub fn length(&self) -> f32;
    pub fn normalize(&self) -> Self;
}
```

Implements `Add`, `Sub`, `Mul<f32>`.

### Other Types

```rust
pub type Tick = u64;

pub struct LocationId(pub u32);
pub struct FlowFieldId(pub u32);

pub enum Species {
    Human,
    Dwarf,
    Elf,
}
```

## Error Handling (error.rs)

```rust
#[derive(Error, Debug)]
pub enum ArcError {
    #[error("Entity not found: {0:?}")]
    EntityNotFound(EntityId),

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

## Usage

```rust
use crate::core::types::{EntityId, Vec2, Tick};
use crate::core::error::{ArcError, Result};

fn find_entity(world: &World, id: EntityId) -> Result<&Entity> {
    world.get(id).ok_or(ArcError::EntityNotFound(id))
}
```

## Constraints

### DO
```rust
// Use EntityId for all entity references
let id: EntityId = EntityId::new();

// Use Result<T> for fallible operations
fn load_data() -> Result<Config> { ... }

// Use Vec2 for positions
let pos = Vec2::new(10.0, 20.0);
```

### DON'T
```rust
// Don't use raw UUIDs
let id: Uuid = Uuid::new_v4();  // WRONG - wrap in EntityId

// Don't use panic for errors
let entity = world.get(id).unwrap();  // WRONG - return Result

// Don't use tuples for positions
let pos: (f32, f32) = (10.0, 20.0);  // WRONG - use Vec2
```
```

**Step 2: Commit**

```bash
git add src/core/README.md
git commit -m "docs(core): add module README"
```

---

## Task 7: Create spatial/ README

**File:** `src/spatial/README.md`

**Step 1: Create the spatial module documentation**

```markdown
# Spatial Module

> Spatial data structures for efficient neighbor queries and pathfinding.

## Files

| File | Purpose | Status |
|------|---------|--------|
| `mod.rs` | Module exports | Complete |
| `grid.rs` | Generic 2D grid | Complete |
| `sparse_hash.rs` | Sparse hash grid for entities | Complete |
| `flow_field.rs` | Flow field pathfinding | Stub |

## SparseHashGrid (sparse_hash.rs)

Primary spatial index for entity queries:

```rust
pub struct SparseHashGrid {
    cell_size: f32,
    cells: AHashMap<(i32, i32), Vec<EntityId>>,
}
```

### Methods

```rust
impl SparseHashGrid {
    pub fn new(cell_size: f32) -> Self;
    pub fn clear(&mut self);
    pub fn insert(&mut self, entity: EntityId, pos: Vec2);
    pub fn remove(&mut self, entity: EntityId, pos: Vec2);
    pub fn query_neighbors(&self, pos: Vec2) -> impl Iterator<Item = EntityId>;
    pub fn query_radius(&self, center: Vec2, radius: f32, positions: &[Vec2]) -> Vec<EntityId>;
    pub fn rebuild<'a>(&mut self, entities: impl Iterator<Item = (EntityId, Vec2)>);
}
```

### Usage in Simulation

```rust
// In tick.rs - rebuild grid each tick
let mut grid = SparseHashGrid::new(10.0);
grid.rebuild(ids.iter().cloned().zip(positions.iter().cloned()));

// Query neighbors for perception
let nearby = grid.query_neighbors(observer_pos)
    .filter(|&e| e != observer_id)
    .collect::<Vec<_>>();
```

## Grid<T> (grid.rs)

Generic 2D grid for terrain, flow fields, etc:

```rust
pub struct Grid<T: Clone + Default> {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub origin: Vec2,
    data: Vec<T>,
}
```

### Methods

```rust
impl<T: Clone + Default> Grid<T> {
    pub fn new(width: usize, height: usize, cell_size: f32, origin: Vec2) -> Self;
    pub fn get(&self, x: usize, y: usize) -> Option<&T>;
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T>;
    pub fn set(&mut self, x: usize, y: usize, value: T);
    pub fn world_to_cell(&self, pos: Vec2) -> (usize, usize);
    pub fn sample(&self, pos: Vec2) -> Option<&T>;
    pub fn cell_center(&self, x: usize, y: usize) -> Vec2;
}
```

## Performance Notes

- `SparseHashGrid` uses `ahash` for fast hashing
- Cell size of 10.0 works well for typical perception ranges (50.0)
- Grid rebuilt each tick - future optimization: incremental updates
- `query_neighbors` returns 3x3 cell neighborhood (O(9) cells)

## Constraints

### DO
```rust
// Use SparseHashGrid for entity spatial queries
let grid = SparseHashGrid::new(10.0);

// Rebuild grid when positions change significantly
grid.rebuild(entities_with_positions);

// Use Grid<T> for static terrain data
let terrain: Grid<TerrainType> = Grid::new(100, 100, 1.0, Vec2::default());
```

### DON'T
```rust
// Don't query without rebuilding after movement
grid.query_neighbors(pos);  // May be stale if entities moved

// Don't use tiny cell sizes
let grid = SparseHashGrid::new(0.1);  // WRONG - too many cells

// Don't iterate all entities for spatial queries
for entity in all_entities {
    if entity.pos.distance(&target) < range { ... }  // WRONG - O(n)
}
```
```

**Step 2: Commit**

```bash
git add src/spatial/README.md
git commit -m "docs(spatial): add module README"
```

---

## Task 8: Create actions/ README

**File:** `src/actions/README.md`

**Step 1: Create the actions module documentation**

```markdown
# Actions Module

> Catalog of all possible actions and their properties.

## Files

| File | Purpose | Status |
|------|---------|--------|
| `mod.rs` | Module exports | Complete |
| `catalog.rs` | Action definitions | Complete |
| `movement.rs` | Movement actions | Stub |
| `survival.rs` | Survival actions | Stub |
| `work.rs` | Work actions | Stub |
| `social.rs` | Social actions | Stub |
| `combat.rs` | Combat actions | Stub |

## ActionId (catalog.rs)

All available actions:

```rust
pub enum ActionId {
    // Movement
    MoveTo, Follow, Flee,

    // Survival
    Rest, Eat, SeekSafety,

    // Work
    Build, Craft, Gather, Repair,

    // Social
    TalkTo, Help, Trade,

    // Combat
    Attack, Defend, Charge, HoldPosition,

    // Idle
    IdleWander, IdleObserve,
}
```

## Action Properties

### Category

```rust
impl ActionId {
    pub fn category(&self) -> ActionCategory {
        match self {
            MoveTo | Follow | Flee => Movement,
            Rest | Eat | SeekSafety => Survival,
            Build | Craft | Gather | Repair => Work,
            TalkTo | Help | Trade => Social,
            Attack | Defend | Charge | HoldPosition => Combat,
            IdleWander | IdleObserve => Idle,
        }
    }
}
```

### Needs Satisfaction

```rust
impl ActionId {
    pub fn satisfies_needs(&self) -> Vec<(NeedType, f32)> {
        match self {
            Rest => vec![(NeedType::Rest, 0.3)],
            Eat => vec![(NeedType::Food, 0.5)],
            SeekSafety | Flee => vec![(NeedType::Safety, 0.3)],
            TalkTo | Help => vec![(NeedType::Social, 0.3)],
            Build | Craft | Gather => vec![(NeedType::Purpose, 0.3)],
            _ => vec![],
        }
    }
}
```

### Interruptibility

```rust
impl ActionId {
    pub fn is_interruptible(&self) -> bool {
        match self {
            Attack | Charge => false,  // Can't interrupt mid-swing
            _ => true,
        }
    }
}
```

### Duration

```rust
impl ActionId {
    pub fn base_duration(&self) -> u32 {
        match self {
            Attack | Defend => 1,      // Instant
            TalkTo => 60,              // 1 minute
            Eat => 30,
            Rest => 600,               // 10 minutes
            Craft => 1800,             // 30 minutes
            Build => 3600,             // 1 hour
            _ => 0,                    // Continuous
        }
    }
}
```

## Adding New Actions

1. Add variant to `ActionId` enum
2. Add to `category()` match
3. Add to `satisfies_needs()` if applicable
4. Add to `is_interruptible()` if non-interruptible
5. Add to `base_duration()` if timed
6. Implement execution logic in appropriate submodule

## Constraints

### DO
```rust
// Use ActionId enum for all actions
let action = ActionId::Build;

// Check interruptibility before interrupting
if current_action.is_interruptible() {
    task_queue.cancel_current();
}

// Use satisfies_needs() for need calculation
for (need, amount) in action.satisfies_needs() {
    needs.satisfy(need, amount);
}
```

### DON'T
```rust
// Don't use strings for actions
let action = "build";  // WRONG - use ActionId::Build

// Don't hardcode need satisfaction
if action == ActionId::Eat {
    needs.food -= 0.5;  // WRONG - use satisfies_needs()
}

// Don't skip duration checks
task.progress = 1.0;  // WRONG - check against base_duration()
```
```

**Step 2: Commit**

```bash
git add src/actions/README.md
git commit -m "docs(actions): add module README"
```

---

## Task 9: Create Stub READMEs for Remaining Modules

**Files:** Combat, campaign, battle, ui, genetics, data

**Step 1: Create combat/README.md**

```markdown
# Combat Module

> Combat resolution, weapons, armor, wounds, and morale. **Currently stub files.**

## Files

| File | Purpose | Status |
|------|---------|--------|
| `resolution.rs` | Combat resolution | Stub |
| `weapons.rs` | Weapon properties | Stub |
| `armor.rs` | Armor properties | Stub |
| `wounds.rs` | Wound system | Stub |
| `morale.rs` | Morale system | Stub |

## Planned Design

Combat emerges from property interactions:
- Weapon weight × strength → impact force
- Impact force vs armor coverage → penetration
- Penetration location → wound type
- Wound severity + pain tolerance → morale effect

**No percentage modifiers** - all calculations use property interaction.
```

**Step 2: Create campaign/README.md**

```markdown
# Campaign Module

> Strategic layer: campaign map, locations, routes, weather, supply. **Currently stub files.**

## Files

| File | Purpose | Status |
|------|---------|--------|
| `map.rs` | Campaign map | Stub |
| `location.rs` | Locations | Stub |
| `route.rs` | Routes between locations | Stub |
| `weather.rs` | Weather system | Stub |
| `supply.rs` | Supply logistics | Stub |

## Planned Design

Campaign layer manages:
- Region-based map with locations
- Travel time calculation between locations
- Weather effects on movement/combat
- Supply line logistics
```

**Step 3: Create battle/README.md**

```markdown
# Battle Module

> Tactical combat layer: battle maps, planning, execution. **Currently stub files.**

## Files

| File | Purpose | Status |
|------|---------|--------|
| `battle_map.rs` | Battle terrain | Stub |
| `planning.rs` | Battle planning | Stub |
| `execution.rs` | Battle execution | Stub |
| `courier.rs` | Order delay system | Stub |
| `resolution.rs` | Battle outcomes | Stub |

## Planned Design

Tactical battles with:
- Terrain-based battle maps
- Formation and positioning
- Courier-based order delays (no instant commands)
- Individual entity combat resolution
```

**Step 4: Create ui/README.md**

```markdown
# UI Module

> Terminal-based user interface. **Currently stub files.**

## Files

| File | Purpose | Status |
|------|---------|--------|
| `terminal.rs` | Terminal rendering | Stub |
| `input.rs` | Input handling | Stub |
| `display.rs` | Display components | Stub |

## Planned Design

Using `crossterm` and `ratatui` for:
- Entity status display
- Command input
- Map visualization
- Event log
```

**Step 5: Create genetics/README.md**

```markdown
# Genetics Module

> Genome, phenotype, personality, values. **Currently stub files.**

## Files

| File | Purpose | Status |
|------|---------|--------|
| `genome.rs` | Genetic data | Stub |
| `phenotype.rs` | Physical expression | Stub |
| `personality.rs` | Personality traits | Stub |
| `values.rs` | Value calculation | Stub |

## Planned Design

Genetics determine:
- Physical phenotype (height, strength, etc.)
- Personality traits (Big Five model)
- Initial value weights (species-specific)
```

**Step 6: Create data/README.md**

```markdown
# Data Module

> Asset loading and game data. **Currently stub files.**

## Files

| File | Purpose | Status |
|------|---------|--------|
| `mod.rs` | Module exports | Stub |

## Data Files

- `data/species/human.json` - Human species definition

## Planned Design

Load game data from JSON/TOML:
- Species definitions
- Item databases
- Name generators
```

**Step 7: Create all files and commit**

```bash
# Create all stub READMEs
git add src/combat/README.md src/campaign/README.md src/battle/README.md \
        src/ui/README.md src/genetics/README.md src/data/README.md
git commit -m "docs: add stub READMEs for remaining modules"
```

---

## Task 10: Final Verification

**Step 1: Verify all documentation files exist**

```bash
ls -la CLAUDE.md
ls -la src/*/README.md
```

Expected: All files present

**Step 2: Verify CLAUDE.md is readable**

```bash
cat CLAUDE.md | head -100
```

Expected: Clean markdown output

**Step 3: Run build to ensure no conflicts**

```bash
cargo build
cargo test
```

Expected: Build succeeds, tests pass

**Step 4: Final commit**

```bash
git add .
git commit -m "docs: complete documentation system implementation"
```

---

## Verification Checklist

- [ ] `CLAUDE.md` exists at project root with all sections
- [ ] `src/simulation/README.md` - complete with data flow
- [ ] `src/ecs/README.md` - complete with SoA explanation
- [ ] `src/entity/README.md` - complete with component docs
- [ ] `src/llm/README.md` - complete with boundary rules
- [ ] `src/core/README.md` - complete with type docs
- [ ] `src/spatial/README.md` - complete with usage patterns
- [ ] `src/actions/README.md` - complete with action catalog
- [ ] `src/combat/README.md` - stub
- [ ] `src/campaign/README.md` - stub
- [ ] `src/battle/README.md` - stub
- [ ] `src/ui/README.md` - stub
- [ ] `src/genetics/README.md` - stub
- [ ] `src/data/README.md` - stub
- [ ] Build passes
- [ ] Tests pass

---

## Summary

**Total Tasks:** 10
**Key Deliverables:**
- 1 central CLAUDE.md with architecture, constraints, anti-patterns
- 12 module READMEs (7 complete, 5 stubs for unimplemented modules)
- Consistent format across all documentation
- Agent-friendly with copy-paste code examples
- Emphasis on constraints and boundaries
