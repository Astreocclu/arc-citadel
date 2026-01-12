//! Spawn loadout generation for entity chunk libraries
//!
//! Every entity spawns with chunks appropriate to their archetype and age.
//! No entity should ever have an empty ChunkLibrary.
//!
//! DEPRECATED: Use generate_history_for_role() + generate_chunks_from_history() instead.

#[allow(deprecated)]
use crate::entity::{CraftSpecialty, EntityArchetype, TrainingLevel};
use crate::skills::{ChunkId, ChunkLibrary, PersonalChunkState};
use rand::Rng;

// === SPAWN LOADOUT CONSTANTS ===

/// Depth variance for spawn chunks (±5%)
const DEPTH_VARIANCE: f32 = 0.05;

/// Minimum viable encoding depth
const MIN_CHUNK_DEPTH: f32 = 0.05;

/// Maximum encoding depth at spawn
const MAX_CHUNK_DEPTH: f32 = 0.95;

/// Repetition count multiplier (depth * REPS_PER_DEPTH reps)
const REPS_PER_DEPTH: f32 = 500.0;

/// Formation tick multiplier (depth * this = ticks ago chunk formed)
const FORMATION_TICK_MULTIPLIER: f32 = 10000.0;

// === AGE THRESHOLDS ===

/// Age at which walking becomes automatic
const WALKING_AGE: u32 = 5;

/// Age at which more physical coordination develops
const ADOLESCENT_AGE: u32 = 12;

/// Age at which adult physical baseline reached
const ADULT_AGE: u32 = 16;

// === ARCHETYPE EXPERIENCE CURVES ===

/// Peasant starts informal work around age 10
const PEASANT_WORK_START: u32 = 10;

/// Peasant reaches experience cap over 30 years
const PEASANT_MASTERY_YEARS: f32 = 30.0;

/// Laborer starts more structured work at age 12
const LABORER_WORK_START: u32 = 12;

/// Laborer reaches experience cap faster (25 years)
const LABORER_MASTERY_YEARS: f32 = 25.0;

/// Craftsman apprentice age
const CRAFTSMAN_APPRENTICE_AGE: u32 = 14;

/// Years to craft mastery
const CRAFTSMAN_MASTERY_YEARS: f32 = 20.0;

/// Generate starting chunks based on entity archetype and age.
///
/// # Arguments
/// * `archetype` - The entity's role/profession
/// * `age` - Age in years (affects skill depth)
/// * `tick` - Current simulation tick (for formation_tick)
/// * `rng` - Random number generator for variance
///
/// # Returns
/// A ChunkLibrary with appropriate chunks. Never empty.
pub fn generate_spawn_chunks(
    archetype: EntityArchetype,
    age: u32,
    tick: u64,
    rng: &mut impl Rng,
) -> ChunkLibrary {
    let mut library = ChunkLibrary::new();

    // === UNIVERSAL CHUNKS (everyone who can walk) ===
    add_universal_chunks(&mut library, age, tick, rng);

    // === ARCHETYPE-SPECIFIC CHUNKS ===
    match archetype {
        EntityArchetype::Peasant => add_peasant_chunks(&mut library, age, tick, rng),
        EntityArchetype::Laborer => add_laborer_chunks(&mut library, age, tick, rng),
        EntityArchetype::Craftsman { specialty } => {
            add_craftsman_chunks(&mut library, specialty, age, tick, rng)
        }
        EntityArchetype::Soldier { training } => {
            add_soldier_chunks(&mut library, training, age, tick, rng)
        }
        EntityArchetype::Noble => add_noble_chunks(&mut library, age, tick, rng),
        EntityArchetype::Merchant => add_merchant_chunks(&mut library, age, tick, rng),
        EntityArchetype::Scholar => add_scholar_chunks(&mut library, age, tick, rng),
        EntityArchetype::Child => {
            // Children only have universal chunks (already added above)
        }
    }

    library
}

