//! Campaign AI Agent
//! Two AI factions compete in a campaign simulation

use arc_citadel::campaign::{
    apply_retreat, campaign_tick, resolve_battle, ArmyId, ArmyStance, BattleOutcome, CampaignMap,
    CampaignState, HexCoord, RegionalWeather, ScoutId, ScoutSystem, SupplySystem, VisibilitySystem,
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

/// What the AI knows about an army (own or enemy)
#[derive(Debug, Clone)]
struct ArmyInfo {
    id: ArmyId,
    position: HexCoord,
    unit_count: u32,
    morale: f32,
    supplies_days: f32,
}

/// World model from a faction's perspective (fog of war)
#[derive(Debug)]
struct FactionWorldModel {
    my_armies: Vec<ArmyInfo>,
    known_enemies: Vec<ArmyInfo>,
    explored_hexes: HashSet<HexCoord>,
    enemy_depot: Option<HexCoord>,
}

/// Orders the AI can issue
#[derive(Debug, Clone)]
enum AIOrder {
    MoveTo { army: ArmyId, target: HexCoord },
    Attack { army: ArmyId },
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

    fn perceive(
        &self,
        state: &CampaignState,
        visibility: &VisibilitySystem,
        supply: &SupplySystem,
        enemy_depot: HexCoord,
    ) -> FactionWorldModel {
        let fv = visibility.get_faction(self.faction);

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
                }
            })
            .collect();

        // Use visibility system + direct distance check for nearby enemies
        // This ensures we detect enemies within tactical range even in low-visibility terrain
        let mut known_enemies: Vec<ArmyInfo> = visibility
            .visible_enemies(self.faction, &state.armies)
            .into_iter()
            .map(|a| ArmyInfo {
                id: a.id,
                position: a.position,
                unit_count: a.unit_count,
                morale: a.morale,
                supplies_days: 10.0,
            })
            .collect();

        // Also detect enemies within 5 hexes of any of our armies (tactical proximity)
        for my_army in &my_armies {
            for enemy in state.armies.iter().filter(|a| a.faction != self.faction) {
                let dist = my_army.position.distance(&enemy.position);
                if dist <= 5 && !known_enemies.iter().any(|e| e.id == enemy.id) {
                    known_enemies.push(ArmyInfo {
                        id: enemy.id,
                        position: enemy.position,
                        unit_count: enemy.unit_count,
                        morale: enemy.morale,
                        supplies_days: 10.0,
                    });
                }
            }
        }

        let explored_hexes: HashSet<HexCoord> = fv
            .map(|f| f.intel.keys().copied().collect())
            .unwrap_or_default();

        let enemy_depot_known = explored_hexes.contains(&enemy_depot);

        FactionWorldModel {
            my_armies,
            known_enemies,
            explored_hexes,
            enemy_depot: if enemy_depot_known { Some(enemy_depot) } else { None },
        }
    }

    fn decide(&self, world: &FactionWorldModel, map: &CampaignMap, enemy_depot: HexCoord) -> Vec<AIOrder> {
        let mut orders = Vec::new();

        for army in &world.my_armies {
            let nearest_enemy = world
                .known_enemies
                .iter()
                .min_by_key(|e| army.position.distance(&e.position));

            // CRITICAL: Check for adjacent enemies BEFORE supply check
            // If enemy is right next to us, we fight regardless of supplies
            if let Some(enemy) = nearest_enemy {
                let distance = army.position.distance(&enemy.position);

                // Adjacent or same hex - always engage (no retreat from contact)
                // We MOVE TO the enemy position to enter the same hex and trigger battle
                if distance <= 1 {
                    orders.push(AIOrder::MoveTo {
                        army: army.id,
                        target: enemy.position,
                    });
                    continue;
                }

                // PURSUIT: If enemy is routed (low morale), chase aggressively within 6 hexes
                // This is how battles become decisive - destroy retreating armies
                if enemy_is_routed(enemy) && distance <= 6 {
                    orders.push(AIOrder::MoveTo {
                        army: army.id,
                        target: enemy.position,
                    });
                    continue;
                }

                // Chase weak enemies (low unit count or morale) within 4 hexes
                if enemy_is_weak(enemy) && distance <= 4 && army.morale > 0.4 {
                    orders.push(AIOrder::MoveTo {
                        army: army.id,
                        target: enemy.position,
                    });
                    continue;
                }

                // Enemy is close (2-3 hexes) - chase them if we have any advantage
                // or if we have reasonable supplies
                if distance <= 3 && (has_advantage(army, enemy) || army.supplies_days > 2.0) {
                    orders.push(AIOrder::MoveTo {
                        army: army.id,
                        target: enemy.position,
                    });
                    continue;
                }
            }

            // Now check supplies - only retreat if critically low AND not in contact
            // Threshold lowered from 3.0 to 1.5 days
            if army.supplies_days < 1.5 {
                // Only retreat if depot is reachable (within reasonable distance)
                let depot_distance = army.position.distance(&self.depot);
                if depot_distance <= 10 {
                    orders.push(AIOrder::Retreat { army: army.id });
                    continue;
                }
                // If depot too far, keep fighting - we'll die anyway
            }

            // Check if enemy visible but not adjacent (handled above)
            if let Some(enemy) = nearest_enemy {
                let distance = army.position.distance(&enemy.position);

                // If we have advantage, pursue even at medium distance
                if has_advantage(army, enemy) && distance <= 5 {
                    orders.push(AIOrder::MoveTo {
                        army: army.id,
                        target: enemy.position,
                    });
                    continue;
                }

                // No advantage and enemy visible but not too close - continue toward objective
                // (Don't retreat just because we see an enemy)
            }

            // Deploy scouts if we have spare capacity
            if army.unit_count > 100 && army.supplies_days > 7.0 {
                if let Some(scout_target) = find_scout_target(army.position, &world.explored_hexes, map, enemy_depot) {
                    orders.push(AIOrder::DeployScout {
                        from_army: army.id,
                        target: scout_target,
                    });
                    // Don't continue - army can still move while scouts deploy
                }
            }

            // Advance toward enemy depot if known
            if let Some(enemy_depot) = world.enemy_depot {
                if army.supplies_days > 5.0 && army.position != enemy_depot {
                    orders.push(AIOrder::MoveTo {
                        army: army.id,
                        target: enemy_depot,
                    });
                    continue;
                }
            }

            // If depot not known, advance toward enemy side of map to explore
            // Blue (PolityId 1) goes toward (17, 17), Red (PolityId 2) goes toward (2, 2)
            if world.enemy_depot.is_none() && army.supplies_days > 5.0 {
                // Keep advancing until we find the enemy depot
                let target = enemy_depot;
                if army.position.distance(&target) > 2 {
                    orders.push(AIOrder::MoveTo {
                        army: army.id,
                        target,
                    });
                    continue;
                }
            }

            // Even if at enemy depot (holding it), don't need to hold order
            orders.push(AIOrder::Hold { army: army.id });
        }

        orders
    }

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
                    let map_clone = state.map.clone();
                    if let Some(a) = state.get_army_mut(army) {
                        a.order_move_to(target, &map_clone);
                        a.stance = ArmyStance::Aggressive;
                        if verbose {
                            println!("  {} advances toward ({}, {})", a.name, target.q, target.r);
                        }
                    }
                }
                AIOrder::Attack { army } => {
                    if let Some(a) = state.get_army_mut(army) {
                        a.stance = ArmyStance::Aggressive;
                        if verbose {
                            println!("  {} set to aggressive, seeking engagement", a.name);
                        }
                    }
                }
                AIOrder::Retreat { army } => {
                    let depot = self.depot;
                    let map_clone = state.map.clone();
                    if let Some(a) = state.get_army_mut(army) {
                        a.order_move_to(depot, &map_clone);
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
                        if self.scout_ids.len() < 3 {
                            let scout_id = scouts.deploy_scout(a);
                            let map_clone = state.map.clone();
                            if let Some(s) = scouts.get_scout_mut(scout_id) {
                                s.assign_recon(target, &map_clone);
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

fn has_advantage(my_army: &ArmyInfo, enemy: &ArmyInfo) -> bool {
    let my_strength = my_army.unit_count as f32 * my_army.morale;
    let enemy_strength = enemy.unit_count as f32 * enemy.morale;
    my_strength > enemy_strength * 1.2
}

/// Check if enemy is routed (broken morale) and should be pursued
fn enemy_is_routed(enemy: &ArmyInfo) -> bool {
    enemy.morale < 0.3
}

/// Check if enemy is weak and vulnerable to attack
fn enemy_is_weak(enemy: &ArmyInfo) -> bool {
    enemy.unit_count < 100 || enemy.morale < 0.5
}

fn find_scout_target(
    from: HexCoord,
    explored: &HashSet<HexCoord>,
    map: &CampaignMap,
    enemy_depot: HexCoord,
) -> Option<HexCoord> {
    // Prioritize unexplored hexes in the direction of enemy depot
    let mut candidates = Vec::new();

    for radius in 1..=15 {
        for q in (from.q - radius)..=(from.q + radius) {
            for r in (from.r - radius)..=(from.r + radius) {
                let coord = HexCoord::new(q, r);
                if from.distance(&coord) <= radius
                    && map.contains(&coord)
                    && !explored.contains(&coord)
                {
                    // Score by proximity to enemy depot (lower is better)
                    let depot_dist = coord.distance(&enemy_depot);
                    candidates.push((coord, depot_dist));
                }
            }
        }

        // Once we have candidates at this radius, pick best one
        if !candidates.is_empty() {
            candidates.sort_by_key(|(_, dist)| *dist);
            return Some(candidates[0].0);
        }
    }
    None
}

#[derive(Debug, Clone)]
enum Victory {
    Military { winner: PolityId },
    DepotCapture { winner: PolityId },
    Stalemate,
}

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
        let red_on_blue = state.armies.iter().any(|a| {
            a.faction == PolityId(2) && a.position == blue_depot && a.unit_count > 10
        });
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

fn check_victory(
    state: &CampaignState,
    occupation: &DepotOccupation,
    day: u32,
    max_days: u32,
) -> Option<Victory> {
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

    if occupation.red_depot_occupied_days >= 3 {
        return Some(Victory::DepotCapture { winner: PolityId(1) });
    }
    if occupation.blue_depot_occupied_days >= 3 {
        return Some(Victory::DepotCapture { winner: PolityId(2) });
    }

    if day >= max_days {
        return Some(Victory::Stalemate);
    }

    None
}

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

fn main() {
    let args = Args::parse();

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║           ARC CITADEL: AI CAMPAIGN BATTLE                     ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    let map = CampaignMap::generate_simple(20, 20, args.seed);
    let blue_depot = HexCoord::new(2, 2);
    let red_depot = HexCoord::new(17, 17);

    println!("[Setup]");
    println!("Map: 20x20 hexes, seed {}", args.seed);
    println!("Faction 1 (Blue): 2 armies, depot at ({}, {})", blue_depot.q, blue_depot.r);
    println!("Faction 2 (Red): 2 armies, depot at ({}, {})", red_depot.q, red_depot.r);
    println!();

    let mut state = CampaignState::new(map.clone());
    let mut supply = SupplySystem::new();
    let mut weather = RegionalWeather::new();
    let mut visibility = VisibilitySystem::new();
    let mut scouts = ScoutSystem::new();

    visibility.register_faction(PolityId(1));
    visibility.register_faction(PolityId(2));

    supply.create_depot(blue_depot, PolityId(1));
    supply.create_depot(red_depot, PolityId(2));

    let blue_army1 = state.spawn_army("Blue Legion".to_string(), PolityId(1), blue_depot);
    state.get_army_mut(blue_army1).unwrap().unit_count = 300;
    state.get_army_mut(blue_army1).unwrap().stance = ArmyStance::Aggressive;
    supply.register_army(blue_army1);
    supply.get_army_supply_mut(blue_army1).unwrap().foraging = true;
    supply.get_army_supply_mut(blue_army1).unwrap().supplies = 30.0; // Extra supplies for campaign
    supply.get_army_supply_mut(blue_army1).unwrap().max_supplies = 40.0;

    let blue_army2 = state.spawn_army("Blue Guard".to_string(), PolityId(1), HexCoord::new(5, 5));
    state.get_army_mut(blue_army2).unwrap().unit_count = 200;
    supply.register_army(blue_army2);
    supply.get_army_supply_mut(blue_army2).unwrap().foraging = true;
    supply.get_army_supply_mut(blue_army2).unwrap().supplies = 30.0;
    supply.get_army_supply_mut(blue_army2).unwrap().max_supplies = 40.0;

    let red_army1 = state.spawn_army("Red Host".to_string(), PolityId(2), red_depot);
    state.get_army_mut(red_army1).unwrap().unit_count = 350;
    state.get_army_mut(red_army1).unwrap().stance = ArmyStance::Aggressive;
    supply.register_army(red_army1);
    supply.get_army_supply_mut(red_army1).unwrap().foraging = true;
    supply.get_army_supply_mut(red_army1).unwrap().supplies = 30.0;
    supply.get_army_supply_mut(red_army1).unwrap().max_supplies = 40.0;

    let red_army2 = state.spawn_army("Red Vanguard".to_string(), PolityId(2), HexCoord::new(15, 15));
    state.get_army_mut(red_army2).unwrap().unit_count = 150;
    supply.register_army(red_army2);
    supply.get_army_supply_mut(red_army2).unwrap().foraging = true;
    supply.get_army_supply_mut(red_army2).unwrap().supplies = 30.0;
    supply.get_army_supply_mut(red_army2).unwrap().max_supplies = 40.0;

    let mut blue_ai = FactionAI::new(PolityId(1), blue_depot);
    let mut red_ai = FactionAI::new(PolityId(2), red_depot);

    let mut stats = SimulationStats::new();
    let mut occupation = DepotOccupation::new();

    for day in 1..=args.max_days {
        if args.verbose || day % 10 == 0 {
            println!("[Day {}]", day);
        }

        weather.update(1.0, day, args.seed.wrapping_mul(day as u64));
        let current_weather = weather.global_weather.current_weather;

        let scout_armies = scouts.armies_with_scouts();
        visibility.update(&state.armies, &scout_armies, &map, &weather, state.current_day);

        let blue_world = blue_ai.perceive(&state, &visibility, &supply, red_depot);
        let red_world = red_ai.perceive(&state, &visibility, &supply, blue_depot);

        if args.verbose {
            // Print army positions with morale
            for army_info in &blue_world.my_armies {
                let morale = state.get_army(army_info.id).map(|a| a.morale).unwrap_or(0.0);
                println!("  Blue {}: at ({}, {}), {} units, morale {:.2}",
                    state.get_army(army_info.id).map(|a| a.name.as_str()).unwrap_or("?"),
                    army_info.position.q, army_info.position.r, army_info.unit_count, morale);
            }
            for army_info in &red_world.my_armies {
                let morale = state.get_army(army_info.id).map(|a| a.morale).unwrap_or(0.0);
                println!("  Red {}: at ({}, {}), {} units, morale {:.2}",
                    state.get_army(army_info.id).map(|a| a.name.as_str()).unwrap_or("?"),
                    army_info.position.q, army_info.position.r, army_info.unit_count, morale);
            }

            if !blue_world.known_enemies.is_empty() || !red_world.known_enemies.is_empty() {
                println!("  Blue sees {} enemies, Red sees {} enemies",
                    blue_world.known_enemies.len(), red_world.known_enemies.len());
            }
        }

        let blue_orders = blue_ai.decide(&blue_world, &map, red_depot);
        let red_orders = red_ai.decide(&red_world, &map, blue_depot);

        if args.verbose {
            println!("Blue AI orders:");
        }
        blue_ai.execute(blue_orders, &mut state, &mut scouts, args.verbose);
        if args.verbose {
            println!("Red AI orders:");
        }
        red_ai.execute(red_orders, &mut state, &mut scouts, args.verbose);

        supply.tick(&mut state.armies, &map, 1.0);
        scouts.tick(&state.armies, &map, 1.0, state.current_day);

        // Morale recovery: armies recover 0.05 morale per day, faster at depot (0.15)
        const MORALE_RECOVERY_RATE: f32 = 0.05;
        const DEPOT_MORALE_BONUS: f32 = 0.10;
        for army in &mut state.armies {
            let at_depot = (army.faction == PolityId(1) && army.position == blue_depot)
                || (army.faction == PolityId(2) && army.position == red_depot);
            let recovery = if at_depot {
                MORALE_RECOVERY_RATE + DEPOT_MORALE_BONUS
            } else {
                MORALE_RECOVERY_RATE
            };
            army.morale = (army.morale + recovery).min(1.0);
        }

        let events = campaign_tick(&mut state, 1.0);

        // Also check for armies already at same position that should fight
        // This handles the case where armies are already co-located
        let mut pending_battles: Vec<(ArmyId, ArmyId, HexCoord)> = Vec::new();
        let positions: std::collections::HashSet<_> = state.armies.iter().map(|a| a.position).collect();
        for position in &positions {
            let armies_here: Vec<_> = state.armies.iter()
                .filter(|a| a.position == *position)
                .collect();
            for (i, a) in armies_here.iter().enumerate() {
                for b in armies_here.iter().skip(i + 1) {
                    // Different factions and both have reasonable morale
                    if a.faction != b.faction && a.morale > 0.2 && b.morale > 0.2 {
                        // Check they're not already marked as engaged
                        if a.engaged_with != Some(b.id) && b.engaged_with != Some(a.id) {
                            pending_battles.push((a.id, b.id, *position));
                        }
                    }
                }
            }
        }

        // Process pending co-located battles
        for (army_a, army_b, position) in pending_battles {
            let a_name = state.get_army(army_a).map(|a| a.name.clone()).unwrap_or_default();
            let b_name = state.get_army(army_b).map(|a| a.name.clone()).unwrap_or_default();

            println!("BATTLE at ({}, {}): {} vs {}", position.q, position.r, a_name, b_name);

            let (mut attacker, mut defender) = {
                let a = state.get_army(army_a).unwrap().clone();
                let b = state.get_army(army_b).unwrap().clone();
                (a, b)
            };

            let result = resolve_battle(&mut attacker, &mut defender, &map, current_weather, 10);

            let (blue_cas, red_cas) = if attacker.faction == PolityId(1) {
                (result.attacker_casualties, result.defender_casualties)
            } else {
                (result.defender_casualties, result.attacker_casualties)
            };
            stats.record_battle(blue_cas, red_cas);

            println!("  Result: {:?}", result.outcome);

            if let Some(a) = state.get_army_mut(army_a) {
                a.unit_count = attacker.unit_count;
                a.morale = attacker.morale;
            }
            if let Some(b) = state.get_army_mut(army_b) {
                b.unit_count = defender.unit_count;
                b.morale = defender.morale;
            }

            // Pursuit casualties: 20% on fleeing army
            const PURSUIT_CASUALTY_RATE: f32 = 0.20;

            match result.outcome {
                BattleOutcome::AttackerVictory => {
                    if let Some(b) = state.get_army_mut(army_b) {
                        let pursuit_cas = (b.unit_count as f32 * PURSUIT_CASUALTY_RATE) as u32;
                        b.unit_count = b.unit_count.saturating_sub(pursuit_cas);
                        let (blue, red) = if attacker.faction == PolityId(1) {
                            (0, pursuit_cas)
                        } else {
                            (pursuit_cas, 0)
                        };
                        stats.record_battle(blue, red);
                        apply_retreat(b, position, &map);
                        println!("  {} retreats (pursuit: -{} units)", b_name, pursuit_cas);
                    }
                }
                BattleOutcome::DefenderVictory => {
                    if let Some(a) = state.get_army_mut(army_a) {
                        let pursuit_cas = (a.unit_count as f32 * PURSUIT_CASUALTY_RATE) as u32;
                        a.unit_count = a.unit_count.saturating_sub(pursuit_cas);
                        let (blue, red) = if attacker.faction == PolityId(1) {
                            (pursuit_cas, 0)
                        } else {
                            (0, pursuit_cas)
                        };
                        stats.record_battle(blue, red);
                        apply_retreat(a, position, &map);
                        println!("  {} retreats (pursuit: -{} units)", a_name, pursuit_cas);
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

            // Clear engaged_with so armies can fight again
            if let Some(a) = state.get_army_mut(army_a) {
                a.disengage();
            }
            if let Some(b) = state.get_army_mut(army_b) {
                b.disengage();
            }
        }

        for event in events {
            if let arc_citadel::campaign::CampaignEvent::ArmiesEngaged { army_a, army_b, position } = event {
                let a_name = state.get_army(army_a).map(|a| a.name.clone()).unwrap_or_default();
                let b_name = state.get_army(army_b).map(|a| a.name.clone()).unwrap_or_default();

                println!("BATTLE at ({}, {}): {} vs {}", position.q, position.r, a_name, b_name);

                let (mut attacker, mut defender) = {
                    let a = state.get_army(army_a).unwrap().clone();
                    let b = state.get_army(army_b).unwrap().clone();
                    (a, b)
                };

                let result = resolve_battle(&mut attacker, &mut defender, &map, current_weather, 10);

                let (blue_cas, red_cas) = if attacker.faction == PolityId(1) {
                    (result.attacker_casualties, result.defender_casualties)
                } else {
                    (result.defender_casualties, result.attacker_casualties)
                };
                stats.record_battle(blue_cas, red_cas);

                println!("  Result: {:?}", result.outcome);

                if let Some(a) = state.get_army_mut(army_a) {
                    a.unit_count = attacker.unit_count;
                    a.morale = attacker.morale;
                }
                if let Some(b) = state.get_army_mut(army_b) {
                    b.unit_count = defender.unit_count;
                    b.morale = defender.morale;
                }

                // Pursuit casualties: winner inflicts 20% additional casualties on fleeing army
                const PURSUIT_CASUALTY_RATE: f32 = 0.20;

                match result.outcome {
                    BattleOutcome::AttackerVictory => {
                        if let Some(b) = state.get_army_mut(army_b) {
                            // Pursuit casualties
                            let pursuit_cas = (b.unit_count as f32 * PURSUIT_CASUALTY_RATE) as u32;
                            b.unit_count = b.unit_count.saturating_sub(pursuit_cas);
                            let (blue, red) = if attacker.faction == PolityId(1) {
                                (0, pursuit_cas)
                            } else {
                                (pursuit_cas, 0)
                            };
                            stats.record_battle(blue, red);
                            apply_retreat(b, position, &map);
                            println!("  {} retreats (pursuit: -{} units)", b_name, pursuit_cas);
                        }
                    }
                    BattleOutcome::DefenderVictory => {
                        if let Some(a) = state.get_army_mut(army_a) {
                            // Pursuit casualties
                            let pursuit_cas = (a.unit_count as f32 * PURSUIT_CASUALTY_RATE) as u32;
                            a.unit_count = a.unit_count.saturating_sub(pursuit_cas);
                            let (blue, red) = if attacker.faction == PolityId(1) {
                                (pursuit_cas, 0)
                            } else {
                                (0, pursuit_cas)
                            };
                            stats.record_battle(blue, red);
                            apply_retreat(a, position, &map);
                            println!("  {} retreats (pursuit: -{} units)", a_name, pursuit_cas);
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

                // CRITICAL: Clear engaged_with so armies can fight again next tick
                if let Some(a) = state.get_army_mut(army_a) {
                    a.disengage();
                }
                if let Some(b) = state.get_army_mut(army_b) {
                    b.disengage();
                }
            }
        }

        occupation.update(&state, blue_depot, red_depot);
        stats.update_exploration(&visibility);

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

        if args.verbose {
            println!();
        }
    }
}
