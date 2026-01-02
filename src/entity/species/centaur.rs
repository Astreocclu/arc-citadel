//! Centaur entity archetype and values

use serde::{Deserialize, Serialize};
use crate::core::types::{EntityId, Vec2};
use crate::entity::needs::Needs;
use crate::entity::thoughts::ThoughtBuffer;
use crate::entity::tasks::TaskQueue;
use crate::entity::body::BodyState;
use crate::entity::social::SocialMemory;

/// Centaur-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CentaurValues {
    pub honor: f32,
    pub wanderlust: f32,
    pub pride: f32,
    pub loyalty: f32,
    pub wrath: f32,
}

impl CentaurValues {
    pub fn new() -> Self {
        Self {
            honor: 0.75,
            wanderlust: 0.7,
            pride: 0.6,
            loyalty: 0.65,
            wrath: 0.4,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.honor = rng.gen_range(0.2..0.8);
        self.wanderlust = rng.gen_range(0.2..0.8);
        self.pride = rng.gen_range(0.2..0.8);
        self.loyalty = rng.gen_range(0.2..0.8);
        self.wrath = rng.gen_range(0.2..0.8);
    }
}

/// Centaur archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct CentaurArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<CentaurValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl CentaurArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: CentaurValues) -> EntityId {
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
    fn test_centaur_values_creation() {
        let values = CentaurValues::new();
        assert!((values.honor - 0.75).abs() < 0.01);
        assert!((values.wanderlust - 0.7).abs() < 0.01);
        assert!((values.pride - 0.6).abs() < 0.01);
        assert!((values.loyalty - 0.65).abs() < 0.01);
        assert!((values.wrath - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_centaur_archetype_spawn() {
        let mut archetype = CentaurArchetype::new();
        let id = archetype.spawn(
            "Test Centaur".to_string(),
            Vec2::new(10.0, 20.0),
            CentaurValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}