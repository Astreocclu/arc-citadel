//! Fog of war and visibility system for campaign layer
//!
//! Armies have limited visibility based on terrain, weather, and unit composition.
//! Unknown areas are shrouded; known but unobserved areas show stale information.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::map::{CampaignMap, CampaignTerrain, HexCoord};
use super::route::{Army, ArmyId};
use super::weather::{RegionalWeather, Weather};
use crate::core::types::PolityId;

/// Base visibility range in hexes for a standard army
pub const BASE_VISIBILITY_RANGE: i32 = 3;

/// Scout unit visibility bonus (additive)
pub const SCOUT_VISIBILITY_BONUS: i32 = 2;

/// Visibility state for a hex
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HexVisibility {
    /// Never seen - no information
    Unknown,
    /// Previously seen but not currently visible - may have stale info
    Explored,
    /// Currently visible - accurate information
    Visible,
}

impl Default for HexVisibility {
    fn default() -> Self {
        Self::Unknown
    }
}

/// What a faction knows about a hex
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexIntel {
    pub visibility: HexVisibility,
    pub last_seen_day: f32,
    /// Armies seen at this location (may be stale)
    pub known_armies: Vec<ArmyId>,
    /// Terrain is always remembered once seen
    pub known_terrain: Option<CampaignTerrain>,
    /// Settlement info
    pub known_settlement: Option<String>,
}

impl Default for HexIntel {
    fn default() -> Self {
        Self {
            visibility: HexVisibility::Unknown,
            last_seen_day: 0.0,
            known_armies: Vec::new(),
            known_terrain: None,
            known_settlement: None,
        }
    }
}

impl HexIntel {
    /// Days since last observation
    pub fn staleness(&self, current_day: f32) -> f32 {
        current_day - self.last_seen_day
    }

    /// Is this intel recent? (within 3 days)
    pub fn is_recent(&self, current_day: f32) -> bool {
        self.staleness(current_day) < 3.0
    }
}

/// Visibility data for a single faction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionVisibility {
    pub faction: PolityId,
    pub intel: HashMap<HexCoord, HexIntel>,
    /// Total hexes ever explored
    pub explored_count: u32,
}

impl FactionVisibility {
    pub fn new(faction: PolityId) -> Self {
        Self {
            faction,
            intel: HashMap::new(),
            explored_count: 0,
        }
    }

    /// Get visibility state for a hex
    pub fn get_visibility(&self, coord: &HexCoord) -> HexVisibility {
        self.intel
            .get(coord)
            .map(|i| i.visibility)
            .unwrap_or(HexVisibility::Unknown)
    }

    /// Get intel for a hex
    pub fn get_intel(&self, coord: &HexCoord) -> Option<&HexIntel> {
        self.intel.get(coord)
    }

    /// Check if a hex is currently visible
    pub fn is_visible(&self, coord: &HexCoord) -> bool {
        self.get_visibility(coord) == HexVisibility::Visible
    }

    /// Mark hexes as visible
    pub fn reveal(&mut self, coords: &[HexCoord], armies_at: &HashMap<HexCoord, Vec<ArmyId>>, map: &CampaignMap, current_day: f32) {
        for &coord in coords {
            let intel = self.intel.entry(coord).or_default();

            // First time seeing this hex
            if intel.visibility == HexVisibility::Unknown {
                self.explored_count += 1;
            }

            intel.visibility = HexVisibility::Visible;
            intel.last_seen_day = current_day;

            // Update known armies
            intel.known_armies = armies_at.get(&coord).cloned().unwrap_or_default();

            // Remember terrain
            if let Some(tile) = map.get(&coord) {
                intel.known_terrain = Some(tile.terrain);
                intel.known_settlement = tile.settlement_name.clone();
            }
        }
    }

    /// Mark hexes as no longer visible (but explored)
    pub fn hide(&mut self, coords: &[HexCoord]) {
        for coord in coords {
            if let Some(intel) = self.intel.get_mut(coord) {
                if intel.visibility == HexVisibility::Visible {
                    intel.visibility = HexVisibility::Explored;
                }
            }
        }
    }

