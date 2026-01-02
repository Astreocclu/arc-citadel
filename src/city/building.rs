//! Building archetype with SoA layout

use serde::{Deserialize, Serialize};

/// Type of building
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingType {
    House,
    Farm,
    Workshop,
    Granary,
    Wall,
    Gate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_type_exists() {
        let bt = BuildingType::House;
        assert_eq!(bt, BuildingType::House);
    }
}
