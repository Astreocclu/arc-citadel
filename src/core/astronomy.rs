//! Astronomical system - sun, moons, seasons, and celestial events
//!
//! This module provides the core enums and constants for the astronomical system.

use serde::{Deserialize, Serialize};
use crate::core::calendar::TimePeriod;

// ============================================================================
// Constants
// ============================================================================

/// Number of days in a year
pub const YEAR_LENGTH: u16 = 360;

/// Number of simulation ticks per day
pub const TICKS_PER_DAY: u64 = 1000;

/// Silver Moon (Argent) orbital period in days
pub const ARGENT_PERIOD: u16 = 29;

/// Blood Moon (Sanguine) orbital period in days
pub const SANGUINE_PERIOD: u16 = 83;

/// Argent node precession period in days (~18 years)
pub const ARGENT_NODE_PRECESSION: u32 = 6570;

/// Sanguine node precession period in days (~31 years)
pub const SANGUINE_NODE_PRECESSION: u32 = 11160;

/// Conjunction cycle - LCM(29, 83) - when both moons align
pub const CONJUNCTION_CYCLE: u32 = 2407;

// ============================================================================
// Enums
// ============================================================================

/// Solar phase - 9 phases of the day with exact hour ranges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SolarPhase {
    /// 00:00-04:00 - Deepest darkness
    DeepNight,
    /// 04:00-06:00 - Sky begins to lighten
    PreDawn,
    /// 06:00-08:00 - Sun rises
    Dawn,
    /// 08:00-11:00 - Morning light
    Morning,
    /// 11:00-14:00 - Peak daylight
    Midday,
    /// 14:00-17:00 - Afternoon sun
    Afternoon,
    /// 17:00-19:00 - Sun sets
    Dusk,
    /// 19:00-22:00 - Twilight to dark
    Evening,
    /// 22:00-00:00 - Night begins
    Night,
}

impl SolarPhase {
    /// Get solar phase from hour (0-23)
    pub fn from_hour(hour: u32) -> Self {
        match hour {
            0..=3 => SolarPhase::DeepNight,
            4..=5 => SolarPhase::PreDawn,
            6..=7 => SolarPhase::Dawn,
            8..=10 => SolarPhase::Morning,
            11..=13 => SolarPhase::Midday,
            14..=16 => SolarPhase::Afternoon,
            17..=18 => SolarPhase::Dusk,
            19..=21 => SolarPhase::Evening,
            _ => SolarPhase::Night, // 22, 23
        }
    }

    /// Base light level for this phase (0.0-1.0)
    pub fn base_light_level(&self) -> f32 {
        match self {
            SolarPhase::DeepNight => 0.0,
            SolarPhase::PreDawn => 0.1,
            SolarPhase::Dawn => 0.5,
            SolarPhase::Morning => 0.8,
            SolarPhase::Midday => 1.0,
            SolarPhase::Afternoon => 0.85,
            SolarPhase::Dusk => 0.5,
            SolarPhase::Evening => 0.2,
            SolarPhase::Night => 0.05,
        }
    }
}

/// Convert SolarPhase to TimePeriod for backward compatibility with expectations system
impl From<SolarPhase> for TimePeriod {
    fn from(phase: SolarPhase) -> Self {
        match phase {
            SolarPhase::Dawn | SolarPhase::Morning => TimePeriod::Morning,
            SolarPhase::Midday | SolarPhase::Afternoon => TimePeriod::Afternoon,
            SolarPhase::Dusk | SolarPhase::Evening => TimePeriod::Evening,
            SolarPhase::Night | SolarPhase::DeepNight | SolarPhase::PreDawn => TimePeriod::Night,
        }
    }
}

/// Season of the year (90 days each)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Season {
    /// Days 1-90
    #[default]
    Spring,
    /// Days 91-180
    Summer,
    /// Days 181-270
    Autumn,
    /// Days 271-360
    Winter,
}

impl Season {
    /// Get season from day of year (1-360)
    pub fn from_day_of_year(day: u16) -> Self {
        match day {
            1..=90 => Season::Spring,
            91..=180 => Season::Summer,
            181..=270 => Season::Autumn,
            _ => Season::Winter, // 271-360
        }
    }
}

/// Celestial events that can occur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CelestialEvent {
    // Common (every cycle)
    /// Silver Moon is full
    FullArgent,
    /// Silver Moon is new (dark)
    NewArgent,
    /// Blood Moon is full
    FullSanguine,
    /// Blood Moon is new (dark)
    NewSanguine,

    // Uncommon (every few months)
    /// Both moons within 2 days of full
    NearDoubleFull,
    /// Both moons within 2 days of new
    NearDoubleNew,
    /// Argent eclipses sun
    SilverEclipse,
    /// Sanguine eclipses sun
    BloodEclipse,

    // Rare (years apart)
    /// Both moons exactly full (every 2407 days)
    PerfectDoubleFull,
    /// Both moons exactly new
    PerfectDoubleNew,
    /// Both moons eclipse same day
    DoubleEclipse,
}

