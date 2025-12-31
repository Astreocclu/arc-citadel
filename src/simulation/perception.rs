//! Perception system - what entities notice based on their values

use crate::core::types::{EntityId, Vec2};
use crate::entity::species::human::HumanValues;
use crate::spatial::sparse_hash::SparseHashGrid;

#[derive(Debug, Clone)]
pub struct Perception {
    pub observer: EntityId,
    pub perceived_entities: Vec<PerceivedEntity>,
    pub perceived_objects: Vec<PerceivedObject>,
    pub perceived_events: Vec<PerceivedEvent>,
}

#[derive(Debug, Clone)]
pub struct PerceivedEntity {
    pub entity: EntityId,
    pub distance: f32,
    pub relationship: RelationshipType,
    pub threat_level: f32,
    pub notable_features: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipType {
    Unknown,
    Ally,
    Neutral,
    Hostile,
}

#[derive(Debug, Clone)]
pub struct PerceivedObject {
    pub object_type: String,
    pub position: Vec2,
    pub properties: Vec<ObjectProperty>,
}

#[derive(Debug, Clone)]
pub struct ObjectProperty {
    pub name: String,
    pub value: PropertyValue,
}

#[derive(Debug, Clone)]
pub enum PropertyValue {
    Quality(f32),
    Material(String),
    Condition(f32),
    Aesthetic(f32),
}

#[derive(Debug, Clone)]
pub struct PerceivedEvent {
    pub event_type: String,
    pub participants: Vec<EntityId>,
    pub significance: f32,
}

pub struct PerceptionRanges {
    pub visual_base: f32,
    pub audio_base: f32,
}

impl Default for PerceptionRanges {
    fn default() -> Self {
        Self {
            visual_base: 50.0,
            audio_base: 20.0,
        }
    }
}

pub fn effective_visual_range(
    base: f32,
    fatigue: f32,
    terrain_modifier: f32,
    light_level: f32,
) -> f32 {
    let fatigue_mod = if fatigue > 0.7 { 0.8 } else { 1.0 };
    base * terrain_modifier * light_level * fatigue_mod
}

pub fn filter_perception_human(
    raw_perception: &[PerceivedObject],
    values: &HumanValues,
) -> Vec<PerceivedObject> {
    raw_perception.iter()
        .filter(|obj| {
            if obj.properties.iter().any(|p| p.name == "threat") {
                return true;
            }

            for prop in &obj.properties {
                match &prop.name[..] {
                    "aesthetic" if values.beauty > 0.5 => return true,
                    "quality" if values.beauty > 0.5 || values.ambition > 0.5 => return true,
                    "social_status" if values.honor > 0.5 || values.ambition > 0.5 => return true,
                    "comfort" if values.comfort > 0.5 => return true,
                    "sacred" if values.piety > 0.5 => return true,
                    _ => {}
                }
            }

            false
        })
        .cloned()
        .collect()
}

pub fn perception_system(
    spatial_grid: &SparseHashGrid,
    positions: &[Vec2],
    entity_ids: &[EntityId],
    perception_range: f32,
) -> Vec<Perception> {
    entity_ids.iter().enumerate().map(|(i, &observer_id)| {
        let observer_pos = positions[i];

        let nearby: Vec<_> = spatial_grid.query_neighbors(observer_pos)
            .filter(|&e| e != observer_id)
            .collect();

        let perceived_entities: Vec<_> = nearby.iter()
            .filter_map(|&entity| {
                let entity_idx = entity_ids.iter().position(|&e| e == entity)?;
                let entity_pos = positions[entity_idx];
                let distance = observer_pos.distance(&entity_pos);

                if distance <= perception_range {
                    Some(PerceivedEntity {
                        entity,
                        distance,
                        relationship: RelationshipType::Unknown,
                        threat_level: 0.0,
                        notable_features: vec![],
                    })
                } else {
                    None
                }
            })
            .collect();

        Perception {
            observer: observer_id,
            perceived_entities,
            perceived_objects: vec![],
            perceived_events: vec![],
        }
    }).collect()
}
