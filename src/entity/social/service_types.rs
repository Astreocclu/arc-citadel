//! Service types and trait indicators for expectation-based social dynamics
//!
//! ServiceType categorizes what kind of service an entity provides.
//! TraitIndicator represents observable behavioral traits.

use crate::actions::catalog::ActionId;
use serde::{Deserialize, Serialize};

/// Types of services an entity might provide
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceType {
    Crafting,   // Making things
    Trading,    // Buying/selling
    Helping,    // Assisting others
    Labor,      // General work (gather, build)
    Protection, // Guarding, defending
    Teaching,   // Instruction
    Healing,    // Medical care
}

impl ServiceType {
    pub fn from_action(action: ActionId) -> Option<Self> {
        match action {
            ActionId::Craft => Some(ServiceType::Crafting),
            ActionId::Trade => Some(ServiceType::Trading),
            ActionId::Help => Some(ServiceType::Helping),
            ActionId::Gather | ActionId::Build | ActionId::Repair => Some(ServiceType::Labor),
            ActionId::Defend | ActionId::HoldPosition => Some(ServiceType::Protection),
            // Teaching and Healing would map to future actions
            _ => None,
        }
    }
}

/// Observable behavioral traits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraitIndicator {
    Reliable,   // Does what they say
    Unreliable, // Frequently fails commitments
    Generous,   // Gives beyond obligation
    Stingy,     // Minimal compliance
    Aggressive, // Quick to violence
    Peaceful,   // Avoids conflict
    Punctual,   // On time
    Late,       // Frequently tardy
}

impl TraitIndicator {
    pub fn from_action(action: ActionId) -> Option<Self> {
        match action {
            ActionId::Help => Some(TraitIndicator::Generous),
            ActionId::Flee => Some(TraitIndicator::Peaceful),
            ActionId::Attack | ActionId::Charge => Some(TraitIndicator::Aggressive),
            ActionId::Defend | ActionId::HoldPosition => Some(TraitIndicator::Reliable),
            _ => None,
        }
    }

    /// Returns the opposite trait (for violation detection)
    pub fn opposite(&self) -> Self {
        match self {
            TraitIndicator::Reliable => TraitIndicator::Unreliable,
            TraitIndicator::Unreliable => TraitIndicator::Reliable,
            TraitIndicator::Generous => TraitIndicator::Stingy,
            TraitIndicator::Stingy => TraitIndicator::Generous,
            TraitIndicator::Aggressive => TraitIndicator::Peaceful,
            TraitIndicator::Peaceful => TraitIndicator::Aggressive,
            TraitIndicator::Punctual => TraitIndicator::Late,
            TraitIndicator::Late => TraitIndicator::Punctual,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::catalog::ActionId;

    #[test]
    fn test_service_from_action() {
        assert_eq!(
            ServiceType::from_action(ActionId::Craft),
            Some(ServiceType::Crafting)
        );
        assert_eq!(
            ServiceType::from_action(ActionId::Trade),
            Some(ServiceType::Trading)
        );
        assert_eq!(
            ServiceType::from_action(ActionId::Help),
            Some(ServiceType::Helping)
        );
        assert_eq!(
            ServiceType::from_action(ActionId::Gather),
            Some(ServiceType::Labor)
        );
        assert_eq!(
            ServiceType::from_action(ActionId::Build),
            Some(ServiceType::Labor)
        );
        assert_eq!(
            ServiceType::from_action(ActionId::Repair),
            Some(ServiceType::Labor)
        );
        assert_eq!(
            ServiceType::from_action(ActionId::Defend),
            Some(ServiceType::Protection)
        );
        assert_eq!(
            ServiceType::from_action(ActionId::HoldPosition),
            Some(ServiceType::Protection)
        );
        assert_eq!(ServiceType::from_action(ActionId::MoveTo), None); // Not a service
    }

    #[test]
    fn test_trait_from_action() {
        assert_eq!(
            TraitIndicator::from_action(ActionId::Help),
            Some(TraitIndicator::Generous)
        );
        assert_eq!(
            TraitIndicator::from_action(ActionId::Flee),
            Some(TraitIndicator::Peaceful)
        );
        assert_eq!(
            TraitIndicator::from_action(ActionId::Attack),
            Some(TraitIndicator::Aggressive)
        );
        assert_eq!(
            TraitIndicator::from_action(ActionId::Charge),
            Some(TraitIndicator::Aggressive)
        );
        assert_eq!(
            TraitIndicator::from_action(ActionId::Defend),
            Some(TraitIndicator::Reliable)
        );
        assert_eq!(
            TraitIndicator::from_action(ActionId::HoldPosition),
            Some(TraitIndicator::Reliable)
        );
        assert_eq!(TraitIndicator::from_action(ActionId::MoveTo), None); // Not a trait indicator
    }

    #[test]
    fn test_trait_opposite() {
        assert_eq!(
            TraitIndicator::Reliable.opposite(),
            TraitIndicator::Unreliable
        );
        assert_eq!(
            TraitIndicator::Unreliable.opposite(),
            TraitIndicator::Reliable
        );
        assert_eq!(TraitIndicator::Generous.opposite(), TraitIndicator::Stingy);
        assert_eq!(TraitIndicator::Stingy.opposite(), TraitIndicator::Generous);
        assert_eq!(
            TraitIndicator::Aggressive.opposite(),
            TraitIndicator::Peaceful
        );
        assert_eq!(
            TraitIndicator::Peaceful.opposite(),
            TraitIndicator::Aggressive
        );
        assert_eq!(TraitIndicator::Punctual.opposite(), TraitIndicator::Late);
        assert_eq!(TraitIndicator::Late.opposite(), TraitIndicator::Punctual);
    }
}
