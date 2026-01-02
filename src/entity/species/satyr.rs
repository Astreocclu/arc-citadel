//! Satyr entity archetype and values

use serde::{Deserialize, Serialize};
use crate::core::types::{EntityId, Vec2};
use crate::entity::needs::Needs;
use crate::entity::thoughts::ThoughtBuffer;
use crate::entity::tasks::TaskQueue;
use crate::entity::body::BodyState;
use crate::entity::social::SocialMemory;

/// Satyr-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SatyrValues {
    pub hedonism: f32,
    pub mischief: f32,
    pub charm: f32,
    pub cowardice: f32,
    pub nature_bond: f32,
}

impl SatyrValues {
    pub fn new() -> Self {
        Self {
            hedonism: 0.8,
            mischief: 0.7,
            charm: 0.6,
            cowardice: 0.5,
            nature_bond: 0.6,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.hedonism = rng.gen_range(0.2..0.8);
        self.mischief = rng.gen_range(0.2..0.8);
        self.charm = rng.gen_range(0.2..0.8);
        self.cowardice = rng.gen_range(0.2..0.8);
        self.nature_bond = rng.gen_range(0.2..0.8);
    }
}

/// Satyr archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct SatyrArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<SatyrValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl SatyrArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: SatyrValues) -> EntityId {
        let id = EntityId::new();
        self.ids.push(id);
        self.names.push(name);
        self.positions.push(position);
        self.velocities.push(Vec2::default());
        self.body_states.push(BodyState::default());
        self.needs.push(Needs::default());
        self.thoughts.push(ThoughtBuffer::new());
        self.values.push(values);
        self.task_queues.push(TaskQueue::new());
        self.alive.push(true);
        self.social_memories.push(SocialMemory::default());
        id
    }

    pub fn index_of(&self, id: EntityId) -> Option<usize> {
        self.ids.iter().position(|&eid| eid == id)
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn alive_count(&self) -> usize {
        self.alive.iter().filter(|&&a| a).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_satyr_values_creation() {
        let values = SatyrValues::new();
        assert!((values.hedonism - 0.8).abs() < 0.01);
        assert!((values.mischief - 0.7).abs() < 0.01);
        assert!((values.charm - 0.6).abs() < 0.01);
        assert!((values.cowardice - 0.5).abs() < 0.01);
        assert!((values.nature_bond - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_satyr_archetype_spawn() {
        let mut archetype = SatyrArchetype::new();
        let id = archetype.spawn(
            "Test Satyr".to_string(),
            Vec2::new(10.0, 20.0),
            SatyrValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}