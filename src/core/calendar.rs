//! Calendar system for time-of-day tracking
//!
//! Provides time periods (Morning, Afternoon, Evening, Night) for
//! LocationDuring expectation patterns.

use serde::{Deserialize, Serialize};

/// Time of day periods for LocationDuring expectations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimePeriod {
    Morning,    // 06:00-12:00
    Afternoon,  // 12:00-18:00
    Evening,    // 18:00-22:00
    Night,      // 22:00-06:00
}

impl TimePeriod {
    pub fn from_hour(hour: u32) -> Self {
        match hour {
            6..=11 => TimePeriod::Morning,
            12..=17 => TimePeriod::Afternoon,
            18..=21 => TimePeriod::Evening,
            _ => TimePeriod::Night, // 22-23, 0-5
        }
    }
}

/// Calendar tracks simulation time with day/hour granularity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    tick: u64,
    ticks_per_day: u64,
}

impl Calendar {
    pub fn new(ticks_per_day: u64) -> Self {
        Self {
            tick: 0,
            ticks_per_day,
        }
    }

    pub fn advance(&mut self) {
        self.tick += 1;
    }

    pub fn current_tick(&self) -> u64 {
        self.tick
    }

    pub fn current_day(&self) -> u64 {
        self.tick / self.ticks_per_day
    }

    pub fn current_hour(&self) -> u32 {
        let tick_in_day = self.tick % self.ticks_per_day;
        let hours_per_day = 24;
        ((tick_in_day * hours_per_day) / self.ticks_per_day) as u32
    }

    pub fn current_time_period(&self) -> TimePeriod {
        TimePeriod::from_hour(self.current_hour())
    }

    pub fn ticks_per_day(&self) -> u64 {
        self.ticks_per_day
    }
}

impl Default for Calendar {
    fn default() -> Self {
        Self::new(1000) // Match existing TICKS_PER_DAY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_period_from_hour() {
        assert_eq!(TimePeriod::from_hour(6), TimePeriod::Morning);
        assert_eq!(TimePeriod::from_hour(11), TimePeriod::Morning);
        assert_eq!(TimePeriod::from_hour(12), TimePeriod::Afternoon);
        assert_eq!(TimePeriod::from_hour(17), TimePeriod::Afternoon);
        assert_eq!(TimePeriod::from_hour(18), TimePeriod::Evening);
        assert_eq!(TimePeriod::from_hour(21), TimePeriod::Evening);
        assert_eq!(TimePeriod::from_hour(22), TimePeriod::Night);
        assert_eq!(TimePeriod::from_hour(5), TimePeriod::Night);
    }

    #[test]
    fn test_calendar_advances() {
        let mut cal = Calendar::new(1000); // ticks per day
        assert_eq!(cal.current_tick(), 0);
        assert_eq!(cal.current_day(), 0);

        cal.advance();
        assert_eq!(cal.current_tick(), 1);

        // Advance to next day
        for _ in 0..999 {
            cal.advance();
        }
        assert_eq!(cal.current_tick(), 1000);
        assert_eq!(cal.current_day(), 1);
    }

    #[test]
    fn test_calendar_time_period() {
        let mut cal = Calendar::new(1000); // 1000 ticks per day

        // At tick 0, hour 0 = Night
        assert_eq!(cal.current_time_period(), TimePeriod::Night);

        // Advance to morning (6am = 250 ticks at 1000/day)
        for _ in 0..250 {
            cal.advance();
        }
        assert_eq!(cal.current_time_period(), TimePeriod::Morning);
    }
}
