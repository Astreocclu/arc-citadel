//! Battle Scoring System
//!
//! quantifies battle performance for AI optimization (DSPy/Genetic Algorithms).

use crate::battle::execution::{BattleOutcome, BattleState};
use serde::{Deserialize, Serialize};

/// Weights for different aspects of battle performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreWeights {
    /// Points for winning
    pub win_bonus: f32,
    /// Points deducted for losing
    pub defeat_penalty: f32,
    /// Multiplier for (Enemy Loss % - Friendly Loss %)
    /// Positive means we want to trade efficiently
    pub efficiency_weight: f32,
    /// Points per tick remaining (encourages speed)
    pub speed_bonus: f32,
    /// Points for preserving own strength (1.0 = 100% strength)
    pub survival_weight: f32,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            win_bonus: 1000.0,
            defeat_penalty: -500.0,
            efficiency_weight: 500.0,
            speed_bonus: 0.5,
            survival_weight: 200.0,
        }
    }
}

/// Detailed score report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleScore {
    pub outcome: BattleOutcome,
    pub ticks_taken: u64,
    pub friendly_casualties_percent: f32,
    pub enemy_casualties_percent: f32,
    pub efficiency_delta: f32,
    pub raw_score: f32,
}

/// Calculate score for a completed battle
pub fn calculate_score(state: &BattleState, weights: &ScoreWeights, max_ticks: u64) -> BattleScore {
    let friendly_start = state.friendly_army.total_strength() as f32;
    let friendly_current = state.friendly_army.effective_strength() as f32;
    let enemy_start = state.enemy_army.total_strength() as f32;
    let enemy_current = state.enemy_army.effective_strength() as f32;

    let friendly_loss_pct = if friendly_start > 0.0 {
        1.0 - (friendly_current / friendly_start)
    } else {
        1.0
    };

    let enemy_loss_pct = if enemy_start > 0.0 {
        1.0 - (enemy_current / enemy_start)
    } else {
        1.0
    };

    let efficiency_delta = enemy_loss_pct - friendly_loss_pct;
    
    let mut score = 0.0;

    // Outcome score
    match state.outcome {
        BattleOutcome::DecisiveVictory => score += weights.win_bonus * 1.5,
        BattleOutcome::Victory => score += weights.win_bonus,
        BattleOutcome::PyrrhicVictory => score += weights.win_bonus * 0.5,
        BattleOutcome::Draw | BattleOutcome::MutualRout => score += 0.0,
        BattleOutcome::Defeat => score += weights.defeat_penalty,
        BattleOutcome::DecisiveDefeat => score += weights.defeat_penalty * 1.5,
        BattleOutcome::Undecided => score += weights.defeat_penalty * 2.0, // Should not happen
    }

    // Efficiency score
    score += efficiency_delta * weights.efficiency_weight;

    // Survival score
    score += (1.0 - friendly_loss_pct) * weights.survival_weight;

    // Speed score (only if we won)
    if matches!(
        state.outcome,
        BattleOutcome::DecisiveVictory | BattleOutcome::Victory | BattleOutcome::PyrrhicVictory
    ) {
        let ticks_saved = max_ticks.saturating_sub(state.tick);
        score += ticks_saved as f32 * weights.speed_bonus;
    }

    BattleScore {
        outcome: state.outcome,
        ticks_taken: state.tick,
        friendly_casualties_percent: friendly_loss_pct,
        enemy_casualties_percent: enemy_loss_pct,
        efficiency_delta,
        raw_score: score,
    }
}
