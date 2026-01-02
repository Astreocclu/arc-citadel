//! Goblin entity archetype and values

use serde::{Deserialize, Serialize};
use crate::core::types::{EntityId, Vec2};
use crate::entity::needs::Needs;
use crate::entity::thoughts::ThoughtBuffer;
use crate::entity::tasks::TaskQueue;
use crate::entity::body::BodyState;
use crate::entity::social::SocialMemory;

/// Goblin-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoblinValues {
    pub greed: f32,
    pub cowardice: f32,
    pub pack_rage: f32,
    pub sneakiness: f32,
    pub hunger: f32,
}

impl GoblinValues {
    pub fn new() -> Self {
        Self {
            greed: 0.6,
            cowardice: 0.7,
            pack_rage: 0.3,
            sneakiness: 0.5,
            hunger: 0.4,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.greed = rng.gen_range(0.2..0.8);
        self.cowardice = rng.gen_range(0.2..0.8);
        self.pack_rage = rng.gen_range(0.2..0.8);
        self.sneakiness = rng.gen_range(0.2..0.8);
        self.hunger = rng.gen_range(0.2..0.8);
    }
}

/// Goblin archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct GoblinArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<GoblinValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl GoblinArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: GoblinValues) -> EntityId {
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
    fn test_goblin_values_creation() {
        let values = GoblinValues::new();
        assert!((values.greed - 0.6).abs() < 0.01);
        assert!((values.cowardice - 0.7).abs() < 0.01);
        assert!((values.pack_rage - 0.3).abs() < 0.01);
        assert!((values.sneakiness - 0.5).abs() < 0.01);
        assert!((values.hunger - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_goblin_archetype_spawn() {
        let mut archetype = GoblinArchetype::new();
        let id = archetype.spawn(
            "Test Goblin".to_string(),
            Vec2::new(10.0, 20.0),
            GoblinValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}