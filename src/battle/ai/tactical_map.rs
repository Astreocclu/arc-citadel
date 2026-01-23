//! Tactical map analysis for AI positioning decisions
//!
//! Pre-computes terrain analysis at battle start for efficient AI queries:
//! - Cover values for defensive positioning
//! - Chokepoint detection for defensive holds
//! - Elevation data for high ground advantage

use crate::battle::battle_map::BattleMap;
use crate::battle::hex::{BattleHexCoord, HexDirection};

/// Pre-computed tactical analysis of a battle map
///
/// Built once at battle start and cached for AI use. The AI commander
/// queries this to evaluate positions for defensive value, chokepoint
/// control, and high ground advantage.
pub struct TacticalMap {
    /// Cover value at each position (0.0-1.0)
    pub cover_grid: Vec<Vec<f32>>,
    /// Is this hex a chokepoint?
    pub chokepoint_grid: Vec<Vec<bool>>,
    /// Elevation at each position
    pub elevation_grid: Vec<Vec<i8>>,
    width: u32,
    height: u32,
}

impl TacticalMap {
    /// Analyze a battle map and build tactical layers
    pub fn analyze(map: &BattleMap) -> Self {
        let width = map.width;
        let height = map.height;

        // Initialize grids with default values
        let mut cover_grid = vec![vec![0.0f32; height as usize]; width as usize];
        let mut chokepoint_grid = vec![vec![false; height as usize]; width as usize];
        let mut elevation_grid = vec![vec![0i8; height as usize]; width as usize];

        // Analyze each hex
        for q in 0..width as i32 {
            for r in 0..height as i32 {
                let coord = BattleHexCoord::new(q, r);

                if let Some(hex) = map.get_hex(coord) {
                    // Store cover value
                    cover_grid[q as usize][r as usize] = hex.total_cover();

                    // Store elevation
                    elevation_grid[q as usize][r as usize] = hex.elevation;

                    // Check for chokepoint
                    chokepoint_grid[q as usize][r as usize] =
                        Self::detect_chokepoint(map, coord);
                }
            }
        }

        Self {
            cover_grid,
            chokepoint_grid,
            elevation_grid,
            width,
            height,
        }
    }

    /// Check if a hex is a chokepoint (narrow passage)
    ///
    /// A chokepoint is defined as a hex with exactly 2 passable neighbors
    /// that are in roughly opposite directions (angle_difference >= 2).
    /// This indicates a narrow passage where units must funnel through.
    fn detect_chokepoint(map: &BattleMap, coord: BattleHexCoord) -> bool {
        // First check if this hex itself is passable
        if let Some(hex) = map.get_hex(coord) {
            if hex.terrain.impassable_for_infantry() {
                return false;
            }
        } else {
            return false;
        }

        // Collect passable neighbors with their directions
        let mut passable_neighbors: Vec<HexDirection> = Vec::new();

        for (idx, neighbor) in coord.neighbors().iter().enumerate() {
            if let Some(neighbor_hex) = map.get_hex(*neighbor) {
                if !neighbor_hex.terrain.impassable_for_infantry() {
                    // Get the direction to this neighbor
                    let direction = HexDirection::all()[idx];
                    passable_neighbors.push(direction);
                }
            }
        }

        // A chokepoint has exactly 2 passable neighbors in opposite directions
        if passable_neighbors.len() != 2 {
            return false;
        }

        // Check if the two passable neighbors are roughly opposite
        // (angle_difference >= 2 means at least 120 degrees apart)
        let angle_diff = passable_neighbors[0].angle_difference(passable_neighbors[1]);
        angle_diff >= 2
    }

    /// Check if a coordinate is within bounds
    fn in_bounds(&self, coord: BattleHexCoord) -> bool {
        coord.q >= 0
            && coord.r >= 0
            && coord.q < self.width as i32
            && coord.r < self.height as i32
    }

    /// Check if a hex is a chokepoint
    pub fn is_chokepoint(&self, coord: BattleHexCoord) -> bool {
        if !self.in_bounds(coord) {
            return false;
        }
        self.chokepoint_grid[coord.q as usize][coord.r as usize]
    }

    /// Get cover value at a position
    pub fn cover_at(&self, coord: BattleHexCoord) -> f32 {
        if !self.in_bounds(coord) {
            return 0.0;
        }
        self.cover_grid[coord.q as usize][coord.r as usize]
    }

    /// Get elevation at a position
    pub fn elevation_at(&self, coord: BattleHexCoord) -> i8 {
        if !self.in_bounds(coord) {
            return 0;
        }
        self.elevation_grid[coord.q as usize][coord.r as usize]
    }

