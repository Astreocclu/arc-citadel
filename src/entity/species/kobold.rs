//! Kobold entity archetype and values

use serde::{Deserialize, Serialize};
use crate::core::types::{EntityId, Vec2};
use crate::entity::needs::Needs;
use crate::entity::thoughts::ThoughtBuffer;
use crate::entity::tasks::TaskQueue;
use crate::entity::body::BodyState;
use crate::entity::social::SocialMemory;

/// Kobold-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KoboldValues {
    pub cunning: f32,
    pub cowardice: f32,
    pub industriousness: f32,
    pub pack_loyalty: f32,
    pub spite: f32,
}

impl KoboldValues {
    pub fn new() -> Self {
        Self {
            cunning: 0.7,
            cowardice: 0.8,
            industriousness: 0.6,
            pack_loyalty: 0.5,
            spite: 0.4,
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
        self.cunning = rng.gen_range(0.2..0.8);
        self.cowardice = rng.gen_range(0.2..0.8);
        self.industriousness = rng.gen_range(0.2..0.8);
        self.pack_loyalty = rng.gen_range(0.2..0.8);
        self.spite = rng.gen_range(0.2..0.8);
    }
}

impl crate::entity::species::value_access::ValueAccessor for KoboldValues {
    fn get_value(&self, field_name: &str) -> Option<f32> {
        match field_name {
            "cunning" => Some(self.cunning),
            "cowardice" => Some(self.cowardice),
            "industriousness" => Some(self.industriousness),
            "pack_loyalty" => Some(self.pack_loyalty),
            "spite" => Some(self.spite),
            _ => None,
        }
    }

    fn set_value(&mut self, field_name: &str, value: f32) -> bool {
        match field_name {
            "cunning" => { self.cunning = value; true }
            "cowardice" => { self.cowardice = value; true }
            "industriousness" => { self.industriousness = value; true }
            "pack_loyalty" => { self.pack_loyalty = value; true }
            "spite" => { self.spite = value; true }
            _ => false,
        }
    }

    fn field_names() -> &'static [&'static str] {
        &["cunning", "cowardice", "industriousness", "pack_loyalty", "spite"]
    }
}

/// Kobold archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct KoboldArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<KoboldValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl KoboldArchetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: KoboldValues) -> EntityId {
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
    fn test_kobold_values_creation() {
        let values = KoboldValues::new();
        assert!((values.cunning - 0.7).abs() < 0.01);
        assert!((values.cowardice - 0.8).abs() < 0.01);
        assert!((values.industriousness - 0.6).abs() < 0.01);
        assert!((values.pack_loyalty - 0.5).abs() < 0.01);
        assert!((values.spite - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_kobold_archetype_spawn() {
        let mut archetype = KoboldArchetype::new();
        let id = archetype.spawn(
            "Test Kobold".to_string(),
            Vec2::new(10.0, 20.0),
            KoboldValues::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}