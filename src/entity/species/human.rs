//! Human-specific archetype with SoA layout

use crate::city::BuildingId;
use crate::combat::CombatState;
use crate::core::types::{EntityId, Tick, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::social::{EventBuffer, SocialMemory};
use crate::entity::tasks::TaskQueue;
use crate::entity::thoughts::ThoughtBuffer;

/// Human-specific value vocabulary
#[derive(Debug, Clone, Default)]
pub struct HumanValues {
    pub honor: f32,
    pub beauty: f32,
    pub comfort: f32,
    pub ambition: f32,
    pub loyalty: f32,
    pub love: f32,
    pub justice: f32,
    pub curiosity: f32,
    pub safety: f32,
    pub piety: f32,
}

impl HumanValues {
    pub fn dominant(&self) -> (&'static str, f32) {
        let values = [
            ("honor", self.honor),
            ("beauty", self.beauty),
            ("comfort", self.comfort),
            ("ambition", self.ambition),
            ("loyalty", self.loyalty),
            ("love", self.love),
            ("justice", self.justice),
            ("curiosity", self.curiosity),
            ("safety", self.safety),
            ("piety", self.piety),
        ];
        values
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }
}

/// Structure of Arrays for human entities
pub struct HumanArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub birth_ticks: Vec<Tick>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<HumanValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
    pub event_buffers: Vec<EventBuffer>,
    /// Building skill level (0.0 to 1.0)
    pub building_skills: Vec<f32>,
    /// Combat state for each entity
    pub combat_states: Vec<CombatState>,
    /// Assigned housing (None = homeless)
    pub assigned_houses: Vec<Option<BuildingId>>,
    /// Skill chunk libraries for each entity
    pub chunk_libraries: Vec<crate::skills::ChunkLibrary>,
}

impl HumanArchetype {
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
            event_buffers: Vec::new(),
            building_skills: Vec::new(),
            combat_states: Vec::new(),
            assigned_houses: Vec::new(),
            chunk_libraries: Vec::new(),
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
        self.values.push(HumanValues::default());
        self.task_queues.push(TaskQueue::new());
        self.alive.push(true);
        self.social_memories.push(SocialMemory::new());
        self.event_buffers.push(EventBuffer::default());
        self.building_skills.push(0.0);
        self.combat_states.push(CombatState::default());
        self.assigned_houses.push(None);
        self.chunk_libraries.push(crate::skills::ChunkLibrary::with_basics());
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

    pub fn iter_homeless(&self) -> impl Iterator<Item = usize> + '_ {
        self.alive
            .iter()
            .enumerate()
            .filter(|(idx, &alive)| alive && self.assigned_houses[*idx].is_none())
            .map(|(i, _)| i)
    }
}

impl Default for HumanArchetype {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_has_social_memory() {
        let mut archetype = HumanArchetype::new();
        let id = EntityId::new();
        archetype.spawn(id, "Alice".into(), 0);

        assert_eq!(archetype.social_memories.len(), 1);
        assert_eq!(archetype.social_memories[0].slots.len(), 0); // Empty initially
    }

    #[test]
    fn test_human_has_building_skill() {
        let mut archetype = HumanArchetype::new();
        let id = EntityId::new();
        archetype.spawn(id, "Builder Bob".into(), 0);

        assert_eq!(archetype.building_skills.len(), 1);
        assert_eq!(archetype.building_skills[0], 0.0);
    }

    #[test]
    fn test_human_has_combat_state() {
        let mut archetype = HumanArchetype::new();
        let id = EntityId::new();
        archetype.spawn(id, "Warrior".into(), 0);

        assert_eq!(archetype.combat_states.len(), 1);
        assert!(archetype.combat_states[0].can_fight());
    }

    #[test]
    fn test_human_has_assigned_house() {
        let mut archetype = HumanArchetype::new();
        let id = EntityId::new();
        archetype.spawn(id, "Homeless Harry".into(), 0);

        assert_eq!(archetype.assigned_houses.len(), 1);
        assert_eq!(archetype.assigned_houses[0], None); // Starts homeless
    }

    #[test]
    fn test_iter_homeless() {
        let mut archetype = HumanArchetype::new();

        // Spawn 3 humans
        archetype.spawn(EntityId::new(), "Housed".into(), 0);
        archetype.spawn(EntityId::new(), "Homeless1".into(), 0);
        archetype.spawn(EntityId::new(), "Homeless2".into(), 0);

        // Assign first one to a house
        archetype.assigned_houses[0] = Some(BuildingId::new());

        let homeless: Vec<_> = archetype.iter_homeless().collect();

        assert_eq!(homeless.len(), 2);
        assert!(homeless.contains(&1));
        assert!(homeless.contains(&2));
        assert!(!homeless.contains(&0));
    }

    #[test]
    fn test_human_has_chunk_library() {
        let mut archetype = HumanArchetype::new();
        let id = EntityId::new();
        archetype.spawn(id, "Conscript".into(), 0);

        assert_eq!(archetype.chunk_libraries.len(), 1);
        // Fresh spawn has no combat chunks
        assert!(!archetype.chunk_libraries[0].has_chunk(crate::skills::ChunkId::BasicSwing));
    }
}
