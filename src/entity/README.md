# Entity Module

> Components that define living entities: identity, body, needs, thoughts, tasks, relationships, and species-specific values.

## Module Structure

```
entity/
├── mod.rs              # Module exports
├── identity.rs         # Name, biography (stub)
├── body.rs             # Health, fatigue, wounds
├── needs.rs            # Universal needs system
├── thoughts.rs         # Thought generation and decay
├── tasks.rs            # Task queue management
├── relationships.rs    # Entity relationships (stub)
└── species/
    ├── mod.rs          # Species exports
    └── human.rs        # Human archetype and values
```

## Core Components

### Needs (`needs.rs`)

Universal needs shared by all species:

```rust
pub struct Needs {
    pub rest: f32,    // 0.0 = rested, 1.0 = exhausted
    pub food: f32,    // 0.0 = fed, 1.0 = starving
    pub safety: f32,  // 0.0 = safe, 1.0 = terrified
    pub social: f32,  // 0.0 = fulfilled, 1.0 = lonely
    pub purpose: f32, // 0.0 = purposeful, 1.0 = aimless
}
```

**Key methods:**
```rust
impl Needs {
    // Find the most pressing need
    pub fn most_pressing(&self) -> (NeedType, f32);

    // Check for critical needs (> 0.8)
    pub fn has_critical(&self) -> Option<NeedType>;

    // Increase needs over time
    pub fn decay(&mut self, dt: f32, is_active: bool);

    // Decrease a specific need
    pub fn satisfy(&mut self, need: NeedType, amount: f32);
}
```

### BodyState (`body.rs`)

Physical condition tracking:

```rust
pub struct BodyState {
    pub fatigue: f32,        // 0.0-1.0
    pub hunger: f32,         // 0.0-1.0
    pub pain: f32,           // 0.0-1.0
    pub overall_health: f32, // Computed from wounds
}

impl BodyState {
    // Can this entity take actions?
    pub fn can_act(&self) -> bool {
        self.overall_health > 0.1 &&
        self.fatigue < 0.95 &&
        self.pain < 0.9
    }

    // Can this entity move?
    pub fn can_move(&self) -> bool {
        self.can_act() && self.fatigue < 0.9
    }
}
```

### Thoughts (`thoughts.rs`)

Cognitive/emotional reactions to perceptions:

```rust
pub struct Thought {
    pub valence: Valence,           // Positive or Negative
    pub intensity: f32,             // 0.0-1.0
    pub concept_category: String,   // e.g., "fear", "curiosity", "joy"
    pub cause_description: String,  // What triggered this thought
    pub cause_type: CauseType,      // Object/Entity/Action/Need/Event
    pub cause_entity: Option<EntityId>,
    pub created_tick: u64,
    pub decay_rate: f32,
}

pub struct ThoughtBuffer {
    thoughts: Vec<Thought>,
    max_thoughts: usize,  // Default: 20
}
```

**ThoughtBuffer** automatically evicts the weakest thought when full.

### Tasks (`tasks.rs`)

Action queue for entities:

```rust
pub struct Task {
    pub action: ActionId,
    pub target_position: Option<Vec2>,
    pub target_entity: Option<EntityId>,
    pub priority: TaskPriority,  // Critical, High, Normal, Low
    pub created_tick: Tick,
    pub progress: f32,           // 0.0-1.0
    pub source: TaskSource,      // PlayerCommand, Autonomous, Reaction
}

pub struct TaskQueue {
    current: Option<Task>,
    queued: VecDeque<Task>,
}
```

### HumanValues (`species/human.rs`)

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
    pub curiosity: f32, // Desire to explore
    pub safety: f32,    // Self-preservation
    pub piety: f32,     // Religious devotion
}

