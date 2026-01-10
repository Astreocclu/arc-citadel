//! Chunk identifiers for the hierarchical skill system

use serde::{Deserialize, Serialize};

use crate::skills::ChunkDomain;

/// Unique identifier for a skill chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChunkId {
    // === MELEE ===
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

    // === RANGED ===
    // Level 1 - Micro-chunks
    DrawBow,      // Physical act of drawing bowstring
    LoadCrossbow, // Spanning/winding crossbow mechanism
    BasicAim,     // Visual focus on target
    BasicThrow,   // Throwing motion fundamentals

    // Level 2 - Action chunks
    LooseArrow,   // Draw + Aim + Release (standard bow shot)
    CrossbowShot, // Aim + Trigger (crossbow shot when loaded)
    AimedThrow,   // Aim + Throw (accurate thrown weapon)
    SnapShot,     // Quick bow shot, less accurate

    // Level 3 - Tactical chunks
    RapidFire,   // Multiple arrows in quick succession
    SniperShot,  // Maximum precision, high cost
    VolleyFire,  // Coordinated area fire
    PartingShot, // Fire while retreating (horse archers)
}

impl ChunkId {
    /// Get the domain this chunk belongs to
    pub const fn domain(&self) -> ChunkDomain {
        match self {
            // Combat domain - all melee and ranged chunks
            Self::BasicSwing
            | Self::BasicBlock
            | Self::BasicStance
            | Self::AttackSequence
            | Self::DefendSequence
            | Self::Riposte
            | Self::EngageMelee
            | Self::HandleFlanking
            | Self::DrawBow
            | Self::LoadCrossbow
            | Self::BasicAim
            | Self::BasicThrow
            | Self::LooseArrow
            | Self::CrossbowShot
            | Self::AimedThrow
            | Self::SnapShot
            | Self::RapidFire
            | Self::SniperShot
            | Self::VolleyFire
            | Self::PartingShot => ChunkDomain::Combat,
        }
    }

    /// Get the hierarchy level of this chunk (1-5)
    pub fn level(&self) -> u8 {
        match self {
            // Melee Level 1
            Self::BasicSwing | Self::BasicBlock | Self::BasicStance => 1,
            // Ranged Level 1
            Self::DrawBow | Self::LoadCrossbow | Self::BasicAim | Self::BasicThrow => 1,

            // Melee Level 2
            Self::AttackSequence | Self::DefendSequence | Self::Riposte => 2,
            // Ranged Level 2
            Self::LooseArrow | Self::CrossbowShot | Self::AimedThrow | Self::SnapShot => 2,

            // Melee Level 3
            Self::EngageMelee | Self::HandleFlanking => 3,
            // Ranged Level 3
            Self::RapidFire | Self::SniperShot | Self::VolleyFire | Self::PartingShot => 3,
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            // Melee
            Self::BasicSwing => "Basic Swing",
            Self::BasicBlock => "Basic Block",
            Self::BasicStance => "Basic Stance",
            Self::AttackSequence => "Attack Sequence",
            Self::DefendSequence => "Defend Sequence",
            Self::Riposte => "Riposte",
            Self::EngageMelee => "Engage Melee",
            Self::HandleFlanking => "Handle Flanking",
            // Ranged
            Self::DrawBow => "Draw Bow",
            Self::LoadCrossbow => "Load Crossbow",
            Self::BasicAim => "Basic Aim",
            Self::BasicThrow => "Basic Throw",
            Self::LooseArrow => "Loose Arrow",
            Self::CrossbowShot => "Crossbow Shot",
            Self::AimedThrow => "Aimed Throw",
            Self::SnapShot => "Snap Shot",
            Self::RapidFire => "Rapid Fire",
            Self::SniperShot => "Sniper Shot",
            Self::VolleyFire => "Volley Fire",
            Self::PartingShot => "Parting Shot",
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

    #[test]
    fn test_ranged_chunk_levels() {
        // Level 1 ranged
        assert_eq!(ChunkId::DrawBow.level(), 1);
        assert_eq!(ChunkId::LoadCrossbow.level(), 1);
        assert_eq!(ChunkId::BasicAim.level(), 1);
        assert_eq!(ChunkId::BasicThrow.level(), 1);

        // Level 2 ranged
        assert_eq!(ChunkId::LooseArrow.level(), 2);
        assert_eq!(ChunkId::CrossbowShot.level(), 2);
        assert_eq!(ChunkId::AimedThrow.level(), 2);

        // Level 3 ranged
        assert_eq!(ChunkId::RapidFire.level(), 3);
        assert_eq!(ChunkId::SniperShot.level(), 3);
        assert_eq!(ChunkId::VolleyFire.level(), 3);
    }

    #[test]
    fn test_ranged_chunk_names() {
        assert_eq!(ChunkId::DrawBow.name(), "Draw Bow");
        assert_eq!(ChunkId::LooseArrow.name(), "Loose Arrow");
        assert_eq!(ChunkId::RapidFire.name(), "Rapid Fire");
    }

    #[test]
    fn test_chunk_domains() {
        use crate::skills::ChunkDomain;

        // Combat chunks - melee
        assert_eq!(ChunkId::BasicSwing.domain(), ChunkDomain::Combat);
        assert_eq!(ChunkId::HandleFlanking.domain(), ChunkDomain::Combat);

        // Combat chunks - ranged
        assert_eq!(ChunkId::DrawBow.domain(), ChunkDomain::Combat);
        assert_eq!(ChunkId::RapidFire.domain(), ChunkDomain::Combat);
    }
}
