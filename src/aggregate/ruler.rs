//! Ruler - characters who lead polities
//!
//! Rulers are the decision-makers in the aggregate simulation.
//! They have personalities, skills, opinions, and family relationships.
//! Opinions of other polities belong to rulers, not to polities.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::core::types::{PolityId, RulerId, Species};

/// Personality traits that affect ruler behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonalityTrait {
    Ambitious,    // More likely to expand, claim titles
    Cautious,     // Less likely to declare war
    Charismatic,  // Better diplomacy
    Deceitful,    // More likely to break agreements
    Honorable,    // Keeps alliances, less likely to betray
    Warlike,      // More likely to declare war
    Greedy,       // Focus on economy
    Zealous,      // Religious/ideological focus
}

impl PersonalityTrait {
    /// Modifier to war declaration likelihood (-10 to +10)
    pub fn war_modifier(&self) -> i8 {
        match self {
            Self::Ambitious => 3,
            Self::Cautious => -5,
            Self::Charismatic => 0,
            Self::Deceitful => 1,
            Self::Honorable => -2,
            Self::Warlike => 8,
            Self::Greedy => -1,
            Self::Zealous => 4,
        }
    }

    /// Modifier to diplomatic opinion formation (-10 to +10)
    pub fn diplomacy_modifier(&self) -> i8 {
        match self {
            Self::Ambitious => -2,
            Self::Cautious => 1,
            Self::Charismatic => 5,
            Self::Deceitful => -3,
            Self::Honorable => 3,
            Self::Warlike => -4,
            Self::Greedy => 0,
            Self::Zealous => -2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_trait_affects_behavior() {
        let ambitious = PersonalityTrait::Ambitious;
        let cautious = PersonalityTrait::Cautious;

        // Ambitious increases war likelihood
        assert!(ambitious.war_modifier() > 0);
        // Cautious decreases war likelihood
        assert!(cautious.war_modifier() < 0);
    }
}
