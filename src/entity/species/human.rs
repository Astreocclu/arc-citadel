//! Human-specific archetype with SoA layout

use crate::city::BuildingId;
use crate::combat::CombatState;
use crate::core::types::{EntityId, Tick, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::social::{EventBuffer, SocialMemory};
use crate::entity::tasks::TaskQueue;
use crate::entity::thoughts::ThoughtBuffer;
use crate::entity::EntityArchetype;
use crate::skills::{
    generate_chunks_from_history, generate_history_for_role, generate_spawn_chunks,
    LifeExperience, Role,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

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

    /// Spawn a new entity with default (Peasant) archetype
    /// Use spawn_with_archetype() for specific archetypes
    pub fn spawn(&mut self, id: EntityId, name: String, tick: Tick) {
        self.spawn_with_archetype(id, name, tick, EntityArchetype::Peasant, 25);
    }

    /// Spawn a new entity with chunks based on archetype and age
    pub fn spawn_with_archetype(
        &mut self,
        id: EntityId,
        name: String,
        tick: Tick,
        archetype: EntityArchetype,
        age: u32,
    ) {
        // Create RNG seeded from entity ID for reproducibility
        let mut rng = ChaCha8Rng::seed_from_u64(id.0.as_u128() as u64);

        self.ids.push(id);
        self.names.push(name);
        // Calculate birth tick from age (approximate: 365 days * 24 ticks/day)
        self.birth_ticks.push(tick.saturating_sub((age as u64) * 365 * 24));
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
        self.building_skills.push(0.0); // Deprecated, use chunk_libraries
        self.combat_states.push(CombatState::default());
        self.assigned_houses.push(None);
        self.chunk_libraries
            .push(generate_spawn_chunks(archetype, age, tick, &mut rng));
    }

    /// Spawn a new entity with chunks based on role and age.
    ///
    /// This is the preferred spawn method. It generates a plausible life
    /// history and derives chunks from that history.
    pub fn spawn_with_role(
        &mut self,
        id: EntityId,
        name: String,
        tick: Tick,
        role: Role,
        age: u32,
    ) {
        let mut rng = ChaCha8Rng::seed_from_u64(id.0.as_u128() as u64);

        // Generate life history from role and age
        let history = generate_history_for_role(role, age, &mut rng);

        // Generate chunks from history
        let chunks = generate_chunks_from_history(&history, tick, &mut rng);

        self.ids.push(id);
        self.names.push(name);
        self.birth_ticks.push(tick.saturating_sub((age as u64) * 365 * 24));
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
        self.chunk_libraries.push(chunks);
    }

    /// Spawn with explicit history (for important NPCs)
    pub fn spawn_with_history(
        &mut self,
        id: EntityId,
        name: String,
        tick: Tick,
        history: &[LifeExperience],
        age: u32,
    ) {
        let mut rng = ChaCha8Rng::seed_from_u64(id.0.as_u128() as u64);
        let chunks = generate_chunks_from_history(history, tick, &mut rng);

        self.ids.push(id);
        self.names.push(name);
        self.birth_ticks.push(tick.saturating_sub((age as u64) * 365 * 24));
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
        self.chunk_libraries.push(chunks);
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
        archetype.spawn(id, "Peasant".into(), 0);

        assert_eq!(archetype.chunk_libraries.len(), 1);
        // Default spawn (Peasant, age 25) has physical chunks but no combat chunks
        assert!(archetype.chunk_libraries[0].has_chunk(crate::skills::ChunkId::PhysEfficientGait));
        assert!(!archetype.chunk_libraries[0].has_chunk(crate::skills::ChunkId::BasicSwing));
    }

    #[test]
    fn test_spawn_creates_chunks() {
        let mut archetype = HumanArchetype::new();
        let id = EntityId::new();

        archetype.spawn_with_archetype(
            id,
            "Test".to_string(),
            0,
            crate::entity::EntityArchetype::Peasant,
            25,
        );

        let idx = archetype.index_of(id).unwrap();
        assert!(!archetype.chunk_libraries[idx].chunks().is_empty());

        // Verify chunk state is properly initialized
        let state = archetype.chunk_libraries[idx]
            .get_chunk(crate::skills::ChunkId::PhysEfficientGait)
            .unwrap();
        assert!(state.encoding_depth > 0.0);
        assert!(state.repetition_count > 0);
    }

    #[test]
    fn test_spawn_reproducibility() {
        // Same entity ID should produce identical chunk libraries
        let id = EntityId::new();

        let mut arch1 = HumanArchetype::new();
        arch1.spawn_with_archetype(
            id,
            "Test".to_string(),
            100,
            crate::entity::EntityArchetype::Peasant,
            30,
        );

        let mut arch2 = HumanArchetype::new();
        arch2.spawn_with_archetype(
            id,
            "Test".to_string(),
            100,
            crate::entity::EntityArchetype::Peasant,
            30,
        );

        let state1 = arch1.chunk_libraries[0]
            .get_chunk(crate::skills::ChunkId::PhysEfficientGait)
            .unwrap();
        let state2 = arch2.chunk_libraries[0]
            .get_chunk(crate::skills::ChunkId::PhysEfficientGait)
            .unwrap();

        assert_eq!(state1.encoding_depth, state2.encoding_depth);
        assert_eq!(state1.repetition_count, state2.repetition_count);
    }

    #[test]
    fn test_spawn_with_soldier_archetype() {
        let mut archetype = HumanArchetype::new();
        let id = EntityId::new();

        archetype.spawn_with_archetype(
            id,
            "Soldier".to_string(),
            0,
            crate::entity::EntityArchetype::Soldier {
                training: crate::entity::TrainingLevel::Regular,
            },
            30,
        );

        let idx = archetype.index_of(id).unwrap();
        // Soldier should have combat chunks
        assert!(archetype.chunk_libraries[idx].has_chunk(crate::skills::ChunkId::BasicSwing));
        assert!(archetype.chunk_libraries[idx].has_chunk(crate::skills::ChunkId::BasicStance));
    }

    #[test]
    fn test_spawn_with_role() {
        use crate::skills::Role;

        let mut archetype = HumanArchetype::new();
        let id = EntityId::new();

        archetype.spawn_with_role(id, "Farmer".to_string(), 0, Role::Farmer, 30);

        let idx = archetype.index_of(id).unwrap();
        assert!(archetype.chunk_libraries[idx].has_chunk(crate::skills::ChunkId::PhysSustainedLabor));
    }

    #[test]
    fn test_spawn_with_role_soldier() {
        use crate::skills::Role;

        let mut archetype = HumanArchetype::new();
        let id = EntityId::new();

        archetype.spawn_with_role(id, "Soldier".to_string(), 0, Role::Soldier, 25);

        let idx = archetype.index_of(id).unwrap();
        assert!(archetype.chunk_libraries[idx].has_chunk(crate::skills::ChunkId::BasicStance));
    }

    #[test]
    fn test_spawn_with_history() {
        use crate::skills::{ActivityType, LifeExperience};

        let history = vec![
            LifeExperience {
                activity: ActivityType::GeneralLife,
                duration_years: 12.0,
                intensity: 1.0,
                training_quality: 0.5,
            },
            LifeExperience {
                activity: ActivityType::Smithing,
                duration_years: 20.0,
                intensity: 1.0,
                training_quality: 0.8,
            },
        ];

        let mut archetype = HumanArchetype::new();
        let id = EntityId::new();

        archetype.spawn_with_history(id, "Master Smith".to_string(), 0, &history, 32);

        let idx = archetype.index_of(id).unwrap();
        assert!(archetype.chunk_libraries[idx].has_chunk(crate::skills::ChunkId::CraftBasicHammerWork));

        // Master smith should have deep encoding
        let depth = archetype.chunk_libraries[idx]
            .get_chunk(crate::skills::ChunkId::CraftBasicHammerWork)
            .unwrap()
            .encoding_depth;
        assert!(depth > 0.5);
    }

    #[test]
    fn test_older_farmer_more_skilled() {
        use crate::skills::Role;

        let mut archetype = HumanArchetype::new();

        let young_id = EntityId::new();
        archetype.spawn_with_role(young_id, "Young".to_string(), 0, Role::Farmer, 20);

        let old_id = EntityId::new();
        archetype.spawn_with_role(old_id, "Old".to_string(), 0, Role::Farmer, 50);

        let young_idx = archetype.index_of(young_id).unwrap();
        let old_idx = archetype.index_of(old_id).unwrap();

        let young_depth = archetype.chunk_libraries[young_idx]
            .get_chunk(crate::skills::ChunkId::PhysSustainedLabor)
            .unwrap()
            .encoding_depth;
        let old_depth = archetype.chunk_libraries[old_idx]
            .get_chunk(crate::skills::ChunkId::PhysSustainedLabor)
            .unwrap()
            .encoding_depth;

        assert!(old_depth > young_depth);
    }

    #[test]
    fn test_spawn_with_craftsman_archetype() {
        let mut archetype = HumanArchetype::new();
        let id = EntityId::new();

        archetype.spawn_with_archetype(
            id,
            "Smith".to_string(),
            0,
            crate::entity::EntityArchetype::Craftsman {
                specialty: crate::entity::CraftSpecialty::Smithing,
            },
            35,
        );

        let idx = archetype.index_of(id).unwrap();
        // Smith should have forging chunks
        assert!(archetype.chunk_libraries[idx].has_chunk(crate::skills::ChunkId::CraftBasicHammerWork));
    }
}
