//! Headless Battle Runner
//!
//! Runs AI vs AI battles and outputs JSON scores for DSPy optimization.

use arc_citadel::battle::{
    ai::{load_personality, scoring, AiCommander, AiPersonality},
    Army, ArmyId, BattleFormation, BattleMap, BattleState, BattleUnit, Element, FormationId,
    UnitId, UnitType,
};
use arc_citadel::battle::hex::BattleHexCoord;
use arc_citadel::core::types::EntityId;
use clap::Parser;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::Serialize;

/// Headless Battle Runner - AI vs AI battles for optimization
#[derive(Parser, Debug)]
#[command(name = "battle_runner")]
#[command(about = "Run AI vs AI battles and output scores for DSPy optimization")]
struct Args {
    /// Friendly AI personality name (loaded from data/ai_personalities/)
    #[arg(long, default_value = "default")]
    friendly: String,

    /// Enemy AI personality name (loaded from data/ai_personalities/)
    #[arg(long, default_value = "default")]
    enemy: String,

    /// Map width in hexes
    #[arg(long, default_value_t = 80)]
    map_width: u32,

    /// Map height in hexes
    #[arg(long, default_value_t = 40)]
    map_height: u32,

    /// Maximum ticks before timeout (draw)
    #[arg(long, default_value_t = 3000)]
    max_ticks: u64,

    /// Random seed for deterministic runs
    #[arg(long)]
    seed: Option<u64>,

    /// Output format: json or text
    #[arg(long, default_value = "json")]
    format: String,

    /// Enable verbose battle logging for evaluation
    #[arg(long, short = 'v')]
    verbose: bool,

    /// Disable fog of war for both AIs (they see all enemy units)
    #[arg(long)]
    no_fog: bool,
}

/// JSON output structure
#[derive(Serialize)]
struct BattleResult {
    outcome: String,
    ticks: u64,
    friendly_casualties_percent: f32,
    enemy_casualties_percent: f32,
    efficiency_delta: f32,
    score: f32,
    friendly_personality: String,
    enemy_personality: String,
    seed: u64,
}

