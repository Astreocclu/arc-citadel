//! Formation layout computation
//!
//! Converts formation lines (start/end hex) into individual unit positions.
//! Units are distributed along the line and assigned slots.

use crate::battle::hex::{BattleHexCoord, HexDirection};
use crate::battle::units::{FormationId, FormationShape, UnitId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A drawn formation line defining where units should position themselves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationLine {
    pub id: FormationLineId,
    pub formation_id: FormationId,
    /// Start point of the line (left flank)
    pub start: BattleHexCoord,
    /// End point of the line (right flank)
    pub end: BattleHexCoord,
    /// Direction units face (perpendicular to line)
    pub facing: HexDirection,
    /// Depth of formation (how many ranks deep)
    pub depth: u8,
    /// Assigned unit slots along the line
    pub slots: Vec<FormationSlot>,
}

/// Unique identifier for formation lines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FormationLineId(pub Uuid);

impl FormationLineId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for FormationLineId {
    fn default() -> Self {
        Self::new()
    }
}

/// A slot in a formation line where a unit should position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationSlot {
    pub unit_id: UnitId,
    /// Position on the line (hex coordinate)
    pub position: BattleHexCoord,
    /// Rank (0 = front line, 1 = second rank, etc.)
    pub rank: u8,
}

impl FormationLine {
    /// Create a new formation line between two hex coordinates
    pub fn new(
        formation_id: FormationId,
        start: BattleHexCoord,
        end: BattleHexCoord,
        facing: HexDirection,
    ) -> Self {
        Self {
            id: FormationLineId::new(),
            formation_id,
            start,
            end,
            facing,
            depth: 1,
            slots: Vec::new(),
        }
    }

    /// Set formation depth (number of ranks)
    pub fn with_depth(mut self, depth: u8) -> Self {
        self.depth = depth.max(1);
        self
    }

    /// Get all hexes along the formation line (front rank only)
    pub fn line_hexes(&self) -> Vec<BattleHexCoord> {
        self.start.line_to(&self.end)
    }

    /// Get the length of the formation line in hexes
    pub fn length(&self) -> usize {
        self.line_hexes().len()
    }

    /// Assign units to slots along the formation line
    ///
    /// Units are distributed evenly along the line. If there are more
    /// positions than units, they're spread out. If more units than
    /// positions, they form multiple ranks.
    pub fn assign_units(&mut self, unit_ids: &[UnitId]) {
        self.slots.clear();

        let line_hexes = self.line_hexes();
        if line_hexes.is_empty() || unit_ids.is_empty() {
            return;
        }

        let line_length = line_hexes.len();
        let total_positions = line_length * self.depth as usize;

        if unit_ids.len() <= total_positions {
            // Distribute units across available positions
            // Fill front rank first, then second rank, etc.
            for (i, unit_id) in unit_ids.iter().enumerate() {
                let rank = (i / line_length) as u8;
                let position_in_rank = i % line_length;

                // Get hex position, offset by rank
                let base_hex = line_hexes[position_in_rank];
                let position = offset_by_rank(base_hex, self.facing.opposite(), rank);

                self.slots.push(FormationSlot {
                    unit_id: *unit_id,
                    position,
                    rank,
                });
            }
        } else {
            // More units than positions - pack them in
            // This shouldn't happen in normal gameplay, but handle gracefully
            for (i, unit_id) in unit_ids.iter().enumerate() {
                let position_in_line = i % line_length;
                let rank = (i / line_length) as u8;

                let base_hex = line_hexes[position_in_line];
                let position = offset_by_rank(base_hex, self.facing.opposite(), rank);

                self.slots.push(FormationSlot {
                    unit_id: *unit_id,
                    position,
                    rank,
                });
            }
        }
    }

    /// Get the slot for a specific unit
    pub fn get_slot(&self, unit_id: UnitId) -> Option<&FormationSlot> {
        self.slots.iter().find(|s| s.unit_id == unit_id)
    }

    /// Get the target position for a unit in this formation
    pub fn get_target_position(&self, unit_id: UnitId) -> Option<BattleHexCoord> {
        self.get_slot(unit_id).map(|s| s.position)
    }

