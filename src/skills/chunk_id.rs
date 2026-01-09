//! Chunk identifiers for the hierarchical skill system

use serde::{Deserialize, Serialize};

/// Unique identifier for a skill chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChunkId {
    // Level 1 - Micro-chunks (first learning)
    BasicSwing,
    BasicBlock,
    BasicStance,

    // Level 2 - Action chunks (competent soldier)
    AttackSequence,
    DefendSequence,
    Riposte,

    // Level 3 - Tactical chunks (veteran)
    EngageMelee,
    HandleFlanking,
}

impl ChunkId {
    /// Get the hierarchy level of this chunk (1-5)
    pub fn level(&self) -> u8 {
        match self {
            Self::BasicSwing | Self::BasicBlock | Self::BasicStance => 1,
            Self::AttackSequence | Self::DefendSequence | Self::Riposte => 2,
            Self::EngageMelee | Self::HandleFlanking => 3,
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::BasicSwing => "Basic Swing",
            Self::BasicBlock => "Basic Block",
            Self::BasicStance => "Basic Stance",
            Self::AttackSequence => "Attack Sequence",
            Self::DefendSequence => "Defend Sequence",
            Self::Riposte => "Riposte",
            Self::EngageMelee => "Engage Melee",
            Self::HandleFlanking => "Handle Flanking",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_levels() {
        assert_eq!(ChunkId::BasicSwing.level(), 1);
        assert_eq!(ChunkId::AttackSequence.level(), 2);
        assert_eq!(ChunkId::EngageMelee.level(), 3);
    }

    #[test]
    fn test_chunk_names() {
        assert_eq!(ChunkId::BasicSwing.name(), "Basic Swing");
        assert_eq!(ChunkId::Riposte.name(), "Riposte");
    }
}
