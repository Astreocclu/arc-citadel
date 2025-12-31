# Spatial Module

> Spatial data structures for efficient neighbor queries and pathfinding.

## Module Structure

```
spatial/
├── mod.rs          # Module exports
├── grid.rs         # Generic 2D grid
├── sparse_hash.rs  # Sparse hash grid for entities
└── flow_field.rs   # Flow field pathfinding (stub)
```

## SparseHashGrid (`sparse_hash.rs`)

Primary spatial index for entity queries. Enables O(1) neighbor lookups.

```rust
pub struct SparseHashGrid {
    cell_size: f32,
    cells: AHashMap<(i32, i32), Vec<EntityId>>,
}
```

### Core Operations

```rust
impl SparseHashGrid {
    // Create with cell size (10.0 works well for perception range 50.0)
    pub fn new(cell_size: f32) -> Self;

    // Clear all entities
    pub fn clear(&mut self);

    // Add entity at position
    pub fn insert(&mut self, entity: EntityId, pos: Vec2);

    // Remove entity from position
    pub fn remove(&mut self, entity: EntityId, pos: Vec2);

    // Query entities in 3x3 cell neighborhood
    pub fn query_neighbors(&self, pos: Vec2) -> impl Iterator<Item = EntityId>;

    // Query entities within radius
    pub fn query_radius(&self, center: Vec2, radius: f32, positions: &[Vec2]) -> Vec<EntityId>;

    // Rebuild from entity positions
    pub fn rebuild<'a>(&mut self, entities: impl Iterator<Item = (EntityId, Vec2)>);
}
```

### Usage in Simulation

```rust
// Build spatial index at start of tick
let mut grid = SparseHashGrid::new(10.0);
let positions: Vec<_> = world.humans.positions.iter().cloned().collect();
let ids: Vec<_> = world.humans.ids.iter().cloned().collect();
grid.rebuild(ids.iter().cloned().zip(positions.iter().cloned()));

// Query neighbors for perception
for (i, &observer_id) in ids.iter().enumerate() {
    let observer_pos = positions[i];

    let nearby: Vec<_> = grid.query_neighbors(observer_pos)
        .filter(|&e| e != observer_id)  // Exclude self
        .collect();

    for neighbor_id in nearby {
        // Process visible neighbor
    }
}
```

## Grid<T> (`grid.rs`)

Generic 2D grid for terrain, flow fields, and other spatial data:

```rust
pub struct Grid<T: Clone + Default> {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub origin: Vec2,
    data: Vec<T>,
}
```

### Core Operations

```rust
impl<T: Clone + Default> Grid<T> {
    // Create grid
    pub fn new(width: usize, height: usize, cell_size: f32, origin: Vec2) -> Self;

    // Access by cell coordinates
    pub fn get(&self, x: usize, y: usize) -> Option<&T>;
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T>;
    pub fn set(&mut self, x: usize, y: usize, value: T);

    // Convert world position to cell
    pub fn world_to_cell(&self, pos: Vec2) -> (usize, usize);

    // Sample at world position
    pub fn sample(&self, pos: Vec2) -> Option<&T>;

    // Get cell center in world coordinates
    pub fn cell_center(&self, x: usize, y: usize) -> Vec2;
}
```

### Usage for Terrain

```rust
#[derive(Clone, Default)]
enum TerrainType {
    #[default]
    Grass,
    Forest,
    Water,
    Mountain,
}

// Create terrain grid
let mut terrain = Grid::new(100, 100, 1.0, Vec2::default());

// Set terrain
terrain.set(10, 20, TerrainType::Forest);

// Query terrain at entity position
let entity_pos = Vec2::new(10.5, 20.5);
if let Some(terrain_type) = terrain.sample(entity_pos) {
    match terrain_type {
        TerrainType::Forest => { /* reduce visibility */ }
        TerrainType::Water => { /* slow movement */ }
        _ => {}
    }
}
```

## Performance Characteristics

### SparseHashGrid
- **Insert/Remove**: O(1) average
- **Query Neighbors**: O(k) where k = entities in 9 cells
- **Memory**: Only allocates cells with entities
- **Best cell size**: ~1/5 of typical query radius

### Grid<T>
- **Access**: O(1) direct indexing
- **Memory**: width × height × sizeof(T)
- **Coordinate conversion**: O(1)

## Best Practices

### Rebuilding the Spatial Grid
```rust
// Rebuild at start of each tick when entities move
fn run_perception(world: &World) -> Vec<Perception> {
    let mut grid = SparseHashGrid::new(10.0);

    let positions: Vec<_> = world.humans.positions.iter().cloned().collect();
    let ids: Vec<_> = world.humans.ids.iter().cloned().collect();

    grid.rebuild(ids.iter().cloned().zip(positions.iter().cloned()));

    // Now use grid for queries
    perception_system(&grid, &positions, &ids, 50.0)
}
```

### Choosing Cell Size
```rust
// Cell size should be ~1/5 of perception range
// This gives good balance between:
// - Too small: many cells, overhead
// - Too large: many entities per cell, slow filtering

let perception_range = 50.0;
let cell_size = 10.0;  // 1/5 of range

let grid = SparseHashGrid::new(cell_size);
```

### Efficient Neighbor Queries
```rust
// query_neighbors returns iterator - chain operations efficiently
let threats: Vec<_> = grid.query_neighbors(pos)
    .filter(|&e| e != self_id)           // Exclude self
    .filter(|&e| is_hostile(e))          // Only hostile
    .filter(|&e| in_range(e, pos, 30.0)) // Within attack range
    .collect();
```

## Future: Flow Fields (`flow_field.rs`)

Planned for group pathfinding:
- Single pathfinding calculation shared by many entities
- Direction vectors at each grid cell
- Efficient for large groups moving to same destination

## Testing

```bash
cargo test --lib spatial::
```

### Test Ideas
- Insert and query returns correct entities
- Entities at edge of range included/excluded correctly
- Grid coordinate conversion accurate
