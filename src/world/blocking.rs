//! Blocked cells for pathfinding MVP
//!
//! Provides cell-based blocking for pathfinding and movement systems.
//! Uses a HashSet-based approach for O(1) lookup of blocked cells.

use ahash::AHashSet;
use glam::Vec2;

/// State of a blocking object (for future breach/permeable handling)
#[derive(Debug, Clone, PartialEq)]
pub enum BlockingState {
    /// Completely blocks movement
    Solid,
    /// Has passable gaps (e.g., breached wall)
    Breached { gaps: Vec<Vec2> },
    /// Slows movement (cost multiplier)
    Permeable(f32),
    /// Doesn't block movement
    None,
}

impl BlockingState {
    /// Returns true if movement can pass through
    pub fn can_pass(&self) -> bool {
        match self {
            BlockingState::Solid => false,
            BlockingState::Breached { gaps } => !gaps.is_empty(),
            BlockingState::Permeable(_) => true,
            BlockingState::None => true,
        }
    }

    /// Returns the movement cost multiplier
    pub fn movement_cost(&self) -> f32 {
        match self {
            BlockingState::Solid => f32::INFINITY,
            BlockingState::Breached { .. } => 1.0,
            BlockingState::Permeable(cost) => *cost,
            BlockingState::None => 1.0,
        }
    }
}

impl Default for BlockingState {
    fn default() -> Self {
        BlockingState::None
    }
}

/// Set of blocked grid cells for pathfinding
#[derive(Debug, Clone)]
pub struct BlockedCells {
    cells: AHashSet<(i32, i32)>,
    cell_size: f32,
}

impl BlockedCells {
    /// Create a new BlockedCells with default cell size of 1.0
    pub fn new() -> Self {
        Self {
            cells: AHashSet::new(),
            cell_size: 1.0,
        }
    }

    /// Create a new BlockedCells with specified cell size
    pub fn with_cell_size(cell_size: f32) -> Self {
        Self {
            cells: AHashSet::new(),
            cell_size,
        }
    }

    /// Block a cell at grid coordinates
    pub fn block(&mut self, x: i32, y: i32) {
        self.cells.insert((x, y));
    }

    /// Unblock a cell at grid coordinates
    pub fn unblock(&mut self, x: i32, y: i32) {
        self.cells.remove(&(x, y));
    }

