//! Multi-Formation Battle Laboratory
//!
//! Tests multi-formation battle scenarios with different army compositions
//! and tactical setups. Units start close together to ensure engagement.

use std::collections::HashMap;

use arc_citadel::battle::{
    units::{Army, ArmyId, BattleFormation, BattleUnit, Element, FormationId, UnitId, UnitStance},
    unit_type::UnitType,
    BattleHexCoord, BattleMap, BattleState,
};
use arc_citadel::combat::state::CombatState;
use arc_citadel::core::types::EntityId;
use clap::{Parser, Subcommand};
use serde::Serialize;

/// Multi-Formation Battle Laboratory
#[derive(Parser, Debug)]
#[command(name = "multi_formation_lab")]
#[command(about = "Test multi-formation battle scenarios")]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Maximum ticks to run
    #[arg(long, default_value_t = 200)]
    max_ticks: u64,

    /// Verbose output
    #[arg(long, short = 'v')]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Basic 3v3 formation battle (infantry line vs infantry line)
    Basic,
    /// Flanking scenario (2 formations vs 1 formation)
    Flank,
    /// Mixed arms (pike + archers vs infantry)
    MixedArms,
    /// Hammer and anvil (infantry holds, cavalry flanks)
    HammerAnvil,
    /// Counter-pick (MenAtArms vs Heavy Infantry)
    CounterPick,
    /// 2:1 numerical advantage
    Numerical,
    /// Heavy cavalry charge vs infantry
    CavalryCharge,
    /// Spearmen vs cavalry counter
    SpearCavalry,
    /// Double envelopment (pincer attack)
    DoubleEnvelop,
    /// Quality vs quantity (elite few vs many militia)
    QualityVsQuantity,
    /// All scenarios
    All,
}

#[derive(Debug, Clone, Serialize)]
struct ScenarioResult {
    name: String,
    ticks: u64,
    friendly_initial: u32,
    friendly_remaining: u32,
    friendly_casualties: u32,
    enemy_initial: u32,
    enemy_remaining: u32,
    enemy_casualties: u32,
    outcome: String,
    friendly_routed: u32,
    enemy_routed: u32,
}