/// Add universal physical chunks everyone has from years of living
fn add_universal_chunks(library: &mut ChunkLibrary, age: u32, tick: u64, rng: &mut impl Rng) {
    if age >= WALKING_AGE {
        // Children can walk by age 5, but still requires some conscious effort (0.5)
        set_chunk_with_variance(library, ChunkId::PhysEfficientGait, 0.5, tick, rng);
    }

    if age >= ADOLESCENT_AGE {
        // By adolescence, walking becomes more automatic; quiet movement learned
        set_chunk_with_variance(library, ChunkId::PhysEfficientGait, 0.6, tick, rng);
        set_chunk_with_variance(library, ChunkId::PhysQuietMovement, 0.3, tick, rng);
    }

    if age >= ADULT_AGE {
        // Adult baseline - years of practice yield deeper encoding
        set_chunk_with_variance(library, ChunkId::PhysEfficientGait, 0.7, tick, rng);
        set_chunk_with_variance(library, ChunkId::PhysPowerStance, 0.4, tick, rng);
    }
}

/// Set a chunk with randomized encoding depth and derived state.
///
/// Applies ±5% random variance to create individual differences.
/// Repetition count is derived from depth (0.7 depth → 350 reps).
/// Formation tick is set in the past proportional to depth,
/// simulating that deeper chunks were learned earlier.
fn set_chunk_with_variance(
    library: &mut ChunkLibrary,
    chunk_id: ChunkId,
    base_depth: f32,
    tick: u64,
    rng: &mut impl Rng,
) {
    let variance = rng.gen_range(-DEPTH_VARIANCE..DEPTH_VARIANCE);
    let depth = (base_depth + variance).clamp(MIN_CHUNK_DEPTH, MAX_CHUNK_DEPTH);
    let reps = (depth * REPS_PER_DEPTH) as u32;

    library.set_chunk(
        chunk_id,
        PersonalChunkState {
            encoding_depth: depth,
            repetition_count: reps,
            last_used_tick: tick,
            formation_tick: tick.saturating_sub((depth * FORMATION_TICK_MULTIPLIER) as u64),
        },
    );
}

/// Peasants have physical labor skills but not specialized craft
fn add_peasant_chunks(library: &mut ChunkLibrary, age: u32, tick: u64, rng: &mut impl Rng) {
    let years_working = age.saturating_sub(PEASANT_WORK_START) as f32;
    let experience = (years_working / PEASANT_MASTERY_YEARS).min(1.0);

    // Physical labor - moderate skill from field work
    set_chunk_with_variance(
        library,
        ChunkId::PhysSustainedLabor,
        0.3 + experience * 0.4,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::PhysHeavyLifting,
        0.2 + experience * 0.3,
        tick,
        rng,
    );

    // Rough terrain from rural life
    set_chunk_with_variance(
        library,
        ChunkId::PhysRoughTerrainTravel,
        0.3 + experience * 0.3,
        tick,
        rng,
    );

    // Basic social (village interactions)
    set_chunk_with_variance(
        library,
        ChunkId::SocialActiveListening,
        0.2 + experience * 0.2,
        tick,
        rng,
    );

    // Basic craft (everyone can do rough work)
    set_chunk_with_variance(
        library,
        ChunkId::CraftBasicCut,
        0.1 + experience * 0.3,
        tick,
        rng,
    );
}

/// Laborers have better physical skills than peasants
fn add_laborer_chunks(library: &mut ChunkLibrary, age: u32, tick: u64, rng: &mut impl Rng) {
    let years_working = age.saturating_sub(LABORER_WORK_START) as f32;
    let experience = (years_working / LABORER_MASTERY_YEARS).min(1.0);

    // Strong physical labor
    set_chunk_with_variance(
        library,
        ChunkId::PhysSustainedLabor,
        0.4 + experience * 0.5,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::PhysHeavyLifting,
        0.4 + experience * 0.4,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::PhysPowerStance,
        0.3 + experience * 0.4,
        tick,
        rng,
    );

    // Basic craft from construction work
    set_chunk_with_variance(
        library,
        ChunkId::CraftBasicMeasure,
        0.2 + experience * 0.3,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::CraftBasicCut,
        0.2 + experience * 0.4,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::CraftBasicJoin,
        0.1 + experience * 0.3,
        tick,
        rng,
    );
}

