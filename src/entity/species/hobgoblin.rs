//! Hobgoblin entity archetype and values

use crate::core::types::{EntityId, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::social::SocialMemory;
use crate::entity::tasks::TaskQueue;
use crate::entity::thoughts::ThoughtBuffer;
use serde::{Deserialize, Serialize};

/// Hobgoblin-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HobgoblinValues {
    pub discipline: f32,
    pub ambition: f32,
    pub honor: f32,
    pub pragmatism: f32,
    pub cruelty: f32,
}

impl HobgoblinValues {
    pub fn new() -> Self {
        Self {
            discipline: 0.85,
            ambition: 0.6,
            honor: 0.5,
            pragmatism: 0.7,
            cruelty: 0.4,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.discipline = rng.gen_range(0.2..0.8);
        self.ambition = rng.gen_range(0.2..0.8);
        self.honor = rng.gen_range(0.2..0.8);
        self.pragmatism = rng.gen_range(0.2..0.8);
        self.cruelty = rng.gen_range(0.2..0.8);
    }
}

/// Hobgoblin archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct HobgoblinArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<HobgoblinValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl HobgoblinArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: HobgoblinValues) -> EntityId {
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
    fn test_hobgoblin_values_creation() {
        let values = HobgoblinValues::new();
        assert!((values.discipline - 0.85).abs() < 0.01);
        assert!((values.ambition - 0.6).abs() < 0.01);
        assert!((values.honor - 0.5).abs() < 0.01);
        assert!((values.pragmatism - 0.7).abs() < 0.01);
        assert!((values.cruelty - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_hobgoblin_archetype_spawn() {
        let mut archetype = HobgoblinArchetype::new();
        let id = archetype.spawn(
            "Test Hobgoblin".to_string(),
            Vec2::new(10.0, 20.0),
            HobgoblinValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}
