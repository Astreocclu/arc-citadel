//! Dryad entity archetype and values

use crate::core::types::{EntityId, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::social::SocialMemory;
use crate::entity::tasks::TaskQueue;
use crate::entity::thoughts::ThoughtBuffer;
use serde::{Deserialize, Serialize};

/// Dryad-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DryadValues {
    pub nature_bond: f32,
    pub protectiveness: f32,
    pub patience: f32,
    pub wrath: f32,
    pub allure: f32,
}

impl DryadValues {
    pub fn new() -> Self {
        Self {
            nature_bond: 0.9,
            protectiveness: 0.8,
            patience: 0.7,
            wrath: 0.3,
            allure: 0.5,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.nature_bond = rng.gen_range(0.2..0.8);
        self.protectiveness = rng.gen_range(0.2..0.8);
        self.patience = rng.gen_range(0.2..0.8);
        self.wrath = rng.gen_range(0.2..0.8);
        self.allure = rng.gen_range(0.2..0.8);
    }
}

/// Dryad archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct DryadArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<DryadValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl DryadArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: DryadValues) -> EntityId {
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
    fn test_dryad_values_creation() {
        let values = DryadValues::new();
        assert!((values.nature_bond - 0.9).abs() < 0.01);
        assert!((values.protectiveness - 0.8).abs() < 0.01);
        assert!((values.patience - 0.7).abs() < 0.01);
        assert!((values.wrath - 0.3).abs() < 0.01);
        assert!((values.allure - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_dryad_archetype_spawn() {
        let mut archetype = DryadArchetype::new();
        let id = archetype.spawn(
            "Test Dryad".to_string(),
            Vec2::new(10.0, 20.0),
            DryadValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}
