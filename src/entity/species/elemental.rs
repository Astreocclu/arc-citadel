//! Elemental entity archetype and values

use crate::core::types::{EntityId, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::social::SocialMemory;
use crate::entity::tasks::TaskQueue;
use crate::entity::thoughts::ThoughtBuffer;
use serde::{Deserialize, Serialize};

/// Elemental-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElementalValues {
    pub primal_urge: f32,
    pub elemental_rage: f32,
    pub territorial_instinct: f32,
    pub flickering_will: f32,
}

impl ElementalValues {
    pub fn new() -> Self {
        Self {
            primal_urge: 0.3,
            elemental_rage: 0.1,
            territorial_instinct: 0.5,
            flickering_will: 0.7,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.primal_urge = rng.gen_range(0.2..0.8);
        self.elemental_rage = rng.gen_range(0.2..0.8);
        self.territorial_instinct = rng.gen_range(0.2..0.8);
        self.flickering_will = rng.gen_range(0.2..0.8);
    }
}

/// Elemental archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct ElementalArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<ElementalValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl ElementalArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: ElementalValues) -> EntityId {
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
    fn test_elemental_values_creation() {
        let values = ElementalValues::new();
        assert!((values.primal_urge - 0.3).abs() < 0.01);
        assert!((values.elemental_rage - 0.1).abs() < 0.01);
        assert!((values.territorial_instinct - 0.5).abs() < 0.01);
        assert!((values.flickering_will - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_elemental_archetype_spawn() {
        let mut archetype = ElementalArchetype::new();
        let id = archetype.spawn(
            "Test Elemental".to_string(),
            Vec2::new(10.0, 20.0),
            ElementalValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}
