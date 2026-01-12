//! CLI tool for running LLM geometry generation tests
//!
//! Usage:
//!   cargo run --bin geometry_test -- --model deepseek --output results.json

use arc_citadel::llm::client::LlmClient;
use arc_citadel::spatial::geometry_schema::*;
use arc_citadel::spatial::validation::{CompositeValidator, ValidationReport};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct Args {
    model: String,
    output: PathBuf,
    prompt_file: Option<PathBuf>,
}

fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    let mut model = "deepseek".to_string();
    let mut output = PathBuf::from("geometry_test_results.json");
    let mut prompt_file = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--model" | "-m" => {
                i += 1;
                if i < args.len() {
                    model = args[i].clone();
                }
            }
            "--output" | "-o" => {
                i += 1;
                if i < args.len() {
                    output = PathBuf::from(&args[i]);
                }
            }
            "--prompt" | "-p" => {
                i += 1;
                if i < args.len() {
                    prompt_file = Some(PathBuf::from(&args[i]));
                }
            }
            _ => {}
        }
        i += 1;
    }

    Args {
        model,
        output,
        prompt_file,
    }
}

const DEFAULT_PROMPT: &str = r#"Generate geometry components for a tactical strategy game. Output valid JSON matching the schemas exactly.

Generate:
- 10 wall_segment variants
- 10 archer_tower variants
- 10 trench_segment variants
- 10 gate variants
- 10 street_segment variants
- 5 hex layouts each for: dwarven_forge, human_tavern, elven_glade, defensive_outpost, forest_clearing

Requirements:
- All coordinates in meters
- Polygon vertices counter-clockwise
- Tower firing arcs must sum to 360°
- Street cavalry_charge_viable only if width >= 6m
- All positions within hex bounds [0, 100]

