# Campaign AI Agent Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create an AI agent binary that runs two competing AI factions through a campaign simulation, with each faction making strategic decisions based on visible game state while respecting fog of war.

**Architecture:** Two `FactionAI` instances each maintain a `FactionWorldModel` built from the `VisibilitySystem`. The main loop cycles: perceive → decide → execute → tick systems → check victory. Decisions follow a priority-based utility system.

**Tech Stack:** Rust, existing campaign systems (`CampaignState`, `SupplySystem`, `ScoutSystem`, `VisibilitySystem`, `RegionalWeather`), clap for CLI.

---

## Task 1: Create Basic Binary Structure with CLI Parsing

**Files:**
- Create: `src/bin/campaign_ai.rs`
- Reference: `src/bin/campaign_sim.rs` (pattern)

**Step 1: Create the binary file with CLI parsing**

```rust
//! Campaign AI Agent
//! Two AI factions compete in a campaign simulation

use arc_citadel::campaign::{
    apply_retreat, campaign_tick, resolve_battle, ArmyId, ArmyStance, BattleOutcome,
    CampaignMap, CampaignState, HexCoord, RegionalWeather, ScoutId, ScoutSystem,
    SupplySystem, VisibilitySystem,
};
use arc_citadel::core::types::PolityId;
use clap::Parser;
use std::collections::HashSet;

/// Campaign AI - Two factions compete using AI decision-making
#[derive(Parser, Debug)]
#[command(name = "campaign_ai")]
#[command(about = "Run a campaign simulation with two AI factions")]
struct Args {
    /// Random seed for reproducible runs
    #[arg(long, default_value_t = 42)]
    seed: u64,

    /// Maximum days before stalemate
    #[arg(long, default_value_t = 200)]
    max_days: u32,

    /// Print every AI decision
    #[arg(long, default_value_t = false)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║           ARC CITADEL: AI CAMPAIGN BATTLE                     ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    println!("[Setup]");
    println!("Map: 30x30 hexes, seed {}", args.seed);
    println!("Max days: {}", args.max_days);
    println!("Verbose: {}", args.verbose);
}
```

**Step 2: Verify it compiles**

Run: `cargo build --bin campaign_ai`
Expected: Compiles successfully

**Step 3: Run to verify CLI works**

Run: `cargo run --bin campaign_ai -- --seed 123 --verbose`
Expected: Prints setup info with seed 123 and verbose true

**Step 4: Commit**

```bash
git add src/bin/campaign_ai.rs
git commit -m "feat(campaign-ai): create binary with CLI parsing"
```

---

## Task 2: Create FactionWorldModel and FactionAI Structs

**Files:**
- Modify: `src/bin/campaign_ai.rs`

**Step 1: Add the data structures after the Args struct**

