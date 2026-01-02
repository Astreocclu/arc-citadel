//! Building archetype with SoA layout

use crate::core::types::{Tick, Vec2};
use crate::simulation::resource_zone::ResourceType;
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

    /// Materials required to construct this building
    pub fn required_materials(&self) -> Vec<(ResourceType, u32)> {
        match self {
            BuildingType::House => vec![
                (ResourceType::Wood, 20),
                (ResourceType::Stone, 10),
            ],
            BuildingType::Farm => vec![
                (ResourceType::Wood, 30),
            ],
            BuildingType::Workshop => vec![
                (ResourceType::Wood, 40),
                (ResourceType::Stone, 20),
                (ResourceType::Iron, 5),
            ],
            BuildingType::Granary => vec![
                (ResourceType::Wood, 50),
                (ResourceType::Stone, 30),
            ],
            BuildingType::Wall => vec![
                (ResourceType::Stone, 25),
            ],
            BuildingType::Gate => vec![
                (ResourceType::Wood, 15),
                (ResourceType::Iron, 10),
            ],
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

/// Structure of Arrays for building entities
#[derive(Debug, Clone, Default)]
pub struct BuildingArchetype {
    /// Unique identifiers
    pub ids: Vec<BuildingId>,
    /// Type of each building
    pub building_types: Vec<BuildingType>,
    /// Current state
    pub states: Vec<BuildingState>,
    /// Position in world
    pub positions: Vec<Vec2>,
    /// Construction progress (0.0 to work_required)
    pub construction_progress: Vec<f32>,
    /// Currently assigned worker count
    pub assigned_workers: Vec<u32>,
    /// Owning polity (optional)
    pub polity_ids: Vec<Option<u32>>,
    /// Tick when construction started
    pub started_ticks: Vec<Tick>,
    /// Tick when completed (0 if not complete)
    pub completed_ticks: Vec<Tick>,
}

impl BuildingArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn count(&self) -> usize {
        self.ids.len()
    }

    /// Spawn a new building (starts as construction site)
    pub fn spawn(
        &mut self,
        id: BuildingId,
        building_type: BuildingType,
        position: Vec2,
        tick: Tick,
    ) -> usize {
        let index = self.ids.len();
        self.ids.push(id);
        self.building_types.push(building_type);
        self.states.push(BuildingState::UnderConstruction);
        self.positions.push(position);
        self.construction_progress.push(0.0);
        self.assigned_workers.push(0);
        self.polity_ids.push(None);
        self.started_ticks.push(tick);
        self.completed_ticks.push(0);
        index
    }

    pub fn index_of(&self, id: BuildingId) -> Option<usize> {
        self.ids.iter().position(|&b| b == id)
    }

    /// Iterate over buildings under construction
    pub fn iter_under_construction(&self) -> impl Iterator<Item = usize> + '_ {
        self.states
            .iter()
            .enumerate()
            .filter(|(_, state)| **state == BuildingState::UnderConstruction)
            .map(|(i, _)| i)
    }

    /// Iterate over completed buildings
    pub fn iter_complete(&self) -> impl Iterator<Item = usize> + '_ {
        self.states
            .iter()
            .enumerate()
            .filter(|(_, state)| **state == BuildingState::Complete)
            .map(|(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Vec2;

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

    // Task 3: BuildingArchetype tests
    #[test]
    fn test_building_archetype_spawn() {
        let mut arch = BuildingArchetype::new();
        assert_eq!(arch.count(), 0);

        let id = BuildingId::new();
        let idx = arch.spawn(id, BuildingType::House, Vec2::new(10.0, 20.0), 100);

        assert_eq!(arch.count(), 1);
        assert_eq!(idx, 0);
        assert_eq!(arch.building_types[0], BuildingType::House);
        assert_eq!(arch.states[0], BuildingState::UnderConstruction);
        assert_eq!(arch.construction_progress[0], 0.0);
    }

    #[test]
    fn test_building_archetype_index_of() {
        let mut arch = BuildingArchetype::new();
        let id1 = BuildingId::new();
        let id2 = BuildingId::new();

        arch.spawn(id1, BuildingType::House, Vec2::new(0.0, 0.0), 0);
        arch.spawn(id2, BuildingType::Farm, Vec2::new(10.0, 0.0), 0);

        assert_eq!(arch.index_of(id1), Some(0));
        assert_eq!(arch.index_of(id2), Some(1));
        assert_eq!(arch.index_of(BuildingId::new()), None);
    }

    #[test]
    fn test_building_archetype_iter_under_construction() {
        let mut arch = BuildingArchetype::new();
        arch.spawn(BuildingId::new(), BuildingType::House, Vec2::new(0.0, 0.0), 0);
        arch.spawn(BuildingId::new(), BuildingType::Farm, Vec2::new(10.0, 0.0), 0);

        // Both are under construction
        let under_construction: Vec<_> = arch.iter_under_construction().collect();
        assert_eq!(under_construction.len(), 2);

        // Complete one
        arch.states[0] = BuildingState::Complete;

        let under_construction: Vec<_> = arch.iter_under_construction().collect();
        assert_eq!(under_construction.len(), 1);
        assert_eq!(under_construction[0], 1);
    }

    // Task 9: Material requirements tests
    #[test]
    fn test_building_required_materials() {
        use crate::simulation::resource_zone::ResourceType;

        // House requires Wood and Stone
        let materials = BuildingType::House.required_materials();
        assert_eq!(materials.len(), 2);
        assert!(materials.contains(&(ResourceType::Wood, 20)));
        assert!(materials.contains(&(ResourceType::Stone, 10)));

        // Workshop needs iron
        let workshop_mats = BuildingType::Workshop.required_materials();
        assert!(workshop_mats.iter().any(|(r, _)| *r == ResourceType::Iron));

        // Wall only needs stone
        let wall_mats = BuildingType::Wall.required_materials();
        assert_eq!(wall_mats.len(), 1);
        assert!(wall_mats.contains(&(ResourceType::Stone, 25)));
    }

    #[test]
    fn test_all_building_types_have_materials() {
        // Ensure all building types have material costs defined (non-empty)
        let building_types = [
            BuildingType::House,
            BuildingType::Farm,
            BuildingType::Workshop,
            BuildingType::Granary,
            BuildingType::Wall,
            BuildingType::Gate,
        ];

        for bt in building_types {
            let materials = bt.required_materials();
            assert!(
                !materials.is_empty(),
                "BuildingType {:?} should have material costs defined",
                bt
            );
        }
    }
}
