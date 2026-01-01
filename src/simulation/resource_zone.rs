//! Resource zones for gatherable resources (wood, stone, ore)
//!
//! ResourceZone represents a location where entities can gather resources.
//! Resources deplete when gathered and slowly regenerate over time.

use serde::{Deserialize, Serialize};
use crate::core::types::Vec2;

/// Type of resource available in a zone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Wood,
    Stone,
    Ore,
}

/// A zone where entities can gather resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceZone {
    pub position: Vec2,
    pub resource_type: ResourceType,
    pub radius: f32,
    pub current: f32,    // 0.0 to 1.0
    pub max: f32,        // Usually 1.0
    pub regen_rate: f32, // Per tick
}

impl ResourceZone {
    /// Create a new resource zone at the given position
    pub fn new(position: Vec2, resource_type: ResourceType, radius: f32) -> Self {
        Self {
            position,
            resource_type,
            radius,
            current: 1.0,
            max: 1.0,
            regen_rate: 0.0001, // Very slow regen
        }
    }

    /// Check if a position is within this resource zone
    pub fn contains(&self, pos: Vec2) -> bool {
        self.position.distance(&pos) <= self.radius
    }

    /// Gather resources, returns amount actually gathered
    pub fn gather(&mut self, amount: f32) -> f32 {
        let gathered = amount.min(self.current);
        self.current -= gathered;
        gathered
    }

    /// Regenerate resources over time
    pub fn regenerate(&mut self) {
        self.current = (self.current + self.regen_rate).min(self.max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Vec2;

    #[test]
    fn test_resource_zone_creation() {
        let zone = ResourceZone::new(
            Vec2::new(10.0, 20.0),
            ResourceType::Wood,
            5.0, // radius
        );
        assert_eq!(zone.resource_type, ResourceType::Wood);
        assert!((zone.current - 1.0).abs() < 0.01); // Starts full
        assert!(zone.contains(Vec2::new(12.0, 22.0))); // Inside
        assert!(!zone.contains(Vec2::new(20.0, 20.0))); // Outside
    }

    #[test]
    fn test_resource_zone_depletion() {
        let mut zone = ResourceZone::new(
            Vec2::new(0.0, 0.0),
            ResourceType::Stone,
            5.0,
        );

        let gathered = zone.gather(0.3);
        assert!((gathered - 0.3).abs() < 0.01);
        assert!((zone.current - 0.7).abs() < 0.01);

        // Can't gather more than available
        let gathered = zone.gather(1.0);
        assert!((gathered - 0.7).abs() < 0.01);
        assert!(zone.current < 0.01);
    }

    #[test]
    fn test_resource_zone_regeneration() {
        let mut zone = ResourceZone::new(
            Vec2::new(0.0, 0.0),
            ResourceType::Ore,
            5.0,
        );

        // Deplete the zone
        zone.gather(1.0);
        assert!(zone.current < 0.01);

        // Regenerate
        zone.regenerate();
        assert!((zone.current - 0.0001).abs() < 0.0001);

        // Multiple regenerations
        for _ in 0..1000 {
            zone.regenerate();
        }
        assert!(zone.current > 0.05); // Should have regenerated significantly
        assert!(zone.current <= zone.max); // But not beyond max
    }

    #[test]
    fn test_resource_zone_contains_boundary() {
        let zone = ResourceZone::new(
            Vec2::new(0.0, 0.0),
            ResourceType::Wood,
            5.0,
        );

        // Exactly on the boundary should be contained
        assert!(zone.contains(Vec2::new(5.0, 0.0)));
        assert!(zone.contains(Vec2::new(0.0, 5.0)));

        // Just outside should not be contained
        assert!(!zone.contains(Vec2::new(5.1, 0.0)));
        assert!(!zone.contains(Vec2::new(0.0, 5.1)));
    }
}
