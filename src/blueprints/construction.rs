//! Construction system for blueprint instances.
//!
//! This module handles construction progress, material requirements, and stage
//! transitions for blueprint instances. Constructed entities progress through
//! stages as work is applied, with each stage potentially modifying properties
//! and visual appearance.

use std::collections::HashMap;

use super::expression::Expr;
use super::instance::BlueprintInstance;
use super::schema::{Blueprint, ConstructionStageDef};

/// Progress construction on an instance.
///
/// Applies work to the instance's construction progress and handles stage
/// transitions. Each stage can modify properties (via overrides) and geometry
/// (via height multiplier).
///
/// # Arguments
/// * `instance` - The blueprint instance being constructed
/// * `work_amount` - Amount of work to apply (in work units)
/// * `blueprint` - The blueprint definition containing construction info
///
/// # Returns
/// `true` if construction completed this tick, `false` otherwise
pub fn apply_work(
    instance: &mut BlueprintInstance,
    work_amount: f32,
    blueprint: &Blueprint,
) -> bool {
    if instance.is_complete() {
        return false;
    }

    let construction = match &blueprint.construction {
        Some(c) => c,
        None => {
            // No construction defined, instantly complete
            instance.construction_progress = 1.0;
            instance.construction_stage = None;
            return true;
        }
    };

    // Calculate total work required
    let total_work = match Expr::parse(&construction.base_time) {
        Ok(expr) => expr.evaluate(&instance.parameters).unwrap_or(100.0),
        Err(_) => 100.0,
    };

    // Apply work
    let progress_delta = work_amount / total_work;
    instance.construction_progress = (instance.construction_progress + progress_delta).min(1.0);

    // Find current stage
    let new_stage = find_construction_stage(&construction.stages, instance.construction_progress);
    if let Some(stage) = new_stage {
        if instance.construction_stage.as_ref() != Some(&stage.id) {
            instance.construction_stage = Some(stage.id.clone());
            // Apply stage property overrides
            instance.apply_overrides(&stage.overrides);
            // Scale geometry height
            instance.geometry.height *= stage.height_multiplier;
        }
    }

    instance.is_complete()
}

/// Get materials required for construction.
///
/// Evaluates the cost expressions from the blueprint definition using the
/// provided parameters to determine material quantities.
///
/// # Arguments
/// * `blueprint` - The blueprint definition containing construction costs
/// * `params` - Parameter values to use for expression evaluation
///
/// # Returns
/// A map of material names to required quantities (rounded up)
pub fn get_required_materials(
    blueprint: &Blueprint,
    params: &HashMap<String, f32>,
) -> HashMap<String, u32> {
    let mut materials = HashMap::new();

    if let Some(construction) = &blueprint.construction {
        for (material, expr_str) in &construction.cost {
            if let Ok(expr) = Expr::parse(expr_str) {
                if let Ok(amount) = expr.evaluate(params) {
                    materials.insert(material.clone(), amount.ceil() as u32);
                }
            }
        }
    }

    materials
}

/// Get labor cap (max workers) for construction.
///
/// Evaluates the labor_cap expression from the blueprint definition.
/// Returns 1 if no labor cap is defined or if evaluation fails.
///
/// # Arguments
/// * `blueprint` - The blueprint definition containing labor cap expression
/// * `params` - Parameter values to use for expression evaluation
///
/// # Returns
/// Maximum number of workers that can contribute simultaneously
pub fn get_labor_cap(blueprint: &Blueprint, params: &HashMap<String, f32>) -> u32 {
    if let Some(construction) = &blueprint.construction {
        if !construction.labor_cap.is_empty() {
            if let Ok(expr) = Expr::parse(&construction.labor_cap) {
                if let Ok(cap) = expr.evaluate(params) {
                    return cap.ceil() as u32;
                }
            }
        }
    }
    1 // Default to 1 worker
}