fn run_scenario(
    name: &str,
    friendly_army: Army,
    enemy_army: Army,
    max_ticks: u64,
    verbose: bool,
) -> ScenarioResult {
    let friendly_initial = friendly_army.total_strength();
    let enemy_initial = enemy_army.total_strength();

    let map = BattleMap::new(30, 20);
    let mut state = BattleState::new(map, friendly_army, enemy_army);
    state.start_battle();

    // Manually trigger movement toward each other (no AI, direct control)
    // Set all units to Moving stance and assign simple movement vectors

    if verbose {
        eprintln!("=== {} ===", name);
        eprintln!("Friendly: {} troops", friendly_initial);
        eprintln!("Enemy: {} troops", enemy_initial);
    }

    let mut entity_states: HashMap<EntityId, CombatState> = HashMap::new();

    // Initialize entity states from unit equipment
    for formation in &state.friendly_army.formations {
        for unit in &formation.units {
            for element in &unit.elements {
                let equipment = element.equipment_type.unwrap_or(unit.unit_type);
                let props = equipment.default_properties();
                for entity_id in &element.entities {
                    entity_states.insert(*entity_id, CombatState {
                        weapon: props.avg_weapon.clone(),
                        armor: props.avg_armor.clone(),
                        ..Default::default()
                    });
                }
            }
        }
    }
    for formation in &state.enemy_army.formations {
        for unit in &formation.units {
            for element in &unit.elements {
                let equipment = element.equipment_type.unwrap_or(unit.unit_type);
                let props = equipment.default_properties();
                for entity_id in &element.entities {
                    entity_states.insert(*entity_id, CombatState {
                        weapon: props.avg_weapon.clone(),
                        armor: props.avg_armor.clone(),
                        ..Default::default()
                    });
                }
            }
        }
    }

    // Run battle ticks with manual unit movement
    let mut actual_tick = 0u64;
    for tick in 0..max_ticks {
        actual_tick = tick;
        // Move friendly units toward enemy (simple: increase q)
        for formation in &mut state.friendly_army.formations {
            for unit in &mut formation.units {
                if unit.can_fight() && unit.stance != UnitStance::Routing {
                    // Find nearest enemy
                    let mut nearest_enemy_pos = None;
                    let mut min_dist = u32::MAX;
                    for ef in &state.enemy_army.formations {
                        for eu in &ef.units {
                            if eu.can_fight() {
                                let dist = unit.position.distance(&eu.position);
                                if dist < min_dist {
                                    min_dist = dist;
                                    nearest_enemy_pos = Some(eu.position);
                                }
                            }
                        }
                    }

                    if let Some(target) = nearest_enemy_pos {
                        if min_dist > 1 {
                            // Move toward enemy
                            let dq = (target.q - unit.position.q).signum();
                            let dr = (target.r - unit.position.r).signum();
                            unit.position = BattleHexCoord::new(
                                unit.position.q + dq,
                                unit.position.r + dr,
                            );
                            unit.stance = UnitStance::Moving;
                        } else {
                            unit.stance = UnitStance::Formed;
                        }
                    }
                }
            }
        }

        // Move enemy units toward friendly
        for formation in &mut state.enemy_army.formations {
            for unit in &mut formation.units {
                if unit.can_fight() && unit.stance != UnitStance::Routing {
                    let mut nearest_friendly_pos = None;
                    let mut min_dist = u32::MAX;
                    for ff in &state.friendly_army.formations {
                        for fu in &ff.units {
                            if fu.can_fight() {
                                let dist = unit.position.distance(&fu.position);
                                if dist < min_dist {
                                    min_dist = dist;
                                    nearest_friendly_pos = Some(fu.position);
                                }
                            }
                        }
                    }

                    if let Some(target) = nearest_friendly_pos {
                        if min_dist > 1 {
                            let dq = (target.q - unit.position.q).signum();
                            let dr = (target.r - unit.position.r).signum();
                            unit.position = BattleHexCoord::new(
                                unit.position.q + dq,
                                unit.position.r + dr,
                            );
                            unit.stance = UnitStance::Moving;
                        } else {
                            unit.stance = UnitStance::Formed;
                        }
                    }
                }
            }
        }

        // Find and resolve all engagements
        let mut engagements = Vec::new();
        for ff in &state.friendly_army.formations {
            for fu in &ff.units {
                if !fu.can_fight() { continue; }
                for ef in &state.enemy_army.formations {
                    for eu in &ef.units {
                        if !eu.can_fight() { continue; }
                        if fu.position.distance(&eu.position) <= 1 {
                            engagements.push((fu.id, eu.id));
                        }
                    }
                }
            }
        }

        // Resolve each engagement
        for (friendly_id, enemy_id) in &engagements {
            // Find the units
            let friendly_unit = state.friendly_army.formations.iter()
                .flat_map(|f| f.units.iter())
                .find(|u| u.id == *friendly_id);
            let enemy_unit = state.enemy_army.formations.iter()
                .flat_map(|f| f.units.iter())
                .find(|u| u.id == *enemy_id);

            if let (Some(fu), Some(eu)) = (friendly_unit, enemy_unit) {
                // Resolve combat
                let result = arc_citadel::battle::resolution::resolve_unit_combat(
                    fu, eu, &mut entity_states, &state.map
                );

                // Apply results
                for formation in &mut state.friendly_army.formations {
                    for unit in &mut formation.units {
                        if unit.id == *friendly_id {
                            unit.casualties += result.attacker_casualties;
                            unit.stress += result.attacker_stress_delta;
                            // Check morale break
                            if unit.stress >= unit.stress_threshold() {
                                unit.stance = UnitStance::Routing;
                            }
                        }
                    }
                }
                for formation in &mut state.enemy_army.formations {
                    for unit in &mut formation.units {
                        if unit.id == *enemy_id {
                            unit.casualties += result.defender_casualties;
                            unit.stress += result.defender_stress_delta;
                            if unit.stress >= unit.stress_threshold() {
                                unit.stance = UnitStance::Routing;
                            }
                        }
                    }
                }

                if verbose && (result.attacker_casualties > 0 || result.defender_casualties > 0) {
                    eprintln!("  Tick {}: Combat! Friendly: {} cas, {:.2} stress | Enemy: {} cas, {:.2} stress",
                        tick,
                        result.attacker_casualties, result.attacker_stress_delta,
                        result.defender_casualties, result.defender_stress_delta);
                }
            }
        }

        // Check for battle end
        let friendly_effective = state.friendly_army.effective_strength();
        let enemy_effective = state.enemy_army.effective_strength();

        if friendly_effective == 0 || enemy_effective == 0 {
            if verbose {
                eprintln!("  Battle ended at tick {}", tick);
            }
            break;
        }
    }

    // Calculate results
    let friendly_remaining = state.friendly_army.effective_strength();
    let enemy_remaining = state.enemy_army.effective_strength();
    let friendly_casualties = friendly_initial - friendly_remaining;
    let enemy_casualties = enemy_initial - enemy_remaining;

    let friendly_routed = state.friendly_army.formations.iter()
        .flat_map(|f| f.units.iter())
        .filter(|u| matches!(u.stance, UnitStance::Routing))
        .count() as u32;
    let enemy_routed = state.enemy_army.formations.iter()
        .flat_map(|f| f.units.iter())
        .filter(|u| matches!(u.stance, UnitStance::Routing))
        .count() as u32;

    let outcome = if enemy_remaining == 0 && friendly_remaining > 0 {
        "Victory"
    } else if friendly_remaining == 0 && enemy_remaining > 0 {
        "Defeat"
    } else if friendly_casualties < enemy_casualties {
        "Minor Victory"
    } else if enemy_casualties < friendly_casualties {
        "Minor Defeat"
    } else {
        "Draw"
    };

    ScenarioResult {
        name: name.to_string(),
        ticks: actual_tick,
        friendly_initial: friendly_initial as u32,
        friendly_remaining: friendly_remaining as u32,
        friendly_casualties: friendly_casualties as u32,
        enemy_initial: enemy_initial as u32,
        enemy_remaining: enemy_remaining as u32,
        enemy_casualties: enemy_casualties as u32,
        outcome: outcome.to_string(),
        friendly_routed,
        enemy_routed,
    }
}

