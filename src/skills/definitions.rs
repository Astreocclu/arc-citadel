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
        components: ChunkComponents::Composite(&[
            ChunkId::BasicStance,
            ChunkId::BasicSwing,
        ]),
        context_requirements: &[ContextTag::InMelee],
        prerequisite_chunks: &[ChunkId::BasicStance, ChunkId::BasicSwing],
        base_repetitions: 50,
    },
    ChunkDefinition {
        id: ChunkId::DefendSequence,
        name: "Defend Sequence",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::BasicBlock,
            ChunkId::BasicStance,
        ]),
        context_requirements: &[ContextTag::InMelee, ContextTag::EnemyVisible],
        prerequisite_chunks: &[ChunkId::BasicBlock, ChunkId::BasicStance],
        base_repetitions: 50,
    },
    ChunkDefinition {
        id: ChunkId::Riposte,
        name: "Riposte",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::BasicBlock,
            ChunkId::BasicSwing,
        ]),
        context_requirements: &[ContextTag::InMelee, ContextTag::EnemyVisible],
        prerequisite_chunks: &[ChunkId::BasicBlock, ChunkId::BasicSwing],
        base_repetitions: 80,
    },

    // Level 3 - Tactical chunks
    ChunkDefinition {
        id: ChunkId::EngageMelee,
        name: "Engage Melee",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::AttackSequence,
            ChunkId::DefendSequence,
        ]),
        context_requirements: &[ContextTag::InMelee],
        prerequisite_chunks: &[ChunkId::AttackSequence, ChunkId::DefendSequence],
        base_repetitions: 200,
    },
    ChunkDefinition {
        id: ChunkId::HandleFlanking,
        name: "Handle Flanking",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::EngageMelee,
        ]),
        context_requirements: &[ContextTag::InMelee, ContextTag::MultipleEnemies],
        prerequisite_chunks: &[ChunkId::EngageMelee],
        base_repetitions: 300,
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
        components: ChunkComponents::Composite(&[
            ChunkId::CraftBasicMeasure,
        ]),
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
        prerequisite_chunks: &[ChunkId::CraftDrawOutMetal, ChunkId::CraftBasicMeasure, ChunkId::CraftFinishSurface],
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
        prerequisite_chunks: &[ChunkId::CraftDrawOutMetal, ChunkId::CraftUpsetMetal, ChunkId::CraftBasicMeasure],
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
        prerequisite_chunks: &[ChunkId::CraftShapeWood, ChunkId::CraftBasicJoin, ChunkId::CraftFinishSurface],
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
        prerequisite_chunks: &[ChunkId::CraftBasicMeasure, ChunkId::CraftBasicCut, ChunkId::CraftBasicJoin],
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
        prerequisite_chunks: &[ChunkId::CraftForgeKnife, ChunkId::CraftDrawOutMetal, ChunkId::CraftBasicWeld],
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
        prerequisite_chunks: &[ChunkId::CraftForgeToolHead, ChunkId::CraftDrawOutMetal, ChunkId::CraftBasicWeld],
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
        prerequisite_chunks: &[ChunkId::CraftBasicWeld, ChunkId::CraftDrawOutMetal, ChunkId::CraftForgeKnife],
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
        prerequisite_chunks: &[ChunkId::CraftAssessAndExecute, ChunkId::CraftForgeMasterwork],
        base_repetitions: 1500,
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
            assert_eq!(def.level, def.id.level(),
                "Definition level mismatch for {:?}", def.id);
        }
    }

    #[test]
    fn test_prerequisites_exist() {
        for def in CHUNK_LIBRARY {
            for prereq in def.prerequisite_chunks {
                assert!(get_chunk_definition(*prereq).is_some(),
                    "Missing prerequisite {:?} for {:?}", prereq, def.id);
            }
        }
    }

    #[test]
    fn test_composite_components_exist() {
        for def in CHUNK_LIBRARY {
            if let ChunkComponents::Composite(components) = &def.components {
                for comp in *components {
                    assert!(get_chunk_definition(*comp).is_some(),
                        "Missing component {:?} for {:?}", comp, def.id);
                }
            }
        }
    }
}
