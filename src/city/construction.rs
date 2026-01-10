//! Construction system - handles building progress from worker contributions

use crate::city::building::{BuildingArchetype, BuildingState};

/// Base construction rate per tick
const BASE_RATE: f32 = 1.0;

/// Result of a worker contribution
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContributionResult {
    /// Work contributed, building still under construction
    InProgress { contributed: f32 },
    /// Work contributed, building is now complete
    Completed { contributed: f32 },
    /// Building is already complete
    AlreadyComplete,
    /// Building not found
    NotFound,
}

/// Calculate team contribution with diminishing returns
///
/// Formula: sqrt(workers)
/// - 1 worker = 1.0
/// - 2 workers = 1.41
/// - 3 workers = 1.73
/// - 4 workers = 2.0
/// - 5 workers = 2.24
pub fn calculate_team_contribution(worker_count: u32, max_workers: u32) -> f32 {
    let effective = worker_count.min(max_workers) as f32;
    effective.sqrt()
}

/// Calculate individual worker contribution per tick
///
/// Formula: BASE_RATE * (0.5 + skill * 0.5) * (1.0 - fatigue.clamp(0.0, 1.0) * 0.4)
/// - Skill 0.0, Fatigue 0.0 => 0.5
/// - Skill 1.0, Fatigue 0.0 => 1.0
/// - Skill 1.0, Fatigue 1.0 => 0.6
/// - Skill 0.5, Fatigue 0.5 => 0.6
pub fn calculate_worker_contribution(building_skill: f32, fatigue: f32) -> f32 {
    let skill_multiplier = 0.5 + building_skill * 0.5; // 0.5 to 1.0
    let fatigue_penalty = 1.0 - fatigue.clamp(0.0, 1.0) * 0.4; // 0.6 to 1.0
    BASE_RATE * skill_multiplier * fatigue_penalty
}

/// Apply construction work to a building
pub fn apply_construction_work(
    buildings: &mut BuildingArchetype,
    building_idx: usize,
    work_amount: f32,
    current_tick: u64,
) -> ContributionResult {
    if building_idx >= buildings.count() {
        return ContributionResult::NotFound;
    }

    if buildings.states[building_idx] == BuildingState::Complete {
        return ContributionResult::AlreadyComplete;
    }

    let work_required = buildings.building_types[building_idx].work_required();
    buildings.construction_progress[building_idx] += work_amount;

    if buildings.construction_progress[building_idx] >= work_required {
        buildings.states[building_idx] = BuildingState::Complete;
        buildings.completed_ticks[building_idx] = current_tick;
        ContributionResult::Completed {
            contributed: work_amount,
        }
    } else {
        ContributionResult::InProgress {
            contributed: work_amount,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::city::building::{BuildingId, BuildingType};
    use crate::core::types::Vec2;

    #[test]
    fn test_team_contribution_diminishing_returns() {
        // 1 worker = 1.0
        assert!((calculate_team_contribution(1, 5) - 1.0).abs() < 0.01);
        // 2 workers = ~1.41
        assert!((calculate_team_contribution(2, 5) - 1.414).abs() < 0.01);
        // 4 workers = 2.0
        assert!((calculate_team_contribution(4, 5) - 2.0).abs() < 0.01);
        // Over max is capped
        assert!(
            (calculate_team_contribution(10, 5) - calculate_team_contribution(5, 5)).abs() < 0.01
        );
    }

    #[test]
    fn test_worker_contribution_formula() {
        // No skill, no fatigue: 0.5 * 1.0 = 0.5
        let contrib = calculate_worker_contribution(0.0, 0.0);
        assert!((contrib - 0.5).abs() < 0.01);

        // Max skill, no fatigue: 1.0 * 1.0 = 1.0
        let contrib = calculate_worker_contribution(1.0, 0.0);
        assert!((contrib - 1.0).abs() < 0.01);

        // Max skill, max fatigue: 1.0 * 0.6 = 0.6
        let contrib = calculate_worker_contribution(1.0, 1.0);
        assert!((contrib - 0.6).abs() < 0.01);

        // Mid skill (0.75), mid fatigue: 0.75 * 0.8 = 0.6
        let contrib = calculate_worker_contribution(0.5, 0.5);
        assert!((contrib - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_worker_contribution_fatigue_clamped() {
        // Fatigue > 1.0 should be clamped
        let contrib_clamped = calculate_worker_contribution(1.0, 2.0);
        let contrib_max = calculate_worker_contribution(1.0, 1.0);
        assert!((contrib_clamped - contrib_max).abs() < 0.01);

        // Fatigue < 0.0 should be clamped
        let contrib_clamped = calculate_worker_contribution(1.0, -1.0);
        let contrib_zero = calculate_worker_contribution(1.0, 0.0);
        assert!((contrib_clamped - contrib_zero).abs() < 0.01);
    }

    #[test]
    fn test_apply_construction_work() {
        let mut buildings = BuildingArchetype::new();
        let id = BuildingId::new();
        buildings.spawn(id, BuildingType::Wall, Vec2::new(0.0, 0.0), 0);

        // Wall requires 80.0 work
        let result = apply_construction_work(&mut buildings, 0, 40.0, 100);
        assert!(matches!(result, ContributionResult::InProgress { .. }));
        assert!((buildings.construction_progress[0] - 40.0).abs() < 0.01);

        // Complete it
        let result = apply_construction_work(&mut buildings, 0, 50.0, 200);
        assert!(matches!(result, ContributionResult::Completed { .. }));
        assert_eq!(buildings.states[0], BuildingState::Complete);
        assert_eq!(buildings.completed_ticks[0], 200);
    }

    #[test]
    fn test_apply_construction_already_complete() {
        let mut buildings = BuildingArchetype::new();
        let id = BuildingId::new();
        buildings.spawn(id, BuildingType::Wall, Vec2::new(0.0, 0.0), 0);
        buildings.states[0] = BuildingState::Complete;

        let result = apply_construction_work(&mut buildings, 0, 10.0, 100);
        assert_eq!(result, ContributionResult::AlreadyComplete);
    }

    #[test]
    fn test_apply_construction_not_found() {
        let mut buildings = BuildingArchetype::new();

        let result = apply_construction_work(&mut buildings, 0, 10.0, 100);
        assert_eq!(result, ContributionResult::NotFound);
    }

    #[test]
    fn test_construction_completion_transition() {
        let mut buildings = BuildingArchetype::new();
        let id = BuildingId::new();
        buildings.spawn(id, BuildingType::Gate, Vec2::new(5.0, 5.0), 50);

        // Gate requires 60.0 work
        assert_eq!(buildings.states[0], BuildingState::UnderConstruction);

        // Apply exactly enough work
        let result = apply_construction_work(&mut buildings, 0, 60.0, 150);

        assert!(
            matches!(result, ContributionResult::Completed { contributed } if (contributed - 60.0).abs() < 0.01)
        );
        assert_eq!(buildings.states[0], BuildingState::Complete);
        assert_eq!(buildings.completed_ticks[0], 150);
        assert!((buildings.construction_progress[0] - 60.0).abs() < 0.01);
    }
}