impl CelestialEvent {
    /// Is this a rare event (years apart)?
    pub fn is_rare(&self) -> bool {
        matches!(
            self,
            CelestialEvent::PerfectDoubleFull
                | CelestialEvent::PerfectDoubleNew
                | CelestialEvent::DoubleEclipse
        )
    }

    /// Is this a common event (every cycle)?
    pub fn is_common(&self) -> bool {
        matches!(
            self,
            CelestialEvent::FullArgent
                | CelestialEvent::NewArgent
                | CelestialEvent::FullSanguine
                | CelestialEvent::NewSanguine
        )
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculate moon phase for a given day
///
/// Returns a value from 0.0 to 1.0 where:
/// - 0.0 = new moon (dark)
/// - 0.5 = full moon (bright)
/// - 1.0 = new moon again (completes the cycle)
pub fn calculate_moon_phase(day: u32, period: u16) -> f32 {
    (day % period as u32) as f32 / period as f32
}

/// Calculate node longitude (precesses over time)
///
/// The lunar node is the point where the moon's orbit crosses the ecliptic.
/// This precesses (moves backward) over a long period.
///
/// Returns a value from 0.0 to 360.0 degrees.
pub fn calculate_node_longitude(day: u32, precession_period: u32) -> f32 {
    ((day % precession_period) as f32 / precession_period as f32) * 360.0
}

// ============================================================================
// MoonState
// ============================================================================

/// State of a moon at a given point in time
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MoonState {
    /// Phase: 0.0 = new, 0.5 = full, 1.0 = new again
    pub phase: f32,
    /// Longitude of ascending node (0.0-360.0) for eclipse calculations
    pub node_longitude: f32,
}

impl MoonState {
    /// Create a new moon state for a given day
    pub fn new(day: u32, period: u16, node_precession: u32) -> Self {
        Self {
            phase: calculate_moon_phase(day, period),
            node_longitude: calculate_node_longitude(day, node_precession),
        }
    }

    /// Is the moon full? (phase within 0.05 of 0.5)
    pub fn is_full(&self) -> bool {
        (self.phase - 0.5).abs() < 0.05
    }

    /// Is the moon new? (phase within 0.05 of 0.0 or > 0.95)
    pub fn is_new(&self) -> bool {
        self.phase < 0.05 || self.phase > 0.95
    }

    /// Light contribution at night (0.0-0.15)
    ///
    /// Full moon contributes 0.15 to light level, new moon contributes 0.0.
    /// The contribution smoothly varies based on the illuminated portion.
    pub fn light_contribution(&self) -> f32 {
        // Convert phase to illumination (0.0 at new, 1.0 at full)
        let illumination = if self.phase <= 0.5 {
            self.phase * 2.0
        } else {
            (1.0 - self.phase) * 2.0
        };
        illumination * 0.15
    }

    /// Is eclipse possible? (node aligned with sun within 15 degrees)
    ///
    /// An eclipse can only occur when the moon's orbital node is aligned
    /// with the sun (within about 15 degrees), which happens twice per
    /// eclipse season.
    pub fn eclipse_possible(&self, sun_longitude: f32) -> bool {
        let diff = (self.node_longitude - sun_longitude).abs();
        diff < 15.0 || diff > 345.0 // Within 15 degrees of node
    }
}

// ============================================================================
// Tests (TDD - written first)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solar_phase_from_hour() {
        assert_eq!(SolarPhase::from_hour(0), SolarPhase::DeepNight);
        assert_eq!(SolarPhase::from_hour(3), SolarPhase::DeepNight);
        assert_eq!(SolarPhase::from_hour(4), SolarPhase::PreDawn);
        assert_eq!(SolarPhase::from_hour(6), SolarPhase::Dawn);
        assert_eq!(SolarPhase::from_hour(8), SolarPhase::Morning);
        assert_eq!(SolarPhase::from_hour(11), SolarPhase::Midday);
        assert_eq!(SolarPhase::from_hour(14), SolarPhase::Afternoon);
        assert_eq!(SolarPhase::from_hour(17), SolarPhase::Dusk);
        assert_eq!(SolarPhase::from_hour(19), SolarPhase::Evening);
        assert_eq!(SolarPhase::from_hour(22), SolarPhase::Night);
    }

