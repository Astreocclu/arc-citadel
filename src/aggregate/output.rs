//! Simulation output and serialization

use std::time::Duration;
use serde::{Deserialize, Serialize};

use crate::aggregate::world::AggregateWorld;
use crate::aggregate::events::{HistoryLog, EventType};

/// Complete simulation output
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulationOutput {
    pub final_world: WorldSnapshot,
    pub history: HistoryLog,
    pub statistics: SimulationStats,
}

/// Serializable snapshot of world state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub year: u32,
    pub regions: Vec<crate::aggregate::region::Region>,
    pub polities: Vec<crate::aggregate::polity::Polity>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulationStats {
    pub years_simulated: u32,
    pub simulation_time_ms: u64,
    pub total_events: u32,
    pub wars_fought: u32,
    pub polities_at_start: u32,
    pub polities_at_end: u32,
    pub polities_destroyed: u32,
    pub polities_created: u32,
}

impl SimulationOutput {
    pub fn new(world: AggregateWorld, history: HistoryLog, years: u32, elapsed: Duration) -> Self {
        let polities_alive = world.polities.iter().filter(|p| p.alive).count() as u32;
        let polities_at_start = world.polities.len() as u32;

        let wars_fought = history.events.iter()
            .filter(|e| matches!(e.event_type, EventType::WarDeclared { .. }))
            .count() as u32;

        let total_events = history.events.len() as u32;

        Self {
            final_world: WorldSnapshot {
                year: world.year,
                regions: world.regions,
                polities: world.polities,
            },
            history,
            statistics: SimulationStats {
                years_simulated: years,
                simulation_time_ms: elapsed.as_millis() as u64,
                total_events,
                wars_fought,
                polities_at_start,
                polities_at_end: polities_alive,
                polities_destroyed: polities_at_start.saturating_sub(polities_alive),
                polities_created: 0,
            },
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn summary(&self) -> String {
        format!(
            "Simulated {} years in {}ms\n{} events, {} wars, {} polities remain",
            self.statistics.years_simulated,
            self.statistics.simulation_time_ms,
            self.statistics.total_events,
            self.statistics.wars_fought,
            self.statistics.polities_at_end,
        )
    }
}
