//! Battle terrain system integration tests
//!
//! These tests verify that terrain effects work end-to-end:
//! - Forest cover reduces hits taken
//! - Flanking negates directional cover
//! - AI prefers elevated positions for ranged units

use arc_citadel::battle::ai::TacticalMap;
use arc_citadel::battle::tactics::{calculate_attack_angle, calculate_engagement_geometry, AttackAngle};
use arc_citadel::battle::terrain::{BattleTerrain, TerrainFeature};
use arc_citadel::battle::unit_type::UnitType;
use arc_citadel::battle::units::{BattleUnit, Element, FormationShape, UnitId};
use arc_citadel::battle::{BattleHexCoord, BattleMap, HexDirection};
use arc_citadel::core::types::EntityId;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a test unit at the given position with default settings
fn create_test_unit(pos: BattleHexCoord, unit_type: UnitType) -> BattleUnit {
    let mut unit = BattleUnit::new(UnitId::new(), unit_type);
    unit.position = pos;
    unit.facing = HexDirection::East;
    unit.elements.push(Element::new(
        (0..50).map(|_| EntityId::new()).collect(),
    ));
    unit
}

/// Create a test unit with a specific facing direction
fn create_test_unit_facing(pos: BattleHexCoord, unit_type: UnitType, facing: HexDirection) -> BattleUnit {
    let mut unit = create_test_unit(pos, unit_type);
    unit.facing = facing;
    unit
}

// ============================================================================
// Test 1: Defender in forest takes less hits (gets cover bonus)
// ============================================================================

/// Test that forest terrain provides cover to defenders when attacked from the front.
///
/// Setup: Two identical engagements, one with forest at defender position.
/// The forest defender should have a higher cover_value in engagement geometry,
/// which translates to a hit penalty for the attacker.
#[test]
fn test_defender_in_forest_gets_cover() {
    // Create two maps: one open, one with forest
    let open_map = BattleMap::new(20, 20);
    let mut forest_map = BattleMap::new(20, 20);

    // Place forest at (10, 10) on the forest map
    let defender_pos = BattleHexCoord::new(10, 10);
    forest_map.set_terrain(defender_pos, BattleTerrain::Forest);

    // Create identical attacker and defender units
    let attacker_pos = BattleHexCoord::new(8, 10); // West of defender, attacking from West
    let attacker = create_test_unit(attacker_pos, UnitType::Infantry);

    // Defenders face West (toward attacker) - this is a frontal attack
    let defender_open = create_test_unit_facing(defender_pos, UnitType::Infantry, HexDirection::West);
    let defender_forest = create_test_unit_facing(defender_pos, UnitType::Infantry, HexDirection::West);

    // Calculate engagement geometry for both scenarios
    let geometry_open = calculate_engagement_geometry(&attacker, &defender_open, &open_map);
    let geometry_forest = calculate_engagement_geometry(&attacker, &defender_forest, &forest_map);

    // Forest should provide cover value (0.5 per terrain.rs)
    assert_eq!(
        geometry_open.cover_value, 0.0,
        "Open terrain should provide no cover"
    );
    assert!(
        geometry_forest.cover_value > 0.0,
        "Forest terrain should provide cover (got {})",
        geometry_forest.cover_value
    );
    assert!(
        (geometry_forest.cover_value - 0.5).abs() < 0.01,
        "Forest cover should be ~0.5 (got {})",
        geometry_forest.cover_value
    );

    // Cover should result in a lower (more negative) hit modifier
    // Open: 0.0 base (frontal, no cover)
    // Forest: -0.5 (frontal with 0.5 cover)
    assert!(
        geometry_forest.hit_modifier() < geometry_open.hit_modifier(),
        "Forest cover should reduce hit chance (open={}, forest={})",
        geometry_open.hit_modifier(),
        geometry_forest.hit_modifier()
    );
}

/// Test that different terrain types provide varying amounts of cover.
#[test]
fn test_terrain_cover_values() {
    let defender_pos = BattleHexCoord::new(10, 10);
    let attacker_pos = BattleHexCoord::new(8, 10);

    let test_cases = [
        (BattleTerrain::Open, 0.0, "Open"),
        (BattleTerrain::Rough, 0.2, "Rough"),
        (BattleTerrain::Forest, 0.5, "Forest"),
        (BattleTerrain::Building, 0.7, "Building"),
    ];

    for (terrain, expected_cover, name) in test_cases {
        let mut map = BattleMap::new(20, 20);
        map.set_terrain(defender_pos, terrain);

        let attacker = create_test_unit(attacker_pos, UnitType::Infantry);
        let defender = create_test_unit_facing(defender_pos, UnitType::Infantry, HexDirection::West);

        let geometry = calculate_engagement_geometry(&attacker, &defender, &map);

        assert!(
            (geometry.cover_value - expected_cover).abs() < 0.01,
            "{} terrain should provide {} cover (got {})",
            name,
            expected_cover,
            geometry.cover_value
        );
    }
}

