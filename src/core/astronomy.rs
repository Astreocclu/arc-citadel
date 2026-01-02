//! Astronomical system - sun, moons, seasons, and celestial events
//!
//! This module provides the core enums and constants for the astronomical system.

use ahash::AHashMap;
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
// AstronomicalState
// ============================================================================

/// Main astronomical state - tracks time, celestial bodies, and events
///
/// This struct replaces the old Calendar system and provides:
/// - Tick-based time tracking
/// - Day/year calculations with seasonal changes
/// - Dual moon phase tracking (Argent and Sanguine)
/// - Solar phase and light level calculations
/// - Celestial event detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstronomicalState {
    // Time tracking
    /// Current simulation tick
    pub tick: u64,
    /// Number of ticks per day
    pub ticks_per_day: u64,

    // Derived values (cached, updated when day changes)
    /// Total days since epoch (day 0)
    pub current_day: u32,
    /// Day within the current year (1-360)
    pub day_of_year: u16,
    /// Current year (starts at 1)
    pub year: i32,
    /// Current season based on day_of_year
    pub season: Season,
    /// Current solar phase based on time of day
    pub solar_phase: SolarPhase,
    /// Current light level (0.0-1.0) including lunar contribution
    pub light_level: f32,

    // Moon states
    /// Silver Moon (Argent) state
    pub argent: MoonState,
    /// Blood Moon (Sanguine) state
    pub sanguine: MoonState,

    // Events
    /// Currently active celestial events
    pub active_events: Vec<CelestialEvent>,
    /// Pre-computed event calendar (day -> events)
    pub event_calendar: AHashMap<u32, Vec<CelestialEvent>>,

    // Cache for expensive calculations
    last_updated_day: u32,
}

impl AstronomicalState {
    /// Create a new astronomical state with the given ticks per day
    pub fn new(ticks_per_day: u64) -> Self {
        let mut state = Self {
            tick: 0,
            ticks_per_day,
            current_day: 0,
            day_of_year: 1,
            year: 1,
            season: Season::Spring,
            solar_phase: SolarPhase::DeepNight,
            light_level: 0.0,
            argent: MoonState::default(),
            sanguine: MoonState::default(),
            active_events: Vec::new(),
            event_calendar: AHashMap::new(),
            last_updated_day: u32::MAX, // Force initial update
        };
        state.update_daily();
        state.update_light_level();
        state
    }

    /// Advance simulation by one tick
    pub fn advance_tick(&mut self) {
        self.tick += 1;

        let new_day = (self.tick / self.ticks_per_day) as u32;

        // Update solar phase (changes within day)
        let tick_in_day = self.tick % self.ticks_per_day;
        let hour = ((tick_in_day * 24) / self.ticks_per_day) as u32;
        self.solar_phase = SolarPhase::from_hour(hour);

        // Update light level
        self.update_light_level();

        // Update daily values if day changed
        if new_day != self.current_day {
            self.current_day = new_day;
            self.update_daily();
        }
    }

    /// Update values that change daily (called when day changes)
    pub fn update_daily(&mut self) {
        if self.last_updated_day == self.current_day {
            return;
        }
        self.last_updated_day = self.current_day;

        // Calculate day of year and year
        self.day_of_year = ((self.current_day % YEAR_LENGTH as u32) + 1) as u16;
        self.year = (self.current_day / YEAR_LENGTH as u32) as i32 + 1;

        // Update season
        self.season = Season::from_day_of_year(self.day_of_year);

        // Update moon states
        self.argent = MoonState::new(self.current_day, ARGENT_PERIOD, ARGENT_NODE_PRECESSION);
        self.sanguine = MoonState::new(self.current_day, SANGUINE_PERIOD, SANGUINE_NODE_PRECESSION);

        // Detect active events
        self.detect_events();
    }

