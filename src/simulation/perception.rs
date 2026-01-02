//! Perception system - what entities notice based on their values

use crate::city::building::{BuildingArchetype, BuildingId};
use crate::core::types::{EntityId, Vec2};
use crate::ecs::world::FoodZone;
use crate::entity::social::{Disposition, SocialMemory};
use crate::entity::species::human::HumanValues;
use crate::spatial::sparse_hash::SparseHashGrid;

#[derive(Debug, Clone)]
pub struct Perception {
    pub observer: EntityId,
    pub perceived_entities: Vec<PerceivedEntity>,
    pub perceived_objects: Vec<PerceivedObject>,
    pub perceived_events: Vec<PerceivedEvent>,
    /// Nearest food zone: (zone_id, position, distance)
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    /// Nearest building under construction: (building_id, position, distance)
    pub nearest_building_site: Option<(BuildingId, Vec2, f32)>,
}

#[derive(Debug, Clone)]
pub struct PerceivedEntity {
    pub entity: EntityId,
    pub distance: f32,
    pub relationship: RelationshipType,
    pub disposition: Disposition,
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

/// Find the nearest food zone within perception range
pub fn find_nearest_food_zone(
    observer_pos: Vec2,
    perception_range: f32,
    food_zones: &[FoodZone],
) -> Option<(u32, Vec2, f32)> {
    let mut nearest: Option<(u32, Vec2, f32)> = None;

    for zone in food_zones {
        let distance = observer_pos.distance(&zone.position);
        if distance <= perception_range {
            if nearest.is_none() || distance < nearest.as_ref().unwrap().2 {
                nearest = Some((zone.id, zone.position, distance));
            }
        }
    }

    nearest
}

/// Find nearest building under construction within range
///
/// Returns (BuildingId, position, distance) for the nearest construction site.
/// Only considers buildings with state = UnderConstruction.
pub fn find_nearest_building_site(
    observer_pos: Vec2,
    range: f32,
    buildings: &BuildingArchetype,
) -> Option<(BuildingId, Vec2, f32)> {
    use crate::city::building::BuildingState;

    let mut nearest: Option<(BuildingId, Vec2, f32)> = None;

    for i in buildings.iter_under_construction() {
        let pos = buildings.positions[i];
        let distance = observer_pos.distance(&pos);

        if distance <= range {
            if nearest.is_none() || distance < nearest.unwrap().2 {
                nearest = Some((buildings.ids[i], pos, distance));
            }
        }
    }

    nearest
}

pub fn perception_system(
    spatial_grid: &SparseHashGrid,
    positions: &[Vec2],
    entity_ids: &[EntityId],
    social_memories: &[SocialMemory],
    perception_ranges: &[f32],
) -> Vec<Perception> {
    // Build O(1) lookup map for entity indices
    let id_to_idx: ahash::AHashMap<EntityId, usize> = entity_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    entity_ids.iter().enumerate().map(|(i, &observer_id)| {
        let observer_pos = positions[i];
        let observer_memory = &social_memories[i];
        let perception_range = perception_ranges[i];

        let nearby: Vec<_> = spatial_grid.query_neighbors(observer_pos)
            .filter(|&e| e != observer_id)
            .collect();

        let perceived_entities: Vec<_> = nearby.iter()
            .filter_map(|&entity| {
                let entity_idx = *id_to_idx.get(&entity)?;
                let entity_pos = positions[entity_idx];
                let distance = observer_pos.distance(&entity_pos);

                if distance <= perception_range {
                    // Look up disposition from social memory
                    let disposition = observer_memory.get_disposition(entity);

                    Some(PerceivedEntity {
                        entity,
                        distance,
                        relationship: RelationshipType::Unknown,
                        disposition,
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
            nearest_food_zone: None, // Will be populated by caller with food zone data
            nearest_building_site: None, // Will be populated by caller with building data
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Vec2;
    use crate::ecs::world::{Abundance, FoodZone};
    use crate::entity::social::EventType;

    #[test]
    fn test_perception_finds_food_zone() {
        let zones = vec![
            FoodZone { id: 0, position: Vec2::new(50.0, 50.0), radius: 10.0, abundance: Abundance::Unlimited },
            FoodZone { id: 1, position: Vec2::new(200.0, 200.0), radius: 20.0, abundance: Abundance::Unlimited },
        ];

        let observer_pos = Vec2::new(60.0, 60.0);
        let perception_range = 100.0;

        let nearest = find_nearest_food_zone(observer_pos, perception_range, &zones);

        assert!(nearest.is_some());
        let (zone_id, _zone_pos, distance) = nearest.unwrap();
        assert_eq!(zone_id, 0);  // Closer zone
        assert!(distance < 20.0);
    }

    #[test]
    fn test_find_nearest_building_site() {
        use crate::city::building::{BuildingArchetype, BuildingType, BuildingId, BuildingState};

        let mut buildings = BuildingArchetype::new();
        // Spawn two buildings under construction at different distances
        let id1 = BuildingId::new();
        let id2 = BuildingId::new();
        buildings.spawn(id1, BuildingType::House, Vec2::new(10.0, 0.0), 0);
        buildings.spawn(id2, BuildingType::Farm, Vec2::new(5.0, 0.0), 0);

        let observer = Vec2::new(0.0, 0.0);
        let result = super::find_nearest_building_site(observer, 50.0, &buildings);

        assert!(result.is_some());
        let (id, _, dist) = result.unwrap();
        assert_eq!(id, id2); // Farm is closer at distance 5.0
        assert!((dist - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_find_nearest_building_site_ignores_completed() {
        use crate::city::building::{BuildingArchetype, BuildingType, BuildingId, BuildingState};

        let mut buildings = BuildingArchetype::new();
        let id1 = BuildingId::new();
        let id2 = BuildingId::new();
        buildings.spawn(id1, BuildingType::House, Vec2::new(5.0, 0.0), 0);  // Closer
        buildings.spawn(id2, BuildingType::Farm, Vec2::new(10.0, 0.0), 0);  // Farther

        // Complete the closer building
        buildings.states[0] = BuildingState::Complete;

        let observer = Vec2::new(0.0, 0.0);
        let result = super::find_nearest_building_site(observer, 50.0, &buildings);

        // Should find the farther one since the closer one is complete
        assert!(result.is_some());
        let (id, _, dist) = result.unwrap();
        assert_eq!(id, id2);
        assert!((dist - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_find_nearest_building_site_none_in_range() {
        use crate::city::building::{BuildingArchetype, BuildingType, BuildingId};

        let mut buildings = BuildingArchetype::new();
        buildings.spawn(BuildingId::new(), BuildingType::House, Vec2::new(100.0, 0.0), 0);

        let observer = Vec2::new(0.0, 0.0);
        // Perception range is only 50, building is at 100
        let result = super::find_nearest_building_site(observer, 50.0, &buildings);

        assert!(result.is_none());
    }

    #[test]
    fn test_perception_includes_disposition() {
        use crate::spatial::sparse_hash::SparseHashGrid;

        // Create two entities
        let alice = EntityId::new();
        let bob = EntityId::new();

        let positions = vec![
            Vec2::new(0.0, 0.0),  // Alice at origin
            Vec2::new(5.0, 0.0),  // Bob nearby
        ];
        let ids = vec![alice, bob];

        // Build spatial grid
        let mut grid = SparseHashGrid::new(10.0);
        grid.rebuild(ids.iter().cloned().zip(positions.iter().cloned()));

        // Create social memories - Alice has positive memories of Bob
        let mut alice_memory = SocialMemory::new();
        alice_memory.record_encounter(bob, EventType::AidReceived, 0.8, 0);
        alice_memory.record_encounter(bob, EventType::GiftReceived, 0.6, 10);

        let bob_memory = SocialMemory::new();  // Bob has no memories

        let social_memories = vec![alice_memory, bob_memory];

        // Run perception (both entities have base range 50.0)
        let perception_ranges = vec![50.0, 50.0];
        let perceptions = perception_system(&grid, &positions, &ids, &social_memories, &perception_ranges);

        // Alice's perception of Bob should include disposition
        let alice_perception = perceptions.iter().find(|p| p.observer == alice).unwrap();
        assert_eq!(alice_perception.perceived_entities.len(), 1);

        let perceived_bob = &alice_perception.perceived_entities[0];
        assert_eq!(perceived_bob.entity, bob);
        assert_eq!(perceived_bob.disposition, Disposition::Favorable);

        // Bob's perception of Alice should be Unknown (no memories)
        let bob_perception = perceptions.iter().find(|p| p.observer == bob).unwrap();
        assert_eq!(bob_perception.perceived_entities.len(), 1);

        let perceived_alice = &bob_perception.perceived_entities[0];
        assert_eq!(perceived_alice.entity, alice);
        assert_eq!(perceived_alice.disposition, Disposition::Unknown);
    }
}
