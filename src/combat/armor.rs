//! Armor properties for categorical combat resolution
//!
//! Armor has exactly three properties: Rigidity, Padding, Coverage.
//! These determine outcomes via lookup tables against weapon properties.

use serde::{Deserialize, Serialize};

/// Material hardness - determines if edge can penetrate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rigidity {
    /// Clothing, robes
    Cloth,
    /// Cured hide, thick cloth
    Leather,
    /// Interlocking rings
    Mail,
    /// Solid metal
    Plate,
}

/// Impact absorption - determines trauma from mass
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Padding {
    /// Bare skin or cloth only
    None,
    /// Thin gambeson, leather backing
    Light,
    /// Full gambeson, layered padding
    Heavy,
}

/// How much of body is protected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Coverage {
    /// Unarmored
    None,
    /// Some gaps (standard armor)
    Partial,
    /// Nearly complete (full harness)
    Full,
}

/// Complete armor properties
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmorProperties {
    pub rigidity: Rigidity,
    pub padding: Padding,
    pub coverage: Coverage,
}

impl ArmorProperties {
    /// No armor at all
    pub fn none() -> Self {
        Self {
            rigidity: Rigidity::Cloth,
            padding: Padding::None,
            coverage: Coverage::None,
        }
    }

    /// Light armor (leather)
    pub fn leather() -> Self {
        Self {
            rigidity: Rigidity::Leather,
            padding: Padding::Light,
            coverage: Coverage::Partial,
        }
    }

    /// Medium armor (mail)
    pub fn mail() -> Self {
        Self {
            rigidity: Rigidity::Mail,
            padding: Padding::Light,
            coverage: Coverage::Partial,
        }
    }

    /// Heavy armor (plate)
    pub fn plate() -> Self {
        Self {
            rigidity: Rigidity::Plate,
            padding: Padding::Heavy,
            coverage: Coverage::Full,
        }
    }
}

impl Default for ArmorProperties {
    fn default() -> Self {
        Self::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plate_armor() {
        let plate = ArmorProperties {
            rigidity: Rigidity::Plate,
            padding: Padding::Heavy,
            coverage: Coverage::Full,
        };
        assert_eq!(plate.rigidity, Rigidity::Plate);
    }

    #[test]
    fn test_unarmored() {
        let naked = ArmorProperties::none();
        assert_eq!(naked.rigidity, Rigidity::Cloth);
        assert_eq!(naked.padding, Padding::None);
        assert_eq!(naked.coverage, Coverage::None);
    }

    #[test]
    fn test_armor_presets() {
        let leather = ArmorProperties::leather();
        assert_eq!(leather.rigidity, Rigidity::Leather);

        let mail = ArmorProperties::mail();
        assert_eq!(mail.rigidity, Rigidity::Mail);

        let plate = ArmorProperties::plate();
        assert_eq!(plate.rigidity, Rigidity::Plate);
        assert_eq!(plate.padding, Padding::Heavy);
    }
}
