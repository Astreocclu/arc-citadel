//! Campaign map - hex-based strategic layer
//!
//! Provides the strategic map where armies move between locations.

use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::core::types::PolityId;

/// Axial hex coordinate (q, r system)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HexCoord {
    pub q: i32, // Column
    pub r: i32, // Row
}

impl HexCoord {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Get all 6 adjacent hexes
    pub fn neighbors(&self) -> [HexCoord; 6] {
        [
            HexCoord::new(self.q + 1, self.r),
            HexCoord::new(self.q + 1, self.r - 1),
            HexCoord::new(self.q, self.r - 1),
            HexCoord::new(self.q - 1, self.r),
            HexCoord::new(self.q - 1, self.r + 1),
            HexCoord::new(self.q, self.r + 1),
        ]
    }

    /// Distance in hex steps using axial coordinate formula
    pub fn distance(&self, other: &HexCoord) -> i32 {
        let dq = (self.q - other.q).abs();
        let dr = (self.r - other.r).abs();
        let ds = ((self.q + self.r) - (other.q + other.r)).abs();
        (dq + dr + ds) / 2
    }

    /// Convert to cube coordinates for certain calculations
    pub fn to_cube(&self) -> (i32, i32, i32) {
        let x = self.q;
        let z = self.r;
        let y = -x - z;
        (x, y, z)
    }
}

/// Terrain types affecting movement and visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CampaignTerrain {
    Plains,
    Forest,
    Hills,
    Mountains,
    Swamp,
    Desert,
    River, // Along hex edges - provides fast travel
    Coast,
}

impl CampaignTerrain {
    /// Movement cost in days per hex
    pub fn movement_cost(&self) -> f32 {
        match self {
            Self::Plains => 1.0,
            Self::Forest => 2.0,
            Self::Hills => 2.0,
            Self::Mountains => 4.0,
            Self::Swamp => 3.0,
            Self::Desert => 2.0,
            Self::River => 0.5, // Fast travel along rivers
            Self::Coast => 1.5,
        }
    }

    /// Visibility range modifier
    pub fn visibility_modifier(&self) -> f32 {
        match self {
            Self::Plains => 1.0,
            Self::Forest => 0.3,    // Hard to see through
            Self::Hills => 1.2,     // Can see from elevation
            Self::Mountains => 1.5, // Great vantage
            Self::Swamp => 0.5,
            Self::Desert => 1.3,
            Self::River => 1.0,
            Self::Coast => 1.0,
        }
    }

    /// Defense bonus when defending in this terrain
    pub fn defense_bonus(&self) -> f32 {
        match self {
            Self::Plains => 0.0,
            Self::Forest => 0.2,
            Self::Hills => 0.3,
            Self::Mountains => 0.5,
            Self::Swamp => 0.1,
            Self::Desert => 0.0,
            Self::River => -0.1, // Harder to defend river crossings
            Self::Coast => 0.1,
        }
    }
}

impl Default for CampaignTerrain {
    fn default() -> Self {
        Self::Plains
    }
}

/// A single hex tile on the campaign map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexTile {
    pub coord: HexCoord,
    pub terrain: CampaignTerrain,
    pub elevation: i32, // Meters, affects visibility
    pub controller: Option<PolityId>,
    pub has_settlement: bool,
    pub settlement_name: Option<String>,
}

impl HexTile {
    pub fn new(coord: HexCoord, terrain: CampaignTerrain) -> Self {
        Self {
            coord,
            terrain,
            elevation: 0,
            controller: None,
            has_settlement: false,
            settlement_name: None,
        }
    }

    pub fn with_elevation(mut self, elevation: i32) -> Self {
        self.elevation = elevation;
        self
    }

    pub fn with_settlement(mut self, name: &str) -> Self {
        self.has_settlement = true;
        self.settlement_name = Some(name.to_string());
        self
    }
}

/// The campaign map containing all hex tiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignMap {
    pub hexes: HashMap<HexCoord, HexTile>,
    pub width: i32,
    pub height: i32,
}

