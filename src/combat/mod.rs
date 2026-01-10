//! Combat resolution system
//!
//! Philosophy: Property interaction, not percentage modifiers.
//! NO multiplicative stacking. Categorical outcomes from comparisons.
//!
//! Two victory paths:
//! 1. DAMAGE PATH: Inflict wounds until they can't fight
//! 2. MORALE PATH: Inflict stress until they break and flee

pub mod armor;
pub mod body_zone;
pub mod constants;
pub mod formation;
pub mod morale;
pub mod penetration;
pub mod resolution;
pub mod skill;
pub mod stance;
pub mod state;
pub mod trauma;
pub mod weapons;
pub mod wounds;

pub use armor::{ArmorProperties, Coverage, Padding, Rigidity};
pub use body_zone::{BodyZone, WoundSeverity};
pub use formation::{FormationState, PressureCategory, ShockType};
pub use morale::{BreakResult, MoraleState, StressSource};
pub use penetration::{resolve_penetration, PenetrationResult};
pub use resolution::{resolve_exchange, Combatant, ExchangeResult};
pub use skill::{CombatSkill, SkillLevel};
pub use stance::{CombatStance, StanceTransitions, TransitionTrigger};
pub use state::CombatState;
pub use trauma::{resolve_trauma, TraumaResult};
pub use weapons::{Edge, Mass, Reach, WeaponProperties, WeaponSpecial};
pub use wounds::{combine_results, Wound};
