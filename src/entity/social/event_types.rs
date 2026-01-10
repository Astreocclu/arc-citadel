use serde::{Deserialize, Serialize};

/// Valence of a memory - positive or negative
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Valence {
    Positive,
    Negative,
}

/// Types of social events that create memories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    // Positive events
    AidReceived, // They helped me
    AidGiven,    // I helped them
    GiftReceived,
    GiftGiven,
    SharedExperience, // Survived danger together
    Compliment,
    PromiseKept,

    // Negative events
    HarmReceived, // They hurt me
    HarmGiven,    // I hurt them
    Insult,
    Theft,
    Betrayal,
    PromiseBroken,

    // Neutral but formative
    FirstMeeting,
    Transaction, // Trade, business
    Observation, // Witnessed them do something notable
}

impl EventType {
    /// Default valence for this event type
    pub fn default_valence(&self) -> Valence {
        match self {
            EventType::AidReceived
            | EventType::AidGiven
            | EventType::GiftReceived
            | EventType::GiftGiven
            | EventType::SharedExperience
            | EventType::Compliment
            | EventType::PromiseKept => Valence::Positive,

            EventType::HarmReceived
            | EventType::HarmGiven
            | EventType::Insult
            | EventType::Theft
            | EventType::Betrayal
            | EventType::PromiseBroken => Valence::Negative,

            // Neutral events default to positive (slight familiarity bonus)
            EventType::FirstMeeting | EventType::Transaction | EventType::Observation => {
                Valence::Positive
            }
        }
    }

    /// Base intensity for this event type (0.0 to 1.0)
    pub fn base_intensity(&self) -> f32 {
        match self {
            EventType::Betrayal | EventType::SharedExperience => 0.9,
            EventType::HarmReceived | EventType::AidReceived => 0.7,
            EventType::GiftReceived | EventType::PromiseKept | EventType::PromiseBroken => 0.6,
            EventType::HarmGiven | EventType::AidGiven | EventType::GiftGiven => 0.5,
            EventType::Theft | EventType::Insult | EventType::Compliment => 0.4,
            EventType::FirstMeeting => 0.3,
            EventType::Transaction | EventType::Observation => 0.2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_events_have_positive_valence() {
        let positive_events = [
            EventType::AidReceived,
            EventType::AidGiven,
            EventType::GiftReceived,
            EventType::GiftGiven,
            EventType::SharedExperience,
            EventType::Compliment,
            EventType::PromiseKept,
        ];

        for event in positive_events {
            assert_eq!(
                event.default_valence(),
                Valence::Positive,
                "{:?} should be positive",
                event
            );
        }
    }

    #[test]
    fn test_negative_events_have_negative_valence() {
        let negative_events = [
            EventType::HarmReceived,
            EventType::HarmGiven,
            EventType::Insult,
            EventType::Theft,
            EventType::Betrayal,
            EventType::PromiseBroken,
        ];

        for event in negative_events {
            assert_eq!(
                event.default_valence(),
                Valence::Negative,
                "{:?} should be negative",
                event
            );
        }
    }

    #[test]
    fn test_neutral_events_default_to_positive() {
        let neutral_events = [
            EventType::FirstMeeting,
            EventType::Transaction,
            EventType::Observation,
        ];

        for event in neutral_events {
            assert_eq!(
                event.default_valence(),
                Valence::Positive,
                "{:?} should default to positive",
                event
            );
        }
    }

    #[test]
    fn test_base_intensity_in_valid_range() {
        let all_events = [
            EventType::AidReceived,
            EventType::AidGiven,
            EventType::GiftReceived,
            EventType::GiftGiven,
            EventType::SharedExperience,
            EventType::Compliment,
            EventType::PromiseKept,
            EventType::HarmReceived,
            EventType::HarmGiven,
            EventType::Insult,
            EventType::Theft,
            EventType::Betrayal,
            EventType::PromiseBroken,
            EventType::FirstMeeting,
            EventType::Transaction,
            EventType::Observation,
        ];

        for event in all_events {
            let intensity = event.base_intensity();
            assert!(
                intensity >= 0.0 && intensity <= 1.0,
                "{:?} intensity {} out of range",
                event,
                intensity
            );
        }
    }

    #[test]
    fn test_betrayal_is_most_intense() {
        assert_eq!(EventType::Betrayal.base_intensity(), 0.9);
        assert_eq!(EventType::SharedExperience.base_intensity(), 0.9);
    }
}
