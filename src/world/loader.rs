//! Load world objects from JSON placement files
//!
//! This module provides `PlacementLoader` which converts JSON placement files
//! (from worldgen) into `WorldObjects` containing instantiated blueprints.

use crate::blueprints::BlueprintRegistry;
use crate::world::objects::WorldObjects;
use crate::world::placement::{ObjectState, PlacementFile};
use glam::Vec2;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur when loading placements
#[derive(Debug, Error)]
pub enum LoadError {
    /// JSON parsing failed
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),
    /// Referenced blueprint was not found in registry
    #[error("Blueprint not found: {0}")]
    BlueprintNotFound(String),
    /// Blueprint instantiation failed
    #[error("Blueprint instantiation failed: {0}")]
    InstantiationFailed(String),
    /// File I/O error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Loader that converts JSON placement files into WorldObjects
pub struct PlacementLoader<'a> {
    registry: &'a BlueprintRegistry,
}

impl<'a> PlacementLoader<'a> {
    /// Create a new loader with the given blueprint registry
    pub fn new(registry: &'a BlueprintRegistry) -> Self {
        Self { registry }
    }

    /// Load placements from a JSON string
    pub fn load_from_json(&self, json: &str) -> Result<WorldObjects, LoadError> {
        let file: PlacementFile = serde_json::from_str(json)?;
        self.load_from_placements(&file)
    }

    /// Load placements from a JSON file on disk
    pub fn load_from_file(&self, path: &Path) -> Result<WorldObjects, LoadError> {
        let content = std::fs::read_to_string(path)?;
        self.load_from_json(&content)
    }

    /// Process a PlacementFile and create WorldObjects
    fn load_from_placements(&self, file: &PlacementFile) -> Result<WorldObjects, LoadError> {
        let mut objects = WorldObjects::new();

        for placement in &file.placements {
            // Look up blueprint ID by name
            let blueprint_id = self
                .registry
                .id_by_name(&placement.template)
                .ok_or_else(|| LoadError::BlueprintNotFound(placement.template.clone()))?;

            // Convert PlacedByJson to runtime PlacedBy
            let placed_by = placement.placed_by.to_runtime();

            // Extract position and rotation
            let position = Vec2::new(placement.position[0], placement.position[1]);
            let rotation = placement.rotation_deg.unwrap_or(0.0).to_radians();

            // Instantiate the blueprint
            let mut instance = self
                .registry
                .instantiate(
                    blueprint_id,
                    placement.parameters.clone(),
                    position,
                    rotation,
                    placed_by,
                    None, // owner - could be extended later
                )
                .map_err(|e| LoadError::InstantiationFailed(e.to_string()))?;

            // Handle construction state
            // The registry sets initial state based on origin type.
            // For placements, we override based on explicit state.
            match placement.state {
                Some(ObjectState::Complete) | None => {
                    // Complete or no state (natural) = fully constructed
                    instance.construction_progress = 1.0;
                    instance.construction_stage = Some("complete".to_string());
                    // Update current_hp to max since it's complete
                    instance.current_hp = instance.max_hp;
                }
                Some(ObjectState::UnderConstruction) => {
                    instance.construction_progress = placement.construction_progress.unwrap_or(0.0);
                    // HP scales with construction progress
                    instance.current_hp = instance.max_hp * instance.construction_progress;
                }
            }

            // Handle damage state
            if let Some(ref damage_state) = placement.damage_state {
                instance.damage_state = damage_state.clone();
            }

            // Handle HP ratio override
            if let Some(hp_ratio) = placement.current_hp_ratio {
                instance.current_hp = instance.max_hp * hp_ratio;
            }

            objects.add(instance);
        }

        Ok(objects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprints::BlueprintRegistry;
    use std::path::Path;

    fn setup_registry() -> BlueprintRegistry {
        let mut registry = BlueprintRegistry::new();
        let data_path = Path::new("data/blueprints");
        if data_path.exists() {
            registry.load_directory(data_path).ok();
        }
        registry
    }

    #[test]
    fn test_load_placement_file() {
        let registry = setup_registry();

        // Skip test if blueprints not loaded
        if registry.id_by_name("oak_tree").is_none() {
            eprintln!("Skipping test: oak_tree blueprint not found");
            return;
        }

        let json = r#"{
            "version": 1,
            "placements": [
                {
                    "id": "tree_001",
                    "template": "oak_tree",
                    "position": [50.0, 60.0],
                    "placed_by": "TerrainGen",
                    "parameters": { "height": 15.0, "canopy_radius": 6.0, "trunk_radius": 0.5 }
                }
            ]
        }"#;

        let loader = PlacementLoader::new(&registry);
        let objects = loader.load_from_json(json).unwrap();

        assert_eq!(objects.len(), 1);
        // Get first object
        let obj = objects.iter().next().unwrap();
        assert_eq!(obj.blueprint_name, "oak_tree");
        assert_eq!(obj.construction_progress, 1.0); // Natural = complete
    }

    #[test]
    fn test_load_constructed_complete() {
        let registry = setup_registry();

        // Skip test if blueprints not loaded
        if registry.id_by_name("stone_wall").is_none() {
            eprintln!("Skipping test: stone_wall blueprint not found");
            return;
        }

        let json = r#"{
            "version": 1,
            "placements": [
                {
                    "id": "wall_001",
                    "template": "stone_wall",
                    "position": [100.0, 100.0],
                    "placed_by": { "HistorySim": { "polity_id": 1, "year": -100 } },
                    "state": "complete",
                    "parameters": { "length": 10.0, "height": 3.0, "thickness": 0.6 }
                }
            ]
        }"#;

        let loader = PlacementLoader::new(&registry);
        let objects = loader.load_from_json(json).unwrap();

        let wall = objects.iter().next().unwrap();
        assert_eq!(wall.construction_progress, 1.0);
        // HP should equal max for complete construction
        assert_eq!(wall.current_hp, wall.max_hp);
    }

