use super::event_types::EventType;
use crate::core::types::EntityId;
use serde::{Deserialize, Serialize};

/// A recent event witnessed or performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentEvent {
    pub event_type: EventType,
    pub actor: EntityId,
    pub tick: u64,
}

/// Ring buffer of recent events for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBuffer {
    pub events: Vec<RecentEvent>,
    capacity: usize,
}

impl EventBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, event: RecentEvent) {
        if self.events.len() >= self.capacity {
            self.events.remove(0); // Remove oldest
        }
        self.events.push(event);
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Find a recent event by actor and type within last N ticks
    pub fn find_recent(
        &self,
        actor: EntityId,
        event_type: EventType,
        within_ticks: u64,
    ) -> Option<&RecentEvent> {
        let min_tick = self
            .events
            .last()
            .map(|e| e.tick.saturating_sub(within_ticks))
            .unwrap_or(0);

        self.events
            .iter()
            .rev()
            .find(|e| e.actor == actor && e.event_type == event_type && e.tick >= min_tick)
    }

    /// Get events involving a specific actor
    pub fn events_by_actor(&self, actor: EntityId) -> impl Iterator<Item = &RecentEvent> {
        self.events.iter().filter(move |e| e.actor == actor)
    }

    /// Clear old events beyond a tick threshold
    pub fn clear_before(&mut self, tick: u64) {
        self.events.retain(|e| e.tick >= tick);
    }
}

impl Default for EventBuffer {
    fn default() -> Self {
        Self::new(10) // Default capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_buffer_ring() {
        let mut buffer = EventBuffer::new(3); // Small for testing

        buffer.push(RecentEvent {
            event_type: EventType::Observation,
            actor: EntityId::new(),
            tick: 1,
        });
        buffer.push(RecentEvent {
            event_type: EventType::Transaction,
            actor: EntityId::new(),
            tick: 2,
        });
        buffer.push(RecentEvent {
            event_type: EventType::AidReceived,
            actor: EntityId::new(),
            tick: 3,
        });

        assert_eq!(buffer.len(), 3);

        // Push fourth - oldest should be evicted
        buffer.push(RecentEvent {
            event_type: EventType::Betrayal,
            actor: EntityId::new(),
            tick: 4,
        });

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.events[0].tick, 2); // tick 1 evicted
    }

    #[test]
    fn test_find_recent_event() {
        let mut buffer = EventBuffer::new(10);
        let actor = EntityId::new();

        buffer.push(RecentEvent {
            event_type: EventType::AidReceived,
            actor,
            tick: 100,
        });

        let found = buffer.find_recent(actor, EventType::AidReceived, 50);
        assert!(found.is_some());

        let not_found = buffer.find_recent(actor, EventType::Betrayal, 50);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_default_capacity() {
        let buffer = EventBuffer::default();
        assert_eq!(buffer.capacity, 10);
    }

    #[test]
    fn test_is_empty() {
        let buffer = EventBuffer::new(5);
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_events_by_actor() {
        let mut buffer = EventBuffer::new(10);
        let actor1 = EntityId::new();
        let actor2 = EntityId::new();

        buffer.push(RecentEvent {
            event_type: EventType::Observation,
            actor: actor1,
            tick: 1,
        });
        buffer.push(RecentEvent {
            event_type: EventType::Transaction,
            actor: actor2,
            tick: 2,
        });
        buffer.push(RecentEvent {
            event_type: EventType::AidReceived,
            actor: actor1,
            tick: 3,
        });

        let actor1_events: Vec<_> = buffer.events_by_actor(actor1).collect();
        assert_eq!(actor1_events.len(), 2);
        assert!(actor1_events.iter().all(|e| e.actor == actor1));
    }

    #[test]
    fn test_clear_before() {
        let mut buffer = EventBuffer::new(10);
        let actor = EntityId::new();

        buffer.push(RecentEvent {
            event_type: EventType::Observation,
            actor,
            tick: 10,
        });
        buffer.push(RecentEvent {
            event_type: EventType::Transaction,
            actor,
            tick: 20,
        });
        buffer.push(RecentEvent {
            event_type: EventType::AidReceived,
            actor,
            tick: 30,
        });

        assert_eq!(buffer.len(), 3);

        buffer.clear_before(25);
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.events[0].tick, 30);
    }

    #[test]
    fn test_find_recent_respects_tick_range() {
        let mut buffer = EventBuffer::new(10);
        let actor = EntityId::new();

        buffer.push(RecentEvent {
            event_type: EventType::AidReceived,
            actor,
            tick: 10,
        });
        buffer.push(RecentEvent {
            event_type: EventType::Transaction,
            actor,
            tick: 100,
        });

        // Looking for AidReceived within 50 ticks of most recent (100)
        // min_tick = 100 - 50 = 50, so tick 10 is outside range
        let found = buffer.find_recent(actor, EventType::AidReceived, 50);
        assert!(found.is_none());

        // But within 100 ticks it should be found
        let found = buffer.find_recent(actor, EventType::AidReceived, 100);
        assert!(found.is_some());
    }
}