fn create_unit(unit_type: UnitType, size: u32, position: BattleHexCoord) -> BattleUnit {
    let mut unit = BattleUnit::new(UnitId::new(), unit_type);
    unit.position = position;
    unit.elements.push(Element::with_equipment(
        (0..size).map(|_| EntityId::new()).collect(),
        unit_type,
    ));
    unit
}

fn create_mixed_unit(
    base_type: UnitType,
    composition: &[(UnitType, u32)],
    position: BattleHexCoord,
) -> BattleUnit {
    let mut unit = BattleUnit::new(UnitId::new(), base_type);
    unit.position = position;
    for (ut, count) in composition {
        unit.elements.push(Element::with_equipment(
            (0..*count).map(|_| EntityId::new()).collect(),
            *ut,
        ));
    }
    unit
}

// ============================================================================
// SCENARIOS
// ============================================================================

fn scenario_basic() -> (Army, Army) {
    // 3 units of 50 infantry each, facing each other
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
    formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(5, 8)));
    formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(5, 10)));
    formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(5, 12)));
    friendly.formations.push(formation);

    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut e_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    e_formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(15, 8)));
    e_formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(15, 10)));
    e_formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(15, 12)));
    enemy.formations.push(e_formation);

    (friendly, enemy)
}

fn scenario_flank() -> (Army, Army) {
    // 2 friendly formations vs 1 enemy - flanking attack
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());

    // Main formation (frontal)
    let mut main_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    main_formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(5, 10)));
    main_formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(5, 12)));
    friendly.formations.push(main_formation);

    // Flanking formation (coming from south)
    let mut flank_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    flank_formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(15, 16)));
    flank_formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(17, 16)));
    friendly.formations.push(flank_formation);

    // Enemy single formation
    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut e_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    e_formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(15, 10)));
    e_formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(15, 12)));
    e_formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(15, 14)));
    enemy.formations.push(e_formation);

    (friendly, enemy)
}

