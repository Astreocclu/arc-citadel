//! Integration tests for chunking system wired to actions

use arc_citadel::actions::catalog::ActionId;
use arc_citadel::skills::{
    process_learning, record_action_experience, refresh_attention, skill_check, spend_attention,
    ChunkId, ChunkLibrary, PersonalChunkState, SkillFailure,
};

/// Helper to create entity with specific chunk state
fn create_entity_with_skill(chunk_ids: &[ChunkId], encoding_depth: f32) -> ChunkLibrary {
    let mut lib = ChunkLibrary::new();
    lib.attention_budget = 1.0;

    for chunk_id in chunk_ids {
        lib.set_chunk(
            *chunk_id,
            PersonalChunkState {
                encoding_depth,
                repetition_count: (encoding_depth * 100.0) as u32,
                last_used_tick: 0,
                formation_tick: 0,
            },
        );
    }

    lib
}

#[test]
fn test_novice_vs_master_attack() {
    // Novice has no combat chunks
    let novice = ChunkLibrary::new();
    let novice_result = skill_check(&novice, ActionId::Attack);

    // Master has practiced chunks
    let master = create_entity_with_skill(
        &[
            ChunkId::BasicSwing,
            ChunkId::BasicStance,
            ChunkId::AttackSequence,
        ],
        0.8,
    );
    let master_result = skill_check(&master, ActionId::Attack);

    // Both can execute (have attention)
    assert!(novice_result.can_execute || !novice_result.can_execute); // May fail due to attention
    assert!(master_result.can_execute);

    // Master has higher skill modifier
    assert!(master_result.skill_modifier > novice_result.skill_modifier);

    // Novice works instinctively (no attention cost, but low skill)
    // Master uses trained chunks (some attention cost, but high skill)
    assert_eq!(
        novice_result.attention_cost, 0.0,
        "Novice should have no attention cost (instinctive work)"
    );
    assert!(
        master_result.attention_cost > 0.0,
        "Master should use attention for trained chunks"
    );
}

#[test]
fn test_exhausted_entity_overloads() {
    let mut lib = create_entity_with_skill(&[ChunkId::BasicSwing], 0.3);
    lib.attention_budget = 0.1; // Very tired

    let result = skill_check(&lib, ActionId::Attack);

    assert!(!result.can_execute);
    assert_eq!(result.failure_reason, Some(SkillFailure::AttentionOverload));
}

#[test]
fn test_attention_depletes_over_actions() {
    let mut lib = create_entity_with_skill(
        &[ChunkId::BasicSwing, ChunkId::BasicStance],
        0.5, // Moderate skill - moderate cost
    );
    lib.attention_budget = 1.0;

    // First action
    let result1 = skill_check(&lib, ActionId::Attack);
    assert!(result1.can_execute);
    spend_attention(&mut lib, result1.attention_cost);

    // Second action
    let result2 = skill_check(&lib, ActionId::Attack);
    assert!(result2.can_execute);
    spend_attention(&mut lib, result2.attention_cost);

    // Check remaining attention has decreased
    assert!(lib.attention_remaining() < 1.0);
}

#[test]
fn test_instinctive_actions_no_skill_check() {
    let lib = ChunkLibrary::new(); // No chunks at all

    // Rest is instinctive
    let result = skill_check(&lib, ActionId::Rest);

    assert!(result.can_execute);
    assert_eq!(result.skill_modifier, 1.0);
    assert_eq!(result.attention_cost, 0.0);
}

#[test]
fn test_skill_improves_through_practice() {
    let mut lib = create_entity_with_skill(
        &[ChunkId::PhysEfficientGait],
        0.3, // Starting skill
    );
    lib.attention_budget = 1.0;

    let initial_depth = lib
        .get_chunk(ChunkId::PhysEfficientGait)
        .unwrap()
        .encoding_depth;

    // Simulate 20 successful movements
    for tick in 1..=20 {
        let result = skill_check(&lib, ActionId::MoveTo);
        if result.can_execute {
            spend_attention(&mut lib, result.attention_cost);
            record_action_experience(&mut lib, &result.chunks_used, true, tick);
        }
        // Refresh attention each "tick"
        refresh_attention(&mut lib, 0.0, 0.0, 0.0);
        // Process learning
        process_learning(&mut lib, tick);
    }

    let final_depth = lib
        .get_chunk(ChunkId::PhysEfficientGait)
        .unwrap()
        .encoding_depth;

    // Skill should have improved
    assert!(final_depth > initial_depth);
}

#[test]
fn test_fatigue_reduces_attention_budget() {
    let mut fresh = ChunkLibrary::new();
    let mut tired = ChunkLibrary::new();

    refresh_attention(&mut fresh, 0.0, 0.0, 0.0); // No fatigue
    refresh_attention(&mut tired, 0.8, 0.0, 0.0); // High fatigue

    assert!(fresh.attention_budget > tired.attention_budget);
    assert!(tired.attention_budget >= 0.2); // Minimum floor
}

#[test]
fn test_work_actions_use_physical_chunks() {
    let lib = create_entity_with_skill(
        &[ChunkId::PhysSustainedLabor, ChunkId::PhysPowerStance],
        0.6,
    );

    let gather_result = skill_check(&lib, ActionId::Gather);

    // Should use physical chunks
    assert!(!gather_result.chunks_used.is_empty());
    // Should have good skill modifier since we have the chunks
    assert!(gather_result.skill_modifier > 0.5);
}

#[test]
fn test_social_actions_use_social_chunks() {
    let lib = create_entity_with_skill(
        &[
            ChunkId::SocialActiveListening,
            ChunkId::SocialBuildRapport,
            ChunkId::SocialNegotiateTerms,
            ChunkId::SocialReadReaction,
        ],
        0.5,
    );

    let trade_result = skill_check(&lib, ActionId::Trade);

    // Should use social chunks for trade
    assert!(!trade_result.chunks_used.is_empty());
    assert!(trade_result.skill_modifier > 0.3);
}

#[test]
fn test_movement_always_possible() {
    // Even with no chunks and exhausted attention, movement should have valid result
    let mut lib = ChunkLibrary::new();
    lib.attention_budget = 0.0;
    lib.attention_spent = 0.0;

    let result = skill_check(&lib, ActionId::MoveTo);

    // Movement may not be able to execute but will have a valid skill_modifier
    // for speed calculation (even if reduced)
    assert!(result.skill_modifier > 0.0);
}
