//! Engagement detection between units
//!
//! Units engage when adjacent. Combat follows engagement.

use crate::battle::hex::BattleHexCoord;
use crate::battle::planning::EngagementRule;
use crate::battle::units::{BattleUnit, UnitId};

/// Potential engagement between two units
#[derive(Debug, Clone)]
pub struct PotentialEngagement {
    pub attacker_id: UnitId,
    pub defender_id: UnitId,
    pub distance: u32,
}

/// Detect if two units should engage
pub fn detect_engagement(unit_a: &BattleUnit, unit_b: &BattleUnit) -> Option<PotentialEngagement> {
    // Can't engage if either is broken or has no strength
    if !unit_a.can_fight() || !unit_b.can_fight() {
        return None;
    }

    let distance = unit_a.position.distance(&unit_b.position);

    // Must be adjacent (distance 1) for melee engagement
    if distance > 1 {
        return None;
    }

    Some(PotentialEngagement {
        attacker_id: unit_a.id,
        defender_id: unit_b.id,
        distance,
    })
}

/// Check if unit should initiate combat based on engagement rules
pub fn should_initiate_combat(
    _unit: &BattleUnit,
    engagement_rule: &EngagementRule,
    is_being_attacked: bool,
) -> bool {
    match engagement_rule {
        EngagementRule::Aggressive => true,
        EngagementRule::Defensive => is_being_attacked,
        EngagementRule::HoldFire => false,
        EngagementRule::Skirmish => true,
    }
}

/// Find all engagements between two armies
pub fn find_all_engagements(
    friendly_units: &[&BattleUnit],
    enemy_units: &[&BattleUnit],
) -> Vec<PotentialEngagement> {
    let mut engagements = Vec::new();

    for friendly in friendly_units {
        for enemy in enemy_units {
            if let Some(engagement) = detect_engagement(friendly, enemy) {
                engagements.push(engagement);
            }
        }
    }

    engagements
}

/// Check if a unit is flanked (enemy in rear arc)
pub fn is_flanked(unit: &BattleUnit, enemy_positions: &[BattleHexCoord]) -> bool {
    let rear_offset = unit.facing.opposite().offset();
    let rear_hex = BattleHexCoord::new(
        unit.position.q + rear_offset.q,
        unit.position.r + rear_offset.r,
    );

    enemy_positions.contains(&rear_hex)
}

