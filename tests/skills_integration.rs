//! Integration tests for hierarchical chunking skill system

use arc_citadel::combat::{CombatSkill, SkillLevel};
use arc_citadel::skills::{
    calculate_attention_budget, process_learning, resolve_attack, ChunkId, ChunkLibrary,
    CombatContext, ContextTag, PersonalChunkState,
};

/// Test 1: Fresh conscript spends all attention on basic action
#[test]
fn test_conscript_attention_overload() {
    let mut lib = ChunkLibrary::new();
    lib.attention_budget = 1.0;

    let ctx = CombatContext::new().with_tag(ContextTag::InMelee);

    // First attack should succeed but spend most attention
    let result1 = resolve_attack(&mut lib, &ctx, 0);
    assert!(result1.is_success());

    // Should have very little attention left
    assert!(lib.attention_remaining() < 0.2);

    // Second attack should fail due to attention overload
    let result2 = resolve_attack(&mut lib, &ctx, 1);
    assert!(matches!(
        result2,
        arc_citadel::skills::ActionResult::AttentionOverload
    ));
}

/// Test 2: Veteran executes actions cheaply
#[test]
fn test_veteran_has_bandwidth() {
    let mut lib = ChunkLibrary::veteran(0);
    lib.attention_budget = 1.0;

    let ctx = CombatContext::new()
        .with_tag(ContextTag::InMelee)
        .with_tag(ContextTag::EnemyVisible);

    // Attack should be cheap
    let result1 = resolve_attack(&mut lib, &ctx, 100);
    assert!(result1.is_success());
    assert!(lib.attention_remaining() > 0.5);

    // Can afford multiple actions
    let result2 = resolve_attack(&mut lib, &ctx, 101);
    assert!(result2.is_success());
    assert!(lib.attention_remaining() > 0.0);
}

/// Test 3: SkillLevel reflects chunk mastery
#[test]
fn test_skill_level_progression() {
    // Conscript = Novice
    let lib = ChunkLibrary::new();
    let skill = CombatSkill::from_chunk_library(&lib);
    assert_eq!(skill.level, SkillLevel::Novice);

    // Trained soldier >= Trained
    let lib = ChunkLibrary::trained_soldier(0);
    let skill = CombatSkill::from_chunk_library(&lib);
    assert!(skill.level >= SkillLevel::Trained);

    // Veteran >= Veteran
    let lib = ChunkLibrary::veteran(0);
    let skill = CombatSkill::from_chunk_library(&lib);
    assert!(skill.level >= SkillLevel::Veteran);
}

/// Test 4: Learning increases encoding depth
#[test]
fn test_learning_progression() {
    use arc_citadel::skills::calculate_encoding_depth;

    // Create a library with a chunk at a consistent encoding depth based on rep count
    // trained_soldier has inconsistent depth/rep values, so we create our own
    let mut lib = ChunkLibrary::new();
    let initial_reps = 50;
    lib.set_chunk(
        ChunkId::BasicSwing,
        PersonalChunkState {
            encoding_depth: calculate_encoding_depth(initial_reps),
            repetition_count: initial_reps,
            last_used_tick: 0,
            formation_tick: 0,
        },
    );

    let initial_depth = lib.get_chunk(ChunkId::BasicSwing).unwrap().encoding_depth;
    let initial_reps_check = lib.get_chunk(ChunkId::BasicSwing).unwrap().repetition_count;

    // Simulate 50 successful swings
    for tick in 1..=50 {
        lib.attention_budget = 1.0;
        lib.attention_spent = 0.0;

        let ctx = CombatContext::new().with_tag(ContextTag::InMelee);
        let _ = resolve_attack(&mut lib, &ctx, tick as u64);
        process_learning(&mut lib, tick as u64);
    }

    let final_depth = lib.get_chunk(ChunkId::BasicSwing).unwrap().encoding_depth;
    let final_reps = lib.get_chunk(ChunkId::BasicSwing).unwrap().repetition_count;

    // After 50 more successful uses, rep count should increase
    assert!(final_reps > initial_reps_check);

    // And encoding depth should increase (since depth is calculated from rep count)
    assert!(final_depth > initial_depth);
}

/// Test 5: Fatigue reduces attention budget
#[test]
fn test_fatigue_reduces_attention() {
    let fresh = calculate_attention_budget(0.0, 0.0, 0.0);
    let tired = calculate_attention_budget(0.5, 0.0, 0.0);
    let exhausted = calculate_attention_budget(1.0, 0.0, 0.0);

    assert_eq!(fresh, 1.0);
    assert!(tired < fresh);
    assert!(exhausted < tired);
    assert!(exhausted >= 0.2); // Minimum floor
}

/// Test 6: Chunk formation from prerequisites
#[test]
fn test_chunk_formation() {
    let mut lib = ChunkLibrary::new();

    // Give prerequisites at sufficient depth
    lib.set_chunk(
        ChunkId::BasicStance,
        PersonalChunkState {
            encoding_depth: 0.5,
            repetition_count: 50,
            last_used_tick: 0,
            formation_tick: 0,
        },
    );
    lib.set_chunk(
        ChunkId::BasicSwing,
        PersonalChunkState {
            encoding_depth: 0.5,
            repetition_count: 50,
            last_used_tick: 0,
            formation_tick: 0,
        },
    );

    assert!(!lib.has_chunk(ChunkId::AttackSequence));

    process_learning(&mut lib, 1000);

    assert!(lib.has_chunk(ChunkId::AttackSequence));
}

/// Test 7: Qualitative difference between novice and master
#[test]
fn test_qualitative_skill_difference() {
    let ctx = CombatContext::new()
        .with_tag(ContextTag::InMelee)
        .with_tag(ContextTag::EnemyVisible)
        .with_tag(ContextTag::MultipleEnemies);

    // Conscript: should struggle to execute even one action well
    let mut conscript = ChunkLibrary::new();
    conscript.attention_budget = 1.0;
    let result = resolve_attack(&mut conscript, &ctx, 0);
    let conscript_remaining = conscript.attention_remaining();
    let conscript_skill = result.skill_modifier();

    // Veteran: should execute easily with attention to spare
    let mut veteran = ChunkLibrary::veteran(0);
    veteran.attention_budget = 1.0;
    let result = resolve_attack(&mut veteran, &ctx, 0);
    let veteran_remaining = veteran.attention_remaining();
    let veteran_skill = result.skill_modifier();

    // Veteran should have:
    // 1. More attention remaining
    assert!(veteran_remaining > conscript_remaining + 0.3);

    // 2. Higher skill modifier (better execution quality)
    assert!(veteran_skill > conscript_skill + 0.3);
}
