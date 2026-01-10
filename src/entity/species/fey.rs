//! Fey entity archetype and values

use crate::core::types::{EntityId, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::social::SocialMemory;
use crate::entity::tasks::TaskQueue;
use crate::entity::thoughts::ThoughtBuffer;
use serde::{Deserialize, Serialize};

/// Fey-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeyValues {
    pub whimsy: f32,
    pub cruelty: f32,
    pub bargain_hunger: f32,
    pub territoriality: f32,
    pub fear_of_iron: f32,
}

impl FeyValues {
    pub fn new() -> Self {
        Self {
            whimsy: 0.5,
            cruelty: 0.3,
            bargain_hunger: 0.6,
            territoriality: 0.4,
            fear_of_iron: 0.7,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.whimsy = rng.gen_range(0.2..0.8);
        self.cruelty = rng.gen_range(0.2..0.8);
        self.bargain_hunger = rng.gen_range(0.2..0.8);
        self.territoriality = rng.gen_range(0.2..0.8);
        self.fear_of_iron = rng.gen_range(0.2..0.8);
    }
}

/// Fey archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct FeyArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<FeyValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl FeyArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: FeyValues) -> EntityId {
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
    fn test_fey_values_creation() {
        let values = FeyValues::new();
        assert!((values.whimsy - 0.5).abs() < 0.01);
        assert!((values.cruelty - 0.3).abs() < 0.01);
        assert!((values.bargain_hunger - 0.6).abs() < 0.01);
        assert!((values.territoriality - 0.4).abs() < 0.01);
        assert!((values.fear_of_iron - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_fey_archetype_spawn() {
        let mut archetype = FeyArchetype::new();
        let id = archetype.spawn(
            "Test Fey".to_string(),
            Vec2::new(10.0, 20.0),
            FeyValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}
