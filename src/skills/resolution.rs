//! Action resolution using chunk system
//!
//! Resolves combat actions through chunking - finding the best matching chunk,
//! spending attention, and determining outcome variance.

use crate::skills::{
    can_afford_attention, get_chunk_definition, risks_fumble, ChunkId, ChunkLibrary, CombatContext,
    Experience,
};

/// Result of attempting an action
#[derive(Debug, Clone)]
pub enum ActionResult {
    /// Action succeeded
    Success {
        /// Skill modifier (0.0 to 1.0) affecting outcome quality
        skill_modifier: f32,
        /// Chunk used (if any)
        chunk_used: Option<ChunkId>,
    },
    /// Action succeeded critically (master execution)
    Critical {
        skill_modifier: f32,
        chunk_used: Option<ChunkId>,
    },
    /// Action failed cleanly
    Failure,
    /// Action fumbled (negative outcome possible)
    Fumble,
    /// Not enough attention to attempt
    AttentionOverload,
}

impl ActionResult {
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            ActionResult::Success { .. } | ActionResult::Critical { .. }
        )
    }

    pub fn skill_modifier(&self) -> f32 {
        match self {
            ActionResult::Success { skill_modifier, .. } => *skill_modifier,
            ActionResult::Critical { skill_modifier, .. } => *skill_modifier,
            _ => 0.0,
        }
    }
}

/// Find the best chunk for an intended action in context
///
/// Returns (ChunkId, encoding_depth) of best match, or None if no applicable chunks
pub fn find_best_chunk(
    library: &ChunkLibrary,
    action_chunks: &[ChunkId],
    context: &CombatContext,
) -> Option<(ChunkId, f32)> {
    let mut best: Option<(ChunkId, f32, f32)> = None; // (id, depth, score)

    for chunk_id in action_chunks {
        if let Some(state) = library.get_chunk(*chunk_id) {
            if let Some(def) = get_chunk_definition(*chunk_id) {
                let context_quality = context.match_quality(def.context_requirements);

                // Skip if context doesn't match at all
                if context_quality < 0.5 {
                    continue;
                }

                // Score: level matters, but encoding depth (efficiency) matters more
                // A well-practiced level 2 chunk beats a forming level 3 chunk
                let score =
                    (def.level as f32) * 3.0 + state.encoding_depth * 10.0 + context_quality * 3.0;

                if best.map_or(true, |(_, _, s)| score > s) {
                    best = Some((*chunk_id, state.encoding_depth, score));
                }
            }
        }
    }

    best.map(|(id, depth, _)| (id, depth))
}

/// Chunks applicable for offensive melee actions
pub const ATTACK_CHUNKS: &[ChunkId] = &[
    ChunkId::BasicSwing,
    ChunkId::AttackSequence,
    ChunkId::EngageMelee,
    ChunkId::HandleFlanking,
];

/// Chunks applicable for defensive actions
pub const DEFENSE_CHUNKS: &[ChunkId] = &[
    ChunkId::BasicBlock,
    ChunkId::DefendSequence,
    ChunkId::EngageMelee,
    ChunkId::HandleFlanking,
];

/// Chunks applicable for riposte (counter-attack)
pub const RIPOSTE_CHUNKS: &[ChunkId] = &[ChunkId::Riposte, ChunkId::EngageMelee];

/// Resolve an attack action
///
/// Returns result and records experience
pub fn resolve_attack(
    library: &mut ChunkLibrary,
    context: &CombatContext,
    tick: u64,
) -> ActionResult {
    resolve_action(library, ATTACK_CHUNKS, context, tick)
}

/// Resolve a defense action
pub fn resolve_defense(
    library: &mut ChunkLibrary,
    context: &CombatContext,
    tick: u64,
) -> ActionResult {
    resolve_action(library, DEFENSE_CHUNKS, context, tick)
}

/// Resolve a riposte action
pub fn resolve_riposte(
    library: &mut ChunkLibrary,
    context: &CombatContext,
    tick: u64,
) -> ActionResult {
    resolve_action(library, RIPOSTE_CHUNKS, context, tick)
}

