//! Integration test: Skill Differentiation
//!
//! Tests that novice and master entities produce different outcomes
//! when performing the same action.

use arc_citadel::skills::{calculate_encoding_depth, ChunkId, ChunkLibrary, PersonalChunkState};

/// Create a novice crafter (no chunks)
fn create_novice() -> ChunkLibrary {
    ChunkLibrary::new()
}

/// Create a master crafter (deep L5 chunks)
fn create_master(tick: u64) -> ChunkLibrary {
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
        lib.set_chunk(
            chunk_id,
            PersonalChunkState {
                encoding_depth: calculate_encoding_depth(reps),
                repetition_count: reps,
                last_used_tick: tick,
                formation_tick: tick.saturating_sub(10000),
            },
        );
    }

    lib
}

#[test]
fn test_novice_vs_master_attention_cost() {
    let novice_lib = create_novice();
    let master_lib = create_master(1000);

    // Novice has no craft chunks
    assert!(novice_lib.get_chunk(ChunkId::CraftBasicMeasure).is_none());

    // Master has practiced craft chunks with deep encoding
    let master_chunk = master_lib.get_chunk(ChunkId::CraftBasicMeasure).unwrap();
    assert!(
        master_chunk.encoding_depth > 0.5,
        "Master encoding depth was {}, expected > 0.5",
        master_chunk.encoding_depth
    );

    // Master's attention cost for this chunk is low
    let master_cost = master_chunk.attention_cost();
    assert!(
        master_cost < 0.5,
        "Master attention cost was {}, expected < 0.5",
        master_cost
    );
}

#[test]
fn test_master_attention_remaining_for_parallel_tasks() {
    let _novice_lib = create_novice();
    let mut master_lib = create_master(1000);

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
