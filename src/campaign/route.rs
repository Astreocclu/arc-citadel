//! Army movement and orders for campaign layer
//!
//! Armies are groups of units that move across the campaign map.

use serde::{Deserialize, Serialize};

use super::map::{CampaignMap, HexCoord};
use crate::core::types::PolityId;

/// Unique identifier for an army
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArmyId(pub u32);

/// Army stance affects behavior when encountering enemies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArmyStance {
    Aggressive, // Engage enemies on sight
    Defensive,  // Hold position, defend if attacked
    Evasive,    // Avoid combat when possible
}

impl Default for ArmyStance {
    fn default() -> Self {
        Self::Defensive
    }
}

/// Orders that can be given to an army
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArmyOrder {
    MoveTo(HexCoord),
    Patrol(Vec<HexCoord>),
    Guard(HexCoord),
    Halt,
}

/// Result of movement execution
#[derive(Debug, Clone, PartialEq)]
pub enum MovementResult {
    NoOrders,
    Moving,
    Arrived,
    Blocked,
    Intercepted(ArmyId),
}

/// An army on the campaign map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Army {
    pub id: ArmyId,
    pub name: String,
    pub faction: PolityId,
    pub position: HexCoord,
    pub unit_count: u32,
    pub morale: f32,               // 0.0 - 1.0
    pub stance: ArmyStance,
    pub orders: Option<ArmyOrder>,
    pub movement_points: f32,      // Accumulated movement progress
    pub path_cache: Option<Vec<HexCoord>>, // Cached path to destination
    pub engaged_with: Option<ArmyId>, // Currently engaged in battle with this army
}

impl Army {
    pub fn new(id: ArmyId, name: String, faction: PolityId, position: HexCoord) -> Self {
        Self {
            id,
            name,
            faction,
            position,
            unit_count: 100,
            morale: 1.0,
            stance: ArmyStance::default(),
            orders: None,
            movement_points: 0.0,
            path_cache: None,
            engaged_with: None,
        }
    }

    pub fn with_units(mut self, count: u32) -> Self {
        self.unit_count = count;
        self
    }

    pub fn with_stance(mut self, stance: ArmyStance) -> Self {
        self.stance = stance;
        self
    }

    /// Give movement orders to the army
    pub fn order_move_to(&mut self, destination: HexCoord, map: &CampaignMap) {
        self.orders = Some(ArmyOrder::MoveTo(destination));
        // Pre-compute path
        self.path_cache = map.find_path(self.position, destination);
    }

    /// Give guard orders to the army
    pub fn order_guard(&mut self, position: HexCoord) {
        self.orders = Some(ArmyOrder::Guard(position));
        self.path_cache = None;
    }

    /// Halt the army
    pub fn order_halt(&mut self) {
        self.orders = Some(ArmyOrder::Halt);
        self.path_cache = None;
    }

    /// Calculate movement cost to enter a hex
    pub fn movement_cost_to(&self, map: &CampaignMap, to: &HexCoord) -> f32 {
        let Some(tile) = map.get(to) else {
            return f32::INFINITY;
        };

        let base_cost = tile.terrain.movement_cost();

        // Larger armies move slower
        let size_penalty = 1.0 + (self.unit_count as f32 / 1000.0).min(0.5);

        // Low morale slows movement
        let morale_penalty = if self.morale < 0.3 { 1.5 } else { 1.0 };

        base_cost * size_penalty * morale_penalty
    }

