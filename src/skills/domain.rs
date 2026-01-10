//! Skill domains for chunk categorization

use serde::{Deserialize, Serialize};

/// Domain categories for skill chunks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChunkDomain {
    /// Combat: melee, ranged, defensive maneuvers
    Combat,
    /// Craft: smithing, carpentry, tailoring, etc.
    Craft,
    /// Social: persuasion, negotiation, deception
    Social,
    /// Medicine: wound care, surgery, herbalism
    Medicine,
    /// Leadership: command, tactics, morale
    Leadership,
    /// Knowledge: research, teaching, languages
    Knowledge,
    /// Physical: athletics, stealth, climbing
    Physical,
}

impl ChunkDomain {
    /// Get all domains
    pub fn all() -> &'static [ChunkDomain] {
        &[
            ChunkDomain::Combat,
            ChunkDomain::Craft,
            ChunkDomain::Social,
            ChunkDomain::Medicine,
            ChunkDomain::Leadership,
            ChunkDomain::Knowledge,
            ChunkDomain::Physical,
        ]
    }

    /// Human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            ChunkDomain::Combat => "Combat",
            ChunkDomain::Craft => "Craft",
            ChunkDomain::Social => "Social",
            ChunkDomain::Medicine => "Medicine",
            ChunkDomain::Leadership => "Leadership",
            ChunkDomain::Knowledge => "Knowledge",
            ChunkDomain::Physical => "Physical",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_domains() {
        assert_eq!(ChunkDomain::all().len(), 7);
    }

    #[test]
    fn test_domain_names() {
        assert_eq!(ChunkDomain::Combat.name(), "Combat");
        assert_eq!(ChunkDomain::Craft.name(), "Craft");
    }
}