impl HumanValues {
    // Find the dominant value
    pub fn dominant(&self) -> (&'static str, f32);
}
```

## Universal vs Species-Specific

| Component | Scope | Rationale |
|-----------|-------|-----------|
| Needs | Universal | All species need food, rest, safety |
| BodyState | Universal | All species have physical bodies |
| Thoughts | Universal | All species have cognitive reactions |
| Tasks | Universal | All species perform actions |
| **Values** | **Species-specific** | Different species value different concepts |

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
    │
    ▼
Need Satisfaction
```

## Best Practices

### Working with Needs
```rust
// Check for critical needs first
if let Some(critical) = needs.has_critical() {
    match critical {
        NeedType::Safety => { /* immediate safety response */ }
        NeedType::Food => { /* seek food */ }
        NeedType::Rest => { /* find safe place to rest */ }
        _ => {}
    }
}

// Find most pressing need for moderate responses
let (need_type, level) = needs.most_pressing();
if level > 0.6 {
    // Address this need
}

// Satisfy needs when actions complete
needs.satisfy(NeedType::Food, 0.5);
```

### Working with Thoughts
```rust
// Add a new thought
let thought = Thought::new(
    Valence::Positive,
    0.7,  // intensity
    "joy",
    "found a beautiful sunset",
    CauseType::Event,
    current_tick,
);
thought_buffer.add(thought);

// Query thoughts
if let Some(strongest) = thought_buffer.strongest() {
    println!("Strongest thought: {} ({})",
        strongest.concept_category,
        strongest.intensity);
}

// Decay thoughts each tick
thought_buffer.decay_all();
```

### Working with Tasks
```rust
// Create a task
let task = Task::new(ActionId::Build, TaskPriority::Normal, current_tick)
    .with_position(Vec2::new(10.0, 20.0))
    .from_player();

// Add to queue (respects priority)
task_queue.push(task);

// Check current task
if let Some(current) = task_queue.current() {
    println!("Working on: {:?}", current.action);
}

// Complete or cancel
task_queue.complete_current();
task_queue.cancel_current();
```

### Working with Values
```rust
// Find dominant value
let (name, strength) = human_values.dominant();
println!("This human values {} most ({:.0}%)", name, strength * 100.0);

// Use values to filter perception
if values.beauty > 0.5 {
    // This entity notices beautiful things
}

if values.curiosity > 0.7 {
    // This entity is drawn to explore
}
```

## Adding New Components

### 1. Create the Component
```rust
// src/entity/memory.rs
pub struct Memory {
    pub events: Vec<MemorizedEvent>,
    pub max_memories: usize,
}
```

### 2. Add to Archetype
```rust
// src/entity/species/human.rs
pub struct HumanArchetype {
    // ... existing fields
    pub memories: Vec<Memory>,  // Add parallel array
}

impl HumanArchetype {
    pub fn spawn(&mut self, ...) {
        // ... existing spawns
        self.memories.push(Memory::new());  // Initialize
    }
}
```

### 3. Use in Systems
```rust
// Access like any other component
let idx = world.humans.index_of(entity_id)?;
let memory = &mut world.humans.memories[idx];
memory.add_event(event);
```

## Critical Implementation Details

### Safety Need is Asymmetric

Unlike other needs that only **increase** over time, safety **decreases**:

```rust
// Other needs increase (0.0 → 1.0 over time)
self.rest += 0.001;   // Gets more tired
self.food += 0.0005;  // Gets hungrier

// Safety DECREASES (1.0 → 0.0 over time)
self.safety = (self.safety - 0.01).max(0.0);  // Calms down
```

This prevents entities from being permanently scared after a threat is gone.

### TaskPriority Uses Explicit Ordering

Priority is defined with explicit numeric values for safe comparison:

```rust
#[repr(u8)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}
```

Higher values = higher priority. The `TaskQueue::push` method relies on this ordering to insert tasks correctly.

### Need Decay Rates

Needs increase at different rates (per tick when active):

| Need | Rate | Time to Critical (~0.8) |
|------|------|------------------------|
| Rest | 0.001 | ~800 ticks |
| Food | 0.0005 | ~1600 ticks |
| Social | 0.0003 | ~2600 ticks |
| Purpose | 0.0002 | ~4000 ticks |

Active entities decay 1.5x faster for rest.

## Testing

```bash
cargo test --lib entity::
```

### Test Ideas
- Needs decay correctly over time
- Critical needs trigger at threshold
- ThoughtBuffer evicts weakest when full
- TaskQueue respects priority ordering
