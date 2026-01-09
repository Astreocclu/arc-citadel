//! Hex grid coordinate system for terrain rendering.
//!
//! Uses axial coordinates (q, r) with pointy-top hexagons.

use glam::Vec2;

/// Hex size constant (distance from center to corner)
pub const HEX_SIZE: f32 = 10.0;

/// Axial hex coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Convert axial to cube coordinates for algorithms.
    pub fn to_cube(&self) -> (i32, i32, i32) {
        let x = self.q;
        let z = self.r;
        let y = -x - z;
        (x, y, z)
    }

    /// Convert hex coordinate to world position (center of hex).
    pub fn to_world(&self) -> Vec2 {
        let x = HEX_SIZE * (3.0_f32.sqrt() * self.q as f32 + 3.0_f32.sqrt() / 2.0 * self.r as f32);
        let y = HEX_SIZE * (3.0 / 2.0 * self.r as f32);
        Vec2::new(x, y)
    }

    /// Get the 6 neighbor coordinates.
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

    /// Distance to another hex (in hex steps).
    pub fn distance(&self, other: &HexCoord) -> i32 {
        let (x1, y1, z1) = self.to_cube();
        let (x2, y2, z2) = other.to_cube();
        ((x1 - x2).abs() + (y1 - y2).abs() + (z1 - z2).abs()) / 2
    }
}

/// Convert world position to nearest hex coordinate.
pub fn world_to_hex(pos: Vec2) -> HexCoord {
    let q = (3.0_f32.sqrt() / 3.0 * pos.x - 1.0 / 3.0 * pos.y) / HEX_SIZE;
    let r = (2.0 / 3.0 * pos.y) / HEX_SIZE;
    hex_round(q, r)
}

/// Round fractional hex coordinates to integer.
fn hex_round(q: f32, r: f32) -> HexCoord {
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

    HexCoord::new(rq as i32, rr as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_world_origin() {
        let hex = HexCoord::new(0, 0);
        let world = hex.to_world();
        assert!((world.x).abs() < 0.001);
        assert!((world.y).abs() < 0.001);
    }

    #[test]
    fn test_hex_distance() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(2, -1);
        assert_eq!(a.distance(&b), 2);
    }

    #[test]
    fn test_world_to_hex_roundtrip() {
        let original = HexCoord::new(3, -2);
        let world = original.to_world();
        let back = world_to_hex(world);
        assert_eq!(original, back);
    }

    #[test]
    fn test_neighbor_count() {
        let hex = HexCoord::new(0, 0);
        assert_eq!(hex.neighbors().len(), 6);
    }
}
