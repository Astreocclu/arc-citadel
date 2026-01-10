//! Static chunk definitions - the global library all entities reference

use crate::skills::{ChunkId, ContextTag};

/// What a chunk contains
#[derive(Debug, Clone)]
pub enum ChunkComponents {
    /// Level 0: Cannot be decomposed further (internal)
    Atomic,
    /// Level 1+: Composed of other chunks
    Composite(&'static [ChunkId]),
}

/// Definition of a skill chunk
#[derive(Debug, Clone)]
pub struct ChunkDefinition {
    pub id: ChunkId,
    pub name: &'static str,
    pub level: u8,
    pub components: ChunkComponents,
    pub context_requirements: &'static [ContextTag],
    pub prerequisite_chunks: &'static [ChunkId],
    /// Base number of repetitions to form this chunk
    pub base_repetitions: u32,
}

/// Global chunk library - static definitions
pub static CHUNK_LIBRARY: &[ChunkDefinition] = &[
    // Level 1 - Micro-chunks
    ChunkDefinition {
        id: ChunkId::BasicSwing,
        name: "Basic Swing",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[ContextTag::InMelee],
        prerequisite_chunks: &[],
        base_repetitions: 20,
    },
    ChunkDefinition {
        id: ChunkId::BasicBlock,
        name: "Basic Block",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[ContextTag::InMelee, ContextTag::EnemyVisible],
        prerequisite_chunks: &[],
        base_repetitions: 20,
    },
    ChunkDefinition {
        id: ChunkId::BasicStance,
        name: "Basic Stance",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 10,
    },
    // Level 2 - Action chunks
    ChunkDefinition {
        id: ChunkId::AttackSequence,
        name: "Attack Sequence",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::BasicStance, ChunkId::BasicSwing]),
        context_requirements: &[ContextTag::InMelee],
        prerequisite_chunks: &[ChunkId::BasicStance, ChunkId::BasicSwing],
        base_repetitions: 50,
    },
    ChunkDefinition {
        id: ChunkId::DefendSequence,
        name: "Defend Sequence",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::BasicBlock, ChunkId::BasicStance]),
        context_requirements: &[ContextTag::InMelee, ContextTag::EnemyVisible],
        prerequisite_chunks: &[ChunkId::BasicBlock, ChunkId::BasicStance],
        base_repetitions: 50,
    },
    ChunkDefinition {
        id: ChunkId::Riposte,
        name: "Riposte",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::BasicBlock, ChunkId::BasicSwing]),
        context_requirements: &[ContextTag::InMelee, ContextTag::EnemyVisible],
        prerequisite_chunks: &[ChunkId::BasicBlock, ChunkId::BasicSwing],
        base_repetitions: 80,
    },
    // Level 3 - Tactical chunks
    ChunkDefinition {
        id: ChunkId::EngageMelee,
        name: "Engage Melee",
        level: 3,
        components: ChunkComponents::Composite(&[ChunkId::AttackSequence, ChunkId::DefendSequence]),
        context_requirements: &[ContextTag::InMelee],
        prerequisite_chunks: &[ChunkId::AttackSequence, ChunkId::DefendSequence],
        base_repetitions: 200,
    },
    ChunkDefinition {
        id: ChunkId::HandleFlanking,
        name: "Handle Flanking",
        level: 3,
        components: ChunkComponents::Composite(&[ChunkId::EngageMelee]),
        context_requirements: &[ContextTag::InMelee, ContextTag::MultipleEnemies],
        prerequisite_chunks: &[ChunkId::EngageMelee],
        base_repetitions: 300,
    },
    // === RANGED DOMAIN ===
    // Level 1 - Micro-chunks (Atomic)
    ChunkDefinition {
        id: ChunkId::DrawBow,
        name: "Draw Bow",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[ContextTag::HasBow, ContextTag::AtRange],
        prerequisite_chunks: &[],
        base_repetitions: 30,
    },
    ChunkDefinition {
        id: ChunkId::LoadCrossbow,
        name: "Load Crossbow",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[ContextTag::HasCrossbow],
        prerequisite_chunks: &[],
        base_repetitions: 15,
    },
    ChunkDefinition {
        id: ChunkId::BasicAim,
        name: "Basic Aim",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[ContextTag::TargetVisible],
        prerequisite_chunks: &[],
        base_repetitions: 20,
    },
    ChunkDefinition {
        id: ChunkId::BasicThrow,
        name: "Basic Throw",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[ContextTag::HasThrown, ContextTag::AtRange],
        prerequisite_chunks: &[],
        base_repetitions: 15,
    },
    // Level 2 - Action chunks (Composite)
    ChunkDefinition {
        id: ChunkId::LooseArrow,
        name: "Loose Arrow",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::DrawBow, ChunkId::BasicAim]),
        context_requirements: &[
            ContextTag::HasBow,
            ContextTag::AtRange,
            ContextTag::TargetVisible,
            ContextTag::AmmoAvailable,
        ],
        prerequisite_chunks: &[ChunkId::DrawBow, ChunkId::BasicAim],
        base_repetitions: 80,
    },
    ChunkDefinition {
        id: ChunkId::CrossbowShot,
        name: "Crossbow Shot",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::BasicAim]),
        context_requirements: &[
            ContextTag::HasCrossbow,
            ContextTag::CrossbowLoaded,
            ContextTag::AtRange,
            ContextTag::TargetVisible,
        ],
        prerequisite_chunks: &[ChunkId::BasicAim],
        base_repetitions: 40,
    },
    ChunkDefinition {
        id: ChunkId::AimedThrow,
        name: "Aimed Throw",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::BasicThrow, ChunkId::BasicAim]),
        context_requirements: &[
            ContextTag::HasThrown,
            ContextTag::AtRange,
            ContextTag::TargetVisible,
        ],
        prerequisite_chunks: &[ChunkId::BasicThrow, ChunkId::BasicAim],
        base_repetitions: 50,
    },
    ChunkDefinition {
        id: ChunkId::SnapShot,
        name: "Snap Shot",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::DrawBow]),
        context_requirements: &[
            ContextTag::HasBow,
            ContextTag::AtRange,
            ContextTag::AmmoAvailable,
        ],
        prerequisite_chunks: &[ChunkId::DrawBow],
        base_repetitions: 60,
    },
    // Level 3 - Tactical chunks
    ChunkDefinition {
        id: ChunkId::RapidFire,
        name: "Rapid Fire",
        level: 3,
        components: ChunkComponents::Composite(&[ChunkId::LooseArrow, ChunkId::SnapShot]),
        context_requirements: &[
            ContextTag::HasBow,
            ContextTag::AtRange,
            ContextTag::AmmoAvailable,
            ContextTag::TargetVisible,
        ],
        prerequisite_chunks: &[ChunkId::LooseArrow, ChunkId::SnapShot],
        base_repetitions: 200,
    },
    ChunkDefinition {
        id: ChunkId::SniperShot,
        name: "Sniper Shot",
        level: 3,
        components: ChunkComponents::Composite(&[ChunkId::LooseArrow]),
        context_requirements: &[
            ContextTag::HasBow,
            ContextTag::AtRange,
            ContextTag::AmmoAvailable,
            ContextTag::TargetVisible,
        ],
        prerequisite_chunks: &[ChunkId::LooseArrow],
        base_repetitions: 250,
    },
    ChunkDefinition {
        id: ChunkId::VolleyFire,
        name: "Volley Fire",
        level: 3,
        components: ChunkComponents::Composite(&[ChunkId::LooseArrow]),
        context_requirements: &[
            ContextTag::HasBow,
            ContextTag::AtRange,
            ContextTag::AmmoAvailable,
        ],
        prerequisite_chunks: &[ChunkId::LooseArrow],
        base_repetitions: 150,
    },
    ChunkDefinition {
        id: ChunkId::PartingShot,
        name: "Parting Shot",
        level: 3,
        components: ChunkComponents::Composite(&[ChunkId::SnapShot]),
        context_requirements: &[
            ContextTag::HasBow,
            ContextTag::AtRange,
            ContextTag::AmmoAvailable,
        ],
        prerequisite_chunks: &[ChunkId::SnapShot],
        base_repetitions: 180,
    },
    // === CRAFT DOMAIN ===
    // Level 1 - Micro-chunks (Atomic)
    ChunkDefinition {
        id: ChunkId::CraftBasicHeatCycle,
        name: "Basic Heat Cycle",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 15,
    },
    ChunkDefinition {
        id: ChunkId::CraftBasicHammerWork,
        name: "Basic Hammer Work",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 20,
    },
    ChunkDefinition {
        id: ChunkId::CraftBasicMeasure,
        name: "Basic Measure",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 10,
    },
    ChunkDefinition {
        id: ChunkId::CraftBasicCut,
        name: "Basic Cut",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 15,
    },
    ChunkDefinition {
        id: ChunkId::CraftBasicJoin,
        name: "Basic Join",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 15,
    },
    // Level 2 - Technique chunks (Composite)
    ChunkDefinition {
        id: ChunkId::CraftDrawOutMetal,
        name: "Draw Out Metal",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::CraftBasicHeatCycle,
            ChunkId::CraftBasicHammerWork,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::CraftBasicHeatCycle, ChunkId::CraftBasicHammerWork],
        base_repetitions: 40,
    },
    ChunkDefinition {
        id: ChunkId::CraftUpsetMetal,
        name: "Upset Metal",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::CraftBasicHeatCycle,
            ChunkId::CraftBasicHammerWork,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::CraftBasicHeatCycle, ChunkId::CraftBasicHammerWork],
        base_repetitions: 40,
    },
    ChunkDefinition {
        id: ChunkId::CraftBasicWeld,
        name: "Basic Weld",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::CraftBasicHeatCycle,
            ChunkId::CraftBasicHammerWork,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::CraftBasicHeatCycle, ChunkId::CraftBasicHammerWork],
        base_repetitions: 60,
    },
    ChunkDefinition {
        id: ChunkId::CraftShapeWood,
        name: "Shape Wood",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::CraftBasicMeasure,
            ChunkId::CraftBasicCut,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::CraftBasicMeasure, ChunkId::CraftBasicCut],
        base_repetitions: 35,
    },
    ChunkDefinition {
        id: ChunkId::CraftFinishSurface,
        name: "Finish Surface",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::CraftBasicMeasure]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::CraftBasicMeasure],
        base_repetitions: 25,
    },
    // Level 3 - Product chunks
    ChunkDefinition {
        id: ChunkId::CraftForgeKnife,
        name: "Forge Knife",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::CraftDrawOutMetal,
            ChunkId::CraftBasicMeasure,
            ChunkId::CraftFinishSurface,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::CraftDrawOutMetal,
            ChunkId::CraftBasicMeasure,
            ChunkId::CraftFinishSurface,
        ],
        base_repetitions: 100,
    },
    ChunkDefinition {
        id: ChunkId::CraftForgeToolHead,
        name: "Forge Tool Head",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::CraftDrawOutMetal,
            ChunkId::CraftUpsetMetal,
            ChunkId::CraftBasicMeasure,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::CraftDrawOutMetal,
            ChunkId::CraftUpsetMetal,
            ChunkId::CraftBasicMeasure,
        ],
        base_repetitions: 120,
    },
    ChunkDefinition {
        id: ChunkId::CraftBuildFurniture,
        name: "Build Furniture",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::CraftShapeWood,
            ChunkId::CraftBasicJoin,
            ChunkId::CraftFinishSurface,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::CraftShapeWood,
            ChunkId::CraftBasicJoin,
            ChunkId::CraftFinishSurface,
        ],
        base_repetitions: 150,
    },
    ChunkDefinition {
        id: ChunkId::CraftSewGarment,
        name: "Sew Garment",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::CraftBasicMeasure,
            ChunkId::CraftBasicCut,
            ChunkId::CraftBasicJoin,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::CraftBasicMeasure,
            ChunkId::CraftBasicCut,
            ChunkId::CraftBasicJoin,
        ],
        base_repetitions: 80,
    },
    // Level 4 - Complex product chunks
    ChunkDefinition {
        id: ChunkId::CraftForgeSword,
        name: "Forge Sword",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::CraftForgeKnife,
            ChunkId::CraftDrawOutMetal,
            ChunkId::CraftBasicWeld,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::CraftForgeKnife,
            ChunkId::CraftDrawOutMetal,
            ChunkId::CraftBasicWeld,
        ],
        base_repetitions: 300,
    },
    ChunkDefinition {
        id: ChunkId::CraftForgeArmor,
        name: "Forge Armor",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::CraftForgeToolHead,
            ChunkId::CraftDrawOutMetal,
            ChunkId::CraftBasicWeld,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::CraftForgeToolHead,
            ChunkId::CraftDrawOutMetal,
            ChunkId::CraftBasicWeld,
        ],
        base_repetitions: 400,
    },
    ChunkDefinition {
        id: ChunkId::CraftBuildStructure,
        name: "Build Structure",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::CraftBuildFurniture,
            ChunkId::CraftShapeWood,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::CraftBuildFurniture, ChunkId::CraftShapeWood],
        base_repetitions: 500,
    },
    ChunkDefinition {
        id: ChunkId::CraftPatternWeld,
        name: "Pattern Weld",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::CraftBasicWeld,
            ChunkId::CraftDrawOutMetal,
            ChunkId::CraftForgeKnife,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::CraftBasicWeld,
            ChunkId::CraftDrawOutMetal,
            ChunkId::CraftForgeKnife,
        ],
        base_repetitions: 350,
    },
    // Level 5 - Mastery chunks
    ChunkDefinition {
        id: ChunkId::CraftAssessAndExecute,
        name: "Assess and Execute",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::CraftForgeSword,
            ChunkId::CraftForgeArmor,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::CraftForgeSword, ChunkId::CraftForgeArmor],
        base_repetitions: 800,
    },
    ChunkDefinition {
        id: ChunkId::CraftForgeMasterwork,
        name: "Forge Masterwork",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::CraftForgeSword,
            ChunkId::CraftPatternWeld,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::CraftForgeSword, ChunkId::CraftPatternWeld],
        base_repetitions: 1000,
    },
    ChunkDefinition {
        id: ChunkId::CraftInnovativeTechnique,
        name: "Innovative Technique",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::CraftAssessAndExecute,
            ChunkId::CraftForgeMasterwork,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::CraftAssessAndExecute,
            ChunkId::CraftForgeMasterwork,
        ],
        base_repetitions: 1500,
    },
    // === SOCIAL DOMAIN ===
    // Level 1 - Micro-chunks (Atomic)
    ChunkDefinition {
        id: ChunkId::SocialActiveListening,
        name: "Active Listening",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 20,
    },
    ChunkDefinition {
        id: ChunkId::SocialProjectConfidence,
        name: "Project Confidence",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 25,
    },
    ChunkDefinition {
        id: ChunkId::SocialEmpathicMirror,
        name: "Empathic Mirror",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 20,
    },
    ChunkDefinition {
        id: ChunkId::SocialCreateTension,
        name: "Create Tension",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 15,
    },
    // Level 2 - Technique chunks (Composite)
    ChunkDefinition {
        id: ChunkId::SocialBuildRapport,
        name: "Build Rapport",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialActiveListening,
            ChunkId::SocialEmpathicMirror,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::SocialActiveListening,
            ChunkId::SocialEmpathicMirror,
        ],
        base_repetitions: 50,
    },
    ChunkDefinition {
        id: ChunkId::SocialProjectAuthority,
        name: "Project Authority",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialProjectConfidence,
            ChunkId::SocialCreateTension,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::SocialProjectConfidence,
            ChunkId::SocialCreateTension,
        ],
        base_repetitions: 45,
    },
    ChunkDefinition {
        id: ChunkId::SocialReadReaction,
        name: "Read Reaction",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialActiveListening,
            ChunkId::SocialEmpathicMirror,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::SocialActiveListening,
            ChunkId::SocialEmpathicMirror,
        ],
        base_repetitions: 40,
    },
    ChunkDefinition {
        id: ChunkId::SocialDeflectInquiry,
        name: "Deflect Inquiry",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialProjectConfidence,
            ChunkId::SocialActiveListening,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::SocialProjectConfidence,
            ChunkId::SocialActiveListening,
        ],
        base_repetitions: 55,
    },
    ChunkDefinition {
        id: ChunkId::SocialEmotionalAppeal,
        name: "Emotional Appeal",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialEmpathicMirror,
            ChunkId::SocialCreateTension,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::SocialEmpathicMirror, ChunkId::SocialCreateTension],
        base_repetitions: 60,
    },
    // Level 3 - Tactical chunks
    ChunkDefinition {
        id: ChunkId::SocialNegotiateTerms,
        name: "Negotiate Terms",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialBuildRapport,
            ChunkId::SocialReadReaction,
            ChunkId::SocialProjectAuthority,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::SocialBuildRapport,
            ChunkId::SocialReadReaction,
            ChunkId::SocialProjectAuthority,
        ],
        base_repetitions: 120,
    },
    ChunkDefinition {
        id: ChunkId::SocialIntimidate,
        name: "Intimidate",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialProjectAuthority,
            ChunkId::SocialCreateTension,
            ChunkId::SocialReadReaction,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::SocialProjectAuthority,
            ChunkId::SocialCreateTension,
            ChunkId::SocialReadReaction,
        ],
        base_repetitions: 100,
    },
    ChunkDefinition {
        id: ChunkId::SocialPersuade,
        name: "Persuade",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialBuildRapport,
            ChunkId::SocialEmotionalAppeal,
            ChunkId::SocialReadReaction,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::SocialBuildRapport,
            ChunkId::SocialEmotionalAppeal,
            ChunkId::SocialReadReaction,
        ],
        base_repetitions: 150,
    },
    ChunkDefinition {
        id: ChunkId::SocialDeceive,
        name: "Deceive",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialProjectConfidence,
            ChunkId::SocialDeflectInquiry,
            ChunkId::SocialReadReaction,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::SocialProjectConfidence,
            ChunkId::SocialDeflectInquiry,
            ChunkId::SocialReadReaction,
        ],
        base_repetitions: 130,
    },
    ChunkDefinition {
        id: ChunkId::SocialInspire,
        name: "Inspire",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialProjectConfidence,
            ChunkId::SocialEmotionalAppeal,
            ChunkId::SocialBuildRapport,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::SocialProjectConfidence,
            ChunkId::SocialEmotionalAppeal,
            ChunkId::SocialBuildRapport,
        ],
        base_repetitions: 80,
    },
    // Level 4 - Strategic chunks
    ChunkDefinition {
        id: ChunkId::SocialWorkRoom,
        name: "Work Room",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialNegotiateTerms,
            ChunkId::SocialPersuade,
            ChunkId::SocialReadReaction,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::SocialNegotiateTerms,
            ChunkId::SocialPersuade,
            ChunkId::SocialReadReaction,
        ],
        base_repetitions: 300,
    },
    ChunkDefinition {
        id: ChunkId::SocialPoliticalManeuver,
        name: "Political Maneuver",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialNegotiateTerms,
            ChunkId::SocialDeceive,
            ChunkId::SocialIntimidate,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::SocialNegotiateTerms,
            ChunkId::SocialDeceive,
            ChunkId::SocialIntimidate,
        ],
        base_repetitions: 350,
    },
    ChunkDefinition {
        id: ChunkId::SocialLeadGroup,
        name: "Lead Group",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialInspire,
            ChunkId::SocialProjectAuthority,
            ChunkId::SocialNegotiateTerms,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::SocialInspire,
            ChunkId::SocialProjectAuthority,
            ChunkId::SocialNegotiateTerms,
        ],
        base_repetitions: 250,
    },
    ChunkDefinition {
        id: ChunkId::SocialMediateConflict,
        name: "Mediate Conflict",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialNegotiateTerms,
            ChunkId::SocialBuildRapport,
            ChunkId::SocialPersuade,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::SocialNegotiateTerms,
            ChunkId::SocialBuildRapport,
            ChunkId::SocialPersuade,
        ],
        base_repetitions: 200,
    },
    // Level 5 - Mastery chunks
    ChunkDefinition {
        id: ChunkId::SocialOmniscience,
        name: "Social Omniscience",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialWorkRoom,
            ChunkId::SocialReadReaction,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::SocialWorkRoom, ChunkId::SocialReadReaction],
        base_repetitions: 700,
    },
    ChunkDefinition {
        id: ChunkId::SocialManipulateDynamics,
        name: "Manipulate Dynamics",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialPoliticalManeuver,
            ChunkId::SocialWorkRoom,
            ChunkId::SocialDeceive,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::SocialPoliticalManeuver,
            ChunkId::SocialWorkRoom,
            ChunkId::SocialDeceive,
        ],
        base_repetitions: 1000,
    },
    ChunkDefinition {
        id: ChunkId::SocialCultOfPersonality,
        name: "Cult of Personality",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::SocialLeadGroup,
            ChunkId::SocialInspire,
            ChunkId::SocialManipulateDynamics,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::SocialLeadGroup,
            ChunkId::SocialInspire,
            ChunkId::SocialManipulateDynamics,
        ],
        base_repetitions: 800,
    },
    // === MEDICINE DOMAIN ===
    // Level 1 - Micro-chunks (Atomic)
    ChunkDefinition {
        id: ChunkId::MedWoundAssessment,
        name: "Wound Assessment",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 15,
    },
    ChunkDefinition {
        id: ChunkId::MedBasicCleaning,
        name: "Basic Cleaning",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 10,
    },
    ChunkDefinition {
        id: ChunkId::MedBasicSuture,
        name: "Basic Suture",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 25,
    },
    ChunkDefinition {
        id: ChunkId::MedVitalCheck,
        name: "Vital Check",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 10,
    },
    // Level 2 - Technique chunks (Composite)
    ChunkDefinition {
        id: ChunkId::MedTreatLaceration,
        name: "Treat Laceration",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::MedWoundAssessment,
            ChunkId::MedBasicCleaning,
            ChunkId::MedBasicSuture,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::MedWoundAssessment,
            ChunkId::MedBasicCleaning,
            ChunkId::MedBasicSuture,
        ],
        base_repetitions: 50,
    },
    ChunkDefinition {
        id: ChunkId::MedSetFracture,
        name: "Set Fracture",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::MedWoundAssessment,
            ChunkId::MedVitalCheck,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::MedWoundAssessment, ChunkId::MedVitalCheck],
        base_repetitions: 60,
    },
    ChunkDefinition {
        id: ChunkId::MedPreparePoultice,
        name: "Prepare Poultice",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::MedWoundAssessment]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::MedWoundAssessment],
        base_repetitions: 40,
    },
    ChunkDefinition {
        id: ChunkId::MedDiagnoseIllness,
        name: "Diagnose Illness",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::MedVitalCheck]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::MedVitalCheck],
        base_repetitions: 55,
    },
    ChunkDefinition {
        id: ChunkId::MedPainManagement,
        name: "Pain Management",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::MedVitalCheck,
            ChunkId::MedPreparePoultice,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::MedVitalCheck],
        base_repetitions: 45,
    },
    // Level 3 - Treatment chunks
    ChunkDefinition {
        id: ChunkId::MedFieldSurgery,
        name: "Field Surgery",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::MedTreatLaceration,
            ChunkId::MedVitalCheck,
            ChunkId::MedPainManagement,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::MedTreatLaceration,
            ChunkId::MedVitalCheck,
            ChunkId::MedPainManagement,
        ],
        base_repetitions: 150,
    },
    ChunkDefinition {
        id: ChunkId::MedTreatInfection,
        name: "Treat Infection",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::MedDiagnoseIllness,
            ChunkId::MedBasicCleaning,
            ChunkId::MedPreparePoultice,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::MedDiagnoseIllness,
            ChunkId::MedBasicCleaning,
            ChunkId::MedPreparePoultice,
        ],
        base_repetitions: 120,
    },
    ChunkDefinition {
        id: ChunkId::MedDeliverBaby,
        name: "Deliver Baby",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::MedVitalCheck,
            ChunkId::MedWoundAssessment,
            ChunkId::MedPainManagement,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::MedVitalCheck,
            ChunkId::MedWoundAssessment,
            ChunkId::MedPainManagement,
        ],
        base_repetitions: 100,
    },
    ChunkDefinition {
        id: ChunkId::MedAmputation,
        name: "Amputation",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::MedFieldSurgery,
            ChunkId::MedTreatLaceration,
            ChunkId::MedPainManagement,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::MedFieldSurgery,
            ChunkId::MedTreatLaceration,
            ChunkId::MedPainManagement,
        ],
        base_repetitions: 180,
    },
    // Level 4 - Complex treatment chunks
    ChunkDefinition {
        id: ChunkId::MedBattlefieldTriage,
        name: "Battlefield Triage",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::MedWoundAssessment,
            ChunkId::MedVitalCheck,
            ChunkId::MedFieldSurgery,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::MedWoundAssessment,
            ChunkId::MedVitalCheck,
            ChunkId::MedFieldSurgery,
        ],
        base_repetitions: 300,
    },
    ChunkDefinition {
        id: ChunkId::MedComplexSurgery,
        name: "Complex Surgery",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::MedFieldSurgery,
            ChunkId::MedTreatInfection,
            ChunkId::MedAmputation,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::MedFieldSurgery,
            ChunkId::MedTreatInfection,
            ChunkId::MedAmputation,
        ],
        base_repetitions: 400,
    },
    ChunkDefinition {
        id: ChunkId::MedEpidemicResponse,
        name: "Epidemic Response",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::MedDiagnoseIllness,
            ChunkId::MedTreatInfection,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::MedDiagnoseIllness, ChunkId::MedTreatInfection],
        base_repetitions: 350,
    },
    // Level 5 - Mastery chunks
    ChunkDefinition {
        id: ChunkId::MedDiagnosticIntuition,
        name: "Diagnostic Intuition",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::MedBattlefieldTriage,
            ChunkId::MedDiagnoseIllness,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::MedBattlefieldTriage, ChunkId::MedDiagnoseIllness],
        base_repetitions: 700,
    },
    ChunkDefinition {
        id: ChunkId::MedSurgicalExcellence,
        name: "Surgical Excellence",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::MedComplexSurgery,
            ChunkId::MedFieldSurgery,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::MedComplexSurgery, ChunkId::MedFieldSurgery],
        base_repetitions: 800,
    },
    ChunkDefinition {
        id: ChunkId::MedHolisticTreatment,
        name: "Holistic Treatment",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::MedDiagnosticIntuition,
            ChunkId::MedSurgicalExcellence,
            ChunkId::MedEpidemicResponse,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::MedDiagnosticIntuition,
            ChunkId::MedSurgicalExcellence,
            ChunkId::MedEpidemicResponse,
        ],
        base_repetitions: 1000,
    },
];