/// Craftsmen have deep specialty skills developed through apprenticeship
fn add_craftsman_chunks(
    library: &mut ChunkLibrary,
    specialty: CraftSpecialty,
    age: u32,
    tick: u64,
    rng: &mut impl Rng,
) {
    let years_training = age.saturating_sub(CRAFTSMAN_APPRENTICE_AGE) as f32;
    let experience = (years_training / CRAFTSMAN_MASTERY_YEARS).min(1.0);

    // All craftsmen have basic craft chunks from general training
    set_chunk_with_variance(
        library,
        ChunkId::CraftBasicMeasure,
        0.4 + experience * 0.4,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::CraftBasicCut,
        0.4 + experience * 0.4,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::CraftBasicJoin,
        0.3 + experience * 0.4,
        tick,
        rng,
    );

    // Physical from manual work
    set_chunk_with_variance(
        library,
        ChunkId::PhysSustainedLabor,
        0.4 + experience * 0.2,
        tick,
        rng,
    );

    // Specialty-specific chunks
    match specialty {
        CraftSpecialty::Smithing => {
            set_chunk_with_variance(
                library,
                ChunkId::CraftBasicHeatCycle,
                0.3 + experience * 0.5,
                tick,
                rng,
            );
            set_chunk_with_variance(
                library,
                ChunkId::CraftBasicHammerWork,
                0.3 + experience * 0.5,
                tick,
                rng,
            );
            set_chunk_with_variance(
                library,
                ChunkId::CraftDrawOutMetal,
                0.2 + experience * 0.5,
                tick,
                rng,
            );
            set_chunk_with_variance(
                library,
                ChunkId::CraftUpsetMetal,
                0.2 + experience * 0.5,
                tick,
                rng,
            );

            // Advanced smithing unlocks with experience
            if experience > 0.3 {
                set_chunk_with_variance(
                    library,
                    ChunkId::CraftBasicWeld,
                    experience * 0.6,
                    tick,
                    rng,
                );
                set_chunk_with_variance(
                    library,
                    ChunkId::CraftForgeKnife,
                    (experience - 0.3) * 0.8,
                    tick,
                    rng,
                );
            }
            if experience > 0.5 {
                set_chunk_with_variance(
                    library,
                    ChunkId::CraftForgeToolHead,
                    (experience - 0.5) * 1.0,
                    tick,
                    rng,
                );
                set_chunk_with_variance(
                    library,
                    ChunkId::CraftForgeSword,
                    (experience - 0.5) * 0.8,
                    tick,
                    rng,
                );
            }
            if experience > 0.8 {
                set_chunk_with_variance(
                    library,
                    ChunkId::CraftPatternWeld,
                    (experience - 0.8) * 1.5,
                    tick,
                    rng,
                );
                set_chunk_with_variance(
                    library,
                    ChunkId::CraftForgeArmor,
                    (experience - 0.8) * 1.2,
                    tick,
                    rng,
                );
            }
        }
        CraftSpecialty::Carpentry => {
            set_chunk_with_variance(
                library,
                ChunkId::CraftShapeWood,
                0.3 + experience * 0.5,
                tick,
                rng,
            );
            set_chunk_with_variance(
                library,
                ChunkId::CraftFinishSurface,
                0.2 + experience * 0.4,
                tick,
                rng,
            );

            if experience > 0.3 {
                set_chunk_with_variance(
                    library,
                    ChunkId::CraftBuildFurniture,
                    (experience - 0.3) * 0.9,
                    tick,
                    rng,
                );
            }
            if experience > 0.6 {
                set_chunk_with_variance(
                    library,
                    ChunkId::CraftBuildStructure,
                    (experience - 0.6) * 1.0,
                    tick,
                    rng,
                );
            }
        }
        CraftSpecialty::Masonry => {
            // Stone work uses physical strength plus basic craft
            set_chunk_with_variance(
                library,
                ChunkId::PhysHeavyLifting,
                0.4 + experience * 0.4,
                tick,
                rng,
            );
            if experience > 0.4 {
                set_chunk_with_variance(
                    library,
                    ChunkId::CraftBuildStructure,
                    (experience - 0.4) * 0.9,
                    tick,
                    rng,
                );
            }
        }
        CraftSpecialty::Cooking => {
            // Cooks use heat control similar to smiths but different application
            set_chunk_with_variance(
                library,
                ChunkId::CraftBasicHeatCycle,
                0.3 + experience * 0.4,
                tick,
                rng,
            );
        }
        CraftSpecialty::Tailoring => {
            set_chunk_with_variance(
                library,
                ChunkId::CraftSewGarment,
                0.2 + experience * 0.6,
                tick,
                rng,
            );
        }
        CraftSpecialty::Leatherwork => {
            // Similar to tailoring with different materials
            set_chunk_with_variance(
                library,
                ChunkId::CraftSewGarment,
                0.2 + experience * 0.5,
                tick,
                rng,
            );
        }
    }
}

