# Campaign Module

> Strategic layer: campaign map, locations, routes, weather, and supply lines.

## Module Structure

```
campaign/
├── mod.rs       # Module exports
├── map.rs       # Campaign map (stub)
├── location.rs  # Location types (stub)
├── route.rs     # Routes between locations (stub)
├── weather.rs   # Weather system (stub)
└── supply.rs    # Supply logistics (stub)
```

## Status: Stub Implementation

This module is planned but not yet implemented.

## Planned Design

### Campaign Map

```
┌─────────────────────────────────────────┐
│              CAMPAIGN MAP               │
│                                         │
│    [Castle]───────[Village]             │
│        │              │                 │
│        │              │                 │
│    [Forest]───────[River]───[Bridge]    │
│        │                        │       │
│        │                        │       │
│    [Mountain]              [Town]       │
│                                         │
└─────────────────────────────────────────┘
```

### Locations

```rust
pub trait Location {
    fn name(&self) -> &str;
    fn position(&self) -> Vec2;
    fn terrain_type(&self) -> TerrainType;
    fn resources(&self) -> Vec<ResourceType>;
    fn population(&self) -> usize;
}

pub struct Town { /* ... */ }
pub struct Forest { /* ... */ }
pub struct Mountain { /* ... */ }
```

### Routes

Travel time depends on:
- Distance between locations
- Terrain difficulty
- Weather conditions
- Supply burden

```rust
pub struct Route {
    pub from: LocationId,
    pub to: LocationId,
    pub distance: f32,
    pub terrain: TerrainType,
}

impl Route {
    pub fn travel_time(&self, weather: &Weather, supply_weight: f32) -> u32 {
        // Time emerges from interactions
    }
}
```

### Weather System

Weather affects:
- Movement speed
- Combat effectiveness
- Supply consumption
- Morale

```rust
pub struct Weather {
    pub temperature: f32,
    pub precipitation: Precipitation,
    pub wind_speed: f32,
    pub visibility: f32,
}
```

### Supply Lines

```rust
pub struct SupplyLine {
    pub source: LocationId,
    pub destination: LocationId,
    pub capacity: f32,
    pub current_flow: f32,
}

pub struct SupplyState {
    pub food: f32,
    pub ammunition: f32,
    pub medicine: f32,
}
```

## Integration Points

### With `battle/`
- Campaign positions determine battle locations
- Battle outcomes affect campaign control

### With `entity/needs.rs`
- Supply affects entity food need
- Travel affects rest need

### With `simulation/`
- Campaign events generate entity perceptions

## Future Implementation

1. **Start with map and locations** as data structures
2. **Add routes** with basic travel time
3. **Implement weather** affecting travel
4. **Add supply lines** for resource flow
