//! Building archetype with SoA layout

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

impl BuildingType {
    /// Base work required to construct this building type
    pub fn work_required(&self) -> f32 {
        match self {
            BuildingType::House => 100.0,
            BuildingType::Farm => 150.0,
            BuildingType::Workshop => 200.0,
            BuildingType::Granary => 120.0,
            BuildingType::Wall => 80.0,
            BuildingType::Gate => 60.0,
        }
    }

    /// Maximum workers that can contribute effectively
    pub fn max_workers(&self) -> u32 {
        match self {
            BuildingType::House => 3,
            BuildingType::Farm => 5,
            BuildingType::Workshop => 4,
            BuildingType::Granary => 4,
            BuildingType::Wall => 6,
            BuildingType::Gate => 4,
        }
    }

    /// Size of the building (width, height in units)
    pub fn size(&self) -> (f32, f32) {
        match self {
            BuildingType::House => (2.0, 2.0),
            BuildingType::Farm => (4.0, 4.0),
            BuildingType::Workshop => (3.0, 3.0),
            BuildingType::Granary => (3.0, 3.0),
            BuildingType::Wall => (1.0, 1.0),
            BuildingType::Gate => (2.0, 1.0),
        }
    }
}

/// Current state of a building
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingState {
    /// Construction site - not yet usable
    UnderConstruction,
    /// Fully operational
    Complete,
    /// Needs repair before use
    Damaged,
}

/// Unique building identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BuildingId(pub u64);

impl BuildingId {
    /// Create a new unique BuildingId using UUID
    pub fn new() -> Self {
        Self(Uuid::new_v4().as_u128() as u64)
    }
}

impl Default for BuildingId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_type_exists() {
        let bt = BuildingType::House;
        assert_eq!(bt, BuildingType::House);
    }

    #[test]
    fn test_building_state_variants() {
        // Test that all BuildingState variants exist and are distinct
        let under_construction = BuildingState::UnderConstruction;
        let complete = BuildingState::Complete;
        let damaged = BuildingState::Damaged;

        assert_ne!(under_construction, complete);
        assert_ne!(complete, damaged);
        assert_ne!(under_construction, damaged);
    }

    #[test]
    fn test_building_id_unique() {
        // Test that BuildingId::new() generates unique IDs
        let id1 = BuildingId::new();
        let id2 = BuildingId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_building_work_required() {
        assert_eq!(BuildingType::House.work_required(), 100.0);
        assert_eq!(BuildingType::Farm.work_required(), 150.0);
        assert!(BuildingType::Workshop.work_required() > BuildingType::House.work_required());
    }

    #[test]
    fn test_building_max_workers() {
        assert_eq!(BuildingType::House.max_workers(), 3);
        assert!(BuildingType::Farm.max_workers() >= BuildingType::House.max_workers());
    }

    #[test]
    fn test_building_size() {
        let (w, h) = BuildingType::House.size();
        assert_eq!((w, h), (2.0, 2.0));

        let (w, h) = BuildingType::Farm.size();
        assert_eq!((w, h), (4.0, 4.0));
    }
}
