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

/// Ruler skills affecting governance and war
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Skills {
    pub diplomacy: i8,    // -10 to +10: affects opinion formation, alliance success
    pub martial: i8,      // -10 to +10: affects military effectiveness
    pub stewardship: i8,  // -10 to +10: affects economic growth
    pub intrigue: i8,     // -10 to +10: affects espionage, plot success
}

impl Skills {
    /// Create new skills, clamping values to valid range
    pub fn new(diplomacy: i8, martial: i8, stewardship: i8, intrigue: i8) -> Self {
        Self {
            diplomacy: diplomacy.clamp(-10, 10),
            martial: martial.clamp(-10, 10),
            stewardship: stewardship.clamp(-10, 10),
            intrigue: intrigue.clamp(-10, 10),
        }
    }
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

/// Opinion of a ruler toward another polity
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Opinion {
    pub base_value: i16,                  // -100 to +100
    pub trust: i8,                        // -10 to +10 (separate from liking)
    pub modifiers: Vec<OpinionModifier>,  // Temporary modifiers
}

/// Temporary modifier to opinion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpinionModifier {
    pub reason: String,
    pub value: i8,           // -50 to +50
    pub turns_remaining: u8,
}

impl Opinion {
    pub fn new(base_value: i16) -> Self {
        Self {
            base_value: base_value.clamp(-100, 100),
            trust: 0,
            modifiers: Vec::new(),
        }
    }

    /// Calculate effective opinion including all modifiers
    pub fn effective_value(&self) -> i16 {
        let modifier_sum: i16 = self.modifiers.iter().map(|m| m.value as i16).sum();
        (self.base_value + modifier_sum).clamp(-100, 100)
    }

    /// Add a temporary modifier
    pub fn add_modifier(&mut self, reason: &str, value: i8, turns: u8) {
        self.modifiers.push(OpinionModifier {
            reason: reason.to_string(),
            value,
            turns_remaining: turns,
        });
    }

    /// Decay modifiers by one turn, removing expired ones
    pub fn decay_modifiers(&mut self) {
        for modifier in &mut self.modifiers {
            modifier.turns_remaining = modifier.turns_remaining.saturating_sub(1);
        }
        self.modifiers.retain(|m| m.turns_remaining > 0);
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

    #[test]
    fn test_skills_default() {
        let skills = Skills::default();
        assert_eq!(skills.diplomacy, 0);
        assert_eq!(skills.martial, 0);
        assert_eq!(skills.stewardship, 0);
        assert_eq!(skills.intrigue, 0);
    }

    #[test]
    fn test_skills_clamped() {
        let skills = Skills::new(15, -15, 5, -5);
        // Values should be clamped to -10..=10
        assert_eq!(skills.diplomacy, 10);
        assert_eq!(skills.martial, -10);
        assert_eq!(skills.stewardship, 5);
        assert_eq!(skills.intrigue, -5);
    }

    #[test]
    fn test_opinion_effective_value() {
        let mut opinion = Opinion::new(-20);
        assert_eq!(opinion.effective_value(), -20);

        // Add a positive modifier
        opinion.add_modifier("trade_agreement", 15, 10);
        assert_eq!(opinion.effective_value(), -5); // -20 + 15 = -5
    }

    #[test]
    fn test_opinion_decay_modifiers() {
        let mut opinion = Opinion::new(0);
        opinion.add_modifier("recent_gift", 10, 2);

        opinion.decay_modifiers();
        assert_eq!(opinion.modifiers.len(), 1);
        assert_eq!(opinion.modifiers[0].turns_remaining, 1);

        opinion.decay_modifiers();
        assert_eq!(opinion.modifiers.len(), 0); // Expired
    }
}
