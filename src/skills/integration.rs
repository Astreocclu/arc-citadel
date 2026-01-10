//! Integration layer connecting chunking system to action execution
//!
//! Provides functions to:
//! - Calculate skill modifier for any action
//! - Check attention budget before execution
//! - Record experience after execution

use crate::actions::catalog::ActionId;
use crate::skills::{
    action_mapping::get_chunks_for_action, calculate_attention_budget, can_afford_attention,
    get_chunk_definition, risks_fumble, ChunkComponents, ChunkId, ChunkLibrary, Experience,
};

/// Result of skill check before action execution
#[derive(Debug, Clone)]
pub struct SkillCheckResult {
    /// Skill modifier (0.1 to 1.0) affecting outcome quality
    /// Higher = better outcomes, lower variance
    pub skill_modifier: f32,
    /// Attention cost to execute this action
    pub attention_cost: f32,
    /// Chunks that will be used (for experience recording)
    pub chunks_used: Vec<ChunkId>,
    /// Whether execution can proceed
    pub can_execute: bool,
    /// If can't execute, why
    pub failure_reason: Option<SkillFailure>,
}

/// Reasons skill check might fail
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillFailure {
    /// Not enough attention remaining
    AttentionOverload,
    /// Barely enough attention - high fumble risk
    FumbleRisk,
}

/// Perform skill check for an action
///
/// Call this BEFORE executing an action to get skill_modifier.
/// If can_execute is false, handle failure appropriately.
pub fn skill_check(library: &ChunkLibrary, action: ActionId) -> SkillCheckResult {
    let required_chunks = get_chunks_for_action(action);

    // No skill requirements - execute with full competence
    if required_chunks.is_empty() {
        return SkillCheckResult {
            skill_modifier: 1.0,
            attention_cost: 0.0,
            chunks_used: Vec::new(),
            can_execute: true,
            failure_reason: None,
        };
    }

    // Calculate skill and cost from chunks
    let (attention_cost, skill_modifier, chunks_used) =
        calculate_action_skill(required_chunks, library);

    // Check attention budget
    if !can_afford_attention(library.attention_remaining(), attention_cost) {
        return SkillCheckResult {
            skill_modifier,
            attention_cost,
            chunks_used,
            can_execute: false,
            failure_reason: Some(SkillFailure::AttentionOverload),
        };
    }

    // Check fumble risk (low attention + low skill = dangerous)
    let remaining_after = library.attention_remaining() - attention_cost;
    if risks_fumble(remaining_after) && skill_modifier < 0.3 {
        return SkillCheckResult {
            skill_modifier,
            attention_cost,
            chunks_used,
            can_execute: false,
            failure_reason: Some(SkillFailure::FumbleRisk),
        };
    }

    SkillCheckResult {
        skill_modifier,
        attention_cost,
        chunks_used,
        can_execute: true,
        failure_reason: None,
    }
}

/// Calculate skill modifier and attention cost from required chunks
///
/// Returns (attention_cost, skill_modifier, chunks_actually_used)
///
/// Design principle:
/// - Attention cost is only for chunks you HAVE (conscious effort to coordinate them)
/// - Missing chunks = work instinctively with low skill, but no attention cost
/// - This allows unskilled entities to work (poorly) without attention overload
fn calculate_action_skill(
    required_chunks: &[ChunkId],
    library: &ChunkLibrary,
) -> (f32, f32, Vec<ChunkId>) {
    let mut total_cost = 0.0;
    let mut total_modifier = 1.0;
    let mut chunks_used = Vec::new();
    let mut chunks_found = 0;

    for chunk_id in required_chunks {
        if let Some(state) = library.get_chunk(*chunk_id) {
            // Has this chunk - cost and skill based on encoding depth
            let cost = 1.0 - state.encoding_depth;
            // Skill ranges from 0.5 (just learned) to 1.0 (mastered)
            let modifier = 0.5 + (state.encoding_depth * 0.5);

            total_cost += cost;
            total_modifier *= modifier;
            chunks_used.push(*chunk_id);
            chunks_found += 1;
        } else {
            // Doesn't have this chunk - check if we can decompose to sub-chunks
            if let Some(def) = get_chunk_definition(*chunk_id) {
                match &def.components {
                    ChunkComponents::Atomic => {
                        // No chunk, work instinctively: low skill, no attention cost
                        total_modifier *= 0.3;
                    }
                    ChunkComponents::Composite(sub_chunks) => {
                        // Recursively check sub-chunks
                        let (sub_cost, sub_mod, sub_used) =
                            calculate_action_skill(sub_chunks, library);
                        total_cost += sub_cost;
                        total_modifier *= sub_mod;
                        if !sub_used.is_empty() {
                            chunks_found += 1;
                        }
                        chunks_used.extend(sub_used);
                    }
                }
            } else {
                // Unknown chunk - work instinctively with low skill
                total_modifier *= 0.3;
            }
        }
    }

    // Average cost across chunks actually used (avoids divide-by-zero)
    let avg_cost = if chunks_found > 0 {
        total_cost / chunks_found as f32
    } else {
        0.0 // No chunks used = no attention cost (instinctive work)
    };

    (avg_cost, total_modifier.clamp(0.1, 1.0), chunks_used)
}

