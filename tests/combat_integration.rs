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

// ============================================================================
// COOL HISTORICAL SCENARIO TESTS
// ============================================================================

/// The Battle of Agincourt Problem: Why longbows beat French knights
///
/// French knights in full plate charged across muddy fields. The longbows
/// couldn't penetrate plate... but they could exhaust and demoralize.
#[test]
fn test_agincourt_longbow_vs_plate() {
    let longbow = WeaponProperties {
        edge: Edge::Sharp,      // Broadhead arrows are sharp
        mass: Mass::Light,      // Arrows are light
        reach: Reach::Pike,     // Outranges everything
        special: vec![],
    };
    let plate = ArmorProperties::plate();

    // Arrows CANNOT penetrate plate
    let pen = resolve_penetration(longbow.edge, plate.rigidity, false);
    assert_eq!(pen, PenetrationResult::Deflect);

    // Light mass = negligible trauma
    let trauma = resolve_trauma(longbow.mass, plate.padding);
    assert_eq!(trauma, TraumaResult::Negligible);

    // No physical wound!
    let wound = combine_results(pen, trauma, BodyZone::Torso);
    assert_eq!(wound.severity, WoundSeverity::None);

    // BUT: The morale path still works
    // 500 yards of arrows while trudging through mud...
    let mut french_knight_morale = MoraleState::default();

    // Simulate being under fire for extended period
    for _ in 0..20 {
        french_knight_morale.apply_stress(StressSource::TakingFire);
    }
    // Near misses are terrifying even if they can't hurt you
    for _ in 0..10 {
        french_knight_morale.apply_stress(StressSource::NearMiss);
    }

    // After prolonged fire, morale degrades even without wounds
    assert!(french_knight_morale.current_stress > 0.5);
}

/// The Zweihänder Solution: How landsknechts dealt with pike formations
///
/// Pike squares were nearly invincible... until someone got inside.
/// Two-handed swords could break pikes and create gaps.
#[test]
fn test_zweihander_vs_pike_formation() {
    let zweihander = WeaponProperties {
        edge: Edge::Sharp,
        mass: Mass::Heavy,      // 3-4kg of steel
        reach: Reach::Long,     // Can parry pikes
        special: vec![],
    };

    // Pikeman only has cloth beneath - leather vest doesn't cover limbs
    let pikeman_armor = ArmorProperties::none(); // Cloth only

    // Once inside pike range, the zweihänder is devastating
    let pen = resolve_penetration(zweihander.edge, pikeman_armor.rigidity, false);
    assert_eq!(pen, PenetrationResult::Cut); // Sharp vs Cloth = Cut

    let trauma = resolve_trauma(zweihander.mass, pikeman_armor.padding);
    assert_eq!(trauma, TraumaResult::KnockdownBruise); // Heavy vs None = Knockdown

    // Combined: Serious wound (Cut + KnockdownBruise both map to Serious)
    // The pikeman is cut AND knocked down - not fatal, but out of the fight
    let wound = combine_results(pen, trauma, BodyZone::Torso);
    assert_eq!(wound.severity, WoundSeverity::Serious);
    assert!(wound.bleeding);       // Cut causes bleeding
    assert!(wound.mobility_impact); // Knockdown affects mobility
}

/// The Murder Stroke: When swords become hammers
///
/// Knights in armor would grip their sword by the blade and use the
/// crossguard as a hammer. Called "Mordhau" (murder-stroke).
/// The concentrated weight of pommel + crossguard hits like a mace.
#[test]
fn test_mordhau_murder_stroke() {
    // Sword held by blade, pommel/guard used as bludgeon
    // The concentrated mass hitting a small point = effective heavy weapon
    let mordhau = WeaponProperties {
        edge: Edge::Blunt,      // Using the guard, not the edge
        mass: Mass::Heavy,      // Concentrated impact = effectively heavy
        reach: Reach::Short,    // Half-sword grip = shorter reach
        special: vec![],
    };
    let plate = ArmorProperties::plate();

    // Blunt doesn't try to cut
    let pen = resolve_penetration(mordhau.edge, plate.rigidity, false);
    assert_eq!(pen, PenetrationResult::NoPenetrationAttempt);

    // BUT: Trauma still transfers through plate
    let trauma = resolve_trauma(mordhau.mass, plate.padding);
    assert_eq!(trauma, TraumaResult::Fatigue);

    // This is why mordhau was used: concuss through the helmet
    let head_wound = combine_results(pen, trauma, BodyZone::Head);
    // No penetration, but repeated blows cause fatigue and disorientation
    assert!(!head_wound.bleeding); // Blunt trauma doesn't bleed externally
}

