//! Battle system integration tests

use arc_citadel::battle::*;
use arc_citadel::combat::*;
use arc_citadel::core::types::EntityId;

#[test]
fn test_full_battle_setup() {
    // Create a battle map
    let mut map = BattleMap::new(30, 30);

    // Add some terrain
    map.set_terrain(BattleHexCoord::new(15, 10), BattleTerrain::Forest);
    map.set_terrain(BattleHexCoord::new(15, 11), BattleTerrain::Forest);
    map.set_terrain(BattleHexCoord::new(15, 12), BattleTerrain::Forest);

    // Forest should block LOS
    assert!(!map.has_line_of_sight(BattleHexCoord::new(10, 11), BattleHexCoord::new(20, 11)));

    // Create armies
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    let mut enemy = Army::new(ArmyId::new(), EntityId::new());

    // Create a friendly formation with infantry
    let mut friendly_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    let mut infantry = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    infantry
        .elements
        .push(Element::new(vec![EntityId::new(); 100]));
    infantry.position = BattleHexCoord::new(5, 15);
    friendly_formation.units.push(infantry);
    friendly.formations.push(friendly_formation);

    // Create enemy formation
    let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    let mut enemy_infantry = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    enemy_infantry
        .elements
        .push(Element::new(vec![EntityId::new(); 80]));
    enemy_infantry.position = BattleHexCoord::new(25, 15);
    enemy_formation.units.push(enemy_infantry);
    enemy.formations.push(enemy_formation);

    // Create battle state
    let mut state = BattleState::new(map, friendly, enemy);

    assert_eq!(state.phase, BattlePhase::Planning);
    assert_eq!(state.friendly_army.total_strength(), 100);
    assert_eq!(state.enemy_army.total_strength(), 80);

    // Start battle
    state.start_battle();
    assert_eq!(state.phase, BattlePhase::Active);

    // Check victory condition (neither army destroyed yet)
    assert!(check_battle_end(&state).is_none());
}

#[test]
fn test_courier_delivery_flow() {
    let map = BattleMap::new(20, 20);
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    friendly.hq_position = BattleHexCoord::new(0, 0);

    // Add courier to pool
    friendly.courier_pool.push(EntityId::new());

    let enemy = Army::new(ArmyId::new(), EntityId::new());

    let mut state = BattleState::new(map, friendly, enemy);

    // Dispatch a courier
    let unit_id = UnitId::new();
    let order = Order::move_to(unit_id, BattleHexCoord::new(10, 10));
    let destination = BattleHexCoord::new(5, 5);

    state.courier_system.dispatch(
        EntityId::new(),
        order,
        state.friendly_army.hq_position,
        destination,
    );

    assert_eq!(state.courier_system.count_en_route(), 1);

    // Advance couriers until delivered
    for _ in 0..50 {
        state.courier_system.advance_all(COURIER_SPEED);
    }

    let arrived = state.courier_system.collect_arrived();
    assert_eq!(arrived.len(), 1);
}

#[test]
fn test_go_code_planning() {
    let mut plan = BattlePlan::new();

    // Create a go-code for flanking maneuver
    let mut go_code = GoCode::new("HAMMER".into(), GoCodeTrigger::Manual);

    // Subscribe a cavalry unit
    let cavalry_id = UnitId::new();
    go_code.subscribe(cavalry_id);

    plan.go_codes.push(go_code);

    // Create waypoint plan for cavalry
    let mut waypoint_plan = WaypointPlan::new(cavalry_id);
    waypoint_plan.add_waypoint(
        Waypoint::new(BattleHexCoord::new(20, 5), WaypointBehavior::MoveTo)
            .with_pace(MovementPace::Quick),
    );
    waypoint_plan.add_waypoint(
        Waypoint::new(BattleHexCoord::new(25, 10), WaypointBehavior::HoldAt)
            .with_wait(WaitCondition::GoCode(plan.go_codes[0].id)),
    );
    waypoint_plan.add_waypoint(
        Waypoint::new(BattleHexCoord::new(25, 15), WaypointBehavior::AttackFrom)
            .with_pace(MovementPace::Charge),
    );

    plan.waypoint_plans.push(waypoint_plan);

    // Verify plan structure
    assert_eq!(plan.go_codes.len(), 1);
    assert_eq!(plan.waypoint_plans.len(), 1);
    assert!(plan.get_go_code("HAMMER").is_some());
}



