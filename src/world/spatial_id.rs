//! Unified spatial ID for entities and world objects

use crate::blueprints::InstanceId;
use crate::core::types::EntityId;

/// Unified ID type for spatial grid storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpatialId {
    /// An entity (human, orc, etc.)
    Entity(EntityId),
    /// A world object (wall, tree, building)
    Object(InstanceId),
}

impl SpatialId {
    /// Check if this is an entity ID
    pub fn is_entity(&self) -> bool {
        matches!(self, Self::Entity(_))
    }

    /// Check if this is an object ID
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    /// Get the entity ID if this is an entity
    pub fn as_entity(&self) -> Option<EntityId> {
        match self {
            Self::Entity(id) => Some(*id),
            Self::Object(_) => None,
        }
    }

    /// Get the instance ID if this is an object
    pub fn as_object(&self) -> Option<InstanceId> {
        match self {
            Self::Object(id) => Some(*id),
            Self::Entity(_) => None,
        }
    }
}

impl From<EntityId> for SpatialId {
    fn from(id: EntityId) -> Self {
        Self::Entity(id)
    }
}

impl From<InstanceId> for SpatialId {
    fn from(id: InstanceId) -> Self {
        Self::Object(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprints::InstanceId;
    use crate::core::types::EntityId;

    #[test]
    fn test_spatial_id_entity() {
        let entity_id = EntityId::new();
        let spatial = SpatialId::Entity(entity_id);

        assert!(spatial.is_entity());
        assert!(!spatial.is_object());
        assert_eq!(spatial.as_entity(), Some(entity_id));
        assert_eq!(spatial.as_object(), None);
    }

    #[test]
    fn test_spatial_id_object() {
        let instance_id = InstanceId(42);
        let spatial = SpatialId::Object(instance_id);

        assert!(!spatial.is_entity());
        assert!(spatial.is_object());
        assert_eq!(spatial.as_entity(), None);
        assert_eq!(spatial.as_object(), Some(instance_id));
    }

    #[test]
    fn test_spatial_id_hash_eq() {
        use std::collections::HashSet;

        let s1 = SpatialId::Object(InstanceId(1));
        let s2 = SpatialId::Object(InstanceId(1));
        let s3 = SpatialId::Object(InstanceId(2));

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);

        let mut set = HashSet::new();
        set.insert(s1);
        assert!(set.contains(&s2));
        assert!(!set.contains(&s3));
    }
}