    /// Score a position for defensive value
    ///
    /// Higher scores indicate better defensive positions.
    /// Factors:
    /// - Cover (0-1): Direct damage reduction
    /// - Elevation (scaled): High ground advantage
    /// - Chokepoint bonus: Force enemy through narrow passages
    pub fn defensive_score(&self, coord: BattleHexCoord) -> f32 {
        if !self.in_bounds(coord) {
            return 0.0;
        }

        let cover = self.cover_at(coord);
        let elevation = self.elevation_at(coord);
        let is_chokepoint = self.is_chokepoint(coord);

        // Base score from cover (0-1)
        let mut score = cover;

        // Elevation bonus: +0.1 per elevation level (capped at +0.5)
        let elevation_bonus = (elevation as f32 * 0.1).clamp(0.0, 0.5);
        score += elevation_bonus;

        // Chokepoint bonus: +0.3 for defensive chokepoints
        if is_chokepoint {
            score += 0.3;
        }

        // Cap at 1.5 (can exceed 1.0 with elevation and chokepoint)
        score.min(1.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::terrain::BattleTerrain;

    #[test]
    fn test_tactical_map_creation() {
        let map = BattleMap::new(10, 10);
        let tactical = TacticalMap::analyze(&map);

        assert_eq!(tactical.width, 10);
        assert_eq!(tactical.height, 10);
    }

    #[test]
    fn test_cover_at_open_terrain() {
        let map = BattleMap::new(10, 10);
        let tactical = TacticalMap::analyze(&map);

        // Open terrain has 0 cover
        let cover = tactical.cover_at(BattleHexCoord::new(5, 5));
        assert_eq!(cover, 0.0);
    }

    #[test]
    fn test_cover_at_forest() {
        let mut map = BattleMap::new(10, 10);
        map.set_terrain(BattleHexCoord::new(5, 5), BattleTerrain::Forest);

        let tactical = TacticalMap::analyze(&map);
        let cover = tactical.cover_at(BattleHexCoord::new(5, 5));

        // Forest provides 0.5 cover
        assert_eq!(cover, 0.5);
    }

    #[test]
    fn test_elevation_at() {
        let mut map = BattleMap::new(10, 10);
        map.set_elevation(BattleHexCoord::new(5, 5), 3);

        let tactical = TacticalMap::analyze(&map);
        let elevation = tactical.elevation_at(BattleHexCoord::new(5, 5));

        assert_eq!(elevation, 3);
    }

    #[test]
    fn test_chokepoint_detection() {
        let mut map = BattleMap::new(20, 20);

        // Create a narrow passage (cliffs on both sides)
        // Leave the passage at q=10, with cliffs at q=9 and q=11
        for r in 0..20 {
            map.set_terrain(BattleHexCoord::new(9, r), BattleTerrain::Cliff);
            map.set_terrain(BattleHexCoord::new(11, r), BattleTerrain::Cliff);
        }

        // Clear the gaps at r=10 so the passage has exits
        map.set_terrain(BattleHexCoord::new(9, 10), BattleTerrain::Open);
        map.set_terrain(BattleHexCoord::new(11, 10), BattleTerrain::Open);

        let tactical = TacticalMap::analyze(&map);

        // The hex at (10, 10) should be a chokepoint because:
        // - q=9 column is mostly cliff, but (9, 10) is open
        // - q=11 column is mostly cliff, but (11, 10) is open
        // Wait, this doesn't create the expected chokepoint pattern.
        // Let me reconsider: we need a hex where only 2 neighbors are passable.

        // Actually, (10, 5) should be a chokepoint:
        // - Neighbors: (11, 5) cliff, (11, 4) cliff, (10, 4) open, (9, 5) cliff, (9, 6) cliff, (10, 6) open
        // So (10, 5) has 2 passable neighbors at (10, 4) and (10, 6) - north/south
        let is_choke = tactical.is_chokepoint(BattleHexCoord::new(10, 5));
        assert!(
            is_choke,
            "Expected (10, 5) to be a chokepoint in narrow passage"
        );
    }

    #[test]
    fn test_open_field_not_chokepoint() {
        let map = BattleMap::new(20, 20);
        let tactical = TacticalMap::analyze(&map);

        // In an open field, no hex should be a chokepoint
        let is_choke = tactical.is_chokepoint(BattleHexCoord::new(10, 10));
        assert!(
            !is_choke,
            "Open field hex should not be detected as chokepoint"
        );
    }

    #[test]
    fn test_defensive_score_open() {
        let map = BattleMap::new(10, 10);
        let tactical = TacticalMap::analyze(&map);

        // Open terrain with no elevation has minimal defensive value
        let score = tactical.defensive_score(BattleHexCoord::new(5, 5));
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_defensive_score_with_elevation() {
        let mut map = BattleMap::new(10, 10);
        map.set_elevation(BattleHexCoord::new(5, 5), 2);

        let tactical = TacticalMap::analyze(&map);
        let score = tactical.defensive_score(BattleHexCoord::new(5, 5));

        // Elevation 2 gives +0.2 bonus
        assert!((score - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_defensive_score_with_cover() {
        let mut map = BattleMap::new(10, 10);
        map.set_terrain(BattleHexCoord::new(5, 5), BattleTerrain::Forest);

        let tactical = TacticalMap::analyze(&map);
        let score = tactical.defensive_score(BattleHexCoord::new(5, 5));

        // Forest has 0.5 cover
        assert!((score - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_defensive_score_combined() {
        let mut map = BattleMap::new(10, 10);
        map.set_terrain(BattleHexCoord::new(5, 5), BattleTerrain::Forest);
        map.set_elevation(BattleHexCoord::new(5, 5), 3);

        let tactical = TacticalMap::analyze(&map);
        let score = tactical.defensive_score(BattleHexCoord::new(5, 5));

        // Forest (0.5) + Elevation 3 (0.3) = 0.8
        assert!((score - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_out_of_bounds_returns_defaults() {
        let map = BattleMap::new(10, 10);
        let tactical = TacticalMap::analyze(&map);

        let out_of_bounds = BattleHexCoord::new(100, 100);

        assert_eq!(tactical.cover_at(out_of_bounds), 0.0);
        assert_eq!(tactical.elevation_at(out_of_bounds), 0);
        assert!(!tactical.is_chokepoint(out_of_bounds));
        assert_eq!(tactical.defensive_score(out_of_bounds), 0.0);
    }

    #[test]
    fn test_negative_coords_out_of_bounds() {
        let map = BattleMap::new(10, 10);
        let tactical = TacticalMap::analyze(&map);

        let negative = BattleHexCoord::new(-1, -1);

        assert_eq!(tactical.cover_at(negative), 0.0);
        assert_eq!(tactical.elevation_at(negative), 0);
        assert!(!tactical.is_chokepoint(negative));
    }
}
