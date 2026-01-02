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
}
