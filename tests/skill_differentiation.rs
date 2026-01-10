//! Integration test: Skill Differentiation
//!
//! Tests that novice and master entities produce different outcomes
//! when performing the same action.

use arc_citadel::genetics::Phenotype;
use arc_citadel::skills::{
    calculate_encoding_depth, compute_craftsmanship, ChunkId, ChunkLibrary,
    PersonalChunkState, SkillLevel,
};

/// Create a novice crafter (no chunks)
fn create_novice() -> (ChunkLibrary, Phenotype) {
    (ChunkLibrary::new(), Phenotype::default())
}

/// Create a master crafter (deep L5 chunks)
fn create_master(tick: u64) -> (ChunkLibrary, Phenotype) {
    let mut lib = ChunkLibrary::new();

    // Add all craft chunks with deep encoding
    let craft_chunks = [
        (ChunkId::CraftBasicHeatCycle, 500),
        (ChunkId::CraftBasicHammerWork, 500),
        (ChunkId::CraftBasicMeasure, 400),
        (ChunkId::CraftBasicCut, 400),
        (ChunkId::CraftBasicJoin, 400),
        (ChunkId::CraftDrawOutMetal, 300),
        (ChunkId::CraftUpsetMetal, 300),
        (ChunkId::CraftBasicWeld, 250),
        (ChunkId::CraftForgeKnife, 200),
        (ChunkId::CraftForgeToolHead, 200),
        (ChunkId::CraftForgeSword, 150),
        (ChunkId::CraftForgeArmor, 150),
        (ChunkId::CraftPatternWeld, 100),
        (ChunkId::CraftAssessAndExecute, 80),
        (ChunkId::CraftForgeMasterwork, 50),
    ];

    for (chunk_id, reps) in craft_chunks {
        lib.set_chunk(chunk_id, PersonalChunkState {
            encoding_depth: calculate_encoding_depth(reps),
            repetition_count: reps,
            last_used_tick: tick,
            formation_tick: tick.saturating_sub(10000),
        });
    }

    (lib, Phenotype::default())
}

#[test]
fn test_novice_vs_master_display_stats() {
    let (novice_lib, novice_pheno) = create_novice();
    let (master_lib, master_pheno) = create_master(1000);

    let novice_stat = compute_craftsmanship(&novice_lib, &novice_pheno);
    let master_stat = compute_craftsmanship(&master_lib, &master_pheno);

    // Novice should be Untrained
    assert_eq!(novice_stat.level, SkillLevel::Untrained);
    assert_eq!(novice_stat.bar_fill, 0.0);

    // Master should be Expert or Master (has L5 chunks)
    assert!(
        master_stat.level == SkillLevel::Expert || master_stat.level == SkillLevel::Master,
        "Master should be Expert or Master, got {:?}",
        master_stat.level
    );
    assert!(master_stat.bar_fill > 0.5);
}

#[test]
fn test_novice_high_attention_cost() {
    let (novice_lib, _) = create_novice();
    let (master_lib, _) = create_master(1000);

    // Novice has no chunks - would need to use atomics (high cost)
    // Master has deep chunks - low attention cost

    // Check attention cost for a craft action
    // Novice: no chunk = ~1.0 attention cost
    // Master: deep chunk = ~0.1-0.2 attention cost

    let novice_best = novice_lib.domain_summary(arc_citadel::skills::ChunkDomain::Craft);
    let master_best = master_lib.domain_summary(arc_citadel::skills::ChunkDomain::Craft);

    // Novice has no craft chunks
    assert_eq!(novice_best.chunk_count, 0);

    // Master has many craft chunks
    assert!(master_best.chunk_count >= 10);
    // With the logarithmic encoding curve (LEARNING_RATE = 0.01),
    // 50-500 reps gives encoding depths of ~0.33-0.83, averaging around 0.65
    assert!(
        master_best.average_encoding() > 0.5,
        "Master average encoding was {}, expected > 0.5",
        master_best.average_encoding()
    );
}

#[test]
fn test_master_attention_remaining_for_parallel_tasks() {
    let (_, _) = create_novice();
    let (mut master_lib, _) = create_master(1000);

    // Master can execute a craft chunk and still have attention for other things
    master_lib.attention_budget = 1.0;
    master_lib.attention_spent = 0.0;

    // Simulate using a deep chunk (encoding 0.85 = attention cost 0.15)
    let chunk_cost = 1.0 - 0.85; // 0.15
    assert!(master_lib.spend_attention(chunk_cost));

    // Master still has attention for quality assessment, teaching, etc.
    assert!(master_lib.attention_remaining() > 0.8);

    // Novice would need full attention just for basic operations
    // (No chunks = attention cost ~1.0 per atomic action)
}
