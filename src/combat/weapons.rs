//! Weapon properties for categorical combat resolution
//!
//! Weapons have exactly three properties: Edge, Mass, Reach.
//! These determine outcomes via lookup tables, not modifiers.

use serde::{Deserialize, Serialize};

/// Sharpness category - determines penetration potential
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Edge {
    /// Surgical sharpness (scalpels, fine blades)
    Razor,
    /// Combat sharpness (swords, axes)
    Sharp,
    /// No edge (maces, hammers, fists)
    Blunt,
}

/// Weight category - determines trauma potential
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mass {
    /// Daggers, small weapons (<1kg)
    Light,
    /// Swords, axes (1-3kg)
    Medium,
    /// Warhammers, greatswords (3-6kg)
    Heavy,
    /// Horse + rider, siege weapons (>100kg)
    Massive,
}

/// Distance category - determines strike order in exchanges
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Reach {
    /// Touching distance (fists, daggers)
    Grapple,
    /// Arm's length (swords, maces)
    Short,
    /// Extended arm (bastard swords, axes)
    Medium,
    /// Spear length (spears, halberds)
    Long,
    /// Formation weapons (pikes, lances)
    Pike,
}

/// Special weapon properties (optional capabilities)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponSpecial {
    /// Can find gaps in armor (estocs, bodkins)
    Piercing,
    /// Can pull shields/dismount (billhooks)
    Hooking,
    /// Can be thrown (javelins, axes)
    Throwable,
    /// Requires both hands
    TwoHanded,
    /// Effective vs shields (axes)
    Shieldbreaker,
}

/// Complete weapon properties
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponProperties {
    pub edge: Edge,
    pub mass: Mass,
    pub reach: Reach,
    pub special: Vec<WeaponSpecial>,
}

impl WeaponProperties {
    /// Check if weapon has a specific special property
    pub fn has_special(&self, special: WeaponSpecial) -> bool {
        self.special.contains(&special)
    }

    /// Common weapon: Sword
    pub fn sword() -> Self {
        Self {
            edge: Edge::Sharp,
            mass: Mass::Medium,
            reach: Reach::Short,
            special: vec![],
        }
    }

    /// Common weapon: Mace
    pub fn mace() -> Self {
        Self {
            edge: Edge::Blunt,
            mass: Mass::Heavy,
            reach: Reach::Short,
            special: vec![],
        }
    }

    /// Common weapon: Spear
    pub fn spear() -> Self {
        Self {
            edge: Edge::Sharp,
            mass: Mass::Medium,
            reach: Reach::Long,
            special: vec![WeaponSpecial::Piercing],
        }
    }

    /// Common weapon: Dagger
    pub fn dagger() -> Self {
        Self {
            edge: Edge::Sharp,
            mass: Mass::Light,
            reach: Reach::Grapple,
            special: vec![WeaponSpecial::Piercing],
        }
    }

    /// Common weapon: Fists (unarmed)
    pub fn fists() -> Self {
        Self {
            edge: Edge::Blunt,
            mass: Mass::Light,
            reach: Reach::Grapple,
            special: vec![],
        }
    }

    /// Common weapon: Axe (orcs, woodcutters)
    pub fn axe() -> Self {
        Self {
            edge: Edge::Sharp,
            mass: Mass::Heavy,
            reach: Reach::Medium,
            special: vec![WeaponSpecial::Shieldbreaker],
        }
    }

    /// Common weapon: Club (simple blunt weapon)
    pub fn club() -> Self {
        Self {
            edge: Edge::Blunt,
            mass: Mass::Medium,
            reach: Reach::Short,
            special: vec![],
        }
    }
}

impl Default for WeaponProperties {
    fn default() -> Self {
        Self::fists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sword_properties() {
        let sword = WeaponProperties {
            edge: Edge::Sharp,
            mass: Mass::Medium,
            reach: Reach::Short,
            special: vec![],
        };
        assert_eq!(sword.edge, Edge::Sharp);
        assert_eq!(sword.mass, Mass::Medium);
    }

    #[test]
    fn test_mace_is_blunt() {
        let mace = WeaponProperties {
            edge: Edge::Blunt,
            mass: Mass::Heavy,
            reach: Reach::Short,
            special: vec![],
        };
        assert_eq!(mace.edge, Edge::Blunt);
    }

    #[test]
    fn test_reach_ordering() {
        assert!(Reach::Pike > Reach::Long);
        assert!(Reach::Long > Reach::Medium);
        assert!(Reach::Medium > Reach::Short);
        assert!(Reach::Short > Reach::Grapple);
    }

    #[test]
    fn test_common_weapons() {
        let sword = WeaponProperties::sword();
        assert_eq!(sword.edge, Edge::Sharp);

        let mace = WeaponProperties::mace();
        assert_eq!(mace.edge, Edge::Blunt);

        let spear = WeaponProperties::spear();
        assert!(spear.has_special(WeaponSpecial::Piercing));
    }
}
