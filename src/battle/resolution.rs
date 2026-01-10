//! Mass combat resolution at different LOD levels
//!
//! Uses categorical property comparisons - NO percentage modifiers.

use crate::battle::unit_type::UnitType;
use crate::battle::units::BattleUnit;
use crate::combat::{ArmorProperties, Edge, Mass, Padding, Rigidity, ShockType, WeaponProperties};

/// Level of detail for combat resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatLOD {
    Individual, // LOD-0: full individual combat
    Element,    // LOD-1: element-level statistical
    Unit,       // LOD-2: unit-level statistical
    Formation,  // LOD-3: formation-level aggregate
}

/// Result of unit-level combat
#[derive(Debug, Clone)]
pub struct UnitCombatResult {
    pub attacker_casualties: u32,
    pub defender_casualties: u32,
    pub attacker_stress_delta: f32,
    pub defender_stress_delta: f32,
    pub attacker_fatigue_delta: f32,
    pub defender_fatigue_delta: f32,
    pub pressure_shift: f32,
}

/// Result of a shock attack (charge, flank, etc.)
#[derive(Debug, Clone)]
pub struct ShockResult {
    pub immediate_casualties: u32,
    pub stress_spike: f32,
    pub triggered_break_check: bool,
}

/// Calculate base casualty rate from weapon vs armor
///
/// Returns casualties per combatant per tick (ADDITIVE, not percentage).
pub fn calculate_casualty_rate(
    weapon: &WeaponProperties,
    armor: &ArmorProperties,
    pressure: f32,
) -> f32 {
    // Base rate from property matchup (categorical, not percentage)
    let base_rate = match (weapon.edge, armor.rigidity) {
        // Sharp vs different armors
        (Edge::Razor, Rigidity::Cloth) => 0.06,
        (Edge::Razor, Rigidity::Leather) => 0.04,
        (Edge::Razor, Rigidity::Mail) => 0.01,
        (Edge::Razor, Rigidity::Plate) => 0.003,

        (Edge::Sharp, Rigidity::Cloth) => 0.05,
        (Edge::Sharp, Rigidity::Leather) => 0.03,
        (Edge::Sharp, Rigidity::Mail) => 0.01,
        (Edge::Sharp, Rigidity::Plate) => 0.005,

        // Blunt weapons care about mass vs padding
        (Edge::Blunt, _) => match (weapon.mass, armor.padding) {
            (Mass::Massive, Padding::None) => 0.08,
            (Mass::Massive, Padding::Light) => 0.05,
            (Mass::Massive, Padding::Heavy) => 0.03,

            (Mass::Heavy, Padding::None) => 0.04,
            (Mass::Heavy, Padding::Light) => 0.02,
            (Mass::Heavy, Padding::Heavy) => 0.01,

            (Mass::Medium, Padding::None) => 0.02,
            (Mass::Medium, Padding::Light) => 0.01,
            (Mass::Medium, Padding::Heavy) => 0.005,

            (Mass::Light, _) => 0.005,
        },
    };

    // Pressure modifier (ADDITIVE, not multiplicative)
    let pressure_modifier = pressure * 0.02; // +/-2% per pressure point

    (base_rate + pressure_modifier).clamp(0.001, 0.15)
}

/// Calculate stress delta from combat
pub fn calculate_stress_delta(
    _unit: &BattleUnit,
    casualties: u32,
    is_flanked: bool,
    is_surrounded: bool,
) -> f32 {
    let mut stress = 0.0;

    // Base combat stress
    stress += 0.01;

    // Casualty stress (additive per casualty)
    stress += casualties as f32 * 0.02;

    // Flanked stress
    if is_flanked {
        stress += crate::battle::constants::FLANK_STRESS;
    }

    // Surrounded stress
    if is_surrounded {
        stress += 0.10;
    }

    stress
}

/// Resolve unit-level combat
pub fn resolve_unit_combat(
    attacker: &BattleUnit,
    defender: &BattleUnit,
    pressure: f32,
) -> UnitCombatResult {
    let attacker_props = attacker.unit_type.default_properties();
    let defender_props = defender.unit_type.default_properties();

    // Calculate casualty rates
    let defender_casualty_rate = calculate_casualty_rate(
        &attacker_props.avg_weapon,
        &defender_props.avg_armor,
        pressure,
    );

    let attacker_casualty_rate = calculate_casualty_rate(
        &defender_props.avg_weapon,
        &attacker_props.avg_armor,
        -pressure, // Pressure works against defender
    );

    // Apply casualties
    let defender_casualties =
        (defender_casualty_rate * defender.effective_strength() as f32).ceil() as u32;
    let attacker_casualties =
        (attacker_casualty_rate * attacker.effective_strength() as f32).ceil() as u32;

    // Calculate stress
    let attacker_stress = calculate_stress_delta(attacker, attacker_casualties, false, false);
    let defender_stress = calculate_stress_delta(defender, defender_casualties, false, false);

    // Fatigue from combat
    let fatigue_rate = crate::battle::constants::FATIGUE_RATE_COMBAT;

    // Pressure shifts based on casualties
    let pressure_shift = if defender_casualties > attacker_casualties {
        0.05
    } else if attacker_casualties > defender_casualties {
        -0.05
    } else {
        0.0
    };

    UnitCombatResult {
        attacker_casualties,
        defender_casualties,
        attacker_stress_delta: attacker_stress,
        defender_stress_delta: defender_stress,
        attacker_fatigue_delta: fatigue_rate,
        defender_fatigue_delta: fatigue_rate,
        pressure_shift,
    }
}

