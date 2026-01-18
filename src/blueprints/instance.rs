//! Runtime instance types for spawned blueprints.
//!
//! This module provides types for instantiated blueprints - the runtime representations
//! of entities created from blueprint definitions. Instances store cached evaluated
//! properties computed at spawn time, along with mutable state that changes during gameplay.

use glam::Vec2;
use std::collections::HashMap;

use super::schema::{BlueprintId, PropertyOverrides};

/// Unique identifier for an instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceId(pub u64);

/// Tracking who placed this instance and when
#[derive(Debug, Clone)]
pub enum PlacedBy {
    /// Terrain generation during worldgen
    TerrainGen,
    /// NPC polity during history simulation
    HistorySim { polity_id: u32, year: i32 },
    /// Player during gameplay
    Gameplay { tick: u64 },
}

/// Evaluated military properties (cached at instantiation)
#[derive(Debug, Clone, Default)]
pub struct MilitaryProperties {
    pub max_hp: f32,
    pub hardness: f32,
    pub cover_value: f32,
    pub blocks_movement: bool,
    pub blocks_los: bool,
    pub movement_cost: f32,
    pub flammable: bool,
    pub elevation: f32,
}

/// Evaluated civilian properties (cached at instantiation)
#[derive(Debug, Clone, Default)]
pub struct CivilianProperties {
    pub pedestrian_capacity: u32,
    pub cart_accessible: bool,
    pub worker_capacity: u32,
    pub storage_capacity: u32,
    pub prestige_modifier: f32,
    /// Aesthetic value (0.0-1.0) - perceived by high-beauty entities
    pub aesthetic_value: f32,
    /// Sacred/holy significance (0.0-1.0) - perceived by high-piety entities
    pub sacred_value: f32,
}

/// Evaluated geometry (cached at instantiation)
#[derive(Debug, Clone)]
pub struct EvaluatedGeometry {
    pub width: f32,
    pub depth: f32,
    pub height: f32,
    pub footprint: Vec<Vec2>, // Polygon vertices in local space
}

/// A resolved anchor point
#[derive(Debug, Clone)]
pub struct ResolvedAnchor {
    pub name: String,
    pub position: glam::Vec3,
    pub direction: glam::Vec3,
    pub tags: Vec<String>,
}

/// Breach in a damaged structure
#[derive(Debug, Clone)]
pub struct Breach {
    pub position: Vec2,
    pub width: f32,
}

/// A spawned instance of a blueprint
#[derive(Debug, Clone)]
pub struct BlueprintInstance {
    pub id: InstanceId,
    pub blueprint_id: BlueprintId,
    pub blueprint_name: String,

    /// The parameter values used to instantiate
    pub parameters: HashMap<String, f32>,

    /// World position and rotation
    pub position: Vec2,
    pub rotation: f32,

    /// Cached evaluated geometry
    pub geometry: EvaluatedGeometry,

    /// Current HP (mutable during gameplay)
    pub current_hp: f32,
    /// Cached max HP from evaluation
    pub max_hp: f32,
    /// Current damage state name
    pub damage_state: String,
    /// Active breaches
    pub breaches: Vec<Breach>,

    /// Cached military properties
    pub military: MilitaryProperties,
    /// Cached civilian properties
    pub civilian: CivilianProperties,

    /// Resolved anchor points
    pub anchors: Vec<ResolvedAnchor>,

    /// Construction progress (0.0 to 1.0, 1.0 = complete)
    pub construction_progress: f32,
    /// Current construction stage id
    pub construction_stage: Option<String>,

    /// Who placed this and when
    pub placed_by: PlacedBy,
    /// Owner faction (None for natural features)
    pub owner: Option<u32>,
}

impl BlueprintInstance {
    /// Get current HP as a ratio of max HP
    pub fn hp_ratio(&self) -> f32 {
        if self.max_hp == 0.0 {
            1.0
        } else {
            self.current_hp / self.max_hp
        }
    }

    /// Check if this instance is fully constructed
    pub fn is_complete(&self) -> bool {
        self.construction_progress >= 1.0
    }

    /// Apply damage, returns true if destroyed
    pub fn apply_damage(&mut self, amount: f32) -> bool {
        self.current_hp = (self.current_hp - amount).max(0.0);
        self.current_hp <= 0.0
    }