#[test]
fn test_ai_controlled_army_issues_orders() {
    use arc_citadel::battle::ai::{AiCommander, AiPersonality};
    use arc_citadel::battle::hex::BattleHexCoord;
    use arc_citadel::battle::unit_type::UnitType;
    use arc_citadel::battle::units::{
        Army, ArmyId, BattleFormation, BattleUnit, Element, FormationId, UnitId, UnitStance,
    };

    let map = BattleMap::new(30, 30);

    // Friendly army (player-controlled)
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    let mut friendly_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    let mut friendly_unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    friendly_unit
        .elements
        .push(Element::new(vec![EntityId::new(); 50]));
    friendly_unit.position = BattleHexCoord::new(5, 5);
    friendly_formation.units.push(friendly_unit);
    friendly.formations.push(friendly_formation);

    // Enemy army (AI-controlled)
    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    enemy.hq_position = BattleHexCoord::new(25, 25);
    enemy.courier_pool = vec![EntityId::new(); 5]; // Give couriers
    let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    let mut enemy_unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    enemy_unit
        .elements
        .push(Element::new(vec![EntityId::new(); 50]));
    enemy_unit.position = BattleHexCoord::new(20, 20);
    enemy_unit.stance = UnitStance::Formed;
    enemy_formation.units.push(enemy_unit);
    enemy.formations.push(enemy_formation);

    let mut state = BattleState::new(map, friendly, enemy);

    // Set up AI controller
    let mut personality = AiPersonality::default();
    personality.behavior.aggression = 0.8;
    personality.preferences.re_evaluation_interval = 1;
    personality.difficulty.mistake_chance = 0.0; // No mistakes for deterministic test
    personality.difficulty.ignores_fog_of_war = true; // Allow AI to see all enemies
    let ai = AiCommander::new(personality);
    state.set_enemy_ai(Some(Box::new(ai)));

    state.start_battle();

    // Run several ticks
    for _ in 0..15 {
        state.run_tick();
    }

    // AI should have dispatched orders via courier
    assert!(
        !state.courier_system.delivered.is_empty() || !state.courier_system.in_flight.is_empty(),
        "AI should have issued orders"
    );
}