/// Core action resolution
fn resolve_action(
    library: &mut ChunkLibrary,
    applicable_chunks: &[ChunkId],
    context: &CombatContext,
    tick: u64,
) -> ActionResult {
    // Find best chunk
    let (chunk_id, encoding_depth, attention_cost) =
        if let Some((id, depth)) = find_best_chunk(library, applicable_chunks, context) {
            (Some(id), depth, 1.0 - depth)
        } else {
            // No chunk - use atomics (very expensive)
            (None, 0.1, 0.9)
        };

    // Check attention budget
    if !can_afford_attention(library.attention_remaining(), attention_cost) {
        return ActionResult::AttentionOverload;
    }

    // Spend attention
    library.spend_attention(attention_cost);

    // Record experience
    if let Some(id) = chunk_id {
        library.record_experience(Experience {
            chunk_id: id,
            success: true, // Will be updated by caller if action fails
            tick,
        });

        // Update last used tick
        if let Some(state) = library.get_chunk_mut(id) {
            state.last_used_tick = tick;
        }
    }

    // Check fumble risk
    if risks_fumble(library.attention_remaining()) && encoding_depth < 0.3 {
        return ActionResult::Fumble;
    }

    // Determine success level
    // Higher encoding = more likely critical, lower variance
    if encoding_depth > 0.9 {
        ActionResult::Critical {
            skill_modifier: encoding_depth,
            chunk_used: chunk_id,
        }
    } else {
        ActionResult::Success {
            skill_modifier: encoding_depth,
            chunk_used: chunk_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::ContextTag;

    #[test]
    fn test_no_chunks_high_cost() {
        let mut lib = ChunkLibrary::new();
        lib.attention_budget = 1.0;

        let ctx = CombatContext::new().with_tag(ContextTag::InMelee);
        let result = resolve_attack(&mut lib, &ctx, 0);

        // Should succeed but spend most attention
        assert!(result.is_success());
        assert!(lib.attention_remaining() < 0.2);
    }

    #[test]
    fn test_veteran_low_cost() {
        let mut lib = ChunkLibrary::veteran(0);
        lib.attention_budget = 1.0;

        let ctx = CombatContext::new()
            .with_tag(ContextTag::InMelee)
            .with_tag(ContextTag::EnemyVisible);

        let result = resolve_attack(&mut lib, &ctx, 100);

        assert!(result.is_success());
        // Should have lots of attention remaining
        assert!(lib.attention_remaining() > 0.6);
    }

    #[test]
    fn test_attention_overload() {
        let mut lib = ChunkLibrary::new();
        lib.attention_budget = 0.5;
        lib.attention_spent = 0.5;

        let ctx = CombatContext::new().with_tag(ContextTag::InMelee);
        let result = resolve_attack(&mut lib, &ctx, 0);

        assert!(matches!(result, ActionResult::AttentionOverload));
    }

    #[test]
    fn test_find_best_chunk_prefers_higher_level() {
        let lib = ChunkLibrary::veteran(0);
        let ctx = CombatContext::new()
            .with_tag(ContextTag::InMelee)
            .with_tag(ContextTag::EnemyVisible);

        let best = find_best_chunk(&lib, ATTACK_CHUNKS, &ctx);

        // Should prefer EngageMelee (level 3) over BasicSwing (level 1)
        assert!(best.is_some());
        let (id, _) = best.unwrap();
        assert!(id.level() >= 2);
    }

    #[test]
    fn test_context_affects_selection() {
        let lib = ChunkLibrary::veteran(0);

        // Without MultipleEnemies, shouldn't select HandleFlanking
        let ctx = CombatContext::new().with_tag(ContextTag::InMelee);
        let best = find_best_chunk(&lib, &[ChunkId::HandleFlanking, ChunkId::BasicSwing], &ctx);

        // HandleFlanking requires MultipleEnemies - should get BasicSwing
        if let Some((id, _)) = best {
            assert_ne!(id, ChunkId::HandleFlanking);
        }
    }

    #[test]
    fn test_experience_recorded() {
        let mut lib = ChunkLibrary::trained_soldier(0);
        lib.attention_budget = 1.0;

        let ctx = CombatContext::new().with_tag(ContextTag::InMelee);
        let _ = resolve_attack(&mut lib, &ctx, 100);

        assert!(!lib.pending_experiences().is_empty());
    }
}
