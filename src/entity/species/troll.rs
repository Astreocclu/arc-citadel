//! Troll entity archetype and values

use crate::core::types::{EntityId, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::social::SocialMemory;
use crate::entity::tasks::TaskQueue;
use crate::entity::thoughts::ThoughtBuffer;
use serde::{Deserialize, Serialize};

/// Troll-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrollValues {
    pub territoriality: f32,
    pub hunger: f32,
    pub rage: f32,
    pub patience: f32,
    pub recklessness: f32,
}

impl TrollValues {
    pub fn new() -> Self {
        Self {
            territoriality: 0.85,
            hunger: 0.3,
            rage: 0.2,
            patience: 0.65,
            recklessness: 0.75,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.territoriality = rng.gen_range(0.2..0.8);
        self.hunger = rng.gen_range(0.2..0.8);
        self.rage = rng.gen_range(0.2..0.8);
        self.patience = rng.gen_range(0.2..0.8);
        self.recklessness = rng.gen_range(0.2..0.8);
    }
}

/// Troll archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct TrollArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<TrollValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl TrollArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: TrollValues) -> EntityId {
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
    fn test_troll_values_creation() {
        let values = TrollValues::new();
        assert!((values.territoriality - 0.85).abs() < 0.01);
        assert!((values.hunger - 0.3).abs() < 0.01);
        assert!((values.rage - 0.2).abs() < 0.01);
        assert!((values.patience - 0.65).abs() < 0.01);
        assert!((values.recklessness - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_troll_archetype_spawn() {
        let mut archetype = TrollArchetype::new();
        let id = archetype.spawn(
            "Test Troll".to_string(),
            Vec2::new(10.0, 20.0),
            TrollValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}
