//! Minotaur entity archetype and values

use serde::{Deserialize, Serialize};
use crate::core::types::{EntityId, Vec2};
use crate::entity::needs::Needs;
use crate::entity::thoughts::ThoughtBuffer;
use crate::entity::tasks::TaskQueue;
use crate::entity::body::BodyState;
use crate::entity::social::SocialMemory;

/// Minotaur-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MinotaurValues {
    pub rage: f32,
    pub isolation: f32,
    pub territoriality: f32,
    pub hunger: f32,
    pub cunning: f32,
}

impl MinotaurValues {
    pub fn new() -> Self {
        Self {
            rage: 0.7,
            isolation: 0.6,
            territoriality: 0.8,
            hunger: 0.5,
            cunning: 0.4,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.rage = rng.gen_range(0.2..0.8);
        self.isolation = rng.gen_range(0.2..0.8);
        self.territoriality = rng.gen_range(0.2..0.8);
        self.hunger = rng.gen_range(0.2..0.8);
        self.cunning = rng.gen_range(0.2..0.8);
    }
}

/// Minotaur archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct MinotaurArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<MinotaurValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl MinotaurArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: MinotaurValues) -> EntityId {
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
    fn test_minotaur_values_creation() {
        let values = MinotaurValues::new();
        assert!((values.rage - 0.7).abs() < 0.01);
        assert!((values.isolation - 0.6).abs() < 0.01);
        assert!((values.territoriality - 0.8).abs() < 0.01);
        assert!((values.hunger - 0.5).abs() < 0.01);
        assert!((values.cunning - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_minotaur_archetype_spawn() {
        let mut archetype = MinotaurArchetype::new();
        let id = archetype.spawn(
            "Test Minotaur".to_string(),
            Vec2::new(10.0, 20.0),
            MinotaurValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}