    #[test]
    fn test_season_from_day() {
        assert_eq!(Season::from_day_of_year(1), Season::Spring);
        assert_eq!(Season::from_day_of_year(90), Season::Spring);
        assert_eq!(Season::from_day_of_year(91), Season::Summer);
        assert_eq!(Season::from_day_of_year(180), Season::Summer);
        assert_eq!(Season::from_day_of_year(181), Season::Autumn);
        assert_eq!(Season::from_day_of_year(270), Season::Autumn);
        assert_eq!(Season::from_day_of_year(271), Season::Winter);
        assert_eq!(Season::from_day_of_year(360), Season::Winter);
    }

    #[test]
    fn test_solar_phase_to_time_period() {
        use crate::core::calendar::TimePeriod;

        assert_eq!(TimePeriod::from(SolarPhase::Dawn), TimePeriod::Morning);
        assert_eq!(TimePeriod::from(SolarPhase::Morning), TimePeriod::Morning);
        assert_eq!(TimePeriod::from(SolarPhase::Midday), TimePeriod::Afternoon);
        assert_eq!(TimePeriod::from(SolarPhase::Afternoon), TimePeriod::Afternoon);
        assert_eq!(TimePeriod::from(SolarPhase::Dusk), TimePeriod::Evening);
        assert_eq!(TimePeriod::from(SolarPhase::Evening), TimePeriod::Evening);
        assert_eq!(TimePeriod::from(SolarPhase::Night), TimePeriod::Night);
        assert_eq!(TimePeriod::from(SolarPhase::DeepNight), TimePeriod::Night);
        assert_eq!(TimePeriod::from(SolarPhase::PreDawn), TimePeriod::Night);
    }

    #[test]
    fn test_constants() {
        // Verify constants have expected values
        assert_eq!(YEAR_LENGTH, 360);
        assert_eq!(TICKS_PER_DAY, 1000);
        assert_eq!(ARGENT_PERIOD, 29);
        assert_eq!(SANGUINE_PERIOD, 83);
        assert_eq!(ARGENT_NODE_PRECESSION, 6570);
        assert_eq!(SANGUINE_NODE_PRECESSION, 11160);
        assert_eq!(CONJUNCTION_CYCLE, 2407);
    }

    #[test]
    fn test_celestial_event_rarity() {
        // Common events
        assert!(!CelestialEvent::FullArgent.is_rare());
        assert!(!CelestialEvent::NewArgent.is_rare());
        assert!(!CelestialEvent::FullSanguine.is_rare());
        assert!(!CelestialEvent::NewSanguine.is_rare());

        // Uncommon events
        assert!(!CelestialEvent::NearDoubleFull.is_rare());
        assert!(!CelestialEvent::NearDoubleNew.is_rare());
        assert!(!CelestialEvent::SilverEclipse.is_rare());
        assert!(!CelestialEvent::BloodEclipse.is_rare());

        // Rare events
        assert!(CelestialEvent::PerfectDoubleFull.is_rare());
        assert!(CelestialEvent::PerfectDoubleNew.is_rare());
        assert!(CelestialEvent::DoubleEclipse.is_rare());
    }

    #[test]
    fn test_solar_phase_base_light_level() {
        // Deep night should be darkest
        assert_eq!(SolarPhase::DeepNight.base_light_level(), 0.0);

        // Midday should be brightest
        assert_eq!(SolarPhase::Midday.base_light_level(), 1.0);

        // Dawn and dusk should be transitional
        assert_eq!(SolarPhase::Dawn.base_light_level(), 0.5);
        assert_eq!(SolarPhase::Dusk.base_light_level(), 0.5);

        // Morning and afternoon should be bright
        assert!(SolarPhase::Morning.base_light_level() > 0.7);
        assert!(SolarPhase::Afternoon.base_light_level() > 0.7);
    }

    #[test]
    fn test_celestial_event_common() {
        // Common events
        assert!(CelestialEvent::FullArgent.is_common());
        assert!(CelestialEvent::NewArgent.is_common());
        assert!(CelestialEvent::FullSanguine.is_common());
        assert!(CelestialEvent::NewSanguine.is_common());

        // Uncommon and rare events are not common
        assert!(!CelestialEvent::NearDoubleFull.is_common());
        assert!(!CelestialEvent::PerfectDoubleFull.is_common());
    }

    // ========================================================================
    // Task 2 Tests: MoonState and Phase Calculations (TDD - written first)
    // ========================================================================