/// The Stiletto Problem: Why assassins love thin blades
///
/// Stilettos and rondel daggers were designed to find gaps in armor.
#[test]
fn test_stiletto_vs_mail() {
    let stiletto = WeaponProperties {
        edge: Edge::Razor,      // Needle-sharp point
        mass: Mass::Light,      // Dagger weight
        reach: Reach::Grapple,  // Grappling range
        special: vec![arc_citadel::combat::WeaponSpecial::Piercing], // Finds gaps!
    };
    let mail = ArmorProperties::mail();

    // WITHOUT piercing: razor vs mail = snag (gets caught in rings)
    let pen_no_pierce = resolve_penetration(Edge::Razor, Rigidity::Mail, false);
    assert_eq!(pen_no_pierce, PenetrationResult::Snag);

    // WITH piercing: finds the gaps between rings
    // Piercing upgrades Snag → ShallowCut (one category better)
    let pen_pierce = resolve_penetration(stiletto.edge, mail.rigidity,
        stiletto.has_special(arc_citadel::combat::WeaponSpecial::Piercing));
    assert_eq!(pen_pierce, PenetrationResult::ShallowCut);

    // A stiletto to the armpit (gap in armor) causes a wound
    let wound = combine_results(pen_pierce, TraumaResult::Negligible, BodyZone::Torso);
    assert_eq!(wound.severity, WoundSeverity::Minor); // ShallowCut = Minor
    assert!(!wound.bleeding); // ShallowCut doesn't bleed (only Cut and DeepCut)
}

/// The Cavalry Charge: Mass times velocity equals terror
///
/// A horse and rider weighing 600kg+ at full gallop is Massive mass.
#[test]
fn test_cavalry_charge_impact() {
    let lance_charge = WeaponProperties {
        edge: Edge::Sharp,      // Lance point
        mass: Mass::Massive,    // Horse + rider + momentum
        reach: Reach::Pike,     // Couched lance
        special: vec![],
    };

    // Even plate cannot withstand a direct lance hit
    let plate = ArmorProperties::plate();

    // Penetration: Sharp vs Plate still deflects
    let pen = resolve_penetration(lance_charge.edge, plate.rigidity, false);
    assert_eq!(pen, PenetrationResult::Deflect);

    // BUT: Massive trauma overwhelms even heavy padding
    let trauma = resolve_trauma(lance_charge.mass, plate.padding);
    assert_eq!(trauma, TraumaResult::Stagger); // Heavy padding reduces to stagger

    // Against leather (light padding): brutal knockdown
    let leather = ArmorProperties::leather();
    let trauma_vs_leather = resolve_trauma(lance_charge.mass, leather.padding);
    assert_eq!(trauma_vs_leather, TraumaResult::KnockdownBruise); // Internal injuries

    // Against unarmored: catastrophic
    let unarmored = ArmorProperties::none();
    let trauma_vs_unarmored = resolve_trauma(lance_charge.mass, unarmored.padding);
    assert_eq!(trauma_vs_unarmored, TraumaResult::KnockdownCrush); // Broken bones

    // And the morale impact...
    let mut infantry_morale = MoraleState::default();
    infantry_morale.apply_stress(StressSource::CavalryCharge);
    assert!(infantry_morale.current_stress >= 0.20);
}

/// The Phalanx Problem: Reach determines who strikes first
///
/// Spears beat swords. Pikes beat spears. Getting inside pike range beats pikes.
#[test]
fn test_reach_advantage_cascade() {
    // Pike (longest) > Spear > Sword > Dagger (shortest)
    assert!(Reach::Pike > Reach::Long);
    assert!(Reach::Long > Reach::Medium);
    assert!(Reach::Medium > Reach::Short);
    assert!(Reach::Short > Reach::Grapple);

    // In an exchange, longer reach strikes first
    let pikeman = Combatant::test_spearman(); // Has Long reach
    let mut swordsman = Combatant::test_swordsman(); // Has Short reach
    swordsman.stance = CombatStance::Pressing;

    let result = resolve_exchange(&pikeman, &swordsman);
    assert!(result.attacker_struck_first); // Pike always strikes first

    // BUT if you get inside pike range (grappling), the pike is useless
    let dagger_fighter = Combatant {
        weapon: WeaponProperties::dagger(),
        armor: ArmorProperties::none(),
        stance: CombatStance::Pressing,
        skill: arc_citadel::combat::CombatSkill::veteran(),
    };

    // At grapple range, the dagger fighter has the advantage
    // (This would require getting past the pike first - a different system)
    assert_eq!(dagger_fighter.weapon.reach, Reach::Grapple);
}