    /// Update light level based on solar phase and moons
    pub fn update_light_level(&mut self) {
        let base = self.solar_phase.base_light_level();

        // Add lunar contribution only at night (when base light is low)
        let lunar = if base < 0.3 {
            self.argent.light_contribution() + self.sanguine.light_contribution()
        } else {
            0.0
        };

        self.light_level = (base + lunar).min(1.0);
    }

    /// Detect celestial events for current day
    pub fn detect_events(&mut self) {
        self.active_events.clear();

        // Check moon phases
        if self.argent.is_full() {
            self.active_events.push(CelestialEvent::FullArgent);
        }
        if self.argent.is_new() {
            self.active_events.push(CelestialEvent::NewArgent);
        }
        if self.sanguine.is_full() {
            self.active_events.push(CelestialEvent::FullSanguine);
        }
        if self.sanguine.is_new() {
            self.active_events.push(CelestialEvent::NewSanguine);
        }

        // Check double events
        if self.argent.is_full() && self.sanguine.is_full() {
            // Check if perfect (both exactly at 0.5)
            if (self.argent.phase - 0.5).abs() < 0.02 && (self.sanguine.phase - 0.5).abs() < 0.02 {
                self.active_events.push(CelestialEvent::PerfectDoubleFull);
            } else {
                self.active_events.push(CelestialEvent::NearDoubleFull);
            }
        }
        if self.argent.is_new() && self.sanguine.is_new() {
            if self.argent.phase < 0.02 && self.sanguine.phase < 0.02 {
                self.active_events.push(CelestialEvent::PerfectDoubleNew);
            } else {
                self.active_events.push(CelestialEvent::NearDoubleNew);
            }
        }

        // Check eclipses (simplified: eclipse when new moon + node aligned)
        let sun_longitude = (self.day_of_year as f32 / YEAR_LENGTH as f32) * 360.0;
        if self.argent.is_new() && self.argent.eclipse_possible(sun_longitude) {
            self.active_events.push(CelestialEvent::SilverEclipse);
        }
        if self.sanguine.is_new() && self.sanguine.eclipse_possible(sun_longitude) {
            self.active_events.push(CelestialEvent::BloodEclipse);
        }

        // Double eclipse
        if self.active_events.contains(&CelestialEvent::SilverEclipse)
            && self.active_events.contains(&CelestialEvent::BloodEclipse)
        {
            self.active_events.push(CelestialEvent::DoubleEclipse);
        }
    }

    /// Get current hour (0-23)
    pub fn hour(&self) -> u32 {
        let tick_in_day = self.tick % self.ticks_per_day;
        ((tick_in_day * 24) / self.ticks_per_day) as u32
    }

    /// Get TimePeriod for backward compatibility with expectations system
    pub fn time_period(&self) -> TimePeriod {
        TimePeriod::from(self.solar_phase)
    }

    /// Check if a specific event is active today
    pub fn has_event(&self, event: CelestialEvent) -> bool {
        self.active_events.contains(&event)
    }
}

impl Default for AstronomicalState {
    fn default() -> Self {
        Self::new(TICKS_PER_DAY)
    }
}

// ============================================================================
// FoundingModifiers
// ============================================================================