    /// Apply property overrides from damage/construction state
    pub fn apply_overrides(&mut self, overrides: &PropertyOverrides) {
        if let Some(cover) = overrides.cover_value {
            self.military.cover_value = cover;
        }
        if let Some(blocks) = overrides.blocks_movement {
            self.military.blocks_movement = blocks;
        }
        if let Some(los) = overrides.blocks_los {
            self.military.blocks_los = los;
        }
        if let Some(hard) = overrides.hardness {
            self.military.hardness = hard;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hp_ratio() {
        let instance = BlueprintInstance {
            id: InstanceId(1),
            blueprint_id: BlueprintId(1),
            blueprint_name: "test".to_string(),
            parameters: HashMap::new(),
            position: Vec2::ZERO,
            rotation: 0.0,
            geometry: EvaluatedGeometry {
                width: 5.0,
                depth: 1.0,
                height: 3.0,
                footprint: vec![],
            },
            current_hp: 50.0,
            max_hp: 100.0,
            damage_state: "intact".to_string(),
            breaches: vec![],
            military: MilitaryProperties::default(),
            civilian: CivilianProperties::default(),
            anchors: vec![],
            construction_progress: 1.0,
            construction_stage: None,
            placed_by: PlacedBy::TerrainGen,
            owner: None,
        };

        assert_eq!(instance.hp_ratio(), 0.5);
        assert!(instance.is_complete());
    }

    #[test]
    fn test_hp_ratio_zero_max() {
        let instance = BlueprintInstance {
            id: InstanceId(1),
            blueprint_id: BlueprintId(1),
            blueprint_name: "test".to_string(),
            parameters: HashMap::new(),
            position: Vec2::ZERO,
            rotation: 0.0,
            geometry: EvaluatedGeometry {
                width: 1.0,
                depth: 1.0,
                height: 1.0,
                footprint: vec![],
            },
            current_hp: 0.0,
            max_hp: 0.0,
            damage_state: "intact".to_string(),
            breaches: vec![],
            military: MilitaryProperties::default(),
            civilian: CivilianProperties::default(),
            anchors: vec![],
            construction_progress: 1.0,
            construction_stage: None,
            placed_by: PlacedBy::TerrainGen,
            owner: None,
        };

        // Should return 1.0 to avoid division by zero
        assert_eq!(instance.hp_ratio(), 1.0);
    }

    #[test]
    fn test_apply_damage() {
        let mut instance = BlueprintInstance {
            id: InstanceId(1),
            blueprint_id: BlueprintId(1),
            blueprint_name: "test".to_string(),
            parameters: HashMap::new(),
            position: Vec2::ZERO,
            rotation: 0.0,
            geometry: EvaluatedGeometry {
                width: 5.0,
                depth: 1.0,
                height: 3.0,
                footprint: vec![],
            },
            current_hp: 100.0,
            max_hp: 100.0,
            damage_state: "intact".to_string(),
            breaches: vec![],
            military: MilitaryProperties::default(),
            civilian: CivilianProperties::default(),
            anchors: vec![],
            construction_progress: 1.0,
            construction_stage: None,
            placed_by: PlacedBy::Gameplay { tick: 0 },
            owner: Some(1),
        };

        assert!(!instance.apply_damage(30.0));
        assert_eq!(instance.current_hp, 70.0);

        assert!(instance.apply_damage(100.0));
        assert_eq!(instance.current_hp, 0.0);
    }

    #[test]
    fn test_apply_overrides() {
        let mut instance = BlueprintInstance {
            id: InstanceId(1),
            blueprint_id: BlueprintId(1),
            blueprint_name: "test".to_string(),
            parameters: HashMap::new(),
            position: Vec2::ZERO,
            rotation: 0.0,
            geometry: EvaluatedGeometry {
                width: 5.0,
                depth: 1.0,
                height: 3.0,
                footprint: vec![],
            },
            current_hp: 100.0,
            max_hp: 100.0,
            damage_state: "intact".to_string(),
            breaches: vec![],
            military: MilitaryProperties {
                max_hp: 100.0,
                hardness: 10.0,
                cover_value: 0.8,
                blocks_movement: true,
                blocks_los: true,
                movement_cost: 1.0,
                flammable: false,
                elevation: 0.0,
            },
            civilian: CivilianProperties::default(),
            anchors: vec![],
            construction_progress: 1.0,
            construction_stage: None,
            placed_by: PlacedBy::TerrainGen,
            owner: None,
        };

        let overrides = PropertyOverrides {
            cover_value: Some(0.3),
            blocks_movement: Some(false),
            blocks_los: None,
            hardness: Some(5.0),
        };

        instance.apply_overrides(&overrides);

        assert_eq!(instance.military.cover_value, 0.3);
        assert!(!instance.military.blocks_movement);
        assert!(instance.military.blocks_los); // Unchanged
        assert_eq!(instance.military.hardness, 5.0);
    }

    #[test]
    fn test_construction_incomplete() {
        let instance = BlueprintInstance {
            id: InstanceId(1),
            blueprint_id: BlueprintId(1),
            blueprint_name: "wall".to_string(),
            parameters: HashMap::new(),
            position: Vec2::ZERO,
            rotation: 0.0,
            geometry: EvaluatedGeometry {
                width: 5.0,
                depth: 1.0,
                height: 3.0,
                footprint: vec![],
            },
            current_hp: 50.0,
            max_hp: 100.0,
            damage_state: "under_construction".to_string(),
            breaches: vec![],
            military: MilitaryProperties::default(),
            civilian: CivilianProperties::default(),
            anchors: vec![],
            construction_progress: 0.5,
            construction_stage: Some("foundation".to_string()),
            placed_by: PlacedBy::Gameplay { tick: 100 },
            owner: Some(1),
        };

        assert!(!instance.is_complete());
        assert_eq!(instance.construction_progress, 0.5);
        assert_eq!(instance.construction_stage, Some("foundation".to_string()));
    }

    #[test]
    fn test_placed_by_variants() {
        let terrain = PlacedBy::TerrainGen;
        let history = PlacedBy::HistorySim {
            polity_id: 42,
            year: -500,
        };
        let gameplay = PlacedBy::Gameplay { tick: 12345 };

        // Just verify these compile and can be matched
        match terrain {
            PlacedBy::TerrainGen => {}
            _ => panic!("Expected TerrainGen"),
        }

        match history {
            PlacedBy::HistorySim { polity_id, year } => {
                assert_eq!(polity_id, 42);
                assert_eq!(year, -500);
            }
            _ => panic!("Expected HistorySim"),
        }

        match gameplay {
            PlacedBy::Gameplay { tick } => {
                assert_eq!(tick, 12345);
            }
            _ => panic!("Expected Gameplay"),
        }
    }
}
