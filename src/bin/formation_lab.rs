//! Formation Laboratory - Test harness for formation optimization
//!
//! Runs hundreds of unit-vs-unit combat scenarios to explore:
//! - Formation shape effects (Line, Column, Wedge, Square, Skirmish)
//! - Unit composition (pike:archer:infantry ratios)
//! - Equipment matchups (weapon vs armor)
//! - Terrain interactions
//! - Morale cascade behavior

use std::collections::HashMap;

use arc_citadel::battle::{
    battle_map::BattleMap,
    units::{BattleUnit, Element, FormationShape, UnitId},
    unit_type::UnitType,
    resolution::resolve_unit_combat,
};
use arc_citadel::combat::state::CombatState;
use arc_citadel::core::types::EntityId;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

/// Formation Laboratory - Explore combat dynamics through simulation
#[derive(Parser, Debug)]
#[command(name = "formation_lab")]
#[command(about = "Test harness for formation and combat optimization")]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Number of trials per configuration (for statistical significance)
    #[arg(long, default_value_t = 100)]
    trials: u32,

    /// Combat rounds per trial
    #[arg(long, default_value_t = 10)]
    rounds: u32,

    /// Output format: json, csv, or text
    #[arg(long, default_value = "text")]
    format: String,

    /// Verbose output (show individual trial results)
    #[arg(long, short = 'v')]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Test formation shape effects
    Shapes,
    /// Test unit composition ratios
    Composition,
    /// Test equipment matchups
    Equipment,
    /// Test terrain effects
    Terrain,
    /// Test morale cascade behavior
    Morale,
    /// Run all tests
    All,
}

/// Results from a single combat trial
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrialResult {
    attacker_casualties: u32,
    defender_casualties: u32,
    attacker_stress: f32,
    defender_stress: f32,
    rounds_to_rout: Option<u32>,
}

/// Aggregated results from multiple trials
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AggregateResult {
    config_name: String,
    trials: u32,
    avg_attacker_casualties: f32,
    avg_defender_casualties: f32,
    casualty_exchange_ratio: f32, // defender_cas / attacker_cas
    avg_rounds_to_decision: f32,
    attacker_win_rate: f32,
    defender_win_rate: f32,
    draw_rate: f32,
}

/// Configuration for a test scenario
#[derive(Debug, Clone)]
struct TestConfig {
    name: String,
    attacker: UnitConfig,
    defender: UnitConfig,
}

/// Configuration for a single unit
#[derive(Debug, Clone)]
struct UnitConfig {
    total_size: u32,
    /// (UnitType, count) pairs for elements
    composition: Vec<(UnitType, u32)>,
    formation_shape: FormationShape,
}

impl UnitConfig {
    fn pure(unit_type: UnitType, size: u32) -> Self {
        Self {
            total_size: size,
            composition: vec![(unit_type, size)],
            formation_shape: FormationShape::default(),
        }
    }

    fn mixed(composition: Vec<(UnitType, u32)>) -> Self {
        let total: u32 = composition.iter().map(|(_, c)| c).sum();
        Self {
            total_size: total,
            composition,
            formation_shape: FormationShape::default(),
        }
    }

    fn with_shape(mut self, shape: FormationShape) -> Self {
        self.formation_shape = shape;
        self
    }

    fn build(&self) -> BattleUnit {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.formation_shape = self.formation_shape.clone();

        for (unit_type, count) in &self.composition {
            let entities: Vec<EntityId> = (0..*count).map(|_| EntityId::new()).collect();
            unit.elements.push(Element::with_equipment(entities, *unit_type));
        }

        unit
    }
}

