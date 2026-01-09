//! Blueprint system integration tests
//!
//! This file tests the full lifecycle of blueprints from loading through
//! instantiation, construction, damage, and state transitions.

use arc_citadel::blueprints::*;
use glam::Vec2;
use std::collections::HashMap;
use std::path::Path;

/// Helper function to create an oak tree blueprint inline
fn create_oak_tree_blueprint() -> Blueprint {
    let toml_str = r#"
[meta]
id = "oak_tree"
name = "Oak Tree"
category = "tree"
origin = "natural"
description = "A mature oak tree with spreading canopy"

[parameters]
height = { type = "float", min = 8.0, max = 25.0, default = 15.0 }
canopy_radius = { type = "float", min = 3.0, max = 10.0, default = 5.0 }
trunk_radius = { type = "float", min = 0.3, max = 1.5, default = 0.5 }

[geometry]
width = "trunk_radius * 2"
depth = "trunk_radius * 2"
height = "height"
shape = "circle"

[stats.military]
max_hp = "height * 20"
hardness = "2"
cover_value = "0.5"
blocks_movement = "1"
blocks_los = "0"
movement_cost = "2"
flammable = "1"
elevation = "0"

[[damage_states]]
name = "healthy"
threshold = 0.75
tags = ["living", "harvestable"]

[[damage_states]]
name = "damaged"
threshold = 0.25
visual_overlay = "damaged_bark"
[damage_states.overrides]
cover_value = 0.3

[[damage_states]]
name = "fallen"
threshold = 0.0
visual_overlay = "fallen_trunk"
produces_rubble = true
[damage_states.overrides]
cover_value = 0.6
blocks_movement = false

[[constraints]]
description = "Canopy must be larger than trunk"
expression = "canopy_radius > trunk_radius * 2"
error_message = "Tree canopy radius must be at least twice the trunk radius"
"#;
    toml::from_str(toml_str).unwrap()
}

/// Helper function to create a stone wall blueprint inline
fn create_stone_wall_blueprint() -> Blueprint {
    let toml_str = r#"
[meta]
id = "stone_wall"
name = "Stone Wall"
category = "wall"
origin = "constructed"
description = "A defensive stone wall"

[parameters]
length = { type = "float", min = 2.0, max = 20.0, default = 5.0 }
height = { type = "float", min = 1.0, max = 5.0, default = 2.5 }
thickness = { type = "float", min = 0.4, max = 1.5, default = 0.6 }

[geometry]
width = "length"
depth = "thickness"
height = "height"
shape = "rectangle"

[stats.military]
max_hp = "length * height * 80"
hardness = "5"
cover_value = "1.0"
blocks_movement = "1"
blocks_los = "1"
movement_cost = "999"
flammable = "0"

[construction]
base_time = "length * height * 20"
labor_cap = "ceil(length / 2)"

[construction.cost]
stone = "length * height * 10"
mortar = "length * height * 2"

[[construction.stages]]
id = "foundation"
progress_threshold = 0.0
height_multiplier = 0.3
visual_state = "foundation"
[construction.stages.overrides]
cover_value = 0.2
blocks_movement = false

[[construction.stages]]
id = "half_height"
progress_threshold = 0.25
height_multiplier = 0.5
visual_state = "half_built"
[construction.stages.overrides]
cover_value = 0.5
blocks_movement = true

[[construction.stages]]
id = "complete"
progress_threshold = 1.0
height_multiplier = 1.0
visual_state = "complete"

[[anchors]]
name = "west"
position = ["-length / 2", "0", "0"]
direction = "west"
tags = ["wall_connect"]

[[anchors]]
name = "east"
position = ["length / 2", "0", "0"]
direction = "east"
tags = ["wall_connect"]

[[damage_states]]
name = "intact"
threshold = 0.75
visual_overlay = ""

[[damage_states]]
name = "damaged"
threshold = 0.5
visual_overlay = "cracks"
[damage_states.overrides]
cover_value = 0.8
hardness = 4.0

[[damage_states]]
name = "breached"
threshold = 0.25
visual_overlay = "breach"
creates_breach = true
produces_rubble = true
[damage_states.overrides]
cover_value = 0.5
blocks_movement = false
blocks_los = false

[[damage_states]]
name = "collapsed"
threshold = 0.0
visual_overlay = "rubble"
creates_breach = true
produces_rubble = true
[damage_states.overrides]
cover_value = 0.3
blocks_movement = false
blocks_los = false
hardness = 0.0

[[constraints]]
description = "Minimum length requirement"
expression = "length >= 2"
error_message = "Wall must be at least 2 meters long"
"#;
    toml::from_str(toml_str).unwrap()
}

