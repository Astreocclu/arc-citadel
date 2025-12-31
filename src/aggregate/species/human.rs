//! Human species behavior stub

use crate::aggregate::polity::Polity;
use crate::aggregate::world::AggregateWorld;
use crate::aggregate::events::EventType;

/// Generate events for human polities
pub fn tick(_polity: &Polity, _world: &AggregateWorld, _year: u32) -> Vec<EventType> {
    Vec::new()
}
