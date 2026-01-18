# Battle Module

> Tactical combat with fog of war and courier delays. NOT Total War: terrain is dense, vision is scarce, control is delegated.

## Module Structure (8322 LOC total)

```
battle/
├── mod.rs              # Module exports (74 re-exported items)
├── execution.rs        # Battle execution loop (1633 LOC)
├── orders.rs           # Order system (981 LOC)
├── movement.rs         # Unit movement (679 LOC)
├── formation_layout.rs # Formation positioning (634 LOC)
├── planning.rs         # Battle planning system
├── battle_map.rs       # Hex-based battle terrain
├── hex.rs              # Hex coordinate system
├── courier.rs          # Order delay system
├── morale.rs           # Unit morale and breaking
├── engagement.rs       # Combat engagement detection
├── resolution.rs       # Unit combat resolution
├── triggers.rs         # Go-code trigger system
├── visibility.rs       # Fog of war
├── terrain.rs          # Terrain effects
├── pathfinding.rs      # Unit pathfinding
├── unit_type.rs        # Unit type definitions
├── units.rs            # Unit management
├── constants.rs        # Battle constants
├── ai/                 # Battle AI subsystem
│   ├── mod.rs          # BattleAI trait
│   ├── decision.rs     # AI decision making
│   ├── personalities/  # AI personality configs
│   └── preset.rs       # Preset AI behaviors
└── ranged.rs           # (orphaned - depends on unimplemented weapons)
```

## Status: COMPLETE IMPLEMENTATION

The battle system is fully implemented with:
- Hex-based tactical maps
- Courier-delayed orders
- Fog of war visibility
- Morale and rout mechanics
- Formation positioning
- Go-code triggers for contingency planning

## Key Design Principles

**NOT Total War:**
- Terrain is dense (navigation is a puzzle)
- Vision is scarce (information must be gathered)
- Orders go through couriers (not instant)
- Same simulation at all accessibility levels

## Core Types

### BattleState

```rust
pub struct BattleState {
    pub phase: BattlePhase,
    pub outcome: BattleOutcome,
    pub current_tick: Tick,
    pub armies: Vec<Army>,
    pub couriers: CourierSystem,
    pub event_log: BattleEventLog,
}
```

### BattlePlan

```rust
pub struct BattlePlan {
    pub formations: Vec<UnitDeployment>,
    pub waypoints: Vec<WaypointPlan>,
    pub go_codes: Vec<GoCode>,
    pub contingencies: Vec<Contingency>,
}
```

### CourierSystem

```rust
pub struct CourierSystem {
    pub in_flight: Vec<CourierInFlight>,
}

pub struct CourierInFlight {
    pub id: CourierId,
    pub order: Order,
    pub target_unit: UnitId,
    pub arrival_tick: Tick,
    pub status: CourierStatus,
}
```

## Battle Execution Flow

Each tick during battle (from `execution.rs`):

```rust
pub fn execute_battle_tick(state: &mut BattleState, map: &BattleMap) -> BattleEventLog {
    // 1. Movement phase
    advance_unit_movement(&mut state.armies, map);

    // 2. Courier delivery
    state.couriers.deliver_orders(state.current_tick);

    // 3. Engagement detection
    let engagements = find_all_engagements(&state.armies, map);

    // 4. Combat resolution
    resolve_unit_combat(&mut state.armies, &engagements);

    // 5. Morale checks
    for army in &mut state.armies {
        for unit in &mut army.units {
            apply_stress(unit, &engagements);
            if check_morale_break(unit) {
                process_morale_break(unit);
            }
            check_rally(unit);
        }
    }

    // 6. Go-code triggers
    evaluate_all_gocodes(state, map);

    // 7. Check battle end
    check_battle_end(state);
}
```

## Courier System

Orders don't arrive instantly - couriers carry commands:

```rust
// Send an order (order travels to unit)
courier_system.send_order(order, target_unit, current_tick, distance);

// Orders arrive based on distance and courier speed
pub const COURIER_SPEED: f32 = 10.0; // hexes per tick
```

This creates realistic fog of war and forces planning ahead.

## Go-Codes and Contingencies

Pre-planned responses to battlefield conditions:

```rust
pub struct GoCode {
    pub id: GoCodeId,
    pub name: String,
    pub trigger: GoCodeTrigger,
    pub orders: Vec<Order>,
}

pub enum GoCodeTrigger {
    ManualActivation,
    UnitReachesWaypoint { unit: UnitId, waypoint: BattleHexCoord },
    EnemySpotted { threshold: usize },
    UnitEngaged { unit: UnitId },
    CasualtyThreshold { percentage: f32 },
}
```

## Formation System

Units deploy in formations:

```rust
pub fn compute_formation_positions(
    deployment: &UnitDeployment,
    center: BattleHexCoord,
    facing: HexDirection,
) -> Vec<FormationSlot>
```

Formation types:
- Line (standard infantry)
- Column (marching)
- Wedge (cavalry charge)
- Square (defensive)

## Morale System

```rust
pub fn check_morale_break(unit: &Unit) -> bool
pub fn process_morale_break(unit: &mut Unit)
pub fn check_rally(unit: &mut Unit) -> bool
pub fn calculate_contagion_stress(unit: &Unit, nearby: &[Unit]) -> f32
```

Morale affected by:
- Casualties taken
- Nearby allies routing
- Commander presence
- Surrounded/flanked status

## Visibility System

```rust
pub fn update_army_visibility(army: &mut Army, map: &BattleMap, enemies: &[Army])

pub struct ArmyVisibility {
    pub visible_hexes: HashSet<BattleHexCoord>,
    pub spotted_units: Vec<UnitId>,
}
```

## Integration Points

### With `combat/`
- `resolve_unit_combat()` uses combat resolution
- Individual wound and morale effects

### With `skills/`
- Combat skill affects engagement outcomes
- Attention budget for complex maneuvers

### With `simulation/`
- Battle triggered from entity actions
- Results affect entity needs

## Testing

```bash
cargo test --lib battle::
```

Key test files:
- Battle execution loop
- Courier delivery timing
- Morale break/rally
- Formation positioning
- Go-code triggers
