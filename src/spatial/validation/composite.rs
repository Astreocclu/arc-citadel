//! Composite validator that runs all validation checks

use super::{
    CivilianValidator, ConnectionValidator, GeometricValidator, PhysicalValidator,
    TacticalValidator, ValidationError,
};
use crate::spatial::geometry_schema::{Component, HexLayout};

/// Result of running all validators
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub passed_geometric: bool,
    pub passed_tactical: bool,
    pub passed_connection: bool,
    pub passed_physical: bool,
    pub passed_civilian: bool,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            passed_geometric: true,
            passed_tactical: true,
            passed_connection: true,
            passed_physical: true,
            passed_civilian: true,
        }
    }

    pub fn add_geometric_errors(&mut self, errors: Vec<ValidationError>) {
        if !errors.is_empty() {
            self.passed_geometric = false;
            self.is_valid = false;
            self.errors.extend(errors);
        }
    }

    pub fn add_tactical_errors(&mut self, errors: Vec<ValidationError>) {
        if !errors.is_empty() {
            self.passed_tactical = false;
            self.is_valid = false;
            self.errors.extend(errors);
        }
    }

    pub fn add_connection_errors(&mut self, errors: Vec<ValidationError>) {
        if !errors.is_empty() {
            self.passed_connection = false;
            self.is_valid = false;
            self.errors.extend(errors);
        }
    }

    pub fn add_physical_errors(&mut self, errors: Vec<ValidationError>) {
        if !errors.is_empty() {
            self.passed_physical = false;
            self.is_valid = false;
            self.errors.extend(errors);
        }
    }

    pub fn add_civilian_errors(&mut self, errors: Vec<ValidationError>) {
        if !errors.is_empty() {
            self.passed_civilian = false;
            self.is_valid = false;
            self.errors.extend(errors);
        }
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CompositeValidator;

impl CompositeValidator {
    /// Validate a single component
    pub fn validate_component(component: &Component) -> ValidationReport {
        let mut report = ValidationReport::new();

        match component {
            Component::WallSegment(wall) => {
                // Geometric: footprint polygon
                report.add_geometric_errors(GeometricValidator::validate_polygon(
                    &wall.footprint.vertices,
                ));

                // Physical: wall height plausible
                if wall.dimensions.height > 10.0 {
                    report.add_physical_errors(vec![ValidationError::PhysicalImplausible {
                        description: format!("Wall height {}m > 10m max", wall.dimensions.height),
                    }]);
                }
            }

            Component::ArcherTower(tower) => {
                // Geometric: footprint polygon
                report.add_geometric_errors(GeometricValidator::validate_polygon(
                    &tower.footprint.vertices,
                ));

                // Tactical: firing arcs sum to 360
                report.add_tactical_errors(TacticalValidator::validate_firing_arcs(
                    &tower.firing_positions,
                ));

                // Physical: platform clearance
                report.add_physical_errors(PhysicalValidator::validate_platform_clearance(
                    tower.dimensions.platform_height,
                    2.0,
                ));
            }

            Component::TrenchSegment(trench) => {
                // Geometric: footprint polygon
                report.add_geometric_errors(GeometricValidator::validate_polygon(
                    &trench.footprint.vertices,
                ));

                // Geometric: interior zone polygons
                for zone in &trench.zones {
                    report
                        .add_geometric_errors(GeometricValidator::validate_polygon(&zone.polygon));
                }

                // Physical: trench depth
                report.add_physical_errors(PhysicalValidator::validate_trench_depth(
                    trench.dimensions.depth,
                ));
            }

            Component::Gate(gate) => {
                // Geometric: footprint polygon
                report.add_geometric_errors(GeometricValidator::validate_polygon(
                    &gate.footprint.vertices,
                ));
            }

            Component::StreetSegment(street) => {
                // Geometric: footprint polygon
                report.add_geometric_errors(GeometricValidator::validate_polygon(
                    &street.footprint.vertices,
                ));

                // Civilian: capacity and width checks
                report.add_civilian_errors(CivilianValidator::validate_street(street));
            }
        }

        report
    }

    /// Validate a hex layout
    pub fn validate_hex_layout(layout: &HexLayout) -> ValidationReport {
        let mut report = ValidationReport::new();
        let hex_size = layout.hex_size as f32;

        // Validate each zone
        for zone in &layout.zones {
            // Geometric: polygon validity
            report.add_geometric_errors(GeometricValidator::validate_polygon(&zone.polygon));

            // Geometric: bounds check
            report.add_geometric_errors(GeometricValidator::validate_bounds(
                &zone.polygon,
                hex_size,
                hex_size,
            ));

            // Civilian: worker capacity
            let area = CivilianValidator::polygon_area(&zone.polygon);
            report.add_civilian_errors(CivilianValidator::validate_zone_capacity(zone, area));
        }

        // Check zone overlaps
        for i in 0..layout.zones.len() {
            for j in (i + 1)..layout.zones.len() {
                report.add_geometric_errors(GeometricValidator::validate_no_overlap(
                    &layout.zones[i].id,
                    &layout.zones[i].polygon,
                    &layout.zones[j].id,
                    &layout.zones[j].polygon,
                ));
            }
        }

        // Validate features are in bounds
        for feature in &layout.features {
            if feature.position[0] < 0.0
                || feature.position[0] > hex_size
                || feature.position[1] < 0.0
                || feature.position[1] > hex_size
            {
                report.add_geometric_errors(vec![ValidationError::OutOfBounds {
                    coordinate: feature.position,
                    bounds: [hex_size, hex_size],
                }]);
            }
        }

        // Validate hex connections
        report.add_connection_errors(ConnectionValidator::validate_hex_connection_position(
            "north",
            layout.connections.north.position,
            hex_size,
        ));
        report.add_connection_errors(ConnectionValidator::validate_hex_connection_position(
            "south",
            layout.connections.south.position,
            hex_size,
        ));
        report.add_connection_errors(ConnectionValidator::validate_hex_connection_position(
            "east",
            layout.connections.east.position,
            hex_size,
        ));
        report.add_connection_errors(ConnectionValidator::validate_hex_connection_position(
            "west",
            layout.connections.west.position,
            hex_size,
        ));

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::geometry_schema::*;

    #[test]
    fn test_wall_segment_validation() {
        let wall = WallSegment {
            variant_id: "test_wall".into(),
            display_name: "Test Wall".into(),
            dimensions: WallDimensions {
                length: 3.0,
                height: 2.0,
                thickness: 0.6,
            },
            footprint: Footprint {
                shape: "rectangle".into(),
                vertices: vec![[0.0, 0.0], [3.0, 0.0], [3.0, 0.6], [0.0, 0.6]],
                origin: "center_base".into(),
            },
            properties: WallProperties {
                blocks_movement: true,
                blocks_los: true,
                provides_cover: CoverLevel::Full,
                cover_direction: "perpendicular_to_length".into(),
                destructible: true,
                hp: 500,
                material: "stone".into(),
            },
            connection_points: vec![],
            tactical_notes: "Test".into(),
        };

        let report = CompositeValidator::validate_component(&Component::WallSegment(wall));
        assert!(report.is_valid);
        assert!(report.passed_geometric);
        assert!(report.passed_physical);
    }

    #[test]
    fn test_wall_too_tall() {
        let wall = WallSegment {
            variant_id: "tall_wall".into(),
            display_name: "Too Tall Wall".into(),
            dimensions: WallDimensions {
                length: 3.0,
                height: 15.0, // Too tall!
                thickness: 0.6,
            },
            footprint: Footprint {
                shape: "rectangle".into(),
                vertices: vec![[0.0, 0.0], [3.0, 0.0], [3.0, 0.6], [0.0, 0.6]],
                origin: "center_base".into(),
            },
            properties: WallProperties {
                blocks_movement: true,
                blocks_los: true,
                provides_cover: CoverLevel::Full,
                cover_direction: "perpendicular_to_length".into(),
                destructible: true,
                hp: 500,
                material: "stone".into(),
            },
            connection_points: vec![],
            tactical_notes: "Test".into(),
        };

        let report = CompositeValidator::validate_component(&Component::WallSegment(wall));
        assert!(!report.is_valid);
        assert!(!report.passed_physical);
    }
}
