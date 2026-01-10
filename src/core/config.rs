//! Simulation configuration with documented constants
//!
//! All magic numbers are collected here with explanations of their purpose
//! and how they interact with each other.

/// Configuration for the simulation systems
///
/// These values have been tuned to produce good emergent behavior.
/// Changing them will affect gameplay pacing and feel.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    // === SPATIAL SYSTEM ===
    /// Size of each cell in the spatial hash grid (world units)
    ///
    /// Should be approximately 1/5 of perception_range for optimal performance.
    /// Smaller = more cells, higher memory, fewer entities per cell
    /// Larger = fewer cells, lower memory, more entities to filter per query
    pub grid_cell_size: f32,

    /// How far entities can perceive other entities (world units)
    ///
    /// This affects:
    /// - How crowded perceptions become
    /// - Social interaction opportunities
    /// - Threat awareness distance
    pub perception_range: f32,

    // === NEED SYSTEM ===
    /// Rate at which rest need increases per tick when active
    ///
    /// At default rate (0.001), an entity reaches critical rest need (~0.8)
    /// in about 800 ticks of activity.
    pub rest_decay_rate: f32,

    /// Rate at which food need increases per tick
    ///
    /// At default rate (0.0005), an entity reaches critical hunger
    /// in about 1600 ticks (roughly 2x rest rate).
    pub food_decay_rate: f32,

    /// Rate at which social need increases per tick
    ///
    /// At default rate (0.0003), social needs build slowly,
    /// creating gradual pressure to seek interaction.
    pub social_decay_rate: f32,

    /// Rate at which purpose need increases per tick
    ///
    /// At default rate (0.0002), purpose is the slowest-building need,
    /// reflecting that aimlessness develops gradually.
    pub purpose_decay_rate: f32,

    /// Rate at which safety need decreases per tick (when no threats)
    ///
    /// At default rate (0.01), safety anxiety fades relatively quickly
    /// once threats are gone. This is intentionally faster than other
    /// decay rates to prevent entities from being permanently scared.
    pub safety_recovery_rate: f32,

    /// Multiplier for need decay when entity is actively working
    ///
    /// Active entities get tired faster. At 1.5x, an active entity
    /// reaches exhaustion 50% faster than a resting one.
    pub activity_multiplier: f32,

    /// Threshold above which a need is considered "critical"
    ///
    /// Critical needs trigger immediate responses and override
    /// normal action selection. This creates urgency.
    pub critical_need_threshold: f32,

    /// Threshold above which a need is considered "moderate"
    ///
    /// Moderate needs influence action selection but don't
    /// completely override other considerations.
    pub moderate_need_threshold: f32,

    // === THOUGHT SYSTEM ===
    /// Rate at which thought intensity decreases per tick
    ///
    /// At default rate (0.01), a thought at intensity 1.0 fades to
    /// 0.0 in about 100 ticks. This creates a "memory horizon".
    pub thought_decay_rate: f32,

    /// Maximum number of active thoughts per entity
    ///
    /// When buffer is full, weakest thoughts are evicted.
    /// More thoughts = richer mental state but higher memory use.
    pub max_thoughts: usize,

    /// Minimum intensity for a value-driven impulse to trigger
    ///
    /// Thoughts below this intensity won't drive value-based actions.
    /// Higher = more selective responses to strong feelings.
    pub impulse_intensity_threshold: f32,

    // === TASK SYSTEM ===
    /// Progress rate for continuous/instant actions (duration = 0)
    ///
    /// At 0.1 per tick, continuous actions complete in 10 ticks.
    pub progress_rate_instant: f32,

    /// Progress rate for quick actions (duration 1-60 ticks)
    ///
    /// At 0.05 per tick, quick actions complete in 20 ticks.
    pub progress_rate_quick: f32,

    /// Progress rate for long actions (duration > 60 ticks)
    ///
    /// At 0.02 per tick, long actions complete in 50 ticks.
    pub progress_rate_long: f32,

    /// Multiplier for need satisfaction from actions
    ///
    /// At 0.05, an action that nominally satisfies a need by 0.5
    /// actually provides 0.025 satisfaction per tick.
    ///
    /// WHY 0.05?
    /// - Actions run for multiple ticks, so total satisfaction accumulates
    /// - A Rest action (0.3 satisfaction × 0.05 × ~50 ticks) = 0.75 total
    /// - This creates meaningful time investment for need satisfaction
    pub satisfaction_multiplier: f32,

    // === PARALLELIZATION ===
    /// Minimum entity count before using parallel processing
    ///
    /// Below this threshold, thread overhead exceeds benefits.
    /// At 1000, we only parallelize when there are enough entities
    /// to justify the synchronization cost.
    pub parallel_threshold: usize,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            // Spatial (perception_range / 5 = cell_size)
            grid_cell_size: 10.0,
            perception_range: 50.0,

            // Need decay rates (rest > food > social > purpose)
            rest_decay_rate: 0.001,
            food_decay_rate: 0.0005,
            social_decay_rate: 0.0003,
            purpose_decay_rate: 0.0002,
            safety_recovery_rate: 0.01,
            activity_multiplier: 1.5,

            // Need thresholds
            critical_need_threshold: 0.8,
            moderate_need_threshold: 0.6,

            // Thoughts
            thought_decay_rate: 0.01,
            max_thoughts: 20,
            impulse_intensity_threshold: 0.7,

            // Task progress
            progress_rate_instant: 0.1,
            progress_rate_quick: 0.05,
            progress_rate_long: 0.02,
            satisfaction_multiplier: 0.05,

            // Parallelization
            parallel_threshold: 1000,
        }
    }
}

impl SimulationConfig {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate configuration for internal consistency
    pub fn validate(&self) -> Result<(), String> {
        // Cell size should be <= perception range / 3 for good query performance
        if self.grid_cell_size > self.perception_range / 3.0 {
            return Err(format!(
                "grid_cell_size ({}) should be <= perception_range / 3 ({:.1})",
                self.grid_cell_size,
                self.perception_range / 3.0
            ));
        }

        // Thresholds should be ordered
        if self.moderate_need_threshold >= self.critical_need_threshold {
            return Err(format!(
                "moderate_need_threshold ({}) should be < critical_need_threshold ({})",
                self.moderate_need_threshold, self.critical_need_threshold
            ));
        }

        // Decay rates should be positive
        if self.rest_decay_rate <= 0.0 || self.food_decay_rate <= 0.0 {
            return Err("Decay rates must be positive".into());
        }

        Ok(())
    }
}

// === GLOBAL CONFIG ACCESS ===

use std::sync::OnceLock;

static CONFIG: OnceLock<SimulationConfig> = OnceLock::new();

/// Get the global simulation config (initializes with defaults if not set)
pub fn config() -> &'static SimulationConfig {
    CONFIG.get_or_init(SimulationConfig::default)
}

/// Set the global simulation config (can only be called once)
///
/// Returns Err if config was already set.
pub fn set_config(config: SimulationConfig) -> Result<(), SimulationConfig> {
    CONFIG.set(config)
}
