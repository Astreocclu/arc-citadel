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
    // === LEADERSHIP DOMAIN ===
    // Level 1 - Micro-chunks (Atomic)
    ChunkDefinition {
        id: ChunkId::LeadCommandPresence,
        name: "Command Presence",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 20,
    },
    ChunkDefinition {
        id: ChunkId::LeadClearOrder,
        name: "Clear Order",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 15,
    },
    ChunkDefinition {
        id: ChunkId::LeadSituationalRead,
        name: "Situational Read",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 25,
    },
    // Level 2 - Technique chunks (Composite)
    ChunkDefinition {
        id: ChunkId::LeadIssueCommand,
        name: "Issue Command",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::LeadCommandPresence,
            ChunkId::LeadClearOrder,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::LeadCommandPresence, ChunkId::LeadClearOrder],
        base_repetitions: 40,
    },
    ChunkDefinition {
        id: ChunkId::LeadAssessUnitState,
        name: "Assess Unit State",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::LeadSituationalRead]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::LeadSituationalRead],
        base_repetitions: 35,
    },
    ChunkDefinition {
        id: ChunkId::LeadDelegateTask,
        name: "Delegate Task",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::LeadClearOrder,
            ChunkId::LeadSituationalRead,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::LeadClearOrder, ChunkId::LeadSituationalRead],
        base_repetitions: 45,
    },
    ChunkDefinition {
        id: ChunkId::LeadMaintainCalm,
        name: "Maintain Calm",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::LeadCommandPresence]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::LeadCommandPresence],
        base_repetitions: 50,
    },
    // Level 3 - Tactical chunks
    ChunkDefinition {
        id: ChunkId::LeadDirectFormation,
        name: "Direct Formation",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::LeadIssueCommand,
            ChunkId::LeadAssessUnitState,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::LeadIssueCommand, ChunkId::LeadAssessUnitState],
        base_repetitions: 120,
    },
    ChunkDefinition {
        id: ChunkId::LeadRespondToCrisis,
        name: "Respond to Crisis",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::LeadMaintainCalm,
            ChunkId::LeadSituationalRead,
            ChunkId::LeadIssueCommand,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::LeadMaintainCalm,
            ChunkId::LeadSituationalRead,
            ChunkId::LeadIssueCommand,
        ],
        base_repetitions: 150,
    },
    ChunkDefinition {
        id: ChunkId::LeadRallyWavering,
        name: "Rally Wavering",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::LeadCommandPresence,
            ChunkId::LeadAssessUnitState,
            ChunkId::LeadMaintainCalm,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::LeadCommandPresence,
            ChunkId::LeadAssessUnitState,
            ChunkId::LeadMaintainCalm,
        ],
        base_repetitions: 100,
    },
    ChunkDefinition {
        id: ChunkId::LeadCoordinateUnits,
        name: "Coordinate Units",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::LeadDelegateTask,
            ChunkId::LeadIssueCommand,
            ChunkId::LeadAssessUnitState,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::LeadDelegateTask,
            ChunkId::LeadIssueCommand,
            ChunkId::LeadAssessUnitState,
        ],
        base_repetitions: 180,
    },
    // Level 4 - Strategic chunks
    ChunkDefinition {
        id: ChunkId::LeadBattleManagement,
        name: "Battle Management",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::LeadDirectFormation,
            ChunkId::LeadCoordinateUnits,
            ChunkId::LeadRespondToCrisis,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::LeadDirectFormation,
            ChunkId::LeadCoordinateUnits,
            ChunkId::LeadRespondToCrisis,
        ],
        base_repetitions: 350,
    },
    ChunkDefinition {
        id: ChunkId::LeadCampaignPlanning,
        name: "Campaign Planning",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::LeadCoordinateUnits,
            ChunkId::LeadSituationalRead,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::LeadCoordinateUnits, ChunkId::LeadSituationalRead],
        base_repetitions: 400,
    },
    ChunkDefinition {
        id: ChunkId::LeadOrganizationBuilding,
        name: "Organization Building",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::LeadDelegateTask,
            ChunkId::LeadAssessUnitState,
            ChunkId::LeadRallyWavering,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::LeadDelegateTask,
            ChunkId::LeadAssessUnitState,
            ChunkId::LeadRallyWavering,
        ],
        base_repetitions: 450,
    },
    // Level 5 - Mastery chunks
    ChunkDefinition {
        id: ChunkId::LeadReadBattleFlow,
        name: "Read Battle Flow",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::LeadBattleManagement,
            ChunkId::LeadSituationalRead,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::LeadBattleManagement, ChunkId::LeadSituationalRead],
        base_repetitions: 700,
    },
    ChunkDefinition {
        id: ChunkId::LeadInspireArmy,
        name: "Inspire Army",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::LeadRallyWavering,
            ChunkId::LeadCommandPresence,
            ChunkId::LeadBattleManagement,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::LeadRallyWavering,
            ChunkId::LeadCommandPresence,
            ChunkId::LeadBattleManagement,
        ],
        base_repetitions: 800,
    },
    ChunkDefinition {
        id: ChunkId::LeadStrategicIntuition,
        name: "Strategic Intuition",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::LeadReadBattleFlow,
            ChunkId::LeadCampaignPlanning,
            ChunkId::LeadBattleManagement,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::LeadReadBattleFlow,
            ChunkId::LeadCampaignPlanning,
            ChunkId::LeadBattleManagement,
        ],
        base_repetitions: 1000,
    },
    // === KNOWLEDGE DOMAIN ===
    // Level 1 - Micro-chunks (Atomic)
    ChunkDefinition {
        id: ChunkId::KnowFluentReading,
        name: "Fluent Reading",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 20,
    },
    ChunkDefinition {
        id: ChunkId::KnowFluentWriting,
        name: "Fluent Writing",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 25,
    },
    ChunkDefinition {
        id: ChunkId::KnowArithmetic,
        name: "Arithmetic",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 15,
    },
    ChunkDefinition {
        id: ChunkId::KnowMemorization,
        name: "Memorization",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 20,
    },
    // Level 2 - Technique chunks (Composite)
    ChunkDefinition {
        id: ChunkId::KnowResearchSource,
        name: "Research Source",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::KnowFluentReading,
            ChunkId::KnowMemorization,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::KnowFluentReading, ChunkId::KnowMemorization],
        base_repetitions: 50,
    },
    ChunkDefinition {
        id: ChunkId::KnowComposeDocument,
        name: "Compose Document",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::KnowFluentWriting,
            ChunkId::KnowFluentReading,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::KnowFluentWriting, ChunkId::KnowFluentReading],
        base_repetitions: 60,
    },
    ChunkDefinition {
        id: ChunkId::KnowMathematicalProof,
        name: "Mathematical Proof",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::KnowArithmetic,
            ChunkId::KnowFluentWriting,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::KnowArithmetic, ChunkId::KnowFluentWriting],
        base_repetitions: 80,
    },
    ChunkDefinition {
        id: ChunkId::KnowTeachConcept,
        name: "Teach Concept",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::KnowFluentReading,
            ChunkId::KnowMemorization,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::KnowFluentReading, ChunkId::KnowMemorization],
        base_repetitions: 55,
    },
    ChunkDefinition {
        id: ChunkId::KnowTranslateText,
        name: "Translate Text",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::KnowFluentReading,
            ChunkId::KnowFluentWriting,
            ChunkId::KnowMemorization,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::KnowFluentReading,
            ChunkId::KnowFluentWriting,
            ChunkId::KnowMemorization,
        ],
        base_repetitions: 70,
    },
    // Level 3 - Application chunks
    ChunkDefinition {
        id: ChunkId::KnowAnalyzeText,
        name: "Analyze Text",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::KnowResearchSource,
            ChunkId::KnowFluentReading,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::KnowResearchSource, ChunkId::KnowFluentReading],
        base_repetitions: 120,
    },
    ChunkDefinition {
        id: ChunkId::KnowSynthesizeSources,
        name: "Synthesize Sources",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::KnowResearchSource,
            ChunkId::KnowComposeDocument,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::KnowResearchSource, ChunkId::KnowComposeDocument],
        base_repetitions: 150,
    },
    ChunkDefinition {
        id: ChunkId::KnowFormalArgument,
        name: "Formal Argument",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::KnowMathematicalProof,
            ChunkId::KnowComposeDocument,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::KnowMathematicalProof, ChunkId::KnowComposeDocument],
        base_repetitions: 130,
    },
    ChunkDefinition {
        id: ChunkId::KnowInstructStudent,
        name: "Instruct Student",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::KnowTeachConcept,
            ChunkId::KnowAnalyzeText,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::KnowTeachConcept, ChunkId::KnowAnalyzeText],
        base_repetitions: 100,
    },
    // Level 4 - Expert chunks
    ChunkDefinition {
        id: ChunkId::KnowOriginalResearch,
        name: "Original Research",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::KnowSynthesizeSources,
            ChunkId::KnowAnalyzeText,
            ChunkId::KnowFormalArgument,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::KnowSynthesizeSources,
            ChunkId::KnowAnalyzeText,
            ChunkId::KnowFormalArgument,
        ],
        base_repetitions: 350,
    },
    ChunkDefinition {
        id: ChunkId::KnowComprehensiveTreatise,
        name: "Comprehensive Treatise",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::KnowSynthesizeSources,
            ChunkId::KnowFormalArgument,
            ChunkId::KnowComposeDocument,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::KnowSynthesizeSources,
            ChunkId::KnowFormalArgument,
            ChunkId::KnowComposeDocument,
        ],
        base_repetitions: 400,
    },
    ChunkDefinition {
        id: ChunkId::KnowCurriculumDesign,
        name: "Curriculum Design",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::KnowInstructStudent,
            ChunkId::KnowSynthesizeSources,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::KnowInstructStudent, ChunkId::KnowSynthesizeSources],
        base_repetitions: 300,
    },
    // Level 5 - Mastery chunks
    ChunkDefinition {
        id: ChunkId::KnowParadigmIntegration,
        name: "Paradigm Integration",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::KnowOriginalResearch,
            ChunkId::KnowComprehensiveTreatise,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::KnowOriginalResearch,
            ChunkId::KnowComprehensiveTreatise,
        ],
        base_repetitions: 800,
    },
    ChunkDefinition {
        id: ChunkId::KnowIntellectualLegacy,
        name: "Intellectual Legacy",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::KnowParadigmIntegration,
            ChunkId::KnowCurriculumDesign,
            ChunkId::KnowOriginalResearch,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::KnowParadigmIntegration,
            ChunkId::KnowCurriculumDesign,
            ChunkId::KnowOriginalResearch,
        ],
        base_repetitions: 1000,
    },
    // === PHYSICAL DOMAIN ===
    // Level 1 - Micro-chunks (Atomic)
    ChunkDefinition {
        id: ChunkId::PhysEfficientGait,
        name: "Efficient Gait",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 20,
    },
    ChunkDefinition {
        id: ChunkId::PhysQuietMovement,
        name: "Quiet Movement",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 25,
    },
    ChunkDefinition {
        id: ChunkId::PhysPowerStance,
        name: "Power Stance",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 15,
    },
    ChunkDefinition {
        id: ChunkId::PhysClimbGrip,
        name: "Climb Grip",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 20,
    },
    // Level 2 - Technique chunks (Composite)
    ChunkDefinition {
        id: ChunkId::PhysDistanceRunning,
        name: "Distance Running",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::PhysEfficientGait]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysEfficientGait],
        base_repetitions: 60,
    },
    ChunkDefinition {
        id: ChunkId::PhysHeavyLifting,
        name: "Heavy Lifting",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::PhysPowerStance]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysPowerStance],
        base_repetitions: 40,
    },
    ChunkDefinition {
        id: ChunkId::PhysSilentApproach,
        name: "Silent Approach",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysQuietMovement,
            ChunkId::PhysEfficientGait,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysQuietMovement, ChunkId::PhysEfficientGait],
        base_repetitions: 55,
    },
    ChunkDefinition {
        id: ChunkId::PhysRockClimbing,
        name: "Rock Climbing",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::PhysClimbGrip, ChunkId::PhysPowerStance]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysClimbGrip, ChunkId::PhysPowerStance],
        base_repetitions: 50,
    },
    ChunkDefinition {
        id: ChunkId::PhysHorseControl,
        name: "Horse Control",
        level: 2,
        components: ChunkComponents::Composite(&[ChunkId::PhysPowerStance]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysPowerStance],
        base_repetitions: 70,
    },
    // Level 3 - Application chunks
    ChunkDefinition {
        id: ChunkId::PhysSustainedLabor,
        name: "Sustained Labor",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysDistanceRunning,
            ChunkId::PhysHeavyLifting,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysDistanceRunning, ChunkId::PhysHeavyLifting],
        base_repetitions: 120,
    },
    ChunkDefinition {
        id: ChunkId::PhysInfiltration,
        name: "Infiltration",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysSilentApproach,
            ChunkId::PhysRockClimbing,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysSilentApproach, ChunkId::PhysRockClimbing],
        base_repetitions: 150,
    },
    ChunkDefinition {
        id: ChunkId::PhysRoughTerrainTravel,
        name: "Rough Terrain Travel",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysDistanceRunning,
            ChunkId::PhysRockClimbing,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysDistanceRunning, ChunkId::PhysRockClimbing],
        base_repetitions: 100,
    },
    ChunkDefinition {
        id: ChunkId::PhysCavalryRiding,
        name: "Cavalry Riding",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysHorseControl,
            ChunkId::PhysPowerStance,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysHorseControl, ChunkId::PhysPowerStance],
        base_repetitions: 130,
    },
    ChunkDefinition {
        id: ChunkId::PhysSwimming,
        name: "Swimming",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysEfficientGait,
            ChunkId::PhysPowerStance,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysEfficientGait, ChunkId::PhysPowerStance],
        base_repetitions: 80,
    },
    // Level 4 - Expert chunks
    ChunkDefinition {
        id: ChunkId::PhysLaborLeadership,
        name: "Labor Leadership",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysSustainedLabor,
            ChunkId::PhysHeavyLifting,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysSustainedLabor, ChunkId::PhysHeavyLifting],
        base_repetitions: 300,
    },
    ChunkDefinition {
        id: ChunkId::PhysScoutMission,
        name: "Scout Mission",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysInfiltration,
            ChunkId::PhysRoughTerrainTravel,
            ChunkId::PhysSilentApproach,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::PhysInfiltration,
            ChunkId::PhysRoughTerrainTravel,
            ChunkId::PhysSilentApproach,
        ],
        base_repetitions: 350,
    },
    ChunkDefinition {
        id: ChunkId::PhysMountedCombat,
        name: "Mounted Combat",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysCavalryRiding,
            ChunkId::PhysHorseControl,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysCavalryRiding, ChunkId::PhysHorseControl],
        base_repetitions: 400,
    },
    ChunkDefinition {
        id: ChunkId::PhysSurvivalTravel,
        name: "Survival Travel",
        level: 4,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysRoughTerrainTravel,
            ChunkId::PhysSwimming,
            ChunkId::PhysSustainedLabor,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[
            ChunkId::PhysRoughTerrainTravel,
            ChunkId::PhysSwimming,
            ChunkId::PhysSustainedLabor,
        ],
        base_repetitions: 380,
    },
    // Level 5 - Mastery chunks
    ChunkDefinition {
        id: ChunkId::PhysTirelessEndurance,
        name: "Tireless Endurance",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysSustainedLabor,
            ChunkId::PhysSurvivalTravel,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysSustainedLabor, ChunkId::PhysSurvivalTravel],
        base_repetitions: 800,
    },
    ChunkDefinition {
        id: ChunkId::PhysShadowMovement,
        name: "Shadow Movement",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysInfiltration,
            ChunkId::PhysScoutMission,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysInfiltration, ChunkId::PhysScoutMission],
        base_repetitions: 900,
    },
    ChunkDefinition {
        id: ChunkId::PhysCentaurUnity,
        name: "Centaur Unity",
        level: 5,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysMountedCombat,
            ChunkId::PhysCavalryRiding,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysMountedCombat, ChunkId::PhysCavalryRiding],
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
