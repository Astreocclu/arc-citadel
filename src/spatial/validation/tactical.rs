//! Tactical validation: firing arcs, cover positions, chokepoints

use super::ValidationError;
use crate::spatial::geometry_schema::FiringPosition;

pub struct TacticalValidator;

impl TacticalValidator {
    /// Validate that firing positions cover 360 degrees without gaps and each arc <= 180 degrees
    pub fn validate_firing_arcs(positions: &[FiringPosition]) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if positions.is_empty() {
            errors.push(ValidationError::FiringArcGap {
                missing_degrees: 360.0,
            });
            return errors;
        }

        // Check individual arc widths
        for pos in positions {
            if pos.firing_arc.arc_width > 180.0 {
                errors.push(ValidationError::ArcTooWide {
                    position_id: pos.id.clone(),
                    width: pos.firing_arc.arc_width,
                    max: 180.0,
                });
            }
        }

        // Convert arcs to intervals and check coverage
        let intervals = Self::arcs_to_intervals(positions);
        let merged = Self::merge_intervals(intervals);
        let total_coverage: f32 = merged.iter().map(|(s, e)| e - s).sum();

        if (total_coverage - 360.0).abs() > 1.0 {
            errors.push(ValidationError::FiringArcGap {
                missing_degrees: 360.0 - total_coverage,
            });
        }

        errors
    }

    /// Convert firing arcs to [start, end] intervals in [0, 360)
    fn arcs_to_intervals(positions: &[FiringPosition]) -> Vec<(f32, f32)> {
        let mut intervals = Vec::new();

        for pos in positions {
            let half_width = pos.firing_arc.arc_width / 2.0;
            let center = pos.firing_arc.center_angle;

            let start = (center - half_width).rem_euclid(360.0);
            let end = (center + half_width).rem_euclid(360.0);

            if start > end {
                // Wraps around 0
                intervals.push((start, 360.0));
                intervals.push((0.0, end));
            } else {
                intervals.push((start, end));
            }
        }

        intervals
    }

    /// Merge overlapping intervals
    fn merge_intervals(mut intervals: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
        if intervals.is_empty() {
            return vec![];
        }

        intervals.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut merged: Vec<(f32, f32)> = vec![intervals[0]];

        for (start, end) in intervals.into_iter().skip(1) {
            let last = merged.last_mut().unwrap();
            if start <= last.1 {
                // Overlapping or adjacent - extend
                last.1 = last.1.max(end);
            } else {
                merged.push((start, end));
            }
        }

        merged
    }

    /// Validate chokepoint width (should be < 8m for tactical advantage)
    pub fn validate_chokepoint_width(
        width: f32,
        is_marked_chokepoint: bool,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if is_marked_chokepoint && width >= 8.0 {
            errors.push(ValidationError::PhysicalImplausible {
                description: format!("Marked as chokepoint but width {}m >= 8m", width),
            });
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::geometry_schema::{CoverLevel, FiringArc};

    #[test]
    fn test_interval_merging() {
        let intervals = vec![(0.0, 90.0), (90.0, 180.0), (180.0, 270.0), (270.0, 360.0)];
        let merged = TacticalValidator::merge_intervals(intervals);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], (0.0, 360.0));
    }

    #[test]
    fn test_wraparound_arc() {
        // Arc centered at 0 (north) with 90 width: covers 315-45
        let intervals = TacticalValidator::arcs_to_intervals(&[FiringPosition {
            id: "north".into(),
            position: [0.0, 0.0, 0.0],
            firing_arc: FiringArc {
                center_angle: 0.0,
                arc_width: 90.0,
            },
            elevation: 8.0,
            cover_value: CoverLevel::Full,
            capacity: 1,
        }]);
        // Should produce two intervals: [315, 360) and [0, 45)
        assert_eq!(intervals.len(), 2);
    }
}