    /// Calculate facing direction based on line orientation
    ///
    /// Returns the direction perpendicular to the line,
    /// defaulting to the nearest cardinal hex direction.
    pub fn calculate_facing(&self) -> HexDirection {
        // Direction along the line
        let dq = self.end.q - self.start.q;
        let dr = self.end.r - self.start.r;

        // Perpendicular direction (rotate 90 degrees in hex space)
        // In axial coords, perpendicular to (dq, dr) is roughly (-dr, dq+dr) or similar
        // We'll snap to nearest hex direction

        // Use a simple heuristic: if line goes mainly E-W, face N or S
        // If line goes mainly NE-SW or NW-SE, face accordingly
        if dq.abs() > dr.abs() {
            // Mainly horizontal line - face north or south
            if dq > 0 {
                HexDirection::NorthWest // Facing "up" from an eastward line
            } else {
                HexDirection::SouthEast
            }
        } else if dr < 0 {
            // Line going up-right
            HexDirection::East
        } else {
            HexDirection::West
        }
    }
}

/// Offset a hex position by a number of ranks in the given direction
fn offset_by_rank(hex: BattleHexCoord, direction: HexDirection, rank: u8) -> BattleHexCoord {
    let offset = direction.offset();
    BattleHexCoord::new(hex.q + offset.q * rank as i32, hex.r + offset.r * rank as i32)
}

/// Compute positions for a formation shape relative to a center and facing
///
/// This converts the abstract FormationShape into concrete hex positions.
pub fn compute_formation_positions(
    center: BattleHexCoord,
    facing: HexDirection,
    shape: &FormationShape,
    unit_count: usize,
) -> Vec<BattleHexCoord> {
    match shape {
        FormationShape::Line { depth } => {
            compute_line_positions(center, facing, *depth as usize, unit_count)
        }
        FormationShape::Column { width } => {
            compute_column_positions(center, facing, *width as usize, unit_count)
        }
        FormationShape::Wedge { angle: _ } => {
            compute_wedge_positions(center, facing, unit_count)
        }
        FormationShape::Square => compute_square_positions(center, facing, unit_count),
        FormationShape::Skirmish { dispersion } => {
            compute_skirmish_positions(center, *dispersion, unit_count)
        }
    }
}

fn compute_line_positions(
    center: BattleHexCoord,
    facing: HexDirection,
    depth: usize,
    unit_count: usize,
) -> Vec<BattleHexCoord> {
    let mut positions = Vec::with_capacity(unit_count);

    // Get perpendicular direction for line extent
    let line_dir = perpendicular_direction(facing);
    let line_offset = line_dir.offset();

    // Calculate line width based on unit count and depth
    let units_per_rank = (unit_count + depth - 1) / depth;
    let half_width = units_per_rank as i32 / 2;

    for i in 0..unit_count {
        let rank = i / units_per_rank;
        let position_in_rank = (i % units_per_rank) as i32 - half_width;

        // Position along the line
        let line_pos = BattleHexCoord::new(
            center.q + line_offset.q * position_in_rank,
            center.r + line_offset.r * position_in_rank,
        );

        // Offset back for deeper ranks
        let rank_offset = facing.opposite().offset();
        let final_pos = BattleHexCoord::new(
            line_pos.q + rank_offset.q * rank as i32,
            line_pos.r + rank_offset.r * rank as i32,
        );

        positions.push(final_pos);
    }

    positions
}

fn compute_column_positions(
    center: BattleHexCoord,
    facing: HexDirection,
    width: usize,
    unit_count: usize,
) -> Vec<BattleHexCoord> {
    let mut positions = Vec::with_capacity(unit_count);

    let forward_offset = facing.offset();
    let side_dir = perpendicular_direction(facing);
    let side_offset = side_dir.offset();

    let half_width = width as i32 / 2;

    for i in 0..unit_count {
        let row = i / width;
        let col = (i % width) as i32 - half_width;

        let pos = BattleHexCoord::new(
            center.q + forward_offset.q * row as i32 + side_offset.q * col,
            center.r + forward_offset.r * row as i32 + side_offset.r * col,
        );

        positions.push(pos);
    }

    positions
}