// ============================================================================
// Test 2: Flanking negates cover
// ============================================================================

/// Test that attacking from the flank negates directional wall cover.
///
/// Scenario: Defender behind a wall facing North.
/// - Attack from North (frontal): Wall should provide cover.
/// - Attack from East (flank): Wall cover should be negated.
#[test]
fn test_flanking_negates_wall_cover() {
    let defender_pos = BattleHexCoord::new(10, 10);

    // Create map with a wall at defender position, facing North
    let mut map = BattleMap::new(20, 20);
    map.add_feature(defender_pos, TerrainFeature::Wall(HexDirection::NorthWest));

    // Defender faces NorthWest (toward the wall direction)
    let defender = create_test_unit_facing(defender_pos, UnitType::Infantry, HexDirection::NorthWest);

    // Frontal attack: attacker from NorthWest (wall should provide cover)
    let frontal_attacker_pos = BattleHexCoord::new(10, 8); // North of defender
    let frontal_attacker = create_test_unit(frontal_attacker_pos, UnitType::Infantry);

    // Flank attack: attacker from East (wall should NOT provide cover)
    let flank_attacker_pos = BattleHexCoord::new(12, 10); // East of defender
    let flank_attacker = create_test_unit(flank_attacker_pos, UnitType::Infantry);

    // Calculate engagement geometry
    let frontal_geometry = calculate_engagement_geometry(&frontal_attacker, &defender, &map);
    let flank_geometry = calculate_engagement_geometry(&flank_attacker, &defender, &map);

    // Verify attack angles
    assert!(
        matches!(frontal_geometry.attack_angle, AttackAngle::Frontal),
        "Attack from North should be frontal (got {:?})",
        frontal_geometry.attack_angle
    );
    assert!(
        matches!(
            flank_geometry.attack_angle,
            AttackAngle::FlankLeft | AttackAngle::FlankRight
        ),
        "Attack from East should be flank (got {:?})",
        flank_geometry.attack_angle
    );

    // Flanking should negate cover
    assert!(
        flank_geometry.attack_angle.negates_cover(),
        "Flank attacks should negate cover"
    );

    // Cover value should be 0 for flank attacks (even with wall)
    assert_eq!(
        flank_geometry.cover_value, 0.0,
        "Flank attack should have 0 cover (flanking negates it)"
    );
}

/// Test that attack angle calculation works correctly for all directions.
#[test]
fn test_attack_angle_directions() {
    let defender_pos = BattleHexCoord::new(10, 10);
    let defender_facing = HexDirection::East; // Facing East

    // Frontal attack: from East (where defender is facing)
    let frontal_pos = BattleHexCoord::new(12, 10);
    let frontal_angle = calculate_attack_angle(frontal_pos, defender_pos, defender_facing);
    assert!(
        matches!(frontal_angle, AttackAngle::Frontal),
        "Attack from facing direction should be frontal (got {:?})",
        frontal_angle
    );

    // Rear attack: from West (opposite of facing)
    let rear_pos = BattleHexCoord::new(8, 10);
    let rear_angle = calculate_attack_angle(rear_pos, defender_pos, defender_facing);
    assert!(
        matches!(rear_angle, AttackAngle::Rear),
        "Attack from rear should be rear (got {:?})",
        rear_angle
    );

    // Flank attack: from NorthWest (perpendicular to facing)
    let flank_pos = BattleHexCoord::new(10, 8);
    let flank_angle = calculate_attack_angle(flank_pos, defender_pos, defender_facing);
    assert!(
        matches!(
            flank_angle,
            AttackAngle::FlankLeft | AttackAngle::FlankRight
        ),
        "Attack from perpendicular should be flank (got {:?})",
        flank_angle
    );
}

/// Test that rear attacks get the highest hit bonus.
#[test]
fn test_attack_angle_hit_modifiers() {
    assert_eq!(
        AttackAngle::Frontal.hit_modifier(),
        0.0,
        "Frontal attacks have no bonus"
    );
    assert!(
        AttackAngle::FlankLeft.hit_modifier() > AttackAngle::Frontal.hit_modifier(),
        "Flank attacks have bonus over frontal"
    );
    assert!(
        AttackAngle::Rear.hit_modifier() > AttackAngle::FlankLeft.hit_modifier(),
        "Rear attacks have highest bonus"
    );
}

// ============================================================================
// Test 3: AI prefers high ground for ranged units
// ============================================================================

