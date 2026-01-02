//! Gnoll entity archetype and values

use serde::{Deserialize, Serialize};
use crate::core::types::{EntityId, Vec2};
use crate::entity::needs::Needs;
use crate::entity::thoughts::ThoughtBuffer;
use crate::entity::tasks::TaskQueue;
use crate::entity::body::BodyState;
use crate::entity::social::SocialMemory;

/// Gnoll-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GnollValues {
    pub bloodlust: f32,
    pub pack_instinct: f32,
    pub hunger: f32,
    pub cruelty: f32,
    pub dominance: f32,
}

impl GnollValues {
    pub fn new() -> Self {
        Self {
            bloodlust: 0.7,
            pack_instinct: 0.8,
            hunger: 0.6,
            cruelty: 0.5,
            dominance: 0.4,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.bloodlust = rng.gen_range(0.2..0.8);
        self.pack_instinct = rng.gen_range(0.2..0.8);
        self.hunger = rng.gen_range(0.2..0.8);
        self.cruelty = rng.gen_range(0.2..0.8);
        self.dominance = rng.gen_range(0.2..0.8);
    }
}

/// Gnoll archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct GnollArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<GnollValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl GnollArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: GnollValues) -> EntityId {
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
    fn test_gnoll_values_creation() {
        let values = GnollValues::new();
        assert!((values.bloodlust - 0.7).abs() < 0.01);
        assert!((values.pack_instinct - 0.8).abs() < 0.01);
        assert!((values.hunger - 0.6).abs() < 0.01);
        assert!((values.cruelty - 0.5).abs() < 0.01);
        assert!((values.dominance - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_gnoll_archetype_spawn() {
        let mut archetype = GnollArchetype::new();
        let id = archetype.spawn(
            "Test Gnoll".to_string(),
            Vec2::new(10.0, 20.0),
            GnollValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}