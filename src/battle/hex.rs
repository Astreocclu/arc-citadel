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

    /// Linear interpolation between two hex coordinates
    /// t should be between 0.0 and 1.0
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let q = self.q as f32 + (other.q - self.q) as f32 * t;
        let r = self.r as f32 + (other.r - self.r) as f32 * t;
        Self::round(q, r)
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

    /// Get the primary direction from this hex to another.
    /// For hexes that aren't direct neighbors, returns the direction
    /// of the dominant component of the delta.
    pub fn direction_to(&self, other: BattleHexCoord) -> HexDirection {
        let dq = other.q - self.q;
        let dr = other.r - self.r;

        // Handle same position - default to East
        if dq == 0 && dr == 0 {
            return HexDirection::East;
        }

        // Hex coordinate direction mapping based on offset() definitions:
        // East:      (1, 0)  -> dq > 0, dr == 0
        // NorthEast: (1, -1) -> dq > 0, dr < 0
        // NorthWest: (0, -1) -> dq <= 0, dr < 0
        // West:      (-1, 0) -> dq < 0, dr == 0
        // SouthWest: (-1, 1) -> dq < 0, dr > 0
        // SouthEast: (0, 1)  -> dq >= 0, dr > 0
        match (dq.signum(), dr.signum()) {
            (1, 0) => HexDirection::East,
            (1, -1) => HexDirection::NorthEast,
            (0, -1) => HexDirection::NorthWest,
            (-1, 0) => HexDirection::West,
            (-1, 1) => HexDirection::SouthWest,
            (0, 1) => HexDirection::SouthEast,
            // Mixed cases: determine dominant direction
            (1, 1) => {
                // Between East and SouthEast - use magnitude to decide
                if dq >= dr {
                    HexDirection::East
                } else {
                    HexDirection::SouthEast
                }
            }
            (-1, -1) => {
                // Between West and NorthWest - use magnitude to decide
                if dq.abs() >= dr.abs() {
                    HexDirection::West
                } else {
                    HexDirection::NorthWest
                }
            }
            _ => HexDirection::East, // Fallback (shouldn't happen with valid signum)
        }
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

    /// Convert direction to index (0-5)
    pub fn to_index(&self) -> usize {
        match self {
            HexDirection::East => 0,
            HexDirection::NorthEast => 1,
            HexDirection::NorthWest => 2,
            HexDirection::West => 3,
            HexDirection::SouthWest => 4,
            HexDirection::SouthEast => 5,
        }
    }

    /// Compute the angular difference between two directions (0-3)
    /// Returns the minimum number of 60-degree steps between directions:
    /// 0 = same direction
    /// 1 = adjacent (60 degrees)
    /// 2 = two steps (120 degrees)
    /// 3 = opposite (180 degrees)
    pub fn angle_difference(&self, other: HexDirection) -> u8 {
        let a = self.to_index() as i8;
        let b = other.to_index() as i8;
        let diff = (a - b).abs();
        // Return minimum of clockwise or counter-clockwise distance
        diff.min(6 - diff) as u8
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

    #[test]
    fn test_angle_difference_same() {
        assert_eq!(HexDirection::East.angle_difference(HexDirection::East), 0);
    }

    #[test]
    fn test_angle_difference_adjacent() {
        assert_eq!(HexDirection::East.angle_difference(HexDirection::NorthEast), 1);
        assert_eq!(HexDirection::East.angle_difference(HexDirection::SouthEast), 1);
    }

    #[test]
    fn test_angle_difference_opposite() {
        assert_eq!(HexDirection::East.angle_difference(HexDirection::West), 3);
        assert_eq!(HexDirection::NorthEast.angle_difference(HexDirection::SouthWest), 3);
    }

    #[test]
    fn test_angle_difference_two_steps() {
        assert_eq!(HexDirection::East.angle_difference(HexDirection::NorthWest), 2);
        assert_eq!(HexDirection::East.angle_difference(HexDirection::SouthWest), 2);
    }

    #[test]
    fn test_direction_to() {
        let from = BattleHexCoord::new(5, 5);

        // Test all 6 cardinal directions based on offset() definitions
        let to_east = BattleHexCoord::new(6, 5);
        let to_northeast = BattleHexCoord::new(6, 4);
        let to_northwest = BattleHexCoord::new(5, 4);
        let to_west = BattleHexCoord::new(4, 5);
        let to_southwest = BattleHexCoord::new(4, 6);
        let to_southeast = BattleHexCoord::new(5, 6);

        assert_eq!(from.direction_to(to_east), HexDirection::East);
        assert_eq!(from.direction_to(to_northeast), HexDirection::NorthEast);
        assert_eq!(from.direction_to(to_northwest), HexDirection::NorthWest);
        assert_eq!(from.direction_to(to_west), HexDirection::West);
        assert_eq!(from.direction_to(to_southwest), HexDirection::SouthWest);
        assert_eq!(from.direction_to(to_southeast), HexDirection::SouthEast);
    }

    #[test]
    fn test_direction_to_same_position() {
        let coord = BattleHexCoord::new(5, 5);
        // Same position defaults to East
        assert_eq!(coord.direction_to(coord), HexDirection::East);
    }

    #[test]
    fn test_direction_to_distant() {
        let from = BattleHexCoord::new(0, 0);

        // Test direction to distant hexes (should use dominant direction)
        let far_east = BattleHexCoord::new(10, 0);
        let far_northeast = BattleHexCoord::new(5, -5);
        let far_west = BattleHexCoord::new(-10, 0);

        assert_eq!(from.direction_to(far_east), HexDirection::East);
        assert_eq!(from.direction_to(far_northeast), HexDirection::NorthEast);
        assert_eq!(from.direction_to(far_west), HexDirection::West);
    }
}