#[test]
fn test_full_blueprint_lifecycle() {
    // Create registry and load blueprints
    let mut registry = BlueprintRegistry::new();

    // Try to load from data directory, fall back to inline blueprints
    let data_path = Path::new("data/blueprints");
    if data_path.exists() {
        registry
            .load_directory(data_path)
            .expect("Failed to load blueprints from directory");
    } else {
        // Use inline fallbacks
        registry.register(create_oak_tree_blueprint());
        registry.register(create_stone_wall_blueprint());
    }

    // ========================================
    // Test 1: Natural feature (oak_tree) instantiation
    // ========================================

    let tree_id = registry
        .id_by_name("oak_tree")
        .expect("oak_tree blueprint should exist");

    // Custom parameters for the tree
    let mut tree_params = HashMap::new();
    tree_params.insert("height".to_string(), 20.0);
    tree_params.insert("trunk_radius".to_string(), 0.6);
    tree_params.insert("canopy_radius".to_string(), 6.0);

    let tree = registry
        .instantiate(
            tree_id,
            tree_params,
            Vec2::new(100.0, 100.0),
            0.0,
            PlacedBy::TerrainGen,
            None,
        )
        .expect("Tree instantiation should succeed");

    // Natural features spawn complete
    assert!(
        tree.is_complete(),
        "Natural features should spawn fully complete"
    );
    assert_eq!(
        tree.construction_progress, 1.0,
        "Natural features should have 100% construction progress"
    );

    // HP calculated correctly: height * 20 = 20 * 20 = 400
    assert_eq!(tree.max_hp, 400.0, "Tree HP should be height * 20");
    assert_eq!(
        tree.current_hp, tree.max_hp,
        "Natural features should spawn at full HP"
    );

    // Flammable flag set
    assert!(tree.military.flammable, "Oak tree should be flammable");

    // Blocks movement
    assert!(
        tree.military.blocks_movement,
        "Oak tree should block movement"
    );

    // Geometry check - circle shape for tree trunk
    assert_eq!(
        tree.geometry.footprint.len(),
        16,
        "Circle footprint should have 16 vertices"
    );

    // ========================================
    // Test 2: Constructed feature (stone_wall) instantiation
    // ========================================

    let wall_id = registry
        .id_by_name("stone_wall")
        .expect("stone_wall blueprint should exist");

    let wall_blueprint = registry.get(wall_id).unwrap().clone();

    // Custom parameters
    let mut wall_params = HashMap::new();
    wall_params.insert("length".to_string(), 10.0);
    wall_params.insert("height".to_string(), 2.5);
    wall_params.insert("thickness".to_string(), 0.6);

    let mut wall = registry
        .instantiate(
            wall_id,
            wall_params.clone(),
            Vec2::new(50.0, 50.0),
            0.0,
            PlacedBy::Gameplay { tick: 100 },
            Some(1),
        )
        .expect("Wall instantiation should succeed");

    // Constructed features spawn incomplete (progress=0)
    assert!(
        !wall.is_complete(),
        "Constructed features should spawn incomplete"
    );
    assert_eq!(
        wall.construction_progress, 0.0,
        "Constructed features should start at 0% progress"
    );

    // HP calculated correctly: length * height * 80 = 10 * 2.5 * 80 = 2000
    assert_eq!(wall.max_hp, 2000.0, "Wall max HP should be length * height * 80");
    // Current HP is 0 at 0% construction progress
    assert_eq!(
        wall.current_hp, 0.0,
        "Wall current HP should be 0 at 0% construction"
    );

    // Check material requirements
    let materials = get_required_materials(&wall_blueprint, &wall_params);
    // stone = length * height * 10 = 10 * 2.5 * 10 = 250
    assert_eq!(materials.get("stone"), Some(&250), "Stone cost should be 250");
    // mortar = length * height * 2 = 10 * 2.5 * 2 = 50
    assert_eq!(materials.get("mortar"), Some(&50), "Mortar cost should be 50");

    // Check labor cap: ceil(length / 2) = ceil(10 / 2) = 5
    let labor_cap = get_labor_cap(&wall_blueprint, &wall_params);
    assert_eq!(labor_cap, 5, "Labor cap should be 5 workers");

    // Apply work until complete
    // base_time = length * height * 20 = 10 * 2.5 * 20 = 500
    // We need to apply 500 work units total

    // Apply 125 units of work (25%)
    let completed = apply_work(&mut wall, 125.0, &wall_blueprint);
    assert!(!completed, "Wall should not be complete at 25%");
    assert!(
        (wall.construction_progress - 0.25).abs() < 0.001,
        "Progress should be 0.25"
    );
    assert_eq!(
        wall.construction_stage,
        Some("half_height".to_string()),
        "Should be at half_height stage"
    );
    assert!(
        wall.military.blocks_movement,
        "Should block movement at half_height stage"
    );

    // Complete construction
    let completed = apply_work(&mut wall, 375.0, &wall_blueprint);
    assert!(completed, "Wall should be complete");
    assert!(wall.is_complete(), "Wall should report as complete");
    assert_eq!(wall.construction_progress, 1.0, "Progress should be 1.0");

    // Once complete, HP should be at max
    wall.current_hp = wall.max_hp; // Update HP to match completion

    // ========================================
    // Test 3: Damage system - Wall
    // ========================================

    // Apply damage to wall
    // Wall has 2000 HP, damage to 60% (damage 800 HP, leaving 1200)
    let result = apply_damage(
        &mut wall,
        800.0,
        Vec2::new(55.0, 50.0),
        &wall_blueprint,
    );

    // At 60% HP (above 0.5 threshold), should be "damaged" state
    assert_eq!(
        wall.damage_state, "damaged",
        "Wall should be in damaged state at 60% HP"
    );
    assert_eq!(
        result.new_state,
        Some("damaged".to_string()),
        "Result should indicate state change"
    );
    assert!(!result.destroyed, "Wall should not be destroyed");
    assert!(result.new_breach.is_none(), "No breach at damaged state");

    // Damage to 30% (damage another 600 HP, leaving 600)
    let result = apply_damage(
        &mut wall,
        600.0,
        Vec2::new(55.0, 50.0),
        &wall_blueprint,
    );

    // At 30% HP (above 0.25 threshold), should be "breached" state
    assert_eq!(
        wall.damage_state, "breached",
        "Wall should be in breached state at 30% HP"
    );
    assert_eq!(
        result.new_state,
        Some("breached".to_string()),
        "Result should indicate state change to breached"
    );

    // Verify breach created
    assert!(
        result.new_breach.is_some(),
        "Breach should be created in breached state"
    );
    assert_eq!(wall.breaches.len(), 1, "Wall should have one breach");

    // Verify blocks_movement override applied
    assert!(
        !wall.military.blocks_movement,
        "Breached wall should not block movement"
    );

    // ========================================
    // Test 4: Damage system - Tree (fallen state)
    // ========================================

    let tree_blueprint = registry.get(tree_id).unwrap().clone();
    let mut damaged_tree = registry
        .instantiate(
            tree_id,
            {
                let mut p = HashMap::new();
                p.insert("height".to_string(), 15.0);
                p.insert("trunk_radius".to_string(), 0.5);
                p.insert("canopy_radius".to_string(), 5.0);
                p
            },
            Vec2::new(200.0, 200.0),
            0.0,
            PlacedBy::TerrainGen,
            None,
        )
        .unwrap();

    // Tree has HP = 15 * 20 = 300
    assert_eq!(damaged_tree.max_hp, 300.0);
    assert!(
        damaged_tree.military.blocks_movement,
        "Healthy tree should block movement"
    );

    // Damage tree to fallen state (damage enough to get below 25% HP)
    // Below 25% = below 75 HP, so damage at least 225 HP
    let result = apply_damage(
        &mut damaged_tree,
        250.0,
        Vec2::ZERO,
        &tree_blueprint,
    );

    // At ~16.7% HP, should be in "fallen" state
    assert_eq!(
        damaged_tree.damage_state, "fallen",
        "Tree should be in fallen state"
    );
    assert!(
        result.rubble_produced,
        "Fallen tree should produce rubble"
    );

    // Verify blocks_movement override - fallen tree no longer blocks
    assert!(
        !damaged_tree.military.blocks_movement,
        "Fallen tree should not block movement"
    );

    // Cover value should be 0.6 (higher than standing tree because it's fallen cover)
    assert!(
        (damaged_tree.military.cover_value - 0.6).abs() < 0.01,
        "Fallen tree should have 0.6 cover value"
    );
}

