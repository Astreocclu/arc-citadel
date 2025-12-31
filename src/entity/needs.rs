//! Universal needs that drive entity behavior

use serde::{Deserialize, Serialize};

/// Universal needs shared by all species
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Needs {
    /// 0.0 = fully rested, 1.0 = desperate for rest
    pub rest: f32,
    /// 0.0 = fed, 1.0 = starving
    pub food: f32,
    /// 0.0 = safe, 1.0 = in mortal danger
    pub safety: f32,
    /// 0.0 = socially satisfied, 1.0 = lonely
    pub social: f32,
    /// 0.0 = has purpose, 1.0 = aimless
    pub purpose: f32,
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            rest: 0.2,
            food: 0.2,
            safety: 0.1,
            social: 0.3,
            purpose: 0.3,
        }
    }
}

impl Needs {
    /// Get most pressing need
    pub fn most_pressing(&self) -> (NeedType, f32) {
        let needs = [
            (NeedType::Rest, self.rest),
            (NeedType::Food, self.food),
            (NeedType::Safety, self.safety),
            (NeedType::Social, self.social),
            (NeedType::Purpose, self.purpose),
        ];
        needs.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }

    /// Check if any need is critical (> 0.8)
    pub fn has_critical(&self) -> Option<NeedType> {
        if self.safety > 0.8 { return Some(NeedType::Safety); }
        if self.food > 0.8 { return Some(NeedType::Food); }
        if self.rest > 0.8 { return Some(NeedType::Rest); }
        None
    }

    /// Decay needs over time (called each tick)
    pub fn decay(&mut self, dt: f32, is_active: bool) {
        let activity_mult = if is_active { 1.5 } else { 1.0 };

        self.rest += 0.001 * dt * activity_mult;
        self.food += 0.0005 * dt;
        self.social += 0.0003 * dt;
        self.purpose += 0.0002 * dt;

        self.safety = (self.safety - 0.01 * dt).max(0.0);

        self.rest = self.rest.min(1.0);
        self.food = self.food.min(1.0);
        self.social = self.social.min(1.0);
        self.purpose = self.purpose.min(1.0);
    }

    /// Satisfy a need
    pub fn satisfy(&mut self, need: NeedType, amount: f32) {
        match need {
            NeedType::Rest => self.rest = (self.rest - amount).max(0.0),
            NeedType::Food => self.food = (self.food - amount).max(0.0),
            NeedType::Safety => self.safety = (self.safety - amount).max(0.0),
            NeedType::Social => self.social = (self.social - amount).max(0.0),
            NeedType::Purpose => self.purpose = (self.purpose - amount).max(0.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeedType {
    Rest,
    Food,
    Safety,
    Social,
    Purpose,
}