/// Soldiers have combat skills based on training level
fn add_soldier_chunks(
    library: &mut ChunkLibrary,
    training: TrainingLevel,
    _age: u32,
    tick: u64,
    rng: &mut impl Rng,
) {
    let base = training.base_skill();

    // Combat fundamentals - all soldiers have these
    set_chunk_with_variance(library, ChunkId::BasicStance, base, tick, rng);
    set_chunk_with_variance(library, ChunkId::BasicSwing, base, tick, rng);
    set_chunk_with_variance(library, ChunkId::BasicBlock, base, tick, rng);

    // Physical fitness - soldiers are fit
    set_chunk_with_variance(library, ChunkId::PhysSustainedLabor, base + 0.1, tick, rng);
    set_chunk_with_variance(library, ChunkId::PhysDistanceRunning, base, tick, rng);

    // Militia+ has weapon coordination
    if base >= 0.35 {
        set_chunk_with_variance(library, ChunkId::AttackSequence, base - 0.1, tick, rng);
    }

    // Regular+ has defensive sequences
    if base >= 0.5 {
        set_chunk_with_variance(library, ChunkId::DefendSequence, base - 0.15, tick, rng);
        set_chunk_with_variance(library, ChunkId::Riposte, base - 0.2, tick, rng);
    }

    // Veteran+ has engagement skills
    if base >= 0.7 {
        set_chunk_with_variance(library, ChunkId::EngageMelee, base - 0.3, tick, rng);
    }

    // Elite has mastery
    if base >= 0.85 {
        set_chunk_with_variance(library, ChunkId::HandleFlanking, base - 0.35, tick, rng);
    }

    // Leadership chunks for higher ranks
    if base >= 0.5 {
        set_chunk_with_variance(library, ChunkId::LeadCommandPresence, base - 0.2, tick, rng);
    }
    if base >= 0.7 {
        set_chunk_with_variance(library, ChunkId::LeadIssueCommand, base - 0.3, tick, rng);
    }
}

/// Nobles have social, leadership, and some combat skills
fn add_noble_chunks(library: &mut ChunkLibrary, age: u32, tick: u64, rng: &mut impl Rng) {
    let years_training = age.saturating_sub(8) as f32; // Nobles educated early
    let experience = (years_training / 25.0).min(1.0);

    // Social skills (primary noble domain)
    set_chunk_with_variance(
        library,
        ChunkId::SocialProjectConfidence,
        0.4 + experience * 0.4,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::SocialBuildRapport,
        0.3 + experience * 0.4,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::SocialReadReaction,
        0.2 + experience * 0.5,
        tick,
        rng,
    );

    // Leadership (nobles expected to lead)
    set_chunk_with_variance(
        library,
        ChunkId::LeadCommandPresence,
        0.3 + experience * 0.4,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::LeadClearOrder,
        0.2 + experience * 0.4,
        tick,
        rng,
    );

    // Combat training (nobles learn to fight)
    set_chunk_with_variance(
        library,
        ChunkId::BasicStance,
        0.3 + experience * 0.3,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::BasicSwing,
        0.3 + experience * 0.3,
        tick,
        rng,
    );

    // Riding (essential noble skill)
    set_chunk_with_variance(
        library,
        ChunkId::PhysHorseControl,
        0.3 + experience * 0.4,
        tick,
        rng,
    );

    // Advanced social for experienced nobles
    if experience > 0.5 {
        set_chunk_with_variance(
            library,
            ChunkId::SocialNegotiateTerms,
            (experience - 0.5) * 0.8,
            tick,
            rng,
        );
        set_chunk_with_variance(
            library,
            ChunkId::SocialProjectAuthority,
            (experience - 0.5) * 0.8,
            tick,
            rng,
        );
        set_chunk_with_variance(
            library,
            ChunkId::LeadIssueCommand,
            (experience - 0.5) * 0.7,
            tick,
            rng,
        );
    }
}

