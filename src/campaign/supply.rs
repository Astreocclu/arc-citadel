//! Supply system for campaign layer
//!
//! Armies consume supplies and must forage or maintain supply lines.
//! Running out of supplies causes attrition and morale loss.

use serde::{Deserialize, Serialize};

use super::map::{CampaignMap, CampaignTerrain, HexCoord};
use super::route::{Army, ArmyId};
use crate::core::types::PolityId;

/// Days of supplies an army carries
pub const BASE_SUPPLY_DAYS: f32 = 14.0;

/// Daily supply consumption per 100 soldiers
pub const SUPPLY_CONSUMPTION_PER_100: f32 = 1.0;

/// Foraging efficiency by terrain (supplies gained per day per 100 soldiers)
pub const FORAGE_BASE_RATE: f32 = 0.3;

/// Attrition rate when out of supplies (fraction of army lost per day)
pub const STARVATION_ATTRITION_RATE: f32 = 0.02;

/// Morale loss per day when starving
pub const STARVATION_MORALE_LOSS: f32 = 0.05;

/// Supply depot storing supplies at a location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyDepot {
    pub id: DepotId,
    pub position: HexCoord,
    pub owner: PolityId,
    pub supplies: f32,         // Days worth of supplies for 100 soldiers
    pub capacity: f32,         // Maximum supplies
    pub supply_rate: f32,      // Supplies generated per day (from nearby settlements)
}

/// Unique identifier for a supply depot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DepotId(pub u32);

impl SupplyDepot {
    pub fn new(id: DepotId, position: HexCoord, owner: PolityId) -> Self {
        Self {
            id,
            position,
            owner,
            supplies: 100.0,
            capacity: 500.0,
            supply_rate: 5.0,
        }
    }

    pub fn with_capacity(mut self, capacity: f32) -> Self {
        self.capacity = capacity;
        self
    }

    pub fn with_supply_rate(mut self, rate: f32) -> Self {
        self.supply_rate = rate;
        self
    }

    /// Generate supplies for this tick
    pub fn generate_supplies(&mut self, dt_days: f32) {
        self.supplies = (self.supplies + self.supply_rate * dt_days).min(self.capacity);
    }

    /// Transfer supplies to an army
    pub fn transfer_to_army(&mut self, amount: f32) -> f32 {
        let transfer = amount.min(self.supplies);
        self.supplies -= transfer;
        transfer
    }
}

/// Supply status for an army
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmySupply {
    pub army_id: ArmyId,
    pub supplies: f32,            // Days of supplies remaining (normalized to 100 soldiers)
    pub max_supplies: f32,        // Carrying capacity
    pub foraging: bool,           // Currently foraging
    pub last_resupply: HexCoord,  // Last depot visited
}

impl ArmySupply {
    pub fn new(army_id: ArmyId) -> Self {
        Self {
            army_id,
            supplies: BASE_SUPPLY_DAYS,
            max_supplies: BASE_SUPPLY_DAYS * 1.5, // Can carry extra
            foraging: false,
            last_resupply: HexCoord::new(0, 0),
        }
    }

    /// Calculate daily supply consumption for the army
    pub fn daily_consumption(&self, unit_count: u32) -> f32 {
        (unit_count as f32 / 100.0) * SUPPLY_CONSUMPTION_PER_100
    }

    /// Consume supplies for a tick
    pub fn consume(&mut self, unit_count: u32, dt_days: f32) -> bool {
        let consumption = self.daily_consumption(unit_count) * dt_days;
        self.supplies -= consumption;
        self.supplies > 0.0
    }

    /// Check if army is starving
    pub fn is_starving(&self) -> bool {
        self.supplies <= 0.0
    }

    /// Days until starvation at current consumption rate
    pub fn days_until_starvation(&self, unit_count: u32) -> f32 {
        if self.supplies <= 0.0 {
            return 0.0;
        }
        let daily = self.daily_consumption(unit_count);
        if daily <= 0.0 {
            return f32::INFINITY;
        }
        self.supplies / daily
    }

    /// Add supplies (e.g., from depot or foraging)
    pub fn add_supplies(&mut self, amount: f32) {
        self.supplies = (self.supplies + amount).min(self.max_supplies);
    }
}

