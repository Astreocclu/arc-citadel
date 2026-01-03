//! Morale and stress system
//!
//! Stress accumulates from combat. When it exceeds threshold, units break.

use crate::battle::constants::{CONTAGION_STRESS, OFFICER_DEATH_STRESS};
use crate::battle::units::{BattleUnit, UnitStance};

/// Result of a morale check
#[derive(Debug, Clone)]
pub struct MoraleCheckResult {
    pub breaks: bool,
    pub rallies: bool,
    pub stress_delta: f32,
}

impl Default for MoraleCheckResult {
    fn default() -> Self {
        Self {
            breaks: false,
            rallies: false,
            stress_delta: 0.0,
        }
    }
}

/// Check if a unit breaks due to stress
pub fn check_morale_break(unit: &BattleUnit) -> MoraleCheckResult {
    let mut result = MoraleCheckResult::default();

    // Already routing units can't break again
    if matches!(unit.stance, UnitStance::Routing) {
        return result;
    }

    // Check stress vs threshold
    let threshold = unit.stress_threshold();
    if unit.stress >= threshold {
        result.breaks = true;
    }

    result
}

/// Check if a routing unit can rally
pub fn check_rally(unit: &BattleUnit, is_near_enemy: bool, is_near_leader: bool) -> MoraleCheckResult {
    let mut result = MoraleCheckResult::default();

    // Only routing units can rally
    if !matches!(unit.stance, UnitStance::Routing) {
        return result;
    }

    // Can't rally near enemies
    if is_near_enemy {
        return result;
    }

    // Rally conditions
    let rally_threshold = 0.5; // Need stress below this to rally
    let stress_after_recovery = unit.stress - 0.1; // Routing reduces stress

    if stress_after_recovery < rally_threshold {
        result.rallies = true;
        result.stress_delta = -0.1;
    }

    // Leader nearby helps
    if is_near_leader && stress_after_recovery < rally_threshold + 0.2 {
        result.rallies = true;
        result.stress_delta = -0.15;
    }

    result
}

/// Calculate stress contagion from nearby routing units
pub fn calculate_contagion_stress(unit: &BattleUnit, nearby_routing_count: usize) -> f32 {
    // Already routing units don't get more contagion stress
    if matches!(unit.stance, UnitStance::Routing) {
        return 0.0;
    }

    nearby_routing_count as f32 * CONTAGION_STRESS
}

/// Calculate stress from officer death
pub fn calculate_officer_death_stress(had_leader: bool, leader_died: bool) -> f32 {
    if had_leader && leader_died {
        OFFICER_DEATH_STRESS
    } else {
        0.0
    }
}

/// Apply stress to a unit
pub fn apply_stress(unit: &mut BattleUnit, stress_delta: f32) {
    unit.stress = (unit.stress + stress_delta).clamp(0.0, 2.0);
}

/// Process morale break for a unit
pub fn process_morale_break(unit: &mut BattleUnit) {
    unit.stance = UnitStance::Routing;
    unit.cohesion = (unit.cohesion * 0.5).max(0.1);
}

/// Process rally for a unit
pub fn process_rally(unit: &mut BattleUnit) {
    unit.stance = UnitStance::Rallying;
    // Will transition to Formed after a few ticks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::unit_type::UnitType;
    use crate::battle::units::{Element, UnitId};
    use crate::core::types::EntityId;

    #[test]
    fn test_unit_breaks_when_stress_exceeds_threshold() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        // Infantry base_stress_threshold is 1.0, plus 0.1 for high cohesion = 1.1
        // So use stress >= 1.1 to break
        unit.stress = 1.2;

        let result = check_morale_break(&unit);

        assert!(result.breaks);
    }

    #[test]
    fn test_unit_holds_with_low_stress() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        unit.stress = 0.1; // Low stress

        let result = check_morale_break(&unit);

        assert!(!result.breaks);
    }

    #[test]
    fn test_routing_unit_can_rally_when_safe() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        unit.stance = UnitStance::Routing;
        unit.stress = 0.3; // Low enough to rally

        let result = check_rally(&unit, false, false);

        assert!(result.rallies);
    }

    #[test]
    fn test_routing_unit_cant_rally_near_enemy() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        unit.stance = UnitStance::Routing;
        unit.stress = 0.3;

        let result = check_rally(&unit, true, false); // Near enemy

        assert!(!result.rallies);
    }

    #[test]
    fn test_contagion_stress() {
        let unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);

        let stress_0 = calculate_contagion_stress(&unit, 0);
        let stress_2 = calculate_contagion_stress(&unit, 2);

        assert_eq!(stress_0, 0.0);
        assert!(stress_2 > 0.0);
        assert_eq!(stress_2, 2.0 * CONTAGION_STRESS);
    }

    #[test]
    fn test_apply_stress_clamped() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.stress = 1.9;

        apply_stress(&mut unit, 0.5);

        assert_eq!(unit.stress, 2.0); // Clamped to max
    }

    #[test]
    fn test_leader_helps_rally() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        unit.stance = UnitStance::Routing;
        unit.stress = 0.6; // Too high for normal rally

        // Without leader: can't rally
        assert!(!check_rally(&unit, false, false).rallies);

        // With leader: can rally
        assert!(check_rally(&unit, false, true).rallies);
    }

    #[test]
    fn test_routing_unit_cannot_break_again() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        unit.stance = UnitStance::Routing;
        unit.stress = 2.0; // Maximum stress

        let result = check_morale_break(&unit);

        // Already routing, can't break again
        assert!(!result.breaks);
    }

    #[test]
    fn test_routing_unit_no_contagion_stress() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.stance = UnitStance::Routing;

        let stress = calculate_contagion_stress(&unit, 5);

        assert_eq!(stress, 0.0);
    }

    #[test]
    fn test_officer_death_stress() {
        // Leader died
        let stress = calculate_officer_death_stress(true, true);
        assert_eq!(stress, OFFICER_DEATH_STRESS);

        // No leader, can't die
        let stress = calculate_officer_death_stress(false, true);
        assert_eq!(stress, 0.0);

        // Leader alive
        let stress = calculate_officer_death_stress(true, false);
        assert_eq!(stress, 0.0);
    }

    #[test]
    fn test_process_morale_break() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.cohesion = 0.9;
        unit.stance = UnitStance::Formed;

        process_morale_break(&mut unit);

        assert_eq!(unit.stance, UnitStance::Routing);
        assert!(unit.cohesion < 0.5); // Cohesion reduced
    }

    #[test]
    fn test_process_rally() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.stance = UnitStance::Routing;

        process_rally(&mut unit);

        assert_eq!(unit.stance, UnitStance::Rallying);
    }

    #[test]
    fn test_apply_stress_minimum_clamped() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.stress = 0.1;

        apply_stress(&mut unit, -0.5);

        assert_eq!(unit.stress, 0.0); // Clamped to min
    }

    #[test]
    fn test_non_routing_unit_cannot_rally() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        unit.stance = UnitStance::Formed; // Not routing
        unit.stress = 0.1;

        let result = check_rally(&unit, false, true);

        assert!(!result.rallies);
    }
}