impl CampaignMap {
    /// Create a new empty campaign map
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            hexes: HashMap::new(),
            width,
            height,
        }
    }

    /// Generate a simple map with varied terrain
    pub fn generate_simple(width: i32, height: i32, seed: u64) -> Self {
        let mut map = Self::new(width, height);

        // Simple pseudo-random terrain generation
        for q in 0..width {
            for r in 0..height {
                let coord = HexCoord::new(q, r);
                let hash = Self::simple_hash(q, r, seed);
                let terrain = match hash % 8 {
                    0 | 1 | 2 => CampaignTerrain::Plains,
                    3 => CampaignTerrain::Forest,
                    4 => CampaignTerrain::Hills,
                    5 => CampaignTerrain::Mountains,
                    6 => CampaignTerrain::Swamp,
                    _ => CampaignTerrain::Desert,
                };
                let elevation = ((hash / 8) % 500) as i32;
                let tile = HexTile::new(coord, terrain).with_elevation(elevation);
                map.hexes.insert(coord, tile);
            }
        }

        map
    }

    fn simple_hash(q: i32, r: i32, seed: u64) -> u64 {
        let mut h = seed;
        h = h.wrapping_mul(31).wrapping_add(q as u64);
        h = h.wrapping_mul(31).wrapping_add(r as u64);
        h ^ (h >> 16)
    }

    /// Get a tile at the given coordinate
    pub fn get(&self, coord: &HexCoord) -> Option<&HexTile> {
        self.hexes.get(coord)
    }

    /// Get a mutable tile at the given coordinate
    pub fn get_mut(&mut self, coord: &HexCoord) -> Option<&mut HexTile> {
        self.hexes.get_mut(coord)
    }

    /// Check if a coordinate is within the map bounds
    pub fn contains(&self, coord: &HexCoord) -> bool {
        self.hexes.contains_key(coord)
    }

    /// Get passable neighbors of a hex
    pub fn passable_neighbors(&self, coord: &HexCoord) -> Vec<HexCoord> {
        coord
            .neighbors()
            .into_iter()
            .filter(|n| self.contains(n))
            .collect()
    }

    /// A* pathfinding from start to goal
    pub fn find_path(&self, start: HexCoord, goal: HexCoord) -> Option<Vec<HexCoord>> {
        if !self.contains(&start) || !self.contains(&goal) {
            return None;
        }

        if start == goal {
            return Some(vec![start]);
        }

        struct Node {
            coord: HexCoord,
            #[allow(dead_code)]
            g_cost: f32,
            f_cost: f32,
        }

        impl PartialEq for Node {
            fn eq(&self, other: &Self) -> bool {
                self.coord == other.coord
            }
        }
        impl Eq for Node {}

        impl Ord for Node {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                other
                    .f_cost
                    .partial_cmp(&self.f_cost)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        }
        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<HexCoord, HexCoord> = HashMap::new();
        let mut g_score: HashMap<HexCoord, f32> = HashMap::new();
        let mut closed_set: HashSet<HexCoord> = HashSet::new();

        g_score.insert(start, 0.0);
        open_set.push(Node {
            coord: start,
            g_cost: 0.0,
            f_cost: start.distance(&goal) as f32,
        });

        while let Some(current) = open_set.pop() {
            if current.coord == goal {
                // Reconstruct path
                let mut path = vec![goal];
                let mut current_coord = goal;
                while let Some(&prev) = came_from.get(&current_coord) {
                    path.push(prev);
                    current_coord = prev;
                }
                path.reverse();
                return Some(path);
            }

            if closed_set.contains(&current.coord) {
                continue;
            }
            closed_set.insert(current.coord);

            for neighbor in self.passable_neighbors(&current.coord) {
                if closed_set.contains(&neighbor) {
                    continue;
                }

                let neighbor_tile = self.get(&neighbor).unwrap();
                let movement_cost = neighbor_tile.terrain.movement_cost();
                let tentative_g = g_score.get(&current.coord).unwrap_or(&f32::INFINITY)
                    + movement_cost;

                if tentative_g < *g_score.get(&neighbor).unwrap_or(&f32::INFINITY) {
                    came_from.insert(neighbor, current.coord);
                    g_score.insert(neighbor, tentative_g);
                    let h = neighbor.distance(&goal) as f32;
                    open_set.push(Node {
                        coord: neighbor,
                        g_cost: tentative_g,
                        f_cost: tentative_g + h,
                    });
                }
            }
        }

        None // No path found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_coord_distance() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(2, 1);
        assert_eq!(a.distance(&b), 3);

        let c = HexCoord::new(0, 0);
        let d = HexCoord::new(0, 3);
        assert_eq!(c.distance(&d), 3);
    }

    #[test]
    fn test_hex_neighbors() {
        let center = HexCoord::new(0, 0);
        let neighbors = center.neighbors();
        assert_eq!(neighbors.len(), 6);

        // All neighbors should be distance 1 away
        for n in neighbors {
            assert_eq!(center.distance(&n), 1);
        }
    }

    #[test]
    fn test_terrain_movement_cost() {
        assert_eq!(CampaignTerrain::Plains.movement_cost(), 1.0);
        assert_eq!(CampaignTerrain::Mountains.movement_cost(), 4.0);
        assert_eq!(CampaignTerrain::River.movement_cost(), 0.5);
    }

    #[test]
    fn test_map_generation() {
        let map = CampaignMap::generate_simple(10, 10, 42);
        assert_eq!(map.hexes.len(), 100);

        // Check that all expected hexes exist
        for q in 0..10 {
            for r in 0..10 {
                assert!(map.contains(&HexCoord::new(q, r)));
            }
        }
    }

    #[test]
    fn test_pathfinding_simple() {
        let map = CampaignMap::generate_simple(5, 5, 42);
        let start = HexCoord::new(0, 0);
        let goal = HexCoord::new(4, 4);

        let path = map.find_path(start, goal);
        assert!(path.is_some());

        let path = path.unwrap();
        assert_eq!(path.first(), Some(&start));
        assert_eq!(path.last(), Some(&goal));
    }

    #[test]
    fn test_pathfinding_same_hex() {
        let map = CampaignMap::generate_simple(5, 5, 42);
        let hex = HexCoord::new(2, 2);

        let path = map.find_path(hex, hex);
        assert_eq!(path, Some(vec![hex]));
    }
}
