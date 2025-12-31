//! Thought generation and management

use crate::core::types::EntityId;
use serde::{Deserialize, Serialize};

/// A thought is a reaction to a perceived event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    pub valence: Valence,
    pub intensity: f32,
    pub concept_category: String,
    pub cause_description: String,
    pub cause_type: CauseType,
    pub cause_entity: Option<EntityId>,
    pub created_tick: u64,
    pub decay_rate: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Valence {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CauseType {
    Object,
    Entity,
    Action,
    Need,
    Event,
}

impl Thought {
    pub fn new(
        valence: Valence,
        intensity: f32,
        concept: impl Into<String>,
        description: impl Into<String>,
        cause_type: CauseType,
        tick: u64,
    ) -> Self {
        Self {
            valence,
            intensity,
            concept_category: concept.into(),
            cause_description: description.into(),
            cause_type,
            cause_entity: None,
            created_tick: tick,
            decay_rate: 0.01,
        }
    }

    pub fn decay(&mut self) {
        self.intensity = (self.intensity - self.decay_rate).max(0.0);
    }

    pub fn is_faded(&self) -> bool {
        self.intensity < 0.1
    }
}

/// Buffer of active thoughts
#[derive(Debug, Clone, Default)]
pub struct ThoughtBuffer {
    thoughts: Vec<Thought>,
    max_thoughts: usize,
}

impl ThoughtBuffer {
    pub fn new() -> Self {
        Self {
            thoughts: Vec::new(),
            max_thoughts: 20,
        }
    }

    pub fn add(&mut self, thought: Thought) {
        if self.thoughts.len() >= self.max_thoughts {
            if let Some(pos) = self.thoughts
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.intensity.partial_cmp(&b.intensity).unwrap())
                .map(|(i, _)| i)
            {
                if self.thoughts[pos].intensity < thought.intensity {
                    self.thoughts.remove(pos);
                } else {
                    return;
                }
            }
        }
        self.thoughts.push(thought);
    }

    pub fn decay_all(&mut self) {
        for thought in &mut self.thoughts {
            thought.decay();
        }
        self.thoughts.retain(|t| !t.is_faded());
    }

    pub fn strongest(&self) -> Option<&Thought> {
        self.thoughts.iter()
            .max_by(|a, b| a.intensity.partial_cmp(&b.intensity).unwrap())
    }

    pub fn about_entity(&self, entity: EntityId) -> impl Iterator<Item = &Thought> {
        self.thoughts.iter()
            .filter(move |t| t.cause_entity == Some(entity))
    }

    pub fn positive(&self) -> impl Iterator<Item = &Thought> {
        self.thoughts.iter().filter(|t| t.valence == Valence::Positive)
    }

    pub fn negative(&self) -> impl Iterator<Item = &Thought> {
        self.thoughts.iter().filter(|t| t.valence == Valence::Negative)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Thought> {
        self.thoughts.iter()
    }
}