    #[test]
    fn test_load_under_construction() {
        let registry = setup_registry();

        // Skip test if blueprints not loaded
        if registry.id_by_name("stone_wall").is_none() {
            eprintln!("Skipping test: stone_wall blueprint not found");
            return;
        }

        let json = r#"{
            "version": 1,
            "placements": [
                {
                    "id": "wall_002",
                    "template": "stone_wall",
                    "position": [200.0, 100.0],
                    "placed_by": { "Gameplay": { "tick": 500 } },
                    "state": "under_construction",
                    "construction_progress": 0.3,
                    "parameters": { "length": 8.0, "height": 2.5, "thickness": 0.5 }
                }
            ]
        }"#;

        let loader = PlacementLoader::new(&registry);
        let objects = loader.load_from_json(json).unwrap();

        let wall = objects.iter().next().unwrap();
        assert!((wall.construction_progress - 0.3).abs() < 0.001);
        // HP should be 30% of max
        let expected_hp = wall.max_hp * 0.3;
        assert!((wall.current_hp - expected_hp).abs() < 0.001);
    }

    #[test]
    fn test_load_damaged_object() {
        let registry = setup_registry();

        // Skip test if blueprints not loaded
        if registry.id_by_name("stone_wall").is_none() {
            eprintln!("Skipping test: stone_wall blueprint not found");
            return;
        }

        let json = r#"{
            "version": 1,
            "placements": [
                {
                    "id": "wall_003",
                    "template": "stone_wall",
                    "position": [300.0, 100.0],
                    "placed_by": { "HistorySim": { "polity_id": 2, "year": -50 } },
                    "state": "complete",
                    "damage_state": "damaged",
                    "current_hp_ratio": 0.6,
                    "parameters": { "length": 5.0, "height": 2.0, "thickness": 0.5 }
                }
            ]
        }"#;

        let loader = PlacementLoader::new(&registry);
        let objects = loader.load_from_json(json).unwrap();

        let wall = objects.iter().next().unwrap();
        assert_eq!(wall.damage_state, "damaged");
        // HP should be 60% of max
        let expected_hp = wall.max_hp * 0.6;
        assert!((wall.current_hp - expected_hp).abs() < 0.001);
    }

    #[test]
    fn test_blueprint_not_found() {
        let registry = setup_registry();

        let json = r#"{
            "version": 1,
            "placements": [
                {
                    "id": "nonexistent_001",
                    "template": "nonexistent_blueprint",
                    "position": [0.0, 0.0],
                    "placed_by": "TerrainGen",
                    "parameters": {}
                }
            ]
        }"#;

        let loader = PlacementLoader::new(&registry);
        let result = loader.load_from_json(json);

        assert!(result.is_err());
        match result {
            Err(LoadError::BlueprintNotFound(name)) => {
                assert_eq!(name, "nonexistent_blueprint");
            }
            _ => panic!("Expected BlueprintNotFound error"),
        }
    }

    #[test]
    fn test_json_parse_error() {
        let registry = setup_registry();

        let json = r#"{ invalid json }"#;

        let loader = PlacementLoader::new(&registry);
        let result = loader.load_from_json(json);

        assert!(result.is_err());
        assert!(matches!(result, Err(LoadError::JsonError(_))));
    }

    #[test]
    fn test_multiple_placements() {
        let registry = setup_registry();

        // Skip test if blueprints not loaded
        if registry.id_by_name("oak_tree").is_none()
            || registry.id_by_name("rock_outcrop").is_none()
        {
            eprintln!("Skipping test: required blueprints not found");
            return;
        }

        let json = r#"{
            "version": 1,
            "placements": [
                {
                    "id": "tree_001",
                    "template": "oak_tree",
                    "position": [10.0, 10.0],
                    "placed_by": "TerrainGen",
                    "parameters": {}
                },
                {
                    "id": "tree_002",
                    "template": "oak_tree",
                    "position": [20.0, 20.0],
                    "placed_by": "TerrainGen",
                    "parameters": {}
                },
                {
                    "id": "rock_001",
                    "template": "rock_outcrop",
                    "position": [30.0, 30.0],
                    "placed_by": "TerrainGen",
                    "parameters": {}
                }
            ]
        }"#;

        let loader = PlacementLoader::new(&registry);
        let objects = loader.load_from_json(json).unwrap();

        assert_eq!(objects.len(), 3);
    }

    #[test]
    fn test_rotation() {
        let registry = setup_registry();

        // Skip test if blueprints not loaded
        if registry.id_by_name("stone_wall").is_none() {
            eprintln!("Skipping test: stone_wall blueprint not found");
            return;
        }

        let json = r#"{
            "version": 1,
            "placements": [
                {
                    "id": "wall_rot",
                    "template": "stone_wall",
                    "position": [0.0, 0.0],
                    "rotation_deg": 90.0,
                    "placed_by": "TerrainGen",
                    "parameters": { "length": 5.0, "height": 2.0, "thickness": 0.5 }
                }
            ]
        }"#;

        let loader = PlacementLoader::new(&registry);
        let objects = loader.load_from_json(json).unwrap();

        let wall = objects.iter().next().unwrap();
        // 90 degrees in radians is approximately PI/2
        let expected_rotation = 90.0_f32.to_radians();
        assert!((wall.rotation - expected_rotation).abs() < 0.001);
    }

    #[test]
    fn test_empty_placements() {
        let registry = setup_registry();

        let json = r#"{
            "version": 1,
            "placements": []
        }"#;

        let loader = PlacementLoader::new(&registry);
        let objects = loader.load_from_json(json).unwrap();

        assert!(objects.is_empty());
    }
}