fn run_trial(
    attacker_config: &UnitConfig,
    defender_config: &UnitConfig,
    rounds: u32,
    verbose: bool,
) -> TrialResult {
    let mut entity_states: HashMap<EntityId, CombatState> = HashMap::new();
    let attacker = attacker_config.build();
    let defender = defender_config.build();
    // Create minimal battle map for combat resolution (open terrain)
    let map = BattleMap::new(10, 10);

    let mut total_att_cas = 0u32;
    let mut total_def_cas = 0u32;
    let mut total_att_stress = 0.0f32;
    let mut total_def_stress = 0.0f32;
    let mut rounds_to_rout = None;

    for round in 0..rounds {
        let result = resolve_unit_combat(&attacker, &defender, &mut entity_states, &map);

        total_att_cas += result.attacker_casualties;
        total_def_cas += result.defender_casualties;
        total_att_stress += result.attacker_stress_delta;
        total_def_stress += result.defender_stress_delta;

        if verbose {
            eprintln!(
                "  Round {}: att_cas={}, def_cas={}, att_stress={:.2}, def_stress={:.2}",
                round + 1,
                result.attacker_casualties,
                result.defender_casualties,
                total_att_stress,
                total_def_stress
            );
        }

        // Check for rout (stress threshold ~2.0 for standard units)
        if rounds_to_rout.is_none() {
            if total_att_stress > 2.0 || total_def_stress > 2.0 {
                rounds_to_rout = Some(round + 1);
            }
        }

        // Check for one side eliminated
        let att_remaining = attacker_config.total_size.saturating_sub(total_att_cas);
        let def_remaining = defender_config.total_size.saturating_sub(total_def_cas);
        if att_remaining == 0 || def_remaining == 0 {
            if rounds_to_rout.is_none() {
                rounds_to_rout = Some(round + 1);
            }
            break;
        }
    }

    TrialResult {
        attacker_casualties: total_att_cas,
        defender_casualties: total_def_cas,
        attacker_stress: total_att_stress,
        defender_stress: total_def_stress,
        rounds_to_rout,
    }
}

fn run_test(config: &TestConfig, trials: u32, rounds: u32, verbose: bool) -> AggregateResult {
    let mut results = Vec::with_capacity(trials as usize);

    for trial in 0..trials {
        if verbose {
            eprintln!("Trial {}/{} for {}", trial + 1, trials, config.name);
        }
        let result = run_trial(&config.attacker, &config.defender, rounds, verbose);
        results.push(result);
    }

    // Aggregate
    let total_att_cas: u32 = results.iter().map(|r| r.attacker_casualties).sum();
    let total_def_cas: u32 = results.iter().map(|r| r.defender_casualties).sum();

    let avg_att_cas = total_att_cas as f32 / trials as f32;
    let avg_def_cas = total_def_cas as f32 / trials as f32;

    let exchange_ratio = if avg_att_cas > 0.0 {
        avg_def_cas / avg_att_cas
    } else {
        f32::INFINITY
    };

    let rounds_sum: u32 = results
        .iter()
        .filter_map(|r| r.rounds_to_rout)
        .sum();
    let rounds_count = results.iter().filter(|r| r.rounds_to_rout.is_some()).count();
    let avg_rounds = if rounds_count > 0 {
        rounds_sum as f32 / rounds_count as f32
    } else {
        rounds as f32
    };

    // Win rates based on casualties
    let mut att_wins = 0;
    let mut def_wins = 0;
    let mut draws = 0;
    for r in &results {
        if r.defender_casualties > r.attacker_casualties * 2 {
            att_wins += 1;
        } else if r.attacker_casualties > r.defender_casualties * 2 {
            def_wins += 1;
        } else {
            draws += 1;
        }
    }

    AggregateResult {
        config_name: config.name.clone(),
        trials,
        avg_attacker_casualties: avg_att_cas,
        avg_defender_casualties: avg_def_cas,
        casualty_exchange_ratio: exchange_ratio,
        avg_rounds_to_decision: avg_rounds,
        attacker_win_rate: att_wins as f32 / trials as f32,
        defender_win_rate: def_wins as f32 / trials as f32,
        draw_rate: draws as f32 / trials as f32,
    }
}

fn print_results(results: &[AggregateResult], format: &str) {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(results).unwrap());
        }
        "csv" => {
            println!("config,trials,avg_att_cas,avg_def_cas,exchange_ratio,avg_rounds,att_win_rate,def_win_rate,draw_rate");
            for r in results {
                println!(
                    "{},{},{:.1},{:.1},{:.2},{:.1},{:.2},{:.2},{:.2}",
                    r.config_name,
                    r.trials,
                    r.avg_attacker_casualties,
                    r.avg_defender_casualties,
                    r.casualty_exchange_ratio,
                    r.avg_rounds_to_decision,
                    r.attacker_win_rate,
                    r.defender_win_rate,
                    r.draw_rate
                );
            }
        }
        _ => {
            println!("\n{:=<80}", "");
            println!("FORMATION LAB RESULTS");
            println!("{:=<80}\n", "");

            for r in results {
                println!("Configuration: {}", r.config_name);
                println!("  Trials: {}", r.trials);
                println!(
                    "  Avg Casualties: attacker={:.1}, defender={:.1}",
                    r.avg_attacker_casualties, r.avg_defender_casualties
                );
                println!("  Exchange Ratio: {:.2}:1 (defender:attacker)", r.casualty_exchange_ratio);
                println!("  Avg Rounds to Decision: {:.1}", r.avg_rounds_to_decision);
                println!(
                    "  Win Rates: attacker={:.0}%, defender={:.0}%, draw={:.0}%",
                    r.attacker_win_rate * 100.0,
                    r.defender_win_rate * 100.0,
                    r.draw_rate * 100.0
                );
                println!();
            }
        }
    }
}

