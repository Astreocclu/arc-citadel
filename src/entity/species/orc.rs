//! Orc-specific archetype with SoA layout
//!
//! # Design Notes (TODO)
//!
//! - **Naming**: `OrcArchetype` uses ECS terminology (SoA container), not game design
//!   terminology (character class). Consider renaming to `OrcStorage` or `Orcs` for clarity.
//!
//! - **Initial values**: Currently spawns with `OrcValues::default()` (all 0.0). A proper
//!   spawner should roll initial values within species-appropriate ranges.
//!
//! - **Character classes**: Warrior/Shaman/etc. archetypes are a separate concept not yet
//!   implemented. Would be spawn templates that set value ranges.
//!
//! - **Lifecycle**: `alive: Vec<bool>` duplicates pattern from HumanArchetype. Consider
//!   whether lifecycle state should be centralized in World or a DeathSystem.

use crate::core::types::{EntityId, Vec2, Tick};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::thoughts::ThoughtBuffer;
use crate::entity::tasks::TaskQueue;
use crate::entity::social::SocialMemory;

/// Orc-specific value vocabulary
///
/// Orcs prioritize strength, dominance, and clan loyalty over
/// human concepts like honor, beauty, and piety.
#[derive(Debug, Clone, Default)]
pub struct OrcValues {
    pub rage: f32,
    pub strength: f32,
    pub dominance: f32,
    pub clan_loyalty: f32,
    pub blood_debt: f32,
    pub territory: f32,
    pub combat_prowess: f32,
}

impl OrcValues {
    pub fn dominant(&self) -> (&'static str, f32) {
        let values = [
            ("rage", self.rage),
            ("strength", self.strength),
            ("dominance", self.dominance),
            ("clan_loyalty", self.clan_loyalty),
            ("blood_debt", self.blood_debt),
            ("territory", self.territory),
            ("combat_prowess", self.combat_prowess),
        ];
        values.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }
}

/// Structure of Arrays for orc entities
pub struct OrcArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub birth_ticks: Vec<Tick>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<OrcValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl OrcArchetype {
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
        self.values.push(OrcValues::default());
        self.task_queues.push(TaskQueue::new());
        self.alive.push(true);
        self.social_memories.push(SocialMemory::new());
    }

    pub fn index_of(&self, id: EntityId) -> Option<usize> {
        self.ids.iter().position(|&e| e == id)
    }

    pub fn iter_living(&self) -> impl Iterator<Item = usize> + '_ {
        self.alive.iter()
            .enumerate()
            .filter(|(_, &alive)| alive)
            .map(|(i, _)| i)
    }
}

impl Default for OrcArchetype {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orc_default_values() {
        let values = OrcValues::default();
        assert_eq!(values.rage, 0.0);
        assert_eq!(values.strength, 0.0);
        assert_eq!(values.dominance, 0.0);
        assert_eq!(values.clan_loyalty, 0.0);
        assert_eq!(values.blood_debt, 0.0);
        assert_eq!(values.territory, 0.0);
        assert_eq!(values.combat_prowess, 0.0);
    }

    #[test]
    fn test_orc_dominant_value() {
        let mut values = OrcValues::default();
        values.rage = 0.9;
        values.dominance = 0.3;

        let (name, level) = values.dominant();
        assert_eq!(name, "rage");
        assert_eq!(level, 0.9);
    }

    #[test]
    fn test_orc_dominant_strength() {
        let mut values = OrcValues::default();
        values.strength = 0.8;
        values.rage = 0.2;

        let (name, level) = values.dominant();
        assert_eq!(name, "strength");
        assert_eq!(level, 0.8);
    }

    #[test]
    fn test_orc_has_social_memory() {
        let mut archetype = OrcArchetype::new();
        let id = EntityId::new();
        archetype.spawn(id, "Grukk".into(), 0);

        assert_eq!(archetype.social_memories.len(), 1);
        assert_eq!(archetype.social_memories[0].slots.len(), 0); // Empty initially
    }

    #[test]
    fn test_orc_archetype_spawn() {
        let mut archetype = OrcArchetype::new();
        let id = EntityId::new();
        archetype.spawn(id, "Thraka".into(), 100);

        assert_eq!(archetype.count(), 1);
        assert_eq!(archetype.names[0], "Thraka");
        assert_eq!(archetype.birth_ticks[0], 100);
        assert!(archetype.alive[0]);
    }

    #[test]
    fn test_orc_iter_living() {
        let mut archetype = OrcArchetype::new();
        archetype.spawn(EntityId::new(), "Orc1".into(), 0);
        archetype.spawn(EntityId::new(), "Orc2".into(), 0);
        archetype.spawn(EntityId::new(), "Orc3".into(), 0);

        // Kill the second orc
        archetype.alive[1] = false;

        let living: Vec<_> = archetype.iter_living().collect();
        assert_eq!(living, vec![0, 2]);
    }
}
