# Campaign Module

> Strategic layer: campaign map, army movement, weather, supply logistics, visibility, battle resolution, and scouts.

## Module Structure

```
campaign/
├── mod.rs       # Module exports
├── map.rs       # Hex-based campaign map with A* pathfinding
├── location.rs  # Location types (settlements, strategic points)
├── route.rs     # Army movement and engagement system
├── weather.rs   # Weather and seasonal effects
├── supply.rs    # Supply depots, foraging, and starvation
├── visibility.rs # Fog of war and intel tracking
├── battle.rs    # Battle resolution system
└── scouts.rs    # Scout units and reconnaissance
```

## Status: Complete

All core campaign systems implemented and tested (36 tests, 3000+ days/second throughput).

## Core Systems

### Hex Map (`map.rs`)

Axial coordinate hex grid with A* pathfinding.

```rust
// Hex coordinates (axial)
pub struct HexCoord { pub q: i32, pub r: i32 }

// Terrain types with movement costs
pub enum CampaignTerrain {
    Plains,   // 1.0 days
    Forest,   // 1.5 days
    Hills,    // 2.0 days
    Mountains,// 3.0 days
    Swamp,    // 2.5 days
    Desert,   // 1.8 days
}

// A* pathfinding
let path = map.find_path(start, destination); // ~750µs average
```

### Army Movement (`route.rs`)

```rust
pub struct Army {
    pub id: ArmyId,
    pub faction: PolityId,
    pub position: HexCoord,
    pub unit_count: u32,
    pub morale: f32,
    pub stance: ArmyStance,  // Aggressive, Defensive, Evasive
    pub orders: Option<ArmyOrder>,
}

// Stances affect interception and combat
pub enum ArmyStance {
    Aggressive, // 1.2x attack, intercepts enemies
    Defensive,  // 1.3x defense, holds position
    Evasive,    // Avoids combat when possible
}
```

### Weather System (`weather.rs`)

```rust
pub enum Weather {
    Clear, Cloudy, Rain, HeavyRain, Snow, Blizzard, Fog, Sandstorm
}

pub enum Season { Spring, Summer, Autumn, Winter }

// Modifiers
weather.movement_modifier()      // 0.3 - 1.0
weather.visibility_modifier()    // 0.2 - 1.0
weather.ranged_combat_modifier() // 0.3 - 1.0
```

### Supply System (`supply.rs`)

```rust
// Constants
BASE_SUPPLY_DAYS: f32 = 14.0;
SUPPLY_CONSUMPTION_PER_100: f32 = 1.0;
FORAGE_BASE_RATE: f32 = 0.3;
STARVATION_ATTRITION_RATE: f32 = 0.02;  // 2% per day

// Supply depots
pub struct SupplyDepot {
    pub position: HexCoord,
    pub owner: PolityId,
    pub supplies: f32,
    pub capacity: f32,
}

// Forage yields by terrain
calculate_forage_yield(terrain, weather, army_size)
```

### Visibility / Fog of War (`visibility.rs`)

```rust
pub enum HexVisibility { Unknown, Explored, Visible }

pub struct HexIntel {
    pub visibility: HexVisibility,
    pub last_seen_day: f32,
    pub known_armies: Vec<ArmyId>,
    pub known_terrain: Option<CampaignTerrain>,
}

// Constants
BASE_VISIBILITY_RANGE: i32 = 3;  // Base range in hexes

// Modifiers
calculate_visibility_range(army, terrain, weather, has_scouts)
```

### Battle Resolution (`battle.rs`)

```rust
pub enum BattleOutcome { AttackerVictory, DefenderVictory, Draw, Ongoing }

pub struct BattleResult {
    pub outcome: BattleOutcome,
    pub attacker_casualties: u32,
    pub defender_casualties: u32,
    pub rounds_fought: u32,
    pub attacker_routed: bool,
    pub defender_routed: bool,
}

// Combat strength calculation
calculate_combat_strength(army, terrain, weather, is_attacker)

// Full battle resolution
resolve_battle(attacker, defender, map, weather, max_rounds)

// Constants
BASE_CASUALTY_RATE: f32 = 0.05;   // 5% casualties per round
ROUT_THRESHOLD: f32 = 0.2;        // Rout at 20% morale
AGGRESSIVE_ATTACK_BONUS: f32 = 1.2;
DEFENSIVE_DEFENSE_BONUS: f32 = 1.3;
```

### Scout System (`scouts.rs`)

```rust
pub enum ScoutMission {
    Recon(HexCoord),  // Scout a location
    Shadow(ArmyId),   // Follow enemy army
    Patrol,           // Patrol route
    Return,           // Return to parent army
    Idle,
}

pub struct Scout {
    pub id: ScoutId,
    pub parent_army: ArmyId,
    pub position: HexCoord,
    pub mission: ScoutMission,
    pub intel_gathered: Vec<ScoutIntel>,
    pub hidden: bool,
}

// Constants
SCOUT_SPEED_MULTIPLIER: f32 = 1.5;  // 50% faster than armies
SCOUT_VISIBILITY_BONUS: i32 = 2;    // +2 hex visibility
SCOUT_DETECTION_RANGE: i32 = 4;     // Can spot at 4 hexes
SCOUT_EVASION_CHANCE: f32 = 0.7;    // 70% chance to evade
```

## Campaign Tick

```rust
// Full campaign tick integrating all systems
let events = campaign_tick(&mut state, dt_days);
supply_system.tick(&mut state.armies, &map, dt_days);
weather.update(dt_days, day_of_year, seed);
visibility.update(&armies, &scout_armies, &map, &weather, current_day);
let scout_events = scouts.tick(&armies, &map, dt_days, current_day);
```

## Performance

| Operation | Throughput |
|-----------|------------|
| Campaign simulation | 3,000+ days/second |
| A* pathfinding (20x20) | 750µs average |
| Visibility update | <1ms |

## Integration Points

### With `battle/`
- Campaign engagements trigger tactical battles
- Battle outcomes update army unit counts and morale

### With `entity/needs.rs`
- Supply affects army morale
- Starvation causes attrition

### With `simulation/`
- Campaign events can generate entity perceptions
- Strategic decisions influence individual behavior

## Test Binary

```bash
cargo run --bin campaign_sim
```

Demonstrates all systems working together over 50 simulated days.
