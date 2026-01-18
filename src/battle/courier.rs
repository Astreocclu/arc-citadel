//! Courier system for order delivery
//!
//! Orders are NOT instant. Couriers carry commands across the battlefield.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::battle::formation_layout::FormationLineId;
use crate::battle::hex::{BattleHexCoord, HexDirection};
use crate::battle::planning::{EngagementRule, GoCodeId};
use crate::battle::units::{FormationId, FormationShape, UnitId};
use crate::core::types::{EntityId, Tick};

/// Unique identifier for couriers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CourierId(pub Uuid);

impl CourierId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CourierId {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of orders that can be sent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    MoveTo(BattleHexCoord),
    Attack(UnitId),
    Defend(BattleHexCoord),
    Retreat(Vec<BattleHexCoord>), // Retreat route
    ChangeFormation(FormationShape),
    ChangeEngagement(EngagementRule),
    ExecuteGoCode(GoCodeId),
    Rally,
    HoldPosition,
    /// Form a line between two hex coordinates
    /// Units will be distributed along the line and move to their assigned slots
    FormLine {
        start: BattleHexCoord,
        end: BattleHexCoord,
        facing: HexDirection,
        depth: u8,
    },
    /// Move to assigned slot in a formation line
    MoveToFormationSlot(FormationLineId),
}

/// Target of an order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderTarget {
    Unit(UnitId),
    Formation(FormationId),
}

/// An order to be delivered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_type: OrderType,
    pub target: OrderTarget,
    pub issued_at: Tick,
}

impl Order {
    pub fn new(order_type: OrderType, target: OrderTarget, tick: Tick) -> Self {
        Self {
            order_type,
            target,
            issued_at: tick,
        }
    }

    /// Convenience: create a move order
    pub fn move_to(unit_id: UnitId, destination: BattleHexCoord) -> Self {
        Self {
            order_type: OrderType::MoveTo(destination),
            target: OrderTarget::Unit(unit_id),
            issued_at: 0,
        }
    }

    /// Convenience: create a retreat order
    pub fn retreat(unit_id: UnitId, route: Vec<BattleHexCoord>) -> Self {
        Self {
            order_type: OrderType::Retreat(route),
            target: OrderTarget::Unit(unit_id),
            issued_at: 0,
        }
    }

    /// Convenience: create an attack order
    pub fn attack(unit_id: UnitId, target: UnitId) -> Self {
        Self {
            order_type: OrderType::Attack(target),
            target: OrderTarget::Unit(unit_id),
            issued_at: 0,
        }
    }

    /// Convenience: create a hold position order
    pub fn hold(unit_id: UnitId) -> Self {
        Self {
            order_type: OrderType::HoldPosition,
            target: OrderTarget::Unit(unit_id),
            issued_at: 0,
        }
    }
}

/// Status of a courier in flight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CourierStatus {
    #[default]
    EnRoute,
    Arrived,
    Intercepted, // Caught by enemy
    Lost,        // Courier killed
}

/// A courier carrying an order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourierInFlight {
    pub id: CourierId,
    pub courier_entity: EntityId,
    pub order: Order,

    pub source: BattleHexCoord,
    pub destination: BattleHexCoord,
    pub current_position: BattleHexCoord,

    pub progress: f32, // Progress to next hex (0.0 to 1.0)
    pub path: Vec<BattleHexCoord>,

    pub status: CourierStatus,
}

impl CourierInFlight {
    pub fn new(
        courier_entity: EntityId,
        order: Order,
        source: BattleHexCoord,
        destination: BattleHexCoord,
    ) -> Self {
        // Simple path for now (straight line)
        let path = source.line_to(&destination);

        Self {
            id: CourierId::new(),
            courier_entity,
            order,
            source,
            destination,
            current_position: source,
            progress: 0.0,
            path,
            status: CourierStatus::EnRoute,
        }
    }

    /// Has the courier arrived?
    pub fn has_arrived(&self) -> bool {
        matches!(self.status, CourierStatus::Arrived)
    }

    /// Is the courier still en route?
    pub fn is_en_route(&self) -> bool {
        matches!(self.status, CourierStatus::EnRoute)
    }

    /// Was the courier intercepted?
    pub fn was_intercepted(&self) -> bool {
        matches!(self.status, CourierStatus::Intercepted)
    }

