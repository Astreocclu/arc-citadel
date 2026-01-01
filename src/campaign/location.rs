//! Location - a place on the campaign map
//!
//! Locations are the nodes in the campaign map graph.
//! They have a controller (polity that owns them) and various properties.

use serde::{Deserialize, Serialize};
use crate::core::types::{LocationId, PolityId};

/// A location on the campaign map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub controller: Option<PolityId>,  // Which polity controls this location
    pub population: u32,
    pub fortification: u8,  // 0-10, affects siege difficulty
}

impl Location {
    pub fn new(id: LocationId, name: String) -> Self {
        Self {
            id,
            name,
            controller: None,
            population: 0,
            fortification: 0,
        }
    }

    /// Transfer control to a new polity
    pub fn transfer_control(&mut self, new_controller: Option<PolityId>) {
        self.controller = new_controller;
    }

    /// Check if controlled by a specific polity
    pub fn is_controlled_by(&self, polity: PolityId) -> bool {
        self.controller == Some(polity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_controller() {
        let mut loc = Location::new(LocationId(1), "Castle".to_string());
        assert!(loc.controller.is_none());

        loc.transfer_control(Some(PolityId(1)));
        assert!(loc.is_controlled_by(PolityId(1)));

        loc.transfer_control(Some(PolityId(2)));
        assert!(!loc.is_controlled_by(PolityId(1)));
        assert!(loc.is_controlled_by(PolityId(2)));
    }
}