Output a single JSON object with all components."#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();

    println!("=== LLM Geometry Generation Test ===");
    println!("Model: {}", args.model);
    println!("Output: {}", args.output.display());

    // Load prompt
    let prompt = if let Some(path) = &args.prompt_file {
        fs::read_to_string(path)?
    } else {
        DEFAULT_PROMPT.to_string()
    };

    // Create LLM client
    let client = LlmClient::from_env()?;

    println!("\nSending generation request to LLM...");
    let response = client
        .complete(
            "You are a geometry generator for a tactical strategy game. Output only valid JSON.",
            &prompt,
        )
        .await?;

    println!("Received response ({} chars)", response.len());

    // Extract JSON from response
    let json_start = response.find('{').unwrap_or(0);
    let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
    let json_str = &response[json_start..json_end];

    // Parse response
    println!("\nParsing response...");
    let result: GeometryTestResult = match serde_json::from_str(json_str) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to parse JSON: {}", e);
            eprintln!("Raw response:\n{}", &response[..response.len().min(2000)]);
            return Err(e.into());
        }
    };

    // Validate all components
    println!("\nValidating components...");
    let mut total = 0u32;
    let mut passed_geometric = 0u32;
    let mut passed_tactical = 0u32;
    let mut passed_connection = 0u32;
    let mut passed_physical = 0u32;
    let mut passed_civilian = 0u32;
    let mut failed: Vec<FailedComponent> = Vec::new();

    // Helper to process validation report
    let mut process_report = |id: &str, report: ValidationReport| {
        total += 1;
        if report.passed_geometric {
            passed_geometric += 1;
        }
        if report.passed_tactical {
            passed_tactical += 1;
        }
        if report.passed_connection {
            passed_connection += 1;
        }
        if report.passed_physical {
            passed_physical += 1;
        }
        if report.passed_civilian {
            passed_civilian += 1;
        }
        if !report.is_valid {
            failed.push(FailedComponent {
                id: id.to_string(),
                failure_reason: format!("{:?}", report.errors),
            });
        }
    };

    // Validate wall segments
    for wall in &result.wall_segments {
        let report = CompositeValidator::validate_component(&Component::WallSegment(wall.clone()));
        process_report(&wall.variant_id, report);
    }

    // Validate archer towers
    for tower in &result.archer_towers {
        let report = CompositeValidator::validate_component(&Component::ArcherTower(tower.clone()));
        process_report(&tower.variant_id, report);
    }

    // Validate trenches
    for trench in &result.trenches {
        let report =
            CompositeValidator::validate_component(&Component::TrenchSegment(trench.clone()));
        process_report(&trench.variant_id, report);
    }

    // Validate gates
    for gate in &result.gates {
        let report = CompositeValidator::validate_component(&Component::Gate(gate.clone()));
        process_report(&gate.variant_id, report);
    }

    // Validate streets
    for street in &result.street_segments {
        let report =
            CompositeValidator::validate_component(&Component::StreetSegment(street.clone()));
        process_report(&street.variant_id, report);
    }

    // Validate hex layouts
    for layout in &result.hex_layouts.dwarven_forge {
        let report = CompositeValidator::validate_hex_layout(layout);
        process_report(&layout.variant_id, report);
    }
    for layout in &result.hex_layouts.human_tavern {
        let report = CompositeValidator::validate_hex_layout(layout);
        process_report(&layout.variant_id, report);
    }
    for layout in &result.hex_layouts.elven_glade {
        let report = CompositeValidator::validate_hex_layout(layout);
        process_report(&layout.variant_id, report);
    }
    for layout in &result.hex_layouts.defensive_outpost {
        let report = CompositeValidator::validate_hex_layout(layout);
        process_report(&layout.variant_id, report);
    }
    for layout in &result.hex_layouts.forest_clearing {
        let report = CompositeValidator::validate_hex_layout(layout);
        process_report(&layout.variant_id, report);
    }

    // Build final results
    let final_results = ValidationResults {
        total_components: total,
        passed_geometric,
        passed_tactical,
        passed_connection,
        passed_physical,
        passed_civilian,
        failed_components: failed,
    };

    // Print summary
    println!("\n=== Validation Results ===");
    println!("Total components: {}", total);
    println!(
        "Passed geometric: {} ({:.1}%)",
        passed_geometric,
        100.0 * passed_geometric as f64 / total as f64
    );
    println!(
        "Passed tactical:  {} ({:.1}%)",
        passed_tactical,
        100.0 * passed_tactical as f64 / total as f64
    );
    println!(
        "Passed connection: {} ({:.1}%)",
        passed_connection,
        100.0 * passed_connection as f64 / total as f64
    );
    println!(
        "Passed physical:  {} ({:.1}%)",
        passed_physical,
        100.0 * passed_physical as f64 / total as f64
    );
    println!(
        "Passed civilian:  {} ({:.1}%)",
        passed_civilian,
        100.0 * passed_civilian as f64 / total as f64
    );

    let _all_passed = final_results.failed_components.is_empty();
    let pass_rate = if total > 0 {
        100.0 * (total - final_results.failed_components.len() as u32) as f64 / total as f64
    } else {
        0.0
    };

    println!("\nOverall pass rate: {:.1}%", pass_rate);
    if pass_rate >= 70.0 {
        println!("RESULT: LLM geometry generation is VIABLE");
    } else if pass_rate >= 40.0 {
        println!("RESULT: Need validation + correction loop");
    } else {
        println!("RESULT: Fall back to procedural generation");
    }

    // Save results
    let output_data = serde_json::json!({
        "test_run_id": result.test_run_id,
        "model": result.model,
        "timestamp": result.timestamp,
        "validation_results": final_results,
        "raw_components": {
            "wall_segments": result.wall_segments.len(),
            "archer_towers": result.archer_towers.len(),
            "trenches": result.trenches.len(),
            "gates": result.gates.len(),
            "street_segments": result.street_segments.len(),
            "hex_layouts": {
                "dwarven_forge": result.hex_layouts.dwarven_forge.len(),
                "human_tavern": result.hex_layouts.human_tavern.len(),
                "elven_glade": result.hex_layouts.elven_glade.len(),
                "defensive_outpost": result.hex_layouts.defensive_outpost.len(),
                "forest_clearing": result.hex_layouts.forest_clearing.len(),
            }
        }
    });

    fs::write(&args.output, serde_json::to_string_pretty(&output_data)?)?;
    println!("\nResults saved to: {}", args.output.display());

    Ok(())
}
