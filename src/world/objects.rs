//! World objects storage and queries

use crate::blueprints::{BlueprintInstance, InstanceId};
use ahash::AHashMap;
use glam::Vec2;

/// Storage for all world objects (walls, trees, buildings, etc.)
pub struct WorldObjects {
    /// All instances by ID
    instances: AHashMap<InstanceId, BlueprintInstance>,
}

impl WorldObjects {
    pub fn new() -> Self {
        Self {
            instances: AHashMap::new(),
        }
    }

    /// Add a world object
    pub fn add(&mut self, instance: BlueprintInstance) {
        self.instances.insert(instance.id, instance);
    }

    /// Get an object by ID
    pub fn get(&self, id: InstanceId) -> Option<&BlueprintInstance> {
        self.instances.get(&id)
    }

    /// Get mutable reference to an object
    pub fn get_mut(&mut self, id: InstanceId) -> Option<&mut BlueprintInstance> {
        self.instances.get_mut(&id)
    }

    /// Remove an object
    pub fn remove(&mut self, id: InstanceId) -> Option<BlueprintInstance> {
        self.instances.remove(&id)
    }

    /// Get all objects within radius of a point
    pub fn get_in_radius(&self, center: Vec2, radius: f32) -> Vec<&BlueprintInstance> {
        let radius_sq = radius * radius;
        self.instances
            .values()
            .filter(|obj| obj.position.distance_squared(center) <= radius_sq)
            .collect()
    }

    /// Iterate over all objects
    pub fn iter(&self) -> impl Iterator<Item = &BlueprintInstance> {
        self.instances.values()
    }

    /// Number of objects
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
}

impl Default for WorldObjects {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprints::{
        BlueprintId, BlueprintInstance, EvaluatedGeometry, InstanceId, PlacedBy,
    };
    use glam::Vec2;
    use std::collections::HashMap;

    fn make_test_instance(id: u64, name: &str, pos: Vec2) -> BlueprintInstance {
        BlueprintInstance {
            id: InstanceId(id),
            blueprint_id: BlueprintId(1),
            blueprint_name: name.to_string(),
            parameters: HashMap::new(),
            position: pos,
            rotation: 0.0,
            geometry: EvaluatedGeometry {
                width: 1.0,
                depth: 1.0,
                height: 1.0,
                footprint: vec![],
            },
            current_hp: 100.0,
            max_hp: 100.0,
            damage_state: "intact".to_string(),
            breaches: vec![],
            construction_progress: 1.0,
            construction_stage: None,
            placed_by: PlacedBy::TerrainGen,
            military: Default::default(),
            civilian: Default::default(),
            anchors: vec![],
            owner: None,
        }
    }

    #[test]
    fn test_add_and_get_object() {
        let mut objects = WorldObjects::new();
        let instance = make_test_instance(1, "oak_tree", Vec2::new(10.0, 20.0));

        objects.add(instance.clone());

        let retrieved = objects.get(InstanceId(1));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().blueprint_name, "oak_tree");
    }

    #[test]
    fn test_get_by_position() {
        let mut objects = WorldObjects::new();
        objects.add(make_test_instance(1, "tree", Vec2::new(10.0, 10.0)));
        objects.add(make_test_instance(2, "rock", Vec2::new(50.0, 50.0)));

        let nearby = objects.get_in_radius(Vec2::new(12.0, 12.0), 10.0);
        assert_eq!(nearby.len(), 1);
        assert_eq!(nearby[0].blueprint_name, "tree");
    }

    #[test]
    fn test_remove_object() {
        let mut objects = WorldObjects::new();
        objects.add(make_test_instance(1, "wall", Vec2::new(0.0, 0.0)));

        assert!(objects.get(InstanceId(1)).is_some());
        objects.remove(InstanceId(1));
        assert!(objects.get(InstanceId(1)).is_none());
    }
}
