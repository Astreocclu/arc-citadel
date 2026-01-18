//! Blueprint registry for loading and instantiating blueprints.
//!
//! This module provides the `BlueprintRegistry` which manages blueprint definitions,
//! handles TOML file loading, and creates runtime instances with evaluated properties.

use glam::{Vec2, Vec3};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use super::expression::Expr;
use super::instance::{
    BlueprintInstance, CivilianProperties, EvaluatedGeometry, InstanceId, MilitaryProperties,
    PlacedBy, ResolvedAnchor,
};
use super::schema::{
    AnchorDef, Blueprint, BlueprintCategory, BlueprintId, OriginType, ParameterType,
};
use super::EvalError;

/// Error type for blueprint operations
#[derive(Debug)]
pub enum BlueprintError {
    /// File I/O error
    IoError(std::io::Error),
    /// TOML parsing error
    ParseError(String),
    /// Blueprint not found
    NotFound(String),
    /// Validation failed
    ValidationError(Vec<String>),
    /// Expression evaluation error
    ExpressionError(EvalError),
}

impl std::fmt::Display for BlueprintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlueprintError::IoError(e) => write!(f, "I/O error: {}", e),
            BlueprintError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            BlueprintError::NotFound(name) => write!(f, "Blueprint not found: {}", name),
            BlueprintError::ValidationError(errors) => {
                write!(f, "Validation errors: {}", errors.join(", "))
            }
            BlueprintError::ExpressionError(e) => write!(f, "Expression error: {}", e),
        }
    }
}

impl std::error::Error for BlueprintError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BlueprintError::IoError(e) => Some(e),
            BlueprintError::ExpressionError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for BlueprintError {
    fn from(e: std::io::Error) -> Self {
        BlueprintError::IoError(e)
    }
}

impl From<EvalError> for BlueprintError {
    fn from(e: EvalError) -> Self {
        BlueprintError::ExpressionError(e)
    }
}

/// Registry for managing blueprint definitions
pub struct BlueprintRegistry {
    /// Blueprints indexed by ID
    blueprints: HashMap<BlueprintId, Blueprint>,
    /// Map from name to ID for fast lookup
    by_name: HashMap<String, BlueprintId>,
    /// Map from category to IDs for filtering
    by_category: HashMap<BlueprintCategory, Vec<BlueprintId>>,
    /// Next blueprint ID to assign
    next_id: u32,
    /// Next instance ID (thread-safe counter)
    next_instance_id: AtomicU64,
}

