# Battle Module

> Tactical combat: battle maps, planning, execution, courier system.

## Module Structure

```
battle/
├── mod.rs          # Module exports
├── battle_map.rs   # Battle terrain (stub)
├── planning.rs     # Battle planning (stub)
├── execution.rs    # Battle execution (stub)
├── courier.rs      # Order delay system (stub)
└── resolution.rs   # Battle outcomes (stub)
```

## Status: Stub Implementation

This module is planned but not yet implemented.

## Planned Design

### Battle Map

```
┌─────────────────────────────────────────┐
│              BATTLE MAP                 │
│                                         │
│  ████  Hill  ████    ░░░░ River ░░░░   │
│  ████████████████    ░░░░░░░░░░░░░░░   │
│                                         │
│  ▓▓▓▓ Forest ▓▓▓▓                      │
│  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓    ═══ Bridge ═══   │
│                                         │
│        ○ Unit A        ● Enemy Unit    │
│        ○ Unit B                        │
│                                         │
└─────────────────────────────────────────┘
```

### Terrain Effects

```rust
pub struct BattleMap {
    pub terrain: Grid<TerrainType>,
    pub elevation: Grid<f32>,
    pub cover: Grid<CoverType>,
}

pub enum TerrainType {
    Open,
    Forest,
    Hill,
    Water,
    Bridge,
    Building,
}

// Terrain affects movement and combat
impl TerrainType {
    pub fn movement_cost(&self) -> f32;
    pub fn defense_bonus(&self) -> f32;
    pub fn visibility(&self) -> f32;
}
```

### Courier System

Orders don't arrive instantly. Couriers carry commands:

```rust
pub struct Courier {
    pub order: Order,
    pub from: EntityId,      // Commander
    pub to: EntityId,        // Unit
    pub sent_tick: Tick,
    pub arrival_tick: Tick,  // Based on distance
}

pub struct CourierSystem {
    pub in_transit: Vec<Courier>,
}

impl CourierSystem {
    pub fn send_order(&mut self, order: Order, from: EntityId, to: EntityId, distance: f32) {
        let travel_time = calculate_courier_time(distance);
        // Order arrives later
    }

    pub fn deliver_orders(&mut self, current_tick: Tick) -> Vec<(EntityId, Order)> {
        // Return orders that have arrived
    }
}
```

### Battle Planning

```rust
pub struct BattlePlan {
    pub formations: Vec<Formation>,
    pub objectives: Vec<Objective>,
    pub contingencies: Vec<Contingency>,
}

pub struct Formation {
    pub units: Vec<EntityId>,
    pub shape: FormationShape,
    pub facing: Vec2,
}
```

### Battle Execution

Each tick during battle:
1. Deliver arrived courier orders
2. Units execute current orders
3. Combat resolution for engaged units
4. Morale checks for wounded/surrounded units
5. Rout and pursuit handling

## Integration Points

### With `combat/`
- Individual combat resolution
- Wound and morale effects

### With `entity/`
- Entity positioning and movement
- Needs affected by battle stress

### With `campaign/`
- Battle location from campaign map
- Battle outcome affects campaign

## Design Principles

### Courier Delays Create Emergence

Orders take time to reach units:
- Player gives order → courier dispatched
- Courier travels to unit → takes time
- Unit receives order → begins execution

This creates realistic fog of war and forces planning ahead.

### Formation Matters

Units in formation gain benefits:
- Morale support from nearby allies
- Coordinated attacks
- Defensive bonuses

Breaking formation has consequences.

## Future Implementation

1. **Start with battle map** and terrain
2. **Add courier system** for order delays
3. **Implement formations** and positioning
4. **Add battle execution** loop
5. **Integrate with combat/** for resolution
