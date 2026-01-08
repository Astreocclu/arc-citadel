//! Connection validation: wall alignment, entry points, hex connections

use super::ValidationError;
use crate::spatial::geometry_schema::ConnectionPoint;

pub struct ConnectionValidator;

impl ConnectionValidator {
    /// Validate that two connection points align within tolerance
    pub fn validate_alignment(
        p1: &ConnectionPoint,
        p2: &ConnectionPoint,
        tolerance: f32,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        let dx = (p1.position[0] - p2.position[0]).abs();
        let dy = (p1.position[1] - p2.position[1]).abs();
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > tolerance {
            errors.push(ValidationError::ConnectionMisaligned {
                point1: p1.id.clone(),
                point2: p2.id.clone(),
                distance,
            });
        }

        errors
    }

    /// Validate hex connection is at valid edge position
    pub fn validate_hex_connection_position(
        direction: &str,
        position: Option<[f32; 2]>,
        hex_size: f32,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if let Some([x, y]) = position {
            let valid = match direction {
                "north" => (y - hex_size).abs() < 0.1 && x >= 0.0 && x <= hex_size,
                "south" => y.abs() < 0.1 && x >= 0.0 && x <= hex_size,
                "east" => (x - hex_size).abs() < 0.1 && y >= 0.0 && y <= hex_size,
                "west" => x.abs() < 0.1 && y >= 0.0 && y <= hex_size,
                _ => false,
            };

            if !valid {
                errors.push(ValidationError::PhysicalImplausible {
                    description: format!(
                        "Hex connection {} at [{}, {}] not on {} edge",
                        direction, x, y, direction
                    ),
                });
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_connection_north() {
        // Position on north edge (y = 100)
        let errors = ConnectionValidator::validate_hex_connection_position(
            "north",
            Some([50.0, 100.0]),
            100.0,
        );
        assert!(errors.is_empty());
    }

    #[test]
    fn test_hex_connection_invalid() {
        // Position in middle of hex, not on edge
        let errors = ConnectionValidator::validate_hex_connection_position(
            "north",
            Some([50.0, 50.0]),
            100.0,
        );
        assert!(!errors.is_empty());
    }
}
