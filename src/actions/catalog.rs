//! Action definitions and catalog

use serde::{Deserialize, Serialize};
use crate::entity::needs::NeedType;

/// Unique action identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionId {
    MoveTo,
    Follow,
    Flee,
    Rest,
    Eat,
    SeekSafety,
    Build,
    Craft,
    Gather,
    Repair,
    TalkTo,
    Help,
    Trade,
    Attack,
    Defend,
    Charge,
    HoldPosition,
    IdleWander,
    IdleObserve,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCategory {
    Movement,
    Survival,
    Work,
    Social,
    Combat,
    Idle,
}

impl ActionId {
    pub fn category(&self) -> ActionCategory {
        match self {
            ActionId::MoveTo | ActionId::Follow | ActionId::Flee => ActionCategory::Movement,
            ActionId::Rest | ActionId::Eat | ActionId::SeekSafety => ActionCategory::Survival,
            ActionId::Build | ActionId::Craft | ActionId::Gather | ActionId::Repair => ActionCategory::Work,
            ActionId::TalkTo | ActionId::Help | ActionId::Trade => ActionCategory::Social,
            ActionId::Attack | ActionId::Defend | ActionId::Charge | ActionId::HoldPosition => ActionCategory::Combat,
            ActionId::IdleWander | ActionId::IdleObserve => ActionCategory::Idle,
        }
    }

    pub fn satisfies_needs(&self) -> Vec<(NeedType, f32)> {
        match self {
            ActionId::Rest => vec![(NeedType::Rest, 0.3)],
            ActionId::Eat => vec![(NeedType::Food, 0.5)],
            ActionId::SeekSafety | ActionId::Flee => vec![(NeedType::Safety, 0.3)],
            ActionId::TalkTo | ActionId::Help => vec![(NeedType::Social, 0.3)],
            ActionId::Build | ActionId::Craft | ActionId::Gather => vec![(NeedType::Purpose, 0.3)],
            _ => vec![],
        }
    }

    pub fn is_interruptible(&self) -> bool {
        match self {
            ActionId::Attack | ActionId::Charge => false,
            _ => true,
        }
    }

    pub fn base_duration(&self) -> u32 {
        match self {
            ActionId::Attack | ActionId::Defend => 1,
            ActionId::TalkTo => 60,
            ActionId::Rest => 600,
            ActionId::Eat => 30,
            ActionId::Build => 3600,
            ActionId::Craft => 1800,
            _ => 0,
        }
    }
}

pub struct ActionAvailability {
    pub available: bool,
    pub reason: Option<String>,
}

impl ActionAvailability {
    pub fn yes() -> Self {
        Self { available: true, reason: None }
    }

    pub fn no(reason: impl Into<String>) -> Self {
        Self { available: false, reason: Some(reason.into()) }
    }
}
