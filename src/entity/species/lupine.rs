//! Lupine entity archetype and values

use crate::core::types::{EntityId, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::social::SocialMemory;
use crate::entity::tasks::TaskQueue;
use crate::entity::thoughts::ThoughtBuffer;
use serde::{Deserialize, Serialize};

/// Lupine-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LupineValues {
    pub bestial_rage: f32,
    pub human_restraint: f32,
    pub pack_loyalty: f32,
    pub territorial_hunger: f32,
}

impl LupineValues {
    pub fn new() -> Self {
        Self {
            bestial_rage: 0.3,
            human_restraint: 0.7,
            pack_loyalty: 0.8,
            territorial_hunger: 0.5,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.bestial_rage = rng.gen_range(0.2..0.8);
        self.human_restraint = rng.gen_range(0.2..0.8);
        self.pack_loyalty = rng.gen_range(0.2..0.8);
        self.territorial_hunger = rng.gen_range(0.2..0.8);
    }
}

/// Lupine archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct LupineArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<LupineValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl LupineArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: LupineValues) -> EntityId {
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
    fn test_lupine_values_creation() {
        let values = LupineValues::new();
        assert!((values.bestial_rage - 0.3).abs() < 0.01);
        assert!((values.human_restraint - 0.7).abs() < 0.01);
        assert!((values.pack_loyalty - 0.8).abs() < 0.01);
        assert!((values.territorial_hunger - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_lupine_archetype_spawn() {
        let mut archetype = LupineArchetype::new();
        let id = archetype.spawn(
            "Test Lupine".to_string(),
            Vec2::new(10.0, 20.0),
            LupineValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}
