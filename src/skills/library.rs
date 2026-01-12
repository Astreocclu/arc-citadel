//! Per-entity chunk state storage

use crate::skills::ChunkId;
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

    /// Create a library for a trained worker (builders, crafters, gatherers)
    pub fn trained_worker(tick: u64) -> Self {
        let mut lib = Self::new();

        // Physical labor chunks - well practiced
        lib.chunks.insert(
            ChunkId::PhysSustainedLabor,
            PersonalChunkState {
                encoding_depth: 0.8,
                repetition_count: 200,
                last_used_tick: tick,
                formation_tick: tick.saturating_sub(2000),
            },
        );

        // Crafting chunks - practiced
        lib.chunks.insert(
            ChunkId::CraftBasicMeasure,
            PersonalChunkState {
                encoding_depth: 0.8,
                repetition_count: 150,
                last_used_tick: tick,
                formation_tick: tick.saturating_sub(1500),
            },
        );
        lib.chunks.insert(
            ChunkId::CraftBasicCut,
            PersonalChunkState {
                encoding_depth: 0.7,
                repetition_count: 100,
                last_used_tick: tick,
                formation_tick: tick.saturating_sub(1000),
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
}