/// Modifiers applied to settlements based on founding conditions
///
/// When a settlement is founded, the day, season, and any active celestial
/// events determine its character. These modifiers affect starting resources,
/// population, cultural traits, and strategic tendencies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FoundingModifiers {
    // Season-based multipliers (default 1.0 for multiplicative, 0.0 for additive)
    /// Efficiency of stockpile storage (multiplicative, default 1.0)
    pub stockpile_efficiency: f32,
    /// Multiplier on initial population (multiplicative, default 1.0)
    pub initial_population_mult: f32,
    /// Bonus to growth rate (additive, default 0.0)
    pub growth_rate: f32,
    /// Weight toward defensive structures (additive, default 0.0)
    pub defensive_weight: f32,
    /// Bonus to trade infrastructure (additive, default 0.0)
    pub trade_infrastructure: f32,
    /// Bonus to harvest storage capacity (additive, default 0.0)
    pub harvest_storage: f32,
    /// Efficiency of resource usage (additive, default 0.0)
    pub resource_efficiency: f32,

    // Boolean traits
    /// Settlement has siege mentality (defensive, insular)
    pub siege_mentality: bool,
    /// Settlement has preparation trait (stores resources)
    pub preparation_trait: bool,
    /// Settlement is blessed (favorable founding)
    pub blessed: bool,
    /// Settlement has secrecy trait (hidden, underground)
    pub secrecy_trait: bool,

    // Astronomical event bonuses (additive)
    /// Preference for underground structures
    pub underground_preference: f32,
    /// Cultural tendency toward stealth and subterfuge
    pub stealth_culture: f32,
    /// Cultural tendency toward martial pursuits
    pub martial_culture: f32,
    /// Cultural tendency toward theocracy
    pub theocratic_tendency: f32,
    /// Baseline morale modifier (additive)
    pub morale_baseline: f32,
    /// Tendency to expand (additive, can be negative)
    pub expansion_tendency: f32,
    /// Bonus to fertility/birth rate
    pub fertility_bonus: f32,
    /// Weight of superstitious beliefs
    pub superstition_weight: f32,
    /// Affinity for supernatural/magical elements
    pub supernatural_affinity: f32,

    // Bias tags for hex generation and settlement features
    /// Tags that bias toward certain features (e.g., "defensive", "agricultural")
    pub bias_tags: Vec<String>,
    /// Tags that bias against certain features (e.g., "exposed", "fortified")
    pub bias_against: Vec<String>,

    // Narrative flavor
    /// Descriptive text about the founding conditions
    pub flavor_text: String,
}

impl FoundingModifiers {
    /// Calculate founding modifiers based on day, season, and active events
    ///
    /// This is the main entry point for calculating what modifiers apply
    /// to a settlement founded under the given conditions.
    ///
    /// # Arguments
    /// * `day_of_year` - The day of the year (1-360)
    /// * `season` - The current season
    /// * `events` - Active celestial events on this day
    pub fn calculate(day_of_year: u16, season: Season, events: &[CelestialEvent]) -> Self {
        let mut modifiers = Self {
            stockpile_efficiency: 1.0,
            initial_population_mult: 1.0,
            ..Default::default()
        };

        // Apply season modifiers
        modifiers.apply_season(day_of_year, season);

        // Apply event modifiers
        for event in events {
            modifiers.apply_event(*event);
        }

        // Generate flavor text
        modifiers.generate_flavor(season, events);

        modifiers
    }

    /// Apply season-based modifiers
    ///
    /// Each season (and sub-period within seasons) grants different bonuses
    /// and penalties reflecting the challenges and opportunities of founding
    /// a settlement in that time of year.
    fn apply_season(&mut self, day_of_year: u16, season: Season) {
        match season {
            Season::Spring => {
                if day_of_year <= 45 {
                    // Early spring: renewal, optimism, agricultural focus
                    self.growth_rate += 0.1;
                    self.morale_baseline += 0.1;
                    self.bias_tags.push("agricultural".to_string());
                    self.bias_tags.push("optimistic".to_string());
                } else {
                    // Late spring: expansion, fertility, growth
                    self.fertility_bonus += 0.15;
                    self.expansion_tendency += 0.2;
                    self.bias_tags.push("expanding".to_string());
                }
            }
            Season::Summer => {
                // Summer: abundance, trade, openness (but less defensive)
                self.initial_population_mult += 0.15;
                self.trade_infrastructure += 0.2;
                self.defensive_weight -= 0.1;
                self.bias_tags.push("commercial".to_string());
                self.bias_tags.push("open".to_string());
                self.bias_against.push("fortified".to_string());
            }
            Season::Autumn => {
                // Autumn: harvest, preparation, balance
                self.harvest_storage += 0.2;
                self.preparation_trait = true;
                self.bias_tags.push("prepared".to_string());
                self.bias_tags.push("balanced".to_string());
            }
            Season::Winter => {
                if day_of_year >= 300 {
                    // Deep winter: survival mode, defensive, industrial
                    self.stockpile_efficiency += 0.15;
                    self.initial_population_mult -= 0.2;
                    self.defensive_weight += 0.3;
                    self.siege_mentality = true;
                    self.bias_tags.push("defensive".to_string());
                    self.bias_tags.push("industrial".to_string());
                    self.bias_against.push("exposed".to_string());
                } else {
                    // Early winter: caution, resource efficiency
                    self.resource_efficiency += 0.1;
                    self.morale_baseline -= 0.15; // Cautious/pessimistic baseline
                    self.bias_tags.push("cautious".to_string());
                }
            }
        }
    }