// =============================================================================
// TEST GENERATORS
// =============================================================================

fn generate_shape_tests() -> Vec<TestConfig> {
    let shapes = vec![
        ("Line_d2", FormationShape::Line { depth: 2 }),
        ("Line_d4", FormationShape::Line { depth: 4 }),
        ("Line_d6", FormationShape::Line { depth: 6 }),
        ("Column_w4", FormationShape::Column { width: 4 }),
        ("Column_w8", FormationShape::Column { width: 8 }),
        ("Wedge_30", FormationShape::Wedge { angle: 30.0 }),
        ("Wedge_60", FormationShape::Wedge { angle: 60.0 }),
        ("Square", FormationShape::Square),
        ("Skirmish_low", FormationShape::Skirmish { dispersion: 0.3 }),
        ("Skirmish_high", FormationShape::Skirmish { dispersion: 0.7 }),
    ];

    let mut configs = Vec::new();

    // Test each shape vs standard Line formation
    for (name, shape) in &shapes {
        configs.push(TestConfig {
            name: format!("{} vs Line_d2 (100 infantry each)", name),
            attacker: UnitConfig::pure(UnitType::Infantry, 100).with_shape(shape.clone()),
            defender: UnitConfig::pure(UnitType::Infantry, 100)
                .with_shape(FormationShape::Line { depth: 2 }),
        });
    }

    // Test shapes with mixed units
    for (name, shape) in &shapes {
        configs.push(TestConfig {
            name: format!("{} mixed (70 pike + 30 archer) vs Infantry", name),
            attacker: UnitConfig::mixed(vec![
                (UnitType::Spearmen, 70),
                (UnitType::Archers, 30),
            ])
            .with_shape(shape.clone()),
            defender: UnitConfig::pure(UnitType::Infantry, 100),
        });
    }

    configs
}

fn generate_composition_tests() -> Vec<TestConfig> {
    let mut configs = Vec::new();

    // Pike ratios
    let pike_ratios = vec![
        (100, 0, 0, "100% Pike"),
        (80, 20, 0, "80% Pike + 20% Archer"),
        (70, 30, 0, "70% Pike + 30% Archer"),
        (60, 40, 0, "60% Pike + 40% Archer"),
        (50, 50, 0, "50% Pike + 50% Archer"),
        (50, 25, 25, "50% Pike + 25% Archer + 25% Infantry"),
        (0, 100, 0, "100% Archer"),
        (0, 50, 50, "50% Archer + 50% Infantry"),
        (0, 0, 100, "100% Infantry"),
        (33, 33, 34, "Equal Mix"),
    ];

    for (pike, archer, infantry, name) in &pike_ratios {
        let mut composition = Vec::new();
        if *pike > 0 {
            composition.push((UnitType::Spearmen, *pike));
        }
        if *archer > 0 {
            composition.push((UnitType::Archers, *archer));
        }
        if *infantry > 0 {
            composition.push((UnitType::Infantry, *infantry));
        }

        // vs pure infantry
        configs.push(TestConfig {
            name: format!("{} vs 100 Infantry", name),
            attacker: UnitConfig::mixed(composition.clone()),
            defender: UnitConfig::pure(UnitType::Infantry, 100),
        });

        // vs heavy infantry
        configs.push(TestConfig {
            name: format!("{} vs 100 Heavy Infantry", name),
            attacker: UnitConfig::mixed(composition.clone()),
            defender: UnitConfig::pure(UnitType::HeavyInfantry, 100),
        });
    }

    // Mirror matches
    for (pike, archer, infantry, name) in &pike_ratios {
        let mut composition = Vec::new();
        if *pike > 0 {
            composition.push((UnitType::Spearmen, *pike));
        }
        if *archer > 0 {
            composition.push((UnitType::Archers, *archer));
        }
        if *infantry > 0 {
            composition.push((UnitType::Infantry, *infantry));
        }

        configs.push(TestConfig {
            name: format!("{} vs {} (mirror)", name, name),
            attacker: UnitConfig::mixed(composition.clone()),
            defender: UnitConfig::mixed(composition),
        });
    }

    configs
}

