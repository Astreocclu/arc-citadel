//! JSON schema types for worldgen placement output
//!
//! This module defines types for deserializing world object placements
//! from the Python worldgen pipeline. The placements are stored as JSON
//! files containing positioned blueprint instances with metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root structure for placement JSON files
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlacementFile {
    /// Schema version (currently 1)
    pub version: u32,
    /// Optional metadata about this placement file
    #[serde(default)]
    pub metadata: Option<PlacementMetadata>,
    /// List of placed objects
    pub placements: Vec<Placement>,
}

/// Optional metadata for the placement file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlacementMetadata {
    /// Human-readable name for this placement set
    #[serde(default)]
    pub name: Option<String>,
    /// Description of what these placements represent
    #[serde(default)]
    pub description: Option<String>,
    /// Tool or process that created this file
    #[serde(default)]
    pub created_by: Option<String>,
}

/// A single placed object in the world
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Placement {
    /// Unique identifier for this placement
    pub id: String,
    /// Blueprint template name to instantiate
    pub template: String,
    /// World position [x, y]
    pub position: [f32; 2],
    /// Rotation in degrees (defaults to 0)
    #[serde(default)]
    pub rotation_deg: Option<f32>,
    /// Who/what placed this object
    pub placed_by: PlacedByJson,
    /// Construction state
    #[serde(default)]
    pub state: Option<ObjectState>,
    /// Damage state name (e.g., "intact", "damaged", "ruined")
    #[serde(default)]
    pub damage_state: Option<String>,
    /// Current HP as ratio of max (0.0 to 1.0)
    #[serde(default)]
    pub current_hp_ratio: Option<f32>,
    /// Construction progress (0.0 to 1.0)
    #[serde(default)]
    pub construction_progress: Option<f32>,
    /// Blueprint parameter overrides
    #[serde(default)]
    pub parameters: HashMap<String, f32>,
    /// Tags for filtering/querying
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Object construction state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectState {
    /// Fully constructed
    Complete,
    /// Still being built
    UnderConstruction,
}

/// Who placed this object - JSON deserialization type
///
/// Handles both string "TerrainGen" and objects like:
/// - `{ "HistorySim": { "polity_id": 42, "year": -500 } }`
/// - `{ "Gameplay": { "tick": 12345 } }`
#[derive(Debug, Clone, PartialEq)]
pub enum PlacedByJson {
    /// Terrain generation (string "TerrainGen")
    TerrainGen,
    /// NPC polity during history simulation
    HistorySim { polity_id: u32, year: i32 },
    /// Player during gameplay
    Gameplay { tick: u64 },
}

impl PlacedByJson {
    /// Convert to runtime PlacedBy type
    pub fn to_runtime(&self) -> crate::blueprints::PlacedBy {
        match self {
            PlacedByJson::TerrainGen => crate::blueprints::PlacedBy::TerrainGen,
            PlacedByJson::HistorySim { polity_id, year } => {
                crate::blueprints::PlacedBy::HistorySim {
                    polity_id: *polity_id,
                    year: *year,
                }
            }
            PlacedByJson::Gameplay { tick } => {
                crate::blueprints::PlacedBy::Gameplay { tick: *tick }
            }
        }
    }
}

// Custom serde implementation for PlacedByJson
// Handles both string "TerrainGen" and object variants
impl<'de> Deserialize<'de> for PlacedByJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct PlacedByVisitor;

        impl<'de> Visitor<'de> for PlacedByVisitor {
            type Value = PlacedByJson;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string \"TerrainGen\" or object with HistorySim/Gameplay")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value == "TerrainGen" {
                    Ok(PlacedByJson::TerrainGen)
                } else {
                    Err(de::Error::unknown_variant(value, &["TerrainGen"]))
                }
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                if let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "HistorySim" => {
                            #[derive(Deserialize)]
                            struct HistorySimData {
                                polity_id: u32,
                                year: i32,
                            }
                            let data: HistorySimData = map.next_value()?;
                            Ok(PlacedByJson::HistorySim {
                                polity_id: data.polity_id,
                                year: data.year,
                            })
                        }
                        "Gameplay" => {
                            #[derive(Deserialize)]
                            struct GameplayData {
                                tick: u64,
                            }
                            let data: GameplayData = map.next_value()?;
                            Ok(PlacedByJson::Gameplay { tick: data.tick })
                        }
                        _ => Err(de::Error::unknown_variant(
                            &key,
                            &["HistorySim", "Gameplay"],
                        )),
                    }
                } else {
                    Err(de::Error::custom("expected non-empty map"))
                }
            }
        }

        deserializer.deserialize_any(PlacedByVisitor)
    }
}

impl Serialize for PlacedByJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        match self {
            PlacedByJson::TerrainGen => serializer.serialize_str("TerrainGen"),
            PlacedByJson::HistorySim { polity_id, year } => {
                let mut map = serializer.serialize_map(Some(1))?;
                #[derive(Serialize)]
                struct HistorySimData {
                    polity_id: u32,
                    year: i32,
                }
                map.serialize_entry(
                    "HistorySim",
                    &HistorySimData {
                        polity_id: *polity_id,
                        year: *year,
                    },
                )?;
                map.end()
            }
            PlacedByJson::Gameplay { tick } => {
                let mut map = serializer.serialize_map(Some(1))?;
                #[derive(Serialize)]
                struct GameplayData {
                    tick: u64,
                }
                map.serialize_entry("Gameplay", &GameplayData { tick: *tick })?;
                map.end()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_JSON: &str = r#"{
        "version": 1,
        "placements": [
            {
                "id": "castle_001",
                "template": "castle_keep",
                "position": [100.0, 200.0],
                "rotation_deg": 45.0,
                "placed_by": { "HistorySim": { "polity_id": 42, "year": -500 } },
                "state": "complete",
                "damage_state": "damaged",
                "current_hp_ratio": 0.6,
                "parameters": { "length": 30.0, "height": 25.0 },
                "tags": ["fortification", "stone"]
            },
            {
                "id": "tree_001",
                "template": "oak_tree",
                "position": [50.0, 60.0],
                "placed_by": "TerrainGen",
                "parameters": { "height": 15.0 }
            }
        ]
    }"#;

    #[test]
    fn test_deserialize_placement_file() {
        let file: PlacementFile = serde_json::from_str(SAMPLE_JSON).unwrap();
        assert_eq!(file.version, 1);
        assert_eq!(file.placements.len(), 2);
    }

    #[test]
    fn test_placement_with_history_sim() {
        let file: PlacementFile = serde_json::from_str(SAMPLE_JSON).unwrap();
        let castle = &file.placements[0];

        assert_eq!(castle.id, "castle_001");
        assert_eq!(castle.template, "castle_keep");
        assert_eq!(castle.position, [100.0, 200.0]);
        assert_eq!(castle.rotation_deg, Some(45.0));
        assert!(matches!(castle.placed_by, PlacedByJson::HistorySim { .. }));
        assert_eq!(castle.state, Some(ObjectState::Complete));
        assert_eq!(castle.damage_state, Some("damaged".to_string()));
    }

    #[test]
    fn test_placement_natural_defaults() {
        let file: PlacementFile = serde_json::from_str(SAMPLE_JSON).unwrap();
        let tree = &file.placements[1];

        assert_eq!(tree.template, "oak_tree");
        assert!(matches!(tree.placed_by, PlacedByJson::TerrainGen));
        assert!(tree.state.is_none());
        assert!(tree.rotation_deg.is_none());
    }
}