fn compute_wedge_positions(
    center: BattleHexCoord,
    facing: HexDirection,
    unit_count: usize,
) -> Vec<BattleHexCoord> {
    let mut positions = Vec::with_capacity(unit_count);

    // Wedge: leader at front, units spread behind in V shape
    positions.push(center);

    let forward_offset = facing.offset();
    let left_dir = perpendicular_direction(facing);
    let right_dir = left_dir.opposite();

    let mut row = 1;
    let mut placed = 1;

    while placed < unit_count {
        // Each row has 2 more units than the previous (V expands)
        for side in 0..=row {
            if placed >= unit_count {
                break;
            }

            // Left side
            let left_offset = left_dir.offset();
            let back_offset = facing.opposite().offset();
            let left_pos = BattleHexCoord::new(
                center.q + back_offset.q * row + left_offset.q * side,
                center.r + back_offset.r * row + left_offset.r * side,
            );
            positions.push(left_pos);
            placed += 1;

            if placed >= unit_count || side == 0 {
                continue;
            }

            // Right side (mirror)
            let right_offset = right_dir.offset();
            let right_pos = BattleHexCoord::new(
                center.q + back_offset.q * row + right_offset.q * side,
                center.r + back_offset.r * row + right_offset.r * side,
            );
            positions.push(right_pos);
            placed += 1;
        }

        row += 1;
    }

    positions
}

fn compute_square_positions(
    center: BattleHexCoord,
    facing: HexDirection,
    unit_count: usize,
) -> Vec<BattleHexCoord> {
    let mut positions = Vec::with_capacity(unit_count);

    // Square: roughly equal width and depth
    let side_length = (unit_count as f32).sqrt().ceil() as usize;

    let forward_offset = facing.offset();
    let side_dir = perpendicular_direction(facing);
    let side_offset = side_dir.offset();

    let half_side = side_length as i32 / 2;

    for i in 0..unit_count {
        let row = (i / side_length) as i32 - half_side;
        let col = (i % side_length) as i32 - half_side;

        let pos = BattleHexCoord::new(
            center.q + forward_offset.q * row + side_offset.q * col,
            center.r + forward_offset.r * row + side_offset.r * col,
        );

        positions.push(pos);
    }

    positions
}

fn compute_skirmish_positions(
    center: BattleHexCoord,
    dispersion: f32,
    unit_count: usize,
) -> Vec<BattleHexCoord> {
    let mut positions = Vec::with_capacity(unit_count);

    // Skirmish: loose spiral pattern around center
    // Dispersion affects spacing (higher = more spread out)
    let spacing = (dispersion * 2.0).max(1.0) as i32;

    positions.push(center);

    let mut placed = 1;
    let mut ring = 1;

    while placed < unit_count {
        // Get hexes in ring, but skip based on dispersion
        let ring_hexes = center.hexes_in_range(ring as u32 * spacing as u32);

        for hex in ring_hexes {
            if placed >= unit_count {
                break;
            }
            // Only use hexes at roughly the right distance
            let dist = center.distance(&hex);
            if dist >= ring as u32 && dist < (ring + 1) as u32 {
                positions.push(hex);
                placed += 1;
            }
        }

        ring += 1;

        // Safety valve
        if ring > 20 {
            break;
        }
    }

    positions
}

