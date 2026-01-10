//! Combat context tags for chunk applicability

use serde::{Deserialize, Serialize};

/// Context tags that determine which chunks are applicable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextTag {
    // Spatial
    InMelee,
    AtRange,
    Flanked,
    Flanking,
    HighGround,

    // Equipment - Melee
    HasSword,
    HasShield,
    HasPolearm,
    Armored,

    // Equipment - Ranged
    HasBow,
    HasCrossbow,
    HasThrown,
    AmmoAvailable,
    CrossbowLoaded,

    // Opponent/Target
    EnemyVisible,
    MultipleEnemies,
    TargetVisible,
    TargetInCover,

    // State
    Fresh,
    Fatigued,
}

/// A set of context tags for a combat situation
#[derive(Debug, Clone, Default)]
pub struct CombatContext {
    tags: Vec<ContextTag>,
}

impl CombatContext {
    pub fn new() -> Self {
        Self { tags: Vec::new() }
    }

    pub fn with_tag(mut self, tag: ContextTag) -> Self {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self
    }

    pub fn has(&self, tag: ContextTag) -> bool {
        self.tags.contains(&tag)
    }

    pub fn tags(&self) -> &[ContextTag] {
        &self.tags
    }

    /// Calculate match quality against requirements (0.0 to 1.0)
    pub fn match_quality(&self, requirements: &[ContextTag]) -> f32 {
        if requirements.is_empty() {
            return 1.0;
        }
        let matched = requirements.iter().filter(|r| self.has(**r)).count();
        matched as f32 / requirements.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let ctx = CombatContext::new()
            .with_tag(ContextTag::InMelee)
            .with_tag(ContextTag::HasSword);

        assert!(ctx.has(ContextTag::InMelee));
        assert!(ctx.has(ContextTag::HasSword));
        assert!(!ctx.has(ContextTag::AtRange));
    }

    #[test]
    fn test_match_quality_full() {
        let ctx = CombatContext::new()
            .with_tag(ContextTag::InMelee)
            .with_tag(ContextTag::HasSword);

        let reqs = &[ContextTag::InMelee, ContextTag::HasSword];
        assert_eq!(ctx.match_quality(reqs), 1.0);
    }

    #[test]
    fn test_match_quality_partial() {
        let ctx = CombatContext::new()
            .with_tag(ContextTag::InMelee);

        let reqs = &[ContextTag::InMelee, ContextTag::HasSword];
        assert_eq!(ctx.match_quality(reqs), 0.5);
    }

    #[test]
    fn test_match_quality_empty_requirements() {
        let ctx = CombatContext::new();
        assert_eq!(ctx.match_quality(&[]), 1.0);
    }

    #[test]
    fn test_ranged_context_tags() {
        let ctx = CombatContext::new()
            .with_tag(ContextTag::AtRange)
            .with_tag(ContextTag::HasBow)
            .with_tag(ContextTag::TargetVisible);

        assert!(ctx.has(ContextTag::AtRange));
        assert!(ctx.has(ContextTag::HasBow));
        assert!(ctx.has(ContextTag::TargetVisible));
        assert!(!ctx.has(ContextTag::HasCrossbow));
    }
}
