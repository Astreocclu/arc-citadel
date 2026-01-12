//! Two-stage geometry generation test
//!
//! Stage 1: Reasoner generates component skeletons with descriptions
//! Stage 2: Chat fills in actual geometry in batches

use arc_citadel::llm::client::LlmClient;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct ComponentSkeleton {
    variant_id: String,
    display_name: String,
    description: String,
    intended_purpose: String,
    geometry_placeholder: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SkeletonResponse {
    #[serde(default)]
    wall_segments: Vec<ComponentSkeleton>,
    #[serde(default)]
    archer_towers: Vec<ComponentSkeleton>,
    #[serde(default)]
    trenches: Vec<ComponentSkeleton>,
    #[serde(default)]
    gates: Vec<ComponentSkeleton>,
    #[serde(default)]
    street_segments: Vec<ComponentSkeleton>,
}

fn get_schema_snippet(component_type: &str) -> &'static str {
    match component_type {
        "wall_segment" => {
            r#"
## REQUIRED SCHEMA FOR wall_segment:
```json
{
  "component_type": "wall_segment",
  "variant_id": "string",
  "display_name": "string",
  "dimensions": {
    "length": 1.0-10.0,
    "height": 1.5-10.0,
    "thickness": 0.3-2.0
  },
  "footprint": {
    "shape": "rectangle",
    "vertices": [[x,y], ...],  // counter-clockwise, closed polygon
    "origin": "center_base"
  },
  "properties": {
    "blocks_movement": true,
    "blocks_los": true,
    "provides_cover": "full|partial|none",
    "cover_direction": "perpendicular_to_length",
    "destructible": true,
    "hp": 100-1000,
    "material": "stone|wood|earth"
  },
  "connection_points": [
    {"id": "west", "position": [x,y], "direction": "west|east|north|south", "compatible_with": ["wall_segment","gate"]}
  ]
}
```"#
        }
        "archer_tower" => {
            r#"
## REQUIRED SCHEMA FOR archer_tower:
```json
{
  "component_type": "archer_tower",
  "variant_id": "string",
  "display_name": "string",
  "dimensions": {
    "base_width": 4.0-12.0,
    "base_length": 4.0-12.0,
    "height": 8.0-20.0
  },
  "footprint": {
    "shape": "rectangle|circle|polygon",
    "vertices": [[x,y], ...],  // counter-clockwise, closed polygon
    "origin": "center_base"
  },
  "firing_arcs": [
    {"start_angle": 0, "end_angle": 90},
    {"start_angle": 270, "end_angle": 360}
  ],
  "range": 20-50,
  "archer_capacity": 2-12,
  "properties": {
    "blocks_movement": true,
    "blocks_los": true,
    "provides_cover": "full",
    "destructible": true,
    "hp": 500-2000,
    "material": "stone|wood"
  },
  "connection_points": [
    {"id": "north", "position": [x,y], "direction": "north", "compatible_with": ["wall_segment"]}
  ]
}
```
IMPORTANT: firing_arcs is REQUIRED. Angles are 0-360 degrees, 0=East, 90=North, counter-clockwise."#
        }
        "trench_segment" => {
            r#"
## REQUIRED SCHEMA FOR trench_segment:
```json
{
  "component_type": "trench_segment",
  "variant_id": "string",
  "display_name": "string",
  "dimensions": {
    "length": 3.0-15.0,
    "width": 0.8-4.0,
    "depth": 0.5-3.0
  },
  "footprint": {
    "shape": "rectangle|polygon",
    "vertices": [[x,y], ...],
    "origin": "center_base"
  },
  "properties": {
    "blocks_movement": false,
    "blocks_los": false,
    "provides_cover": "full|partial",
    "cover_direction": "from_above",
    "has_firing_step": true|false,
    "movement_penalty": 0.3-0.7,
    "water_filled": true|false
  },
  "connection_points": [
    {"id": "end_a", "position": [x,y], "direction": "west", "compatible_with": ["trench_segment"]}
  ]
}
```"#
        }
        "gate" => {
            r#"
## REQUIRED SCHEMA FOR gate:
```json
{
  "component_type": "gate",
  "variant_id": "string",
  "display_name": "string",
  "dimensions": {
    "width": 2.5-8.0,
    "height": 3.0-8.0,
    "thickness": 0.3-1.0
  },
  "footprint": {
    "shape": "rectangle",
    "vertices": [[x,y], ...],
    "origin": "center_base"
  },
  "passable_by": ["infantry", "cavalry", "cart", "siege_engine"],
  "properties": {
    "blocks_movement": true,
    "blocks_los": true,
    "provides_cover": "full",
    "destructible": true,
    "hp": 200-1500,
    "material": "wood|iron|stone",
    "door_type": "single|double|portcullis|drawbridge",
    "open_state": "closed"
  },
  "connection_points": [
    {"id": "west", "position": [x,y], "direction": "west", "compatible_with": ["wall_segment"]}
  ]
}
```
IMPORTANT: passable_by is REQUIRED. Use: infantry (width>=2.5m), cavalry (width>=6m), cart (width>=4m), siege_engine (width>=8m)"#
        }
        "street_segment" => {
            r#"
## REQUIRED SCHEMA FOR street_segment:
```json
{
  "component_type": "street_segment",
  "variant_id": "string",
  "display_name": "string",
  "dimensions": {
    "length": 5.0-50.0,
    "width": 2.0-10.0
  },
  "footprint": {
    "shape": "rectangle|polygon",
    "vertices": [[x,y], ...],
    "origin": "center_base"
  },
  "passable_by": ["infantry", "cavalry", "cart"],
  "traffic_capacity": 10-200,
  "properties": {
    "blocks_movement": false,
    "blocks_los": false,
    "surface_type": "dirt|cobblestone|paved|gravel",
    "movement_bonus": 1.0-1.3
  },
  "connection_points": [
    {"id": "end_a", "position": [x,y], "direction": "west", "compatible_with": ["street_segment","gate"]}
  ]
}
```
IMPORTANT: passable_by and traffic_capacity are REQUIRED. Width determines passable_by: 2m=infantry, 4m+=cart, 6m+=cavalry"#
        }
        _ => "",
    }
}

