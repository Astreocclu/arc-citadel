//! Elf-specific archetype with SoA layout
//!
//! Elves value beauty, wisdom, and harmony with nature above all else.
//! They are graceful, long-lived, and deeply connected to the natural world.

use crate::core::types::{EntityId, Tick, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::social::SocialMemory;
use crate::entity::tasks::TaskQueue;
use crate::entity::thoughts::ThoughtBuffer;

/// Elf-specific value vocabulary
///
/// Elves prioritize beauty, wisdom, and nature over
/// human concepts like ambition and comfort.
#[derive(Debug, Clone)]
pub struct ElfValues {
    /// Appreciation for aesthetic perfection
    pub beauty: f32,
    /// Knowledge and understanding
    pub wisdom: f32,
    /// Connection to the natural world
    pub nature: f32,
    /// Graceful, elegant movement
    pub grace: f32,
    /// Separation from other races
    pub aloofness: f32,
    /// Skill with the bow
    pub archery: f32,
    /// Perception of magical energies
    pub arcane_sense: f32,
}

impl Default for ElfValues {
    fn default() -> Self {
        // Elves are graceful, wise, and nature-loving
        Self {
            beauty: 0.8,       // Strong aesthetic sense
            wisdom: 0.7,       // Wise from long lives
            nature: 0.7,       // Connected to natural world
            grace: 0.8,        // Graceful movement
            aloofness: 0.5,    // Moderate separation from others
            archery: 0.6,      // Skilled archers
            arcane_sense: 0.4, // Some magical awareness
        }
    }
}

impl ElfValues {
    pub fn dominant(&self) -> (&'static str, f32) {
        let values = [
            ("beauty", self.beauty),
            ("wisdom", self.wisdom),
            ("nature", self.nature),
            ("grace", self.grace),
            ("aloofness", self.aloofness),
            ("archery", self.archery),
            ("arcane_sense", self.arcane_sense),
        ];
        values
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }
}

/// Structure of Arrays for elf entities
pub struct ElfArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub birth_ticks: Vec<Tick>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<ElfValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl ElfArchetype {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            names: Vec::new(),
            birth_ticks: Vec::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
            body_states: Vec::new(),
            needs: Vec::new(),
            thoughts: Vec::new(),
            values: Vec::new(),
            task_queues: Vec::new(),
            alive: Vec::new(),
            social_memories: Vec::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.ids.len()
    }

    pub fn spawn(&mut self, id: EntityId, name: String, tick: Tick) {
        self.ids.push(id);
        self.names.push(name);
        self.birth_ticks.push(tick);
        self.positions.push(Vec2::default());
        self.velocities.push(Vec2::default());
        self.body_states.push(BodyState::default());
        self.needs.push(Needs::default());
        self.thoughts.push(ThoughtBuffer::new());
        self.values.push(ElfValues::default());
        self.task_queues.push(TaskQueue::new());
        self.alive.push(true);
        self.social_memories.push(SocialMemory::new());
    }

    pub fn index_of(&self, id: EntityId) -> Option<usize> {
        self.ids.iter().position(|&e| e == id)
    }

    pub fn iter_living(&self) -> impl Iterator<Item = usize> + '_ {
        self.alive
            .iter()
            .enumerate()
            .filter(|(_, &alive)| alive)
            .map(|(i, _)| i)
    }
}

impl Default for ElfArchetype {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf_default_values() {
        let values = ElfValues::default();
        // Elves should have high beauty and grace
        assert!(values.beauty > 0.7);
        assert!(values.grace > 0.7);
    }

    #[test]
    fn test_elf_spawn() {
        let mut archetype = ElfArchetype::new();
        let id = EntityId::new();
        archetype.spawn(id, "Legolas".to_string(), 0);

        assert_eq!(archetype.count(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
        assert!(archetype.alive[0]);
    }

    #[test]
    fn test_elf_dominant_value() {
        let values = ElfValues::default();
        let (name, _) = values.dominant();
        // Grace or beauty should be dominant at 0.8
        assert!(name == "beauty" || name == "grace");
    }
}