    /// Apply celestial event modifiers
    ///
    /// Rare celestial events at founding leave lasting impressions on
    /// settlement culture and capabilities.
    fn apply_event(&mut self, event: CelestialEvent) {
        match event {
            CelestialEvent::PerfectDoubleFull | CelestialEvent::NearDoubleFull => {
                // Double full moon: blessing, prosperity, expansion
                self.morale_baseline += 0.1;
                self.expansion_tendency += 0.25;
                self.blessed = true;
                self.fertility_bonus += 0.1;
                self.bias_tags.push("blessed".to_string());
                self.bias_tags.push("prosperous".to_string());
            }
            CelestialEvent::PerfectDoubleNew | CelestialEvent::NearDoubleNew => {
                // Double new moon (The Dark): underground, secretive
                self.underground_preference += 0.3;
                self.stealth_culture += 0.2;
                self.superstition_weight += 0.2;
                self.secrecy_trait = true;
                self.bias_tags.push("underground".to_string());
                self.bias_tags.push("secretive".to_string());
                self.bias_against.push("surface".to_string());
            }
            CelestialEvent::SilverEclipse => {
                // Silver eclipse: theocratic, superstitious
                self.theocratic_tendency += 0.15;
                self.superstition_weight += 0.3;
                self.bias_tags.push("theocratic".to_string());
                self.bias_tags.push("silver".to_string());
            }
            CelestialEvent::BloodEclipse => {
                // Blood eclipse: martial, warlike
                self.martial_culture += 0.2;
                self.superstition_weight += 0.25;
                self.bias_tags.push("martial".to_string());
                self.bias_tags.push("blood".to_string());
            }
            CelestialEvent::DoubleEclipse => {
                // Double eclipse: mystical, isolated
                self.supernatural_affinity += 0.3;
                self.superstition_weight += 0.4;
                self.expansion_tendency -= 0.2; // Isolation tendency
                self.bias_tags.push("mystical".to_string());
                self.bias_tags.push("isolated".to_string());
            }
            // Common events don't significantly affect founding
            _ => {}
        }
    }