/// Find the appropriate construction stage for a given progress value.
///
/// Stages are sorted by progress_threshold ascending. Returns the highest
/// stage where progress >= threshold.
///
/// # Arguments
/// * `stages` - List of construction stage definitions from the blueprint
/// * `progress` - Current construction progress (0.0 to 1.0)
///
/// # Returns
/// The matching construction stage definition, or None if no stages defined
fn find_construction_stage<'a>(
    stages: &'a [ConstructionStageDef],
    progress: f32,
) -> Option<&'a ConstructionStageDef> {
    if stages.is_empty() {
        return None;
    }

    // Sort stages by progress_threshold ascending
    let mut sorted: Vec<&ConstructionStageDef> = stages.iter().collect();
    sorted.sort_by(|a, b| {
        a.progress_threshold
            .partial_cmp(&b.progress_threshold)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Find highest stage where progress >= threshold
    sorted
        .into_iter()
        .rev()
        .find(|s| progress >= s.progress_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprints::*;
    use glam::Vec2;

    fn create_test_blueprint() -> Blueprint {
        let toml_str = r#"
[meta]
id = "test_wall"
name = "Test Wall"
category = "wall"
origin = "constructed"

[parameters]
length = { type = "float", min = 1.0, max = 20.0, default = 5.0 }

[geometry]
width = "length"
depth = "0.5"
height = "3.0"

[stats.military]
max_hp = "100"

[construction]
base_time = "100"
labor_cap = "ceil(length / 2)"

[construction.cost]
stone = "length * 10"
wood = "length * 2"

[[construction.stages]]
id = "foundation"
progress_threshold = 0.0
height_multiplier = 0.2
visual_state = "foundation"
overrides = { blocks_movement = false }

[[construction.stages]]
id = "half"
progress_threshold = 0.5
height_multiplier = 0.6
visual_state = "half"
overrides = { blocks_movement = true }

[[construction.stages]]
id = "complete"
progress_threshold = 1.0
height_multiplier = 1.0
visual_state = "complete"
"#;
        toml::from_str(toml_str).unwrap()
    }

    #[test]
    fn test_get_required_materials() {
        let blueprint = create_test_blueprint();
        let mut params = HashMap::new();
        params.insert("length".to_string(), 10.0);

        let materials = get_required_materials(&blueprint, &params);

        assert_eq!(materials.get("stone"), Some(&100)); // 10 * 10
        assert_eq!(materials.get("wood"), Some(&20)); // 10 * 2
    }

    #[test]
    fn test_get_required_materials_empty() {
        // Blueprint with no construction
        let toml_str = r#"
[meta]
id = "natural_rock"
name = "Rock"
category = "rock"
origin = "natural"

[parameters]

[geometry]
width = "1.0"
depth = "1.0"
height = "1.0"

[stats.military]
max_hp = "50"
"#;
        let blueprint: Blueprint = toml::from_str(toml_str).unwrap();
        let params = HashMap::new();

        let materials = get_required_materials(&blueprint, &params);

        assert!(materials.is_empty());
    }

    #[test]
    fn test_get_labor_cap() {
        let blueprint = create_test_blueprint();
        let mut params = HashMap::new();
        params.insert("length".to_string(), 10.0);

        let cap = get_labor_cap(&blueprint, &params);
        assert_eq!(cap, 5); // ceil(10 / 2)
    }

    #[test]
    fn test_get_labor_cap_default() {
        // Blueprint with empty labor_cap
        let toml_str = r#"
[meta]
id = "simple_wall"
name = "Simple Wall"
category = "wall"
origin = "constructed"

[parameters]

[geometry]
width = "5.0"
depth = "0.5"
height = "3.0"

[stats.military]
max_hp = "100"

[construction]
base_time = "100"

[construction.cost]
stone = "50"
"#;
        let blueprint: Blueprint = toml::from_str(toml_str).unwrap();
        let params = HashMap::new();

        let cap = get_labor_cap(&blueprint, &params);
        assert_eq!(cap, 1); // Default
    }

    #[test]
    fn test_get_labor_cap_no_construction() {
        // Blueprint with no construction
        let toml_str = r#"
[meta]
id = "natural_rock"
name = "Rock"
category = "rock"
origin = "natural"

[parameters]

[geometry]
width = "1.0"
depth = "1.0"
height = "1.0"

[stats.military]
max_hp = "50"
"#;
        let blueprint: Blueprint = toml::from_str(toml_str).unwrap();
        let params = HashMap::new();

        let cap = get_labor_cap(&blueprint, &params);
        assert_eq!(cap, 1); // Default
    }

    #[test]
    fn test_apply_work() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint.clone());

        let mut params = HashMap::new();
        params.insert("length".to_string(), 5.0);

        let mut instance = registry
            .instantiate(
                id,
                params,
                Vec2::ZERO,
                0.0,
                PlacedBy::Gameplay { tick: 0 },
                Some(1),
            )
            .unwrap();

        // Start at 0 progress
        assert_eq!(instance.construction_progress, 0.0);
        assert_eq!(instance.construction_stage, Some("foundation".to_string()));
        assert!(!instance.is_complete());

        // Apply 50 work (base_time is 100)
        let completed = apply_work(&mut instance, 50.0, &blueprint);
        assert!(!completed);
        assert_eq!(instance.construction_progress, 0.5);
        assert_eq!(instance.construction_stage, Some("half".to_string()));

        // Apply remaining work
        let completed = apply_work(&mut instance, 50.0, &blueprint);
        assert!(completed);
        assert_eq!(instance.construction_progress, 1.0);
        assert!(instance.is_complete());
    }

    #[test]
    fn test_apply_work_already_complete() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint.clone());

        let mut params = HashMap::new();
        params.insert("length".to_string(), 5.0);

        let mut instance = registry
            .instantiate(
                id,
                params,
                Vec2::ZERO,
                0.0,
                PlacedBy::Gameplay { tick: 0 },
                Some(1),
            )
            .unwrap();

        // Force complete
        instance.construction_progress = 1.0;

        // Applying more work should do nothing
        let completed = apply_work(&mut instance, 100.0, &blueprint);
        assert!(!completed); // Returns false when already complete
        assert_eq!(instance.construction_progress, 1.0);
    }

    #[test]
    fn test_apply_work_no_construction() {
        // Blueprint with no construction defined
        let toml_str = r#"
[meta]
id = "natural_rock"
name = "Rock"
category = "rock"
origin = "natural"

[parameters]

[geometry]
width = "1.0"
depth = "1.0"
height = "1.0"

[stats.military]
max_hp = "50"
"#;
        let blueprint: Blueprint = toml::from_str(toml_str).unwrap();

        let mut registry = BlueprintRegistry::new();
        let id = registry.register(blueprint.clone());

        let mut instance = registry
            .instantiate(
                id,
                HashMap::new(),
                Vec2::ZERO,
                0.0,
                PlacedBy::TerrainGen,
                None,
            )
            .unwrap();

        // Force incomplete (even though Natural origin would normally be complete)
        instance.construction_progress = 0.5;

        // Should instantly complete
        let completed = apply_work(&mut instance, 1.0, &blueprint);
        assert!(completed);
        assert_eq!(instance.construction_progress, 1.0);
        assert!(instance.construction_stage.is_none());
    }

    #[test]
    fn test_apply_work_overwork() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint.clone());

        let mut params = HashMap::new();
        params.insert("length".to_string(), 5.0);

        let mut instance = registry
            .instantiate(
                id,
                params,
                Vec2::ZERO,
                0.0,
                PlacedBy::Gameplay { tick: 0 },
                Some(1),
            )
            .unwrap();

        // Apply more work than needed
        let completed = apply_work(&mut instance, 200.0, &blueprint);
        assert!(completed);
        assert_eq!(instance.construction_progress, 1.0); // Capped at 1.0
        assert!(instance.is_complete());
    }

    #[test]
    fn test_find_construction_stage() {
        let stages = vec![
            ConstructionStageDef {
                id: "foundation".to_string(),
                progress_threshold: 0.0,
                height_multiplier: 0.2,
                visual_state: "foundation".to_string(),
                overrides: Default::default(),
            },
            ConstructionStageDef {
                id: "half".to_string(),
                progress_threshold: 0.5,
                height_multiplier: 0.6,
                visual_state: "half".to_string(),
                overrides: Default::default(),
            },
            ConstructionStageDef {
                id: "complete".to_string(),
                progress_threshold: 1.0,
                height_multiplier: 1.0,
                visual_state: "complete".to_string(),
                overrides: Default::default(),
            },
        ];

        // At 0%
        let stage = find_construction_stage(&stages, 0.0);
        assert_eq!(stage.unwrap().id, "foundation");

        // At 25%
        let stage = find_construction_stage(&stages, 0.25);
        assert_eq!(stage.unwrap().id, "foundation");

        // At 50%
        let stage = find_construction_stage(&stages, 0.5);
        assert_eq!(stage.unwrap().id, "half");

        // At 75%
        let stage = find_construction_stage(&stages, 0.75);
        assert_eq!(stage.unwrap().id, "half");

        // At 100%
        let stage = find_construction_stage(&stages, 1.0);
        assert_eq!(stage.unwrap().id, "complete");
    }

    #[test]
    fn test_find_construction_stage_empty() {
        let stages: Vec<ConstructionStageDef> = vec![];
        let stage = find_construction_stage(&stages, 0.5);
        assert!(stage.is_none());
    }

    #[test]
    fn test_stage_overrides_applied() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint.clone());

        let mut params = HashMap::new();
        params.insert("length".to_string(), 5.0);

        let mut instance = registry
            .instantiate(
                id,
                params,
                Vec2::ZERO,
                0.0,
                PlacedBy::Gameplay { tick: 0 },
                Some(1),
            )
            .unwrap();

        // At foundation stage, blocks_movement should be false (from override)
        assert!(!instance.military.blocks_movement);

        // Progress to half stage
        apply_work(&mut instance, 50.0, &blueprint);

        // At half stage, blocks_movement should be true (from override)
        assert!(instance.military.blocks_movement);
    }

    #[test]
    fn test_materials_with_fractional_result() {
        let toml_str = r#"
[meta]
id = "fractional_wall"
name = "Fractional Wall"
category = "wall"
origin = "constructed"

[parameters]
length = { type = "float", min = 1.0, max = 20.0, default = 5.0 }

[geometry]
width = "length"
depth = "0.5"
height = "3.0"

[stats.military]
max_hp = "100"

[construction]
base_time = "100"

[construction.cost]
stone = "length * 3.3"
"#;
        let blueprint: Blueprint = toml::from_str(toml_str).unwrap();
        let mut params = HashMap::new();
        params.insert("length".to_string(), 3.0);

        let materials = get_required_materials(&blueprint, &params);

        // 3.0 * 3.3 = 9.9, ceil = 10
        assert_eq!(materials.get("stone"), Some(&10));
    }
}