/// Calculate shock attack casualties
fn calculate_shock_casualties(
    _attacker: &BattleUnit,
    defender: &BattleUnit,
    shock_type: ShockType,
) -> u32 {
    // Cavalry charge = Massive mass hitting front rank
    let front_rank_size = (defender.effective_strength() as f32 * 0.2) as u32;

    let defender_props = defender.unit_type.default_properties();

    // Survival rate based on padding
    let survival_rate = match defender_props.avg_armor.padding {
        Padding::None => 0.3,  // 70% casualties
        Padding::Light => 0.5, // 50% casualties
        Padding::Heavy => 0.7, // 30% casualties
    };

    let mut casualties = (front_rank_size as f32 * (1.0 - survival_rate)) as u32;

    // Spearmen reduce charge effectiveness
    if defender.unit_type == UnitType::Spearmen {
        casualties /= 2;
    }

    // Shock type modifiers
    match shock_type {
        ShockType::CavalryCharge => {} // Base calculation
        ShockType::FlankAttack => casualties = casualties * 2 / 3,
        ShockType::RearCharge => casualties = casualties * 3 / 2,
        ShockType::Ambush => casualties = casualties * 5 / 4,
    }

    casualties
}

/// Resolve a shock attack
pub fn resolve_shock_attack(
    attacker: &BattleUnit,
    defender: &BattleUnit,
    shock_type: ShockType,
) -> ShockResult {
    let casualties = calculate_shock_casualties(attacker, defender, shock_type);

    let stress_spike = shock_type.stress_spike()
        + (casualties as f32 / defender.effective_strength() as f32) * 0.20;

    let defender_threshold = defender.stress_threshold();
    let triggered_break_check = defender.stress + stress_spike > defender_threshold * 0.7;

    ShockResult {
        immediate_casualties: casualties,
        stress_spike,
        triggered_break_check,
    }
}

/// Determine LOD for a combat
pub fn determine_combat_lod(
    total_combatants: usize,
    is_player_focused: bool,
    is_near_objective: bool,
) -> CombatLOD {
    if is_player_focused {
        CombatLOD::Individual
    } else if is_near_objective || total_combatants < 50 {
        CombatLOD::Element
    } else if total_combatants < 200 {
        CombatLOD::Unit
    } else {
        CombatLOD::Formation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::units::{Element, UnitId};
    use crate::core::types::EntityId;

    #[test]
    fn test_casualty_rate_sharp_vs_cloth() {
        let weapon = WeaponProperties::sword();
        let armor = ArmorProperties::none();
        let rate = calculate_casualty_rate(&weapon, &armor, 0.0);
        assert!(rate > 0.03);
    }

    #[test]
    fn test_casualty_rate_sharp_vs_plate() {
        let weapon = WeaponProperties::sword();
        let armor = ArmorProperties::plate();
        let rate = calculate_casualty_rate(&weapon, &armor, 0.0);
        assert!(rate < 0.01);
    }

    #[test]
    fn test_pressure_affects_rate_additively() {
        let weapon = WeaponProperties::sword();
        let armor = ArmorProperties::leather();

        let rate_neutral = calculate_casualty_rate(&weapon, &armor, 0.0);
        let rate_positive = calculate_casualty_rate(&weapon, &armor, 0.5);
        let rate_negative = calculate_casualty_rate(&weapon, &armor, -0.5);

        assert!(rate_positive > rate_neutral);
        assert!(rate_negative < rate_neutral);

        let delta_pos = rate_positive - rate_neutral;
        let delta_neg = rate_neutral - rate_negative;
        assert!((delta_pos - delta_neg).abs() < 0.005);
    }

    #[test]
    fn test_spearmen_reduce_charge_casualties() {
        let mut cavalry = BattleUnit::new(UnitId::new(), UnitType::HeavyCavalry);
        cavalry
            .elements
            .push(Element::new(vec![EntityId::new(); 50]));

        let mut infantry = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        infantry
            .elements
            .push(Element::new(vec![EntityId::new(); 100]));

        let mut spearmen = BattleUnit::new(UnitId::new(), UnitType::Spearmen);
        spearmen
            .elements
            .push(Element::new(vec![EntityId::new(); 100]));

        let infantry_result = resolve_shock_attack(&cavalry, &infantry, ShockType::CavalryCharge);
        let spearmen_result = resolve_shock_attack(&cavalry, &spearmen, ShockType::CavalryCharge);

        assert!(spearmen_result.immediate_casualties < infantry_result.immediate_casualties);
    }

    #[test]
    fn test_determine_lod() {
        assert_eq!(determine_combat_lod(30, true, false), CombatLOD::Individual);
        assert_eq!(determine_combat_lod(30, false, true), CombatLOD::Element);
        assert_eq!(determine_combat_lod(30, false, false), CombatLOD::Element);
        assert_eq!(determine_combat_lod(100, false, false), CombatLOD::Unit);
        assert_eq!(
            determine_combat_lod(300, false, false),
            CombatLOD::Formation
        );
    }

    #[test]
    fn test_stress_delta_increases_with_casualties() {
        let unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);

        let stress_0 = calculate_stress_delta(&unit, 0, false, false);
        let stress_10 = calculate_stress_delta(&unit, 10, false, false);

        assert!(stress_10 > stress_0);
    }
}
