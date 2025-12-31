//! Human-specific archetype with SoA layout

use crate::core::types::{EntityId, Vec2, Tick};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::thoughts::ThoughtBuffer;
use crate::entity::tasks::TaskQueue;

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
        values.into_iter()
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

impl Default for HumanArchetype {
    fn default() -> Self {
        Self::new()
    }
}