/// Get the direction perpendicular to a facing direction
fn perpendicular_direction(facing: HexDirection) -> HexDirection {
    // Rotate 60 degrees (one hex direction) to get perpendicular
    match facing {
        HexDirection::East => HexDirection::NorthEast,
        HexDirection::NorthEast => HexDirection::NorthWest,
        HexDirection::NorthWest => HexDirection::West,
        HexDirection::West => HexDirection::SouthWest,
        HexDirection::SouthWest => HexDirection::SouthEast,
        HexDirection::SouthEast => HexDirection::East,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formation_line_hexes() {
        let line = FormationLine::new(
            FormationId::new(),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(5, 0),
            HexDirection::NorthEast,
        );

        let hexes = line.line_hexes();
        assert_eq!(hexes.len(), 6); // 0 to 5 inclusive
        assert_eq!(hexes[0], BattleHexCoord::new(0, 0));
        assert_eq!(hexes[5], BattleHexCoord::new(5, 0));
    }

    #[test]
    fn test_assign_units_single_rank() {
        let mut line = FormationLine::new(
            FormationId::new(),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(4, 0),
            HexDirection::NorthEast,
        );

        let unit_ids = vec![UnitId::new(), UnitId::new(), UnitId::new()];
        line.assign_units(&unit_ids);

        assert_eq!(line.slots.len(), 3);

        // All should be in front rank
        for slot in &line.slots {
            assert_eq!(slot.rank, 0);
        }
    }

    #[test]
    fn test_assign_units_multiple_ranks() {
        let mut line = FormationLine::new(
            FormationId::new(),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(2, 0), // 3 positions
            HexDirection::NorthEast,
        )
        .with_depth(2);

        // 5 units, 3 positions per rank = 3 front, 2 back
        let unit_ids: Vec<_> = (0..5).map(|_| UnitId::new()).collect();
        line.assign_units(&unit_ids);

        assert_eq!(line.slots.len(), 5);

        let front_rank = line.slots.iter().filter(|s| s.rank == 0).count();
        let second_rank = line.slots.iter().filter(|s| s.rank == 1).count();

        assert_eq!(front_rank, 3);
        assert_eq!(second_rank, 2);
    }

    #[test]
    fn test_get_slot() {
        let mut line = FormationLine::new(
            FormationId::new(),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(2, 0),
            HexDirection::NorthEast,
        );

        let unit_id = UnitId::new();
        line.assign_units(&[unit_id]);

        let slot = line.get_slot(unit_id);
        assert!(slot.is_some());
        assert_eq!(slot.unwrap().unit_id, unit_id);
    }

    #[test]
    fn test_get_target_position() {
        let mut line = FormationLine::new(
            FormationId::new(),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(2, 0),
            HexDirection::NorthEast,
        );

        let unit_id = UnitId::new();
        line.assign_units(&[unit_id]);

        let pos = line.get_target_position(unit_id);
        assert!(pos.is_some());
    }

    #[test]
    fn test_compute_line_formation() {
        let positions = compute_formation_positions(
            BattleHexCoord::new(5, 5),
            HexDirection::East,
            &FormationShape::Line { depth: 1 },
            5,
        );

        assert_eq!(positions.len(), 5);
        // All positions should be roughly in a line
    }

    #[test]
    fn test_compute_column_formation() {
        let positions = compute_formation_positions(
            BattleHexCoord::new(5, 5),
            HexDirection::East,
            &FormationShape::Column { width: 2 },
            6,
        );

        assert_eq!(positions.len(), 6);
    }

    #[test]
    fn test_compute_square_formation() {
        let positions = compute_formation_positions(
            BattleHexCoord::new(5, 5),
            HexDirection::East,
            &FormationShape::Square,
            9,
        );

        assert_eq!(positions.len(), 9);
    }

    #[test]
    fn test_compute_wedge_formation() {
        let positions = compute_formation_positions(
            BattleHexCoord::new(5, 5),
            HexDirection::East,
            &FormationShape::Wedge { angle: 45.0 },
            7,
        );

        assert_eq!(positions.len(), 7);
        // First position should be the center (leader at front)
        assert_eq!(positions[0], BattleHexCoord::new(5, 5));
    }

    #[test]
    fn test_compute_skirmish_formation() {
        let positions = compute_formation_positions(
            BattleHexCoord::new(5, 5),
            HexDirection::East,
            &FormationShape::Skirmish { dispersion: 1.0 },
            10,
        );

        assert_eq!(positions.len(), 10);
        // First position should be center
        assert_eq!(positions[0], BattleHexCoord::new(5, 5));
    }

    #[test]
    fn test_offset_by_rank() {
        let hex = BattleHexCoord::new(5, 5);
        let rank1 = offset_by_rank(hex, HexDirection::West, 1);
        let rank2 = offset_by_rank(hex, HexDirection::West, 2);

        // Each rank should be further west
        assert!(rank1.q < hex.q);
        assert!(rank2.q < rank1.q);
    }

    #[test]
    fn test_empty_unit_list() {
        let mut line = FormationLine::new(
            FormationId::new(),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(2, 0),
            HexDirection::East,
        );

        line.assign_units(&[]);
        assert!(line.slots.is_empty());
    }

    #[test]
    fn test_single_hex_line() {
        let mut line = FormationLine::new(
            FormationId::new(),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(0, 0), // Same start and end
            HexDirection::East,
        );

        let unit_ids = vec![UnitId::new()];
        line.assign_units(&unit_ids);

        assert_eq!(line.slots.len(), 1);
        assert_eq!(line.slots[0].position, BattleHexCoord::new(0, 0));
    }

    #[test]
    fn test_formation_line_length() {
        let line = FormationLine::new(
            FormationId::new(),
            BattleHexCoord::new(0, 0),
            BattleHexCoord::new(10, 0),
            HexDirection::NorthEast,
        );

        assert_eq!(line.length(), 11); // 0 to 10 inclusive
    }
}
