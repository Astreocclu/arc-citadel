//! Battle map with hex grid, terrain, and line of sight
//!
//! Maps are DENSE with terrain - navigation is a puzzle, not optional.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::battle::hex::BattleHexCoord;
use crate::battle::terrain::{BattleTerrain, TerrainFeature};
use crate::core::types::EntityId;

/// Visibility state for fog of war
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VisibilityState {
    #[default]
    Unknown,    // Never seen
    Remembered, // Seen before, not currently observed
    Observed,   // Currently visible
}

/// A single hex on the battle map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleHex {
    pub coord: BattleHexCoord,
    pub terrain: BattleTerrain,
    pub elevation: i8,
    pub features: Vec<TerrainFeature>,
    pub occupants: Vec<EntityId>,
    pub visibility: VisibilityState,
}

impl BattleHex {
    pub fn new(coord: BattleHexCoord, terrain: BattleTerrain) -> Self {
        Self {
            coord,
            terrain,
            elevation: 0,
            features: Vec::new(),
            occupants: Vec::new(),
            visibility: VisibilityState::Unknown,
        }
    }

    /// Total movement cost including features
    pub fn total_movement_cost(&self) -> f32 {
        let base = self.terrain.movement_cost();
        let feature_cost: f32 = self.features.iter().map(|f| f.movement_cost_modifier()).sum();
        base + feature_cost
    }

    /// Total cover value including features
    pub fn total_cover(&self) -> f32 {
        let base = self.terrain.cover_value();
        let feature_cover: f32 = self.features.iter().map(|f| f.defense_bonus()).sum();
        (base + feature_cover).min(1.0) // Cap at 1.0
    }

    /// Does this hex block line of sight?
    pub fn blocks_los(&self) -> bool {
        self.terrain.blocks_los() || self.features.iter().any(|f| f.blocks_los())
    }
}

/// Objective on the battle map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub coord: BattleHexCoord,
    pub name: String,
    pub required_for_victory: bool,
}

/// The full battle map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleMap {
    pub hexes: HashMap<BattleHexCoord, BattleHex>,
    pub width: u32,
    pub height: u32,
    pub friendly_deployment: Vec<BattleHexCoord>,
    pub enemy_deployment: Vec<BattleHexCoord>,
    pub objectives: Vec<Objective>,
}

impl BattleMap {
    /// Create a new battle map with open terrain
    pub fn new(width: u32, height: u32) -> Self {
        let mut hexes = HashMap::new();

        for q in 0..width as i32 {
            for r in 0..height as i32 {
                let coord = BattleHexCoord::new(q, r);
                hexes.insert(coord, BattleHex::new(coord, BattleTerrain::Open));
            }
        }

        Self {
            hexes,
            width,
            height,
            friendly_deployment: Vec::new(),
            enemy_deployment: Vec::new(),
            objectives: Vec::new(),
        }
    }

    /// Get a hex at the given coordinate
    pub fn get_hex(&self, coord: BattleHexCoord) -> Option<&BattleHex> {
        self.hexes.get(&coord)
    }

    /// Get a mutable hex at the given coordinate
    pub fn get_hex_mut(&mut self, coord: BattleHexCoord) -> Option<&mut BattleHex> {
        self.hexes.get_mut(&coord)
    }

    /// Check if coordinate is within map bounds
    pub fn in_bounds(&self, coord: BattleHexCoord) -> bool {
        coord.q >= 0
            && coord.r >= 0
            && coord.q < self.width as i32
            && coord.r < self.height as i32
    }

    /// Check line of sight between two hexes
    pub fn has_line_of_sight(&self, from: BattleHexCoord, to: BattleHexCoord) -> bool {
        let line = from.line_to(&to);

        // Check all hexes except start and end
        for coord in line.iter().skip(1).take(line.len().saturating_sub(2)) {
            if let Some(hex) = self.get_hex(*coord) {
                if hex.blocks_los() {
                    return false;
                }
            }
        }

        true
    }

    /// Set terrain at a coordinate
    pub fn set_terrain(&mut self, coord: BattleHexCoord, terrain: BattleTerrain) {
        if let Some(hex) = self.get_hex_mut(coord) {
            hex.terrain = terrain;
        }
    }

    /// Set elevation at a coordinate
    pub fn set_elevation(&mut self, coord: BattleHexCoord, elevation: i8) {
        if let Some(hex) = self.get_hex_mut(coord) {
            hex.elevation = elevation;
        }
    }

    /// Add a feature at a coordinate
    pub fn add_feature(&mut self, coord: BattleHexCoord, feature: TerrainFeature) {
        if let Some(hex) = self.get_hex_mut(coord) {
            if !hex.features.contains(&feature) {
                hex.features.push(feature);
            }
        }
    }

    /// Get elevation difference (positive = from is higher)
    pub fn elevation_difference(&self, from: BattleHexCoord, to: BattleHexCoord) -> i8 {
        let from_elev = self.get_hex(from).map(|h| h.elevation).unwrap_or(0);
        let to_elev = self.get_hex(to).map(|h| h.elevation).unwrap_or(0);
        from_elev - to_elev
    }

    /// Get all hexes visible from a position with given range
    pub fn visible_hexes(&self, from: BattleHexCoord, range: u32) -> Vec<BattleHexCoord> {
        from.hexes_in_range(range)
            .into_iter()
            .filter(|coord| self.in_bounds(*coord) && self.has_line_of_sight(from, *coord))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battle_hex_creation() {
        let hex = BattleHex::new(BattleHexCoord::new(0, 0), BattleTerrain::Open);
        assert_eq!(hex.terrain, BattleTerrain::Open);
        assert_eq!(hex.elevation, 0);
    }

    #[test]
    fn test_battle_map_creation() {
        let map = BattleMap::new(10, 10);
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 10);
    }

    #[test]
    fn test_battle_map_get_hex() {
        let map = BattleMap::new(10, 10);
        let hex = map.get_hex(BattleHexCoord::new(5, 5));
        assert!(hex.is_some());
    }

    #[test]
    fn test_battle_map_out_of_bounds() {
        let map = BattleMap::new(10, 10);
        let hex = map.get_hex(BattleHexCoord::new(100, 100));
        assert!(hex.is_none());
    }

    #[test]
    fn test_line_of_sight_open() {
        let map = BattleMap::new(10, 10);
        let from = BattleHexCoord::new(0, 0);
        let to = BattleHexCoord::new(5, 0);
        assert!(map.has_line_of_sight(from, to));
    }

    #[test]
    fn test_line_of_sight_blocked_by_forest() {
        let mut map = BattleMap::new(10, 10);
        map.set_terrain(BattleHexCoord::new(2, 0), BattleTerrain::Forest);

        let from = BattleHexCoord::new(0, 0);
        let to = BattleHexCoord::new(5, 0);
        assert!(!map.has_line_of_sight(from, to));
    }

    #[test]
    fn test_elevation_difference() {
        let mut map = BattleMap::new(10, 10);
        map.set_elevation(BattleHexCoord::new(0, 0), 3);
        map.set_elevation(BattleHexCoord::new(5, 5), 1);

        let diff = map.elevation_difference(BattleHexCoord::new(0, 0), BattleHexCoord::new(5, 5));
        assert_eq!(diff, 2); // 3 - 1
    }

    #[test]
    fn test_total_movement_cost_with_feature() {
        let mut hex = BattleHex::new(BattleHexCoord::new(0, 0), BattleTerrain::Open);
        let base_cost = hex.total_movement_cost();

        hex.features.push(TerrainFeature::Hill);
        let with_hill = hex.total_movement_cost();

        assert!(with_hill > base_cost);
    }
}
