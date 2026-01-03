//! Hex coordinate system for battle maps (axial coordinates)
//!
//! Uses axial coordinates (q, r) for easy neighbor calculation.

use serde::{Deserialize, Serialize};

/// Axial hex coordinate for battle map
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct BattleHexCoord {
    pub q: i32,
    pub r: i32,
}

impl BattleHexCoord {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Cube coordinate S (derived from q and r)
    pub fn s(&self) -> i32 {
        -self.q - self.r
    }

    /// Manhattan distance in hex space
    pub fn distance(&self, other: &Self) -> u32 {
        let dq = (self.q - other.q).abs();
        let dr = (self.r - other.r).abs();
        let ds = (self.s() - other.s()).abs();
        ((dq + dr + ds) / 2) as u32
    }

    /// Get all 6 neighboring hex coordinates
    pub fn neighbors(&self) -> [BattleHexCoord; 6] {
        [
            BattleHexCoord::new(self.q + 1, self.r),
            BattleHexCoord::new(self.q + 1, self.r - 1),
            BattleHexCoord::new(self.q, self.r - 1),
            BattleHexCoord::new(self.q - 1, self.r),
            BattleHexCoord::new(self.q - 1, self.r + 1),
            BattleHexCoord::new(self.q, self.r + 1),
        ]
    }

    /// Get hex coordinates in a line from self to other (inclusive)
    pub fn line_to(&self, other: &Self) -> Vec<BattleHexCoord> {
        let n = self.distance(other) as i32;
        if n == 0 {
            return vec![*self];
        }

        let mut results = Vec::with_capacity((n + 1) as usize);
        for i in 0..=n {
            let t = i as f32 / n as f32;
            let q = self.q as f32 + (other.q - self.q) as f32 * t;
            let r = self.r as f32 + (other.r - self.r) as f32 * t;
            results.push(Self::round(q, r));
        }
        results
    }

    /// Round floating point hex to nearest integer hex
    fn round(q: f32, r: f32) -> Self {
        let s = -q - r;
        let mut rq = q.round();
        let mut rr = r.round();
        let rs = s.round();

        let q_diff = (rq - q).abs();
        let r_diff = (rr - r).abs();
        let s_diff = (rs - s).abs();

        if q_diff > r_diff && q_diff > s_diff {
            rq = -rr - rs;
        } else if r_diff > s_diff {
            rr = -rq - rs;
        }

        Self::new(rq as i32, rr as i32)
    }

    /// Get all hexes within range (inclusive)
    pub fn hexes_in_range(&self, range: u32) -> Vec<BattleHexCoord> {
        let range = range as i32;
        let mut results = Vec::new();
        for q in -range..=range {
            for r in (-range).max(-q - range)..=range.min(-q + range) {
                results.push(BattleHexCoord::new(self.q + q, self.r + r));
            }
        }
        results
    }
}

/// Direction enum for hex facing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HexDirection {
    #[default]
    East,
    NorthEast,
    NorthWest,
    West,
    SouthWest,
    SouthEast,
}

impl HexDirection {
    /// Get the hex offset for this direction
    pub fn offset(&self) -> BattleHexCoord {
        match self {
            HexDirection::East => BattleHexCoord::new(1, 0),
            HexDirection::NorthEast => BattleHexCoord::new(1, -1),
            HexDirection::NorthWest => BattleHexCoord::new(0, -1),
            HexDirection::West => BattleHexCoord::new(-1, 0),
            HexDirection::SouthWest => BattleHexCoord::new(-1, 1),
            HexDirection::SouthEast => BattleHexCoord::new(0, 1),
        }
    }

    /// Get opposite direction
    pub fn opposite(&self) -> Self {
        match self {
            HexDirection::East => HexDirection::West,
            HexDirection::NorthEast => HexDirection::SouthWest,
            HexDirection::NorthWest => HexDirection::SouthEast,
            HexDirection::West => HexDirection::East,
            HexDirection::SouthWest => HexDirection::NorthEast,
            HexDirection::SouthEast => HexDirection::NorthWest,
        }
    }

    /// All directions
    pub fn all() -> [HexDirection; 6] {
        [
            HexDirection::East,
            HexDirection::NorthEast,
            HexDirection::NorthWest,
            HexDirection::West,
            HexDirection::SouthWest,
            HexDirection::SouthEast,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_coord_creation() {
        let coord = BattleHexCoord::new(5, 10);
        assert_eq!(coord.q, 5);
        assert_eq!(coord.r, 10);
    }

    #[test]
    fn test_hex_distance_same() {
        let a = BattleHexCoord::new(0, 0);
        assert_eq!(a.distance(&a), 0);
    }

    #[test]
    fn test_hex_distance_adjacent() {
        let a = BattleHexCoord::new(0, 0);
        let b = BattleHexCoord::new(1, 0);
        assert_eq!(a.distance(&b), 1);
    }

    #[test]
    fn test_hex_neighbors_count() {
        let coord = BattleHexCoord::new(5, 5);
        assert_eq!(coord.neighbors().len(), 6);
    }

    #[test]
    fn test_hex_line() {
        let a = BattleHexCoord::new(0, 0);
        let b = BattleHexCoord::new(3, 0);
        let line = a.line_to(&b);
        assert_eq!(line.len(), 4); // Includes start and end
    }

    #[test]
    fn test_hexes_in_range() {
        let center = BattleHexCoord::new(0, 0);
        let range_1 = center.hexes_in_range(1);
        assert_eq!(range_1.len(), 7); // Center + 6 neighbors
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(HexDirection::East.opposite(), HexDirection::West);
        assert_eq!(HexDirection::NorthEast.opposite(), HexDirection::SouthWest);
    }
}