fn scenario_mixed_arms() -> (Army, Army) {
    // Pike + Archer formation vs pure Infantry
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());

    // Mixed pike + archer units
    formation.units.push(create_mixed_unit(
        UnitType::Infantry,
        &[(UnitType::Spearmen, 35), (UnitType::Archers, 15)],
        BattleHexCoord::new(5, 10),
    ));
    formation.units.push(create_mixed_unit(
        UnitType::Infantry,
        &[(UnitType::Spearmen, 35), (UnitType::Archers, 15)],
        BattleHexCoord::new(5, 12),
    ));
    friendly.formations.push(formation);

    // Enemy pure infantry
    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut e_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    e_formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(15, 10)));
    e_formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(15, 12)));
    enemy.formations.push(e_formation);

    (friendly, enemy)
}

fn scenario_hammer_anvil() -> (Army, Army) {
    // Infantry anvil + Cavalry hammer vs Infantry
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());

    // Anvil: Heavy Infantry holds the line
    let mut anvil = BattleFormation::new(FormationId::new(), EntityId::new());
    anvil.units.push(create_unit(UnitType::HeavyInfantry, 40, BattleHexCoord::new(5, 10)));
    anvil.units.push(create_unit(UnitType::HeavyInfantry, 40, BattleHexCoord::new(5, 12)));
    friendly.formations.push(anvil);

    // Hammer: Cavalry flanks
    let mut hammer = BattleFormation::new(FormationId::new(), EntityId::new());
    hammer.units.push(create_unit(UnitType::HeavyCavalry, 30, BattleHexCoord::new(10, 16)));
    hammer.units.push(create_unit(UnitType::Cavalry, 30, BattleHexCoord::new(12, 16)));
    friendly.formations.push(hammer);

    // Enemy infantry formation
    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut e_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    e_formation.units.push(create_unit(UnitType::Infantry, 60, BattleHexCoord::new(15, 10)));
    e_formation.units.push(create_unit(UnitType::Infantry, 60, BattleHexCoord::new(15, 12)));
    e_formation.units.push(create_unit(UnitType::Infantry, 60, BattleHexCoord::new(15, 14)));
    enemy.formations.push(e_formation);

    (friendly, enemy)
}

fn scenario_counter_pick() -> (Army, Army) {
    // MenAtArms vs Heavy Infantry - testing anti-armor
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
    formation.units.push(create_unit(UnitType::MenAtArms, 50, BattleHexCoord::new(5, 10)));
    formation.units.push(create_unit(UnitType::MenAtArms, 50, BattleHexCoord::new(5, 12)));
    friendly.formations.push(formation);

    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut e_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    e_formation.units.push(create_unit(UnitType::HeavyInfantry, 50, BattleHexCoord::new(15, 10)));
    e_formation.units.push(create_unit(UnitType::HeavyInfantry, 50, BattleHexCoord::new(15, 12)));
    enemy.formations.push(e_formation);

    (friendly, enemy)
}

fn scenario_numerical_superiority() -> (Army, Army) {
    // 2:1 numerical advantage
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
    formation.units.push(create_unit(UnitType::Infantry, 100, BattleHexCoord::new(5, 8)));
    formation.units.push(create_unit(UnitType::Infantry, 100, BattleHexCoord::new(5, 10)));
    formation.units.push(create_unit(UnitType::Infantry, 100, BattleHexCoord::new(5, 12)));
    formation.units.push(create_unit(UnitType::Infantry, 100, BattleHexCoord::new(5, 14)));
    friendly.formations.push(formation);

    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut e_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    e_formation.units.push(create_unit(UnitType::Infantry, 100, BattleHexCoord::new(15, 10)));
    e_formation.units.push(create_unit(UnitType::Infantry, 100, BattleHexCoord::new(15, 12)));
    enemy.formations.push(e_formation);

    (friendly, enemy)
}

