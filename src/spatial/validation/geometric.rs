//! Geometric validation: polygon validity, winding, bounds, overlaps

use super::{ValidationError, Winding};
use geo::{Intersects, LineString, Polygon};

pub struct GeometricValidator;

impl GeometricValidator {
    /// Validate a polygon represented as a list of [x, y] vertices
    pub fn validate_polygon(vertices: &[[f32; 2]]) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check minimum vertices
        if vertices.len() < 3 {
            errors.push(ValidationError::InsufficientVertices {
                count: vertices.len(),
                minimum: 3,
            });
            return errors; // Can't do further checks
        }

        // Convert to geo types
        let coords: Vec<(f64, f64)> = vertices
            .iter()
            .map(|[x, y]| (*x as f64, *y as f64))
            .collect();

        // Check for self-intersection
        if Self::is_self_intersecting(&coords) {
            errors.push(ValidationError::SelfIntersecting {
                description: "Polygon edges cross each other".into(),
            });
        }

        // Check winding order (should be CCW for exterior)
        if !Self::is_counter_clockwise(&coords) {
            errors.push(ValidationError::InvalidWinding {
                expected: Winding::CounterClockwise,
            });
        }

        errors
    }

    /// Check if polygon vertices are in counter-clockwise order
    /// Uses the shoelace formula: positive area = CCW
    fn is_counter_clockwise(coords: &[(f64, f64)]) -> bool {
        let mut sum = 0.0;
        for i in 0..coords.len() {
            let j = (i + 1) % coords.len();
            sum += (coords[j].0 - coords[i].0) * (coords[j].1 + coords[i].1);
        }
        sum < 0.0 // Negative sum = CCW in standard coords
    }

    /// Check if polygon edges intersect each other (excluding adjacent edges)
    fn is_self_intersecting(coords: &[(f64, f64)]) -> bool {
        let n = coords.len();
        if n < 4 {
            return false; // Triangle can't self-intersect
        }

        for i in 0..n {
            let a1 = coords[i];
            let a2 = coords[(i + 1) % n];

            for j in (i + 2)..n {
                // Skip adjacent edges
                if j == (i + n - 1) % n {
                    continue;
                }

                let b1 = coords[j];
                let b2 = coords[(j + 1) % n];

                if Self::segments_intersect(a1, a2, b1, b2) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if two line segments intersect (proper intersection, not touching)
    fn segments_intersect(a1: (f64, f64), a2: (f64, f64), b1: (f64, f64), b2: (f64, f64)) -> bool {
        let d1 = Self::cross_product_sign(b1, b2, a1);
        let d2 = Self::cross_product_sign(b1, b2, a2);
        let d3 = Self::cross_product_sign(a1, a2, b1);
        let d4 = Self::cross_product_sign(a1, a2, b2);

        if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
            && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
        {
            return true;
        }
        false
    }

    fn cross_product_sign(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> f64 {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    }

    /// Validate that all vertices are within bounds [0, max]
    pub fn validate_bounds(vertices: &[[f32; 2]], max_x: f32, max_y: f32) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for vertex in vertices {
            if vertex[0] < 0.0 || vertex[0] > max_x || vertex[1] < 0.0 || vertex[1] > max_y {
                errors.push(ValidationError::OutOfBounds {
                    coordinate: *vertex,
                    bounds: [max_x, max_y],
                });
            }
        }
        errors
    }

    /// Validate that two polygons don't overlap
    pub fn validate_no_overlap(
        zone1_id: &str,
        vertices1: &[[f32; 2]],
        zone2_id: &str,
        vertices2: &[[f32; 2]],
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if vertices1.len() < 3 || vertices2.len() < 3 {
            return errors; // Invalid polygons handled elsewhere
        }

        let poly1 = Self::to_geo_polygon(vertices1);
        let poly2 = Self::to_geo_polygon(vertices2);

        if poly1.intersects(&poly2) {
            errors.push(ValidationError::PolygonOverlap {
                zone1: zone1_id.to_string(),
                zone2: zone2_id.to_string(),
            });
        }

        errors
    }

    fn to_geo_polygon(vertices: &[[f32; 2]]) -> Polygon<f64> {
        let mut coords: Vec<(f64, f64)> = vertices
            .iter()
            .map(|[x, y]| (*x as f64, *y as f64))
            .collect();
        // Close the polygon
        if let Some(first) = coords.first().cloned() {
            coords.push(first);
        }
        Polygon::new(LineString::from(coords), vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ccw_detection() {
        // CCW rectangle
        let ccw = vec![(0.0, 0.0), (3.0, 0.0), (3.0, 2.0), (0.0, 2.0)];
        assert!(GeometricValidator::is_counter_clockwise(&ccw));

        // CW rectangle
        let cw = vec![(0.0, 0.0), (0.0, 2.0), (3.0, 2.0), (3.0, 0.0)];
        assert!(!GeometricValidator::is_counter_clockwise(&cw));
    }
}
