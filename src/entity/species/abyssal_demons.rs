//! AbyssalDemons entity archetype and values

use crate::core::types::{EntityId, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::social::SocialMemory;
use crate::entity::tasks::TaskQueue;
use crate::entity::thoughts::ThoughtBuffer;
use serde::{Deserialize, Serialize};

/// AbyssalDemons-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AbyssalDemonsValues {
    pub soul_hunger: f32,
    pub corruptive_urge: f32,
    pub malicious_cunning: f32,
    pub abyssal_rage: f32,
}

impl AbyssalDemonsValues {
    pub fn new() -> Self {
        Self {
            soul_hunger: 0.3,
            corruptive_urge: 0.5,
            malicious_cunning: 0.4,
            abyssal_rage: 0.2,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.soul_hunger = rng.gen_range(0.2..0.8);
        self.corruptive_urge = rng.gen_range(0.2..0.8);
        self.malicious_cunning = rng.gen_range(0.2..0.8);
        self.abyssal_rage = rng.gen_range(0.2..0.8);
    }
}

/// AbyssalDemons archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct AbyssalDemonsArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<AbyssalDemonsValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl AbyssalDemonsArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: AbyssalDemonsValues) -> EntityId {
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
    fn test_abyssal_demons_values_creation() {
        let values = AbyssalDemonsValues::new();
        assert!((values.soul_hunger - 0.3).abs() < 0.01);
        assert!((values.corruptive_urge - 0.5).abs() < 0.01);
        assert!((values.malicious_cunning - 0.4).abs() < 0.01);
        assert!((values.abyssal_rage - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_abyssal_demons_archetype_spawn() {
        let mut archetype = AbyssalDemonsArchetype::new();
        let id = archetype.spawn(
            "Test AbyssalDemons".to_string(),
            Vec2::new(10.0, 20.0),
            AbyssalDemonsValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}