fn scenario_cavalry_charge() -> (Army, Army) {
    // Heavy Cavalry charge against infantry line
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
    formation.units.push(create_unit(UnitType::HeavyCavalry, 40, BattleHexCoord::new(5, 10)));
    formation.units.push(create_unit(UnitType::HeavyCavalry, 40, BattleHexCoord::new(5, 12)));
    friendly.formations.push(formation);

    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut e_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    e_formation.units.push(create_unit(UnitType::Infantry, 60, BattleHexCoord::new(15, 10)));
    e_formation.units.push(create_unit(UnitType::Infantry, 60, BattleHexCoord::new(15, 12)));
    enemy.formations.push(e_formation);

    (friendly, enemy)
}

fn scenario_spear_vs_cavalry() -> (Army, Army) {
    // Spearmen counter cavalry
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
    formation.units.push(create_unit(UnitType::Spearmen, 50, BattleHexCoord::new(5, 10)));
    formation.units.push(create_unit(UnitType::Spearmen, 50, BattleHexCoord::new(5, 12)));
    friendly.formations.push(formation);

    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut e_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    e_formation.units.push(create_unit(UnitType::HeavyCavalry, 50, BattleHexCoord::new(15, 10)));
    e_formation.units.push(create_unit(UnitType::HeavyCavalry, 50, BattleHexCoord::new(15, 12)));
    enemy.formations.push(e_formation);

    (friendly, enemy)
}

fn scenario_double_envelopment() -> (Army, Army) {
    // Classic double envelopment - 3 formations pincer attack
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());

    // Center holds
    let mut center = BattleFormation::new(FormationId::new(), EntityId::new());
    center.units.push(create_unit(UnitType::Infantry, 30, BattleHexCoord::new(5, 10)));
    friendly.formations.push(center);

    // Left flank
    let mut left = BattleFormation::new(FormationId::new(), EntityId::new());
    left.units.push(create_unit(UnitType::Cavalry, 30, BattleHexCoord::new(3, 5)));
    friendly.formations.push(left);

    // Right flank
    let mut right = BattleFormation::new(FormationId::new(), EntityId::new());
    right.units.push(create_unit(UnitType::Cavalry, 30, BattleHexCoord::new(3, 15)));
    friendly.formations.push(right);

    // Enemy single formation
    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut e_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    e_formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(15, 8)));
    e_formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(15, 10)));
    e_formation.units.push(create_unit(UnitType::Infantry, 50, BattleHexCoord::new(15, 12)));
    enemy.formations.push(e_formation);

    (friendly, enemy)
}

fn scenario_quality_vs_quantity() -> (Army, Army) {
    // Elite few vs many militia
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
    formation.units.push(create_unit(UnitType::MenAtArms, 30, BattleHexCoord::new(5, 10)));
    friendly.formations.push(formation);

    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut e_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    e_formation.units.push(create_unit(UnitType::Levy, 50, BattleHexCoord::new(15, 8)));
    e_formation.units.push(create_unit(UnitType::Levy, 50, BattleHexCoord::new(15, 10)));
    e_formation.units.push(create_unit(UnitType::Levy, 50, BattleHexCoord::new(15, 12)));
    enemy.formations.push(e_formation);

    (friendly, enemy)
}

fn print_results(results: &[ScenarioResult]) {
    println!("\n{:=<80}", "");
    println!("MULTI-FORMATION BATTLE RESULTS");
    println!("{:=<80}\n", "");

    for r in results {
        println!("Scenario: {}", r.name);
        println!("  Outcome: {}", r.outcome);
        println!("  Ticks: {}", r.ticks);
        println!("  Friendly: {}/{} remaining ({} casualties, {} routed)",
            r.friendly_remaining, r.friendly_initial, r.friendly_casualties, r.friendly_routed);
        println!("  Enemy: {}/{} remaining ({} casualties, {} routed)",
            r.enemy_remaining, r.enemy_initial, r.enemy_casualties, r.enemy_routed);

        let exchange = if r.friendly_casualties > 0 {
            r.enemy_casualties as f32 / r.friendly_casualties as f32
        } else if r.enemy_casualties > 0 {
            f32::INFINITY
        } else {
            1.0
        };
        println!("  Exchange ratio: {:.2}:1 (enemy:friendly)", exchange);
        println!();
    }

    // Summary
    println!("{:=<80}", "");
    println!("SUMMARY");
    println!("{:=<80}", "");

    let victories = results.iter().filter(|r| r.outcome.contains("Victory")).count();
    let defeats = results.iter().filter(|r| r.outcome.contains("Defeat")).count();
    let draws = results.iter().filter(|r| r.outcome == "Draw").count();

    println!("Victories: {}, Defeats: {}, Draws: {}", victories, defeats, draws);
}

