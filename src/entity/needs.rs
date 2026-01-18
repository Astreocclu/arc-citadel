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
        needs
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }

    /// Check if any need is critical (> threshold)
    /// Threshold tuned to 0.65 so entities respond to needs sooner,
    /// enabling observable behavior within evaluation windows.
    pub fn has_critical(&self) -> Option<NeedType> {
        const CRITICAL_THRESHOLD: f32 = 0.65;
        if self.safety > CRITICAL_THRESHOLD {
            return Some(NeedType::Safety);
        }
        if self.food > CRITICAL_THRESHOLD {
            return Some(NeedType::Food);
        }
        if self.rest > CRITICAL_THRESHOLD {
            return Some(NeedType::Rest);
        }
        None
    }

    /// Decay needs over time (called each tick)
    ///
    /// Decay rates are tuned so that:
    /// - Rest: reaches critical (~0.65) in ~800 active ticks
    /// - Food: reaches critical in ~1200 ticks (eat less often than rest)
    /// - Social: slow buildup, ~2000+ ticks to critical
    /// - Purpose: slowest, ~3000+ ticks to critical
    /// - Safety: RECOVERS when no threats (asymmetric to others)
    ///
    /// See `core::config::SimulationConfig` for tunable values.
    pub fn decay(&mut self, dt: f32, is_active: bool) {
        // Active entities tire 50% faster (working, moving, fighting)
        let activity_mult = if is_active { 1.5 } else { 1.0 };

        // Needs INCREASE over time (0.0 = satisfied, 1.0 = desperate)
        // Target: reach 0.65 (critical) around tick 800 from starting value of ~0.3
        // This means ~0.35 increase over 800 ticks = 0.0004375/tick base
        // With activity_mult=1.5 when active: 0.0004375 / 1.5 = 0.000292/tick
        // Hierarchy: rest > food > social > purpose
        self.rest += 0.0003 * dt * activity_mult;   // ~800 active ticks to critical
        self.food += 0.0003 * dt;                   // ~1200 ticks to critical
        self.social += 0.00015 * dt;                // ~2300 ticks to critical (slow per E5)
        self.purpose += 0.0001 * dt;                // ~3500 ticks to critical (slowest)

        // Safety DECREASES when no threats (opposite of other needs)
        // Fast decay ensures safety returns to baseline after combat
        self.safety = (self.safety - 0.005 * dt).max(0.0);

        // Clamp to valid range
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