/// Merchants have social and assessment skills
fn add_merchant_chunks(library: &mut ChunkLibrary, age: u32, tick: u64, rng: &mut impl Rng) {
    let years_trading = age.saturating_sub(14) as f32;
    let experience = (years_trading / 25.0).min(1.0);

    // Social skills (merchant bread and butter)
    set_chunk_with_variance(
        library,
        ChunkId::SocialBuildRapport,
        0.4 + experience * 0.4,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::SocialReadReaction,
        0.3 + experience * 0.5,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::SocialNegotiateTerms,
        0.3 + experience * 0.5,
        tick,
        rng,
    );

    // Travel (merchants move around)
    set_chunk_with_variance(
        library,
        ChunkId::PhysRoughTerrainTravel,
        0.3 + experience * 0.3,
        tick,
        rng,
    );

    // Basic literacy for contracts
    set_chunk_with_variance(
        library,
        ChunkId::KnowFluentReading,
        0.3 + experience * 0.3,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::KnowArithmetic,
        0.4 + experience * 0.4,
        tick,
        rng,
    );

    // Advanced social for experienced merchants
    if experience > 0.5 {
        set_chunk_with_variance(
            library,
            ChunkId::SocialPersuade,
            (experience - 0.5) * 0.8,
            tick,
            rng,
        );
    }
    if experience > 0.7 {
        set_chunk_with_variance(
            library,
            ChunkId::SocialDeceive,
            (experience - 0.7) * 0.6,
            tick,
            rng,
        );
    }
}