    /// Execute movement for this tick
    pub fn execute_movement(&mut self, map: &CampaignMap, dt_days: f32) -> MovementResult {
        let Some(ref orders) = self.orders else {
            return MovementResult::NoOrders;
        };

        let destination = match orders {
            ArmyOrder::MoveTo(dest) => *dest,
            ArmyOrder::Guard(pos) => {
                if self.position == *pos {
                    return MovementResult::NoOrders; // Already at guard position
                }
                *pos
            }
            ArmyOrder::Patrol(waypoints) => {
                // Simple patrol: move to first waypoint, cycle when reached
                if let Some(first) = waypoints.first() {
                    *first
                } else {
                    return MovementResult::NoOrders;
                }
            }
            ArmyOrder::Halt => return MovementResult::NoOrders,
        };

        if self.position == destination {
            self.orders = None;
            self.path_cache = None;
            return MovementResult::Arrived;
        }

        // Ensure we have a valid path
        if self.path_cache.is_none() {
            self.path_cache = map.find_path(self.position, destination);
        }

        let Some(ref path) = self.path_cache else {
            return MovementResult::Blocked;
        };

        // Find next hex in path
        let current_idx = path.iter().position(|&h| h == self.position).unwrap_or(0);
        let Some(&next_hex) = path.get(current_idx + 1) else {
            // Reached end of path
            self.orders = None;
            self.path_cache = None;
            return MovementResult::Arrived;
        };

        // Accumulate movement points
        self.movement_points += dt_days;

        let cost = self.movement_cost_to(map, &next_hex);
        if self.movement_points >= cost {
            self.movement_points -= cost;
            self.position = next_hex;

            if self.position == destination {
                self.orders = None;
                self.path_cache = None;
                return MovementResult::Arrived;
            }

            MovementResult::Moving
        } else {
            MovementResult::Moving
        }
    }

    /// Check if this army should intercept another
    pub fn should_intercept(&self, other: &Army) -> bool {
        if self.faction == other.faction {
            return false; // Same faction
        }

        // Don't intercept if already engaged with this army
        if self.engaged_with == Some(other.id) || other.engaged_with == Some(self.id) {
            return false;
        }

        // Armies with broken morale won't initiate combat
        if self.morale < 0.2 {
            return false;
        }

        match self.stance {
            ArmyStance::Aggressive => true,
            ArmyStance::Defensive => self.position == other.position,
            ArmyStance::Evasive => false,
        }
    }

    /// Clear engagement status (e.g., after battle resolution)
    pub fn disengage(&mut self) {
        self.engaged_with = None;
    }
}

/// Campaign state holding all armies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignState {
    pub map: CampaignMap,
    pub armies: Vec<Army>,
    pub current_day: f32,
    next_army_id: u32,
}

impl CampaignState {
    pub fn new(map: CampaignMap) -> Self {
        Self {
            map,
            armies: Vec::new(),
            current_day: 0.0,
            next_army_id: 1,
        }
    }

    /// Spawn a new army
    pub fn spawn_army(&mut self, name: String, faction: PolityId, position: HexCoord) -> ArmyId {
        let id = ArmyId(self.next_army_id);
        self.next_army_id += 1;
        let army = Army::new(id, name, faction, position);
        self.armies.push(army);
        id
    }

    /// Get an army by ID
    pub fn get_army(&self, id: ArmyId) -> Option<&Army> {
        self.armies.iter().find(|a| a.id == id)
    }

    /// Get a mutable army by ID
    pub fn get_army_mut(&mut self, id: ArmyId) -> Option<&mut Army> {
        self.armies.iter_mut().find(|a| a.id == id)
    }

    /// Get all armies at a position
    pub fn armies_at(&self, position: HexCoord) -> Vec<&Army> {
        self.armies.iter().filter(|a| a.position == position).collect()
    }

    /// Check for interceptions at a position
    pub fn check_interceptions(&self, position: HexCoord) -> Vec<(ArmyId, ArmyId)> {
        let armies_here: Vec<_> = self.armies_at(position);
        let mut interceptions = Vec::new();

        for (i, army_a) in armies_here.iter().enumerate() {
            for army_b in armies_here.iter().skip(i + 1) {
                if army_a.should_intercept(army_b) || army_b.should_intercept(army_a) {
                    interceptions.push((army_a.id, army_b.id));
                }
            }
        }

        interceptions
    }
}

