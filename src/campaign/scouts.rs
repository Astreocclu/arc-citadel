//! Scout units for campaign layer
//!
//! Scouts are small, fast units that provide extended visibility,
//! reconnaissance, and early warning of enemy movements.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::map::{CampaignMap, CampaignTerrain, HexCoord};
use super::route::{Army, ArmyId};
use super::weather::Weather;
use crate::core::types::PolityId;

/// Scout movement speed multiplier (faster than regular armies)
pub const SCOUT_SPEED_MULTIPLIER: f32 = 1.5;

/// Scout visibility bonus in hexes
pub const SCOUT_VISIBILITY_BONUS: i32 = 2;

/// Scout detection range (can spot hidden enemies)
pub const SCOUT_DETECTION_RANGE: i32 = 4;

/// Chance to evade combat when spotted (0.0 - 1.0)
pub const SCOUT_EVASION_CHANCE: f32 = 0.7;

/// Unique identifier for a scout unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScoutId(pub u32);

/// Scout mission type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoutMission {
    /// Scout a specific location
    Recon(HexCoord),
    /// Shadow an enemy army
    Shadow(ArmyId),
    /// Patrol a route repeatedly
    Patrol,
    /// Return to parent army
    Return,
    /// Idle - awaiting orders
    Idle,
}

/// A scout unit on the campaign map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scout {
    pub id: ScoutId,
    pub parent_army: ArmyId,
    pub faction: PolityId,
    pub position: HexCoord,
    pub mission: ScoutMission,
    pub movement_points: f32,
    pub path_cache: Option<Vec<HexCoord>>,
    pub intel_gathered: Vec<ScoutIntel>,
    pub hidden: bool, // Trying to stay undetected
}

/// Intelligence gathered by a scout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoutIntel {
    pub position: HexCoord,
    pub day_gathered: f32,
    pub intel_type: IntelType,
}

/// Types of intelligence scouts can gather
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntelType {
    /// Enemy army spotted
    EnemyArmy {
        army_id: ArmyId,
        estimated_size: u32,
        stance: String,
        heading: Option<HexCoord>,
    },
    /// Terrain feature discovered
    TerrainFeature {
        terrain: CampaignTerrain,
        has_settlement: bool,
    },
    /// Supply depot found
    SupplyDepot {
        owner: PolityId,
    },
    /// Path is blocked or impassable
    BlockedPath,
}

impl Scout {
    pub fn new(id: ScoutId, parent_army: ArmyId, faction: PolityId, position: HexCoord) -> Self {
        Self {
            id,
            parent_army,
            faction,
            position,
            mission: ScoutMission::Idle,
            movement_points: 0.0,
            path_cache: None,
            intel_gathered: Vec::new(),
            hidden: true,
        }
    }

    /// Assign a reconnaissance mission
    pub fn assign_recon(&mut self, target: HexCoord, map: &CampaignMap) {
        self.mission = ScoutMission::Recon(target);
        self.path_cache = map.find_path(self.position, target);
    }

    /// Assign a shadow mission (follow enemy army)
    pub fn assign_shadow(&mut self, target: ArmyId) {
        self.mission = ScoutMission::Shadow(target);
        self.path_cache = None; // Will be recalculated based on target position
    }

    /// Order return to parent army
    pub fn order_return(&mut self) {
        self.mission = ScoutMission::Return;
        self.path_cache = None;
    }

    /// Calculate movement cost (scouts are faster)
    pub fn movement_cost_to(&self, map: &CampaignMap, to: &HexCoord) -> f32 {
        let Some(tile) = map.get(to) else {
            return f32::INFINITY;
        };

        let base_cost = tile.terrain.movement_cost();

        // Scouts move faster
        base_cost / SCOUT_SPEED_MULTIPLIER
    }

    /// Execute movement for this tick
    pub fn execute_movement(&mut self, map: &CampaignMap, dt_days: f32) -> bool {
        let destination = match &self.mission {
            ScoutMission::Recon(target) => Some(*target),
            ScoutMission::Return => {
                // For Return, destination is at end of cached path (set by tick)
                self.path_cache.as_ref().and_then(|p| p.last().copied())
            }
            ScoutMission::Idle => return false,
            ScoutMission::Shadow(_) => None, // Handled separately
            ScoutMission::Patrol => None, // Handled separately
        };

        let Some(dest) = destination else {
            return false;
        };

        if self.position == dest {
            return true; // Arrived
        }

        // Ensure path exists
        if self.path_cache.is_none() {
            self.path_cache = map.find_path(self.position, dest);
        }

        let Some(ref path) = self.path_cache else {
            return false; // No path
        };

        // Find next hex
        let current_idx = path.iter().position(|&h| h == self.position).unwrap_or(0);
        let Some(&next_hex) = path.get(current_idx + 1) else {
            return true; // At end of path
        };

        // Accumulate movement
        self.movement_points += dt_days;

        let cost = self.movement_cost_to(map, &next_hex);
        if self.movement_points >= cost {
            self.movement_points -= cost;
            self.position = next_hex;

            if self.position == dest {
                return true; // Arrived
            }
        }

        false
    }

