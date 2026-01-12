//! Maps ActionId to required skill chunks
//!
//! Each action requires certain chunks for skilled execution.
//! Actions without mappings execute without skill checks.

use crate::actions::catalog::ActionId;
use crate::skills::ChunkId;

/// Get the chunks required for an action
///
/// Returns empty slice if action has no skill requirements
pub fn get_chunks_for_action(action: ActionId) -> &'static [ChunkId] {
    match action {
        // === MOVEMENT ===
        ActionId::MoveTo => &[ChunkId::PhysEfficientGait],
        ActionId::Follow => &[ChunkId::PhysEfficientGait],
        ActionId::Flee => &[ChunkId::PhysDistanceRunning],

        // === SURVIVAL ===
        // Rest, Eat, SeekSafety require no skill - instinctive
        ActionId::Rest | ActionId::Eat | ActionId::SeekSafety => &[],

        // === WORK ===
        // Build/Craft/Repair include multiple chunks so specialists outperform generalists
        // Peasants have: PhysSustainedLabor, CraftBasicCut
        // Laborers also have: CraftBasicMeasure, CraftBasicJoin
        // Craftsmen have all at higher levels
        ActionId::Build => &[
            ChunkId::PhysSustainedLabor,
            ChunkId::CraftBasicMeasure,
            ChunkId::CraftBasicCut,
            ChunkId::CraftBasicJoin,
        ],
        ActionId::Craft => &[
            ChunkId::CraftBasicMeasure,
            ChunkId::CraftBasicCut,
            ChunkId::CraftBasicJoin,
        ],
        ActionId::Gather => &[ChunkId::PhysSustainedLabor],
        ActionId::Repair => &[ChunkId::CraftBasicMeasure, ChunkId::CraftBasicCut],

        // === SOCIAL ===
        ActionId::TalkTo => &[ChunkId::SocialActiveListening, ChunkId::SocialBuildRapport],
        ActionId::Help => &[ChunkId::SocialActiveListening],
        ActionId::Trade => &[ChunkId::SocialNegotiateTerms, ChunkId::SocialReadReaction],

        // === COMBAT ===
        ActionId::Attack => &[ChunkId::BasicSwing, ChunkId::BasicStance],
        ActionId::Defend => &[ChunkId::BasicBlock, ChunkId::BasicStance],
        ActionId::Charge => &[ChunkId::BasicSwing, ChunkId::PhysDistanceRunning],
        ActionId::HoldPosition => &[ChunkId::BasicStance],

        // === IDLE ===
        // Idle actions require no skill
        ActionId::IdleWander | ActionId::IdleObserve => &[],
    }
}

/// Check if an action requires skill (has chunk mappings)
pub fn action_requires_skill(action: ActionId) -> bool {
    !get_chunks_for_action(action).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_actions_have_chunks() {
        assert!(!get_chunks_for_action(ActionId::Attack).is_empty());
        assert!(!get_chunks_for_action(ActionId::Defend).is_empty());
    }

    #[test]
    fn test_idle_actions_have_no_chunks() {
        assert!(get_chunks_for_action(ActionId::IdleWander).is_empty());
        assert!(get_chunks_for_action(ActionId::IdleObserve).is_empty());
    }

    #[test]
    fn test_survival_actions_instinctive() {
        assert!(get_chunks_for_action(ActionId::Rest).is_empty());
        assert!(get_chunks_for_action(ActionId::Eat).is_empty());
    }

    #[test]
    fn test_work_actions_have_chunks() {
        assert!(!get_chunks_for_action(ActionId::Build).is_empty());
        assert!(!get_chunks_for_action(ActionId::Craft).is_empty());
    }

    #[test]
    fn test_action_requires_skill_helper() {
        assert!(action_requires_skill(ActionId::Attack));
        assert!(action_requires_skill(ActionId::Build));
        assert!(!action_requires_skill(ActionId::Rest));
        assert!(!action_requires_skill(ActionId::IdleWander));
    }
}