```rust
/// What the AI knows about an army (own or enemy)
#[derive(Debug, Clone)]
struct ArmyInfo {
    id: ArmyId,
    position: HexCoord,
    unit_count: u32,
    morale: f32,
    supplies_days: f32,
    is_own: bool,
}

/// World model from a faction's perspective (fog of war)
#[derive(Debug)]
struct FactionWorldModel {
    my_armies: Vec<ArmyInfo>,
    known_enemies: Vec<ArmyInfo>,
    explored_hexes: HashSet<HexCoord>,
    my_depot: HexCoord,
    enemy_depot: Option<HexCoord>,
    current_day: f32,
}

/// Orders the AI can issue
#[derive(Debug, Clone)]
enum AIOrder {
    MoveTo { army: ArmyId, target: HexCoord },
    Attack { army: ArmyId, target: ArmyId },
    Retreat { army: ArmyId },
    Hold { army: ArmyId },
    DeployScout { from_army: ArmyId, target: HexCoord },
}

/// AI decision-maker for a single faction
struct FactionAI {
    faction: PolityId,
    depot: HexCoord,
    scout_ids: Vec<ScoutId>,
}

impl FactionAI {
    fn new(faction: PolityId, depot: HexCoord) -> Self {
        Self {
            faction,
            depot,
            scout_ids: Vec::new(),
        }
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build --bin campaign_ai`
Expected: Compiles (may have unused warnings, that's fine)

**Step 3: Commit**

```bash
git add src/bin/campaign_ai.rs
git commit -m "feat(campaign-ai): add FactionWorldModel and FactionAI structs"
```

---

## Task 3: Implement perceive() Method

**Files:**
- Modify: `src/bin/campaign_ai.rs`

**Step 1: Add perceive method to FactionAI**

```rust
impl FactionAI {
    fn new(faction: PolityId, depot: HexCoord) -> Self {
        Self {
            faction,
            depot,
            scout_ids: Vec::new(),
        }
    }

    /// Build world model from what the faction can see
    fn perceive(
        &self,
        state: &CampaignState,
        visibility: &VisibilitySystem,
        supply: &SupplySystem,
        enemy_depot: HexCoord,
    ) -> FactionWorldModel {
        let fv = visibility.get_faction(self.faction);

        // Collect own armies
        let my_armies: Vec<ArmyInfo> = state
            .armies
            .iter()
            .filter(|a| a.faction == self.faction)
            .map(|a| {
                let supplies_days = supply
                    .get_army_supply(a.id)
                    .map(|s| s.days_until_starvation(a.unit_count))
                    .unwrap_or(0.0);
                ArmyInfo {
                    id: a.id,
                    position: a.position,
                    unit_count: a.unit_count,
                    morale: a.morale,
                    supplies_days,
                    is_own: true,
                }
            })
            .collect();

        // Collect visible enemies
        let known_enemies: Vec<ArmyInfo> = visibility
            .visible_enemies(self.faction, &state.armies)
            .into_iter()
            .map(|a| {
                ArmyInfo {
                    id: a.id,
                    position: a.position,
                    unit_count: a.unit_count,
                    morale: a.morale,
                    supplies_days: 10.0, // Unknown, estimate
                    is_own: false,
                }
            })
            .collect();

        // Collect explored hexes
        let explored_hexes: HashSet<HexCoord> = fv
            .map(|f| f.intel.keys().copied().collect())
            .unwrap_or_default();

        // Check if enemy depot is known (in explored hexes)
        let enemy_depot_known = explored_hexes.contains(&enemy_depot);

        FactionWorldModel {
            my_armies,
            known_enemies,
            explored_hexes,
            my_depot: self.depot,
            enemy_depot: if enemy_depot_known { Some(enemy_depot) } else { None },
            current_day: state.current_day,
        }
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build --bin campaign_ai`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/bin/campaign_ai.rs
git commit -m "feat(campaign-ai): implement perceive() method for world model"
```

---

## Task 4: Implement decide() Method with Priority Logic

**Files:**
- Modify: `src/bin/campaign_ai.rs`

**Step 1: Add helper function for advantage calculation**

```rust
/// Check if our army has advantage over enemy (20% margin)
fn has_advantage(my_army: &ArmyInfo, enemy: &ArmyInfo) -> bool {
    let my_strength = my_army.unit_count as f32 * my_army.morale;
    let enemy_strength = enemy.unit_count as f32 * enemy.morale;
    my_strength > enemy_strength * 1.2
}

/// Find nearest unexplored hex from a position
fn find_nearest_unexplored(
    from: HexCoord,
    explored: &HashSet<HexCoord>,
    map: &CampaignMap,
) -> Option<HexCoord> {
    // Search in expanding rings
    for radius in 1..=20 {
        for q in (from.q - radius)..=(from.q + radius) {
            for r in (from.r - radius)..=(from.r + radius) {
                let coord = HexCoord::new(q, r);
                if from.distance(&coord) == radius
                    && map.contains(&coord)
                    && !explored.contains(&coord)
                {
                    return Some(coord);
                }
            }
        }
    }
    None
}
```

**Step 2: Add decide method to FactionAI**

```rust
impl FactionAI {
    // ... existing methods ...

    /// Make decisions based on world model
    /// Priority order:
    /// 1. Critical supply - return to depot
    /// 2. Favorable engagement - attack
    /// 3. Retreat - fall back when outnumbered
    /// 4. Scout unexplored - send scouts
    /// 5. Advance - move toward enemy depot
    /// 6. Hold - default
    fn decide(&self, world: &FactionWorldModel, map: &CampaignMap) -> Vec<AIOrder> {
        let mut orders = Vec::new();

        for army in &world.my_armies {
            // Priority 1: Critical supply (< 3 days)
            if army.supplies_days < 3.0 {
                orders.push(AIOrder::Retreat { army: army.id });
                continue;
            }

            // Find nearest visible enemy
            let nearest_enemy = world
                .known_enemies
                .iter()
                .min_by_key(|e| army.position.distance(&e.position));

            if let Some(enemy) = nearest_enemy {
                let distance = army.position.distance(&enemy.position);

                // Priority 2: Favorable engagement (enemy nearby and we have advantage)
                if distance <= 3 && has_advantage(army, enemy) {
                    orders.push(AIOrder::Attack {
                        army: army.id,
                        target: enemy.id,
                    });
                    continue;
                }

                // Priority 3: Retreat (enemy nearby and we're outnumbered)
                if distance <= 3 && !has_advantage(army, enemy) {
                    orders.push(AIOrder::Retreat { army: army.id });
                    continue;
                }
            }

            // Priority 4: Scout unexplored (if we have good supplies)
            if army.supplies_days > 7.0 {
                if let Some(unexplored) = find_nearest_unexplored(army.position, &world.explored_hexes, map) {
                    // Deploy scout mission instead of moving army
                    orders.push(AIOrder::DeployScout {
                        from_army: army.id,
                        target: unexplored,
                    });
                }
            }

            // Priority 5: Advance toward enemy depot (if known)
            if let Some(enemy_depot) = world.enemy_depot {
                if army.supplies_days > 5.0 && army.position != enemy_depot {
                    orders.push(AIOrder::MoveTo {
                        army: army.id,
                        target: enemy_depot,
                    });
                    continue;
                }
            }

            // Priority 6: Hold position
            orders.push(AIOrder::Hold { army: army.id });
        }

        orders
    }
}
```

**Step 3: Verify it compiles**

Run: `cargo build --bin campaign_ai`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/bin/campaign_ai.rs
git commit -m "feat(campaign-ai): implement decide() with priority-based logic"
```

---

## Task 5: Implement execute() Method

**Files:**
- Modify: `src/bin/campaign_ai.rs`

**Step 1: Add execute method to FactionAI**

```rust
impl FactionAI {
    // ... existing methods ...

    /// Execute orders by modifying game state
    fn execute(
        &mut self,
        orders: Vec<AIOrder>,
        state: &mut CampaignState,
        scouts: &mut ScoutSystem,
        verbose: bool,
    ) {
        for order in orders {
            match order {
                AIOrder::MoveTo { army, target } => {
                    if let Some(a) = state.get_army_mut(army) {
                        a.order_move_to(target, &state.map.clone());
                        a.stance = ArmyStance::Aggressive;
                        if verbose {
                            println!("  {} advances toward ({}, {})", a.name, target.q, target.r);
                        }
                    }
                }
                AIOrder::Attack { army, target: _ } => {
                    if let Some(a) = state.get_army_mut(army) {
                        // Set aggressive stance, movement will handle engagement
                        a.stance = ArmyStance::Aggressive;
                        if verbose {
                            println!("  {} set to aggressive, seeking engagement", a.name);
                        }
                    }
                }
                AIOrder::Retreat { army } => {
                    if let Some(a) = state.get_army_mut(army) {
                        a.order_move_to(self.depot, &state.map.clone());
                        a.stance = ArmyStance::Evasive;
                        if verbose {
                            println!("  {} retreating to depot", a.name);
                        }
                    }
                }
                AIOrder::Hold { army } => {
                    if let Some(a) = state.get_army_mut(army) {
                        a.order_halt();
                        a.stance = ArmyStance::Defensive;
                        if verbose {
                            println!("  {} holding position", a.name);
                        }
                    }
                }
                AIOrder::DeployScout { from_army, target } => {
                    if let Some(a) = state.get_army(from_army) {
                        // Only deploy if we don't have too many scouts
                        if self.scout_ids.len() < 3 {
                            let scout_id = scouts.deploy_scout(a);
                            if let Some(s) = scouts.get_scout_mut(scout_id) {
                                s.assign_recon(target, &state.map);
                            }
                            self.scout_ids.push(scout_id);
                            if verbose {
                                println!("  Scout deployed from {} to ({}, {})", a.name, target.q, target.r);
                            }
                        }
                    }
                }
            }
        }
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build --bin campaign_ai`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/bin/campaign_ai.rs
git commit -m "feat(campaign-ai): implement execute() method for order execution"
```

---

## Task 6: Implement Victory Conditions

**Files:**
- Modify: `src/bin/campaign_ai.rs`

**Step 1: Add victory checking enum and functions**

```rust
/// Victory condition result
#[derive(Debug, Clone)]
enum Victory {
    Military { winner: PolityId },
    DepotCapture { winner: PolityId },
    Stalemate,
}

/// Track depot occupation for capture victory
struct DepotOccupation {
    blue_depot_occupied_days: u32,
    red_depot_occupied_days: u32,
}

impl DepotOccupation {
    fn new() -> Self {
        Self {
            blue_depot_occupied_days: 0,
            red_depot_occupied_days: 0,
        }
    }

    fn update(&mut self, state: &CampaignState, blue_depot: HexCoord, red_depot: HexCoord) {
        // Check if red army is on blue depot
        let red_on_blue = state.armies.iter().any(|a| {
            a.faction == PolityId(2) && a.position == blue_depot && a.unit_count > 10
        });
        // Check if blue army is on red depot
        let blue_on_red = state.armies.iter().any(|a| {
            a.faction == PolityId(1) && a.position == red_depot && a.unit_count > 10
        });

        if red_on_blue {
            self.blue_depot_occupied_days += 1;
        } else {
            self.blue_depot_occupied_days = 0;
        }

        if blue_on_red {
            self.red_depot_occupied_days += 1;
        } else {
            self.red_depot_occupied_days = 0;
        }
    }
}

/// Check all victory conditions
fn check_victory(
    state: &CampaignState,
    occupation: &DepotOccupation,
    day: u32,
    max_days: u32,
) -> Option<Victory> {
    // Military victory: enemy has no armies with >10 units
    let blue_units: u32 = state
        .armies
        .iter()
        .filter(|a| a.faction == PolityId(1))
        .map(|a| a.unit_count)
        .sum();
    let red_units: u32 = state
        .armies
        .iter()
        .filter(|a| a.faction == PolityId(2))
        .map(|a| a.unit_count)
        .sum();

    if blue_units <= 10 {
        return Some(Victory::Military { winner: PolityId(2) });
    }
    if red_units <= 10 {
        return Some(Victory::Military { winner: PolityId(1) });
    }

    // Depot capture: 3 consecutive days
    if occupation.red_depot_occupied_days >= 3 {
        return Some(Victory::DepotCapture { winner: PolityId(1) });
    }
    if occupation.blue_depot_occupied_days >= 3 {
        return Some(Victory::DepotCapture { winner: PolityId(2) });
    }

    // Stalemate
    if day >= max_days {
        return Some(Victory::Stalemate);
    }

    None
}
```

**Step 2: Verify it compiles**

Run: `cargo build --bin campaign_ai`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/bin/campaign_ai.rs
git commit -m "feat(campaign-ai): add victory condition checking"
```

---

## Task 7: Implement Statistics Tracking

**Files:**
- Modify: `src/bin/campaign_ai.rs`

**Step 1: Add statistics struct**

```rust
/// Track simulation statistics
struct SimulationStats {
    total_battles: u32,
    blue_casualties: u32,
    red_casualties: u32,
    blue_hexes_explored: u32,
    red_hexes_explored: u32,
}

impl SimulationStats {
    fn new() -> Self {
        Self {
            total_battles: 0,
            blue_casualties: 0,
            red_casualties: 0,
            blue_hexes_explored: 0,
            red_hexes_explored: 0,
        }
    }

    fn record_battle(&mut self, blue_cas: u32, red_cas: u32) {
        self.total_battles += 1;
        self.blue_casualties += blue_cas;
        self.red_casualties += red_cas;
    }

    fn update_exploration(&mut self, visibility: &VisibilitySystem) {
        if let Some(fv) = visibility.get_faction(PolityId(1)) {
            self.blue_hexes_explored = fv.explored_count;
        }
        if let Some(fv) = visibility.get_faction(PolityId(2)) {
            self.red_hexes_explored = fv.explored_count;
        }
    }

    fn print(&self, victory_day: u32) {
        println!("Statistics:");
        println!("  Total battles: {}", self.total_battles);
        println!("  Blue casualties: {}", self.blue_casualties);
        println!("  Red casualties: {}", self.red_casualties);
        println!("  Hexes explored: Blue {}, Red {}", self.blue_hexes_explored, self.red_hexes_explored);
        println!("  Victory on day: {}", victory_day);
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build --bin campaign_ai`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/bin/campaign_ai.rs
git commit -m "feat(campaign-ai): add statistics tracking"
```

---

## Task 8: Implement Main Simulation Loop

**Files:**
- Modify: `src/bin/campaign_ai.rs`

**Step 1: Replace main() with full simulation**

```rust
fn main() {
    let args = Args::parse();

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║           ARC CITADEL: AI CAMPAIGN BATTLE                     ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    // Setup
    let map = CampaignMap::generate_simple(30, 30, args.seed);
    let blue_depot = HexCoord::new(2, 2);
    let red_depot = HexCoord::new(27, 27);

    println!("[Setup]");
    println!("Map: 30x30 hexes, seed {}", args.seed);
    println!("Faction 1 (Blue): 2 armies, depot at ({}, {})", blue_depot.q, blue_depot.r);
    println!("Faction 2 (Red): 2 armies, depot at ({}, {})", red_depot.q, red_depot.r);
    println!();

    // Initialize systems
    let mut state = CampaignState::new(map.clone());
    let mut supply = SupplySystem::new();
    let mut weather = RegionalWeather::new();
    let mut visibility = VisibilitySystem::new();
    let mut scouts = ScoutSystem::new();

    // Register factions
    visibility.register_faction(PolityId(1));
    visibility.register_faction(PolityId(2));

    // Create depots
    supply.create_depot(blue_depot, PolityId(1));
    supply.create_depot(red_depot, PolityId(2));

    // Create Blue armies
    let blue_army1 = state.spawn_army("Blue Legion".to_string(), PolityId(1), blue_depot);
    state.get_army_mut(blue_army1).unwrap().unit_count = 300;
    state.get_army_mut(blue_army1).unwrap().stance = ArmyStance::Aggressive;
    supply.register_army(blue_army1);
    supply.get_army_supply_mut(blue_army1).unwrap().foraging = true;

    let blue_army2 = state.spawn_army("Blue Guard".to_string(), PolityId(1), HexCoord::new(5, 5));
    state.get_army_mut(blue_army2).unwrap().unit_count = 200;
    supply.register_army(blue_army2);
    supply.get_army_supply_mut(blue_army2).unwrap().foraging = true;

    // Create Red armies
    let red_army1 = state.spawn_army("Red Host".to_string(), PolityId(2), red_depot);
    state.get_army_mut(red_army1).unwrap().unit_count = 350;
    state.get_army_mut(red_army1).unwrap().stance = ArmyStance::Aggressive;
    supply.register_army(red_army1);
    supply.get_army_supply_mut(red_army1).unwrap().foraging = true;

    let red_army2 = state.spawn_army("Red Vanguard".to_string(), PolityId(2), HexCoord::new(25, 25));
    state.get_army_mut(red_army2).unwrap().unit_count = 150;
    supply.register_army(red_army2);
    supply.get_army_supply_mut(red_army2).unwrap().foraging = true;

    // Create AI controllers
    let mut blue_ai = FactionAI::new(PolityId(1), blue_depot);
    let mut red_ai = FactionAI::new(PolityId(2), red_depot);

    // Tracking
    let mut stats = SimulationStats::new();
    let mut occupation = DepotOccupation::new();

    // Main loop
    for day in 1..=args.max_days {
        println!("[Day {}]", day);

        // Update weather
        weather.update(1.0, day, args.seed.wrapping_mul(day as u64));
        let current_weather = weather.global_weather.current_weather;

        // Update visibility
        let scout_armies = scouts.armies_with_scouts();
        visibility.update(&state.armies, &scout_armies, &map, &weather, state.current_day);

        // 1. Each AI perceives
        let blue_world = blue_ai.perceive(&state, &visibility, &supply, red_depot);
        let red_world = red_ai.perceive(&state, &visibility, &supply, blue_depot);

        if args.verbose {
            println!("Blue AI: sees {} enemies, knows {} hexes",
                blue_world.known_enemies.len(), blue_world.explored_hexes.len());
            println!("Red AI: sees {} enemies, knows {} hexes",
                red_world.known_enemies.len(), red_world.explored_hexes.len());
        }

        // 2. Each AI decides
        let blue_orders = blue_ai.decide(&blue_world, &map);
        let red_orders = red_ai.decide(&red_world, &map);

        // 3. Execute orders
        if args.verbose {
            println!("Blue AI orders:");
        }
        blue_ai.execute(blue_orders, &mut state, &mut scouts, args.verbose);
        if args.verbose {
            println!("Red AI orders:");
        }
        red_ai.execute(red_orders, &mut state, &mut scouts, args.verbose);

        // 4. Tick systems
        supply.tick(&mut state.armies, &map, 1.0);
        scouts.tick(&state.armies, &map, 1.0, state.current_day);
        let events = campaign_tick(&mut state, 1.0);

        // Handle battles
        for event in events {
            if let arc_citadel::campaign::CampaignEvent::ArmiesEngaged { army_a, army_b, position } = event {
                let a_name = state.get_army(army_a).map(|a| a.name.clone()).unwrap_or_default();
                let b_name = state.get_army(army_b).map(|a| a.name.clone()).unwrap_or_default();

                println!("BATTLE at ({}, {}): {} vs {}", position.q, position.r, a_name, b_name);

                // Resolve battle
                let (mut attacker, mut defender) = {
                    let a = state.get_army(army_a).unwrap().clone();
                    let b = state.get_army(army_b).unwrap().clone();
                    (a, b)
                };

                let result = resolve_battle(&mut attacker, &mut defender, &map, current_weather, 10);

                // Determine which is blue/red for stats
                let (blue_cas, red_cas) = if attacker.faction == PolityId(1) {
                    (result.attacker_casualties, result.defender_casualties)
                } else {
                    (result.defender_casualties, result.attacker_casualties)
                };
                stats.record_battle(blue_cas, red_cas);

                println!("  Result: {:?}", result.outcome);

                // Apply results
                if let Some(a) = state.get_army_mut(army_a) {
                    a.unit_count = attacker.unit_count;
                    a.morale = attacker.morale;
                }
                if let Some(b) = state.get_army_mut(army_b) {
                    b.unit_count = defender.unit_count;
                    b.morale = defender.morale;
                }

                // Apply retreat
                match result.outcome {
                    BattleOutcome::AttackerVictory => {
                        if let Some(b) = state.get_army_mut(army_b) {
                            apply_retreat(b, position, &map);
                            println!("  {} retreats", b_name);
                        }
                    }
                    BattleOutcome::DefenderVictory => {
                        if let Some(a) = state.get_army_mut(army_a) {
                            apply_retreat(a, position, &map);
                            println!("  {} retreats", a_name);
                        }
                    }
                    BattleOutcome::Draw => {
                        if let Some(a) = state.get_army_mut(army_a) {
                            apply_retreat(a, position, &map);
                        }
                        if let Some(b) = state.get_army_mut(army_b) {
                            apply_retreat(b, position, &map);
                        }
                        println!("  Both armies retreat");
                    }
                    BattleOutcome::Ongoing => {}
                }
            }
        }

        // 5. Update tracking
        occupation.update(&state, blue_depot, red_depot);
        stats.update_exploration(&visibility);

        // 6. Check victory
        if let Some(victory) = check_victory(&state, &occupation, day, args.max_days) {
            println!();
            println!("═══════════════════════════════════════════════════════════════");
            match victory {
                Victory::Military { winner } => {
                    let name = if winner == PolityId(1) { "Blue" } else { "Red" };
                    println!("VICTORY: Faction {} ({}) - Military Victory on Day {}", winner.0, name, day);
                }
                Victory::DepotCapture { winner } => {
                    let name = if winner == PolityId(1) { "Blue" } else { "Red" };
                    println!("VICTORY: Faction {} ({}) - Depot Capture on Day {}", winner.0, name, day);
                }
                Victory::Stalemate => {
                    println!("STALEMATE: No victor after {} days", day);
                }
            }
            println!("═══════════════════════════════════════════════════════════════");
            println!();
            stats.print(day);
            return;
        }

        println!();
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build --bin campaign_ai`
Expected: Compiles successfully

**Step 3: Test run**

Run: `cargo run --bin campaign_ai -- --seed 42`
Expected: Simulation runs and produces output showing AI decisions and battles

**Step 4: Commit**

```bash
git add src/bin/campaign_ai.rs
git commit -m "feat(campaign-ai): implement main simulation loop"
```

---

## Task 9: Add Smarter Scout Logic

**Files:**
- Modify: `src/bin/campaign_ai.rs`

**Step 1: Improve scout deployment in decide()**

In the decide() method, replace the scout deployment logic with smarter targeting:

```rust
// Priority 4: Scout unexplored (if we have good supplies and few scouts deployed)
// Only large armies deploy scouts
if army.unit_count > 100 && army.supplies_days > 7.0 {
    // Target unexplored areas toward enemy depot if known, otherwise expand outward
    let scout_target = if let Some(enemy_depot) = world.enemy_depot {
        // Scout toward enemy
        let mid = HexCoord::new(
            (army.position.q + enemy_depot.q) / 2,
            (army.position.r + enemy_depot.r) / 2,
        );
        if !world.explored_hexes.contains(&mid) {
            Some(mid)
        } else {
            find_nearest_unexplored(army.position, &world.explored_hexes, map)
        }
    } else {
        find_nearest_unexplored(army.position, &world.explored_hexes, map)
    };

    if let Some(target) = scout_target {
        orders.push(AIOrder::DeployScout {
            from_army: army.id,
            target,
        });
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build --bin campaign_ai`
Expected: Compiles successfully

**Step 3: Test run with verbose**

Run: `cargo run --bin campaign_ai -- --seed 42 --verbose`
Expected: See scout deployment decisions

**Step 4: Commit**

```bash
git add src/bin/campaign_ai.rs
git commit -m "feat(campaign-ai): improve scout targeting toward enemy depot"
```

---

## Task 10: Final Polish and Testing

**Files:**
- Modify: `src/bin/campaign_ai.rs`

**Step 1: Add more informative output**

Update main loop to show army status periodically:

```rust
// At end of day, show status every 10 days
if day % 10 == 0 || args.verbose {
    println!("Army Status:");
    for army in &state.armies {
        let supply_status = supply.get_army_supply(army.id)
            .map(|s| format!("{:.1} days", s.days_until_starvation(army.unit_count)))
            .unwrap_or_default();
        let faction = if army.faction == PolityId(1) { "Blue" } else { "Red" };
        println!("  {} [{}]: {} units, {:.0}% morale, {} at ({},{})",
            army.name, faction, army.unit_count, army.morale * 100.0,
            supply_status, army.position.q, army.position.r);
    }
}
```

**Step 2: Test with different seeds**

Run these commands to verify varied outcomes:
```bash
cargo run --bin campaign_ai -- --seed 1
cargo run --bin campaign_ai -- --seed 42
cargo run --bin campaign_ai -- --seed 999
```

Expected: Different battle outcomes and victory conditions

**Step 3: Test with max-days**

Run: `cargo run --bin campaign_ai -- --seed 42 --max-days 50`
Expected: Stalemate if no victory in 50 days

**Step 4: Commit**

```bash
git add src/bin/campaign_ai.rs
git commit -m "feat(campaign-ai): add periodic status output and polish"
```

---

## Verification

After completing all tasks, run these verification commands:

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

**Expected Behaviors:**
- Two AI factions compete using only visible information
- Scouts explore and gather intel before armies advance
- Armies retreat when low on supplies
- Battles occur when armies meet and one has advantage
- Game ends with clear victory or stalemate
