//! Dwarf-specific archetype with SoA layout
//!
//! Dwarves value tradition, craftsmanship, and clan honor above all else.
//! They are sturdy, methodical, and fiercely protective of their kin.

use crate::core::types::{EntityId, Tick, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::social::SocialMemory;
use crate::entity::tasks::TaskQueue;
use crate::entity::thoughts::ThoughtBuffer;

/// Dwarf-specific value vocabulary
///
/// Dwarves prioritize tradition, craftsmanship, and clan honor over
/// human concepts like beauty and curiosity.
#[derive(Debug, Clone)]
pub struct DwarfValues {
    /// Respect for ancestral ways
    pub tradition: f32,
    /// Pride in skilled work
    pub craftsmanship: f32,
    /// Family and clan loyalty
    pub clan_honor: f32,
    /// Desire for precious materials
    pub greed: f32,
    /// Resistance to change
    pub stubbornness: f32,
    /// Dedication to defensive works
    pub fortification: f32,
    /// Long memory for wrongs
    pub grudge: f32,
}

impl Default for DwarfValues {
    fn default() -> Self {
        // Dwarves are traditional, skilled, and clan-focused
        Self {
            tradition: 0.7,      // Strong respect for ancestral ways
            craftsmanship: 0.8,  // Excellent craftsmen
            clan_honor: 0.7,     // Fiercely loyal to clan
            greed: 0.4,          // Moderate desire for wealth
            stubbornness: 0.6,   // Resistant to change
            fortification: 0.5,  // Build defenses
            grudge: 0.3,         // Remember slights (starts low)
        }
    }
}

impl DwarfValues {
    pub fn dominant(&self) -> (&'static str, f32) {
        let values = [
            ("tradition", self.tradition),
            ("craftsmanship", self.craftsmanship),
            ("clan_honor", self.clan_honor),
            ("greed", self.greed),
            ("stubbornness", self.stubbornness),
            ("fortification", self.fortification),
            ("grudge", self.grudge),
        ];
        values
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }
}

/// Structure of Arrays for dwarf entities
pub struct DwarfArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub birth_ticks: Vec<Tick>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<DwarfValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl DwarfArchetype {
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
        self.values.push(DwarfValues::default());
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

impl Default for DwarfArchetype {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dwarf_default_values() {
        let values = DwarfValues::default();
        // Dwarves should be excellent craftsmen
        assert!(values.craftsmanship > 0.7);
        // Dwarves should be traditional
        assert!(values.tradition > 0.5);
    }

    #[test]
    fn test_dwarf_spawn() {
        let mut archetype = DwarfArchetype::new();
        let id = EntityId::new();
        archetype.spawn(id, "Thorin".to_string(), 0);

        assert_eq!(archetype.count(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
        assert!(archetype.alive[0]);
    }

    #[test]
    fn test_dwarf_dominant_value() {
        let values = DwarfValues::default();
        let (name, _) = values.dominant();
        // Craftsmanship should be dominant at 0.8
        assert_eq!(name, "craftsmanship");
    }
}
