//! Campaign layer simulation test
//! Tests army movement, supply, weather, visibility, battle resolution, and scouts

use arc_citadel::campaign::{
    apply_retreat, campaign_tick, resolve_battle, ArmyStance, BattleOutcome, CampaignEvent,
    CampaignMap, CampaignState, HexCoord, RegionalWeather, ScoutSystem, SupplySystem,
    VisibilitySystem,
};
use arc_citadel::core::types::PolityId;
use std::collections::HashSet;
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       ARC CITADEL: FULL CAMPAIGN LAYER SIMULATION            ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Generate a 20x20 hex map
    let map = CampaignMap::generate_simple(20, 20, 42);
    println!("Generated {}x{} campaign map ({} hexes)\n", map.width, map.height, map.hexes.len());

    // Show terrain distribution
    let mut terrain_counts = std::collections::HashMap::new();
    for tile in map.hexes.values() {
        *terrain_counts.entry(format!("{:?}", tile.terrain)).or_insert(0) += 1;
    }
    println!("Terrain distribution:");
    for (terrain, count) in &terrain_counts {
        println!("  {}: {} hexes", terrain, count);
    }
    println!();

    // Initialize all systems
    let mut state = CampaignState::new(map.clone());
    let mut supply_system = SupplySystem::new();
    let mut weather = RegionalWeather::new();
    let mut visibility = VisibilitySystem::new();
    let mut scouts = ScoutSystem::new();

    // Register factions
    visibility.register_faction(PolityId(1));
    visibility.register_faction(PolityId(2));

    // Create supply depots
    let _depot1 = supply_system.create_depot(HexCoord::new(0, 0), PolityId(1));
    let _depot2 = supply_system.create_depot(HexCoord::new(19, 19), PolityId(2));
    println!("Supply depots established at (0,0) and (19,19)\n");

    // Faction 1: Kingdom of the East - starts closer for faster engagement
    let army1 = state.spawn_army(
        "Eastern Legion".to_string(),
        PolityId(1),
        HexCoord::new(5, 5),
    );
    state.get_army_mut(army1).unwrap().unit_count = 300;
    state.get_army_mut(army1).unwrap().stance = ArmyStance::Aggressive;
    supply_system.register_army(army1);
    // Enable foraging to sustain armies during march
    supply_system.get_army_supply_mut(army1).unwrap().foraging = true;

    let army2 = state.spawn_army(
        "Eastern Guard".to_string(),
        PolityId(1),
        HexCoord::new(2, 2),
    );
    state.get_army_mut(army2).unwrap().unit_count = 150;
    state.get_army_mut(army2).unwrap().stance = ArmyStance::Defensive;
    supply_system.register_army(army2);
    supply_system.get_army_supply_mut(army2).unwrap().foraging = true;

    // Faction 2: Western Empire - also starts closer
    let army3 = state.spawn_army(
        "Imperial Host".to_string(),
        PolityId(2),
        HexCoord::new(15, 15),
    );
    state.get_army_mut(army3).unwrap().unit_count = 350;
    state.get_army_mut(army3).unwrap().stance = ArmyStance::Aggressive;
    supply_system.register_army(army3);
    supply_system.get_army_supply_mut(army3).unwrap().foraging = true;

    let army4 = state.spawn_army(
        "Imperial Scouts".to_string(),
        PolityId(2),
        HexCoord::new(12, 12),
    );
    state.get_army_mut(army4).unwrap().unit_count = 50;
    state.get_army_mut(army4).unwrap().stance = ArmyStance::Evasive;
    supply_system.register_army(army4);
    supply_system.get_army_supply_mut(army4).unwrap().foraging = true;

    // Deploy scouts
    let scout1 = scouts.deploy_scout(state.get_army(army1).unwrap());
    let scout2 = scouts.deploy_scout(state.get_army(army3).unwrap());
    println!("Scouts deployed from Eastern Legion and Imperial Host\n");

    // Assign scout missions
    if let Some(s) = scouts.get_scout_mut(scout1) {
        s.assign_recon(HexCoord::new(10, 10), &map);
    }
    if let Some(s) = scouts.get_scout_mut(scout2) {
        s.assign_recon(HexCoord::new(10, 10), &map);
    }

    println!("Armies deployed:");
    for army in &state.armies {
        let supply = supply_system.get_army_supply(army.id);
        let supply_days = supply.map(|s| s.supplies).unwrap_or(0.0);
        println!(
            "  {} (Faction {}) at ({},{}) - {} units, {:?}, {:.0} days supplies",
            army.name, army.faction.0, army.position.q, army.position.r,
            army.unit_count, army.stance, supply_days
        );
    }
    println!();

    // Give movement orders
    println!("Issuing orders...");
    state.get_army_mut(army1).unwrap().order_move_to(HexCoord::new(10, 10), &map);
    println!("  Eastern Legion: Move to (10, 10)");
    state.get_army_mut(army3).unwrap().order_move_to(HexCoord::new(10, 10), &map);
    println!("  Imperial Host: Move to (10, 10)");
    state.get_army_mut(army4).unwrap().order_move_to(HexCoord::new(10, 15), &map);
    println!("  Imperial Scouts: Move to (10, 15)");
    println!();

    // Simulate campaign
    let sim_start = Instant::now();
    let mut total_events = 0;
    let mut arrivals = 0;
    let mut engagements = 0;
    let mut battles_resolved = 0;
    let mut supply_events = 0;
    let mut scout_events = 0;

    println!("═══════════════════════════════════════════════════════════════");
    println!("                     CAMPAIGN SIMULATION");
    println!("═══════════════════════════════════════════════════════════════\n");

    let scout_armies: HashSet<_> = scouts.armies_with_scouts();

    for day in 1..=50 {
        let day_of_year = day as u32;

        // Update weather
        weather.update(1.0, day_of_year, day as u64 * 12345);
        let current_weather = weather.global_weather.current_weather;

        // Update visibility
        visibility.update(&state.armies, &scout_armies, &map, &weather, state.current_day);

        // Process supply
        let sup_events = supply_system.tick(&mut state.armies, &map, 1.0);
        for event in &sup_events {
            match event {
                arc_citadel::campaign::SupplyEvent::ArmyStarving { army, attrition } => {
                    let army_data = state.get_army(*army).unwrap();
                    println!("Day {}: {} is starving! Lost {} soldiers", day, army_data.name, attrition);
                }
                arc_citadel::campaign::SupplyEvent::SuppliesExhausted { army } => {
                    let army_data = state.get_army(*army).unwrap();
                    println!("Day {}: {} has run out of supplies!", day, army_data.name);
                }
                _ => {}
            }
        }
        supply_events += sup_events.len();

        // Process scouts
        let sc_events = scouts.tick(&state.armies, &map, 1.0, state.current_day);
        for event in &sc_events {
            match event {
                arc_citadel::campaign::ScoutEvent::ReconComplete { scout: _, position } => {
                    println!("Day {}: Scout completed recon at ({}, {})", day, position.q, position.r);
                }
                arc_citadel::campaign::ScoutEvent::Detected { scout: _, by_army } => {
                    let army_data = state.get_army(*by_army).unwrap();
                    println!("Day {}: Scout detected by {}!", day, army_data.name);
                }
                arc_citadel::campaign::ScoutEvent::IntelDelivered { scout: _, to_army } => {
                    let army_data = state.get_army(*to_army).unwrap();
                    println!("Day {}: Scout returned with intel to {}", day, army_data.name);
                }
                _ => {}
            }
        }
        scout_events += sc_events.len();

        // Process army movement
        let events = campaign_tick(&mut state, 1.0);

        for event in &events {
            match event {
                CampaignEvent::ArmyArrived { army, position } => {
                    let army_data = state.get_army(*army).unwrap();
                    println!(
                        "Day {}: {} arrived at ({}, {})",
                        day, army_data.name, position.q, position.r
                    );
                    arrivals += 1;
                }
                CampaignEvent::ArmiesEngaged { army_a, army_b, position } => {
                    let a_name = state.get_army(*army_a).unwrap().name.clone();
                    let b_name = state.get_army(*army_b).unwrap().name.clone();
                    println!(
                        "Day {}: BATTLE! {} vs {} at ({}, {}) [Weather: {:?}]",
                        day, a_name, b_name, position.q, position.r, current_weather
                    );

                    // Resolve the battle
                    let (mut attacker, mut defender) = {
                        let a = state.get_army(*army_a).unwrap().clone();
                        let b = state.get_army(*army_b).unwrap().clone();
                        (a, b)
                    };

                    let result = resolve_battle(&mut attacker, &mut defender, &map, current_weather, 10);

                    println!(
                        "  Result: {:?} after {} rounds",
                        result.outcome, result.rounds_fought
                    );
                    println!(
                        "  Casualties: {} lost {}, {} lost {}",
                        a_name, result.attacker_casualties,
                        b_name, result.defender_casualties
                    );

                    // Apply battle results back to state
                    if let Some(a) = state.get_army_mut(*army_a) {
                        a.unit_count = attacker.unit_count;
                        a.morale = attacker.morale;
                    }
                    if let Some(b) = state.get_army_mut(*army_b) {
                        b.unit_count = defender.unit_count;
                        b.morale = defender.morale;
                    }

                    // Apply retreat to routed army
                    match result.outcome {
                        BattleOutcome::AttackerVictory => {
                            // Defender routed - retreat
                            if let Some(b) = state.get_army_mut(*army_b) {
                                apply_retreat(b, *position, &map);
                                println!("  {} retreats to ({}, {})", b_name, b.position.q, b.position.r);
                            }
                        }
                        BattleOutcome::DefenderVictory => {
                            // Attacker routed - retreat
                            if let Some(a) = state.get_army_mut(*army_a) {
                                apply_retreat(a, *position, &map);
                                println!("  {} retreats to ({}, {})", a_name, a.position.q, a.position.r);
                            }
                        }
                        BattleOutcome::Draw => {
                            // Both routed - both retreat
                            if let Some(a) = state.get_army_mut(*army_a) {
                                apply_retreat(a, *position, &map);
                                println!("  {} retreats to ({}, {})", a_name, a.position.q, a.position.r);
                            }
                            if let Some(b) = state.get_army_mut(*army_b) {
                                apply_retreat(b, *position, &map);
                                println!("  {} retreats to ({}, {})", b_name, b.position.q, b.position.r);
                            }
                        }
                        BattleOutcome::Ongoing => {
                            // Battle continues - no retreat yet
                        }
                    }

                    battles_resolved += 1;
                    engagements += 1;
                }
                CampaignEvent::ArmyMoved { army, position } => {
                    // Log weather changes and occasional movement
                    if day % 10 == 0 {
                        let army_data = state.get_army(*army).unwrap();
                        println!(
                            "Day {}: {} at ({}, {}) [Weather: {:?}]",
                            day, army_data.name, position.q, position.r, current_weather
                        );
                    }
                }
            }
        }

        total_events += events.len();
    }

    let elapsed = sim_start.elapsed();

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                     SIMULATION COMPLETE");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Final army positions:");
    for army in &state.armies {
        let terrain = map.get(&army.position).map(|t| format!("{:?}", t.terrain)).unwrap_or_default();
        let supply = supply_system.get_army_supply(army.id);
        let supply_days = supply.map(|s| s.supplies).unwrap_or(0.0);
        let visible_enemies = visibility.visible_enemies(army.faction, &state.armies);
        println!(
            "  {} at ({}, {}) [{}] - {} units, {:.0}% morale, {:.1} days supplies, sees {} enemies",
            army.name, army.position.q, army.position.r, terrain,
            army.unit_count, army.morale * 100.0, supply_days, visible_enemies.len()
        );
    }

    println!("\nScout status:");
    for scout in &scouts.scouts {
        println!(
            "  Scout {} at ({}, {}) - Mission: {:?}, Intel gathered: {}",
            scout.id.0, scout.position.q, scout.position.r,
            scout.mission, scout.intel_gathered.len()
        );
    }

    println!("\nStatistics:");
    println!("  Campaign days: 50");
    println!("  Total movement events: {}", total_events);
    println!("  Arrivals: {}", arrivals);
    println!("  Engagements: {}", engagements);
    println!("  Battles resolved: {}", battles_resolved);
    println!("  Supply events: {}", supply_events);
    println!("  Scout events: {}", scout_events);
    println!("  Simulation time: {:?}", elapsed);
    println!("  Days per second: {:.0}", 50.0 / elapsed.as_secs_f64());

    // Weather test
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                        WEATHER TEST");
    println!("═══════════════════════════════════════════════════════════════\n");

    let mut weather_test = RegionalWeather::new();
    let mut weather_counts = std::collections::HashMap::new();
    for day in 0..360 {
        weather_test.update(1.0, day, day as u64 * 999);
        *weather_counts.entry(format!("{:?}", weather_test.global_weather.current_weather)).or_insert(0) += 1;
    }
    println!("Weather distribution over 360 days:");
    for (w, count) in &weather_counts {
        println!("  {}: {} days", w, count);
    }

    // Visibility test
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                      VISIBILITY TEST");
    println!("═══════════════════════════════════════════════════════════════\n");

    for faction_id in [PolityId(1), PolityId(2)] {
        if let Some(fv) = visibility.get_faction(faction_id) {
            let visible_count = fv.intel.values().filter(|i| i.visibility == arc_citadel::campaign::HexVisibility::Visible).count();
            let explored_count = fv.intel.values().filter(|i| i.visibility == arc_citadel::campaign::HexVisibility::Explored).count();
            println!(
                "Faction {} visibility: {} hexes visible, {} explored, {} total mapped",
                faction_id.0, visible_count, explored_count, fv.explored_count
            );
        }
    }

    // Pathfinding benchmark
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                     PATHFINDING BENCHMARK");
    println!("═══════════════════════════════════════════════════════════════\n");

    let path_start = Instant::now();
    let iterations = 1000;
    for _ in 0..iterations {
        let _ = map.find_path(HexCoord::new(0, 0), HexCoord::new(19, 19));
    }
    let path_elapsed = path_start.elapsed();
    println!("  {} pathfinding operations in {:?}", iterations, path_elapsed);
    println!("  Average: {:.2}µs per path", path_elapsed.as_micros() as f64 / iterations as f64);

    if let Some(path) = map.find_path(HexCoord::new(0, 0), HexCoord::new(19, 19)) {
        println!("\n  Example path (0,0) -> (19,19): {} hexes", path.len());
        let total_cost: f32 = path.iter()
            .filter_map(|c| map.get(c))
            .map(|t| t.terrain.movement_cost())
            .sum();
        println!("  Total movement cost: {:.1} days", total_cost);
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║              FULL CAMPAIGN SIMULATION COMPLETE               ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}