/// Look up a chunk definition by ID
pub fn get_chunk_definition(id: ChunkId) -> Option<&'static ChunkDefinition> {
    CHUNK_LIBRARY.iter().find(|def| def.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_has_all_chunks() {
        // Every ChunkId variant should have a definition
        assert!(get_chunk_definition(ChunkId::BasicSwing).is_some());
        assert!(get_chunk_definition(ChunkId::BasicBlock).is_some());
        assert!(get_chunk_definition(ChunkId::BasicStance).is_some());
        assert!(get_chunk_definition(ChunkId::AttackSequence).is_some());
        assert!(get_chunk_definition(ChunkId::DefendSequence).is_some());
        assert!(get_chunk_definition(ChunkId::Riposte).is_some());
        assert!(get_chunk_definition(ChunkId::EngageMelee).is_some());
        assert!(get_chunk_definition(ChunkId::HandleFlanking).is_some());
    }

    #[test]
    fn test_levels_match_chunk_id() {
        for def in CHUNK_LIBRARY {
            assert_eq!(
                def.level,
                def.id.level(),
                "Definition level mismatch for {:?}",
                def.id
            );
        }
    }

    #[test]
    fn test_prerequisites_exist() {
        for def in CHUNK_LIBRARY {
            for prereq in def.prerequisite_chunks {
                assert!(
                    get_chunk_definition(*prereq).is_some(),
                    "Missing prerequisite {:?} for {:?}",
                    prereq,
                    def.id
                );
            }
        }
    }

    #[test]
    fn test_composite_components_exist() {
        for def in CHUNK_LIBRARY {
            if let ChunkComponents::Composite(components) = &def.components {
                for comp in *components {
                    assert!(
                        get_chunk_definition(*comp).is_some(),
                        "Missing component {:?} for {:?}",
                        comp,
                        def.id
                    );
                }
            }
        }
    }

    #[test]
    fn test_ranged_chunks_exist() {
        // All ranged chunks should have definitions
        assert!(get_chunk_definition(ChunkId::DrawBow).is_some());
        assert!(get_chunk_definition(ChunkId::LoadCrossbow).is_some());
        assert!(get_chunk_definition(ChunkId::BasicAim).is_some());
        assert!(get_chunk_definition(ChunkId::BasicThrow).is_some());
        assert!(get_chunk_definition(ChunkId::LooseArrow).is_some());
        assert!(get_chunk_definition(ChunkId::CrossbowShot).is_some());
        assert!(get_chunk_definition(ChunkId::AimedThrow).is_some());
        assert!(get_chunk_definition(ChunkId::SnapShot).is_some());
        assert!(get_chunk_definition(ChunkId::RapidFire).is_some());
        assert!(get_chunk_definition(ChunkId::SniperShot).is_some());
        assert!(get_chunk_definition(ChunkId::VolleyFire).is_some());
        assert!(get_chunk_definition(ChunkId::PartingShot).is_some());
    }

    #[test]
    fn test_ranged_prerequisites() {
        // LooseArrow requires DrawBow and BasicAim
        let def = get_chunk_definition(ChunkId::LooseArrow).unwrap();
        assert!(def.prerequisite_chunks.contains(&ChunkId::DrawBow));
        assert!(def.prerequisite_chunks.contains(&ChunkId::BasicAim));

        // CrossbowShot only requires BasicAim (loading is separate)
        let def = get_chunk_definition(ChunkId::CrossbowShot).unwrap();
        assert!(def.prerequisite_chunks.contains(&ChunkId::BasicAim));
        assert!(!def.prerequisite_chunks.contains(&ChunkId::LoadCrossbow));
    }

    #[test]
    fn test_bow_requires_more_reps_than_crossbow() {
        let bow = get_chunk_definition(ChunkId::LooseArrow).unwrap();
        let crossbow = get_chunk_definition(ChunkId::CrossbowShot).unwrap();

        // Bows are harder to master
        assert!(bow.base_repetitions > crossbow.base_repetitions);
    }
}