    /// Advance courier position by one step
    pub fn advance(&mut self, speed: f32) {
        if !self.is_en_route() {
            return;
        }

        self.progress += speed;

        // Move to next hex when progress reaches 1.0
        while self.progress >= 1.0 && !self.path.is_empty() {
            self.current_position = self.path.remove(0);
            self.progress -= 1.0;
        }

        // Check if arrived
        if self.path.is_empty() && self.current_position == self.destination {
            self.status = CourierStatus::Arrived;
        }
    }

    /// Mark courier as intercepted
    pub fn intercept(&mut self) {
        self.status = CourierStatus::Intercepted;
    }

    /// Mark courier as lost
    pub fn lose(&mut self) {
        self.status = CourierStatus::Lost;
    }

    /// Estimate remaining travel time in ticks
    pub fn estimate_eta(&self, speed: f32) -> u32 {
        if !self.is_en_route() {
            return 0;
        }

        let remaining_hexes = self.path.len() as f32 + (1.0 - self.progress);
        (remaining_hexes / speed).ceil() as u32
    }
}

/// Courier system managing all couriers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CourierSystem {
    pub in_flight: Vec<CourierInFlight>,
    pub delivered: Vec<Order>,
}

impl CourierSystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// Dispatch a new courier
    pub fn dispatch(
        &mut self,
        courier_entity: EntityId,
        order: Order,
        source: BattleHexCoord,
        destination: BattleHexCoord,
    ) -> CourierId {
        let courier = CourierInFlight::new(courier_entity, order, source, destination);
        let id = courier.id;
        self.in_flight.push(courier);
        id
    }

    /// Advance all couriers
    pub fn advance_all(&mut self, speed: f32) {
        for courier in &mut self.in_flight {
            courier.advance(speed);
        }
    }

    /// Collect arrived orders
    pub fn collect_arrived(&mut self) -> Vec<Order> {
        let mut arrived = Vec::new();

        self.in_flight.retain(|courier| {
            if courier.has_arrived() {
                arrived.push(courier.order.clone());
                false // Remove from in_flight
            } else {
                true // Keep in in_flight
            }
        });

        self.delivered.extend(arrived.clone());
        arrived
    }

    /// Get courier by ID
    pub fn get_courier(&self, id: CourierId) -> Option<&CourierInFlight> {
        self.in_flight.iter().find(|c| c.id == id)
    }

    /// Get mutable courier by ID
    pub fn get_courier_mut(&mut self, id: CourierId) -> Option<&mut CourierInFlight> {
        self.in_flight.iter_mut().find(|c| c.id == id)
    }

    /// Count couriers en route
    pub fn count_en_route(&self) -> usize {
        self.in_flight.iter().filter(|c| c.is_en_route()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_courier_creation() {
        let courier = CourierInFlight::new(
            EntityId::new(),
            Order::move_to(UnitId::new(), BattleHexCoord::new(5, 5)),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(10, 10),
        );
        assert_eq!(courier.status, CourierStatus::EnRoute);
    }

    #[test]
    fn test_courier_not_arrived_initially() {
        let courier = CourierInFlight::new(
            EntityId::new(),
            Order::move_to(UnitId::new(), BattleHexCoord::new(5, 5)),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(10, 10),
        );
        assert!(!courier.has_arrived());
    }

    #[test]
    fn test_order_types() {
        let order = Order::retreat(UnitId::new(), vec![BattleHexCoord::new(0, 0)]);
        assert!(matches!(order.order_type, OrderType::Retreat(_)));
    }

    #[test]
    fn test_courier_advance() {
        let mut courier = CourierInFlight::new(
            EntityId::new(),
            Order::hold(UnitId::new()),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(3, 0),
        );

        // Advance until arrived
        for _ in 0..20 {
            courier.advance(0.5);
            if courier.has_arrived() {
                break;
            }
        }

        assert!(courier.has_arrived());
    }

    #[test]
    fn test_courier_system_dispatch_and_collect() {
        let mut system = CourierSystem::new();

        system.dispatch(
            EntityId::new(),
            Order::hold(UnitId::new()),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(0, 0), // Same position = instant delivery
        );

        // Advance once
        system.advance_all(1.0);

        // Should have one arrived
        let arrived = system.collect_arrived();
        assert_eq!(arrived.len(), 1);
    }

    #[test]
    fn test_courier_interception() {
        let mut courier = CourierInFlight::new(
            EntityId::new(),
            Order::hold(UnitId::new()),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(10, 10),
        );

        courier.intercept();
        assert!(courier.was_intercepted());
        assert!(!courier.is_en_route());
    }
}
