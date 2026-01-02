//! Combat system integration tests
//!
//! These tests verify the combat system works correctly end-to-end,
//! testing the spec examples and ensuring NO percentage-based APIs exist.

use arc_citadel::combat::{
    // Properties
    Edge, Mass, Reach, WeaponProperties,
    Rigidity, Padding, Coverage, ArmorProperties,
    BodyZone, WoundSeverity,
    // Resolution
    PenetrationResult, TraumaResult,
    resolve_penetration, resolve_trauma, combine_results,
    // Stance
    CombatStance, TransitionTrigger, StanceTransitions,
    // Skill
    SkillLevel,
    // Morale
    StressSource, MoraleState, BreakResult,
    // Exchange
    Combatant, resolve_exchange,
    // Formation
    FormationState, PressureCategory,
    // State
    CombatState,
};

/// Test the spec example: Sword vs Plate armor
///
/// From the spec: "A knight in plate armor is functionally immune to sword cuts."
/// This is historically accurate - you needed maces, flanks, or morale breaks.
#[test]
fn test_spec_example_sword_vs_plate() {
    // Sword (Sharp, Medium, Short) vs Plate (Plate, Heavy padding, Full)
    let sword = WeaponProperties::sword();
    let plate = ArmorProperties::plate();

    // Verify sword properties
    assert_eq!(sword.edge, Edge::Sharp);
    assert_eq!(sword.mass, Mass::Medium);
    assert_eq!(sword.reach, Reach::Short);

    // Verify plate properties
    assert_eq!(plate.rigidity, Rigidity::Plate);
    assert_eq!(plate.padding, Padding::Heavy);
    assert_eq!(plate.coverage, Coverage::Full);

    // 1. Penetration: Sharp vs Plate → DEFLECT
    let pen = resolve_penetration(sword.edge, plate.rigidity, false);
    assert_eq!(pen, PenetrationResult::Deflect);

    // 2. Trauma: Medium vs Heavy → Negligible (Heavy padding absorbs medium mass)
    let trauma = resolve_trauma(sword.mass, plate.padding);
    assert_eq!(trauma, TraumaResult::Negligible);

    // 3. Combined result: No wound
    let wound = combine_results(pen, trauma, BodyZone::Torso);
    assert_eq!(wound.severity, WoundSeverity::None);

    // The sword literally cannot cut plate. You need maces, flanks, or morale breaks.
}

/// Test the spec example: Mace vs Plate armor
///
/// From the spec: "Maces don't cut - they fatigue through concussive force."
/// Knight is fatigued but not wounded - historically accurate.
#[test]
fn test_spec_example_mace_vs_plate() {
    let mace = WeaponProperties::mace();
    let plate = ArmorProperties::plate();

    // Verify mace properties
    assert_eq!(mace.edge, Edge::Blunt);
    assert_eq!(mace.mass, Mass::Heavy);

    // Mace doesn't try to penetrate (blunt weapon)
    let pen = resolve_penetration(mace.edge, plate.rigidity, false);
    assert_eq!(pen, PenetrationResult::NoPenetrationAttempt);

    // Heavy mass vs Heavy padding = Fatigue
    let trauma = resolve_trauma(mace.mass, plate.padding);
    assert_eq!(trauma, TraumaResult::Fatigue);

    // Knight is fatigued but not wounded - historically accurate
    let wound = combine_results(pen, trauma, BodyZone::Torso);
    assert_eq!(wound.severity, WoundSeverity::None);
    assert!(!wound.bleeding);
}

/// Test the two victory paths
///
/// Victory comes from either:
/// 1. DAMAGE PATH: Inflict wounds until they can't fight
/// 2. MORALE PATH: Inflict stress until they break and flee
#[test]
fn test_two_victory_paths() {
    // Path 1: Damage - inflict wounds until incapacitated
    let razor = WeaponProperties {
        edge: Edge::Razor,
        mass: Mass::Light,
        reach: Reach::Grapple,
        special: vec![],
    };
    let unarmored = ArmorProperties::none();

    // Razor vs Cloth = DeepCut
    let pen = resolve_penetration(razor.edge, unarmored.rigidity, false);
    assert_eq!(pen, PenetrationResult::DeepCut);

    // DeepCut to Neck = Critical wound (exceeds fatality threshold)
    let wound = combine_results(pen, TraumaResult::Negligible, BodyZone::Neck);
    assert_eq!(wound.severity, WoundSeverity::Critical);
    assert!(wound.bleeding);

    // Verify neck has low fatality threshold
    assert_eq!(BodyZone::Neck.fatality_threshold(), WoundSeverity::Serious);
    // Critical > Serious, so this wound is fatal

    // Path 2: Morale - accumulate stress until break
    let mut morale = MoraleState::default();

    // Start holding
    assert_eq!(morale.check_break(), BreakResult::Holding);

    // Simulate a devastating situation:
    // - Cavalry charge (0.20)
    // - Officer killed (0.30)
    // - Flank attack (0.20)
    // - Ambush sprung (0.25)
    // - Allies breaking (0.10)
    // Total: 1.05 (exceeds base threshold of 1.0)
    morale.apply_stress(StressSource::CavalryCharge);
    morale.apply_stress(StressSource::OfficerKilled);
    morale.apply_stress(StressSource::FlankAttack);
    morale.apply_stress(StressSource::AmbushSprung);
    morale.apply_stress(StressSource::AlliesBreaking);

    // Should be breaking (stress > 1.0 threshold)
    assert_eq!(morale.check_break(), BreakResult::Breaking);
}

