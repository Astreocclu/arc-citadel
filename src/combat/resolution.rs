//! Combat exchange resolution
//!
//! An exchange occurs when PRESSING meets any other stance.
//! NO PERCENTAGE MODIFIERS. Property comparisons only.

use crate::combat::{
    combine_results, resolve_penetration, resolve_trauma, ArmorProperties, BodyZone, CombatSkill,
    CombatStance, SkillLevel, WeaponProperties, WeaponSpecial, Wound,
};

/// A combatant in an exchange
#[derive(Debug, Clone)]
pub struct Combatant {
    pub weapon: WeaponProperties,
    pub armor: ArmorProperties,
    pub stance: CombatStance,
    pub skill: CombatSkill,
}

impl Combatant {
    /// Test combatant: swordsman with no armor
    pub fn test_swordsman() -> Self {
        Self {
            weapon: WeaponProperties::sword(),
            armor: ArmorProperties::none(),
            stance: CombatStance::Pressing,
            skill: CombatSkill::trained(),
        }
    }

    /// Test combatant: spearman with no armor
    pub fn test_spearman() -> Self {
        Self {
            weapon: WeaponProperties::spear(),
            armor: ArmorProperties::none(),
            stance: CombatStance::Pressing,
            skill: CombatSkill::trained(),
        }
    }

    /// Test combatant: plate knight with sword
    pub fn test_plate_knight() -> Self {
        Self {
            weapon: WeaponProperties::sword(),
            armor: ArmorProperties::plate(),
            stance: CombatStance::Neutral,
            skill: CombatSkill::veteran(),
        }
    }

    /// Test combatant: unarmored civilian
    pub fn test_unarmored() -> Self {
        Self {
            weapon: WeaponProperties::fists(),
            armor: ArmorProperties::none(),
            stance: CombatStance::Neutral,
            skill: CombatSkill::novice(),
        }
    }
}

/// Result of an exchange
#[derive(Debug, Clone)]
pub struct ExchangeResult {
    /// Did attacker hit defender?
    pub defender_hit: bool,
    /// Did defender hit attacker?
    pub attacker_hit: bool,
    /// Who struck first (if both attacked)?
    pub attacker_struck_first: bool,
    /// Wound to defender (if any)
    pub defender_wound: Option<Wound>,
    /// Wound to attacker (if any)
    pub attacker_wound: Option<Wound>,
}

/// Select a hit zone (deterministic based on skill)
fn select_hit_zone(skill: SkillLevel) -> BodyZone {
    // Higher skill = more likely to hit vital areas
    // This is deterministic, not random
    match skill {
        SkillLevel::Master => BodyZone::Head,
        SkillLevel::Veteran => BodyZone::Torso,
        SkillLevel::Trained => BodyZone::Torso,
        SkillLevel::Novice => BodyZone::Torso,
    }
}

/// Resolve a single hit
fn resolve_hit(weapon: &WeaponProperties, armor: &ArmorProperties, zone: BodyZone) -> Wound {
    let has_piercing = weapon.has_special(WeaponSpecial::Piercing);
    let pen = resolve_penetration(weapon.edge, armor.rigidity, has_piercing);
    let trauma = resolve_trauma(weapon.mass, armor.padding);
    combine_results(pen, trauma, zone)
}

/// Resolve an exchange between attacker and defender
///
/// # Arguments
/// * `attacker` - The combatant initiating (must be PRESSING)
/// * `defender` - The combatant receiving
///
/// # Returns
/// Exchange result with hits and wounds
pub fn resolve_exchange(attacker: &Combatant, defender: &Combatant) -> ExchangeResult {
    // Step 1: Check if defender can respond
    let defender_can_respond = !defender.stance.vulnerable();

    if !defender_can_respond {
        // Free hit - defender is recovering or broken
        let zone = select_hit_zone(attacker.skill.level);
        let wound = resolve_hit(&attacker.weapon, &defender.armor, zone);

        return ExchangeResult {
            defender_hit: true,
            attacker_hit: false,
            attacker_struck_first: true,
            defender_wound: Some(wound),
            attacker_wound: None,
        };
    }

    // Step 2: Both can fight - reach determines strike order
    let attacker_reach = attacker.weapon.reach;
    let defender_reach = defender.weapon.reach;

    let (attacker_struck_first, both_hit) = match attacker_reach.cmp(&defender_reach) {
        std::cmp::Ordering::Greater => (true, true),
        std::cmp::Ordering::Less => (false, true),
        std::cmp::Ordering::Equal => (true, true), // Simultaneous
    };

    // Step 3: Resolve attacker's hit
    let attacker_zone = select_hit_zone(attacker.skill.level);
    let defender_wound = resolve_hit(&attacker.weapon, &defender.armor, attacker_zone);

    // Step 4: Resolve defender's counter (if they can attack)
    let (attacker_hit, attacker_wound) = if defender.stance.can_attack() && both_hit {
        let defender_zone = select_hit_zone(defender.skill.level);
        let wound = resolve_hit(&defender.weapon, &attacker.armor, defender_zone);
        (true, Some(wound))
    } else {
        (false, None)
    };

    ExchangeResult {
        defender_hit: true,
        attacker_hit,
        attacker_struck_first,
        defender_wound: Some(defender_wound),
        attacker_wound,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::body_zone::WoundSeverity;

    #[test]
    fn test_pressing_vs_recovering_is_free_hit() {
        let mut attacker = Combatant::test_swordsman();
        attacker.stance = CombatStance::Pressing;

        let mut defender = Combatant::test_unarmored();
        defender.stance = CombatStance::Recovering;

        let result = resolve_exchange(&attacker, &defender);

        assert!(result.defender_hit);
        assert!(!result.attacker_hit);
    }

    #[test]
    fn test_reach_determines_strike_order() {
        let spearman = Combatant::test_spearman();
        let swordsman = Combatant::test_swordsman();

        let result = resolve_exchange(&spearman, &swordsman);

        assert!(result.attacker_struck_first);
    }

    #[test]
    fn test_sword_vs_plate_no_wound() {
        let attacker = Combatant::test_swordsman();
        let defender = Combatant::test_plate_knight();

        let result = resolve_exchange(&attacker, &defender);

        if let Some(wound) = &result.defender_wound {
            // Sharp vs Plate = Deflect, Medium vs Heavy = Fatigue
            // Worse of (None, None) = None
            assert_eq!(wound.severity, WoundSeverity::None);
        }
    }

    #[test]
    fn test_both_combatants_can_be_hit() {
        // Two swordsmen attacking each other
        let mut attacker = Combatant::test_swordsman();
        attacker.stance = CombatStance::Pressing;

        let mut defender = Combatant::test_swordsman();
        defender.stance = CombatStance::Pressing; // Both attacking

        let result = resolve_exchange(&attacker, &defender);

        // Equal reach = simultaneous, both should be hit
        assert!(result.defender_hit);
        assert!(result.attacker_hit);
    }
}
