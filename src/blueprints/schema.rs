//! Blueprint schema types for TOML deserialization.
//!
//! This module defines the data structures used to load blueprints from TOML files.
//! Blueprints define parameterized game entities (walls, buildings, trees, etc.)
//! with computed properties based on expressions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a blueprint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlueprintId(pub u32);

/// How the entity comes into existence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OriginType {
    /// Spawns instantly (trees, rocks, natural features)
    Natural,
    /// Requires construction process (walls, buildings)
    Constructed,
}

impl Default for OriginType {
    fn default() -> Self {
        OriginType::Constructed
    }
}

/// Category of the blueprint for organization and filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlueprintCategory {
    // Constructed categories
    Wall,
    Tower,
    Gate,
    Trench,
    Street,
    Building,
    Furniture,
    // Natural categories
    Tree,
    Rock,
    Water,
    Vegetation,
    Terrain,
}

/// Type definition for a blueprint parameter
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ParameterType {
    /// Floating-point parameter with range constraints
    Float { min: f32, max: f32, default: f32 },
    /// Integer parameter with range constraints
    Int { min: i32, max: i32, default: i32 },
    /// Boolean parameter
    Bool { default: bool },
}

/// Complete blueprint definition
#[derive(Debug, Clone, Deserialize)]
pub struct Blueprint {
    /// Metadata about the blueprint
    pub meta: BlueprintMeta,
    /// Parameter definitions (name -> type)
    pub parameters: HashMap<String, ParameterType>,
    /// Geometry formula for computing dimensions
    pub geometry: GeometryFormula,
    /// Combat and civilian stats
    #[serde(default)]
    pub stats: Stats,
    /// Construction requirements (None for natural entities)
    #[serde(default)]
    pub construction: Option<ConstructionDef>,
    /// Connection/attachment points
    #[serde(default)]
    pub anchors: Vec<AnchorDef>,
    /// Damage state definitions
    #[serde(default)]
    pub damage_states: Vec<DamageStateDef>,
    /// Validation constraints
    #[serde(default)]
    pub constraints: Vec<ConstraintDef>,
}

/// Blueprint metadata
#[derive(Debug, Clone, Deserialize)]
pub struct BlueprintMeta {
    /// Unique string identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Category for organization
    pub category: BlueprintCategory,
    /// How the entity is created
    #[serde(default)]
    pub origin: OriginType,
    /// Optional description
    #[serde(default)]
    pub description: String,
}

/// Geometry formula for computing entity dimensions
#[derive(Debug, Clone, Deserialize)]
pub struct GeometryFormula {
    /// Width expression (X axis)
    pub width: String,
    /// Depth expression (Y axis)
    pub depth: String,
    /// Height expression (Z axis)
    pub height: String,
    /// Shape type (rectangle, circle, etc.)
    #[serde(default = "default_shape")]
    pub shape: String,
}

fn default_shape() -> String {
    "rectangle".to_string()
}

/// Combined stats for military and civilian use
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Stats {
    /// Military/combat stats
    #[serde(default)]
    pub military: MilitaryStats,
    /// Civilian/economic stats
    #[serde(default)]
    pub civilian: CivilianStats,
}

/// Military and combat statistics (expressions as strings)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MilitaryStats {
    /// Maximum hit points
    #[serde(default = "default_hp")]
    pub max_hp: String,
    /// Material hardness (damage resistance)
    #[serde(default)]
    pub hardness: String,
    /// Cover value for units behind this entity
    #[serde(default)]
    pub cover_value: String,
    /// Whether this blocks unit movement
    #[serde(default)]
    pub blocks_movement: String,
    /// Whether this blocks line of sight
    #[serde(default)]
    pub blocks_los: String,
    /// Movement cost modifier
    #[serde(default)]
    pub movement_cost: String,
    /// Whether this can catch fire
    #[serde(default)]
    pub flammable: String,
    /// Elevation bonus for ranged units
    #[serde(default)]
    pub elevation: String,
}

fn default_hp() -> String {
    "100".to_string()
}

