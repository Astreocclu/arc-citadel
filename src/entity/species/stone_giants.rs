//! StoneGiants entity archetype and values

use crate::core::types::{EntityId, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::social::SocialMemory;
use crate::entity::tasks::TaskQueue;
use crate::entity::thoughts::ThoughtBuffer;
use serde::{Deserialize, Serialize};

/// StoneGiants-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoneGiantsValues {
    pub pride: f32,
    pub greed: f32,
    pub rage: f32,
    pub territoriality: f32,
    pub loneliness: f32,
}

impl StoneGiantsValues {
    pub fn new() -> Self {
        Self {
            pride: 0.5,
            greed: 0.3,
            rage: 0.2,
            territoriality: 0.7,
            loneliness: 0.4,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.pride = rng.gen_range(0.2..0.8);
        self.greed = rng.gen_range(0.2..0.8);
        self.rage = rng.gen_range(0.2..0.8);
        self.territoriality = rng.gen_range(0.2..0.8);
        self.loneliness = rng.gen_range(0.2..0.8);
    }
}

/// StoneGiants archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct StoneGiantsArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<StoneGiantsValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl StoneGiantsArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: StoneGiantsValues) -> EntityId {
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
    fn test_stone_giants_values_creation() {
        let values = StoneGiantsValues::new();
        assert!((values.pride - 0.5).abs() < 0.01);
        assert!((values.greed - 0.3).abs() < 0.01);
        assert!((values.rage - 0.2).abs() < 0.01);
        assert!((values.territoriality - 0.7).abs() < 0.01);
        assert!((values.loneliness - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_stone_giants_archetype_spawn() {
        let mut archetype = StoneGiantsArchetype::new();
        let id = archetype.spawn(
            "Test StoneGiants".to_string(),
            Vec2::new(10.0, 20.0),
            StoneGiantsValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}
