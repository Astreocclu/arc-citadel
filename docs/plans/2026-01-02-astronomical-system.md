# Astronomical System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the existing Calendar system with a full astronomical simulation including dual moons, eclipses, celestial events, and founding conditions.

**Architecture:** Single module `src/core/astronomy.rs` containing AstronomicalState (replaces Calendar), MoonState, FoundingModifiers, and all enums. Backward compatibility via `From<SolarPhase> for TimePeriod`.

**Tech Stack:** Rust, serde for serialization, ahash for HashMap

---

## Task 1: Create Core Enums and Constants

**Files:**
- Create: `src/core/astronomy.rs`
- Modify: `src/core/mod.rs`

**Step 1: Write the failing test**

```rust
// In src/core/astronomy.rs
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
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib core::astronomy`
Expected: FAIL (module doesn't exist)

**Step 3: Write implementation**

```rust
// src/core/astronomy.rs
//! Astronomical system - sun, moons, seasons, and celestial events

use serde::{Deserialize, Serialize};
use crate::core::calendar::TimePeriod;

// ============================================================================
// Constants
// ============================================================================

pub const YEAR_LENGTH: u16 = 360;
pub const TICKS_PER_DAY: u64 = 1000;
pub const ARGENT_PERIOD: u16 = 29;      // Silver Moon orbital period (days)
pub const SANGUINE_PERIOD: u16 = 83;    // Blood Moon orbital period (days)
pub const ARGENT_NODE_PRECESSION: u32 = 6570;   // ~18 years in days
pub const SANGUINE_NODE_PRECESSION: u32 = 11160; // ~31 years in days
pub const CONJUNCTION_CYCLE: u32 = 2407; // LCM(29, 83) - both moons align

// ============================================================================
// Enums
// ============================================================================

/// Solar phase - 9 phases of the day
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SolarPhase {
    DeepNight,  // 00:00-04:00
    PreDawn,    // 04:00-06:00
    Dawn,       // 06:00-08:00
    Morning,    // 08:00-11:00
    Midday,     // 11:00-14:00
    Afternoon,  // 14:00-17:00
    Dusk,       // 17:00-19:00
    Evening,    // 19:00-22:00
    Night,      // 22:00-00:00
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
            _ => SolarPhase::Night,
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

/// Convert SolarPhase to TimePeriod for backward compatibility
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

/// Season of the year
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Season {
    #[default]
    Spring,  // Days 1-90
    Summer,  // Days 91-180
    Autumn,  // Days 181-270
    Winter,  // Days 271-360
}

impl Season {
    /// Get season from day of year (1-360)
    pub fn from_day_of_year(day: u16) -> Self {
        match day {
            1..=90 => Season::Spring,
            91..=180 => Season::Summer,
            181..=270 => Season::Autumn,
            _ => Season::Winter,
        }
    }
}

/// Celestial events that can occur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CelestialEvent {
    // Common (every cycle)
    FullArgent,
    NewArgent,
    FullSanguine,
    NewSanguine,

    // Uncommon (every few months)
    NearDoubleFull,   // Both moons within 2 days of full
    NearDoubleNew,    // Both moons within 2 days of new
    SilverEclipse,    // Argent eclipses sun
    BloodEclipse,     // Sanguine eclipses sun

    // Rare (years apart)
    PerfectDoubleFull,  // Both moons exactly full (every 2407 days)
    PerfectDoubleNew,   // Both moons exactly new
    DoubleEclipse,      // Both moons eclipse same day
}

impl CelestialEvent {
    /// Is this a rare event?
    pub fn is_rare(&self) -> bool {
        matches!(self,
            CelestialEvent::PerfectDoubleFull |
            CelestialEvent::PerfectDoubleNew |
            CelestialEvent::DoubleEclipse
        )
    }
}
```

**Step 4: Add module export**

```rust
// In src/core/mod.rs, add:
pub mod astronomy;
```

**Step 5: Run tests**

Run: `cargo test --lib core::astronomy`
Expected: PASS

**Step 6: Commit**

```bash
git add src/core/astronomy.rs src/core/mod.rs
git commit -m "feat: add astronomical system enums and constants"
```

---

## Task 2: Implement MoonState and Phase Calculations

**Files:**
- Modify: `src/core/astronomy.rs`

**Step 1: Write the failing test**

```rust
// Add to astronomy.rs tests
#[test]
fn test_moon_phase_calculation() {
    // Day 0: new moon (phase = 0.0)
    let phase = calculate_moon_phase(0, ARGENT_PERIOD);
    assert!((phase - 0.0).abs() < 0.01);

    // Day 14-15: full moon for Argent (phase ≈ 0.5)
    let phase = calculate_moon_phase(14, ARGENT_PERIOD);
    assert!((phase - 0.48).abs() < 0.05); // 14/29 ≈ 0.48

    // Day 29: back to new (phase ≈ 0.0 or 1.0)
    let phase = calculate_moon_phase(29, ARGENT_PERIOD);
    assert!(phase < 0.05 || phase > 0.95);

    // Sanguine: Day 41-42 should be full (phase ≈ 0.5)
    let phase = calculate_moon_phase(41, SANGUINE_PERIOD);
    assert!((phase - 0.49).abs() < 0.05); // 41/83 ≈ 0.49
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
fn test_lunar_light_contribution() {
    let full_moon = MoonState { phase: 0.5, node_longitude: 0.0 };
    assert!((full_moon.light_contribution() - 0.15).abs() < 0.01);

    let new_moon = MoonState { phase: 0.0, node_longitude: 0.0 };
    assert!(new_moon.light_contribution() < 0.01);

    let half_moon = MoonState { phase: 0.25, node_longitude: 0.0 };
    assert!(half_moon.light_contribution() > 0.0);
    assert!(half_moon.light_contribution() < 0.15);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib core::astronomy::tests::test_moon`
Expected: FAIL

**Step 3: Write implementation**

```rust
// Add to src/core/astronomy.rs

/// State of a moon
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

    /// Is the moon new? (phase within 0.05 of 0.0 or 1.0)
    pub fn is_new(&self) -> bool {
        self.phase < 0.05 || self.phase > 0.95
    }

    /// Light contribution at night (0.0-0.15)
    /// Full moon = 0.15, new moon = 0.0
    pub fn light_contribution(&self) -> f32 {
        // Convert phase to illumination (0.0 at new, 1.0 at full)
        let illumination = if self.phase <= 0.5 {
            self.phase * 2.0
        } else {
            (1.0 - self.phase) * 2.0
        };
        illumination * 0.15
    }

    /// Is eclipse possible? (node aligned with sun)
    pub fn eclipse_possible(&self, sun_longitude: f32) -> bool {
        let diff = (self.node_longitude - sun_longitude).abs();
        diff < 15.0 || diff > 345.0 // Within 15 degrees of node
    }
}

/// Calculate moon phase for a given day
pub fn calculate_moon_phase(day: u32, period: u16) -> f32 {
    (day % period as u32) as f32 / period as f32
}

/// Calculate node longitude (precesses over time)
pub fn calculate_node_longitude(day: u32, precession_period: u32) -> f32 {
    ((day % precession_period) as f32 / precession_period as f32) * 360.0
}
```

**Step 4: Run tests**

Run: `cargo test --lib core::astronomy::tests::test_moon`
Expected: PASS

**Step 5: Commit**

```bash
git add src/core/astronomy.rs
git commit -m "feat: add MoonState and phase calculations"
```

---

## Task 3: Implement AstronomicalState Core

**Files:**
- Modify: `src/core/astronomy.rs`

**Step 1: Write the failing test**

```rust
// Add to astronomy.rs tests
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib core::astronomy::tests::test_astronomical_state`
Expected: FAIL

**Step 3: Write implementation**

```rust
// Add to src/core/astronomy.rs
use ahash::AHashMap;

/// Main astronomical state - replaces Calendar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstronomicalState {
    // Time tracking
    pub tick: u64,
    pub ticks_per_day: u64,

    // Derived values (cached, updated when day changes)
    pub current_day: u32,       // Total days since epoch
    pub day_of_year: u16,       // 1-360
    pub year: i32,
    pub season: Season,
    pub solar_phase: SolarPhase,
    pub light_level: f32,

    // Moon states
    pub argent: MoonState,
    pub sanguine: MoonState,

    // Events
    pub active_events: Vec<CelestialEvent>,
    pub event_calendar: AHashMap<u32, Vec<CelestialEvent>>,

    // Cache for expensive calculations
    last_updated_day: u32,
}

impl AstronomicalState {
    /// Create new astronomical state
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

    /// Update values that change daily
    fn update_daily(&mut self) {
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
    fn update_light_level(&mut self) {
        let base = self.solar_phase.base_light_level();

        // Add lunar contribution only at night
        let lunar = if base < 0.3 {
            self.argent.light_contribution() + self.sanguine.light_contribution()
        } else {
            0.0
        };

        self.light_level = (base + lunar).min(1.0);
    }

    /// Detect celestial events for current day
    fn detect_events(&mut self) {
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
            && self.active_events.contains(&CelestialEvent::BloodEclipse) {
            self.active_events.push(CelestialEvent::DoubleEclipse);
        }
    }

    /// Get current hour (0-23)
    pub fn hour(&self) -> u32 {
        let tick_in_day = self.tick % self.ticks_per_day;
        ((tick_in_day * 24) / self.ticks_per_day) as u32
    }

    /// Get TimePeriod for backward compatibility
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
```

**Step 4: Run tests**

Run: `cargo test --lib core::astronomy`
Expected: PASS

**Step 5: Commit**

```bash
git add src/core/astronomy.rs
git commit -m "feat: implement AstronomicalState core"
```

---

## Task 4: Implement FoundingModifiers

**Files:**
- Modify: `src/core/astronomy.rs`

**Step 1: Write the failing test**

```rust
// Add to astronomy.rs tests
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib core::astronomy::tests::test_founding`
Expected: FAIL

**Step 3: Write implementation**

```rust
// Add to src/core/astronomy.rs

/// Modifiers applied to settlements based on founding conditions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FoundingModifiers {
    // Season-based multipliers
    pub stockpile_efficiency: f32,      // Default 1.0
    pub initial_population_mult: f32,   // Default 1.0
    pub growth_rate: f32,               // Default 0.0 (additive)
    pub defensive_weight: f32,          // Default 0.0 (additive)
    pub trade_infrastructure: f32,      // Default 0.0 (additive)
    pub harvest_storage: f32,           // Default 0.0 (additive)
    pub resource_efficiency: f32,       // Default 0.0 (additive)

    // Boolean traits
    pub siege_mentality: bool,
    pub preparation_trait: bool,
    pub blessed: bool,
    pub secrecy_trait: bool,

    // Astronomical event bonuses
    pub underground_preference: f32,
    pub stealth_culture: f32,
    pub martial_culture: f32,
    pub theocratic_tendency: f32,
    pub morale_baseline: f32,
    pub expansion_tendency: f32,
    pub fertility_bonus: f32,
    pub superstition_weight: f32,
    pub supernatural_affinity: f32,

    // Bias tags for hex generation
    pub bias_tags: Vec<String>,
    pub bias_against: Vec<String>,

    // Narrative flavor
    pub flavor_text: String,
}

impl FoundingModifiers {
    /// Calculate founding modifiers based on day, season, and active events
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

    fn apply_season(&mut self, day_of_year: u16, season: Season) {
        match season {
            Season::Spring => {
                if day_of_year <= 45 {
                    // Early spring
                    self.growth_rate += 0.1;
                    self.morale_baseline += 0.1;
                    self.bias_tags.push("agricultural".to_string());
                    self.bias_tags.push("optimistic".to_string());
                } else {
                    // Late spring
                    self.fertility_bonus += 0.15;
                    self.expansion_tendency += 0.2;
                    self.bias_tags.push("expanding".to_string());
                }
            }
            Season::Summer => {
                self.initial_population_mult += 0.15;
                self.trade_infrastructure += 0.2;
                self.defensive_weight -= 0.1;
                self.bias_tags.push("commercial".to_string());
                self.bias_tags.push("open".to_string());
                self.bias_against.push("fortified".to_string());
            }
            Season::Autumn => {
                self.harvest_storage += 0.2;
                self.preparation_trait = true;
                self.bias_tags.push("prepared".to_string());
                self.bias_tags.push("balanced".to_string());
            }
            Season::Winter => {
                if day_of_year >= 300 {
                    // Deep winter
                    self.stockpile_efficiency += 0.15;
                    self.initial_population_mult -= 0.2;
                    self.defensive_weight += 0.3;
                    self.siege_mentality = true;
                    self.bias_tags.push("defensive".to_string());
                    self.bias_tags.push("industrial".to_string());
                    self.bias_against.push("exposed".to_string());
                } else {
                    // Early winter
                    self.resource_efficiency += 0.1;
                    self.morale_baseline -= 0.15; // Caution baseline
                    self.bias_tags.push("cautious".to_string());
                }
            }
        }
    }

    fn apply_event(&mut self, event: CelestialEvent) {
        match event {
            CelestialEvent::PerfectDoubleFull | CelestialEvent::NearDoubleFull => {
                self.morale_baseline += 0.1;
                self.expansion_tendency += 0.25;
                self.blessed = true;
                self.fertility_bonus += 0.1;
                self.bias_tags.push("blessed".to_string());
                self.bias_tags.push("prosperous".to_string());
            }
            CelestialEvent::PerfectDoubleNew | CelestialEvent::NearDoubleNew => {
                self.underground_preference += 0.3;
                self.stealth_culture += 0.2;
                self.superstition_weight += 0.2;
                self.secrecy_trait = true;
                self.bias_tags.push("underground".to_string());
                self.bias_tags.push("secretive".to_string());
                self.bias_against.push("surface".to_string());
            }
            CelestialEvent::SilverEclipse => {
                self.theocratic_tendency += 0.15;
                self.superstition_weight += 0.3;
                self.bias_tags.push("theocratic".to_string());
                self.bias_tags.push("silver".to_string());
            }
            CelestialEvent::BloodEclipse => {
                self.martial_culture += 0.2;
                self.superstition_weight += 0.25;
                self.bias_tags.push("martial".to_string());
                self.bias_tags.push("blood".to_string());
            }
            CelestialEvent::DoubleEclipse => {
                self.supernatural_affinity += 0.3;
                self.superstition_weight += 0.4;
                self.expansion_tendency -= 0.2; // Isolation
                self.bias_tags.push("mystical".to_string());
                self.bias_tags.push("isolated".to_string());
            }
            _ => {} // Common events don't affect founding
        }
    }

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
```

**Step 4: Run tests**

Run: `cargo test --lib core::astronomy::tests::test_founding`
Expected: PASS

**Step 5: Commit**

```bash
git add src/core/astronomy.rs
git commit -m "feat: implement FoundingModifiers for settlement creation"
```

---

## Task 5: Integrate with World and Tick System

**Files:**
- Modify: `src/ecs/world.rs`
- Modify: `src/simulation/tick.rs`

**Step 1: Write the failing test**

```rust
// Add to src/ecs/world.rs tests
#[test]
fn test_world_has_astronomy() {
    let world = World::new();

    assert_eq!(world.astronomy.year, 1);
    assert_eq!(world.astronomy.day_of_year, 1);
    assert_eq!(world.astronomy.season, Season::Spring);
}

// Add to src/simulation/tick.rs tests
#[test]
fn test_tick_advances_astronomy() {
    let mut world = World::new();

    let initial_tick = world.astronomy.tick;

    // Run one simulation tick
    run_simulation_tick(&mut world);

    assert!(world.astronomy.tick > initial_tick);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib ecs::world::tests::test_world_has_astronomy`
Expected: FAIL

**Step 3: Modify World struct**

```rust
// In src/ecs/world.rs

// Change imports
use crate::core::astronomy::AstronomicalState;
// Remove: use crate::core::calendar::Calendar;

// In World struct, replace:
// pub calendar: Calendar,
// With:
pub astronomy: AstronomicalState,

// In World::new(), replace:
// calendar: Calendar::default(),
// With:
astronomy: AstronomicalState::default(),
```

**Step 4: Update tick.rs**

```rust
// In src/simulation/tick.rs

// In run_simulation_tick(), add at the beginning:
world.astronomy.advance_tick();

// Remove any references to world.calendar
// The tick() call at the end can be kept for other purposes or removed
```

**Step 5: Fix compilation errors**

Search for all uses of `world.calendar` and `TimePeriod` in the codebase and update:

```rust
// Where you had:
world.calendar.time_period()
// Use:
world.astronomy.time_period()

// Where you had:
world.calendar.tick
// Use:
world.astronomy.tick

// TimePeriod imports stay the same (it's still in calendar.rs)
// OR import from astronomy if we move it there
```

**Step 6: Run tests**

Run: `cargo test`
Expected: PASS (all tests)

**Step 7: Commit**

```bash
git add src/ecs/world.rs src/simulation/tick.rs
git commit -m "feat: integrate AstronomicalState with World and tick system"
```

---

## Task 6: Update Expectations System Compatibility

**Files:**
- Modify: `src/entity/social/expectations.rs` (if needed)
- Verify: `src/simulation/expectation_formation.rs`

**Step 1: Verify TimePeriod compatibility**

The expectations system uses `TimePeriod` in `PatternType::LocationDuring`. We implemented `From<SolarPhase> for TimePeriod`, so this should work.

**Step 2: Write compatibility test**

```rust
// Add to expectations.rs tests or a new integration test
#[test]
fn test_expectations_with_new_astronomy() {
    use crate::core::astronomy::{AstronomicalState, SolarPhase};
    use crate::core::calendar::TimePeriod;

    let state = AstronomicalState::default();

    // Should be able to get TimePeriod from astronomy
    let time_period = state.time_period();

    // Should be compatible with expectations
    let pattern = PatternType::LocationDuring {
        location_id: EntityId::new(),
        time_period,
    };

    assert!(matches!(pattern, PatternType::LocationDuring { .. }));
}
```

**Step 3: Run full test suite**

Run: `cargo test`
Expected: PASS

**Step 4: Commit**

```bash
git add -A
git commit -m "feat: verify expectations system compatibility with astronomy"
```

---

## Task 7: Add Event Calendar Precomputation

**Files:**
- Modify: `src/core/astronomy.rs`

**Step 1: Write the failing test**

```rust
// Add to astronomy.rs tests
#[test]
fn test_precompute_event_calendar() {
    let mut state = AstronomicalState::new(TICKS_PER_DAY);

    // Precompute for 1 year
    state.precompute_events(YEAR_LENGTH as u32);

    // Should have entries for full moons
    // Argent: every 29 days, Sanguine: every 83 days
    let mut full_argent_count = 0;
    let mut full_sanguine_count = 0;

    for (_, events) in &state.event_calendar {
        if events.contains(&CelestialEvent::FullArgent) {
            full_argent_count += 1;
        }
        if events.contains(&CelestialEvent::FullSanguine) {
            full_sanguine_count += 1;
        }
    }

    // ~12 full Argent moons per year (360/29 ≈ 12.4)
    assert!(full_argent_count >= 12 && full_argent_count <= 13);
    // ~4 full Sanguine moons per year (360/83 ≈ 4.3)
    assert!(full_sanguine_count >= 4 && full_sanguine_count <= 5);
}

#[test]
fn test_query_next_event() {
    let mut state = AstronomicalState::new(TICKS_PER_DAY);
    state.precompute_events(YEAR_LENGTH as u32);

    // Should find next full Argent within 29 days
    let next = state.next_event_of_type(CelestialEvent::FullArgent);
    assert!(next.is_some());
    assert!(next.unwrap() <= 29);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib core::astronomy::tests::test_precompute`
Expected: FAIL

**Step 3: Write implementation**

```rust
// Add to AstronomicalState impl
impl AstronomicalState {
    /// Precompute events for the next N days
    pub fn precompute_events(&mut self, days: u32) {
        self.event_calendar.clear();

        let start_day = self.current_day;

        for day in start_day..(start_day + days) {
            let argent = MoonState::new(day, ARGENT_PERIOD, ARGENT_NODE_PRECESSION);
            let sanguine = MoonState::new(day, SANGUINE_PERIOD, SANGUINE_NODE_PRECESSION);

            let mut day_events = Vec::new();

            // Check moon phases
            if argent.is_full() { day_events.push(CelestialEvent::FullArgent); }
            if argent.is_new() { day_events.push(CelestialEvent::NewArgent); }
            if sanguine.is_full() { day_events.push(CelestialEvent::FullSanguine); }
            if sanguine.is_new() { day_events.push(CelestialEvent::NewSanguine); }

            // Check double events
            if argent.is_full() && sanguine.is_full() {
                if (argent.phase - 0.5).abs() < 0.02 && (sanguine.phase - 0.5).abs() < 0.02 {
                    day_events.push(CelestialEvent::PerfectDoubleFull);
                } else {
                    day_events.push(CelestialEvent::NearDoubleFull);
                }
            }
            if argent.is_new() && sanguine.is_new() {
                if argent.phase < 0.02 && sanguine.phase < 0.02 {
                    day_events.push(CelestialEvent::PerfectDoubleNew);
                } else {
                    day_events.push(CelestialEvent::NearDoubleNew);
                }
            }

            // Check eclipses
            let day_of_year = ((day % YEAR_LENGTH as u32) + 1) as u16;
            let sun_longitude = (day_of_year as f32 / YEAR_LENGTH as f32) * 360.0;

            if argent.is_new() && argent.eclipse_possible(sun_longitude) {
                day_events.push(CelestialEvent::SilverEclipse);
            }
            if sanguine.is_new() && sanguine.eclipse_possible(sun_longitude) {
                day_events.push(CelestialEvent::BloodEclipse);
            }

            if day_events.contains(&CelestialEvent::SilverEclipse)
                && day_events.contains(&CelestialEvent::BloodEclipse) {
                day_events.push(CelestialEvent::DoubleEclipse);
            }

            if !day_events.is_empty() {
                self.event_calendar.insert(day, day_events);
            }
        }
    }

    /// Find the next occurrence of a specific event type
    pub fn next_event_of_type(&self, event: CelestialEvent) -> Option<u32> {
        let mut next_day = None;

        for (&day, events) in &self.event_calendar {
            if day > self.current_day && events.contains(&event) {
                match next_day {
                    None => next_day = Some(day),
                    Some(d) if day < d => next_day = Some(day),
                    _ => {}
                }
            }
        }

        next_day.map(|d| d - self.current_day)
    }

    /// Get events for a specific day
    pub fn events_on_day(&self, day: u32) -> &[CelestialEvent] {
        self.event_calendar.get(&day).map(|v| v.as_slice()).unwrap_or(&[])
    }
}
```

**Step 4: Run tests**

Run: `cargo test --lib core::astronomy`
Expected: PASS

**Step 5: Commit**

```bash
git add src/core/astronomy.rs
git commit -m "feat: add event calendar precomputation"
```

---

## Task 8: Integration Tests

**Files:**
- Modify: `tests/emergence_tests.rs`

**Step 1: Write integration tests**

```rust
// Add to tests/emergence_tests.rs

use arc_citadel::core::astronomy::{
    AstronomicalState, Season, SolarPhase, CelestialEvent,
    FoundingModifiers, TICKS_PER_DAY, YEAR_LENGTH,
};

#[test]
fn test_full_year_simulation() {
    let mut state = AstronomicalState::new(TICKS_PER_DAY);

    // Advance through entire year
    for _ in 0..(YEAR_LENGTH as u64 * TICKS_PER_DAY) {
        state.advance_tick();
    }

    assert_eq!(state.year, 2);
    assert_eq!(state.season, Season::Spring);
}

#[test]
fn test_light_levels_through_day() {
    let mut state = AstronomicalState::new(TICKS_PER_DAY);

    let mut min_light = 1.0;
    let mut max_light = 0.0;

    // Advance through one day
    for _ in 0..TICKS_PER_DAY {
        state.advance_tick();
        min_light = min_light.min(state.light_level);
        max_light = max_light.max(state.light_level);
    }

    // Should have variation
    assert!(min_light < 0.3, "Should have darkness at night");
    assert!(max_light > 0.9, "Should have brightness at midday");
}

#[test]
fn test_double_full_is_rare() {
    let mut state = AstronomicalState::new(TICKS_PER_DAY);
    state.precompute_events(CONJUNCTION_CYCLE + 100); // More than one full cycle

    let mut perfect_double_count = 0;
    for (_, events) in &state.event_calendar {
        if events.contains(&CelestialEvent::PerfectDoubleFull) {
            perfect_double_count += 1;
        }
    }

    // Should happen exactly once per conjunction cycle (2407 days)
    assert!(perfect_double_count >= 1);
    assert!(perfect_double_count <= 2); // Could be 2 if we span boundary
}

#[test]
fn test_founding_conditions_integration() {
    let mut state = AstronomicalState::new(TICKS_PER_DAY);

    // Advance to deep winter
    let target_day = 340;
    for _ in 0..(target_day as u64 * TICKS_PER_DAY) {
        state.advance_tick();
    }

    assert_eq!(state.season, Season::Winter);

    // Calculate founding modifiers
    let modifiers = FoundingModifiers::calculate(
        state.day_of_year,
        state.season,
        &state.active_events,
    );

    assert!(modifiers.siege_mentality);
    assert!(modifiers.bias_tags.contains(&"defensive".to_string()));
}
```

**Step 2: Run integration tests**

Run: `cargo test --test emergence_tests`
Expected: PASS

**Step 3: Commit**

```bash
git add tests/emergence_tests.rs
git commit -m "test: add astronomical system integration tests"
```

---

## Task 9: Cleanup and Documentation

**Files:**
- Modify: `src/core/calendar.rs` (deprecation notice)
- Create: `src/core/README.md` (if needed)

**Step 1: Add deprecation notice to old Calendar**

```rust
// In src/core/calendar.rs, add at top:
//! # Deprecated
//!
//! This module is partially deprecated. Use `astronomy::AstronomicalState`
//! for time tracking. `TimePeriod` is still used for backward compatibility
//! with the expectations system.
```

**Step 2: Update CLAUDE.md if needed**

Ensure the module map reflects the new astronomy module.

**Step 3: Run full test suite**

Run: `cargo test`
Expected: PASS (all tests)

**Step 4: Final commit**

```bash
git add -A
git commit -m "docs: add deprecation notice and update documentation"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Core enums and constants | astronomy.rs, mod.rs |
| 2 | MoonState and phase calculations | astronomy.rs |
| 3 | AstronomicalState core | astronomy.rs |
| 4 | FoundingModifiers | astronomy.rs |
| 5 | World and tick integration | world.rs, tick.rs |
| 6 | Expectations compatibility | expectations.rs |
| 7 | Event calendar precomputation | astronomy.rs |
| 8 | Integration tests | emergence_tests.rs |
| 9 | Cleanup and documentation | calendar.rs, README |

**Estimated tasks:** 9
**Key integration points:** World.astronomy replaces World.calendar, TimePeriod::from(SolarPhase) for compatibility