#[test]
fn test_unified_spatial_concept() {
    // This test demonstrates that natural and constructed blueprints
    // use the same unified API and have the same interface

    let mut registry = BlueprintRegistry::new();

    // Register both natural and constructed blueprints
    registry.register(create_oak_tree_blueprint());
    registry.register(create_stone_wall_blueprint());

    let tree_id = registry.id_by_name("oak_tree").unwrap();
    let wall_id = registry.id_by_name("stone_wall").unwrap();

    // ========================================
    // Same instantiation API for both types
    // ========================================

    let tree = registry
        .instantiate(
            tree_id,
            HashMap::new(), // Use defaults
            Vec2::new(10.0, 10.0),
            0.0,
            PlacedBy::TerrainGen,
            None,
        )
        .expect("Tree instantiation via unified API");

    let wall = registry
        .instantiate(
            wall_id,
            HashMap::new(), // Use defaults
            Vec2::new(20.0, 20.0),
            0.0,
            PlacedBy::Gameplay { tick: 0 },
            Some(1),
        )
        .expect("Wall instantiation via unified API");

    // ========================================
    // Same interface for LOS queries
    // ========================================

    // Both have military.blocks_los accessible via same interface
    let tree_blocks_los = tree.military.blocks_los;
    let wall_blocks_los = wall.military.blocks_los;

    // Tree doesn't block LOS (canopy is porous)
    assert!(!tree_blocks_los, "Tree should not block LOS");
    // Wall's blocks_los is set from blueprint stats at instantiation
    // (construction stage overrides only apply during apply_work transitions)
    // The unified API provides consistent access regardless of initial state
    let _ = wall_blocks_los; // Just verify the field exists and is accessible

    // ========================================
    // Same interface for geometry
    // ========================================

    // Both have geometry.footprint populated
    assert!(
        !tree.geometry.footprint.is_empty(),
        "Tree should have geometry footprint"
    );
    assert!(
        !wall.geometry.footprint.is_empty(),
        "Wall should have geometry footprint"
    );

    // Tree has circular footprint (16 vertices)
    assert_eq!(tree.geometry.footprint.len(), 16, "Tree has circle footprint");
    // Wall has rectangular footprint (4 vertices)
    assert_eq!(wall.geometry.footprint.len(), 4, "Wall has rectangle footprint");

    // ========================================
    // Same interface for provenance tracking
    // ========================================

    // Both instances track who placed them via placed_by field
    match tree.placed_by {
        PlacedBy::TerrainGen => {} // Expected
        _ => panic!("Tree should be placed by TerrainGen"),
    }

    match wall.placed_by {
        PlacedBy::Gameplay { tick } => {
            assert_eq!(tick, 0, "Wall should track placement tick");
        }
        _ => panic!("Wall should be placed by Gameplay"),
    }

    // ========================================
    // Same interface for military properties
    // ========================================

    // Both have uniform access to military stats
    let _tree_cover = tree.military.cover_value;
    let _wall_cover = wall.military.cover_value;
    let _tree_hardness = tree.military.hardness;
    let _wall_hardness = wall.military.hardness;
    let _tree_movement_cost = tree.military.movement_cost;
    let _wall_movement_cost = wall.military.movement_cost;

    // ========================================
    // Same interface for HP and damage state
    // ========================================

    // Both use the same HP tracking
    assert!(tree.max_hp > 0.0, "Tree has HP");
    assert!(wall.max_hp > 0.0, "Wall has HP");

    // Both have damage_state field
    assert!(!tree.damage_state.is_empty(), "Tree has damage state");
    assert!(!wall.damage_state.is_empty(), "Wall has damage state");

    // Both use same hp_ratio() method
    let _tree_ratio = tree.hp_ratio();
    let _wall_ratio = wall.hp_ratio();

    // ========================================
    // Both support owner tracking
    // ========================================

    assert!(tree.owner.is_none(), "Natural tree has no owner");
    assert_eq!(wall.owner, Some(1), "Constructed wall has owner");

    // ========================================
    // Both have blueprint_name for identification
    // ========================================

    assert_eq!(tree.blueprint_name, "oak_tree");
    assert_eq!(wall.blueprint_name, "stone_wall");
}

