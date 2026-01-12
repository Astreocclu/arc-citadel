//! Physical plausibility validation: clearances, depths, grounding

use super::ValidationError;

/// Maximum trench depth survivable without ladder
const MAX_TRENCH_DEPTH: f32 = 2.0;

pub struct PhysicalValidator;

impl PhysicalValidator {
    /// Validate platform has minimum standing clearance
    pub fn validate_platform_clearance(height: f32, min_clearance: f32) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if height < min_clearance {
            errors.push(ValidationError::PhysicalImplausible {
                description: format!(
                    "Platform height {}m < {}m minimum clearance",
                    height, min_clearance
                ),
            });
        }

        errors
    }

    /// Validate trench depth is survivable
    pub fn validate_trench_depth(depth: f32) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if depth > MAX_TRENCH_DEPTH {
            errors.push(ValidationError::PhysicalImplausible {
                description: format!(
                    "Trench depth {}m > {}m max (requires ladder)",
                    depth, MAX_TRENCH_DEPTH
                ),
            });
        }

        errors
    }

    /// Validate feature position is grounded (z = 0 or on platform)
    pub fn validate_grounded(
        position: [f32; 3],
        expected_z: f32,
        tolerance: f32,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if (position[2] - expected_z).abs() > tolerance {
            errors.push(ValidationError::PhysicalImplausible {
                description: format!(
                    "Feature at z={} should be at z={} (floating)",
                    position[2], expected_z
                ),
            });
        }

        errors
    }

    /// Validate roof height is above wall height
    pub fn validate_roof_above_wall(roof_height: f32, wall_height: f32) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if roof_height <= wall_height {
            errors.push(ValidationError::PhysicalImplausible {
                description: format!(
                    "Roof height {}m <= wall height {}m",
                    roof_height, wall_height
                ),
            });
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grounded_feature() {
        let errors = PhysicalValidator::validate_grounded([5.0, 5.0, 0.0], 0.0, 0.1);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_floating_feature() {
        let errors = PhysicalValidator::validate_grounded([5.0, 5.0, 5.0], 0.0, 0.1);
        assert!(!errors.is_empty());
    }
}