    /// Clear all visibility (start of turn recalculation)
    pub fn clear_visibility(&mut self) {
        for intel in self.intel.values_mut() {
            if intel.visibility == HexVisibility::Visible {
                intel.visibility = HexVisibility::Explored;
            }
        }
    }
}

/// Calculate visibility range for an army
pub fn calculate_visibility_range(
    army: &Army,
    terrain: CampaignTerrain,
    weather: Weather,
    has_scouts: bool,
) -> i32 {
    let base = BASE_VISIBILITY_RANGE;

    // Terrain modifier
    let terrain_mod = terrain.visibility_modifier();

    // Weather modifier
    let weather_mod = weather.visibility_modifier();

    // Scout bonus
    let scout_bonus = if has_scouts { SCOUT_VISIBILITY_BONUS } else { 0 };

    // Hills and mountains give elevation bonus
    let elevation_bonus = match terrain {
        CampaignTerrain::Hills => 1,
        CampaignTerrain::Mountains => 2,
        _ => 0,
    };

    let effective_range = (base as f32 * terrain_mod * weather_mod) as i32 + scout_bonus + elevation_bonus;
    effective_range.max(1) // Always see at least your own hex
}

/// Get all hexes visible from a position with given range
pub fn get_visible_hexes(center: HexCoord, range: i32, map: &CampaignMap) -> Vec<HexCoord> {
    let mut visible = Vec::new();

    // Simple circular visibility (can be improved with line-of-sight)
    for q in (center.q - range)..=(center.q + range) {
        for r in (center.r - range)..=(center.r + range) {
            let coord = HexCoord::new(q, r);
            if map.contains(&coord) && center.distance(&coord) <= range {
                visible.push(coord);
            }
        }
    }

    visible
}

/// Full visibility system managing all factions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilitySystem {
    pub factions: HashMap<PolityId, FactionVisibility>,
}

impl VisibilitySystem {
    pub fn new() -> Self {
        Self {
            factions: HashMap::new(),
        }
    }

    /// Register a faction
    pub fn register_faction(&mut self, faction: PolityId) {
        if !self.factions.contains_key(&faction) {
            self.factions.insert(faction, FactionVisibility::new(faction));
        }
    }

    /// Get visibility data for a faction
    pub fn get_faction(&self, faction: PolityId) -> Option<&FactionVisibility> {
        self.factions.get(&faction)
    }

    /// Get mutable visibility data for a faction
    pub fn get_faction_mut(&mut self, faction: PolityId) -> Option<&mut FactionVisibility> {
        self.factions.get_mut(&faction)
    }

    /// Update visibility for all factions based on army positions
    pub fn update(
        &mut self,
        armies: &[Army],
        scout_armies: &HashSet<ArmyId>, // Armies with scout capability
        map: &CampaignMap,
        weather: &RegionalWeather,
        current_day: f32,
    ) {
        // Build army position map
        let mut armies_at: HashMap<HexCoord, Vec<ArmyId>> = HashMap::new();
        for army in armies {
            armies_at.entry(army.position).or_default().push(army.id);
        }

        // Clear all current visibility
        for fv in self.factions.values_mut() {
            fv.clear_visibility();
        }

        // Update visibility for each army
        for army in armies {
            let Some(fv) = self.factions.get_mut(&army.faction) else {
                continue;
            };

            // Get terrain at army position
            let terrain = map
                .get(&army.position)
                .map(|t| t.terrain)
                .unwrap_or_default();

            // Get weather at army position
            let wx = weather.get_weather_at(&army.position);

            // Check if army has scouts
            let has_scouts = scout_armies.contains(&army.id);

            // Calculate visibility range
            let range = calculate_visibility_range(army, terrain, wx, has_scouts);

            // Get visible hexes
            let visible = get_visible_hexes(army.position, range, map);

            // Reveal hexes
            fv.reveal(&visible, &armies_at, map, current_day);
        }
    }