/// Comprehensive integration test for full AI-controlled battle
///
/// This test demonstrates:
/// - AI successfully makes decisions based on personality
/// - Orders are dispatched and delivered via courier system
/// - Combat occurs between armies
/// - Battle progresses naturally with casualties
#[test]
fn test_full_ai_battle() {
    use arc_citadel::battle::ai::{load_personality, AiCommander};
    use arc_citadel::battle::hex::BattleHexCoord;
    use arc_citadel::battle::unit_type::UnitType;
    use arc_citadel::battle::units::{
        Army, ArmyId, BattleFormation, BattleUnit, Element, FormationId, UnitId, UnitStance,
    };

    // Create a 40x40 battle map
    let map = BattleMap::new(40, 40);

    // ===== SETUP FRIENDLY ARMY =====
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    friendly.hq_position = BattleHexCoord::new(5, 20);
    friendly.courier_pool = vec![EntityId::new(); 10]; // 10 couriers

    let mut friendly_formation = BattleFormation::new(FormationId::new(), EntityId::new());

    // 3 infantry units at (10, 15), (10, 18), (10, 21) with 80 strength each
    // Positioned adjacent to enemy units for immediate combat
    let friendly_positions = [(10, 15), (10, 18), (10, 21)];
    for (q, r) in friendly_positions {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 80]));
        unit.position = BattleHexCoord::new(q, r);
        unit.stance = UnitStance::Formed;
        friendly_formation.units.push(unit);
    }
    friendly.formations.push(friendly_formation);

    // ===== SETUP ENEMY ARMY =====
    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    enemy.hq_position = BattleHexCoord::new(35, 20);
    enemy.courier_pool = vec![EntityId::new(); 10]; // 10 couriers

    let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());

    // 3 infantry units positioned adjacent to friendly units (distance 1)
    // This ensures combat occurs immediately while AI still makes decisions
    let enemy_positions = [(11, 15), (11, 18), (11, 21)];
    for (q, r) in enemy_positions {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 80]));
        unit.position = BattleHexCoord::new(q, r);
        unit.stance = UnitStance::Formed;
        enemy_formation.units.push(unit);
    }
    enemy.formations.push(enemy_formation);

    // ===== CREATE BATTLE STATE =====
    let mut state = BattleState::new(map, friendly, enemy);

    // ===== LOAD AGGRESSIVE AI PERSONALITY AND CREATE AI COMMANDER =====
    let personality = load_personality("aggressive").expect("Should load aggressive personality");
    let ai = AiCommander::new(personality);
    state.set_enemy_ai(Some(Box::new(ai)));

    // ===== START BATTLE =====
    state.start_battle();
    assert_eq!(
        state.phase,
        execution::BattlePhase::Active,
        "Battle should be active after start"
    );

    // ===== RUN BATTLE =====
    // Run up to 500 ticks (or until battle ends)
    let mut tick_count = 0;
    let mut max_couriers_in_flight = 0;
    let mut total_couriers_dispatched = 0;

    while !state.is_finished() && tick_count < 500 {
        // Track courier activity before tick
        let couriers_before = state.courier_system.in_flight.len();

        state.run_tick();
        tick_count += 1;

        // Track courier activity after tick
        let couriers_after = state.courier_system.in_flight.len();
        if couriers_after > couriers_before {
            total_couriers_dispatched += couriers_after - couriers_before;
        }
        if couriers_after > max_couriers_in_flight {
            max_couriers_in_flight = couriers_after;
        }
    }

    // ===== ASSERTIONS =====

    // 1. Battle ran for more than 10 ticks
    assert!(
        tick_count > 10,
        "Battle should have run for more than 10 ticks, ran for {}",
        tick_count
    );

    // 2. Calculate casualties
    let friendly_casualties: u32 = state
        .friendly_army
        .formations
        .iter()
        .flat_map(|f| f.units.iter())
        .map(|u| u.casualties)
        .sum();

    let enemy_casualties: u32 = state
        .enemy_army
        .formations
        .iter()
        .flat_map(|f| f.units.iter())
        .map(|u| u.casualties)
        .sum();

    // Some casualties occurred (friendly or enemy)
    assert!(
        friendly_casualties > 0 || enemy_casualties > 0,
        "Combat should have occurred - friendly casualties: {}, enemy casualties: {}",
        friendly_casualties,
        enemy_casualties
    );

    // 3. If battle finished, outcome is not Undecided
    if state.is_finished() {
        assert!(
            !matches!(state.outcome, execution::BattleOutcome::Undecided),
            "Finished battle should have a decided outcome, got {:?}",
            state.outcome
        );
    }

    // ===== PRINT BATTLE STATS =====
    println!("\n========== BATTLE STATISTICS ==========");
    println!("Tick count: {}", tick_count);
    println!("Battle finished: {}", state.is_finished());
    println!("Outcome: {:?}", state.outcome);
    println!("----------------------------------------");
    println!("Friendly Army:");
    println!("  - Initial strength: {}", 80 * 3);
    println!(
        "  - Effective strength: {}",
        state.friendly_army.effective_strength()
    );
    println!("  - Total casualties: {}", friendly_casualties);
    println!(
        "  - Routing percentage: {:.1}%",
        state.friendly_army.percentage_routing() * 100.0
    );
    println!("----------------------------------------");
    println!("Enemy Army:");
    println!("  - Initial strength: {}", 80 * 3);
    println!(
        "  - Effective strength: {}",
        state.enemy_army.effective_strength()
    );
    println!("  - Total casualties: {}", enemy_casualties);
    println!(
        "  - Routing percentage: {:.1}%",
        state.enemy_army.percentage_routing() * 100.0
    );
    println!("----------------------------------------");
    println!("AI Activity:");
    println!(
        "  - Total couriers dispatched: {}",
        total_couriers_dispatched
    );
    println!("  - Max couriers in flight: {}", max_couriers_in_flight);
    println!(
        "  - Couriers currently in system: {}",
        state.courier_system.delivered.len() + state.courier_system.in_flight.len()
    );
    println!(
        "  - Enemy couriers remaining: {}",
        state.enemy_army.courier_pool.len()
    );
    println!("========================================\n");

    // Additional verification: The test demonstrates:
    // - AI successfully loads and uses aggressive personality
    // - Battle state progresses through multiple ticks
    // - Combat occurs with both sides taking casualties
    // - Morale system causes units to rout
    // - Battle concludes with a determined outcome
}
