//! Test loading generated world placements

use arc_citadel::blueprints::BlueprintRegistry;
use arc_citadel::world::PlacementLoader;
use std::path::Path;

fn main() {
    println!("Loading blueprints...");

    let mut registry = BlueprintRegistry::new();
    let blueprint_path = Path::new("data/blueprints");

    if !blueprint_path.exists() {
        eprintln!("Blueprint directory not found: {:?}", blueprint_path);
        std::process::exit(1);
    }

    match registry.load_directory(blueprint_path) {
        Ok(ids) => println!("Loaded {} blueprints", ids.len()),
        Err(e) => {
            eprintln!("Failed to load blueprints: {}", e);
            std::process::exit(1);
        }
    }

    // Check some specific blueprints we expect
    let expected = [
        "oak_tree",
        "pine_tree",
        "stone_wall",
        "wooden_house",
        "watchtower",
        "shrine",
        "well",
        "boulder",
        "rock_outcrop",
    ];
    println!("\nChecking expected blueprints:");
    for name in &expected {
        if registry.id_by_name(name).is_some() {
            println!("  [OK] {}", name);
        } else {
            println!("  [MISSING] {}", name);
        }
    }

    println!("\nLoading world placements...");
    let placements_path = Path::new("worldgen/placements.json");

    if !placements_path.exists() {
        eprintln!("Placements file not found: {:?}", placements_path);
        std::process::exit(1);
    }

    let content = std::fs::read_to_string(placements_path).expect("Failed to read placements file");

    let loader = PlacementLoader::new(&registry);
    match loader.load_from_json(&content) {
        Ok(objects) => {
            println!("Successfully loaded {} world objects!", objects.len());

            // Show some stats
            let mut by_blueprint: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            let mut natural_count = 0;
            let mut constructed_count = 0;
            let mut damaged_count = 0;

            for obj in objects.iter() {
                *by_blueprint.entry(obj.blueprint_name.clone()).or_insert(0) += 1;

                match &obj.placed_by {
                    arc_citadel::blueprints::PlacedBy::TerrainGen => natural_count += 1,
                    arc_citadel::blueprints::PlacedBy::HistorySim { .. } => constructed_count += 1,
                    arc_citadel::blueprints::PlacedBy::Gameplay { .. } => {}
                }

                if obj.damage_state != "intact" && obj.damage_state != "healthy" {
                    damaged_count += 1;
                }
            }

            println!("\nObjects by blueprint:");
            let mut sorted: Vec<_> = by_blueprint.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));
            for (name, count) in sorted {
                println!("  {} - {}", name, count);
            }

            println!("\nOrigin breakdown:");
            println!("  Natural (TerrainGen): {}", natural_count);
            println!("  Historical (HistorySim): {}", constructed_count);
            println!("  Damaged objects: {}", damaged_count);
        }
        Err(e) => {
            eprintln!("Failed to load placements: {}", e);
            std::process::exit(1);
        }
    }

    println!("\nWorld loading test passed!");
}