/// Scholars have knowledge and teaching skills
fn add_scholar_chunks(library: &mut ChunkLibrary, age: u32, tick: u64, rng: &mut impl Rng) {
    let years_study = age.saturating_sub(10) as f32;
    let experience = (years_study / 30.0).min(1.0);

    // Knowledge chunks (primary domain)
    set_chunk_with_variance(
        library,
        ChunkId::KnowFluentReading,
        0.4 + experience * 0.4,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::KnowFluentWriting,
        0.3 + experience * 0.5,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::KnowMemorization,
        0.3 + experience * 0.4,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::KnowArithmetic,
        0.4 + experience * 0.4,
        tick,
        rng,
    );

    // Research and teaching develop with experience
    if experience > 0.3 {
        set_chunk_with_variance(
            library,
            ChunkId::KnowResearchSource,
            (experience - 0.3) * 0.9,
            tick,
            rng,
        );
        set_chunk_with_variance(
            library,
            ChunkId::KnowTeachConcept,
            (experience - 0.3) * 0.8,
            tick,
            rng,
        );
    }

    if experience > 0.5 {
        set_chunk_with_variance(
            library,
            ChunkId::KnowComposeDocument,
            (experience - 0.5) * 0.8,
            tick,
            rng,
        );
        set_chunk_with_variance(
            library,
            ChunkId::KnowAnalyzeText,
            (experience - 0.5) * 0.9,
            tick,
            rng,
        );
    }

    if experience > 0.7 {
        set_chunk_with_variance(
            library,
            ChunkId::KnowSynthesizeSources,
            (experience - 0.7) * 1.0,
            tick,
            rng,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_child_has_walking_chunks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let library = generate_spawn_chunks(EntityArchetype::Child, 8, 0, &mut rng);

        // Children 5+ have basic walking chunks
        assert!(library.has_chunk(ChunkId::PhysEfficientGait));
        assert!(!library.chunks().is_empty());
    }

    #[test]
    fn test_very_young_child_no_chunks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let library = generate_spawn_chunks(EntityArchetype::Child, 3, 0, &mut rng);

        // Children under 5 don't have walking automation yet
        assert!(!library.has_chunk(ChunkId::PhysEfficientGait));
        assert!(library.chunks().is_empty());
    }

    #[test]
    fn test_adult_has_more_chunks_than_child() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let child = generate_spawn_chunks(EntityArchetype::Child, 8, 0, &mut rng);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        let adult = generate_spawn_chunks(EntityArchetype::Peasant, 25, 0, &mut rng2);

        assert!(adult.chunks().len() > child.chunks().len());
    }

    #[test]
    fn test_adolescent_chunks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let library = generate_spawn_chunks(EntityArchetype::Child, 14, 0, &mut rng);

        // Adolescents (12+) have walking and quiet movement
        assert!(library.has_chunk(ChunkId::PhysEfficientGait));
        assert!(library.has_chunk(ChunkId::PhysQuietMovement));
        // But not power stance (requires age 16+)
        assert!(!library.has_chunk(ChunkId::PhysPowerStance));
    }

    #[test]
    fn test_adult_universal_chunks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let library = generate_spawn_chunks(EntityArchetype::Peasant, 25, 0, &mut rng);

        // Adults (16+) have full physical automation
        assert!(library.has_chunk(ChunkId::PhysEfficientGait));
        assert!(library.has_chunk(ChunkId::PhysQuietMovement));
        assert!(library.has_chunk(ChunkId::PhysPowerStance));
    }

    #[test]
    fn test_no_empty_libraries_for_adults() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let archetypes = [
            EntityArchetype::Peasant,
            EntityArchetype::Laborer,
            EntityArchetype::Noble,
            EntityArchetype::Merchant,
            EntityArchetype::Scholar,
        ];

        for archetype in archetypes {
            let library = generate_spawn_chunks(archetype, 20, 0, &mut rng);
            assert!(
                !library.chunks().is_empty(),
                "{:?} should have chunks",
                archetype
            );
        }
    }

    #[test]
    fn test_craftsman_has_chunks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let library = generate_spawn_chunks(
            EntityArchetype::Craftsman {
                specialty: CraftSpecialty::Smithing,
            },
            30,
            0,
            &mut rng,
        );

        // Should have universal adult chunks at minimum
        assert!(library.has_chunk(ChunkId::PhysEfficientGait));
        assert!(!library.chunks().is_empty());
    }

    #[test]
    fn test_soldier_has_chunks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let library = generate_spawn_chunks(
            EntityArchetype::Soldier {
                training: TrainingLevel::Regular,
            },
            25,
            0,
            &mut rng,
        );

        // Should have universal adult chunks at minimum
        assert!(library.has_chunk(ChunkId::PhysEfficientGait));
        assert!(!library.chunks().is_empty());
    }

    #[test]
    fn test_encoding_depth_has_variance() {
        // With same seed, same entity gets consistent results
        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);

        let lib1 = generate_spawn_chunks(EntityArchetype::Peasant, 25, 0, &mut rng1);
        let lib2 = generate_spawn_chunks(EntityArchetype::Peasant, 25, 0, &mut rng2);

        // Same seed produces same results
        assert_eq!(
            lib1.get_chunk(ChunkId::PhysEfficientGait)
                .unwrap()
                .encoding_depth,
            lib2.get_chunk(ChunkId::PhysEfficientGait)
                .unwrap()
                .encoding_depth
        );

        // Different seed produces different results
        let mut rng3 = ChaCha8Rng::seed_from_u64(999);
        let lib3 = generate_spawn_chunks(EntityArchetype::Peasant, 25, 0, &mut rng3);

        // Depth should be different (with high probability)
        let depth1 = lib1
            .get_chunk(ChunkId::PhysEfficientGait)
            .unwrap()
            .encoding_depth;
        let depth3 = lib3
            .get_chunk(ChunkId::PhysEfficientGait)
            .unwrap()
            .encoding_depth;

        // They should both be around 0.7 (adult base) but not exactly equal
        assert!(depth1 >= 0.65 && depth1 <= 0.75);
        assert!(depth3 >= 0.65 && depth3 <= 0.75);
    }

    #[test]
    fn test_formation_tick_in_past() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let current_tick = 10000;
        let library = generate_spawn_chunks(EntityArchetype::Peasant, 25, current_tick, &mut rng);

        // Formation tick should be in the past (simulating having learned it before)
        let state = library.get_chunk(ChunkId::PhysEfficientGait).unwrap();
        assert!(state.formation_tick < current_tick);
    }

    #[test]
    fn test_repetition_count_scales_with_depth() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let library = generate_spawn_chunks(EntityArchetype::Peasant, 25, 0, &mut rng);

        let state = library.get_chunk(ChunkId::PhysEfficientGait).unwrap();
        // Repetitions should be approximately depth * 500
        let expected_reps = (state.encoding_depth * 500.0) as u32;
        assert_eq!(state.repetition_count, expected_reps);
    }

    #[test]
    fn test_peasant_has_labor_chunks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let library = generate_spawn_chunks(EntityArchetype::Peasant, 30, 0, &mut rng);

        assert!(library.has_chunk(ChunkId::PhysSustainedLabor));
        assert!(library.has_chunk(ChunkId::SocialActiveListening));
    }

    #[test]
    fn test_laborer_stronger_physical_than_peasant() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let peasant = generate_spawn_chunks(EntityArchetype::Peasant, 30, 0, &mut rng);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        let laborer = generate_spawn_chunks(EntityArchetype::Laborer, 30, 0, &mut rng2);

        let peasant_labor = peasant.get_chunk(ChunkId::PhysSustainedLabor).unwrap();
        let laborer_labor = laborer.get_chunk(ChunkId::PhysSustainedLabor).unwrap();

        assert!(laborer_labor.encoding_depth > peasant_labor.encoding_depth);
    }

    #[test]
    fn test_older_peasant_more_skilled() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let young = generate_spawn_chunks(EntityArchetype::Peasant, 18, 0, &mut rng);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        let old = generate_spawn_chunks(EntityArchetype::Peasant, 45, 0, &mut rng2);

        let young_labor = young.get_chunk(ChunkId::PhysSustainedLabor).unwrap();
        let old_labor = old.get_chunk(ChunkId::PhysSustainedLabor).unwrap();

        assert!(old_labor.encoding_depth > young_labor.encoding_depth);
    }

    #[test]
    fn test_smith_has_smithing_chunks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let smith = generate_spawn_chunks(
            EntityArchetype::Craftsman {
                specialty: CraftSpecialty::Smithing,
            },
            30,
            0,
            &mut rng,
        );

        assert!(smith.has_chunk(ChunkId::CraftBasicHeatCycle));
        assert!(smith.has_chunk(ChunkId::CraftBasicHammerWork));
    }

    #[test]
    fn test_master_smith_has_advanced_chunks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let master = generate_spawn_chunks(
            EntityArchetype::Craftsman {
                specialty: CraftSpecialty::Smithing,
            },
            50,
            0,
            &mut rng,
        );

        // 36 years experience should unlock advanced chunks
        assert!(master.has_chunk(ChunkId::CraftForgeSword));
    }

    #[test]
    fn test_carpenter_has_carpentry_chunks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let carpenter = generate_spawn_chunks(
            EntityArchetype::Craftsman {
                specialty: CraftSpecialty::Carpentry,
            },
            30,
            0,
            &mut rng,
        );

        assert!(carpenter.has_chunk(ChunkId::CraftShapeWood));
        assert!(!carpenter.has_chunk(ChunkId::CraftBasicHeatCycle)); // No smithing
    }

    #[test]
    fn test_levy_has_basic_combat() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let levy = generate_spawn_chunks(
            EntityArchetype::Soldier {
                training: TrainingLevel::Levy,
            },
            25,
            0,
            &mut rng,
        );

        assert!(levy.has_chunk(ChunkId::BasicStance));
        assert!(levy.has_chunk(ChunkId::BasicSwing));
    }

    #[test]
    fn test_veteran_deeper_than_levy() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let levy = generate_spawn_chunks(
            EntityArchetype::Soldier {
                training: TrainingLevel::Levy,
            },
            25,
            0,
            &mut rng,
        );
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        let veteran = generate_spawn_chunks(
            EntityArchetype::Soldier {
                training: TrainingLevel::Veteran,
            },
            25,
            0,
            &mut rng2,
        );

        let levy_stance = levy.get_chunk(ChunkId::BasicStance).unwrap();
        let veteran_stance = veteran.get_chunk(ChunkId::BasicStance).unwrap();

        assert!(veteran_stance.encoding_depth > levy_stance.encoding_depth);
    }

    #[test]
    fn test_elite_has_advanced_combat() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let elite = generate_spawn_chunks(
            EntityArchetype::Soldier {
                training: TrainingLevel::Elite,
            },
            30,
            0,
            &mut rng,
        );

        assert!(elite.has_chunk(ChunkId::EngageMelee));
        assert!(elite.has_chunk(ChunkId::HandleFlanking));
    }

    #[test]
    fn test_noble_has_social_and_combat() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let noble = generate_spawn_chunks(EntityArchetype::Noble, 30, 0, &mut rng);

        assert!(noble.has_chunk(ChunkId::SocialProjectConfidence));
        assert!(noble.has_chunk(ChunkId::LeadCommandPresence));
        assert!(noble.has_chunk(ChunkId::BasicStance)); // Combat training
    }

    #[test]
    fn test_merchant_has_social_and_assessment() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let merchant = generate_spawn_chunks(EntityArchetype::Merchant, 35, 0, &mut rng);

        assert!(merchant.has_chunk(ChunkId::SocialBuildRapport));
        assert!(merchant.has_chunk(ChunkId::SocialNegotiateTerms));
    }

    #[test]
    fn test_scholar_has_knowledge_chunks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let scholar = generate_spawn_chunks(EntityArchetype::Scholar, 40, 0, &mut rng);

        assert!(scholar.has_chunk(ChunkId::KnowFluentReading));
        assert!(scholar.has_chunk(ChunkId::KnowFluentWriting));
    }
}