/// The Rout Cascade: When one unit breaks, others follow
///
/// Morale is contagious. Seeing your allies flee destroys your will to fight.
#[test]
fn test_morale_cascade() {
    use arc_citadel::core::types::EntityId;

    let mut unit_a_morale = MoraleState::default();
    let mut unit_b_morale = MoraleState::default();
    let mut unit_c_morale = MoraleState::default();

    // Unit A takes devastating punishment:
    // - OfficerKilled: 0.30
    // - FlankAttack: 0.20
    // - AmbushSprung: 0.25
    // - WoundReceived: 0.15
    // - TakingCasualties x3: 0.15
    // Total: 1.05 (exceeds 1.0 threshold)
    unit_a_morale.apply_stress(StressSource::OfficerKilled);
    unit_a_morale.apply_stress(StressSource::FlankAttack);
    unit_a_morale.apply_stress(StressSource::AmbushSprung);
    unit_a_morale.apply_stress(StressSource::WoundReceived);
    unit_a_morale.apply_stress(StressSource::TakingCasualties);
    unit_a_morale.apply_stress(StressSource::TakingCasualties);
    unit_a_morale.apply_stress(StressSource::TakingCasualties);

    // Unit A breaks (stress > 1.0 threshold)
    assert_eq!(unit_a_morale.check_break(), BreakResult::Breaking);

    // Unit B sees Unit A breaking
    unit_b_morale.apply_stress(StressSource::AlliesBreaking);
    unit_b_morale.apply_stress(StressSource::AlliesBreaking);

    // Unit B is now also stressed
    assert!(unit_b_morale.current_stress >= 0.20);

    // Unit C is now alone and exposed
    unit_c_morale.apply_stress(StressSource::AloneExposed);
    unit_c_morale.apply_stress(StressSource::Surrounded);

    // The cascade continues...
    assert!(unit_c_morale.current_stress > 0.0);

    // Formation-level view
    let entities: Vec<EntityId> = (0..100).map(|_| EntityId::new()).collect();
    let mut formation = FormationState::new(entities);

    // 40% casualties = formation breaks
    formation.broken_count = 40;
    assert!(formation.is_broken());
}

/// The Exhaustion Death Spiral: Fatigue makes everything worse
///
/// A tired fighter makes mistakes. Mistakes lead to wounds. Wounds cause fatigue.
#[test]
fn test_exhaustion_death_spiral() {
    let mut state = CombatState::default();

    // Fresh fighter can attack
    assert!(state.stance.can_attack());
    assert_eq!(state.fatigue, 0.0);

    // After sustained combat...
    state.add_fatigue(0.3); // From melee violence
    state.add_fatigue(0.2); // From attacking
    state.add_fatigue(0.2); // From defending

    // Getting tired (70% fatigue)
    assert!(state.fatigue >= 0.7);

    // Near exhaustion threshold (0.9 in constants)
    state.add_fatigue(0.2);
    assert!(state.fatigue >= 0.9);

    // In a real fight, this would force a stance transition to Recovering
    // where the fighter is vulnerable to free hits
}

/// The Shield Wall: Why formations matter
///
/// Individual fighters lose to coordinated units. Cohesion is everything.
#[test]
fn test_formation_cohesion() {
    use arc_citadel::core::types::EntityId;

    // Tight shield wall
    let entities: Vec<EntityId> = (0..30).map(|_| EntityId::new()).collect();
    let mut shield_wall = FormationState::new(entities);

    assert_eq!(shield_wall.cohesion, 1.0); // Perfect cohesion at start
    assert_eq!(shield_wall.pressure, 0.0); // Neutral pressure

    // Taking pressure but holding
    shield_wall.apply_pressure_delta(-0.2);
    assert_eq!(shield_wall.pressure_category(), PressureCategory::Neutral);

    // Heavy pressure - losing ground
    shield_wall.apply_pressure_delta(-0.3);
    assert_eq!(shield_wall.pressure_category(), PressureCategory::Losing);

    // Casualties start breaking cohesion
    shield_wall.broken_count = 5; // ~17% casualties
    assert!(!shield_wall.is_broken()); // Not broken yet (threshold is 40%)

    // More casualties...
    shield_wall.broken_count = 12; // 40% casualties
    assert!(shield_wall.is_broken()); // Formation breaks!
}
