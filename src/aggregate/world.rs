//! AggregateWorld - the main world state container

use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::aggregate::region::Region;
use crate::aggregate::polity::Polity;
use crate::core::types::PolityId;

/// The aggregate world state for history simulation
pub struct AggregateWorld {
    /// All regions (pseudo-nodes) in the world
    pub regions: Vec<Region>,
    /// All polities (nations/tribes/holds/groves)
    pub polities: Vec<Polity>,
    /// Currently active wars
    pub active_wars: Vec<War>,
    /// Current simulation year
    pub year: u32,
    /// Random number generator (deterministic)
    pub rng: ChaCha8Rng,
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
        Self {
            regions,
            polities,
            active_wars: Vec::new(),
            year: 0,
            rng,
        }
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
}
