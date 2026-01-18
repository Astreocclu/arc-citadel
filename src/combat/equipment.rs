//! Equipment loadouts for entities based on role
//!
//! Provides appropriate weapon and armor based on entity role.

use crate::combat::{ArmorProperties, WeaponProperties};
use crate::skills::Role;

/// Get default weapon for a given role
pub fn weapon_for_role(role: Role) -> WeaponProperties {
    match role {
        Role::Soldier => WeaponProperties::sword(),
        Role::Guard => WeaponProperties::spear(),
        Role::Noble => WeaponProperties::sword(),
        // Non-combat roles use improvised weapons or fists
        Role::Farmer => WeaponProperties {
            // Pitchfork-like tool
            edge: crate::combat::Edge::Sharp,
            mass: crate::combat::Mass::Medium,
            reach: crate::combat::Reach::Medium,
            special: vec![],
        },
        Role::Miner => WeaponProperties {
            // Pick
            edge: crate::combat::Edge::Sharp,
            mass: crate::combat::Mass::Heavy,
            reach: crate::combat::Reach::Short,
            special: vec![],
        },
        Role::Craftsman(_) => WeaponProperties {
            // Hammer or knife depending on specialty
            edge: crate::combat::Edge::Blunt,
            mass: crate::combat::Mass::Medium,
            reach: crate::combat::Reach::Short,
            special: vec![],
        },
        _ => WeaponProperties::fists(),
    }
}

/// Get default armor for a given role
pub fn armor_for_role(role: Role) -> ArmorProperties {
    match role {
        Role::Soldier => ArmorProperties::mail(),
        Role::Guard => ArmorProperties::leather(),
        Role::Noble => ArmorProperties::mail(),
        // Non-combat roles have no armor
        _ => ArmorProperties::none(),
    }
}

/// Create a CombatState with appropriate equipment for a role
pub fn combat_state_for_role(role: Role) -> crate::combat::CombatState {
    crate::combat::CombatState {
        weapon: weapon_for_role(role),
        armor: armor_for_role(role),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soldier_has_sword() {
        let weapon = weapon_for_role(Role::Soldier);
        assert_eq!(weapon.edge, crate::combat::Edge::Sharp);
        assert_eq!(weapon.mass, crate::combat::Mass::Medium);
    }

    #[test]
    fn test_soldier_has_mail() {
        let armor = armor_for_role(Role::Soldier);
        assert_eq!(armor.rigidity, crate::combat::Rigidity::Mail);
    }

    #[test]
    fn test_farmer_has_no_armor() {
        let armor = armor_for_role(Role::Farmer);
        assert_eq!(armor.coverage, crate::combat::Coverage::None);
    }

    #[test]
    fn test_scholar_has_fists() {
        let weapon = weapon_for_role(Role::Scholar);
        assert_eq!(weapon.reach, crate::combat::Reach::Grapple);
        assert_eq!(weapon.edge, crate::combat::Edge::Blunt);
    }
}
