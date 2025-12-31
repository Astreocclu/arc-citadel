# Actions Module

> Catalog of all possible actions and their properties.

## Module Structure

```
actions/
├── mod.rs       # Module exports
├── catalog.rs   # Action definitions and properties
├── movement.rs  # Movement actions (stub)
├── survival.rs  # Survival actions (stub)
├── work.rs      # Work actions (stub)
├── social.rs    # Social actions (stub)
└── combat.rs    # Combat actions (stub)
```

## ActionId (`catalog.rs`)

All available actions in the game:

```rust
pub enum ActionId {
    // Movement
    MoveTo,
    Follow,
    Flee,

    // Survival
    Rest,
    Eat,
    SeekSafety,

    // Work
    Build,
    Craft,
    Gather,
    Repair,

    // Social
    TalkTo,
    Help,
    Trade,

    // Combat
    Attack,
    Defend,
    Charge,
    HoldPosition,

    // Idle
    IdleWander,
    IdleObserve,
}
```

## Action Properties

### Categories

Actions are grouped by type:

```rust
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
            Self::MoveTo | Self::Follow | Self::Flee => ActionCategory::Movement,
            Self::Rest | Self::Eat | Self::SeekSafety => ActionCategory::Survival,
            Self::Build | Self::Craft | Self::Gather | Self::Repair => ActionCategory::Work,
            Self::TalkTo | Self::Help | Self::Trade => ActionCategory::Social,
            Self::Attack | Self::Defend | Self::Charge | Self::HoldPosition => ActionCategory::Combat,
            Self::IdleWander | Self::IdleObserve => ActionCategory::Idle,
        }
    }
}
```

### Need Satisfaction

Actions satisfy needs when performed:

```rust
impl ActionId {
    pub fn satisfies_needs(&self) -> Vec<(NeedType, f32)> {
        match self {
            Self::Rest => vec![(NeedType::Rest, 0.3)],
            Self::Eat => vec![(NeedType::Food, 0.5)],
            Self::SeekSafety | Self::Flee => vec![(NeedType::Safety, 0.3)],
            Self::TalkTo | Self::Help => vec![(NeedType::Social, 0.3)],
            Self::Build | Self::Craft | Self::Gather => vec![(NeedType::Purpose, 0.3)],
            _ => vec![],
        }
    }
}
```

### Interruptibility

Some actions can't be interrupted mid-execution:

```rust
impl ActionId {
    pub fn is_interruptible(&self) -> bool {
        match self {
            Self::Attack | Self::Charge => false,  // Committed actions
            _ => true,
        }
    }
}
```

### Duration

Base time to complete each action (in ticks):

```rust
impl ActionId {
    pub fn base_duration(&self) -> u32 {
        match self {
            Self::Attack | Self::Defend => 1,    // Instant
            Self::TalkTo => 60,                  // 1 minute
            Self::Eat => 30,                     // 30 seconds
            Self::Rest => 600,                   // 10 minutes
            Self::Craft => 1800,                 // 30 minutes
            Self::Build => 3600,                 // 1 hour
            _ => 0,                              // Continuous/instant
        }
    }
}
```

## Usage Patterns

### Creating Tasks from Actions

```rust
use crate::actions::catalog::ActionId;
use crate::entity::tasks::{Task, TaskPriority};

// Simple task
let task = Task::new(ActionId::Rest, TaskPriority::Normal, current_tick);

// Task with position target
let task = Task::new(ActionId::MoveTo, TaskPriority::High, current_tick)
    .with_position(Vec2::new(100.0, 50.0));

// Task with entity target
let task = Task::new(ActionId::TalkTo, TaskPriority::Normal, current_tick)
    .with_entity(other_entity_id);

// Player-initiated task
let task = Task::new(ActionId::Build, TaskPriority::Normal, current_tick)
    .with_position(build_location)
    .from_player();
```

### Checking Action Properties

```rust
let action = ActionId::Attack;

// Check category
match action.category() {
    ActionCategory::Combat => { /* combat UI */ }
    ActionCategory::Social => { /* social UI */ }
    _ => {}
}

// Check interruptibility before canceling
if action.is_interruptible() {
    task_queue.cancel_current();
} else {
    // Must wait for action to complete
}

// Apply need satisfaction
for (need, amount) in action.satisfies_needs() {
    entity.needs.satisfy(need, amount);
}
```

### Using ActionAvailability

```rust
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

// Check if action is available
fn can_perform(entity: &Entity, action: ActionId) -> ActionAvailability {
    match action {
        ActionId::Rest => {
            if entity.body.can_act() {
                ActionAvailability::yes()
            } else {
                ActionAvailability::no("too injured to rest safely")
            }
        }
        ActionId::Attack => {
            if entity.body.can_act() {
                ActionAvailability::yes()
            } else {
                ActionAvailability::no("unable to fight")
            }
        }
        _ => ActionAvailability::yes(),
    }
}
```

## Adding New Actions

### 1. Add to ActionId Enum

```rust
pub enum ActionId {
    // ... existing actions
    Patrol,     // New action
    Investigate, // New action
}
```

### 2. Update Category Mapping

```rust
impl ActionId {
    pub fn category(&self) -> ActionCategory {
        match self {
            // ... existing mappings
            Self::Patrol => ActionCategory::Movement,
            Self::Investigate => ActionCategory::Movement,
        }
    }
}
```

### 3. Define Need Satisfaction

```rust
impl ActionId {
    pub fn satisfies_needs(&self) -> Vec<(NeedType, f32)> {
        match self {
            // ... existing mappings
            Self::Patrol => vec![(NeedType::Purpose, 0.2)],
            Self::Investigate => vec![(NeedType::Purpose, 0.1)],
        }
    }
}
```

### 4. Set Duration and Interruptibility

```rust
impl ActionId {
    pub fn base_duration(&self) -> u32 {
        match self {
            Self::Patrol => 0,        // Continuous
            Self::Investigate => 60,  // 1 minute
            // ...
        }
    }

    pub fn is_interruptible(&self) -> bool {
        match self {
            Self::Patrol => true,
            Self::Investigate => true,
            // ...
        }
    }
}
```

### 5. Implement Execution Logic

Add execution logic in the appropriate submodule:

```rust
// src/actions/movement.rs
pub fn execute_patrol(entity: &mut Entity, task: &Task, world: &World) -> TaskProgress {
    // Patrol logic here
}
```

## Testing

```bash
cargo test --lib actions::
```

### Test Ideas
- All actions have valid categories
- Need satisfaction amounts are reasonable (0.0-1.0)
- Interruptible/non-interruptible actions correctly identified