/// Civilian and economic statistics (expressions as strings)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CivilianStats {
    /// Number of pedestrians that can use this
    #[serde(default)]
    pub pedestrian_capacity: String,
    /// Whether carts can pass through
    #[serde(default)]
    pub cart_accessible: String,
    /// Number of workers this can support
    #[serde(default)]
    pub worker_capacity: String,
    /// Storage capacity for goods
    #[serde(default)]
    pub storage_capacity: String,
    /// Prestige modifier for nearby buildings
    #[serde(default)]
    pub prestige_modifier: String,
}

/// Construction definition for built entities
#[derive(Debug, Clone, Deserialize)]
pub struct ConstructionDef {
    /// Base construction time expression
    pub base_time: String,
    /// Maximum workers that can contribute
    #[serde(default)]
    pub labor_cap: String,
    /// Resource costs (resource_name -> quantity expression)
    pub cost: HashMap<String, String>,
    /// Construction stages
    #[serde(default)]
    pub stages: Vec<ConstructionStageDef>,
}

/// A stage in the construction process
#[derive(Debug, Clone, Deserialize)]
pub struct ConstructionStageDef {
    /// Stage identifier
    pub id: String,
    /// Progress threshold (0.0-1.0) to enter this stage
    pub progress_threshold: f32,
    /// Height multiplier during this stage
    #[serde(default = "default_height_mult")]
    pub height_multiplier: f32,
    /// Visual state name for rendering
    pub visual_state: String,
    /// Property overrides during this stage
    #[serde(default)]
    pub overrides: PropertyOverrides,
}

fn default_height_mult() -> f32 {
    1.0
}

/// An anchor point for connections or attachments
#[derive(Debug, Clone, Deserialize)]
pub struct AnchorDef {
    /// Name of this anchor point
    pub name: String,
    /// Position as [x, y, z] expressions
    pub position: [String; 3],
    /// Direction expression (e.g., "north", "east", or angle)
    pub direction: String,
    /// Optional tags for filtering connections
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A damage state with visual and gameplay changes
#[derive(Debug, Clone, Deserialize)]
pub struct DamageStateDef {
    /// Name of this damage state
    pub name: String,
    /// HP threshold (0.0-1.0) to enter this state
    pub threshold: f32,
    /// Visual overlay identifier
    #[serde(default)]
    pub visual_overlay: String,
    /// Property overrides in this state
    #[serde(default)]
    pub overrides: PropertyOverrides,
    /// Tags applied in this state
    #[serde(default)]
    pub tags: Vec<String>,
    /// Whether this state creates a breach for pathfinding
    #[serde(default)]
    pub creates_breach: bool,
    /// Whether this state produces rubble
    #[serde(default)]
    pub produces_rubble: bool,
}

/// Property overrides for construction stages and damage states
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PropertyOverrides {
    /// Override cover value
    pub cover_value: Option<f32>,
    /// Override movement blocking
    pub blocks_movement: Option<bool>,
    /// Override LOS blocking
    pub blocks_los: Option<bool>,
    /// Override hardness
    pub hardness: Option<f32>,
}

/// A validation constraint for the blueprint
#[derive(Debug, Clone, Deserialize)]
pub struct ConstraintDef {
    /// Human-readable description of the constraint
    pub description: String,
    /// Expression that must evaluate to non-zero (true)
    pub expression: String,
    /// Error message when constraint is violated
    pub error_message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_stone_wall() {
        let toml_str = r#"
[meta]
id = "stone_wall"
name = "Stone Wall"
category = "wall"
origin = "constructed"

[parameters]
length = { type = "float", min = 2.0, max = 20.0, default = 5.0 }
height = { type = "float", min = 1.0, max = 5.0, default = 2.5 }

[geometry]
width = "length"
depth = "0.6"
height = "height"

[stats.military]
max_hp = "length * height * 80"
blocks_movement = "1"

[construction]
base_time = "100"

[construction.cost]
stone = "length * 10"

[[damage_states]]
name = "intact"
threshold = 0.75
"#;
        let blueprint: Blueprint = toml::from_str(toml_str).unwrap();
        assert_eq!(blueprint.meta.id, "stone_wall");
        assert_eq!(blueprint.meta.category, BlueprintCategory::Wall);
        assert_eq!(blueprint.meta.origin, OriginType::Constructed);
    }