    /// Check if a cell at grid coordinates is blocked
    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        self.cells.contains(&(x, y))
    }

    /// Check if a world position is blocked
    pub fn is_position_blocked(&self, pos: Vec2) -> bool {
        let (cx, cy) = self.world_to_cell(pos);
        self.is_blocked(cx, cy)
    }

    /// Convert world position to cell coordinates
    pub fn world_to_cell(&self, pos: Vec2) -> (i32, i32) {
        let x = (pos.x / self.cell_size).floor() as i32;
        let y = (pos.y / self.cell_size).floor() as i32;
        (x, y)
    }

    /// Block all cells covered by a polygon footprint
    ///
    /// Uses the cell size to determine grid resolution.
    /// The cell_size parameter here is for the footprint scaling.
    pub fn block_footprint(&mut self, footprint: &[Vec2], _footprint_cell_size: f32) {
        if footprint.is_empty() {
            return;
        }

        // Find bounding box
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for p in footprint {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        // Convert to cell coordinates
        let start_x = (min_x / self.cell_size).floor() as i32;
        let start_y = (min_y / self.cell_size).floor() as i32;
        let end_x = (max_x / self.cell_size).ceil() as i32;
        let end_y = (max_y / self.cell_size).ceil() as i32;

        // Check each cell in bounding box
        for cy in start_y..end_y {
            for cx in start_x..end_x {
                // Check cell center
                let cell_center = Vec2::new(
                    (cx as f32 + 0.5) * self.cell_size,
                    (cy as f32 + 0.5) * self.cell_size,
                );

                if point_in_polygon(cell_center, footprint) {
                    self.block(cx, cy);
                }
            }
        }
    }

    /// Clear all blocked cells
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Get the number of blocked cells
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Check if there are no blocked cells
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl Default for BlockedCells {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a point is inside a polygon using ray casting algorithm
///
/// Casts a ray from the point to the right and counts intersections.
/// Odd number of intersections means the point is inside.
pub fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let mut inside = false;
    let n = polygon.len();

    let mut j = n - 1;
    for i in 0..n {
        let pi = polygon[i];
        let pj = polygon[j];

        // Check if the ray from point to the right crosses this edge
        if ((pi.y > point.y) != (pj.y > point.y))
            && (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }

        j = i;
    }

    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocked_cells_insert_remove() {
        let mut blocked = BlockedCells::new();

        blocked.block(5, 10);
        blocked.block(5, 11);

        assert!(blocked.is_blocked(5, 10));
        assert!(blocked.is_blocked(5, 11));
        assert!(!blocked.is_blocked(5, 12));

        blocked.unblock(5, 10);
        assert!(!blocked.is_blocked(5, 10));
    }

    #[test]
    fn test_blocked_cells_from_footprint() {
        let mut blocked = BlockedCells::new();

        // Rectangular footprint from (0,0) to (3,2)
        let footprint = vec![
            glam::Vec2::new(0.0, 0.0),
            glam::Vec2::new(3.0, 0.0),
            glam::Vec2::new(3.0, 2.0),
            glam::Vec2::new(0.0, 2.0),
        ];

        blocked.block_footprint(&footprint, 1.0);

        // Should block cells (0,0), (1,0), (2,0), (0,1), (1,1), (2,1)
        assert!(blocked.is_blocked(0, 0));
        assert!(blocked.is_blocked(1, 0));
        assert!(blocked.is_blocked(2, 0));
        assert!(blocked.is_blocked(0, 1));
        assert!(blocked.is_blocked(1, 1));
        assert!(blocked.is_blocked(2, 1));
        assert!(!blocked.is_blocked(3, 0)); // Outside
    }

    #[test]
    fn test_blocking_state_solid() {
        let state = BlockingState::Solid;
        assert!(!state.can_pass());
    }

    #[test]
    fn test_blocking_state_breached() {
        let state = BlockingState::Breached {
            gaps: vec![glam::Vec2::new(5.0, 5.0)],
        };
        assert!(state.can_pass()); // Has gaps
    }

    #[test]
    fn test_blocking_state_none() {
        let state = BlockingState::None;
        assert!(state.can_pass());
        assert_eq!(state.movement_cost(), 1.0);
    }

    #[test]
    fn test_blocking_state_permeable() {
        let state = BlockingState::Permeable(2.5);
        assert!(state.can_pass());
        assert_eq!(state.movement_cost(), 2.5);
    }

    #[test]
    fn test_blocking_state_default() {
        let state = BlockingState::default();
        assert!(matches!(state, BlockingState::None));
    }

    #[test]
    fn test_blocked_cells_default() {
        let blocked = BlockedCells::default();
        assert!(blocked.is_empty());
        assert_eq!(blocked.len(), 0);
    }

    #[test]
    fn test_blocked_cells_with_cell_size() {
        let blocked = BlockedCells::with_cell_size(2.0);
        assert!(blocked.is_empty());
    }

    #[test]
    fn test_world_to_cell() {
        let blocked = BlockedCells::with_cell_size(10.0);
        assert_eq!(blocked.world_to_cell(Vec2::new(5.0, 15.0)), (0, 1));
        assert_eq!(blocked.world_to_cell(Vec2::new(25.0, 35.0)), (2, 3));
        assert_eq!(blocked.world_to_cell(Vec2::new(-5.0, -15.0)), (-1, -2));
    }

    #[test]
    fn test_is_position_blocked() {
        let mut blocked = BlockedCells::with_cell_size(10.0);
        blocked.block(1, 2);

        assert!(blocked.is_position_blocked(Vec2::new(15.0, 25.0)));
        assert!(!blocked.is_position_blocked(Vec2::new(5.0, 25.0)));
    }

    #[test]
    fn test_clear() {
        let mut blocked = BlockedCells::new();
        blocked.block(1, 1);
        blocked.block(2, 2);
        assert_eq!(blocked.len(), 2);

        blocked.clear();
        assert!(blocked.is_empty());
    }

    #[test]
    fn test_point_in_polygon_square() {
        let square = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];

        assert!(point_in_polygon(Vec2::new(5.0, 5.0), &square));
        assert!(point_in_polygon(Vec2::new(1.0, 1.0), &square));
        assert!(!point_in_polygon(Vec2::new(15.0, 5.0), &square));
        assert!(!point_in_polygon(Vec2::new(-5.0, 5.0), &square));
    }

    #[test]
    fn test_point_in_polygon_triangle() {
        let triangle = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(5.0, 10.0),
        ];

        assert!(point_in_polygon(Vec2::new(5.0, 3.0), &triangle));
        assert!(!point_in_polygon(Vec2::new(0.0, 10.0), &triangle));
    }

    #[test]
    fn test_point_in_polygon_empty() {
        let empty: Vec<Vec2> = vec![];
        assert!(!point_in_polygon(Vec2::new(0.0, 0.0), &empty));
    }

    #[test]
    fn test_breached_empty_gaps() {
        let state = BlockingState::Breached { gaps: vec![] };
        assert!(!state.can_pass()); // No gaps means can't pass
    }
}
