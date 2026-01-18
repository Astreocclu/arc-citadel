//! Weather system for campaign layer
//!
//! Weather affects army movement, visibility, combat effectiveness, and morale.
//! Seasons change weather probabilities and baseline conditions.

use serde::{Deserialize, Serialize};

use super::map::{CampaignMap, HexCoord};

/// Current weather condition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weather {
    Clear,
    Cloudy,
    Rain,
    HeavyRain,
    Snow,
    Blizzard,
    Fog,
    Sandstorm,
}

impl Weather {
    /// Movement speed multiplier (1.0 = normal)
    pub fn movement_modifier(&self) -> f32 {
        match self {
            Self::Clear => 1.0,
            Self::Cloudy => 1.0,
            Self::Rain => 0.8,
            Self::HeavyRain => 0.6,
            Self::Snow => 0.7,
            Self::Blizzard => 0.3,
            Self::Fog => 0.9,
            Self::Sandstorm => 0.4,
        }
    }

    /// Visibility range multiplier
    pub fn visibility_modifier(&self) -> f32 {
        match self {
            Self::Clear => 1.0,
            Self::Cloudy => 0.9,
            Self::Rain => 0.6,
            Self::HeavyRain => 0.4,
            Self::Snow => 0.7,
            Self::Blizzard => 0.2,
            Self::Fog => 0.3,
            Self::Sandstorm => 0.2,
        }
    }

    /// Combat effectiveness modifier for ranged attacks
    pub fn ranged_combat_modifier(&self) -> f32 {
        match self {
            Self::Clear => 1.0,
            Self::Cloudy => 1.0,
            Self::Rain => 0.7,
            Self::HeavyRain => 0.4,
            Self::Snow => 0.8,
            Self::Blizzard => 0.3,
            Self::Fog => 0.5,
            Self::Sandstorm => 0.3,
        }
    }

    /// Daily morale impact from weather
    pub fn morale_impact(&self) -> f32 {
        match self {
            Self::Clear => 0.0,
            Self::Cloudy => 0.0,
            Self::Rain => -0.01,
            Self::HeavyRain => -0.02,
            Self::Snow => -0.01,
            Self::Blizzard => -0.05,
            Self::Fog => 0.0,
            Self::Sandstorm => -0.03,
        }
    }

    /// Daily supply spoilage rate increase
    pub fn supply_spoilage(&self) -> f32 {
        match self {
            Self::Clear => 0.0,
            Self::Cloudy => 0.0,
            Self::Rain => 0.05,
            Self::HeavyRain => 0.1,
            Self::Snow => 0.0,    // Cold preserves
            Self::Blizzard => 0.0,
            Self::Fog => 0.02,
            Self::Sandstorm => 0.05,
        }
    }
}

impl Default for Weather {
    fn default() -> Self {
        Self::Clear
    }
}

/// Season affecting weather probabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    /// Get season from day of year (0-365)
    pub fn from_day(day: u32) -> Self {
        match day % 360 {
            0..=89 => Self::Spring,
            90..=179 => Self::Summer,
            180..=269 => Self::Autumn,
            _ => Self::Winter,
        }
    }

    /// Base temperature modifier
    pub fn temperature_modifier(&self) -> f32 {
        match self {
            Self::Spring => 0.0,
            Self::Summer => 1.0,
            Self::Autumn => 0.0,
            Self::Winter => -1.0,
        }
    }

    /// Weather probabilities [Clear, Cloudy, Rain, HeavyRain, Snow, Blizzard, Fog]
    pub fn weather_weights(&self) -> [f32; 7] {
        match self {
            Self::Spring => [0.3, 0.3, 0.2, 0.1, 0.0, 0.0, 0.1],
            Self::Summer => [0.5, 0.2, 0.15, 0.1, 0.0, 0.0, 0.05],
            Self::Autumn => [0.2, 0.3, 0.25, 0.1, 0.0, 0.0, 0.15],
            Self::Winter => [0.3, 0.2, 0.1, 0.05, 0.2, 0.1, 0.05],
        }
    }
}

impl Default for Season {
    fn default() -> Self {
        Self::Spring
    }
}

/// Weather state for a region or the whole map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherState {
    pub current_weather: Weather,
    pub current_season: Season,
    pub days_with_current: u32,   // How long current weather has persisted
    pub forecast_days: u32,       // Days until weather changes
}

impl WeatherState {
    pub fn new() -> Self {
        Self {
            current_weather: Weather::Clear,
            current_season: Season::Spring,
            days_with_current: 0,
            forecast_days: 3,
        }
    }

    /// Update weather based on elapsed time
    pub fn update(&mut self, dt_days: f32, day_of_year: u32, rng_seed: u64) {
        self.current_season = Season::from_day(day_of_year);
        self.days_with_current += dt_days as u32;

        if self.days_with_current >= self.forecast_days {
            // Time to change weather
            self.current_weather = self.roll_weather(rng_seed);
            self.days_with_current = 0;
            self.forecast_days = self.roll_duration(rng_seed);
        }
    }

    fn roll_weather(&self, seed: u64) -> Weather {
        let weights = self.current_season.weather_weights();
        let roll = (simple_hash(seed, 0) % 100) as f32 / 100.0;

        let mut cumulative = 0.0;
        for (i, &w) in weights.iter().enumerate() {
            cumulative += w;
            if roll < cumulative {
                return match i {
                    0 => Weather::Clear,
                    1 => Weather::Cloudy,
                    2 => Weather::Rain,
                    3 => Weather::HeavyRain,
                    4 => Weather::Snow,
                    5 => Weather::Blizzard,
                    6 => Weather::Fog,
                    _ => Weather::Clear,
                };
            }
        }
        Weather::Clear
    }