    /// Gather intel at current position
    pub fn gather_intel(&mut self, map: &CampaignMap, enemies: &[&Army], current_day: f32) {
        // Record terrain
        if let Some(tile) = map.get(&self.position) {
            self.intel_gathered.push(ScoutIntel {
                position: self.position,
                day_gathered: current_day,
                intel_type: IntelType::TerrainFeature {
                    terrain: tile.terrain,
                    has_settlement: tile.has_settlement,
                },
            });
        }

        // Spot nearby enemies
        for enemy in enemies {
            let distance = self.position.distance(&enemy.position);
            if distance <= SCOUT_DETECTION_RANGE {
                // Estimate size (scouts don't get exact count)
                let estimated = estimate_army_size(enemy.unit_count);

                self.intel_gathered.push(ScoutIntel {
                    position: enemy.position,
                    day_gathered: current_day,
                    intel_type: IntelType::EnemyArmy {
                        army_id: enemy.id,
                        estimated_size: estimated,
                        stance: format!("{:?}", enemy.stance),
                        heading: enemy.orders.as_ref().and_then(|o| match o {
                            super::route::ArmyOrder::MoveTo(dest) => Some(*dest),
                            _ => None,
                        }),
                    },
                });
            }
        }
    }

    /// Check if scout evades detection
    pub fn attempt_evasion(&self, seed: u64) -> bool {
        let roll = simple_hash(seed) % 100;
        (roll as f32 / 100.0) < SCOUT_EVASION_CHANCE
    }

    /// Get visibility range for this scout
    pub fn visibility_range(&self, terrain: CampaignTerrain, weather: Weather) -> i32 {
        let base = super::visibility::BASE_VISIBILITY_RANGE + SCOUT_VISIBILITY_BONUS;

        let terrain_mod = terrain.visibility_modifier();
        let weather_mod = weather.visibility_modifier();

        ((base as f32) * terrain_mod * weather_mod) as i32
    }
}

/// Estimate army size (scouts give rough estimates)
fn estimate_army_size(actual: u32) -> u32 {
    // Round to nearest 50, with some variance
    let rounded = ((actual + 25) / 50) * 50;
    rounded.max(50)
}

fn simple_hash(seed: u64) -> u64 {
    let mut h = seed;
    h = h.wrapping_mul(6364136223846793005);
    h = h.wrapping_add(1442695040888963407);
    h ^ (h >> 32)
}

/// Scout system managing all scouts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoutSystem {
    pub scouts: Vec<Scout>,
    next_scout_id: u32,
}

impl ScoutSystem {
    pub fn new() -> Self {
        Self {
            scouts: Vec::new(),
            next_scout_id: 1,
        }
    }

    /// Deploy a scout from an army
    pub fn deploy_scout(&mut self, army: &Army) -> ScoutId {
        let id = ScoutId(self.next_scout_id);
        self.next_scout_id += 1;

        let scout = Scout::new(id, army.id, army.faction, army.position);
        self.scouts.push(scout);
        id
    }

    /// Get a scout by ID
    pub fn get_scout(&self, id: ScoutId) -> Option<&Scout> {
        self.scouts.iter().find(|s| s.id == id)
    }

    /// Get a mutable scout by ID
    pub fn get_scout_mut(&mut self, id: ScoutId) -> Option<&mut Scout> {
        self.scouts.iter_mut().find(|s| s.id == id)
    }

    /// Get all scouts for a faction
    pub fn faction_scouts(&self, faction: PolityId) -> Vec<&Scout> {
        self.scouts.iter().filter(|s| s.faction == faction).collect()
    }

    /// Get scouts attached to an army
    pub fn army_scouts(&self, army_id: ArmyId) -> Vec<&Scout> {
        self.scouts.iter().filter(|s| s.parent_army == army_id).collect()
    }

    /// Get set of army IDs that have scouts
    pub fn armies_with_scouts(&self) -> HashSet<ArmyId> {
        self.scouts.iter().map(|s| s.parent_army).collect()
    }

