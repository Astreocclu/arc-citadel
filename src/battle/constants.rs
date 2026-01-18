//! Battle system constants - all tunable values in one place
//!
//! These values are ADDITIVE, never multiplicative. No percentage modifiers.

// Battle map scale
pub const BATTLE_HEX_SIZE_METERS: f32 = 20.0;
pub const DEFAULT_BATTLE_WIDTH: u32 = 50;
pub const DEFAULT_BATTLE_HEIGHT: u32 = 40;

// Time
pub const BATTLE_TICK_MS: u32 = 100;
pub const BATTLE_TICK_SIM_SECONDS: f32 = 1.0;
pub const MAX_BATTLE_TICKS: u64 = 6000; // 10 minutes

// Movement (hexes per tick, where 1 tick = 1 second, 1 hex = 20 meters)
// Real-world reference: infantry march ~5 km/h (1.4 m/s), cavalry trot ~14 km/h (3.9 m/s)
pub const INFANTRY_WALK_SPEED: f32 = 0.07;   // ~5 km/h marching pace
pub const INFANTRY_RUN_SPEED: f32 = 0.14;    // ~10 km/h jogging
pub const CAVALRY_WALK_SPEED: f32 = 0.085;   // ~6 km/h
pub const CAVALRY_TROT_SPEED: f32 = 0.20;    // ~14 km/h
pub const CAVALRY_CHARGE_SPEED: f32 = 0.50;  // ~36 km/h (canter/gallop burst)
pub const COURIER_SPEED: f32 = 0.40;         // ~29 km/h (sustained fast pace)
pub const ROUT_SPEED: f32 = 0.18;            // Panicked running, faster than march

// Vision (hexes)
pub const BASE_VISION_RANGE: u32 = 8;
pub const SCOUT_VISION_BONUS: u32 = 4;
pub const ELEVATION_VISION_BONUS: u32 = 2;
pub const FOREST_VISION_PENALTY: u32 = 4;

// Combat rates (per tick) - ADDITIVE
pub const BASE_CASUALTY_RATE: f32 = 0.02;
pub const FATIGUE_RATE_COMBAT: f32 = 0.02;
pub const FATIGUE_RATE_MARCH: f32 = 0.005;
pub const FATIGUE_RECOVERY_RATE: f32 = 0.01;

// Stress - ADDITIVE thresholds
pub const CONTAGION_STRESS: f32 = 0.10;
pub const OFFICER_DEATH_STRESS: f32 = 0.30;
pub const FLANK_STRESS: f32 = 0.20;

// Rally - ticks required to transition from Rallying to Formed
pub const RALLY_TICKS_REQUIRED: u64 = 30;

// Courier
pub const COURIER_INTERCEPTION_RANGE: u32 = 2;
pub const COURIER_INTERCEPTION_CHANCE_PATROL: f32 = 0.5;
pub const COURIER_INTERCEPTION_CHANCE_ALERT: f32 = 0.7;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_size_reasonable() {
        assert!(BATTLE_HEX_SIZE_METERS > 10.0 && BATTLE_HEX_SIZE_METERS < 50.0);
    }

    #[test]
    fn test_speed_ordering() {
        assert!(CAVALRY_CHARGE_SPEED > CAVALRY_TROT_SPEED);
        assert!(CAVALRY_TROT_SPEED > INFANTRY_RUN_SPEED);
        assert!(INFANTRY_RUN_SPEED > INFANTRY_WALK_SPEED);
    }

    #[test]
    fn test_vision_ranges_positive() {
        assert!(BASE_VISION_RANGE > 0);
        assert!(SCOUT_VISION_BONUS > 0);
    }
}
