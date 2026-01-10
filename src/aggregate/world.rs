//! AggregateWorld - the main world state container

use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::aggregate::polity::Polity;
use crate::aggregate::region::Region;
use crate::aggregate::ruler::Ruler;
use crate::core::types::{PolityId, RulerId};

/// The aggregate world state for history simulation
pub struct AggregateWorld {
    /// All regions (pseudo-nodes) in the world
    pub regions: Vec<Region>,
    /// All polities (nations/tribes/holds/groves)
    pub polities: Vec<Polity>,
    /// All rulers (characters who lead polities)
    pub rulers: HashMap<RulerId, Ruler>,
    /// Currently active wars
    pub active_wars: Vec<War>,
    /// Current simulation year
    pub year: u32,
    /// Random number generator (deterministic)
    pub rng: ChaCha8Rng,
    /// Next polity ID to assign
    next_polity_id: u32,
    /// Next ruler ID to assign
    next_ruler_id: u32,
}

/// Active war state machine
pub struct War {
    pub id: u32,
    pub aggressor: u32,
    pub defender: u32,
    pub cause: WarCause,
    pub start_year: u32,
    pub state: WarState,
    pub contested_regions: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WarCause {
    Expansion,
    Grudge(Vec<u32>), // Grudge IDs
    Honor,
    Religion,
    Grief,
    Defense,
}

#[derive(Clone, Debug)]
pub enum WarState {
    Declared,
    Active,
    Stalemate { years: u32 },
    Concluded { victor: Option<u32> },
}

impl AggregateWorld {
    pub fn new(regions: Vec<Region>, polities: Vec<Polity>, rng: ChaCha8Rng) -> Self {
        let next_polity_id = polities.iter().map(|p| p.id.0).max().unwrap_or(0) + 1;

        Self {
            regions,
            polities,
            rulers: HashMap::new(),
            active_wars: Vec::new(),
            year: 0,
            rng,
            next_polity_id,
            next_ruler_id: 1,
        }
    }

    /// Generate a new unique PolityId
    pub fn next_polity_id(&mut self) -> PolityId {
        let id = PolityId(self.next_polity_id);
        self.next_polity_id += 1;
        id
    }

    /// Generate a new unique RulerId
    pub fn next_ruler_id(&mut self) -> RulerId {
        let id = RulerId(self.next_ruler_id);
        self.next_ruler_id += 1;
        id
    }

    /// Get a ruler by ID
    pub fn get_ruler(&self, id: RulerId) -> Option<&Ruler> {
        self.rulers.get(&id)
    }

    /// Get a mutable ruler by ID
    pub fn get_ruler_mut(&mut self, id: RulerId) -> Option<&mut Ruler> {
        self.rulers.get_mut(&id)
    }

    pub fn get_region(&self, id: u32) -> Option<&Region> {
        self.regions.get(id as usize)
    }

    pub fn get_polity(&self, id: u32) -> Option<&Polity> {
        self.polities.iter().find(|p| p.id == PolityId(id))
    }

    pub fn get_polity_mut(&mut self, id: u32) -> Option<&mut Polity> {
        self.polities.iter_mut().find(|p| p.id == PolityId(id))
    }

    pub fn get_polity_by_polity_id(&self, id: PolityId) -> Option<&Polity> {
        self.polities.iter().find(|p| p.id == id)
    }

    pub fn get_polity_by_polity_id_mut(&mut self, id: PolityId) -> Option<&mut Polity> {
        self.polities.iter_mut().find(|p| p.id == id)
    }

    pub fn next_war_id(&self) -> u32 {
        self.active_wars.iter().map(|w| w.id).max().unwrap_or(0) + 1
    }

    /// Get neighboring polity IDs for a given polity
    /// Neighbors are polities that control regions adjacent to this polity's regions
    pub fn get_neighbors(&self, polity_id: PolityId) -> Vec<PolityId> {
        use std::collections::HashSet;

        // Find all regions controlled by this polity
        let my_regions: HashSet<u32> = self
            .regions
            .iter()
            .filter(|r| r.controller == Some(polity_id.0))
            .map(|r| r.id)
            .collect();

        if my_regions.is_empty() {
            return Vec::new();
        }

        // Find all neighboring region IDs
        let neighbor_region_ids: HashSet<u32> = my_regions
            .iter()
            .filter_map(|&region_id| self.get_region(region_id))
            .flat_map(|r| r.neighbors.iter().copied())
            .filter(|id| !my_regions.contains(id))
            .collect();

        // Find polities that control those neighboring regions
        let neighbor_polities: HashSet<PolityId> = neighbor_region_ids
            .iter()
            .filter_map(|&region_id| self.get_region(region_id))
            .filter_map(|r| r.controller.map(PolityId))
            .filter(|&pid| pid != polity_id)
            .collect();

        neighbor_polities.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::rand_core::SeedableRng;

    #[test]
    fn test_world_has_rulers() {
        let world = AggregateWorld::new(vec![], vec![], ChaCha8Rng::seed_from_u64(42));
        assert!(world.rulers.is_empty());
    }

    #[test]
    fn test_next_polity_id() {
        let mut world = AggregateWorld::new(vec![], vec![], ChaCha8Rng::seed_from_u64(42));
        let id1 = world.next_polity_id();
        let id2 = world.next_polity_id();
        assert_eq!(id1, PolityId(1));
        assert_eq!(id2, PolityId(2));
    }

    #[test]
    fn test_next_ruler_id() {
        let mut world = AggregateWorld::new(vec![], vec![], ChaCha8Rng::seed_from_u64(42));
        let id1 = world.next_ruler_id();
        let id2 = world.next_ruler_id();
        assert_eq!(id1, RulerId(1));
        assert_eq!(id2, RulerId(2));
    }
}