fn extract_json(response: &str) -> Option<&str> {
    // Find JSON block in markdown code fence or raw JSON
    if let Some(start) = response.find("```json") {
        let json_start = start + 7;
        if let Some(end) = response[json_start..].find("```") {
            return Some(&response[json_start..json_start + end]);
        }
    }
    // Try finding raw JSON object
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            return Some(&response[start..=end]);
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Two-Stage Geometry Generation ===\n");

    // Load prompts
    let skeleton_prompt = fs::read_to_string("data/prompts/geometry_skeleton.txt")?;
    let details_prompt = fs::read_to_string("data/prompts/geometry_details.txt")?;

    // Get API key
    let api_key = std::env::var("LLM_API_KEY").expect("LLM_API_KEY required");
    let api_url = std::env::var("LLM_API_URL")
        .unwrap_or_else(|_| "https://api.deepseek.com/v1/chat/completions".into());

    // Stage 1: Reasoner generates skeleton
    println!("=== STAGE 1: Skeleton Generation (Reasoner) ===");
    let reasoner = LlmClient::new(api_key.clone(), api_url.clone(), "deepseek-reasoner".into());

    println!("Sending skeleton request to reasoner...");
    let skeleton_response = reasoner
        .complete(
            &skeleton_prompt,
            "Generate the geometry component skeleton now.",
        )
        .await?;

    println!(
        "Received skeleton response ({} chars)",
        skeleton_response.len()
    );

    // Save raw skeleton response
    fs::write("skeleton_raw.txt", &skeleton_response)?;
    println!("Saved raw response to skeleton_raw.txt");

    // Parse skeleton
    let json_str = extract_json(&skeleton_response).ok_or("No JSON found in skeleton response")?;

    let skeleton: SkeletonResponse = match serde_json::from_str(json_str) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to parse skeleton: {}", e);
            println!("JSON excerpt: {}...", &json_str[..json_str.len().min(500)]);
            return Err(e.into());
        }
    };

    println!("\nSkeleton parsed:");
    println!("  Wall segments: {}", skeleton.wall_segments.len());
    println!("  Archer towers: {}", skeleton.archer_towers.len());
    println!("  Trenches: {}", skeleton.trenches.len());
    println!("  Gates: {}", skeleton.gates.len());
    println!("  Street segments: {}", skeleton.street_segments.len());

    // Stage 2: Chat fills in details
    println!("\n=== STAGE 2: Detail Generation (Chat) ===");
    let chat = LlmClient::new(api_key, api_url, "deepseek-chat".into());

    let mut all_components: Vec<serde_json::Value> = Vec::new();

    // Process each component type in batches
    let component_batches = vec![
        ("wall_segment", &skeleton.wall_segments),
        ("archer_tower", &skeleton.archer_towers),
        ("trench_segment", &skeleton.trenches),
        ("gate", &skeleton.gates),
        ("street_segment", &skeleton.street_segments),
    ];

    for (component_type, skeletons) in component_batches {
        if skeletons.is_empty() {
            println!("Skipping {} (no skeletons)", component_type);
            continue;
        }

        println!(
            "\nProcessing {} ({} components)...",
            component_type,
            skeletons.len()
        );

        // Component-specific schema snippets
        let schema_snippet = get_schema_snippet(component_type);

        // Create batch request
        let batch_input = serde_json::json!({
            "component_type": component_type,
            "skeletons": skeletons
        });

        let user_message = format!(
            "{}\n\nFill in geometry for these {} components:\n\n{}",
            schema_snippet,
            component_type,
            serde_json::to_string_pretty(&batch_input)?
        );

        match chat.complete(&details_prompt, &user_message).await {
            Ok(response) => {
                println!("  Received response ({} chars)", response.len());

                if let Some(json_str) = extract_json(&response) {
                    match serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
                        Ok(components) => {
                            println!("  Parsed {} components", components.len());
                            all_components.extend(components);
                        }
                        Err(e) => {
                            // Try parsing as single object
                            match serde_json::from_str::<serde_json::Value>(json_str) {
                                Ok(v) => {
                                    if let Some(arr) = v.as_array() {
                                        println!("  Parsed {} components (nested)", arr.len());
                                        all_components.extend(arr.clone());
                                    } else {
                                        println!("  Parsed 1 component");
                                        all_components.push(v);
                                    }
                                }
                                Err(_) => println!("  Parse error: {}", e),
                            }
                        }
                    }
                } else {
                    println!("  No JSON found in response");
                }
            }
            Err(e) => {
                println!("  API error: {}", e);
            }
        }
    }

    // Save final results
    let output = serde_json::json!({
        "stage1_skeleton": skeleton,
        "stage2_components": all_components,
        "total_components": all_components.len()
    });

    fs::write(
        "twostage_results.json",
        serde_json::to_string_pretty(&output)?,
    )?;
    println!("\n=== Results ===");
    println!("Total components generated: {}", all_components.len());
    println!("Saved to twostage_results.json");

    Ok(())
}