impl BlueprintRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            blueprints: HashMap::new(),
            by_name: HashMap::new(),
            by_category: HashMap::new(),
            next_id: 1,
            next_instance_id: AtomicU64::new(1),
        }
    }

    /// Register a blueprint and return its assigned ID
    pub fn register(&mut self, blueprint: Blueprint) -> BlueprintId {
        let id = BlueprintId(self.next_id);
        self.next_id += 1;

        // Index by name
        self.by_name.insert(blueprint.meta.id.clone(), id);

        // Index by category
        self.by_category
            .entry(blueprint.meta.category)
            .or_default()
            .push(id);

        // Store the blueprint
        self.blueprints.insert(id, blueprint);

        id
    }

    /// Load a blueprint from a TOML file
    pub fn load_file(&mut self, path: &Path) -> Result<BlueprintId, BlueprintError> {
        let content = std::fs::read_to_string(path)?;
        let blueprint: Blueprint = toml::from_str(&content)
            .map_err(|e| BlueprintError::ParseError(format!("{}: {}", path.display(), e)))?;
        Ok(self.register(blueprint))
    }

    /// Load all .toml files from a directory recursively
    pub fn load_directory(&mut self, path: &Path) -> Result<Vec<BlueprintId>, BlueprintError> {
        let mut ids = Vec::new();
        self.load_directory_recursive(path, &mut ids)?;
        Ok(ids)
    }

    fn load_directory_recursive(
        &mut self,
        path: &Path,
        ids: &mut Vec<BlueprintId>,
    ) -> Result<(), BlueprintError> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                self.load_directory_recursive(&entry_path, ids)?;
            } else if entry_path.extension().map_or(false, |ext| ext == "toml") {
                let id = self.load_file(&entry_path)?;
                ids.push(id);
            }
        }
        Ok(())
    }

    /// Get a blueprint by ID
    pub fn get(&self, id: BlueprintId) -> Option<&Blueprint> {
        self.blueprints.get(&id)
    }

    /// Get a blueprint by name
    pub fn get_by_name(&self, name: &str) -> Option<&Blueprint> {
        self.by_name
            .get(name)
            .and_then(|id| self.blueprints.get(id))
    }

    /// Get blueprint ID by name
    pub fn id_by_name(&self, name: &str) -> Option<BlueprintId> {
        self.by_name.get(name).copied()
    }

    /// Get all blueprints in a category
    pub fn get_by_category(&self, category: BlueprintCategory) -> Vec<&Blueprint> {
        self.by_category
            .get(&category)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.blueprints.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Validate parameters against blueprint constraints
    pub fn validate_params(
        &self,
        blueprint_id: BlueprintId,
        params: &HashMap<String, f32>,
    ) -> Result<(), BlueprintError> {
        let blueprint = self
            .blueprints
            .get(&blueprint_id)
            .ok_or_else(|| BlueprintError::NotFound(format!("ID {:?}", blueprint_id)))?;

        let mut errors = Vec::new();

        // Check parameter ranges
        for (name, param_type) in &blueprint.parameters {
            if let Some(&value) = params.get(name) {
                match param_type {
                    ParameterType::Float { min, max, .. } => {
                        if value < *min || value > *max {
                            errors.push(format!(
                                "Parameter '{}' value {} is out of range [{}, {}]",
                                name, value, min, max
                            ));
                        }
                    }
                    ParameterType::Int { min, max, .. } => {
                        let int_val = value as i32;
                        if int_val < *min || int_val > *max {
                            errors.push(format!(
                                "Parameter '{}' value {} is out of range [{}, {}]",
                                name, int_val, min, max
                            ));
                        }
                    }
                    ParameterType::Bool { .. } => {
                        // Bool parameters are valid for any f32 (0.0 = false, non-zero = true)
                    }
                }
            }
        }

        // Check constraints
        let full_params = self.fill_defaults(blueprint, params);
        for constraint in &blueprint.constraints {
            match eval_expr_str(&constraint.expression, &full_params) {
                Ok(result) if result == 0.0 => {
                    errors.push(constraint.error_message.clone());
                }
                Err(e) => {
                    errors.push(format!(
                        "Constraint '{}' error: {}",
                        constraint.description, e
                    ));
                }
                _ => {}
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(BlueprintError::ValidationError(errors))
        }
    }

    /// Create an instance from a blueprint with the given parameters
    pub fn instantiate(
        &self,
        blueprint_id: BlueprintId,
        params: HashMap<String, f32>,
        position: Vec2,
        rotation: f32,
        placed_by: PlacedBy,
        owner: Option<u32>,
    ) -> Result<BlueprintInstance, BlueprintError> {
        let blueprint = self
            .blueprints
            .get(&blueprint_id)
            .ok_or_else(|| BlueprintError::NotFound(format!("ID {:?}", blueprint_id)))?;

        // Fill in defaults for missing params
        let full_params = self.fill_defaults(blueprint, &params);

        // Validate parameters
        self.validate_params(blueprint_id, &full_params)?;

        // Evaluate geometry
        let width = eval_expr_str(&blueprint.geometry.width, &full_params)?;
        let depth = eval_expr_str(&blueprint.geometry.depth, &full_params)?;
        let height = eval_expr_str(&blueprint.geometry.height, &full_params)?;
        let footprint = generate_footprint(&blueprint.geometry.shape, width, depth);

        let geometry = EvaluatedGeometry {
            width,
            depth,
            height,
            footprint,
        };

        // Evaluate military properties
        let max_hp = eval_expr_str_or(&blueprint.stats.military.max_hp, &full_params, 100.0)?;
        let hardness = eval_expr_str_or(&blueprint.stats.military.hardness, &full_params, 0.0)?;
        let cover_value =
            eval_expr_str_or(&blueprint.stats.military.cover_value, &full_params, 0.0)?;
        let blocks_movement =
            eval_expr_str_or(&blueprint.stats.military.blocks_movement, &full_params, 0.0)? != 0.0;
        let blocks_los =
            eval_expr_str_or(&blueprint.stats.military.blocks_los, &full_params, 0.0)? != 0.0;
        let movement_cost =
            eval_expr_str_or(&blueprint.stats.military.movement_cost, &full_params, 1.0)?;
        let flammable =
            eval_expr_str_or(&blueprint.stats.military.flammable, &full_params, 0.0)? != 0.0;
        let elevation = eval_expr_str_or(&blueprint.stats.military.elevation, &full_params, 0.0)?;

        let military = MilitaryProperties {
            max_hp,
            hardness,
            cover_value,
            blocks_movement,
            blocks_los,
            movement_cost,
            flammable,
            elevation,
        };

        // Evaluate civilian properties
        let pedestrian_capacity = eval_expr_str_or(
            &blueprint.stats.civilian.pedestrian_capacity,
            &full_params,
            0.0,
        )? as u32;
        let cart_accessible =
            eval_expr_str_or(&blueprint.stats.civilian.cart_accessible, &full_params, 0.0)? != 0.0;
        let worker_capacity =
            eval_expr_str_or(&blueprint.stats.civilian.worker_capacity, &full_params, 0.0)? as u32;
        let storage_capacity = eval_expr_str_or(
            &blueprint.stats.civilian.storage_capacity,
            &full_params,
            0.0,
        )? as u32;
        let prestige_modifier = eval_expr_str_or(
            &blueprint.stats.civilian.prestige_modifier,
            &full_params,
            0.0,
        )?;

        let civilian = CivilianProperties {
            pedestrian_capacity,
            cart_accessible,
            worker_capacity,
            storage_capacity,
            prestige_modifier,
            aesthetic_value: 0.0,  // TODO: Load from blueprint definition
            sacred_value: 0.0,     // TODO: Load from blueprint definition
        };

        // Resolve anchors
        let anchors: Result<Vec<ResolvedAnchor>, BlueprintError> = blueprint
            .anchors
            .iter()
            .map(|anchor| resolve_anchor(anchor, &full_params))
            .collect();
        let anchors = anchors?;

        // Set initial construction state
        let (construction_progress, construction_stage) = match blueprint.meta.origin {
            OriginType::Natural => (1.0, None),
            OriginType::Constructed => {
                if let Some(ref construction) = blueprint.construction {
                    if construction.stages.is_empty() {
                        (1.0, None)
                    } else {
                        (0.0, Some(construction.stages[0].id.clone()))
                    }
                } else {
                    (1.0, None)
                }
            }
        };

        // Set initial damage state
        let damage_state = if blueprint.damage_states.is_empty() {
            "intact".to_string()
        } else {
            blueprint.damage_states[0].name.clone()
        };

        // Generate unique instance ID
        let instance_id = InstanceId(self.next_instance_id.fetch_add(1, Ordering::SeqCst));

        // Set initial HP based on construction progress
        let current_hp = max_hp * construction_progress;

        Ok(BlueprintInstance {
            id: instance_id,
            blueprint_id,
            blueprint_name: blueprint.meta.id.clone(),
            parameters: full_params,
            position,
            rotation,
            geometry,
            current_hp,
            max_hp,
            damage_state,
            breaches: Vec::new(),
            military,
            civilian,
            anchors,
            construction_progress,
            construction_stage,
            placed_by,
            owner,
        })
    }

    /// Fill in default values for missing parameters
    fn fill_defaults(
        &self,
        blueprint: &Blueprint,
        params: &HashMap<String, f32>,
    ) -> HashMap<String, f32> {
        let mut full_params = params.clone();

        for (name, param_type) in &blueprint.parameters {
            if !full_params.contains_key(name) {
                let default = match param_type {
                    ParameterType::Float { default, .. } => *default,
                    ParameterType::Int { default, .. } => *default as f32,
                    ParameterType::Bool { default } => {
                        if *default {
                            1.0
                        } else {
                            0.0
                        }
                    }
                };
                full_params.insert(name.clone(), default);
            }
        }

        full_params
    }
}

impl Default for BlueprintRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse and evaluate an expression string
pub fn eval_expr_str(expr_str: &str, params: &HashMap<String, f32>) -> Result<f32, BlueprintError> {
    let trimmed = expr_str.trim();
    if trimmed.is_empty() {
        return Err(BlueprintError::ParseError("Empty expression".to_string()));
    }

    let expr = Expr::parse(trimmed).map_err(|e| BlueprintError::ParseError(e.to_string()))?;
    expr.evaluate(params).map_err(BlueprintError::from)
}

/// Parse and evaluate an expression string, returning default for empty strings
pub fn eval_expr_str_or(
    expr_str: &str,
    params: &HashMap<String, f32>,
    default: f32,
) -> Result<f32, BlueprintError> {
    let trimmed = expr_str.trim();
    if trimmed.is_empty() {
        return Ok(default);
    }

    eval_expr_str(trimmed, params)
}

/// Generate a footprint polygon for the given shape and dimensions
pub fn generate_footprint(shape: &str, width: f32, depth: f32) -> Vec<Vec2> {
    match shape.to_lowercase().as_str() {
        "circle" => {
            // Generate 16 vertices for a circle
            let radius_x = width / 2.0;
            let radius_y = depth / 2.0;
            (0..16)
                .map(|i| {
                    let angle = (i as f32) * std::f32::consts::TAU / 16.0;
                    Vec2::new(angle.cos() * radius_x, angle.sin() * radius_y)
                })
                .collect()
        }
        _ => {
            // Default to rectangle with 4 vertices (centered)
            let half_w = width / 2.0;
            let half_d = depth / 2.0;
            vec![
                Vec2::new(-half_w, -half_d),
                Vec2::new(half_w, -half_d),
                Vec2::new(half_w, half_d),
                Vec2::new(-half_w, half_d),
            ]
        }
    }
}

/// Resolve an anchor definition to a concrete anchor with evaluated positions
pub fn resolve_anchor(
    anchor: &AnchorDef,
    params: &HashMap<String, f32>,
) -> Result<ResolvedAnchor, BlueprintError> {
    let x = eval_expr_str(&anchor.position[0], params)?;
    let y = eval_expr_str(&anchor.position[1], params)?;
    let z = eval_expr_str(&anchor.position[2], params)?;

    // Parse direction - can be a cardinal direction or angle
    let direction = match anchor.direction.to_lowercase().as_str() {
        "north" => Vec3::new(0.0, 1.0, 0.0),
        "south" => Vec3::new(0.0, -1.0, 0.0),
        "east" => Vec3::new(1.0, 0.0, 0.0),
        "west" => Vec3::new(-1.0, 0.0, 0.0),
        "up" => Vec3::new(0.0, 0.0, 1.0),
        "down" => Vec3::new(0.0, 0.0, -1.0),
        _ => {
            // Try to parse as angle in degrees
            if let Ok(angle_deg) = anchor.direction.parse::<f32>() {
                let angle_rad = angle_deg.to_radians();
                Vec3::new(angle_rad.cos(), angle_rad.sin(), 0.0)
            } else {
                // Try evaluating as expression
                let angle_deg = eval_expr_str(&anchor.direction, params)?;
                let angle_rad = angle_deg.to_radians();
                Vec3::new(angle_rad.cos(), angle_rad.sin(), 0.0)
            }
        }
    };

    Ok(ResolvedAnchor {
        name: anchor.name.clone(),
        position: Vec3::new(x, y, z),
        direction,
        tags: anchor.tags.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprints::schema::{BlueprintMeta, GeometryFormula, Stats};

    fn create_test_blueprint() -> Blueprint {
        Blueprint {
            meta: BlueprintMeta {
                id: "test_wall".to_string(),
                name: "Test Wall".to_string(),
                category: BlueprintCategory::Wall,
                origin: OriginType::Constructed,
                description: "A test wall".to_string(),
            },
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "length".to_string(),
                    ParameterType::Float {
                        min: 2.0,
                        max: 20.0,
                        default: 5.0,
                    },
                );
                params.insert(
                    "height".to_string(),
                    ParameterType::Float {
                        min: 1.0,
                        max: 5.0,
                        default: 2.5,
                    },
                );
                params
            },
            geometry: GeometryFormula {
                width: "length".to_string(),
                depth: "0.6".to_string(),
                height: "height".to_string(),
                shape: "rectangle".to_string(),
            },
            stats: Stats {
                military: super::super::schema::MilitaryStats {
                    max_hp: "length * height * 80".to_string(),
                    hardness: "5".to_string(),
                    cover_value: "0.8".to_string(),
                    blocks_movement: "1".to_string(),
                    blocks_los: "1".to_string(),
                    movement_cost: "".to_string(),
                    flammable: "0".to_string(),
                    elevation: "".to_string(),
                },
                civilian: Default::default(),
            },
            construction: Some(super::super::schema::ConstructionDef {
                base_time: "100".to_string(),
                labor_cap: "4".to_string(),
                cost: {
                    let mut cost = HashMap::new();
                    cost.insert("stone".to_string(), "length * 10".to_string());
                    cost
                },
                stages: vec![super::super::schema::ConstructionStageDef {
                    id: "foundation".to_string(),
                    progress_threshold: 0.0,
                    height_multiplier: 0.3,
                    visual_state: "foundation".to_string(),
                    overrides: Default::default(),
                }],
            }),
            anchors: vec![],
            damage_states: vec![super::super::schema::DamageStateDef {
                name: "intact".to_string(),
                threshold: 0.75,
                visual_overlay: "".to_string(),
                overrides: Default::default(),
                tags: vec![],
                creates_breach: false,
                produces_rubble: false,
            }],
            constraints: vec![],
        }
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();

        let id = registry.register(blueprint);

        // Verify get works
        let retrieved = registry.get(id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().meta.id, "test_wall");

        // Verify get_by_name works
        let by_name = registry.get_by_name("test_wall");
        assert!(by_name.is_some());
        assert_eq!(by_name.unwrap().meta.id, "test_wall");

        // Verify id_by_name works
        let id_lookup = registry.id_by_name("test_wall");
        assert_eq!(id_lookup, Some(id));

        // Verify get_by_category works
        let walls = registry.get_by_category(BlueprintCategory::Wall);
        assert_eq!(walls.len(), 1);
        assert_eq!(walls[0].meta.id, "test_wall");
    }

    #[test]
    fn test_instantiate() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint);

        let params = {
            let mut p = HashMap::new();
            p.insert("length".to_string(), 10.0);
            p.insert("height".to_string(), 3.0);
            p
        };

        let instance = registry
            .instantiate(
                id,
                params,
                Vec2::new(100.0, 200.0),
                0.0,
                PlacedBy::TerrainGen,
                None,
            )
            .expect("Instantiation should succeed");

        // Verify geometry
        assert_eq!(instance.geometry.width, 10.0);
        assert_eq!(instance.geometry.depth, 0.6);
        assert_eq!(instance.geometry.height, 3.0);
        assert_eq!(instance.geometry.footprint.len(), 4); // Rectangle

        // Verify HP calculation: length * height * 80 = 10 * 3 * 80 = 2400
        assert_eq!(instance.max_hp, 2400.0);

        // Verify military properties
        assert_eq!(instance.military.hardness, 5.0);
        assert!((instance.military.cover_value - 0.8).abs() < f32::EPSILON);
        assert!(instance.military.blocks_movement);
        assert!(instance.military.blocks_los);
        assert!(!instance.military.flammable);

        // Verify position
        assert_eq!(instance.position, Vec2::new(100.0, 200.0));

        // Verify construction state (Constructed with stages = 0.0 progress)
        assert_eq!(instance.construction_progress, 0.0);
        assert_eq!(instance.construction_stage, Some("foundation".to_string()));

        // HP should be 0 since construction_progress is 0
        assert_eq!(instance.current_hp, 0.0);

        // Verify damage state
        assert_eq!(instance.damage_state, "intact");
    }

    #[test]
    fn test_validation() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint);

        // Valid params should pass
        let valid_params = {
            let mut p = HashMap::new();
            p.insert("length".to_string(), 10.0);
            p.insert("height".to_string(), 3.0);
            p
        };
        assert!(registry.validate_params(id, &valid_params).is_ok());

        // Out of range length (max is 20.0)
        let invalid_length = {
            let mut p = HashMap::new();
            p.insert("length".to_string(), 25.0);
            p.insert("height".to_string(), 3.0);
            p
        };
        let result = registry.validate_params(id, &invalid_length);
        assert!(result.is_err());
        match result {
            Err(BlueprintError::ValidationError(errors)) => {
                assert!(!errors.is_empty());
                assert!(errors[0].contains("length"));
                assert!(errors[0].contains("out of range"));
            }
            _ => panic!("Expected ValidationError"),
        }

        // Out of range height (min is 1.0)
        let invalid_height = {
            let mut p = HashMap::new();
            p.insert("length".to_string(), 10.0);
            p.insert("height".to_string(), 0.5);
            p
        };
        let result = registry.validate_params(id, &invalid_height);
        assert!(result.is_err());
    }

    #[test]
    fn test_defaults() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint);

        // Empty params should use defaults
        let empty_params = HashMap::new();

        let instance = registry
            .instantiate(
                id,
                empty_params,
                Vec2::ZERO,
                0.0,
                PlacedBy::TerrainGen,
                None,
            )
            .expect("Instantiation with defaults should succeed");

        // Should use default length=5.0 and height=2.5
        assert_eq!(instance.geometry.width, 5.0);
        assert_eq!(instance.geometry.height, 2.5);

        // HP should be: 5.0 * 2.5 * 80 = 1000 (but construction_progress is 0)
        assert_eq!(instance.max_hp, 1000.0);

        // Verify defaults are stored in parameters
        assert_eq!(instance.parameters.get("length"), Some(&5.0));
        assert_eq!(instance.parameters.get("height"), Some(&2.5));
    }

    #[test]
    fn test_natural_origin_complete() {
        let mut registry = BlueprintRegistry::new();
        let mut blueprint = create_test_blueprint();
        blueprint.meta.origin = OriginType::Natural;
        blueprint.construction = None;
        let id = registry.register(blueprint);

        let instance = registry
            .instantiate(
                id,
                HashMap::new(),
                Vec2::ZERO,
                0.0,
                PlacedBy::TerrainGen,
                None,
            )
            .expect("Natural instantiation should succeed");

        // Natural entities should be complete
        assert_eq!(instance.construction_progress, 1.0);
        assert!(instance.construction_stage.is_none());
        // HP should be max since construction is complete
        assert_eq!(instance.current_hp, instance.max_hp);
    }

    #[test]
    fn test_generate_footprint_rectangle() {
        let footprint = generate_footprint("rectangle", 4.0, 2.0);
        assert_eq!(footprint.len(), 4);

        // Verify corners (centered)
        assert!(footprint.contains(&Vec2::new(-2.0, -1.0)));
        assert!(footprint.contains(&Vec2::new(2.0, -1.0)));
        assert!(footprint.contains(&Vec2::new(2.0, 1.0)));
        assert!(footprint.contains(&Vec2::new(-2.0, 1.0)));
    }

    #[test]
    fn test_generate_footprint_circle() {
        let footprint = generate_footprint("circle", 4.0, 4.0);
        assert_eq!(footprint.len(), 16);

        // First vertex should be at (radius, 0)
        let first = footprint[0];
        assert!((first.x - 2.0).abs() < 0.001);
        assert!(first.y.abs() < 0.001);
    }

    #[test]
    fn test_eval_expr_str() {
        let mut params = HashMap::new();
        params.insert("x".to_string(), 5.0);
        params.insert("y".to_string(), 3.0);

        // Basic evaluation
        let result = eval_expr_str("x + y", &params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 8.0);

        // Empty string should error
        let result = eval_expr_str("", &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_expr_str_or() {
        let params = HashMap::new();

        // Empty string should return default
        let result = eval_expr_str_or("", &params, 42.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42.0);

        // Non-empty should evaluate
        let result = eval_expr_str_or("10 + 5", &params, 0.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 15.0);
    }

    #[test]
    fn test_resolve_anchor_cardinal() {
        let anchor = AnchorDef {
            name: "test".to_string(),
            position: ["1".to_string(), "2".to_string(), "3".to_string()],
            direction: "north".to_string(),
            tags: vec!["tag1".to_string()],
        };

        let params = HashMap::new();
        let resolved = resolve_anchor(&anchor, &params).unwrap();

        assert_eq!(resolved.name, "test");
        assert_eq!(resolved.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(resolved.direction, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(resolved.tags, vec!["tag1".to_string()]);
    }

    #[test]
    fn test_resolve_anchor_with_params() {
        let anchor = AnchorDef {
            name: "end".to_string(),
            position: ["width".to_string(), "0".to_string(), "0".to_string()],
            direction: "east".to_string(),
            tags: vec![],
        };

        let mut params = HashMap::new();
        params.insert("width".to_string(), 10.0);

        let resolved = resolve_anchor(&anchor, &params).unwrap();

        assert_eq!(resolved.position.x, 10.0);
    }

    #[test]
    fn test_blueprint_error_display() {
        let io_err = BlueprintError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(io_err.to_string().contains("I/O error"));

        let parse_err = BlueprintError::ParseError("invalid syntax".to_string());
        assert!(parse_err.to_string().contains("Parse error"));
        assert!(parse_err.to_string().contains("invalid syntax"));

        let not_found = BlueprintError::NotFound("missing_blueprint".to_string());
        assert!(not_found.to_string().contains("not found"));

        let validation_err =
            BlueprintError::ValidationError(vec!["error1".to_string(), "error2".to_string()]);
        assert!(validation_err.to_string().contains("error1"));
        assert!(validation_err.to_string().contains("error2"));

        let expr_err = BlueprintError::ExpressionError(EvalError::DivisionByZero);
        assert!(expr_err.to_string().contains("Expression error"));
    }

    #[test]
    fn test_unique_instance_ids() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint);

        let instance1 = registry
            .instantiate(
                id,
                HashMap::new(),
                Vec2::ZERO,
                0.0,
                PlacedBy::TerrainGen,
                None,
            )
            .unwrap();

        let instance2 = registry
            .instantiate(
                id,
                HashMap::new(),
                Vec2::ZERO,
                0.0,
                PlacedBy::TerrainGen,
                None,
            )
            .unwrap();

        // Each instance should have a unique ID
        assert_ne!(instance1.id, instance2.id);
    }

    #[test]
    fn test_constraint_validation() {
        let mut registry = BlueprintRegistry::new();
        let mut blueprint = create_test_blueprint();

        // Add a constraint: length must be greater than height
        blueprint
            .constraints
            .push(super::super::schema::ConstraintDef {
                description: "Length must be greater than height".to_string(),
                expression: "length > height".to_string(),
                error_message: "Wall length must exceed height".to_string(),
            });

        let id = registry.register(blueprint);

        // Valid: length > height
        let valid = {
            let mut p = HashMap::new();
            p.insert("length".to_string(), 10.0);
            p.insert("height".to_string(), 3.0);
            p
        };
        assert!(registry.validate_params(id, &valid).is_ok());

        // Invalid: length < height
        let invalid = {
            let mut p = HashMap::new();
            p.insert("length".to_string(), 2.0);
            p.insert("height".to_string(), 4.0);
            p
        };
        let result = registry.validate_params(id, &invalid);
        assert!(result.is_err());
        match result {
            Err(BlueprintError::ValidationError(errors)) => {
                assert!(errors.iter().any(|e| e.contains("must exceed height")));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_load_example_blueprints() {
        use std::path::Path;

        let mut registry = BlueprintRegistry::new();

        // This test only runs if data directory exists
        let data_path = Path::new("data/blueprints");
        if data_path.exists() {
            let loaded = registry.load_directory(data_path).unwrap();
            assert!(!loaded.is_empty(), "Should load at least one blueprint");

            // Check specific blueprints
            if let Some(wall) = registry.get_by_name("stone_wall") {
                assert_eq!(wall.meta.origin, OriginType::Constructed);
            }
            if let Some(tree) = registry.get_by_name("oak_tree") {
                assert_eq!(tree.meta.origin, OriginType::Natural);
            }
        }
    }
}