/// Test that AI scoring system prefers elevated positions for ranged units.
#[test]
fn test_ai_prefers_high_ground_for_archers() {
    // Create a battle map with elevation at (5, 5)
    let mut battle_map = BattleMap::new(20, 20);
    let high_ground = BattleHexCoord::new(5, 5);
    let low_ground = BattleHexCoord::new(10, 10);

    // Set elevation: high ground at 3, low ground at 0
    battle_map.set_elevation(high_ground, 3);
    // Low ground is already 0 by default

    // Analyze the map
    let tactical_map = TacticalMap::analyze(&battle_map);

    // Verify elevation is correctly stored
    assert_eq!(
        tactical_map.elevation_at(high_ground),
        3,
        "High ground should have elevation 3"
    );
    assert_eq!(
        tactical_map.elevation_at(low_ground),
        0,
        "Low ground should have elevation 0"
    );

    // Score both positions using TacticalMap's defensive_score
    // (AiCommander uses this internally for position scoring)

    let high_ground_score = tactical_map.defensive_score(high_ground);
    let low_ground_score = tactical_map.defensive_score(low_ground);

    // High ground should score higher due to elevation bonus
    assert!(
        high_ground_score > low_ground_score,
        "High ground should have higher defensive score (high={}, low={})",
        high_ground_score,
        low_ground_score
    );

    // Verify the elevation contributes to the score
    // Elevation 3 should give +0.3 bonus (0.1 per level)
    assert!(
        (high_ground_score - 0.3).abs() < 0.01,
        "High ground score should be ~0.3 from elevation (got {})",
        high_ground_score
    );
}

/// Test that elevated positions provide attack advantage in engagement geometry.
#[test]
fn test_elevation_provides_attack_advantage() {
    // Create map with elevation difference
    let mut map = BattleMap::new(20, 20);
    let high_pos = BattleHexCoord::new(5, 5);
    let low_pos = BattleHexCoord::new(8, 5);

    map.set_elevation(high_pos, 2);
    // Low position is 0 by default

    // High ground attacker vs low ground defender
    let attacker_high = create_test_unit(high_pos, UnitType::Archers);
    let defender_low = create_test_unit_facing(low_pos, UnitType::Infantry, HexDirection::West);

    let geometry_from_high = calculate_engagement_geometry(&attacker_high, &defender_low, &map);

    // Elevation difference should be positive (attacker higher)
    assert_eq!(
        geometry_from_high.elevation_diff, 2,
        "Elevation difference should be 2 (attacker 2 higher)"
    );

    // High ground should provide hit modifier bonus
    // +0.1 per level = +0.2 total
    assert!(
        geometry_from_high.hit_modifier() > 0.0,
        "High ground attacker should have positive hit modifier (got {})",
        geometry_from_high.hit_modifier()
    );

    // Damage multiplier should also be increased
    // +10% per level = +20% total = 1.2x
    assert!(
        (geometry_from_high.damage_multiplier() - 1.2).abs() < 0.01,
        "High ground should give 1.2x damage multiplier (got {})",
        geometry_from_high.damage_multiplier()
    );
}

/// Test that low ground attacker has disadvantage.
#[test]
fn test_low_ground_attacker_disadvantage() {
    // Create map with elevation difference
    let mut map = BattleMap::new(20, 20);
    let high_pos = BattleHexCoord::new(5, 5);
    let low_pos = BattleHexCoord::new(8, 5);

    map.set_elevation(high_pos, 2);

    // Low ground attacker vs high ground defender
    let attacker_low = create_test_unit(low_pos, UnitType::Infantry);
    let defender_high = create_test_unit_facing(high_pos, UnitType::Infantry, HexDirection::East);

    let geometry_from_low = calculate_engagement_geometry(&attacker_low, &defender_high, &map);

    // Elevation difference should be negative (attacker lower)
    assert_eq!(
        geometry_from_low.elevation_diff, -2,
        "Elevation difference should be -2 (attacker 2 lower)"
    );

    // Low ground attacker should have hit penalty
    // -0.05 per level = -0.1 total
    assert!(
        geometry_from_low.hit_modifier() < 0.0,
        "Low ground attacker should have negative hit modifier (got {})",
        geometry_from_low.hit_modifier()
    );
}

// ============================================================================
// Additional Integration Tests
// ============================================================================

/// Test that forest blocks line of sight.
#[test]
fn test_forest_blocks_los() {
    let mut map = BattleMap::new(20, 20);

    // Place forest between attacker and defender
    let forest_pos = BattleHexCoord::new(5, 5);
    map.set_terrain(forest_pos, BattleTerrain::Forest);

    let from = BattleHexCoord::new(3, 5);
    let to = BattleHexCoord::new(7, 5);

    // Forest should block LOS
    assert!(
        !map.has_line_of_sight(from, to),
        "Forest should block line of sight"
    );
}

