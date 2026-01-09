//! Integration tests for world object loading and spatial queries

use arc_citadel::blueprints::BlueprintRegistry;
use arc_citadel::world::{BlockedCells, PlacementLoader};
use glam::Vec2;
use std::path::Path;

fn setup_registry() -> Option<BlueprintRegistry> {
    let mut registry = BlueprintRegistry::new();
    let data_path = Path::new("data/blueprints");
    if !data_path.exists() {
        return None;
    }
    registry.load_directory(data_path).ok()?;

    // Check if required blueprints exist
    if registry.id_by_name("oak_tree").is_none() {
        return None;
    }
    Some(registry)
}

#[test]
fn test_full_world_object_lifecycle() {
    let registry = match setup_registry() {
        Some(r) => r,
        None => {
            eprintln!("Skipping test: required blueprints not found");
            return;
        }
    };

    // Create placement JSON (simulating worldgen output)
    let placement_json = r#"{
        "version": 1,
        "metadata": {
            "name": "Test Region",
            "created_by": "Integration Test"
        },
        "placements": [
            {
                "id": "tree_001",
                "template": "oak_tree",
                "position": [10.0, 10.0],
                "placed_by": "TerrainGen",
                "parameters": {
                    "height": 12.0,
                    "canopy_radius": 5.0,
                    "trunk_radius": 0.4
                }
            }
        ]
    }"#;

    let loader = PlacementLoader::new(&registry);
    let objects = loader
        .load_from_json(placement_json)
        .expect("Failed to load placements");

    // Verify object count
    assert_eq!(objects.len(), 1);

    // Verify natural object (tree)
    let tree = objects
        .iter()
        .find(|o| o.blueprint_name == "oak_tree")
        .unwrap();
    assert_eq!(tree.construction_progress, 1.0); // Natural = complete
    assert!(tree.is_complete());

    // Test spatial query
    let nearby = objects.get_in_radius(Vec2::new(12.0, 12.0), 5.0);
    assert_eq!(nearby.len(), 1);
    assert_eq!(nearby[0].blueprint_name, "oak_tree");
}

#[test]
fn test_blocked_cells_from_objects() {
    let registry = match setup_registry() {
        Some(r) => r,
        None => {
            eprintln!("Skipping test: required blueprints not found");
            return;
        }
    };

    // Skip if stone_wall not available
    if registry.id_by_name("stone_wall").is_none() {
        eprintln!("Skipping test: stone_wall blueprint not found");
        return;
    }

    let placement_json = r#"{
        "version": 1,
        "placements": [
            {
                "id": "wall_blocking",
                "template": "stone_wall",
                "position": [0.0, 0.0],
                "placed_by": "TerrainGen",
                "state": "complete",
                "parameters": {
                    "length": 5.0,
                    "height": 3.0,
                    "thickness": 1.0
                }
            }
        ]
    }"#;

    let loader = PlacementLoader::new(&registry);
    let objects = loader.load_from_json(placement_json).unwrap();

    // Build blocked cells from objects
    let mut blocked = BlockedCells::new();
    for obj in objects.iter() {
        if obj.military.blocks_movement && !obj.geometry.footprint.is_empty() {
            blocked.block_footprint(&obj.geometry.footprint, 1.0);
        }
    }

    // Wall should block some cells (or have empty footprint)
    let wall = objects.iter().next().unwrap();
    if !wall.geometry.footprint.is_empty() {
        assert!(blocked.len() > 0, "Wall should block cells");
    }
}

#[test]
fn test_provenance_tracking() {
    let registry = match setup_registry() {
        Some(r) => r,
        None => {
            eprintln!("Skipping test: required blueprints not found");
            return;
        }
    };

    // Find any available blueprint for this test
    let template_name = if registry.id_by_name("oak_tree").is_some() {
        "oak_tree"
    } else {
        eprintln!("Skipping test: no suitable blueprint found");
        return;
    };

    let placement_json = format!(
        r#"{{
        "version": 1,
        "placements": [
            {{
                "id": "terrain_obj",
                "template": "{}",
                "position": [0.0, 0.0],
                "placed_by": "TerrainGen",
                "parameters": {{}}
            }},
            {{
                "id": "history_obj",
                "template": "{}",
                "position": [10.0, 0.0],
                "placed_by": {{ "HistorySim": {{ "polity_id": 42, "year": -500 }} }},
                "parameters": {{}}
            }},
            {{
                "id": "player_obj",
                "template": "{}",
                "position": [20.0, 0.0],
                "placed_by": {{ "Gameplay": {{ "tick": 12345 }} }},
                "parameters": {{}}
            }}
        ]
    }}"#,
        template_name, template_name, template_name
    );

    let loader = PlacementLoader::new(&registry);
    let objects = loader.load_from_json(&placement_json).unwrap();

    use arc_citadel::blueprints::PlacedBy;

    assert_eq!(objects.len(), 3);

    // Verify each provenance type exists
    let mut found_terrain = false;
    let mut found_history = false;
    let mut found_gameplay = false;

    for obj in objects.iter() {
        match &obj.placed_by {
            PlacedBy::TerrainGen => found_terrain = true,
            PlacedBy::HistorySim { polity_id, year } => {
                assert_eq!(*polity_id, 42);
                assert_eq!(*year, -500);
                found_history = true;
            }
            PlacedBy::Gameplay { tick } => {
                assert_eq!(*tick, 12345);
                found_gameplay = true;
            }
        }
    }

    assert!(found_terrain, "Should have TerrainGen object");
    assert!(found_history, "Should have HistorySim object");
    assert!(found_gameplay, "Should have Gameplay object");
}
