//! Unit types and their default properties
//!
//! Unit properties emerge from equipment aggregation.

use serde::{Deserialize, Serialize};
use crate::combat::{WeaponProperties, ArmorProperties, Edge, Mass, Reach, Rigidity, Padding, Coverage};

/// Type of military unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitType {
    // Infantry
    Levy,           // Cheap, unreliable
    Infantry,       // Standard foot soldiers
    HeavyInfantry,  // Armored, slow, tough
    Spearmen,       // Anti-cavalry, defensive
    Archers,        // Ranged, vulnerable in melee
    Crossbowmen,    // Slower, more punch

    // Cavalry
    LightCavalry,   // Fast, scout, skirmish
    Cavalry,        // Standard mounted
    HeavyCavalry,   // Shock, armored, expensive
    HorseArchers,   // Mobile ranged

    // Special
    Engineers,      // Siege, construction
    Scouts,         // Reconnaissance
    Command,        // Officers, messengers
}

/// Default properties for a unit type
#[derive(Debug, Clone)]
pub struct UnitProperties {
    pub avg_weapon: WeaponProperties,
    pub avg_armor: ArmorProperties,
    pub movement_speed: f32,       // Relative to baseline
    pub vision_range: u32,         // In hexes
    pub base_stress_threshold: f32,
    pub can_charge: bool,
    pub can_skirmish: bool,
}

impl UnitType {
    /// Get default properties for this unit type
    pub fn default_properties(&self) -> UnitProperties {
        match self {
            UnitType::Levy => UnitProperties {
                avg_weapon: WeaponProperties {
                    edge: Edge::Sharp,
                    mass: Mass::Light,
                    reach: Reach::Short,
                    special: vec![],
                },
                avg_armor: ArmorProperties {
                    rigidity: Rigidity::Cloth,
                    padding: Padding::None,
                    coverage: Coverage::None,
                },
                movement_speed: 1.0,
                vision_range: 6,
                base_stress_threshold: 0.6, // Break easily
                can_charge: false,
                can_skirmish: false,
            },

            UnitType::Infantry => UnitProperties {
                avg_weapon: WeaponProperties::sword(),
                avg_armor: ArmorProperties::leather(),
                movement_speed: 1.0,
                vision_range: 6,
                base_stress_threshold: 1.0,
                can_charge: false,
                can_skirmish: false,
            },

            UnitType::HeavyInfantry => UnitProperties {
                avg_weapon: WeaponProperties::sword(),
                avg_armor: ArmorProperties::plate(),
                movement_speed: 0.7,  // Slow
                vision_range: 5,      // Helmet limits vision
                base_stress_threshold: 1.2, // Harder to break
                can_charge: false,
                can_skirmish: false,
            },

            UnitType::Spearmen => UnitProperties {
                avg_weapon: WeaponProperties::spear(),
                avg_armor: ArmorProperties::mail(),
                movement_speed: 0.9,
                vision_range: 6,
                base_stress_threshold: 1.0,
                can_charge: false,
                can_skirmish: false,
            },

            UnitType::Archers => UnitProperties {
                avg_weapon: WeaponProperties {
                    edge: Edge::Sharp,
                    mass: Mass::Light,
                    reach: Reach::Grapple, // Melee is weak
                    special: vec![],
                },
                avg_armor: ArmorProperties::leather(),
                movement_speed: 1.0,
                vision_range: 10, // Good eyes
                base_stress_threshold: 0.8, // Vulnerable
                can_charge: false,
                can_skirmish: true,
            },

            UnitType::Crossbowmen => UnitProperties {
                avg_weapon: WeaponProperties {
                    edge: Edge::Sharp,
                    mass: Mass::Medium,
                    reach: Reach::Grapple,
                    special: vec![],
                },
                avg_armor: ArmorProperties::mail(),
                movement_speed: 0.9,
                vision_range: 8,
                base_stress_threshold: 0.9,
                can_charge: false,
                can_skirmish: false,
            },

            UnitType::LightCavalry => UnitProperties {
                avg_weapon: WeaponProperties {
                    edge: Edge::Sharp,
                    mass: Mass::Light,
                    reach: Reach::Medium,
                    special: vec![],
                },
                avg_armor: ArmorProperties::leather(),
                movement_speed: 2.0, // Fast
                vision_range: 10,    // Good scouts
                base_stress_threshold: 0.8,
                can_charge: true,
                can_skirmish: true,
            },

            UnitType::Cavalry => UnitProperties {
                avg_weapon: WeaponProperties::sword(),
                avg_armor: ArmorProperties::mail(),
                movement_speed: 1.8,
                vision_range: 8,
                base_stress_threshold: 1.0,
                can_charge: true,
                can_skirmish: false,
            },

            UnitType::HeavyCavalry => UnitProperties {
                avg_weapon: WeaponProperties {
                    edge: Edge::Sharp,
                    mass: Mass::Heavy,
                    reach: Reach::Medium,
                    special: vec![],
                },
                avg_armor: ArmorProperties::plate(),
                movement_speed: 1.5,
                vision_range: 6,
                base_stress_threshold: 1.3, // Elite
                can_charge: true,
                can_skirmish: false,
            },

            UnitType::HorseArchers => UnitProperties {
                avg_weapon: WeaponProperties {
                    edge: Edge::Sharp,
                    mass: Mass::Light,
                    reach: Reach::Grapple,
                    special: vec![],
                },
                avg_armor: ArmorProperties::leather(),
                movement_speed: 2.0,
                vision_range: 10,
                base_stress_threshold: 0.8,
                can_charge: false,
                can_skirmish: true,
            },

            UnitType::Engineers => UnitProperties {
                avg_weapon: WeaponProperties::fists(),
                avg_armor: ArmorProperties::none(),
                movement_speed: 0.8,
                vision_range: 6,
                base_stress_threshold: 0.7,
                can_charge: false,
                can_skirmish: false,
            },

            UnitType::Scouts => UnitProperties {
                avg_weapon: WeaponProperties::dagger(),
                avg_armor: ArmorProperties::leather(),
                movement_speed: 1.5,
                vision_range: 12, // Best vision
                base_stress_threshold: 0.7,
                can_charge: false,
                can_skirmish: true,
            },

            UnitType::Command => UnitProperties {
                avg_weapon: WeaponProperties::sword(),
                avg_armor: ArmorProperties::mail(),
                movement_speed: 1.5, // Mounted
                vision_range: 8,
                base_stress_threshold: 1.2, // Leaders
                can_charge: false,
                can_skirmish: false,
            },
        }
    }

