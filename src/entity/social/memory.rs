use serde::{Deserialize, Serialize};
use super::event_types::{EventType, Valence};

/// A single memory about an interaction with another entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipMemory {
    /// What happened
    pub event_type: EventType,
    /// Positive or negative experience
    pub valence: Valence,
    /// How impactful (0.0 to 1.0)
    pub intensity: f32,
    /// Current importance, decays over time (0.0 to 1.0)
    pub salience: f32,
    /// When this memory was formed
    pub tick_created: u64,
}

impl RelationshipMemory {
    pub fn new(event_type: EventType, valence: Valence, intensity: f32, tick: u64) -> Self {
        Self {
            event_type,
            valence,
            intensity: intensity.clamp(0.0, 1.0),
            salience: 1.0, // Starts at full salience
            tick_created: tick,
        }
    }

    /// Create memory with default valence and intensity from event type
    pub fn from_event(event_type: EventType, tick: u64) -> Self {
        Self::new(
            event_type,
            event_type.default_valence(),
            event_type.base_intensity(),
            tick,
        )
    }

    /// Apply decay based on days passed
    /// decay_rate is per-day rate (e.g., 0.02 = 2% per day)
    pub fn apply_decay(&mut self, current_tick: u64, decay_rate: f32) {
        let ticks_per_day = 1000; // TODO: Make configurable
        let days_passed = (current_tick.saturating_sub(self.tick_created)) as f32 / ticks_per_day as f32;
        self.salience = (1.0 - decay_rate).powf(days_passed).max(0.01);
    }

    /// Weighted importance: intensity * salience
    pub fn weighted_importance(&self) -> f32 {
        self.intensity * self.salience
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = RelationshipMemory::new(
            EventType::AidReceived,
            Valence::Positive,
            0.8,
            100,
        );
        assert_eq!(memory.event_type, EventType::AidReceived);
        assert_eq!(memory.valence, Valence::Positive);
        assert!((memory.intensity - 0.8).abs() < 0.01);
        assert!((memory.salience - 1.0).abs() < 0.01); // Starts at full salience
        assert_eq!(memory.tick_created, 100);
    }

    #[test]
    fn test_salience_decay() {
        let mut memory = RelationshipMemory::new(
            EventType::AidReceived,
            Valence::Positive,
            0.8,
            0,
        );

        // After 1000 ticks (1 day), salience should decay by ~2%
        memory.apply_decay(1000, 0.02);
        assert!(memory.salience < 1.0);
        assert!(memory.salience > 0.9);
    }

    #[test]
    fn test_from_event_uses_defaults() {
        let memory = RelationshipMemory::from_event(EventType::Betrayal, 50);

        assert_eq!(memory.event_type, EventType::Betrayal);
        assert_eq!(memory.valence, Valence::Negative); // Betrayal is negative
        assert!((memory.intensity - 0.9).abs() < 0.01); // Betrayal has 0.9 intensity
        assert_eq!(memory.tick_created, 50);
        assert!((memory.salience - 1.0).abs() < 0.01); // Starts at full salience
    }

    #[test]
    fn test_weighted_importance() {
        let mut memory = RelationshipMemory::new(
            EventType::AidReceived,
            Valence::Positive,
            0.8,
            0,
        );

        // Initial: intensity 0.8, salience 1.0
        assert!((memory.weighted_importance() - 0.8).abs() < 0.01);

        // Apply decay
        memory.apply_decay(1000, 0.02);

        // weighted_importance should now be less than 0.8
        assert!(memory.weighted_importance() < 0.8);
        assert!(memory.weighted_importance() > 0.7); // But not too much less
    }

    #[test]
    fn test_intensity_clamped() {
        let memory = RelationshipMemory::new(
            EventType::AidReceived,
            Valence::Positive,
            1.5, // Over 1.0
            0,
        );
        assert_eq!(memory.intensity, 1.0);

        let memory2 = RelationshipMemory::new(
            EventType::AidReceived,
            Valence::Positive,
            -0.5, // Under 0.0
            0,
        );
        assert_eq!(memory2.intensity, 0.0);
    }

    #[test]
    fn test_salience_floor() {
        let mut memory = RelationshipMemory::new(
            EventType::AidReceived,
            Valence::Positive,
            0.8,
            0,
        );

        // Apply massive decay (100000 ticks = 100 days with high decay rate)
        memory.apply_decay(100000, 0.5);

        // Salience should be at floor of 0.01, never 0
        assert!(memory.salience >= 0.01);
    }

    #[test]
    fn test_multiple_decay_applications() {
        let mut memory = RelationshipMemory::new(
            EventType::GiftReceived,
            Valence::Positive,
            0.6,
            0,
        );

        // First decay at tick 1000
        memory.apply_decay(1000, 0.02);
        let salience_after_day1 = memory.salience;

        // Second decay at tick 2000
        memory.apply_decay(2000, 0.02);
        let salience_after_day2 = memory.salience;

        // Salience should continue to decay
        assert!(salience_after_day2 < salience_after_day1);
    }
}
