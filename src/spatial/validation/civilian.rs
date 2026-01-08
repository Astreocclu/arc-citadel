//! Civilian validation: capacity, width requirements, economic plausibility

use super::ValidationError;
use crate::spatial::geometry_schema::{HexZone, StreetSegment};

/// Minimum width for a single cart lane
const CART_LANE_MIN_WIDTH: f32 = 2.5;
/// Minimum width for cavalry charge
const CAVALRY_MIN_WIDTH: f32 = 6.0;
/// Minimum frontage per market stall
const MARKET_STALL_FRONTAGE: f32 = 1.5;
/// Minimum area per worker (m²)
const MIN_AREA_PER_WORKER: f32 = 4.0;

pub struct CivilianValidator;

impl CivilianValidator {
    /// Validate street segment civilian properties
    pub fn validate_street(street: &StreetSegment) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let width = street.dimensions.width;
        let length = street.dimensions.length;

        // Cart lanes require minimum width
        if street.civilian_properties.cart_lanes > 0 {
            let required_width = street.civilian_properties.cart_lanes as f32 * CART_LANE_MIN_WIDTH;
            if width < required_width {
                errors.push(ValidationError::InsufficientWidth {
                    component_id: street.variant_id.clone(),
                    width,
                    required: required_width,
                });
            }
        }

        // Cavalry charge requires minimum width
        if street.military_properties.cavalry_charge_viable && width < CAVALRY_MIN_WIDTH {
            errors.push(ValidationError::PhysicalImplausible {
                description: format!(
                    "Street {} claims cavalry_charge_viable but width {}m < {}m minimum",
                    street.variant_id, width, CAVALRY_MIN_WIDTH
                ),
            });
        }

        // Market stalls require sufficient length
        let max_stalls = (length / MARKET_STALL_FRONTAGE).floor() as u32;
        if street.civilian_properties.market_stall_slots > max_stalls {
            errors.push(ValidationError::CivilianCapacityExceeded {
                zone_id: street.variant_id.clone(),
                claimed: street.civilian_properties.market_stall_slots,
                max: max_stalls,
            });
        }

        // Pedestrian capacity should be reasonable for area
        let area = width * length;
        let max_pedestrians = (area / 0.5).floor() as u32; // ~0.5m² per standing person
        if street.civilian_properties.pedestrian_capacity > max_pedestrians {
            errors.push(ValidationError::CivilianCapacityExceeded {
                zone_id: street.variant_id.clone(),
                claimed: street.civilian_properties.pedestrian_capacity,
                max: max_pedestrians,
            });
        }

        errors
    }

    /// Validate hex zone worker capacity
    pub fn validate_zone_capacity(zone: &HexZone, zone_area: f32) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        let max_workers = (zone_area / MIN_AREA_PER_WORKER).floor() as u32;
        if zone.civilian_properties.worker_capacity > max_workers {
            errors.push(ValidationError::CivilianCapacityExceeded {
                zone_id: zone.id.clone(),
                claimed: zone.civilian_properties.worker_capacity,
                max: max_workers,
            });
        }

        errors
    }

    /// Calculate polygon area using shoelace formula
    pub fn polygon_area(vertices: &[[f32; 2]]) -> f32 {
        if vertices.len() < 3 {
            return 0.0;
        }

        let mut sum = 0.0;
        for i in 0..vertices.len() {
            let j = (i + 1) % vertices.len();
            sum += vertices[i][0] * vertices[j][1];
            sum -= vertices[j][0] * vertices[i][1];
        }
        (sum / 2.0).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polygon_area_rectangle() {
        // 10 x 5 rectangle
        let vertices = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 5.0], [0.0, 5.0]];
        let area = CivilianValidator::polygon_area(&vertices);
        assert!((area - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_polygon_area_triangle() {
        // Triangle with base 4, height 3 -> area = 6
        let vertices = vec![[0.0, 0.0], [4.0, 0.0], [2.0, 3.0]];
        let area = CivilianValidator::polygon_area(&vertices);
        assert!((area - 6.0).abs() < 0.01);
    }
}
