//! Vampire entity archetype and values

use crate::core::types::{EntityId, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::social::SocialMemory;
use crate::entity::tasks::TaskQueue;
use crate::entity::thoughts::ThoughtBuffer;
use serde::{Deserialize, Serialize};

/// Vampire-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VampireValues {
    pub bloodthirst: f32,
    pub arrogance: f32,
    pub secrecy: f32,
    pub dominance: f32,
    pub ennui: f32,
}

impl VampireValues {
    pub fn new() -> Self {
        Self {
            bloodthirst: 0.3,
            arrogance: 0.5,
            secrecy: 0.7,
            dominance: 0.4,
            ennui: 0.2,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.bloodthirst = rng.gen_range(0.2..0.8);
        self.arrogance = rng.gen_range(0.2..0.8);
        self.secrecy = rng.gen_range(0.2..0.8);
        self.dominance = rng.gen_range(0.2..0.8);
        self.ennui = rng.gen_range(0.2..0.8);
    }
}

impl crate::entity::species::value_access::ValueAccessor for VampireValues {
    fn get_value(&self, field_name: &str) -> Option<f32> {
        match field_name {
            "bloodthirst" => Some(self.bloodthirst),
            "arrogance" => Some(self.arrogance),
            "secrecy" => Some(self.secrecy),
            "dominance" => Some(self.dominance),
            "ennui" => Some(self.ennui),
            _ => None,
        }
    }

    fn set_value(&mut self, field_name: &str, value: f32) -> bool {
        match field_name {
            "bloodthirst" => {
                self.bloodthirst = value;
                true
            }
            "arrogance" => {
                self.arrogance = value;
                true
            }
            "secrecy" => {
                self.secrecy = value;
                true
            }
            "dominance" => {
                self.dominance = value;
                true
            }
            "ennui" => {
                self.ennui = value;
                true
            }
            _ => false,
        }
    }

    fn field_names() -> &'static [&'static str] {
        &["bloodthirst", "arrogance", "secrecy", "dominance", "ennui"]
    }
}

/// Vampire archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct VampireArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<VampireValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl VampireArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: VampireValues) -> EntityId {
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
    fn test_vampire_values_creation() {
        let values = VampireValues::new();
        assert!((values.bloodthirst - 0.3).abs() < 0.01);
        assert!((values.arrogance - 0.5).abs() < 0.01);
        assert!((values.secrecy - 0.7).abs() < 0.01);
        assert!((values.dominance - 0.4).abs() < 0.01);
        assert!((values.ennui - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_vampire_archetype_spawn() {
        let mut archetype = VampireArchetype::new();
        let id = archetype.spawn(
            "Test Vampire".to_string(),
            Vec2::new(10.0, 20.0),
            VampireValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}
