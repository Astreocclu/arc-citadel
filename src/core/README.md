# Core Module

> Foundation types, errors, and configuration shared across all modules.

## Module Structure

```
core/
├── mod.rs      # Module exports
├── types.rs    # Core type definitions
├── error.rs    # Error types and Result alias
└── config.rs   # Configuration (stub)
```

## Core Types (`types.rs`)

### EntityId

Unique identifier for entities:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
```

### Vec2

2D position/vector:

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

// Implements Add, Sub, Mul<f32>
let a = Vec2::new(1.0, 2.0);
let b = Vec2::new(3.0, 4.0);
let c = a + b;           // Vec2 { x: 4.0, y: 6.0 }
let d = a * 2.0;         // Vec2 { x: 2.0, y: 4.0 }
let dist = a.distance(&b); // 2.828...
```

### Other Types

```rust
// Simulation time unit
pub type Tick = u64;

// Location on campaign map
pub struct LocationId(pub u32);

// Flow field reference
pub struct FlowFieldId(pub u32);

// Species enumeration
pub enum Species {
    Human,
    Dwarf,
    Elf,
}
```

## Error Handling (`error.rs`)

Unified error type using `thiserror`:

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

// Convenience type alias
pub type Result<T> = std::result::Result<T, ArcError>;
```

## Usage Patterns

### Working with EntityId
```rust
use crate::core::types::EntityId;

// Create new entity ID
let id = EntityId::new();

// Use as hash key
let mut map: HashMap<EntityId, String> = HashMap::new();
map.insert(id, "Marcus".into());

// Compare IDs
if entity_a.id == entity_b.id {
    // Same entity
}
```

### Working with Vec2
```rust
use crate::core::types::Vec2;

// Create positions
let pos = Vec2::new(10.0, 20.0);
let target = Vec2::new(15.0, 25.0);

// Calculate distance
let dist = pos.distance(&target);

// Get direction to target
let direction = (target - pos).normalize();

// Move toward target
let speed = 1.0;
let new_pos = pos + direction * speed;
```

### Working with Result
```rust
use crate::core::error::{ArcError, Result};

fn find_entity(world: &World, id: EntityId) -> Result<usize> {
    world.humans.index_of(id)
        .ok_or(ArcError::EntityNotFound(id))
}

fn process_entity(world: &World, id: EntityId) -> Result<()> {
    let idx = find_entity(world, id)?;  // Propagate error with ?
    // Process entity...
    Ok(())
}

// Handle errors
match process_entity(&world, entity_id) {
    Ok(()) => println!("Success"),
    Err(ArcError::EntityNotFound(id)) => println!("Entity {:?} not found", id),
    Err(e) => println!("Error: {}", e),
}
```

## Best Practices

### Use Type-Safe IDs
```rust
// Each ID type is distinct
let entity_id = EntityId::new();
let location_id = LocationId(42);
let flow_field_id = FlowFieldId(1);

// Can't accidentally mix them up - compiler prevents it
```

### Use Vec2 for Positions
```rust
// Consistent coordinate handling
struct Entity {
    position: Vec2,
    velocity: Vec2,
}

// Easy movement calculation
entity.position = entity.position + entity.velocity * delta_time;
```

### Use Result for Fallible Operations
```rust
// Functions that can fail return Result
fn load_config() -> Result<Config> { ... }
fn find_entity(id: EntityId) -> Result<&Entity> { ... }
fn parse_command(input: &str) -> Result<ParsedIntent> { ... }
```

## Testing

```bash
cargo test --lib core::
```