fn generate_equipment_tests() -> Vec<TestConfig> {
    let mut configs = Vec::new();

    let unit_types = vec![
        (UnitType::Levy, "Levy"),
        (UnitType::Infantry, "Infantry"),
        (UnitType::HeavyInfantry, "HeavyInfantry"),
        (UnitType::MenAtArms, "MenAtArms"),  // Anti-armor specialists
        (UnitType::Spearmen, "Spearmen"),
        (UnitType::Archers, "Archers"),
        (UnitType::Crossbowmen, "Crossbowmen"),
        (UnitType::LightCavalry, "LightCavalry"),
        (UnitType::Cavalry, "Cavalry"),
        (UnitType::HeavyCavalry, "HeavyCavalry"),
    ];

    // All matchups
    for (att_type, att_name) in &unit_types {
        for (def_type, def_name) in &unit_types {
            configs.push(TestConfig {
                name: format!("{} vs {}", att_name, def_name),
                attacker: UnitConfig::pure(*att_type, 50),
                defender: UnitConfig::pure(*def_type, 50),
            });
        }
    }

    configs
}

fn generate_terrain_tests() -> Vec<TestConfig> {
    // Terrain effects would modify combat - for now, test different unit sizes
    // as a proxy for chokepoint/open terrain (narrow frontage vs wide)
    let mut configs = Vec::new();

    // Simulate chokepoint by using smaller units
    let sizes = vec![
        (20, "Narrow (20v20)"),
        (50, "Medium (50v50)"),
        (100, "Wide (100v100)"),
        (200, "Very Wide (200v200)"),
    ];

    for (size, name) in &sizes {
        // Pike advantage in narrow
        configs.push(TestConfig {
            name: format!("{}: Pike vs Infantry", name),
            attacker: UnitConfig::pure(UnitType::Spearmen, *size),
            defender: UnitConfig::pure(UnitType::Infantry, *size),
        });

        // Cavalry advantage in wide
        configs.push(TestConfig {
            name: format!("{}: Cavalry vs Infantry", name),
            attacker: UnitConfig::pure(UnitType::Cavalry, *size),
            defender: UnitConfig::pure(UnitType::Infantry, *size),
        });

        // Archers need space
        configs.push(TestConfig {
            name: format!("{}: Mixed (Pike+Archer) vs Infantry", name),
            attacker: UnitConfig::mixed(vec![
                (UnitType::Spearmen, size / 2),
                (UnitType::Archers, size / 2),
            ]),
            defender: UnitConfig::pure(UnitType::Infantry, *size),
        });
    }

    configs
}

fn generate_morale_tests() -> Vec<TestConfig> {
    let mut configs = Vec::new();

    // Test different stress thresholds by unit type
    let unit_types = vec![
        (UnitType::Levy, "Levy (low morale)"),
        (UnitType::Infantry, "Infantry (standard)"),
        (UnitType::HeavyInfantry, "HeavyInfantry (high morale)"),
        (UnitType::HeavyCavalry, "HeavyCavalry (elite)"),
    ];

    // Test morale cascades with different unit sizes
    for (ut, name) in &unit_types {
        configs.push(TestConfig {
            name: format!("{}: 100 vs 100 Infantry", name),
            attacker: UnitConfig::pure(*ut, 100),
            defender: UnitConfig::pure(UnitType::Infantry, 100),
        });

        // Outnumbered scenarios (morale pressure)
        configs.push(TestConfig {
            name: format!("{}: 50 vs 100 Infantry (outnumbered)", name),
            attacker: UnitConfig::pure(*ut, 50),
            defender: UnitConfig::pure(UnitType::Infantry, 100),
        });

        configs.push(TestConfig {
            name: format!("{}: 100 vs 50 Infantry (overwhelming)", name),
            attacker: UnitConfig::pure(*ut, 100),
            defender: UnitConfig::pure(UnitType::Infantry, 50),
        });
    }

    configs
}

