//! Naga entity archetype and values

use crate::core::types::{EntityId, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::social::SocialMemory;
use crate::entity::tasks::TaskQueue;
use crate::entity::thoughts::ThoughtBuffer;
use serde::{Deserialize, Serialize};

/// Naga-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NagaValues {
    pub duty: f32,
    pub curiosity: f32,
    pub territoriality: f32,
    pub venomous_rage: f32,
}

impl NagaValues {
    pub fn new() -> Self {
        Self {
            duty: 0.8,
            curiosity: 0.4,
            territoriality: 0.7,
            venomous_rage: 0.3,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.duty = rng.gen_range(0.2..0.8);
        self.curiosity = rng.gen_range(0.2..0.8);
        self.territoriality = rng.gen_range(0.2..0.8);
        self.venomous_rage = rng.gen_range(0.2..0.8);
    }
}

/// Naga archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct NagaArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<NagaValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl NagaArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: NagaValues) -> EntityId {
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
    fn test_naga_values_creation() {
        let values = NagaValues::new();
        assert!((values.duty - 0.8).abs() < 0.01);
        assert!((values.curiosity - 0.4).abs() < 0.01);
        assert!((values.territoriality - 0.7).abs() < 0.01);
        assert!((values.venomous_rage - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_naga_archetype_spawn() {
        let mut archetype = NagaArchetype::new();
        let id = archetype.spawn(
            "Test Naga".to_string(),
            Vec2::new(10.0, 20.0),
            NagaValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}