fn main() {
    let args = Args::parse();
    let mut results = Vec::new();

    match args.command {
        Commands::Basic => {
            let (f, e) = scenario_basic();
            results.push(run_scenario("Basic 3v3 Infantry", f, e, args.max_ticks, args.verbose));
        }
        Commands::Flank => {
            let (f, e) = scenario_flank();
            results.push(run_scenario("Flanking Attack", f, e, args.max_ticks, args.verbose));
        }
        Commands::MixedArms => {
            let (f, e) = scenario_mixed_arms();
            results.push(run_scenario("Mixed Arms (Pike+Archer)", f, e, args.max_ticks, args.verbose));
        }
        Commands::HammerAnvil => {
            let (f, e) = scenario_hammer_anvil();
            results.push(run_scenario("Hammer and Anvil", f, e, args.max_ticks, args.verbose));
        }
        Commands::CounterPick => {
            let (f, e) = scenario_counter_pick();
            results.push(run_scenario("Counter-Pick (MenAtArms vs Heavy)", f, e, args.max_ticks, args.verbose));
        }
        Commands::Numerical => {
            let (f, e) = scenario_numerical_superiority();
            results.push(run_scenario("Numerical Superiority (2:1)", f, e, args.max_ticks, args.verbose));
        }
        Commands::CavalryCharge => {
            let (f, e) = scenario_cavalry_charge();
            results.push(run_scenario("Cavalry Charge vs Infantry", f, e, args.max_ticks, args.verbose));
        }
        Commands::SpearCavalry => {
            let (f, e) = scenario_spear_vs_cavalry();
            results.push(run_scenario("Spearmen vs Cavalry", f, e, args.max_ticks, args.verbose));
        }
        Commands::DoubleEnvelop => {
            let (f, e) = scenario_double_envelopment();
            results.push(run_scenario("Double Envelopment", f, e, args.max_ticks, args.verbose));
        }
        Commands::QualityVsQuantity => {
            let (f, e) = scenario_quality_vs_quantity();
            results.push(run_scenario("Quality vs Quantity", f, e, args.max_ticks, args.verbose));
        }
        Commands::All => {
            let (f, e) = scenario_basic();
            results.push(run_scenario("Basic 3v3 Infantry", f, e, args.max_ticks, args.verbose));

            let (f, e) = scenario_flank();
            results.push(run_scenario("Flanking Attack", f, e, args.max_ticks, args.verbose));

            let (f, e) = scenario_mixed_arms();
            results.push(run_scenario("Mixed Arms (Pike+Archer)", f, e, args.max_ticks, args.verbose));

            let (f, e) = scenario_hammer_anvil();
            results.push(run_scenario("Hammer and Anvil", f, e, args.max_ticks, args.verbose));

            let (f, e) = scenario_counter_pick();
            results.push(run_scenario("Counter-Pick (MenAtArms vs Heavy)", f, e, args.max_ticks, args.verbose));

            let (f, e) = scenario_numerical_superiority();
            results.push(run_scenario("Numerical Superiority (2:1)", f, e, args.max_ticks, args.verbose));

            let (f, e) = scenario_cavalry_charge();
            results.push(run_scenario("Cavalry Charge vs Infantry", f, e, args.max_ticks, args.verbose));

            let (f, e) = scenario_spear_vs_cavalry();
            results.push(run_scenario("Spearmen vs Cavalry", f, e, args.max_ticks, args.verbose));

            let (f, e) = scenario_double_envelopment();
            results.push(run_scenario("Double Envelopment", f, e, args.max_ticks, args.verbose));

            let (f, e) = scenario_quality_vs_quantity();
            results.push(run_scenario("Quality vs Quantity", f, e, args.max_ticks, args.verbose));
        }
    }

    print_results(&results);
}
