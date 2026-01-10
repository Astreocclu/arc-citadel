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

    /// Housing capacity for residential buildings
    pub fn housing_capacity(&self) -> u32 {
        match self {
            BuildingType::House => 4,
            _ => 0,
        }
    }

    /// Materials required to construct this building
    pub fn required_materials(&self) -> Vec<(ResourceType, u32)> {
        match self {
            BuildingType::House => vec![(ResourceType::Wood, 20), (ResourceType::Stone, 10)],
            BuildingType::Farm => vec![(ResourceType::Wood, 30)],
            BuildingType::Workshop => vec![
                (ResourceType::Wood, 40),
                (ResourceType::Stone, 20),
                (ResourceType::Iron, 5),
            ],
            BuildingType::Granary => vec![(ResourceType::Wood, 50), (ResourceType::Stone, 30)],
            BuildingType::Wall => vec![(ResourceType::Stone, 25)],
            BuildingType::Gate => vec![(ResourceType::Wood, 15), (ResourceType::Iron, 10)],
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
    /// Active recipe ID (None if not producing)
    pub active_recipes: Vec<Option<String>>,
    /// Production progress (0.0 to 1.0)
    pub production_progress: Vec<f32>,
    /// Workers assigned to production
    pub production_workers: Vec<u32>,
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
        // Production fields
        self.active_recipes.push(None);
        self.production_progress.push(0.0);
        self.production_workers.push(0);
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

    /// Start production of a recipe
    pub fn start_production(&mut self, index: usize, recipe_id: String) -> bool {
        if index >= self.count() {
            return false;
        }
        if self.states[index] != BuildingState::Complete {
            return false;
        }
        self.active_recipes[index] = Some(recipe_id);
        self.production_progress[index] = 0.0;
        true
    }

    /// Advance production by given amount, returns true if cycle completed
    pub fn advance_production(&mut self, index: usize, amount: f32) -> bool {
        if index >= self.count() || self.active_recipes[index].is_none() {
            return false;
        }
        self.production_progress[index] += amount;
        if self.production_progress[index] >= 1.0 {
            self.production_progress[index] = 0.0; // Reset for next cycle
            true
        } else {
            false
        }
    }

    /// Iterate over buildings that are producing
    pub fn iter_producing(&self) -> impl Iterator<Item = usize> + '_ {
        self.active_recipes
            .iter()
            .enumerate()
            .filter(|(i, recipe)| recipe.is_some() && self.states[*i] == BuildingState::Complete)
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
        arch.spawn(
            BuildingId::new(),
            BuildingType::House,
            Vec2::new(0.0, 0.0),
            0,
        );
        arch.spawn(
            BuildingId::new(),
            BuildingType::Farm,
            Vec2::new(10.0, 0.0),
            0,
        );

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

    // Task 12: Production state tests
    #[test]
    fn test_building_production_state_initialized() {
        let mut arch = BuildingArchetype::new();
        let id = BuildingId::new();
        arch.spawn(id, BuildingType::Farm, Vec2::new(0.0, 0.0), 0);

        // Production fields should be initialized
        assert_eq!(arch.active_recipes.len(), 1);
        assert_eq!(arch.production_progress.len(), 1);
        assert_eq!(arch.production_workers.len(), 1);

        // Initial values
        assert_eq!(arch.active_recipes[0], None);
        assert_eq!(arch.production_progress[0], 0.0);
        assert_eq!(arch.production_workers[0], 0);
    }

    #[test]
    fn test_building_start_production() {
        let mut arch = BuildingArchetype::new();
        let id = BuildingId::new();
        arch.spawn(id, BuildingType::Farm, Vec2::new(0.0, 0.0), 0);

        // Can't produce while under construction
        assert!(!arch.start_production(0, "farm_food".into()));
        assert_eq!(arch.active_recipes[0], None);

        // Complete it
        arch.states[0] = BuildingState::Complete;
        assert!(arch.start_production(0, "farm_food".into()));
        assert_eq!(arch.active_recipes[0], Some("farm_food".into()));
        assert_eq!(arch.production_progress[0], 0.0);
    }

    #[test]
    fn test_building_start_production_invalid_index() {
        let mut arch = BuildingArchetype::new();

        // Invalid index should return false
        assert!(!arch.start_production(0, "test".into()));
        assert!(!arch.start_production(100, "test".into()));
    }

    #[test]
    fn test_building_advance_production() {
        let mut arch = BuildingArchetype::new();
        let id = BuildingId::new();
        arch.spawn(id, BuildingType::Farm, Vec2::new(0.0, 0.0), 0);
        arch.states[0] = BuildingState::Complete;
        arch.start_production(0, "farm_food".into());

        // Advance production - not complete yet
        assert!(!arch.advance_production(0, 0.5));
        assert!((arch.production_progress[0] - 0.5).abs() < 0.01);

        // Advance more - should complete (0.5 + 0.6 > 1.0)
        assert!(arch.advance_production(0, 0.6));
        // Progress should reset after completion
        assert!((arch.production_progress[0] - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_building_advance_production_no_recipe() {
        let mut arch = BuildingArchetype::new();
        let id = BuildingId::new();
        arch.spawn(id, BuildingType::Farm, Vec2::new(0.0, 0.0), 0);
        arch.states[0] = BuildingState::Complete;
        // No recipe started

        // Should return false when no active recipe
        assert!(!arch.advance_production(0, 0.5));
    }

    #[test]
    fn test_building_iter_producing() {
        let mut arch = BuildingArchetype::new();
        arch.spawn(
            BuildingId::new(),
            BuildingType::Farm,
            Vec2::new(0.0, 0.0),
            0,
        );
        arch.spawn(
            BuildingId::new(),
            BuildingType::Workshop,
            Vec2::new(10.0, 0.0),
            0,
        );
        arch.spawn(
            BuildingId::new(),
            BuildingType::House,
            Vec2::new(20.0, 0.0),
            0,
        );

        // Complete first two
        arch.states[0] = BuildingState::Complete;
        arch.states[1] = BuildingState::Complete;
        // Third stays under construction

        // Start production on first two
        arch.start_production(0, "farm_food".into());
        arch.start_production(1, "smelt_iron".into());

        // Only completed buildings with active recipes should be producing
        let producing: Vec<_> = arch.iter_producing().collect();
        assert_eq!(producing.len(), 2);
        assert!(producing.contains(&0));
        assert!(producing.contains(&1));
        // Index 2 should not be producing (under construction)
        assert!(!producing.contains(&2));
    }

    #[test]
    fn test_building_iter_producing_filters_incomplete() {
        let mut arch = BuildingArchetype::new();
        arch.spawn(
            BuildingId::new(),
            BuildingType::Farm,
            Vec2::new(0.0, 0.0),
            0,
        );

        // Building under construction with recipe set (shouldn't be possible normally, but test the filter)
        arch.active_recipes[0] = Some("farm_food".into());

        // Should NOT be in producing list because building isn't complete
        let producing: Vec<_> = arch.iter_producing().collect();
        assert!(producing.is_empty());
    }

    #[test]
    fn test_building_housing_capacity() {
        assert_eq!(BuildingType::House.housing_capacity(), 4);
        assert_eq!(BuildingType::Farm.housing_capacity(), 0);
        assert_eq!(BuildingType::Workshop.housing_capacity(), 0);
        assert_eq!(BuildingType::Granary.housing_capacity(), 0);
        assert_eq!(BuildingType::Wall.housing_capacity(), 0);
        assert_eq!(BuildingType::Gate.housing_capacity(), 0);
    }
}