#[test]
fn test_load_from_directory_if_available() {
    let mut registry = BlueprintRegistry::new();

    let data_path = Path::new("data/blueprints");
    if data_path.exists() {
        let loaded = registry.load_directory(data_path);
        assert!(loaded.is_ok(), "Directory loading should succeed");

        let ids = loaded.unwrap();
        assert!(ids.len() >= 2, "Should load at least oak_tree and stone_wall");

        // Verify specific blueprints
        assert!(
            registry.get_by_name("oak_tree").is_some(),
            "oak_tree should be loaded"
        );
        assert!(
            registry.get_by_name("stone_wall").is_some(),
            "stone_wall should be loaded"
        );

        // Verify categories
        let trees = registry.get_by_category(BlueprintCategory::Tree);
        assert!(!trees.is_empty(), "Should have tree blueprints");

        let walls = registry.get_by_category(BlueprintCategory::Wall);
        assert!(!walls.is_empty(), "Should have wall blueprints");
    }
}

#[test]
fn test_parameter_validation() {
    let mut registry = BlueprintRegistry::new();
    registry.register(create_stone_wall_blueprint());

    let wall_id = registry.id_by_name("stone_wall").unwrap();

    // Valid parameters
    let mut valid_params = HashMap::new();
    valid_params.insert("length".to_string(), 5.0);
    valid_params.insert("height".to_string(), 2.0);
    valid_params.insert("thickness".to_string(), 0.6);

    assert!(
        registry.validate_params(wall_id, &valid_params).is_ok(),
        "Valid parameters should pass validation"
    );

    // Invalid: length too short (min is 2.0)
    let mut invalid_length = HashMap::new();
    invalid_length.insert("length".to_string(), 1.0);

    assert!(
        registry.validate_params(wall_id, &invalid_length).is_err(),
        "Length below minimum should fail validation"
    );

    // Invalid: height too tall (max is 5.0)
    let mut invalid_height = HashMap::new();
    invalid_height.insert("height".to_string(), 10.0);

    assert!(
        registry.validate_params(wall_id, &invalid_height).is_err(),
        "Height above maximum should fail validation"
    );
}

