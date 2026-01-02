//! Stockpile - settlement-level resource storage

use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use crate::simulation::resource_zone::ResourceType;

/// A stockpile holding resources for a settlement
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stockpile {
    /// Resources stored: type -> (current, capacity)
    resources: AHashMap<ResourceType, (u32, u32)>,
}

impl Stockpile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set capacity for a resource type
    pub fn set_capacity(&mut self, resource: ResourceType, capacity: u32) {
        let entry = self.resources.entry(resource).or_insert((0, 0));
        entry.1 = capacity;
    }

    /// Get current amount of a resource
    pub fn get(&self, resource: ResourceType) -> u32 {
        self.resources.get(&resource).map(|(c, _)| *c).unwrap_or(0)
    }

    /// Get capacity for a resource
    pub fn capacity(&self, resource: ResourceType) -> u32 {
        self.resources.get(&resource).map(|(_, cap)| *cap).unwrap_or(0)
    }

    /// Try to add resources, returns amount actually added
    pub fn add(&mut self, resource: ResourceType, amount: u32) -> u32 {
        let entry = self.resources.entry(resource).or_insert((0, 100)); // Default capacity 100
        let space = entry.1.saturating_sub(entry.0);
        let added = amount.min(space);
        entry.0 += added;
        added
    }

    /// Try to remove resources, returns amount actually removed
    pub fn remove(&mut self, resource: ResourceType, amount: u32) -> u32 {
        if let Some(entry) = self.resources.get_mut(&resource) {
            let removed = amount.min(entry.0);
            entry.0 -= removed;
            removed
        } else {
            0
        }
    }

    /// Check if stockpile has enough of all required materials
    pub fn has_materials(&self, requirements: &[(ResourceType, u32)]) -> bool {
        requirements.iter().all(|(res, amount)| self.get(*res) >= *amount)
    }

    /// Consume materials for construction, returns true if successful
    pub fn consume_materials(&mut self, requirements: &[(ResourceType, u32)]) -> bool {
        if !self.has_materials(requirements) {
            return false;
        }
        for (res, amount) in requirements {
            self.remove(*res, *amount);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stockpile_add_remove() {
        let mut stockpile = Stockpile::new();
        stockpile.set_capacity(ResourceType::Wood, 50);

        assert_eq!(stockpile.add(ResourceType::Wood, 30), 30);
        assert_eq!(stockpile.get(ResourceType::Wood), 30);

        // Can't exceed capacity
        assert_eq!(stockpile.add(ResourceType::Wood, 30), 20);
        assert_eq!(stockpile.get(ResourceType::Wood), 50);

        // Remove
        assert_eq!(stockpile.remove(ResourceType::Wood, 20), 20);
        assert_eq!(stockpile.get(ResourceType::Wood), 30);
    }

    #[test]
    fn test_stockpile_has_materials() {
        let mut stockpile = Stockpile::new();
        stockpile.add(ResourceType::Wood, 50);
        stockpile.add(ResourceType::Stone, 30);

        let requirements = vec![
            (ResourceType::Wood, 20),
            (ResourceType::Stone, 10),
        ];
        assert!(stockpile.has_materials(&requirements));

        let too_much = vec![
            (ResourceType::Wood, 100),
        ];
        assert!(!stockpile.has_materials(&too_much));
    }

    #[test]
    fn test_stockpile_consume_materials() {
        let mut stockpile = Stockpile::new();
        stockpile.add(ResourceType::Wood, 50);
        stockpile.add(ResourceType::Stone, 30);

        let requirements = vec![
            (ResourceType::Wood, 20),
            (ResourceType::Stone, 10),
        ];

        assert!(stockpile.consume_materials(&requirements));
        assert_eq!(stockpile.get(ResourceType::Wood), 30);
        assert_eq!(stockpile.get(ResourceType::Stone), 20);
    }
}
