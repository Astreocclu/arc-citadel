# Campaign AI Agent Design

## Overview

Create an AI agent binary (`src/bin/campaign_ai.rs`) that runs two competing AI factions through a campaign simulation. Each faction gets its own AI "brain" that makes strategic decisions based on visible game state, respecting fog of war.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     campaign_ai binary                          │
├─────────────────────────────────────────────────────────────────┤
│  CampaignState     - Armies, positions, orders                  │
│  SupplySystem      - Supply tracking per army                   │
│  ScoutSystem       - Scout units and intel                      │
│  VisibilitySystem  - Fog of war per faction                     │
│  WeatherState      - Regional weather                           │
│  CampaignMap       - Hex grid with terrain                      │
├─────────────────────────────────────────────────────────────────┤
│  FactionAI (×2)    - One per faction, makes decisions           │
│    - perceive()    - Build world model from visible state       │
│    - decide()      - Choose orders for each army/scout          │
│    - execute()     - Issue orders to game systems               │
└─────────────────────────────────────────────────────────────────┘
```

### Main Loop

```rust
loop {
    // 1. Each AI perceives (only what their faction can see)
    let blue_world = blue_ai.perceive(&state, &visibility, &scouts);
    let red_world = red_ai.perceive(&state, &visibility, &scouts);

    // 2. Each AI decides (issues movement orders, scout missions)
    let blue_orders = blue_ai.decide(&blue_world);
    let red_orders = red_ai.decide(&red_world);

    // 3. Execute orders
    blue_ai.execute(&mut state, &mut scouts, blue_orders);
    red_ai.execute(&mut state, &mut scouts, red_orders);

    // 4. Systems tick (movement, supply, battles resolve)
    tick_all_systems(...);

    // 5. Check win condition
    if let Some(victor) = check_victory(...) {
        break;
    }
}
```

## AI Decision-Making

### World Model (what the AI knows)

```rust
struct FactionWorldModel {
    my_armies: Vec<ArmyInfo>,      // Position, size, supplies, morale
    known_enemies: Vec<ArmyInfo>,  // From visibility + scout intel
    explored_hexes: HashSet<HexCoord>,
    my_depot: HexCoord,
    enemy_depot: Option<HexCoord>, // If scouted
}
```

### Decision Priorities (in order)

1. **Critical supply** - Army below 3 days supplies? → Return to depot
2. **Favorable engagement** - Enemy visible and we have advantage? → Attack
3. **Retreat** - Enemy visible and we're outnumbered? → Fall back toward depot
4. **Scout unexplored** - Idle scout? → Send to nearest unexplored area
5. **Advance** - Well-supplied army with no target? → Move toward enemy depot
6. **Hold** - Default: stay in place, forage

### Advantage Calculation

```rust
fn has_advantage(my_army: &Army, enemy: &Army) -> bool {
    let my_strength = my_army.unit_count as f32 * my_army.morale;
    let enemy_strength = enemy.unit_count as f32 * enemy.morale;
    my_strength > enemy_strength * 1.2  // 20% margin required
}
```

## Win Conditions

- **Military victory**: Enemy faction has no armies with >10 units remaining
- **Depot capture**: An army occupies the enemy depot for 3 consecutive days
- **Draw**: 200 days pass with no victor (stalemate)

## Output Format

```
╔═══════════════════════════════════════════════════════════════╗
║           ARC CITADEL: AI CAMPAIGN BATTLE                     ║
╚═══════════════════════════════════════════════════════════════╝

[Setup]
Map: 30x30 hexes, seed 12345
Faction 1 (Blue): 2 armies, depot at (2, 2)
Faction 2 (Red): 2 armies, depot at (27, 27)

[Day 1]
Blue AI: Legion advances toward (15, 15), Scout sent to (20, 20)
Red AI: Host holds at depot, Scout sent to (10, 10)

[Day 5]
Blue AI: Legion sees Red Host! Engaging (advantage: 1.3x)
BATTLE at (14, 16): Blue Legion vs Red Host
  Result: Blue victory, Red retreats

[Day 12]
Red AI: All armies destroyed
═══════════════════════════════════════════════════════════════
VICTORY: Faction 1 (Blue) - Military Victory on Day 12
═══════════════════════════════════════════════════════════════

Statistics:
  Total battles: 3
  Blue casualties: 145
  Red casualties: 380
  Hexes explored: Blue 180, Red 95
```

## CLI Arguments

- `--seed <N>` - Random seed for reproducible runs
- `--max-days <N>` - Override 200 day limit (default: 200)
- `--verbose` - Print every AI decision, not just major events

## Implementation Tasks

### Task 1: Create FactionAI struct and world model
**File:** `src/bin/campaign_ai.rs`

Create the core AI data structures:
- `FactionWorldModel` struct with armies, enemies, explored hexes, depots
- `FactionAI` struct with faction ID, depot position, and world model
- `perceive()` method that builds world model from visibility system

### Task 2: Implement decision logic
**File:** `src/bin/campaign_ai.rs`

Implement the priority-based decision system:
- `decide()` method that returns orders for each army/scout
- `has_advantage()` helper for combat decisions
- `find_nearest_unexplored()` for scout targeting
- Order types: MoveTo, Attack, Retreat, Hold

### Task 3: Implement order execution
**File:** `src/bin/campaign_ai.rs`

Connect AI decisions to game systems:
- `execute()` method that applies orders to armies and scouts
- Set army orders via `army.orders = Some(ArmyOrder::MoveTo(target))`
- Assign scout missions via `scout.assign_recon(target, map)`

### Task 4: Create main simulation loop
**File:** `src/bin/campaign_ai.rs`

Build the complete simulation:
- Parse CLI arguments (seed, max-days, verbose)
- Initialize map, factions, armies, scouts, supply depots
- Run main loop: perceive → decide → execute → tick → check victory
- Print formatted output

### Task 5: Implement win condition checks
**File:** `src/bin/campaign_ai.rs`

Add victory detection:
- `check_military_victory()` - no enemy armies with >10 units
- `check_depot_capture()` - army on enemy depot for 3 days
- `check_stalemate()` - day limit reached
- Track depot occupation days

### Task 6: Add statistics tracking
**File:** `src/bin/campaign_ai.rs`

Track and display game metrics:
- Total battles fought
- Casualties per faction
- Hexes explored per faction
- Days until victory
- Simulation time

## Verification

```bash
# Build and run with default settings
cargo run --bin campaign_ai

# Run with specific seed for reproducibility
cargo run --bin campaign_ai -- --seed 42

# Run with verbose output
cargo run --bin campaign_ai -- --verbose

# Run with custom day limit
cargo run --bin campaign_ai -- --max-days 100
```

Expected behavior:
- Two AI factions compete using only visible information
- Scouts explore and gather intel before armies advance
- Armies retreat when low on supplies
- Battles occur when armies meet and one has advantage
- Game ends with clear victory or stalemate
