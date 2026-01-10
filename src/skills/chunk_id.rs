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

    // === CRAFT DOMAIN ===
    // Level 1 - Micro-chunks
    CraftBasicHeatCycle,  // Heat metal to working temperature
    CraftBasicHammerWork, // Basic hammer strikes for shaping
    CraftBasicMeasure,    // Measure and mark materials
    CraftBasicCut,        // Cut materials (wood, leather, cloth)
    CraftBasicJoin,       // Join pieces (nails, stitches, glue)

    // Level 2 - Technique chunks
    CraftDrawOutMetal,  // Lengthen and thin metal through hammering
    CraftUpsetMetal,    // Thicken and shorten metal
    CraftBasicWeld,     // Forge-weld two pieces of metal
    CraftShapeWood,     // Shape wood through carving/planing
    CraftFinishSurface, // Sand, polish, apply finish

    // Level 3 - Product chunks
    CraftForgeKnife,     // Create a basic knife
    CraftForgeToolHead,  // Create tool heads (axe, hammer, etc.)
    CraftBuildFurniture, // Build basic furniture
    CraftSewGarment,     // Sew a complete garment

    // Level 4 - Complex product chunks
    CraftForgeSword,     // Create a sword
    CraftForgeArmor,     // Create armor pieces
    CraftBuildStructure, // Build structural elements
    CraftPatternWeld,    // Create pattern-welded steel

    // Level 5 - Mastery chunks
    CraftAssessAndExecute,   // Assess problem, choose approach, execute
    CraftForgeMasterwork,    // Create masterwork quality items
    CraftInnovativeTechnique, // Develop new techniques

    // === SOCIAL DOMAIN ===
    // Level 1 - Micro-chunks
    SocialActiveListening,   // Focus on speaker, absorb content
    SocialProjectConfidence, // Body language, voice tone, presence
    SocialEmpathicMirror,    // Match emotional state, build connection
    SocialCreateTension,     // Introduce discomfort, silence, pressure

    // Level 2 - Technique chunks
    SocialBuildRapport,      // Establish trust and common ground
    SocialProjectAuthority,  // Command presence and respect
    SocialReadReaction,      // Assess response, adjust approach
    SocialDeflectInquiry,    // Redirect questions, avoid commitments
    SocialEmotionalAppeal,   // Invoke emotions to persuade

    // Level 3 - Tactical chunks
    SocialNegotiateTerms,    // Reach mutually acceptable agreements
    SocialIntimidate,        // Apply pressure through fear
    SocialPersuade,          // Change minds through argument
    SocialDeceive,           // Mislead while appearing truthful
    SocialInspire,           // Motivate through vision and charisma

    // Level 4 - Strategic chunks
    SocialWorkRoom,          // Manage multiple relationships at once
    SocialPoliticalManeuver, // Navigate power structures
    SocialLeadGroup,         // Guide collective decision-making
    SocialMediateConflict,   // Resolve disputes between parties

    // Level 5 - Mastery chunks
    SocialOmniscience,       // Read entire room's dynamics instantly
    SocialManipulateDynamics, // Shape group behavior subtly
    SocialCultOfPersonality, // Build devoted following
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

            // Craft domain - all crafting chunks
            Self::CraftBasicHeatCycle
            | Self::CraftBasicHammerWork
            | Self::CraftBasicMeasure
            | Self::CraftBasicCut
            | Self::CraftBasicJoin
            | Self::CraftDrawOutMetal
            | Self::CraftUpsetMetal
            | Self::CraftBasicWeld
            | Self::CraftShapeWood
            | Self::CraftFinishSurface
            | Self::CraftForgeKnife
            | Self::CraftForgeToolHead
            | Self::CraftBuildFurniture
            | Self::CraftSewGarment
            | Self::CraftForgeSword
            | Self::CraftForgeArmor
            | Self::CraftBuildStructure
            | Self::CraftPatternWeld
            | Self::CraftAssessAndExecute
            | Self::CraftForgeMasterwork
            | Self::CraftInnovativeTechnique => ChunkDomain::Craft,

            // Social domain - all social interaction chunks
            Self::SocialActiveListening
            | Self::SocialProjectConfidence
            | Self::SocialEmpathicMirror
            | Self::SocialCreateTension
            | Self::SocialBuildRapport
            | Self::SocialProjectAuthority
            | Self::SocialReadReaction
            | Self::SocialDeflectInquiry
            | Self::SocialEmotionalAppeal
            | Self::SocialNegotiateTerms
            | Self::SocialIntimidate
            | Self::SocialPersuade
            | Self::SocialDeceive
            | Self::SocialInspire
            | Self::SocialWorkRoom
            | Self::SocialPoliticalManeuver
            | Self::SocialLeadGroup
            | Self::SocialMediateConflict
            | Self::SocialOmniscience
            | Self::SocialManipulateDynamics
            | Self::SocialCultOfPersonality => ChunkDomain::Social,
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

            // Craft Level 1
            Self::CraftBasicHeatCycle
            | Self::CraftBasicHammerWork
            | Self::CraftBasicMeasure
            | Self::CraftBasicCut
            | Self::CraftBasicJoin => 1,

            // Craft Level 2
            Self::CraftDrawOutMetal
            | Self::CraftUpsetMetal
            | Self::CraftBasicWeld
            | Self::CraftShapeWood
            | Self::CraftFinishSurface => 2,

            // Craft Level 3
            Self::CraftForgeKnife
            | Self::CraftForgeToolHead
            | Self::CraftBuildFurniture
            | Self::CraftSewGarment => 3,

            // Craft Level 4
            Self::CraftForgeSword
            | Self::CraftForgeArmor
            | Self::CraftBuildStructure
            | Self::CraftPatternWeld => 4,

            // Craft Level 5
            Self::CraftAssessAndExecute
            | Self::CraftForgeMasterwork
            | Self::CraftInnovativeTechnique => 5,

            // Social Level 1
            Self::SocialActiveListening
            | Self::SocialProjectConfidence
            | Self::SocialEmpathicMirror
            | Self::SocialCreateTension => 1,

            // Social Level 2
            Self::SocialBuildRapport
            | Self::SocialProjectAuthority
            | Self::SocialReadReaction
            | Self::SocialDeflectInquiry
            | Self::SocialEmotionalAppeal => 2,

            // Social Level 3
            Self::SocialNegotiateTerms
            | Self::SocialIntimidate
            | Self::SocialPersuade
            | Self::SocialDeceive
            | Self::SocialInspire => 3,

            // Social Level 4
            Self::SocialWorkRoom
            | Self::SocialPoliticalManeuver
            | Self::SocialLeadGroup
            | Self::SocialMediateConflict => 4,

            // Social Level 5
            Self::SocialOmniscience
            | Self::SocialManipulateDynamics
            | Self::SocialCultOfPersonality => 5,
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
            // Craft Level 1
            Self::CraftBasicHeatCycle => "Basic Heat Cycle",
            Self::CraftBasicHammerWork => "Basic Hammer Work",
            Self::CraftBasicMeasure => "Basic Measure",
            Self::CraftBasicCut => "Basic Cut",
            Self::CraftBasicJoin => "Basic Join",
            // Craft Level 2
            Self::CraftDrawOutMetal => "Draw Out Metal",
            Self::CraftUpsetMetal => "Upset Metal",
            Self::CraftBasicWeld => "Basic Weld",
            Self::CraftShapeWood => "Shape Wood",
            Self::CraftFinishSurface => "Finish Surface",
            // Craft Level 3
            Self::CraftForgeKnife => "Forge Knife",
            Self::CraftForgeToolHead => "Forge Tool Head",
            Self::CraftBuildFurniture => "Build Furniture",
            Self::CraftSewGarment => "Sew Garment",
            // Craft Level 4
            Self::CraftForgeSword => "Forge Sword",
            Self::CraftForgeArmor => "Forge Armor",
            Self::CraftBuildStructure => "Build Structure",
            Self::CraftPatternWeld => "Pattern Weld",
            // Craft Level 5
            Self::CraftAssessAndExecute => "Assess and Execute",
            Self::CraftForgeMasterwork => "Forge Masterwork",
            Self::CraftInnovativeTechnique => "Innovative Technique",
            // Social Level 1
            Self::SocialActiveListening => "Active Listening",
            Self::SocialProjectConfidence => "Project Confidence",
            Self::SocialEmpathicMirror => "Empathic Mirror",
            Self::SocialCreateTension => "Create Tension",
            // Social Level 2
            Self::SocialBuildRapport => "Build Rapport",
            Self::SocialProjectAuthority => "Project Authority",
            Self::SocialReadReaction => "Read Reaction",
            Self::SocialDeflectInquiry => "Deflect Inquiry",
            Self::SocialEmotionalAppeal => "Emotional Appeal",
            // Social Level 3
            Self::SocialNegotiateTerms => "Negotiate Terms",
            Self::SocialIntimidate => "Intimidate",
            Self::SocialPersuade => "Persuade",
            Self::SocialDeceive => "Deceive",
            Self::SocialInspire => "Inspire",
            // Social Level 4
            Self::SocialWorkRoom => "Work Room",
            Self::SocialPoliticalManeuver => "Political Maneuver",
            Self::SocialLeadGroup => "Lead Group",
            Self::SocialMediateConflict => "Mediate Conflict",
            // Social Level 5
            Self::SocialOmniscience => "Social Omniscience",
            Self::SocialManipulateDynamics => "Manipulate Dynamics",
            Self::SocialCultOfPersonality => "Cult of Personality",
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

    #[test]
    fn test_craft_chunks_exist() {
        use crate::skills::ChunkDomain;

        // Level 1 craft chunks
        assert_eq!(ChunkId::CraftBasicHeatCycle.domain(), ChunkDomain::Craft);
        assert_eq!(ChunkId::CraftBasicHammerWork.domain(), ChunkDomain::Craft);
        assert_eq!(ChunkId::CraftBasicMeasure.domain(), ChunkDomain::Craft);

        // Level 2
        assert_eq!(ChunkId::CraftDrawOutMetal.domain(), ChunkDomain::Craft);

        // Level 3
        assert_eq!(ChunkId::CraftForgeKnife.domain(), ChunkDomain::Craft);

        // Level 4
        assert_eq!(ChunkId::CraftForgeSword.domain(), ChunkDomain::Craft);

        // Level 5
        assert_eq!(ChunkId::CraftForgeMasterwork.domain(), ChunkDomain::Craft);
    }

    #[test]
    fn test_social_chunks_exist() {
        use crate::skills::ChunkDomain;

        assert_eq!(ChunkId::SocialActiveListening.domain(), ChunkDomain::Social);
        assert_eq!(ChunkId::SocialBuildRapport.domain(), ChunkDomain::Social);
        assert_eq!(ChunkId::SocialNegotiateTerms.domain(), ChunkDomain::Social);
        assert_eq!(ChunkId::SocialWorkRoom.domain(), ChunkDomain::Social);
        assert_eq!(ChunkId::SocialManipulateDynamics.domain(), ChunkDomain::Social);
    }
}
