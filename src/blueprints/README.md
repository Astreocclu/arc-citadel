# Blueprints Module

> Building definitions, construction system, and property expression language.

## Module Structure (3880 LOC total)

```
blueprints/
├── mod.rs              # Wildcard re-exports from all submodules
├── registry.rs         # Blueprint registry (997 LOC)
├── expression.rs       # Property expression language (1031 LOC)
├── construction.rs     # Construction system (584 LOC)
├── instance.rs         # Building instances
├── schema.rs           # Blueprint schema definitions
└── evaluation.rs       # Expression evaluation
```

## Status: COMPLETE IMPLEMENTATION

The blueprint system is fully implemented with:
- Building type definitions
- Property expression language
- Construction progress tracking
- Instance management

## Core Types

### BlueprintId

```rust
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct BlueprintId(pub u32);
```

### BlueprintInstance

```rust
pub struct BlueprintInstance {
    pub id: InstanceId,
    pub blueprint: BlueprintId,
    pub position: Vec2,
    pub geometry: EvaluatedGeometry,
    pub properties: CivilianProperties,
    pub placed_by: PlacedBy,
}
```

### CivilianProperties

```rust
pub struct CivilianProperties {
    pub housing_capacity: u32,
    pub storage_capacity: f32,
    pub production_rate: f32,
    // ... more properties
}
```

### EvaluatedGeometry

```rust
pub struct EvaluatedGeometry {
    pub width: f32,
    pub height: f32,
    pub shape: GeometryShape,
}
```

## Expression Language

Blueprints use an expression language for dynamic properties:

```rust
// Example expressions
"base_capacity * size_multiplier"
"if housing_type == 'mansion' then 8 else 2"
"min(storage, max_storage)"
```

The expression evaluator handles:
- Arithmetic operations
- Comparisons
- Conditionals
- Variable references

## Registry

```rust
pub struct BlueprintRegistry {
    pub blueprints: HashMap<BlueprintId, Blueprint>,
}

impl BlueprintRegistry {
    pub fn get(&self, id: BlueprintId) -> Option<&Blueprint>
    pub fn register(&mut self, blueprint: Blueprint) -> BlueprintId
    pub fn all(&self) -> impl Iterator<Item = &Blueprint>
}
```

## Construction System

```rust
pub struct ConstructionSite {
    pub blueprint: BlueprintId,
    pub position: Vec2,
    pub progress: f32,
    pub workers: Vec<EntityId>,
}

pub fn advance_construction(site: &mut ConstructionSite, work_amount: f32) -> bool
```

## Key Exports

Via wildcard exports in mod.rs:

```rust
pub use instance::*;
pub use registry::*;
pub use schema::*;
```

## Integration Points

### With `simulation/`
- Construction sites progress each tick
- Workers assigned to building tasks

### With `city/`
- Buildings affect city capacity
- Housing for population

### With `entity/`
- Workers reference construction sites
- Building tasks in task queues

## Usage Example

```rust
// Get a blueprint
let blueprint = registry.get(BlueprintId(1))?;

// Create an instance
let instance = BlueprintInstance {
    id: InstanceId::new(),
    blueprint: blueprint.id,
    position: Vec2::new(100.0, 200.0),
    geometry: evaluate_geometry(&blueprint.geometry_expr),
    properties: evaluate_properties(&blueprint.property_exprs),
    placed_by: PlacedBy::Player,
};
```

## Testing

```bash
cargo test --lib blueprints::
```