/// Check if a unit is surrounded (3+ adjacent enemies)
pub fn is_surrounded(unit: &BattleUnit, enemy_positions: &[BattleHexCoord]) -> bool {
    let adjacent_enemies = unit
        .position
        .neighbors()
        .iter()
        .filter(|n| enemy_positions.contains(n))
        .count();

    adjacent_enemies >= 3
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::hex::HexDirection;
    use crate::battle::unit_type::UnitType;
    use crate::battle::units::{Element, UnitStance};
    use crate::core::types::EntityId;

    #[test]
    fn test_adjacent_units_engage() {
        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.position = BattleHexCoord::new(5, 5);
        attacker.elements.push(Element::new(vec![EntityId::new(); 50]));

        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        defender.position = BattleHexCoord::new(6, 5); // Adjacent
        defender.elements.push(Element::new(vec![EntityId::new(); 50]));

        let result = detect_engagement(&attacker, &defender);

        assert!(result.is_some());
    }

    #[test]
    fn test_distant_units_dont_engage() {
        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.position = BattleHexCoord::new(0, 0);
        attacker.elements.push(Element::new(vec![EntityId::new(); 50]));

        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        defender.position = BattleHexCoord::new(10, 10); // Far away
        defender.elements.push(Element::new(vec![EntityId::new(); 50]));

        let result = detect_engagement(&attacker, &defender);

        assert!(result.is_none());
    }

    #[test]
    fn test_broken_unit_cant_engage() {
        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.position = BattleHexCoord::new(5, 5);
        attacker.elements.push(Element::new(vec![EntityId::new(); 50]));
        attacker.stance = UnitStance::Routing;

        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        defender.position = BattleHexCoord::new(6, 5);
        defender.elements.push(Element::new(vec![EntityId::new(); 50]));

        let result = detect_engagement(&attacker, &defender);

        assert!(result.is_none());
    }

    #[test]
    fn test_is_flanked() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(5, 5);
        unit.facing = HexDirection::East;

        // Enemy directly behind (West)
        let enemy_at_rear = vec![BattleHexCoord::new(4, 5)];
        assert!(is_flanked(&unit, &enemy_at_rear));

        // Enemy in front (East)
        let enemy_in_front = vec![BattleHexCoord::new(6, 5)];
        assert!(!is_flanked(&unit, &enemy_in_front));
    }

    #[test]
    fn test_is_surrounded() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(5, 5);

        // Only 2 enemies - not surrounded
        let two_enemies = vec![BattleHexCoord::new(6, 5), BattleHexCoord::new(4, 5)];
        assert!(!is_surrounded(&unit, &two_enemies));

        // 3 enemies - surrounded
        let three_enemies = vec![
            BattleHexCoord::new(6, 5),
            BattleHexCoord::new(4, 5),
            BattleHexCoord::new(5, 6),
        ];
        assert!(is_surrounded(&unit, &three_enemies));
    }

    #[test]
    fn test_engagement_rules() {
        let unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);

        assert!(should_initiate_combat(
            &unit,
            &EngagementRule::Aggressive,
            false
        ));
        assert!(!should_initiate_combat(
            &unit,
            &EngagementRule::Defensive,
            false
        ));
        assert!(should_initiate_combat(
            &unit,
            &EngagementRule::Defensive,
            true
        ));
        assert!(!should_initiate_combat(
            &unit,
            &EngagementRule::HoldFire,
            true
        ));
    }

    #[test]
    fn test_find_all_engagements() {
        let mut friendly1 = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        friendly1.position = BattleHexCoord::new(5, 5);
        friendly1.elements.push(Element::new(vec![EntityId::new(); 50]));

        let mut friendly2 = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        friendly2.position = BattleHexCoord::new(0, 0);
        friendly2.elements.push(Element::new(vec![EntityId::new(); 50]));

        let mut enemy1 = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        enemy1.position = BattleHexCoord::new(6, 5); // Adjacent to friendly1
        enemy1.elements.push(Element::new(vec![EntityId::new(); 50]));

        let friendly_units: Vec<&BattleUnit> = vec![&friendly1, &friendly2];
        let enemy_units: Vec<&BattleUnit> = vec![&enemy1];

        let engagements = find_all_engagements(&friendly_units, &enemy_units);

        // Only friendly1 should engage with enemy1
        assert_eq!(engagements.len(), 1);
        assert_eq!(engagements[0].attacker_id, friendly1.id);
        assert_eq!(engagements[0].defender_id, enemy1.id);
    }

    #[test]
    fn test_unit_with_no_strength_cant_engage() {
        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.position = BattleHexCoord::new(5, 5);
        // No elements means no strength

        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        defender.position = BattleHexCoord::new(6, 5);
        defender.elements.push(Element::new(vec![EntityId::new(); 50]));

        let result = detect_engagement(&attacker, &defender);

        assert!(result.is_none());
    }

    #[test]
    fn test_rallying_unit_cant_engage() {
        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.position = BattleHexCoord::new(5, 5);
        attacker.elements.push(Element::new(vec![EntityId::new(); 50]));
        attacker.stance = UnitStance::Rallying;

        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        defender.position = BattleHexCoord::new(6, 5);
        defender.elements.push(Element::new(vec![EntityId::new(); 50]));

        let result = detect_engagement(&attacker, &defender);

        assert!(result.is_none());
    }

    #[test]
    fn test_engagement_distance_is_correct() {
        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.position = BattleHexCoord::new(5, 5);
        attacker.elements.push(Element::new(vec![EntityId::new(); 50]));

        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        defender.position = BattleHexCoord::new(6, 5);
        defender.elements.push(Element::new(vec![EntityId::new(); 50]));

        let result = detect_engagement(&attacker, &defender).unwrap();

        assert_eq!(result.distance, 1);
    }

    #[test]
    fn test_skirmish_rule_initiates_combat() {
        let unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        assert!(should_initiate_combat(
            &unit,
            &EngagementRule::Skirmish,
            false
        ));
        assert!(should_initiate_combat(
            &unit,
            &EngagementRule::Skirmish,
            true
        ));
    }
}