/// Calculate foraging yield for a hex
pub fn calculate_forage_yield(terrain: CampaignTerrain) -> f32 {
    let modifier = match terrain {
        CampaignTerrain::Plains => 1.0,
        CampaignTerrain::Forest => 1.2,     // Hunting, gathering
        CampaignTerrain::Hills => 0.7,
        CampaignTerrain::Mountains => 0.2,  // Very little food
        CampaignTerrain::Swamp => 0.5,      // Fish, but dangerous
        CampaignTerrain::Desert => 0.1,     // Almost nothing
        CampaignTerrain::River => 1.5,      // Fishing
        CampaignTerrain::Coast => 1.3,      // Fishing
    };
    FORAGE_BASE_RATE * modifier
}

/// Events related to supply
#[derive(Debug, Clone)]
pub enum SupplyEvent {
    ArmyResupplied { army: ArmyId, depot: DepotId, amount: f32 },
    ArmyForaged { army: ArmyId, position: HexCoord, amount: f32 },
    ArmyStarving { army: ArmyId, attrition: u32 },
    DepotCaptured { depot: DepotId, old_owner: PolityId, new_owner: PolityId },
    SuppliesExhausted { army: ArmyId },
}

/// Supply system state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplySystem {
    pub depots: Vec<SupplyDepot>,
    pub army_supplies: Vec<ArmySupply>,
    next_depot_id: u32,
}

impl SupplySystem {
    pub fn new() -> Self {
        Self {
            depots: Vec::new(),
            army_supplies: Vec::new(),
            next_depot_id: 1,
        }
    }

    /// Create a new supply depot
    pub fn create_depot(&mut self, position: HexCoord, owner: PolityId) -> DepotId {
        let id = DepotId(self.next_depot_id);
        self.next_depot_id += 1;
        self.depots.push(SupplyDepot::new(id, position, owner));
        id
    }

    /// Register an army with the supply system
    pub fn register_army(&mut self, army_id: ArmyId) {
        if !self.army_supplies.iter().any(|s| s.army_id == army_id) {
            self.army_supplies.push(ArmySupply::new(army_id));
        }
    }

    /// Get supply status for an army
    pub fn get_army_supply(&self, army_id: ArmyId) -> Option<&ArmySupply> {
        self.army_supplies.iter().find(|s| s.army_id == army_id)
    }

    /// Get mutable supply status for an army
    pub fn get_army_supply_mut(&mut self, army_id: ArmyId) -> Option<&mut ArmySupply> {
        self.army_supplies.iter_mut().find(|s| s.army_id == army_id)
    }

    /// Get depot at position owned by faction
    pub fn get_depot_at(&self, position: HexCoord, faction: PolityId) -> Option<&SupplyDepot> {
        self.depots.iter()
            .find(|d| d.position == position && d.owner == faction)
    }

    /// Get mutable depot by ID
    pub fn get_depot_mut(&mut self, id: DepotId) -> Option<&mut SupplyDepot> {
        self.depots.iter_mut().find(|d| d.id == id)
    }