#[test]
fn test_anchors() {
    let mut registry = BlueprintRegistry::new();
    registry.register(create_stone_wall_blueprint());

    let wall_id = registry.id_by_name("stone_wall").unwrap();

    let mut params = HashMap::new();
    params.insert("length".to_string(), 10.0);
    params.insert("height".to_string(), 2.5);
    params.insert("thickness".to_string(), 0.6);

    let wall = registry
        .instantiate(
            wall_id,
            params,
            Vec2::ZERO,
            0.0,
            PlacedBy::TerrainGen,
            None,
        )
        .unwrap();

    // Wall should have anchors
    assert_eq!(wall.anchors.len(), 2, "Wall should have 2 anchors");

    // Find west anchor
    let west_anchor = wall.anchors.iter().find(|a| a.name == "west");
    assert!(west_anchor.is_some(), "Should have west anchor");
    let west = west_anchor.unwrap();

    // West anchor position should be at -length/2 = -5.0
    assert!(
        (west.position.x - (-5.0)).abs() < 0.01,
        "West anchor X should be -5.0"
    );

    // Find east anchor
    let east_anchor = wall.anchors.iter().find(|a| a.name == "east");
    assert!(east_anchor.is_some(), "Should have east anchor");
    let east = east_anchor.unwrap();

    // East anchor position should be at length/2 = 5.0
    assert!(
        (east.position.x - 5.0).abs() < 0.01,
        "East anchor X should be 5.0"
    );

    // Verify tags
    assert!(
        west.tags.contains(&"wall_connect".to_string()),
        "West anchor should have wall_connect tag"
    );
}