    /// Check if one army can see another
    pub fn can_see(&self, observer: &Army, target: &Army) -> bool {
        if let Some(fv) = self.factions.get(&observer.faction) {
            fv.is_visible(&target.position)
        } else {
            false
        }
    }

    /// Get all enemy armies visible to a faction
    pub fn visible_enemies<'a>(&self, faction: PolityId, armies: &'a [Army]) -> Vec<&'a Army> {
        let Some(fv) = self.factions.get(&faction) else {
            return Vec::new();
        };

        armies
            .iter()
            .filter(|a| a.faction != faction && fv.is_visible(&a.position))
            .collect()
    }
}

impl Default for VisibilitySystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Visibility events
#[derive(Debug, Clone)]
pub enum VisibilityEvent {
    EnemySpotted { by_faction: PolityId, enemy_army: ArmyId, position: HexCoord },
    EnemyLost { by_faction: PolityId, enemy_army: ArmyId, last_known: HexCoord },
    NewTerritoryExplored { faction: PolityId, hex_count: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaign::route::ArmyId;

    #[test]
    fn test_visibility_range() {
        let army = Army::new(ArmyId(1), "Test".to_string(), PolityId(1), HexCoord::new(0, 0));

        // Clear weather, plains
        let range = calculate_visibility_range(&army, CampaignTerrain::Plains, Weather::Clear, false);
        assert_eq!(range, BASE_VISIBILITY_RANGE);

        // With scouts
        let range_scouts = calculate_visibility_range(&army, CampaignTerrain::Plains, Weather::Clear, true);
        assert_eq!(range_scouts, BASE_VISIBILITY_RANGE + SCOUT_VISIBILITY_BONUS);

        // In fog
        let range_fog = calculate_visibility_range(&army, CampaignTerrain::Plains, Weather::Fog, false);
        assert!(range_fog < range);
    }

    #[test]
    fn test_faction_visibility() {
        let mut fv = FactionVisibility::new(PolityId(1));
        let coord = HexCoord::new(5, 5);

        assert_eq!(fv.get_visibility(&coord), HexVisibility::Unknown);

        let map = CampaignMap::generate_simple(10, 10, 42);
        fv.reveal(&[coord], &HashMap::new(), &map, 0.0);

        assert_eq!(fv.get_visibility(&coord), HexVisibility::Visible);
        assert_eq!(fv.explored_count, 1);

        fv.clear_visibility();
        assert_eq!(fv.get_visibility(&coord), HexVisibility::Explored);
    }

    #[test]
    fn test_visible_hexes() {
        let map = CampaignMap::generate_simple(20, 20, 42);
        let center = HexCoord::new(10, 10);

        let visible = get_visible_hexes(center, 2, &map);

        // Should include center
        assert!(visible.contains(&center));

        // Should include adjacent
        assert!(visible.contains(&HexCoord::new(10, 11)));
        assert!(visible.contains(&HexCoord::new(11, 10)));

        // Should not include far hexes
        assert!(!visible.contains(&HexCoord::new(10, 15)));
    }

    #[test]
    fn test_visibility_system() {
        let mut system = VisibilitySystem::new();
        system.register_faction(PolityId(1));
        system.register_faction(PolityId(2));

        let map = CampaignMap::generate_simple(20, 20, 42);
        let weather = RegionalWeather::new();

        let army1 = Army::new(ArmyId(1), "Army 1".to_string(), PolityId(1), HexCoord::new(5, 5));
        let army2 = Army::new(ArmyId(2), "Army 2".to_string(), PolityId(2), HexCoord::new(5, 7));

        system.update(&[army1.clone(), army2.clone()], &HashSet::new(), &map, &weather, 0.0);

        // Army 1 should see Army 2 (within range 3)
        assert!(system.can_see(&army1, &army2));

        // Check visible enemies
        let armies = [army1, army2];
        let enemies = system.visible_enemies(PolityId(1), &armies);
        assert_eq!(enemies.len(), 1);
    }
}