    /// Is this a mounted unit?
    pub fn is_mounted(&self) -> bool {
        matches!(
            self,
            UnitType::LightCavalry
                | UnitType::Cavalry
                | UnitType::HeavyCavalry
                | UnitType::HorseArchers
                | UnitType::Command
        )
    }

    /// Is this a ranged unit?
    pub fn is_ranged(&self) -> bool {
        matches!(
            self,
            UnitType::Archers | UnitType::Crossbowmen | UnitType::HorseArchers
        )
    }

    /// Can this unit receive cavalry charge bonus?
    pub fn can_charge(&self) -> bool {
        self.default_properties().can_charge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heavy_infantry_slow() {
        let props = UnitType::HeavyInfantry.default_properties();
        assert!(props.movement_speed < 1.0);
    }

    #[test]
    fn test_light_cavalry_fast() {
        let props = UnitType::LightCavalry.default_properties();
        assert!(props.movement_speed > 1.5);
    }

    #[test]
    fn test_scouts_good_vision() {
        let props = UnitType::Scouts.default_properties();
        assert!(props.vision_range > 8);
    }

    #[test]
    fn test_cavalry_is_mounted() {
        assert!(UnitType::LightCavalry.is_mounted());
        assert!(UnitType::HeavyCavalry.is_mounted());
        assert!(!UnitType::Infantry.is_mounted());
    }

    #[test]
    fn test_archers_are_ranged() {
        assert!(UnitType::Archers.is_ranged());
        assert!(UnitType::Crossbowmen.is_ranged());
        assert!(!UnitType::Infantry.is_ranged());
    }

    #[test]
    fn test_spearmen_have_reach() {
        let props = UnitType::Spearmen.default_properties();
        assert_eq!(props.avg_weapon.reach, Reach::Long);
    }
}