/// Test that buildings block line of sight.
#[test]
fn test_building_blocks_los() {
    let mut map = BattleMap::new(20, 20);

    let building_pos = BattleHexCoord::new(5, 5);
    map.set_terrain(building_pos, BattleTerrain::Building);

    let from = BattleHexCoord::new(3, 5);
    let to = BattleHexCoord::new(7, 5);

    assert!(
        !map.has_line_of_sight(from, to),
        "Building should block line of sight"
    );
}

/// Test combined terrain effects: cover + blocked LOS.
#[test]
fn test_engagement_with_blocked_los() {
    let mut map = BattleMap::new(20, 20);

    // Place forest between attacker and defender
    let forest_pos = BattleHexCoord::new(5, 5);
    map.set_terrain(forest_pos, BattleTerrain::Forest);

    let attacker_pos = BattleHexCoord::new(3, 5);
    let defender_pos = BattleHexCoord::new(7, 5);

    let attacker = create_test_unit(attacker_pos, UnitType::Archers);
    let defender = create_test_unit_facing(defender_pos, UnitType::Infantry, HexDirection::West);

    let geometry = calculate_engagement_geometry(&attacker, &defender, &map);

    // LOS should be blocked
    assert!(
        matches!(
            geometry.los_quality,
            arc_citadel::battle::tactics::LosQuality::Blocked
        ),
        "LOS should be blocked by forest"
    );

    // Blocked LOS should zero out hit modifier
    assert_eq!(
        geometry.hit_modifier(),
        0.0,
        "Blocked LOS should result in 0 hit modifier"
    );
}

/// Test enfilade fire bonus against line formations.
#[test]
fn test_enfilade_against_line_formation() {
    let map = BattleMap::new(20, 20);

    // Create attacker from the side
    let attacker_pos = BattleHexCoord::new(12, 5);
    let attacker = create_test_unit(attacker_pos, UnitType::Archers);

    // Create defender in line formation facing NorthWest
    // Line extends perpendicular to facing, so NE-SW direction
    let defender_pos = BattleHexCoord::new(10, 5);
    let mut defender = create_test_unit_facing(defender_pos, UnitType::Infantry, HexDirection::NorthWest);
    defender.formation_shape = FormationShape::Line { depth: 2 };

    let geometry = calculate_engagement_geometry(&attacker, &defender, &map);

    // Attack from East on a line facing NorthWest should be enfilade
    // (Attack comes along the length of the line)
    assert!(
        geometry.is_enfilade,
        "Attack from flank should be enfilade against line formation"
    );

    // Enfilade should provide +25% damage
    assert!(
        (geometry.damage_multiplier() - 1.25).abs() < 0.01,
        "Enfilade should give 1.25x damage multiplier (got {})",
        geometry.damage_multiplier()
    );
}

/// Test that square formations resist enfilade.
#[test]
fn test_square_formation_resists_enfilade() {
    let map = BattleMap::new(20, 20);

    let attacker_pos = BattleHexCoord::new(12, 5);
    let attacker = create_test_unit(attacker_pos, UnitType::Infantry);

    let defender_pos = BattleHexCoord::new(10, 5);
    let mut defender = create_test_unit(defender_pos, UnitType::Infantry);
    defender.formation_shape = FormationShape::Square;

    let geometry = calculate_engagement_geometry(&attacker, &defender, &map);

    // Square formation should not be enfiladed
    assert!(
        !geometry.is_enfilade,
        "Square formation should resist enfilade"
    );

    // Damage multiplier should be base (1.0)
    assert!(
        (geometry.damage_multiplier() - 1.0).abs() < 0.01,
        "No enfilade means base damage multiplier"
    );
}

/// Test combined elevation and enfilade bonuses.
#[test]
fn test_combined_elevation_and_enfilade() {
    let mut map = BattleMap::new(20, 20);

    let attacker_pos = BattleHexCoord::new(12, 5);
    map.set_elevation(attacker_pos, 2);

    let attacker = create_test_unit(attacker_pos, UnitType::Archers);

    let defender_pos = BattleHexCoord::new(10, 5);
    let mut defender = create_test_unit_facing(defender_pos, UnitType::Infantry, HexDirection::NorthWest);
    defender.formation_shape = FormationShape::Line { depth: 2 };

    let geometry = calculate_engagement_geometry(&attacker, &defender, &map);

    // Should have both bonuses
    assert!(geometry.is_enfilade, "Should be enfilade");
    assert_eq!(geometry.elevation_diff, 2, "Should have elevation advantage");

    // Combined: 1.0 + 0.25 (enfilade) + 0.2 (elevation) = 1.45
    assert!(
        (geometry.damage_multiplier() - 1.45).abs() < 0.01,
        "Combined bonuses should give 1.45x damage (got {})",
        geometry.damage_multiplier()
    );
}