    /// Generate narrative flavor text for the founding
    ///
    /// Creates a descriptive string that can be used in narratives or
    /// settlement descriptions.
    fn generate_flavor(&mut self, season: Season, events: &[CelestialEvent]) {
        let season_desc = match season {
            Season::Spring => "in the season of renewal",
            Season::Summer => "under abundant summer skies",
            Season::Autumn => "as leaves fell and stores filled",
            Season::Winter => "in the harshest season",
        };

        let event_desc = if events.iter().any(|e| matches!(e, CelestialEvent::PerfectDoubleFull)) {
            "under the radiant double full moons"
        } else if events.iter().any(|e| matches!(e, CelestialEvent::PerfectDoubleNew)) {
            "under lightless skies"
        } else if events.iter().any(|e| matches!(e, CelestialEvent::DoubleEclipse)) {
            "as both moons devoured the sun"
        } else if events.iter().any(|e| matches!(e, CelestialEvent::BloodEclipse)) {
            "beneath the blood-darkened sun"
        } else if events.iter().any(|e| matches!(e, CelestialEvent::SilverEclipse)) {
            "as silver shadows crossed the sun"
        } else {
            ""
        };

        self.flavor_text = if event_desc.is_empty() {
            format!("Founded {}", season_desc)
        } else {
            format!("Founded {} {}", season_desc, event_desc)
        };
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

    // ========================================================================
    // Task 3 Tests: AstronomicalState Core (TDD - written first)
    // ========================================================================

    #[test]
    fn test_astronomical_state_new() {
        let state = AstronomicalState::new(TICKS_PER_DAY);

        assert_eq!(state.tick, 0);
        assert_eq!(state.current_day, 0);
        assert_eq!(state.day_of_year, 1);
        assert_eq!(state.year, 1);
        assert_eq!(state.season, Season::Spring);
    }

    #[test]
    fn test_astronomical_state_advance_day() {
        let mut state = AstronomicalState::new(TICKS_PER_DAY);

        // Advance one full day
        for _ in 0..TICKS_PER_DAY {
            state.advance_tick();
        }

        assert_eq!(state.current_day, 1);
        assert_eq!(state.day_of_year, 2);
    }

    #[test]
    fn test_astronomical_state_year_rollover() {
        let mut state = AstronomicalState::new(TICKS_PER_DAY);

        // Advance to end of year
        for _ in 0..(YEAR_LENGTH as u64 * TICKS_PER_DAY) {
            state.advance_tick();
        }

        assert_eq!(state.year, 2);
        assert_eq!(state.day_of_year, 1);
        assert_eq!(state.season, Season::Spring);
    }

    #[test]
    fn test_solar_phase_from_tick() {
        let mut state = AstronomicalState::new(TICKS_PER_DAY);

        // At tick 0, hour = 0, phase = DeepNight
        assert_eq!(state.solar_phase, SolarPhase::DeepNight);

        // Advance to midday (tick 500 = hour 12)
        for _ in 0..500 {
            state.advance_tick();
        }
        assert_eq!(state.solar_phase, SolarPhase::Midday);
    }

    #[test]
    fn test_astronomical_state_hour() {
        let mut state = AstronomicalState::new(TICKS_PER_DAY);

        // At tick 0, hour = 0
        assert_eq!(state.hour(), 0);

        // Advance to tick 500 (hour 12)
        for _ in 0..500 {
            state.advance_tick();
        }
        assert_eq!(state.hour(), 12);

        // Advance to tick 750 (hour 18)
        for _ in 0..250 {
            state.advance_tick();
        }
        assert_eq!(state.hour(), 18);
    }

    #[test]
    fn test_astronomical_state_time_period() {
        let mut state = AstronomicalState::new(TICKS_PER_DAY);

        // At midnight, should be Night
        assert_eq!(state.time_period(), TimePeriod::Night);

        // Advance to morning (tick 333 = hour 8)
        for _ in 0..333 {
            state.advance_tick();
        }
        assert_eq!(state.time_period(), TimePeriod::Morning);

        // Advance to midday (tick 500 = hour 12)
        for _ in 0..167 {
            state.advance_tick();
        }
        assert_eq!(state.time_period(), TimePeriod::Afternoon);
    }

    #[test]
    fn test_astronomical_state_has_event() {
        let mut state = AstronomicalState::new(TICKS_PER_DAY);

        // At day 0, Argent should be new (phase = 0)
        assert!(state.has_event(CelestialEvent::NewArgent));
        assert!(!state.has_event(CelestialEvent::FullArgent));

        // Advance to day 14-15 for Argent full moon
        for _ in 0..(14 * TICKS_PER_DAY) {
            state.advance_tick();
        }
        assert!(state.has_event(CelestialEvent::FullArgent));
    }

    #[test]
    fn test_astronomical_state_light_level() {
        let mut state = AstronomicalState::new(TICKS_PER_DAY);

        // At midnight (DeepNight), light should be low
        assert!(state.light_level < 0.3);

        // Advance to midday (tick 500)
        for _ in 0..500 {
            state.advance_tick();
        }
        // At midday, light should be high
        assert!(state.light_level > 0.9);
    }

    #[test]
    fn test_astronomical_state_moon_states() {
        let state = AstronomicalState::new(TICKS_PER_DAY);

        // At day 0, both moons should be in their initial phase
        assert!(state.argent.is_new());  // Argent starts at new moon
        assert!(state.sanguine.is_new()); // Sanguine starts at new moon
    }

    #[test]
    fn test_astronomical_state_default() {
        let state = AstronomicalState::default();

        // Default should use TICKS_PER_DAY
        assert_eq!(state.ticks_per_day, TICKS_PER_DAY);
        assert_eq!(state.tick, 0);
        assert_eq!(state.year, 1);
    }

    #[test]
    fn test_astronomical_state_season_changes() {
        let mut state = AstronomicalState::new(TICKS_PER_DAY);

        // Start in Spring
        assert_eq!(state.season, Season::Spring);

        // Advance to day 91 (Summer)
        for _ in 0..(91 * TICKS_PER_DAY) {
            state.advance_tick();
        }
        assert_eq!(state.season, Season::Summer);

        // Advance to day 181 (Autumn)
        for _ in 0..(90 * TICKS_PER_DAY) {
            state.advance_tick();
        }
        assert_eq!(state.season, Season::Autumn);

        // Advance to day 271 (Winter)
        for _ in 0..(90 * TICKS_PER_DAY) {
            state.advance_tick();
        }
        assert_eq!(state.season, Season::Winter);
    }

    // ========================================================================
    // Task 4 Tests: FoundingModifiers (TDD - written first)
    // ========================================================================

    #[test]
    fn test_founding_modifiers_deep_winter() {
        let modifiers = FoundingModifiers::calculate(342, Season::Winter, &[]);

        assert!(modifiers.stockpile_efficiency > 1.0);
        assert!(modifiers.initial_population_mult < 1.0);
        assert!(modifiers.siege_mentality);
        assert!(modifiers.bias_tags.contains(&"defensive".to_string()));
    }

    #[test]
    fn test_founding_modifiers_summer() {
        let modifiers = FoundingModifiers::calculate(150, Season::Summer, &[]);

        assert!(modifiers.initial_population_mult > 1.0);
        assert!(modifiers.trade_infrastructure > 0.0);
        assert!(!modifiers.siege_mentality);
    }

    #[test]
    fn test_founding_modifiers_with_event() {
        let events = vec![CelestialEvent::PerfectDoubleFull];
        let modifiers = FoundingModifiers::calculate(150, Season::Summer, &events);

        assert!(modifiers.morale_baseline > 0.0);
        assert!(modifiers.blessed);
        assert!(modifiers.bias_tags.contains(&"blessed".to_string()));
    }

    #[test]
    fn test_founding_modifiers_the_dark() {
        let events = vec![CelestialEvent::PerfectDoubleNew];
        let modifiers = FoundingModifiers::calculate(342, Season::Winter, &events);

        // Combined winter + dark modifiers
        assert!(modifiers.underground_preference > 0.0);
        assert!(modifiers.stealth_culture > 0.0);
        assert!(modifiers.siege_mentality);
        assert!(modifiers.secrecy_trait);
    }

    #[test]
    fn test_founding_modifiers_early_spring() {
        let modifiers = FoundingModifiers::calculate(30, Season::Spring, &[]);

        // Early spring (days 1-45)
        assert!(modifiers.growth_rate > 0.0);
        assert!(modifiers.morale_baseline > 0.0);
        assert!(modifiers.bias_tags.contains(&"agricultural".to_string()));
        assert!(modifiers.bias_tags.contains(&"optimistic".to_string()));
    }

    #[test]
    fn test_founding_modifiers_late_spring() {
        let modifiers = FoundingModifiers::calculate(60, Season::Spring, &[]);

        // Late spring (days 46-90)
        assert!(modifiers.fertility_bonus > 0.0);
        assert!(modifiers.expansion_tendency > 0.0);
        assert!(modifiers.bias_tags.contains(&"expanding".to_string()));
    }

    #[test]
    fn test_founding_modifiers_autumn() {
        let modifiers = FoundingModifiers::calculate(200, Season::Autumn, &[]);

        assert!(modifiers.harvest_storage > 0.0);
        assert!(modifiers.preparation_trait);
        assert!(modifiers.bias_tags.contains(&"prepared".to_string()));
        assert!(modifiers.bias_tags.contains(&"balanced".to_string()));
    }

    #[test]
    fn test_founding_modifiers_early_winter() {
        let modifiers = FoundingModifiers::calculate(280, Season::Winter, &[]);

        // Early winter (days 271-299)
        assert!(modifiers.resource_efficiency > 0.0);
        assert!(modifiers.morale_baseline < 0.0); // Caution/pessimism
        assert!(modifiers.bias_tags.contains(&"cautious".to_string()));
        assert!(!modifiers.siege_mentality); // Not deep winter
    }

    #[test]
    fn test_founding_modifiers_silver_eclipse() {
        let events = vec![CelestialEvent::SilverEclipse];
        let modifiers = FoundingModifiers::calculate(150, Season::Summer, &events);

        assert!(modifiers.theocratic_tendency > 0.0);
        assert!(modifiers.superstition_weight > 0.0);
        assert!(modifiers.bias_tags.contains(&"theocratic".to_string()));
        assert!(modifiers.bias_tags.contains(&"silver".to_string()));
    }

    #[test]
    fn test_founding_modifiers_blood_eclipse() {
        let events = vec![CelestialEvent::BloodEclipse];
        let modifiers = FoundingModifiers::calculate(150, Season::Summer, &events);

        assert!(modifiers.martial_culture > 0.0);
        assert!(modifiers.superstition_weight > 0.0);
        assert!(modifiers.bias_tags.contains(&"martial".to_string()));
        assert!(modifiers.bias_tags.contains(&"blood".to_string()));
    }

    #[test]
    fn test_founding_modifiers_double_eclipse() {
        let events = vec![CelestialEvent::DoubleEclipse];
        let modifiers = FoundingModifiers::calculate(150, Season::Summer, &events);

        assert!(modifiers.supernatural_affinity > 0.0);
        assert!(modifiers.superstition_weight > 0.0);
        assert!(modifiers.expansion_tendency < 0.0); // Isolation
        assert!(modifiers.bias_tags.contains(&"mystical".to_string()));
        assert!(modifiers.bias_tags.contains(&"isolated".to_string()));
    }

    #[test]
    fn test_founding_modifiers_flavor_text_summer() {
        let modifiers = FoundingModifiers::calculate(150, Season::Summer, &[]);

        assert!(modifiers.flavor_text.contains("summer"));
    }

    #[test]
    fn test_founding_modifiers_flavor_text_with_event() {
        let events = vec![CelestialEvent::PerfectDoubleFull];
        let modifiers = FoundingModifiers::calculate(150, Season::Summer, &events);

        assert!(modifiers.flavor_text.contains("double full moons"));
    }

    #[test]
    fn test_founding_modifiers_default_values() {
        // Create a default instance to verify baseline values
        let modifiers = FoundingModifiers::default();

        assert_eq!(modifiers.stockpile_efficiency, 0.0);
        assert_eq!(modifiers.initial_population_mult, 0.0);
        assert!(!modifiers.siege_mentality);
        assert!(!modifiers.blessed);
        assert!(modifiers.bias_tags.is_empty());
    }

    #[test]
    fn test_founding_modifiers_summer_bias_against() {
        let modifiers = FoundingModifiers::calculate(150, Season::Summer, &[]);

        // Summer settlements bias against fortified structures
        assert!(modifiers.bias_against.contains(&"fortified".to_string()));
    }
}
