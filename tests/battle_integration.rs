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
fn test_combat_resolution_no_percentage_modifiers() {
    // This test ensures we're using additive, not multiplicative modifiers

    let weapon = WeaponProperties::sword();
    let armor = ArmorProperties::leather();

    // Get rates at different pressures
    let rates: Vec<f32> = (-5..=5)
        .map(|p| calculate_casualty_rate(&weapon, &armor, p as f32 * 0.1))
        .collect();

    // Calculate deltas between adjacent pressures
    let deltas: Vec<f32> = rates.windows(2).map(|w| w[1] - w[0]).collect();

    // All deltas should be approximately equal (additive behavior)
    let avg_delta: f32 = deltas.iter().sum::<f32>() / deltas.len() as f32;
    for delta in &deltas {
        assert!(
            (delta - avg_delta).abs() < 0.001,
            "Deltas should be consistent for additive behavior"
        );
    }
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