    /// Process supply tick for all armies
    pub fn tick(
        &mut self,
        armies: &mut [Army],
        map: &CampaignMap,
        dt_days: f32,
    ) -> Vec<SupplyEvent> {
        let mut events = Vec::new();

        // Generate supplies at depots
        for depot in &mut self.depots {
            depot.generate_supplies(dt_days);
        }

        // Collect army data needed for processing to avoid borrow conflicts
        let army_data: Vec<_> = armies
            .iter()
            .map(|a| (a.id, a.position, a.faction, a.unit_count))
            .collect();

        // Process resupply from depots first (separate pass)
        for (army_id, position, faction, _unit_count) in &army_data {
            // Find depot at army position owned by same faction
            let depot_transfer: Option<(DepotId, f32, f32)> = self.depots
                .iter()
                .find(|d| d.position == *position && d.owner == *faction)
                .and_then(|d| {
                    let supply_idx = self.army_supplies.iter().position(|s| s.army_id == *army_id)?;
                    let needed = self.army_supplies[supply_idx].max_supplies - self.army_supplies[supply_idx].supplies;
                    if needed > 0.0 {
                        Some((d.id, needed, d.supplies))
                    } else {
                        None
                    }
                });

            if let Some((depot_id, needed, available)) = depot_transfer {
                // Transfer supplies from depot to army
                if let Some(depot) = self.depots.iter_mut().find(|d| d.id == depot_id) {
                    let transferred = depot.transfer_to_army(needed.min(available));
                    if transferred > 0.0 {
                        if let Some(supply) = self.get_army_supply_mut(*army_id) {
                            supply.add_supplies(transferred);
                            supply.last_resupply = *position;
                        }
                        events.push(SupplyEvent::ArmyResupplied {
                            army: *army_id,
                            depot: depot_id,
                            amount: transferred,
                        });
                    }
                }
            }
        }

        // Process foraging, consumption, and starvation for each army
        for army in armies.iter_mut() {
            let Some(supply) = self.get_army_supply_mut(army.id) else {
                continue;
            };

            // Forage if enabled
            if supply.foraging {
                if let Some(tile) = map.get(&army.position) {
                    let forage_yield = calculate_forage_yield(tile.terrain) * dt_days;
                    let effective_yield = forage_yield * (army.unit_count as f32 / 100.0);
                    supply.add_supplies(effective_yield);
                    events.push(SupplyEvent::ArmyForaged {
                        army: army.id,
                        position: army.position,
                        amount: effective_yield,
                    });
                }
            }

            // Consume supplies
            let had_supplies = supply.supplies > 0.0;
            supply.consume(army.unit_count, dt_days);

            // Check for starvation
            if supply.is_starving() {
                if had_supplies {
                    events.push(SupplyEvent::SuppliesExhausted { army: army.id });
                }

                // Apply attrition
                let attrition = (army.unit_count as f32 * STARVATION_ATTRITION_RATE * dt_days) as u32;
                if attrition > 0 {
                    army.unit_count = army.unit_count.saturating_sub(attrition);
                    army.morale = (army.morale - STARVATION_MORALE_LOSS * dt_days).max(0.0);
                    events.push(SupplyEvent::ArmyStarving {
                        army: army.id,
                        attrition,
                    });
                }
            }
        }

        events
    }
}

impl Default for SupplySystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaign::route::ArmyId;

    #[test]
    fn test_supply_consumption() {
        let mut supply = ArmySupply::new(ArmyId(1));
        supply.supplies = 10.0;

        // 500 soldiers consume 5x the base rate
        let has_supplies = supply.consume(500, 1.0);
        assert!(has_supplies);
        assert!((supply.supplies - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_days_until_starvation() {
        let mut supply = ArmySupply::new(ArmyId(1));
        supply.supplies = 10.0;

        // 100 soldiers = 10 days
        assert!((supply.days_until_starvation(100) - 10.0).abs() < 0.01);

        // 500 soldiers = 2 days
        assert!((supply.days_until_starvation(500) - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_forage_yield() {
        let forest = calculate_forage_yield(CampaignTerrain::Forest);
        let desert = calculate_forage_yield(CampaignTerrain::Desert);
        let river = calculate_forage_yield(CampaignTerrain::River);

        assert!(forest > desert);
        assert!(river > forest);
    }

    #[test]
    fn test_depot_transfer() {
        let mut depot = SupplyDepot::new(DepotId(1), HexCoord::new(0, 0), PolityId(1));
        depot.supplies = 50.0;

        let transferred = depot.transfer_to_army(30.0);
        assert!((transferred - 30.0).abs() < 0.01);
        assert!((depot.supplies - 20.0).abs() < 0.01);

        // Try to transfer more than available
        let transferred = depot.transfer_to_army(100.0);
        assert!((transferred - 20.0).abs() < 0.01);
        assert!((depot.supplies - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_supply_system_tick() {
        let map = CampaignMap::generate_simple(10, 10, 42);
        let mut system = SupplySystem::new();

        // Create a depot
        let depot_id = system.create_depot(HexCoord::new(5, 5), PolityId(1));

        // Create an army at depot location
        let mut army = Army::new(
            ArmyId(1),
            "Test Army".to_string(),
            PolityId(1),
            HexCoord::new(5, 5),
        );
        army.unit_count = 100;

        system.register_army(army.id);

        // Run tick
        let events = system.tick(&mut [army], &map, 1.0);

        // Should have resupplied
        assert!(events.iter().any(|e| matches!(e, SupplyEvent::ArmyResupplied { .. })));
    }
}