    fn roll_duration(&self, seed: u64) -> u32 {
        // Weather persists 1-5 days typically
        let roll = simple_hash(seed, 1) % 5;
        (roll + 1) as u32
    }
}

impl Default for WeatherState {
    fn default() -> Self {
        Self::new()
    }
}

fn simple_hash(seed: u64, modifier: u64) -> u64 {
    let mut h = seed.wrapping_add(modifier);
    h = h.wrapping_mul(6364136223846793005);
    h = h.wrapping_add(1442695040888963407);
    h ^ (h >> 32)
}

/// Regional weather - different areas can have different weather
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalWeather {
    /// Weather zones - each covers a rectangular region
    pub zones: Vec<WeatherZone>,
    /// Global weather fallback
    pub global_weather: WeatherState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherZone {
    pub center: HexCoord,
    pub radius: i32,
    pub weather: WeatherState,
}

impl RegionalWeather {
    pub fn new() -> Self {
        Self {
            zones: Vec::new(),
            global_weather: WeatherState::new(),
        }
    }

    /// Add a weather zone
    pub fn add_zone(&mut self, center: HexCoord, radius: i32) {
        self.zones.push(WeatherZone {
            center,
            radius,
            weather: WeatherState::new(),
        });
    }

    /// Get weather at a specific hex
    pub fn get_weather_at(&self, coord: &HexCoord) -> Weather {
        // Check if in any zone
        for zone in &self.zones {
            if zone.center.distance(coord) <= zone.radius {
                return zone.weather.current_weather;
            }
        }
        // Fall back to global
        self.global_weather.current_weather
    }

    /// Update all weather
    pub fn update(&mut self, dt_days: f32, day_of_year: u32, base_seed: u64) {
        self.global_weather.update(dt_days, day_of_year, base_seed);

        for (i, zone) in self.zones.iter_mut().enumerate() {
            let zone_seed = base_seed.wrapping_add(i as u64 * 12345);
            zone.weather.update(dt_days, day_of_year, zone_seed);
        }
    }

    /// Get combined movement modifier at a hex (terrain + weather)
    pub fn movement_modifier_at(&self, coord: &HexCoord, map: &CampaignMap) -> f32 {
        let weather = self.get_weather_at(coord);
        let weather_mod = weather.movement_modifier();

        if let Some(tile) = map.get(coord) {
            // Additional penalty for difficult terrain in bad weather
            let terrain_weather_penalty = match (tile.terrain, weather) {
                // Mountains and snow/blizzard
                (super::map::CampaignTerrain::Mountains, Weather::Snow) => 0.8,
                (super::map::CampaignTerrain::Mountains, Weather::Blizzard) => 0.5,
                // Swamps and rain
                (super::map::CampaignTerrain::Swamp, Weather::Rain) => 0.7,
                (super::map::CampaignTerrain::Swamp, Weather::HeavyRain) => 0.5,
                // Desert and sandstorm
                (super::map::CampaignTerrain::Desert, Weather::Sandstorm) => 0.3,
                _ => 1.0,
            };
            weather_mod * terrain_weather_penalty
        } else {
            weather_mod
        }
    }

    /// Get visibility modifier at a hex
    pub fn visibility_modifier_at(&self, coord: &HexCoord, map: &CampaignMap) -> f32 {
        let weather = self.get_weather_at(coord);
        let weather_vis = weather.visibility_modifier();

        if let Some(tile) = map.get(coord) {
            let terrain_vis = tile.terrain.visibility_modifier();
            weather_vis * terrain_vis
        } else {
            weather_vis
        }
    }
}

impl Default for RegionalWeather {
    fn default() -> Self {
        Self::new()
    }
}

/// Weather events for the campaign log
#[derive(Debug, Clone)]
pub enum WeatherEvent {
    WeatherChanged { old: Weather, new: Weather },
    SeasonChanged { old: Season, new: Season },
    ExtremeWeatherWarning { weather: Weather, duration: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_modifiers() {
        assert_eq!(Weather::Clear.movement_modifier(), 1.0);
        assert!(Weather::Blizzard.movement_modifier() < 0.5);
        assert!(Weather::Fog.visibility_modifier() < 0.5);
    }

    #[test]
    fn test_season_from_day() {
        assert_eq!(Season::from_day(0), Season::Spring);
        assert_eq!(Season::from_day(45), Season::Spring);
        assert_eq!(Season::from_day(90), Season::Summer);
        assert_eq!(Season::from_day(180), Season::Autumn);
        assert_eq!(Season::from_day(270), Season::Winter);
        assert_eq!(Season::from_day(360), Season::Spring); // Wraps
    }

    #[test]
    fn test_weather_state_update() {
        let mut weather = WeatherState::new();
        weather.forecast_days = 1;

        // Should trigger weather change after 1 day
        weather.update(2.0, 0, 42);

        // Weather should have changed
        assert_eq!(weather.days_with_current, 0);
        assert!(weather.forecast_days >= 1);
    }

    #[test]
    fn test_regional_weather() {
        let mut regional = RegionalWeather::new();
        regional.add_zone(HexCoord::new(10, 10), 5);

        // Near zone center should get zone weather
        let zone_coord = HexCoord::new(10, 10);
        let _weather = regional.get_weather_at(&zone_coord);

        // Far from zone should get global weather
        let far_coord = HexCoord::new(50, 50);
        let far_weather = regional.get_weather_at(&far_coord);
        assert_eq!(far_weather, regional.global_weather.current_weather);
    }
}