fn main() {
    let args = Args::parse();

    // Determine seed
    let seed = args.seed.unwrap_or_else(|| rand::random());
    let mut rng = StdRng::seed_from_u64(seed);

    // Load personalities
    let mut friendly_personality = load_personality(&args.friendly).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load friendly personality '{}': {}", args.friendly, e);
        eprintln!("Using default personality");
        AiPersonality::default()
    });

    let mut enemy_personality = load_personality(&args.enemy).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load enemy personality '{}': {}", args.enemy, e);
        eprintln!("Using default personality");
        AiPersonality::default()
    });

    // Apply --no-fog flag to disable fog of war
    if args.no_fog {
        friendly_personality.difficulty.ignores_fog_of_war = true;
        enemy_personality.difficulty.ignores_fog_of_war = true;
    }

    // Create AIs with seed-based RNG
    let friendly_ai = AiCommander::with_seed(friendly_personality.clone(), seed);
    let enemy_ai = AiCommander::with_seed(enemy_personality.clone(), seed.wrapping_add(1));

    // Create map
    let map = BattleMap::new(args.map_width, args.map_height);

    // Create test armies with 3 infantry units each (50 entities per unit)
    // Start armies 60 hexes apart on an 80-wide map (extends approach phase)
    let center_y = (args.map_height / 2) as i32;
    let friendly_army = create_test_army(
        "Friendly",
        BattleHexCoord::new(10, center_y),
        &mut rng,
    );
    let enemy_army = create_test_army(
        "Enemy",
        BattleHexCoord::new(70, center_y),
        &mut rng,
    );

    // Create battle state and attach AIs
    let mut state = BattleState::new(map, friendly_army, enemy_army);
    state.set_friendly_ai(Some(Box::new(friendly_ai)));
    state.set_enemy_ai(Some(Box::new(enemy_ai)));
    state.start_battle();

    // Log initial state if verbose
    if args.verbose {
        eprintln!("=== Battle Started ===");
        eprintln!(
            "Friendly army: {} troops in {} formations",
            state.friendly_army.total_strength(),
            state.friendly_army.formations.len()
        );
        eprintln!(
            "Enemy army: {} troops in {} formations",
            state.enemy_army.total_strength(),
            state.enemy_army.formations.len()
        );
        // Log any initial events (like BattleStarted)
        for event in &state.battle_log {
            eprintln!(
                "  [{}] {:?}: {}",
                event.tick, event.event_type, event.description
            );
        }
        eprintln!();
    }

    // Run battle loop
    while !state.is_finished() && state.tick < args.max_ticks {
        if args.verbose {
            eprintln!("=== Tick {} (time_scale: {:.1}x) ===", state.tick, state.time_scale);
            eprintln!(
                "Friendly strength: {}/{}",
                state.friendly_army.effective_strength(),
                state.friendly_army.total_strength()
            );
            eprintln!(
                "Enemy strength: {}/{}",
                state.enemy_army.effective_strength(),
                state.enemy_army.total_strength()
            );
            eprintln!(
                "Couriers in flight: {}, Friendly couriers: {}, Enemy couriers: {}",
                state.courier_system.in_flight.len(),
                state.friendly_army.courier_pool.len(),
                state.enemy_army.courier_pool.len()
            );
            // Show unit positions and stances
            for formation in &state.friendly_army.formations {
                for unit in &formation.units {
                    eprintln!("  Friendly unit at ({},{}) stance={:?} stress={:.2} casualties={}",
                        unit.position.q, unit.position.r, unit.stance, unit.stress, unit.casualties);
                }
            }
            for formation in &state.enemy_army.formations {
                for unit in &formation.units {
                    eprintln!("  Enemy unit at ({},{}) stance={:?} stress={:.2} casualties={}",
                        unit.position.q, unit.position.r, unit.stance, unit.stress, unit.casualties);
                }
            }
        }

        let events_before = state.battle_log.len();
        let _events = state.run_tick();

        if args.verbose {
            // Print new events
            for event in state.battle_log.iter().skip(events_before) {
                eprintln!(
                    "  [{}] {:?}: {}",
                    event.tick, event.event_type, event.description
                );
            }
        }
    }

    // If battle didn't end naturally, end as timeout (handled by check_battle_end)
    // But score calculation handles Undecided outcome

    // Calculate score
    let weights = scoring::ScoreWeights::default();
    let battle_score = scoring::calculate_score(&state, &weights, args.max_ticks);

    // Output result
    let result = BattleResult {
        outcome: format!("{:?}", battle_score.outcome),
        ticks: battle_score.ticks_taken,
        friendly_casualties_percent: battle_score.friendly_casualties_percent,
        enemy_casualties_percent: battle_score.enemy_casualties_percent,
        efficiency_delta: battle_score.efficiency_delta,
        score: battle_score.raw_score,
        friendly_personality: friendly_personality.name,
        enemy_personality: enemy_personality.name,
        seed,
    };

    match args.format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        "text" => {
            println!("Battle Result");
            println!("=============");
            println!("Outcome: {}", result.outcome);
            println!("Ticks: {}", result.ticks);
            println!("Friendly casualties: {:.1}%", result.friendly_casualties_percent * 100.0);
            println!("Enemy casualties: {:.1}%", result.enemy_casualties_percent * 100.0);
            println!("Efficiency delta: {:.3}", result.efficiency_delta);
            println!("Score: {:.1}", result.score);
            println!();
            println!("Personalities: {} vs {}", result.friendly_personality, result.enemy_personality);
            println!("Seed: {}", result.seed);
        }
        _ => {
            eprintln!("Unknown format '{}', defaulting to json", args.format);
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
    }
}

/// Create a test army with 3 infantry units (50 entities each)
fn create_test_army(name: &str, base_position: BattleHexCoord, _rng: &mut StdRng) -> Army {
    let mut army = Army::new(ArmyId::new(), EntityId::new());
    army.hq_position = base_position;

    // Create a single formation with 3 units
    let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
    formation.name = format!("{} Formation", name);

    // Unit 1: center
    let mut unit1 = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    unit1.position = base_position;
    unit1.elements.push(Element::new(vec![EntityId::new(); 50]));
    formation.units.push(unit1);

    // Unit 2: offset north
    let mut unit2 = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    unit2.position = BattleHexCoord::new(base_position.q, base_position.r - 2);
    unit2.elements.push(Element::new(vec![EntityId::new(); 50]));
    formation.units.push(unit2);

    // Unit 3: offset south
    let mut unit3 = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    unit3.position = BattleHexCoord::new(base_position.q, base_position.r + 2);
    unit3.elements.push(Element::new(vec![EntityId::new(); 50]));
    formation.units.push(unit3);

    army.formations.push(formation);

    // Add some couriers to the pool for AI orders
    for _ in 0..5 {
        army.courier_pool.push(EntityId::new());
    }

    army
}
