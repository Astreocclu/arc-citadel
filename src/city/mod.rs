//! City layer - buildings, construction, and production

pub mod building;
pub mod construction;

pub use building::{BuildingArchetype, BuildingId, BuildingState, BuildingType};
pub use construction::{
    apply_construction_work, calculate_team_contribution, calculate_worker_contribution,
    ContributionResult,
};