    /// Process scout movement and intel gathering
    pub fn tick(
        &mut self,
        armies: &[Army],
        map: &CampaignMap,
        dt_days: f32,
        current_day: f32,
    ) -> Vec<ScoutEvent> {
        let mut events = Vec::new();

        for scout in &mut self.scouts {
            // Get enemies visible to this scout
            let enemies: Vec<&Army> = armies
                .iter()
                .filter(|a| a.faction != scout.faction)
                .collect();

            // Handle Return mission - update path to parent army
            if matches!(scout.mission, ScoutMission::Return) {
                if let Some(parent) = armies.iter().find(|a| a.id == scout.parent_army) {
                    if scout.path_cache.is_none() || scout.path_cache.as_ref().map(|p| p.last() != Some(&parent.position)).unwrap_or(true) {
                        scout.path_cache = map.find_path(scout.position, parent.position);
                    }
                }
            }

            // Execute movement
            let arrived = scout.execute_movement(map, dt_days);

            if arrived {
                match scout.mission {
                    ScoutMission::Recon(target) => {
                        events.push(ScoutEvent::ReconComplete {
                            scout: scout.id,
                            position: target,
                        });
                        // Auto-return to parent army after completing recon
                        scout.order_return();
                    }
                    ScoutMission::Return => {
                        events.push(ScoutEvent::IntelDelivered {
                            scout: scout.id,
                            to_army: scout.parent_army,
                        });
                        scout.mission = ScoutMission::Idle;
                    }
                    _ => {}
                }
            }

            // Gather intel
            scout.gather_intel(map, &enemies, current_day);

            // Check for detection by enemies
            for enemy in &enemies {
                if scout.position == enemy.position {
                    if scout.hidden {
                        let seed = (current_day as u64).wrapping_mul(scout.id.0 as u64);
                        if scout.attempt_evasion(seed) {
                            events.push(ScoutEvent::EvadedDetection {
                                scout: scout.id,
                                enemy: enemy.id,
                            });
                        } else {
                            scout.hidden = false;
                            events.push(ScoutEvent::Detected {
                                scout: scout.id,
                                by_army: enemy.id,
                            });
                        }
                    }
                }
            }
        }

        events
    }

    /// Recall all scouts to their parent army
    pub fn recall_all(&mut self, army_id: ArmyId) {
        for scout in &mut self.scouts {
            if scout.parent_army == army_id {
                scout.order_return();
            }
        }
    }

    /// Remove scouts that have returned to their army
    pub fn collect_returned(&mut self, armies: &[Army]) -> Vec<(ArmyId, Vec<ScoutIntel>)> {
        let mut collected = Vec::new();

        self.scouts.retain(|scout| {
            if matches!(scout.mission, ScoutMission::Return) {
                // Check if at parent army position
                if let Some(army) = armies.iter().find(|a| a.id == scout.parent_army) {
                    if scout.position == army.position {
                        // Collect intel and remove scout
                        collected.push((army.id, scout.intel_gathered.clone()));
                        return false; // Remove from list
                    }
                }
            }
            true // Keep
        });

        collected
    }
}

impl Default for ScoutSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Scout events for campaign log
#[derive(Debug, Clone)]
pub enum ScoutEvent {
    ScoutDeployed { scout: ScoutId, from_army: ArmyId },
    ReconComplete { scout: ScoutId, position: HexCoord },
    EnemySpotted { scout: ScoutId, enemy: ArmyId, position: HexCoord },
    Detected { scout: ScoutId, by_army: ArmyId },
    EvadedDetection { scout: ScoutId, enemy: ArmyId },
    ScoutLost { scout: ScoutId },
    IntelDelivered { scout: ScoutId, to_army: ArmyId },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaign::route::ArmyStance;

    fn test_army(id: u32, pos: HexCoord) -> Army {
        Army::new(ArmyId(id), format!("Army {}", id), PolityId(id), pos)
    }

    #[test]
    fn test_scout_creation() {
        let army = test_army(1, HexCoord::new(5, 5));
        let scout = Scout::new(ScoutId(1), army.id, army.faction, army.position);

        assert_eq!(scout.position, HexCoord::new(5, 5));
        assert!(scout.hidden);
        assert!(matches!(scout.mission, ScoutMission::Idle));
    }

    #[test]
    fn test_scout_speed() {
        let map = CampaignMap::generate_simple(10, 10, 42);
        let army = test_army(1, HexCoord::new(5, 5));
        let scout = Scout::new(ScoutId(1), army.id, army.faction, army.position);

        let scout_cost = scout.movement_cost_to(&map, &HexCoord::new(5, 6));
        let army_cost = army.movement_cost_to(&map, &HexCoord::new(5, 6));

        // Scout should be faster
        assert!(scout_cost < army_cost);
    }

    #[test]
    fn test_scout_system() {
        let mut system = ScoutSystem::new();
        let army = test_army(1, HexCoord::new(5, 5));

        let scout_id = system.deploy_scout(&army);
        assert!(system.get_scout(scout_id).is_some());

        let army_scouts = system.army_scouts(army.id);
        assert_eq!(army_scouts.len(), 1);
    }

    #[test]
    fn test_estimate_size() {
        assert_eq!(estimate_army_size(100), 100);
        assert_eq!(estimate_army_size(123), 100); // 123+25=148, /50=2, *50=100
        assert_eq!(estimate_army_size(150), 150); // 150+25=175, /50=3, *50=150
        assert_eq!(estimate_army_size(10), 50); // Minimum
    }

    #[test]
    fn test_scout_intel_gathering() {
        let map = CampaignMap::generate_simple(10, 10, 42);
        let mut scout = Scout::new(ScoutId(1), ArmyId(1), PolityId(1), HexCoord::new(5, 5));

        let enemy = test_army(2, HexCoord::new(5, 6));

        scout.gather_intel(&map, &[&enemy], 0.0);

        // Should have terrain intel and enemy intel
        assert!(scout.intel_gathered.len() >= 2);
    }
}