fn main() {
    let args = Args::parse();

    let mut all_results = Vec::new();

    match args.command {
        Commands::Shapes => {
            println!("=== FORMATION SHAPE TESTS ===\n");
            let configs = generate_shape_tests();
            for config in configs {
                let result = run_test(&config, args.trials, args.rounds, args.verbose);
                all_results.push(result);
            }
        }
        Commands::Composition => {
            println!("=== UNIT COMPOSITION TESTS ===\n");
            let configs = generate_composition_tests();
            for config in configs {
                let result = run_test(&config, args.trials, args.rounds, args.verbose);
                all_results.push(result);
            }
        }
        Commands::Equipment => {
            println!("=== EQUIPMENT MATCHUP TESTS ===\n");
            let configs = generate_equipment_tests();
            for config in configs {
                let result = run_test(&config, args.trials, args.rounds, args.verbose);
                all_results.push(result);
            }
        }
        Commands::Terrain => {
            println!("=== TERRAIN EFFECT TESTS ===\n");
            let configs = generate_terrain_tests();
            for config in configs {
                let result = run_test(&config, args.trials, args.rounds, args.verbose);
                all_results.push(result);
            }
        }
        Commands::Morale => {
            println!("=== MORALE CASCADE TESTS ===\n");
            let configs = generate_morale_tests();
            for config in configs {
                let result = run_test(&config, args.trials, args.rounds, args.verbose);
                all_results.push(result);
            }
        }
        Commands::All => {
            println!("=== RUNNING ALL TESTS ===\n");

            println!("\n--- Formation Shapes ---");
            for config in generate_shape_tests() {
                let result = run_test(&config, args.trials, args.rounds, args.verbose);
                all_results.push(result);
            }

            println!("\n--- Unit Composition ---");
            for config in generate_composition_tests() {
                let result = run_test(&config, args.trials, args.rounds, args.verbose);
                all_results.push(result);
            }

            println!("\n--- Equipment Matchups ---");
            for config in generate_equipment_tests() {
                let result = run_test(&config, args.trials, args.rounds, args.verbose);
                all_results.push(result);
            }

            println!("\n--- Terrain Effects ---");
            for config in generate_terrain_tests() {
                let result = run_test(&config, args.trials, args.rounds, args.verbose);
                all_results.push(result);
            }

            println!("\n--- Morale Cascades ---");
            for config in generate_morale_tests() {
                let result = run_test(&config, args.trials, args.rounds, args.verbose);
                all_results.push(result);
            }
        }
    }

    print_results(&all_results, &args.format);

    // Summary statistics
    if !all_results.is_empty() && args.format == "text" {
        println!("\n{:=<80}", "");
        println!("SUMMARY");
        println!("{:=<80}", "");

        // Find best/worst configs
        let best_exchange = all_results
            .iter()
            .filter(|r| r.casualty_exchange_ratio.is_finite())
            .max_by(|a, b| a.casualty_exchange_ratio.partial_cmp(&b.casualty_exchange_ratio).unwrap());

        let worst_exchange = all_results
            .iter()
            .filter(|r| r.casualty_exchange_ratio.is_finite() && r.casualty_exchange_ratio > 0.0)
            .min_by(|a, b| a.casualty_exchange_ratio.partial_cmp(&b.casualty_exchange_ratio).unwrap());

        if let Some(best) = best_exchange {
            println!(
                "Best Exchange Ratio: {} ({:.2}:1)",
                best.config_name, best.casualty_exchange_ratio
            );
        }

        if let Some(worst) = worst_exchange {
            println!(
                "Worst Exchange Ratio: {} ({:.2}:1)",
                worst.config_name, worst.casualty_exchange_ratio
            );
        }

        let highest_win_rate = all_results
            .iter()
            .max_by(|a, b| a.attacker_win_rate.partial_cmp(&b.attacker_win_rate).unwrap());

        if let Some(best) = highest_win_rate {
            println!(
                "Highest Attacker Win Rate: {} ({:.0}%)",
                best.config_name,
                best.attacker_win_rate * 100.0
            );
        }

        println!("\nTotal configurations tested: {}", all_results.len());
        println!("Trials per configuration: {}", args.trials);
    }
}