    #[test]
    fn test_moon_phase_calculation() {
        // Day 0: new moon (phase = 0.0)
        let phase = calculate_moon_phase(0, ARGENT_PERIOD);
        assert!((phase - 0.0).abs() < 0.01);

        // Day 14-15: full moon for Argent (phase ~ 0.5)
        let phase = calculate_moon_phase(14, ARGENT_PERIOD);
        assert!((phase - 0.48).abs() < 0.05); // 14/29 ~ 0.48

        // Day 29: back to new (phase ~ 0.0 or 1.0)
        let phase = calculate_moon_phase(29, ARGENT_PERIOD);
        assert!(phase < 0.05 || phase > 0.95);

        // Sanguine: Day 41-42 should be full (phase ~ 0.5)
        let phase = calculate_moon_phase(41, SANGUINE_PERIOD);
        assert!((phase - 0.49).abs() < 0.05); // 41/83 ~ 0.49
    }

    #[test]
    fn test_node_longitude_calculation() {
        // Day 0: node at 0 degrees
        let longitude = calculate_node_longitude(0, ARGENT_NODE_PRECESSION);
        assert!((longitude - 0.0).abs() < 0.01);

        // Halfway through precession: node at 180 degrees
        let longitude = calculate_node_longitude(ARGENT_NODE_PRECESSION / 2, ARGENT_NODE_PRECESSION);
        assert!((longitude - 180.0).abs() < 0.1);

        // Full precession: back to 0 degrees
        let longitude = calculate_node_longitude(ARGENT_NODE_PRECESSION, ARGENT_NODE_PRECESSION);
        assert!(longitude < 0.1);
    }

    #[test]
    fn test_moon_state_new() {
        // Create a moon state at day 0
        let moon = MoonState::new(0, ARGENT_PERIOD, ARGENT_NODE_PRECESSION);

        // Should be new moon at day 0
        assert!(moon.is_new());
        assert!(!moon.is_full());

        // Create a moon state at full moon (around day 14-15 for Argent)
        let moon = MoonState::new(14, ARGENT_PERIOD, ARGENT_NODE_PRECESSION);
        assert!((moon.phase - 0.48).abs() < 0.05);
    }

    #[test]
    fn test_moon_state_is_full() {
        let full_moon = MoonState { phase: 0.5, node_longitude: 0.0 };
        assert!(full_moon.is_full());

        let new_moon = MoonState { phase: 0.0, node_longitude: 0.0 };
        assert!(!new_moon.is_full());
        assert!(new_moon.is_new());
    }

    #[test]
    fn test_moon_state_is_new() {
        // Test phase near 0.0
        let new_moon = MoonState { phase: 0.0, node_longitude: 0.0 };
        assert!(new_moon.is_new());

        // Test phase near 1.0
        let new_moon = MoonState { phase: 0.98, node_longitude: 0.0 };
        assert!(new_moon.is_new());

        // Test phase at boundary
        let almost_new = MoonState { phase: 0.04, node_longitude: 0.0 };
        assert!(almost_new.is_new());

        // Test phase just outside boundary
        let not_new = MoonState { phase: 0.1, node_longitude: 0.0 };
        assert!(!not_new.is_new());
    }

    #[test]
    fn test_lunar_light_contribution() {
        // Full moon provides maximum light
        let full_moon = MoonState { phase: 0.5, node_longitude: 0.0 };
        assert!((full_moon.light_contribution() - 0.15).abs() < 0.01);

        // New moon provides no light
        let new_moon = MoonState { phase: 0.0, node_longitude: 0.0 };
        assert!(new_moon.light_contribution() < 0.01);

        // Half moon (waxing) provides intermediate light
        let half_moon = MoonState { phase: 0.25, node_longitude: 0.0 };
        assert!(half_moon.light_contribution() > 0.0);
        assert!(half_moon.light_contribution() < 0.15);

        // Half moon (waning) provides same intermediate light
        let waning_half = MoonState { phase: 0.75, node_longitude: 0.0 };
        let waxing_half = MoonState { phase: 0.25, node_longitude: 0.0 };
        assert!((waning_half.light_contribution() - waxing_half.light_contribution()).abs() < 0.01);
    }

    #[test]
    fn test_eclipse_possible() {
        // Eclipse possible when node is aligned with sun
        let moon = MoonState { phase: 0.0, node_longitude: 0.0 };
        assert!(moon.eclipse_possible(0.0)); // Aligned
        assert!(moon.eclipse_possible(10.0)); // Within 15 degrees
        assert!(!moon.eclipse_possible(30.0)); // Too far

        // Test wraparound near 360 degrees
        let moon = MoonState { phase: 0.0, node_longitude: 355.0 };
        assert!(moon.eclipse_possible(5.0)); // Within 15 degrees across 0
        assert!(moon.eclipse_possible(350.0)); // Within 15 degrees on same side
    }
}