/// Spend attention for an action (call after skill_check succeeds)
pub fn spend_attention(library: &mut ChunkLibrary, cost: f32) {
    library.spend_attention(cost);
}

/// Record experience after action execution
///
/// Call this AFTER action executes with the outcome.
pub fn record_action_experience(
    library: &mut ChunkLibrary,
    chunks_used: &[ChunkId],
    success: bool,
    tick: u64,
) {
    for chunk_id in chunks_used {
        library.record_experience(Experience {
            chunk_id: *chunk_id,
            success,
            tick,
        });

        // Update last_used_tick immediately (even before learning consolidation)
        if let Some(state) = library.get_chunk_mut(*chunk_id) {
            state.last_used_tick = tick;
        }
    }
}

/// Refresh attention budget for a new decision period
///
/// Call at start of tick or other decision point.
pub fn refresh_attention(library: &mut ChunkLibrary, fatigue: f32, pain: f32, stress: f32) {
    library.attention_budget = calculate_attention_budget(fatigue, pain, stress);
    library.attention_spent = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::PersonalChunkState;

    fn setup_library_with_chunks(encoding_depth: f32) -> ChunkLibrary {
        let mut lib = ChunkLibrary::new();
        lib.attention_budget = 1.0;

        // Add basic combat and physical chunks
        for chunk_id in &[
            ChunkId::BasicSwing,
            ChunkId::BasicStance,
            ChunkId::PhysEfficientGait,
        ] {
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
    fn test_skill_check_no_chunks_required() {
        let lib = ChunkLibrary::new();
        let result = skill_check(&lib, ActionId::Rest);

        assert!(result.can_execute);
        assert_eq!(result.skill_modifier, 1.0);
        assert_eq!(result.attention_cost, 0.0);
    }

    #[test]
    fn test_skill_check_with_practiced_chunks() {
        let lib = setup_library_with_chunks(0.7);
        let result = skill_check(&lib, ActionId::Attack);

        assert!(result.can_execute);
        assert!(result.skill_modifier > 0.5);
        assert!(result.attention_cost < 0.5);
    }

    #[test]
    fn test_skill_check_novice_high_cost() {
        let lib = setup_library_with_chunks(0.1);
        let result = skill_check(&lib, ActionId::Attack);

        // Should still be able to execute but with high cost
        assert!(result.can_execute);
        assert!(result.attention_cost > 0.5);
    }

    #[test]
    fn test_skill_check_attention_overload() {
        let mut lib = setup_library_with_chunks(0.3); // Low encoding = high attention cost
        lib.attention_budget = 0.2;
        lib.attention_spent = 0.2;

        // Has chunks with high cost but no attention remaining = overload
        let result = skill_check(&lib, ActionId::Attack);

        assert!(!result.can_execute);
        assert_eq!(result.failure_reason, Some(SkillFailure::AttentionOverload));
    }

    #[test]
    fn test_skill_check_no_chunks_still_executes() {
        let lib = ChunkLibrary::new(); // No chunks at all

        // No chunks = work instinctively with no attention cost
        let result = skill_check(&lib, ActionId::Attack);

        assert!(result.can_execute);
        assert_eq!(result.attention_cost, 0.0);
        assert!(result.skill_modifier <= 0.1); // Very low skill
    }

    #[test]
    fn test_refresh_attention_resets_budget() {
        let mut lib = ChunkLibrary::new();
        lib.attention_budget = 0.5;
        lib.attention_spent = 0.4;

        refresh_attention(&mut lib, 0.0, 0.0, 0.0);

        assert_eq!(lib.attention_budget, 1.0);
        assert_eq!(lib.attention_spent, 0.0);
    }

    #[test]
    fn test_refresh_attention_fatigue_penalty() {
        let mut lib = ChunkLibrary::new();

        refresh_attention(&mut lib, 0.5, 0.0, 0.0);

        // Fatigue should reduce budget
        assert!(lib.attention_budget < 1.0);
        assert!(lib.attention_budget > 0.5);
    }

    #[test]
    fn test_record_experience() {
        let mut lib = setup_library_with_chunks(0.5);

        record_action_experience(&mut lib, &[ChunkId::BasicSwing], true, 100);

        assert!(!lib.pending_experiences().is_empty());
        assert_eq!(lib.pending_experiences()[0].chunk_id, ChunkId::BasicSwing);
        assert!(lib.pending_experiences()[0].success);
    }
}