    #[test]
    fn test_deserialize_natural() {
        let toml_str = r#"
[meta]
id = "oak_tree"
name = "Oak Tree"
category = "tree"
origin = "natural"

[parameters]
height = { type = "float", min = 8.0, max = 25.0, default = 15.0 }

[geometry]
width = "1.0"
depth = "1.0"
height = "height"
shape = "circle"

[stats.military]
max_hp = "height * 20"
flammable = "1"
"#;
        let blueprint: Blueprint = toml::from_str(toml_str).unwrap();
        assert_eq!(blueprint.meta.id, "oak_tree");
        assert_eq!(blueprint.meta.origin, OriginType::Natural);
        assert!(blueprint.construction.is_none());
    }

    #[test]
    fn test_deserialize_parameter_types() {
        let toml_str = r#"
[meta]
id = "test"
name = "Test"
category = "building"

[parameters]
float_param = { type = "float", min = 0.0, max = 10.0, default = 5.0 }
int_param = { type = "int", min = 1, max = 100, default = 50 }
bool_param = { type = "bool", default = true }

[geometry]
width = "1.0"
depth = "1.0"
height = "1.0"
"#;
        let blueprint: Blueprint = toml::from_str(toml_str).unwrap();
        assert_eq!(blueprint.parameters.len(), 3);

        match &blueprint.parameters["float_param"] {
            ParameterType::Float { min, max, default } => {
                assert_eq!(*min, 0.0);
                assert_eq!(*max, 10.0);
                assert_eq!(*default, 5.0);
            }
            _ => panic!("Expected Float parameter"),
        }

        match &blueprint.parameters["int_param"] {
            ParameterType::Int { min, max, default } => {
                assert_eq!(*min, 1);
                assert_eq!(*max, 100);
                assert_eq!(*default, 50);
            }
            _ => panic!("Expected Int parameter"),
        }

        match &blueprint.parameters["bool_param"] {
            ParameterType::Bool { default } => {
                assert!(*default);
            }
            _ => panic!("Expected Bool parameter"),
        }
    }

    #[test]
    fn test_deserialize_with_anchors() {
        let toml_str = r#"
[meta]
id = "gate"
name = "City Gate"
category = "gate"

[parameters]
width = { type = "float", min = 3.0, max = 8.0, default = 4.0 }

[geometry]
width = "width"
depth = "1.0"
height = "5.0"

[[anchors]]
name = "left_wall"
position = ["0", "0", "0"]
direction = "west"
tags = ["wall_connect"]

[[anchors]]
name = "right_wall"
position = ["width", "0", "0"]
direction = "east"
tags = ["wall_connect"]
"#;
        let blueprint: Blueprint = toml::from_str(toml_str).unwrap();
        assert_eq!(blueprint.anchors.len(), 2);
        assert_eq!(blueprint.anchors[0].name, "left_wall");
        assert_eq!(blueprint.anchors[0].direction, "west");
        assert_eq!(blueprint.anchors[0].tags, vec!["wall_connect"]);
        assert_eq!(blueprint.anchors[1].name, "right_wall");
    }

    #[test]
    fn test_deserialize_with_constraints() {
        let toml_str = r#"
[meta]
id = "narrow_passage"
name = "Narrow Passage"
category = "street"

[parameters]
width = { type = "float", min = 1.0, max = 5.0, default = 2.0 }
length = { type = "float", min = 2.0, max = 20.0, default = 5.0 }

[geometry]
width = "width"
depth = "length"
height = "0.1"

[[constraints]]
description = "Width must be less than length"
expression = "width < length"
error_message = "Passage width cannot exceed length"
"#;
        let blueprint: Blueprint = toml::from_str(toml_str).unwrap();
        assert_eq!(blueprint.constraints.len(), 1);
        assert_eq!(blueprint.constraints[0].expression, "width < length");
    }

    #[test]
    fn test_origin_type_default() {
        assert_eq!(OriginType::default(), OriginType::Constructed);
    }

    #[test]
    fn test_shape_default() {
        assert_eq!(default_shape(), "rectangle");
    }

    #[test]
    fn test_hp_default() {
        assert_eq!(default_hp(), "100");
    }

    #[test]
    fn test_height_mult_default() {
        assert_eq!(default_height_mult(), 1.0);
    }
}
