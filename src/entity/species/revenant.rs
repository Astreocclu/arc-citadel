//! Revenant entity archetype and values

use crate::core::types::{EntityId, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::social::SocialMemory;
use crate::entity::tasks::TaskQueue;
use crate::entity::thoughts::ThoughtBuffer;
use serde::{Deserialize, Serialize};

/// Revenant-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RevenantValues {
    pub hunger_for_life: f32,
    pub obedience: f32,
    pub lingering_rage: f32,
    pub territorial_rot: f32,
}

impl RevenantValues {
    pub fn new() -> Self {
        Self {
            hunger_for_life: 0.3,
            obedience: 0.7,
            lingering_rage: 0.5,
            territorial_rot: 0.4,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.hunger_for_life = rng.gen_range(0.2..0.8);
        self.obedience = rng.gen_range(0.2..0.8);
        self.lingering_rage = rng.gen_range(0.2..0.8);
        self.territorial_rot = rng.gen_range(0.2..0.8);
    }
}

/// Revenant archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct RevenantArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<RevenantValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl RevenantArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: RevenantValues) -> EntityId {
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
    fn test_revenant_values_creation() {
        let values = RevenantValues::new();
        assert!((values.hunger_for_life - 0.3).abs() < 0.01);
        assert!((values.obedience - 0.7).abs() < 0.01);
        assert!((values.lingering_rage - 0.5).abs() < 0.01);
        assert!((values.territorial_rot - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_revenant_archetype_spawn() {
        let mut archetype = RevenantArchetype::new();
        let id = archetype.spawn(
            "Test Revenant".to_string(),
            Vec2::new(10.0, 20.0),
            RevenantValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}
