//! Per-entity chunk state storage

use crate::skills::{ChunkDomain, ChunkId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Personal state for a chunk the entity has encountered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalChunkState {
    /// How automatic this chunk is (0.0 to 1.0)
    /// 0.0 = conscious effort, 1.0 = fully compiled
    pub encoding_depth: f32,
    /// Times this chunk has been executed
    pub repetition_count: u32,
    /// Tick when last used (for rust decay)
    pub last_used_tick: u64,
    /// Tick when first formed
    pub formation_tick: u64,
}

impl PersonalChunkState {
    pub fn new(tick: u64) -> Self {
        Self {
            encoding_depth: 0.1, // Just learned
            repetition_count: 1,
            last_used_tick: tick,
            formation_tick: tick,
        }
    }

    /// Attention cost to execute this chunk (1.0 - depth)
    pub fn attention_cost(&self) -> f32 {
        1.0 - self.encoding_depth
    }
}

/// Experience record for learning
#[derive(Debug, Clone)]
pub struct Experience {
    pub chunk_id: ChunkId,
    pub success: bool,
    pub tick: u64,
}

/// Summary of chunks in a specific domain
#[derive(Debug, Clone)]
pub struct DomainSummary {
    pub domain: ChunkDomain,
    pub chunk_count: usize,
    pub total_encoding: f32,
    pub best_chunk: Option<(ChunkId, f32)>,
    pub highest_level: u8,
}

impl DomainSummary {
    /// Average encoding depth (0.0 if no chunks)
    pub fn average_encoding(&self) -> f32 {
        if self.chunk_count == 0 {
            0.0
        } else {
            self.total_encoding / self.chunk_count as f32
        }
    }
}

/// Per-entity skill chunk library
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChunkLibrary {
    /// Chunks this entity has formed
    #[serde(default)]
    chunks: HashMap<ChunkId, PersonalChunkState>,

    /// Current attention budget (refreshes each decision point)
    #[serde(skip)]
    pub attention_budget: f32,

    /// Attention spent this decision point
    #[serde(skip)]
    pub attention_spent: f32,

    /// Pending experiences (cleared during learning consolidation)
    #[serde(skip)]
    pending_experiences: Vec<Experience>,
}

impl ChunkLibrary {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            attention_budget: 1.0,
            attention_spent: 0.0,
            pending_experiences: Vec::new(),
        }
    }

    /// Create a library with basic universal chunks (walking, etc.)
    pub fn with_basics() -> Self {
        let lib = Self::new();
        // Fresh entity has no combat chunks - must learn everything
        lib
    }

    /// Create a library for a trained soldier
    pub fn trained_soldier(tick: u64) -> Self {
        let mut lib = Self::new();

        // Level 1 chunks - well practiced
        lib.chunks.insert(
            ChunkId::BasicSwing,
            PersonalChunkState {
                encoding_depth: 0.6,
                repetition_count: 100,
                last_used_tick: tick,
                formation_tick: tick.saturating_sub(1000),
            },
        );
        lib.chunks.insert(
            ChunkId::BasicBlock,
            PersonalChunkState {
                encoding_depth: 0.5,
                repetition_count: 80,
                last_used_tick: tick,
                formation_tick: tick.saturating_sub(1000),
            },
        );
        lib.chunks.insert(
            ChunkId::BasicStance,
            PersonalChunkState {
                encoding_depth: 0.7,
                repetition_count: 150,
                last_used_tick: tick,
                formation_tick: tick.saturating_sub(1000),
            },
        );

        // Level 2 - forming
        lib.chunks.insert(
            ChunkId::AttackSequence,
            PersonalChunkState {
                encoding_depth: 0.3,
                repetition_count: 30,
                last_used_tick: tick,
                formation_tick: tick.saturating_sub(500),
            },
        );

        lib
    }

    /// Create a library for a veteran
    pub fn veteran(tick: u64) -> Self {
        let mut lib = Self::trained_soldier(tick);

        // Deepen level 1 chunks
        if let Some(state) = lib.chunks.get_mut(&ChunkId::BasicSwing) {
            state.encoding_depth = 0.85;
            state.repetition_count = 500;
        }
        if let Some(state) = lib.chunks.get_mut(&ChunkId::BasicBlock) {
            state.encoding_depth = 0.85;
            state.repetition_count = 500;
        }
        if let Some(state) = lib.chunks.get_mut(&ChunkId::BasicStance) {
            state.encoding_depth = 0.9;
            state.repetition_count = 600;
        }

        // Level 2 chunks - practiced
        lib.chunks.insert(
            ChunkId::AttackSequence,
            PersonalChunkState {
                encoding_depth: 0.7,
                repetition_count: 200,
                last_used_tick: tick,
                formation_tick: tick.saturating_sub(2000),
            },
        );
        lib.chunks.insert(
            ChunkId::DefendSequence,
            PersonalChunkState {
                encoding_depth: 0.65,
                repetition_count: 180,
                last_used_tick: tick,
                formation_tick: tick.saturating_sub(2000),
            },
        );
        lib.chunks.insert(
            ChunkId::Riposte,
            PersonalChunkState {
                encoding_depth: 0.5,
                repetition_count: 100,
                last_used_tick: tick,
                formation_tick: tick.saturating_sub(1500),
            },
        );

        // Level 3 - forming
        lib.chunks.insert(
            ChunkId::EngageMelee,
            PersonalChunkState {
                encoding_depth: 0.3,
                repetition_count: 50,
                last_used_tick: tick,
                formation_tick: tick.saturating_sub(500),
            },
        );

        lib
    }

    // ========================================================================
    // RANGED COMBAT LIBRARIES
    // ========================================================================

    /// Create a trained archer's chunk library
    pub fn trained_archer(formation_tick: u64) -> Self {
        let mut lib = Self::new();

        // Level 1 fundamentals
        lib.chunks.insert(
            ChunkId::DrawBow,
            PersonalChunkState {
                encoding_depth: 0.5,
                repetition_count: 50,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );
        lib.chunks.insert(
            ChunkId::BasicAim,
            PersonalChunkState {
                encoding_depth: 0.5,
                repetition_count: 40,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );

        // Level 2 - standard shot
        lib.chunks.insert(
            ChunkId::LooseArrow,
            PersonalChunkState {
                encoding_depth: 0.4,
                repetition_count: 60,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );
        lib.chunks.insert(
            ChunkId::SnapShot,
            PersonalChunkState {
                encoding_depth: 0.3,
                repetition_count: 30,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );

        lib
    }

    /// Create a veteran archer's chunk library
    pub fn veteran_archer(formation_tick: u64) -> Self {
        let mut lib = Self::trained_archer(formation_tick);

        // Upgrade existing chunks
        lib.chunks.insert(
            ChunkId::DrawBow,
            PersonalChunkState {
                encoding_depth: 0.85,
                repetition_count: 300,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );
        lib.chunks.insert(
            ChunkId::BasicAim,
            PersonalChunkState {
                encoding_depth: 0.8,
                repetition_count: 250,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );
        lib.chunks.insert(
            ChunkId::LooseArrow,
            PersonalChunkState {
                encoding_depth: 0.75,
                repetition_count: 200,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );
        lib.chunks.insert(
            ChunkId::SnapShot,
            PersonalChunkState {
                encoding_depth: 0.7,
                repetition_count: 150,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );

        // Level 3 - advanced techniques
        lib.chunks.insert(
            ChunkId::RapidFire,
            PersonalChunkState {
                encoding_depth: 0.5,
                repetition_count: 100,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );
        lib.chunks.insert(
            ChunkId::SniperShot,
            PersonalChunkState {
                encoding_depth: 0.5,
                repetition_count: 100,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );

        lib
    }

    /// Create a trained crossbowman's chunk library
    pub fn trained_crossbowman(formation_tick: u64) -> Self {
        let mut lib = Self::new();

        // Level 1 fundamentals
        lib.chunks.insert(
            ChunkId::LoadCrossbow,
            PersonalChunkState {
                encoding_depth: 0.5,
                repetition_count: 30,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );
        lib.chunks.insert(
            ChunkId::BasicAim,
            PersonalChunkState {
                encoding_depth: 0.5,
                repetition_count: 40,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );

        // Level 2 - crossbow shot (easy ceiling)
        lib.chunks.insert(
            ChunkId::CrossbowShot,
            PersonalChunkState {
                encoding_depth: 0.6, // Higher floor than bow
                repetition_count: 50,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );

        lib
    }

    /// Create a veteran crossbowman's chunk library
    pub fn veteran_crossbowman(formation_tick: u64) -> Self {
        let mut lib = Self::trained_crossbowman(formation_tick);

        // Crossbows have lower ceiling - max out faster
        lib.chunks.insert(
            ChunkId::LoadCrossbow,
            PersonalChunkState {
                encoding_depth: 0.8, // Not as high as bow mastery
                repetition_count: 100,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );
        lib.chunks.insert(
            ChunkId::BasicAim,
            PersonalChunkState {
                encoding_depth: 0.75,
                repetition_count: 120,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );
        lib.chunks.insert(
            ChunkId::CrossbowShot,
            PersonalChunkState {
                encoding_depth: 0.75, // Lower ceiling than veteran archer
                repetition_count: 100,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );

        lib
    }

    /// Create a trained thrower's chunk library (javelins, axes)
    pub fn trained_thrower(formation_tick: u64) -> Self {
        let mut lib = Self::new();

        lib.chunks.insert(
            ChunkId::BasicThrow,
            PersonalChunkState {
                encoding_depth: 0.5,
                repetition_count: 30,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );
        lib.chunks.insert(
            ChunkId::BasicAim,
            PersonalChunkState {
                encoding_depth: 0.4,
                repetition_count: 25,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );
        lib.chunks.insert(
            ChunkId::AimedThrow,
            PersonalChunkState {
                encoding_depth: 0.4,
                repetition_count: 40,
                last_used_tick: formation_tick,
                formation_tick,
            },
        );

        lib
    }

    /// Check if entity has a chunk
    pub fn has_chunk(&self, id: ChunkId) -> bool {
        self.chunks.contains_key(&id)
    }

    /// Get chunk state (if formed)
    pub fn get_chunk(&self, id: ChunkId) -> Option<&PersonalChunkState> {
        self.chunks.get(&id)
    }

    /// Get mutable chunk state
    pub fn get_chunk_mut(&mut self, id: ChunkId) -> Option<&mut PersonalChunkState> {
        self.chunks.get_mut(&id)
    }

    /// Insert or update a chunk
    pub fn set_chunk(&mut self, id: ChunkId, state: PersonalChunkState) {
        self.chunks.insert(id, state);
    }

    /// Get all formed chunks
    pub fn chunks(&self) -> &HashMap<ChunkId, PersonalChunkState> {
        &self.chunks
    }

    /// Get mutable access to chunks
    pub fn chunks_mut(&mut self) -> &mut HashMap<ChunkId, PersonalChunkState> {
        &mut self.chunks
    }

    /// Remaining attention this decision point
    pub fn attention_remaining(&self) -> f32 {
        (self.attention_budget - self.attention_spent).max(0.0)
    }

    /// Spend attention (returns false if insufficient)
    pub fn spend_attention(&mut self, cost: f32) -> bool {
        if cost > self.attention_remaining() {
            return false;
        }
        self.attention_spent += cost;
        true
    }

    /// Record an experience for later learning
    pub fn record_experience(&mut self, exp: Experience) {
        self.pending_experiences.push(exp);
    }

    /// Get pending experiences
    pub fn pending_experiences(&self) -> &[Experience] {
        &self.pending_experiences
    }

    /// Clear pending experiences (after consolidation)
    pub fn clear_experiences(&mut self) {
        self.pending_experiences.clear();
    }

    /// Get best combat chunk (highest level with good depth)
    pub fn best_combat_chunk(&self) -> Option<(ChunkId, f32)> {
        self.chunks
            .iter()
            .max_by(|a, b| {
                let score_a = a.0.level() as f32 * 10.0 + a.1.encoding_depth * 5.0;
                let score_b = b.0.level() as f32 * 10.0 + b.1.encoding_depth * 5.0;
                score_a.partial_cmp(&score_b).unwrap()
            })
            .map(|(id, state)| (*id, state.encoding_depth))
    }

    /// Get summary of chunks in a specific domain
    pub fn domain_summary(&self, domain: ChunkDomain) -> DomainSummary {
        let domain_chunks: Vec<_> = self
            .chunks
            .iter()
            .filter(|(id, _)| id.domain() == domain)
            .collect();

        let chunk_count = domain_chunks.len();
        let total_encoding: f32 = domain_chunks
            .iter()
            .map(|(_, state)| state.encoding_depth)
            .sum();

        let best_chunk = domain_chunks
            .iter()
            .max_by(|a, b| {
                let score_a = a.0.level() as f32 * 10.0 + a.1.encoding_depth * 5.0;
                let score_b = b.0.level() as f32 * 10.0 + b.1.encoding_depth * 5.0;
                score_a.partial_cmp(&score_b).unwrap()
            })
            .map(|(id, state)| (**id, state.encoding_depth));

        let highest_level = domain_chunks
            .iter()
            .map(|(id, _)| id.level())
            .max()
            .unwrap_or(0);

        DomainSummary {
            domain,
            chunk_count,
            total_encoding,
            best_chunk,
            highest_level,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_library() {
        let lib = ChunkLibrary::new();
        assert!(!lib.has_chunk(ChunkId::BasicSwing));
        assert_eq!(lib.attention_budget, 1.0);
    }

    #[test]
    fn test_trained_soldier_has_chunks() {
        let lib = ChunkLibrary::trained_soldier(1000);
        assert!(lib.has_chunk(ChunkId::BasicSwing));
        assert!(lib.has_chunk(ChunkId::BasicStance));
        assert!(lib.has_chunk(ChunkId::AttackSequence));
    }

    #[test]
    fn test_attention_spending() {
        let mut lib = ChunkLibrary::new();
        lib.attention_budget = 1.0;

        assert!(lib.spend_attention(0.3));
        assert_eq!(lib.attention_remaining(), 0.7);

        assert!(lib.spend_attention(0.5));
        assert!(!lib.spend_attention(0.3)); // Not enough
    }

    #[test]
    fn test_attention_cost_from_depth() {
        let state = PersonalChunkState {
            encoding_depth: 0.8,
            repetition_count: 100,
            last_used_tick: 0,
            formation_tick: 0,
        };
        assert!((state.attention_cost() - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_experience_recording() {
        let mut lib = ChunkLibrary::new();
        lib.record_experience(Experience {
            chunk_id: ChunkId::BasicSwing,
            success: true,
            tick: 100,
        });

        assert_eq!(lib.pending_experiences().len(), 1);
        lib.clear_experiences();
        assert_eq!(lib.pending_experiences().len(), 0);
    }

    #[test]
    fn test_trained_archer_library() {
        let lib = ChunkLibrary::trained_archer(0);

        // Should have bow fundamentals
        assert!(lib.has_chunk(ChunkId::DrawBow));
        assert!(lib.has_chunk(ChunkId::BasicAim));
        assert!(lib.has_chunk(ChunkId::LooseArrow));

        // Should NOT have crossbow chunks
        assert!(!lib.has_chunk(ChunkId::LoadCrossbow));
        assert!(!lib.has_chunk(ChunkId::CrossbowShot));
    }

    #[test]
    fn test_trained_crossbowman_library() {
        let lib = ChunkLibrary::trained_crossbowman(0);

        // Should have crossbow fundamentals
        assert!(lib.has_chunk(ChunkId::LoadCrossbow));
        assert!(lib.has_chunk(ChunkId::BasicAim));
        assert!(lib.has_chunk(ChunkId::CrossbowShot));

        // Should NOT have bow chunks
        assert!(!lib.has_chunk(ChunkId::DrawBow));
        assert!(!lib.has_chunk(ChunkId::LooseArrow));
    }

    #[test]
    fn test_veteran_archer_has_advanced_chunks() {
        let lib = ChunkLibrary::veteran_archer(0);

        // Should have advanced bow chunks
        assert!(lib.has_chunk(ChunkId::RapidFire));
        assert!(lib.has_chunk(ChunkId::SniperShot));

        // With high encoding depth
        assert!(lib.get_chunk(ChunkId::LooseArrow).unwrap().encoding_depth > 0.7);
    }

    #[test]
    fn test_trained_thrower_library() {
        let lib = ChunkLibrary::trained_thrower(0);

        assert!(lib.has_chunk(ChunkId::BasicThrow));
        assert!(lib.has_chunk(ChunkId::BasicAim));
        assert!(lib.has_chunk(ChunkId::AimedThrow));
    }

    #[test]
    fn test_crossbow_lower_ceiling_than_bow() {
        let archer = ChunkLibrary::veteran_archer(0);
        let crossbowman = ChunkLibrary::veteran_crossbowman(0);

        // Veteran archer LooseArrow vs veteran crossbowman CrossbowShot
        let archer_depth = archer.get_chunk(ChunkId::LooseArrow).unwrap().encoding_depth;
        let xbow_depth = crossbowman
            .get_chunk(ChunkId::CrossbowShot)
            .unwrap()
            .encoding_depth;

        // At veteran level, archer should have higher ceiling
        assert!(
            archer_depth >= xbow_depth,
            "Archer depth {} should be >= crossbow depth {}",
            archer_depth,
            xbow_depth
        );
    }

    #[test]
    fn test_domain_summary_empty() {
        use crate::skills::ChunkDomain;

        let lib = ChunkLibrary::new();
        let summary = lib.domain_summary(ChunkDomain::Combat);

        assert_eq!(summary.chunk_count, 0);
        assert_eq!(summary.total_encoding, 0.0);
        assert!(summary.best_chunk.is_none());
    }

    #[test]
    fn test_domain_summary_with_chunks() {
        use crate::skills::ChunkDomain;

        let lib = ChunkLibrary::trained_soldier(1000);
        let summary = lib.domain_summary(ChunkDomain::Combat);

        assert!(summary.chunk_count >= 3);
        assert!(summary.total_encoding > 0.0);
        assert!(summary.best_chunk.is_some());
        assert!(summary.average_encoding() > 0.0);
    }
}
