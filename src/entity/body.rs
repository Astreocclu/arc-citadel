//! Physical body simulation

use serde::{Deserialize, Serialize};

/// Physical state of an entity's body
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BodyState {
    /// 0.0 = fresh, 1.0 = exhausted
    pub fatigue: f32,
    /// 0.0 = fed, 1.0 = starving
    pub hunger: f32,
    /// 0.0 = none, 1.0 = incapacitating
    pub pain: f32,
    /// Computed from wounds
    pub overall_health: f32,
}

impl BodyState {
    pub fn new() -> Self {
        Self {
            fatigue: 0.0,
            hunger: 0.0,
            pain: 0.0,
            overall_health: 1.0,
        }
    }

    /// Check if entity can act
    pub fn can_act(&self) -> bool {
        self.overall_health > 0.1 && self.fatigue < 0.95 && self.pain < 0.9
    }

    /// Check if entity can move
    pub fn can_move(&self) -> bool {
        self.can_act() && self.fatigue < 0.9
    }

    /// Apply fatigue from activity
    pub fn add_fatigue(&mut self, amount: f32) {
        self.fatigue = (self.fatigue + amount).min(1.0);
    }

    /// Recover fatigue from rest
    pub fn recover_fatigue(&mut self, amount: f32) {
        self.fatigue = (self.fatigue - amount).max(0.0);
    }
}

/// Individual wound on a body part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wound {
    pub body_part: BodyPart,
    pub wound_type: WoundType,
    pub severity: f32,
    pub infected: bool,
    pub tick_received: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyPart {
    Head,
    Torso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WoundType {
    Cut,
    Pierce,
    Blunt,
    Burn,
}

/// Collection of wounds on an entity
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Wounds {
    pub wounds: Vec<Wound>,
}

impl Wounds {
    pub fn new() -> Self {
        Self { wounds: Vec::new() }
    }

    pub fn add(&mut self, wound: Wound) {
        self.wounds.push(wound);
    }

    /// Calculate overall health from wounds
    pub fn calculate_health(&self) -> f32 {
        if self.wounds.is_empty() {
            return 1.0;
        }

        let total_severity: f32 = self.wounds.iter()
            .map(|w| w.severity)
            .sum();

        (1.0 - total_severity).max(0.0)
    }

    /// Check if any limb prevents movement
    pub fn can_walk(&self) -> bool {
        !self.wounds.iter().any(|w| {
            matches!(w.body_part, BodyPart::LeftLeg | BodyPart::RightLeg)
                && w.severity > 0.5
        })
    }
}