/// Run a campaign tick
pub fn campaign_tick(state: &mut CampaignState, dt_days: f32) -> Vec<CampaignEvent> {
    let mut events = Vec::new();

    // Store map reference for movement calculations
    let map = state.map.clone();

    // Process army movement
    for army in &mut state.armies {
        let result = army.execute_movement(&map, dt_days);
        match result {
            MovementResult::Arrived => {
                events.push(CampaignEvent::ArmyArrived {
                    army: army.id,
                    position: army.position,
                });
            }
            MovementResult::Moving => {
                events.push(CampaignEvent::ArmyMoved {
                    army: army.id,
                    position: army.position,
                });
            }
            _ => {}
        }
    }

    // Check for interceptions
    let positions: Vec<_> = state.armies.iter().map(|a| a.position).collect();
    let mut new_engagements = Vec::new();
    for position in positions.iter().copied().collect::<std::collections::HashSet<_>>() {
        for (a, b) in state.check_interceptions(position) {
            events.push(CampaignEvent::ArmiesEngaged {
                army_a: a,
                army_b: b,
                position,
            });
            new_engagements.push((a, b));
        }
    }

    // Mark armies as engaged
    for (a, b) in new_engagements {
        if let Some(army_a) = state.get_army_mut(a) {
            army_a.engaged_with = Some(b);
        }
        if let Some(army_b) = state.get_army_mut(b) {
            army_b.engaged_with = Some(a);
        }
    }

    // Advance time
    state.current_day += dt_days;

    events
}

/// Events that can occur during campaign tick
#[derive(Debug, Clone)]
pub enum CampaignEvent {
    ArmyMoved { army: ArmyId, position: HexCoord },
    ArmyArrived { army: ArmyId, position: HexCoord },
    ArmiesEngaged { army_a: ArmyId, army_b: ArmyId, position: HexCoord },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_map() -> CampaignMap {
        CampaignMap::generate_simple(10, 10, 42)
    }

    #[test]
    fn test_army_creation() {
        let army = Army::new(
            ArmyId(1),
            "Test Army".to_string(),
            PolityId(1),
            HexCoord::new(0, 0),
        );
        assert_eq!(army.unit_count, 100);
        assert_eq!(army.morale, 1.0);
    }

    #[test]
    fn test_army_movement() {
        let map = test_map();
        let mut army = Army::new(
            ArmyId(1),
            "Test Army".to_string(),
            PolityId(1),
            HexCoord::new(0, 0),
        );

        army.order_move_to(HexCoord::new(3, 3), &map);

        // Simulate movement over time
        for _ in 0..20 {
            let result = army.execute_movement(&map, 1.0);
            if result == MovementResult::Arrived {
                break;
            }
        }

        assert_eq!(army.position, HexCoord::new(3, 3));
    }

    #[test]
    fn test_campaign_state() {
        let map = test_map();
        let mut state = CampaignState::new(map);

        let army_id = state.spawn_army(
            "First Army".to_string(),
            PolityId(1),
            HexCoord::new(0, 0),
        );

        assert!(state.get_army(army_id).is_some());
        assert_eq!(state.armies.len(), 1);
    }

    #[test]
    fn test_campaign_tick() {
        let map = test_map();
        let mut state = CampaignState::new(map.clone());

        let army_id = state.spawn_army(
            "Moving Army".to_string(),
            PolityId(1),
            HexCoord::new(0, 0),
        );

        state.get_army_mut(army_id).unwrap().order_move_to(HexCoord::new(2, 2), &map);

        // Run several ticks
        for _ in 0..10 {
            let events = campaign_tick(&mut state, 1.0);
            if events.iter().any(|e| matches!(e, CampaignEvent::ArmyArrived { .. })) {
                break;
            }
        }

        assert_eq!(state.get_army(army_id).unwrap().position, HexCoord::new(2, 2));
    }

    #[test]
    fn test_interception() {
        let map = test_map();
        let mut state = CampaignState::new(map);

        // Two armies from different factions at same position
        state.spawn_army("Army 1".to_string(), PolityId(1), HexCoord::new(5, 5));
        let army2_id = state.spawn_army("Army 2".to_string(), PolityId(2), HexCoord::new(5, 5));

        // Set one to aggressive
        state.get_army_mut(army2_id).unwrap().stance = ArmyStance::Aggressive;

        let interceptions = state.check_interceptions(HexCoord::new(5, 5));
        assert_eq!(interceptions.len(), 1);
    }
}