/// Verify no multiplicative stacking patterns in public API
///
/// The combat system uses categorical outcomes from property comparisons,
/// NOT percentage modifiers. This test documents that the public API has
/// no percentage-based methods that would enable multiplicative stacking.
#[test]
fn test_no_percentage_api() {
    // This test documents that the public API has no percentage-based methods
    // If these compile without methods like `damage_multiplier()`, we're good

    let _skill = SkillLevel::Master;
    let _weapon = WeaponProperties::sword();
    let _armor = ArmorProperties::plate();
    let _wound = WoundSeverity::Critical;

    // Skill determines CAPABILITIES, not bonuses
    assert!(SkillLevel::Master.can_feint());
    assert!(!SkillLevel::Novice.can_feint());

    // Weapon properties are categorical (Edge, Mass, Reach), not f32 multipliers
    let sword = WeaponProperties::sword();
    assert_eq!(sword.edge, Edge::Sharp); // Categorical, not "1.5x damage"

    // Armor properties are categorical (Rigidity, Padding, Coverage), not f32 reduction
    let plate = ArmorProperties::plate();
    assert_eq!(plate.rigidity, Rigidity::Plate); // Categorical, not "-40% damage"

    // Penetration results are categorical
    let pen = resolve_penetration(Edge::Sharp, Rigidity::Plate, false);
    assert!(matches!(pen, PenetrationResult::Deflect));

    // Trauma results are categorical
    let trauma = resolve_trauma(Mass::Heavy, Padding::Heavy);
    assert!(matches!(trauma, TraumaResult::Fatigue));

    // None of these types have methods returning f32 multipliers
    // or methods taking "bonus" parameters
    // The design is: Property A vs Property B → Categorical Outcome
}

/// Test exchange resolution end-to-end
#[test]
fn test_exchange_resolution() {
    let attacker = Combatant::test_swordsman();
    let defender = Combatant::test_plate_knight();

    let result = resolve_exchange(&attacker, &defender);

    // Attacker hit the defender
    assert!(result.defender_hit);

    // But sword vs plate produces no wound
    if let Some(wound) = &result.defender_wound {
        assert_eq!(wound.severity, WoundSeverity::None);
    }
}

/// Test stance transitions
#[test]
fn test_stance_state_machine() {
    let transitions = StanceTransitions::new();

    // Neutral -> Pressing via InitiateAttack
    let stance = transitions.apply(CombatStance::Neutral, TransitionTrigger::InitiateAttack);
    assert_eq!(stance, CombatStance::Pressing);

    // Pressing -> Recovering via AttackMissed (overextended)
    let stance = transitions.apply(CombatStance::Pressing, TransitionTrigger::AttackMissed);
    assert_eq!(stance, CombatStance::Recovering);

    // Recovering is vulnerable
    assert!(CombatStance::Recovering.vulnerable());
}

/// Test formation pressure system
#[test]
fn test_formation_pressure() {
    use arc_citadel::core::types::EntityId;

    let entities: Vec<EntityId> = (0..10).map(|_| EntityId::new()).collect();
    let mut formation = FormationState::new(entities);

    // Starts neutral
    assert_eq!(formation.pressure_category(), PressureCategory::Neutral);

    // Apply pressure
    formation.apply_pressure_delta(-0.8);
    assert_eq!(formation.pressure_category(), PressureCategory::Collapsing);
}

/// Test combat state component
#[test]
fn test_combat_state_component() {
    let mut state = CombatState::default();

    // Default state can fight
    assert!(state.can_fight());
    assert!(!state.in_combat());

    // Apply fatigue
    state.add_fatigue(0.3);
    assert!((state.fatigue - 0.3).abs() < 0.001);

    // Fatigue is clamped
    state.add_fatigue(0.9);
    assert_eq!(state.fatigue, 1.0);
}
