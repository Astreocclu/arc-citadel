//! Tests for geometry schema deserialization

use arc_citadel::spatial::geometry_schema::*;
use arc_citadel::spatial::validation::{GeometricValidator, TacticalValidator, ValidationError};

fn make_firing_position(id: &str, center_angle: f32, arc_width: f32) -> FiringPosition {
    FiringPosition {
        id: id.to_string(),
        position: [0.0, 0.0, 8.0],
        firing_arc: FiringArc { center_angle, arc_width },
        elevation: 8.0,
        cover_value: CoverLevel::Full,
        capacity: 1,
    }
}

#[test]
fn test_valid_polygon_passes() {
    let vertices = vec![[0.0, 0.0], [3.0, 0.0], [3.0, 2.0], [0.0, 2.0]];
    let errors = GeometricValidator::validate_polygon(&vertices);
    assert!(errors.is_empty(), "Valid CCW rectangle should pass");
}

#[test]
fn test_self_intersecting_polygon_fails() {
    // Bowtie shape - self-intersecting
    let vertices = vec![[0.0, 0.0], [2.0, 2.0], [2.0, 0.0], [0.0, 2.0]];
    let errors = GeometricValidator::validate_polygon(&vertices);
    assert!(!errors.is_empty(), "Self-intersecting polygon should fail");
}

#[test]
fn test_insufficient_vertices_fails() {
    let vertices = vec![[0.0, 0.0], [1.0, 1.0]];
    let errors = GeometricValidator::validate_polygon(&vertices);
    assert!(errors.iter().any(|e| matches!(e, ValidationError::InsufficientVertices { .. })));
}

#[test]
fn test_wall_segment_deserialize() {
    let json = r#"{
        "component_type": "wall_segment",
        "variant_id": "stone_wall_3m_001",
        "display_name": "Stone Wall Section",
        "dimensions": {
            "length": 3.0,
            "height": 2.0,
            "thickness": 0.6
        },
        "footprint": {
            "shape": "rectangle",
            "vertices": [[0, 0], [3.0, 0], [3.0, 0.6], [0, 0.6]],
            "origin": "center_base"
        },
        "properties": {
            "blocks_movement": true,
            "blocks_los": true,
            "provides_cover": "full",
            "cover_direction": "perpendicular_to_length",
            "destructible": true,
            "hp": 500,
            "material": "stone"
        },
        "connection_points": [
            {"id": "west", "position": [0, 0.3], "direction": "west", "compatible_with": ["wall_segment", "wall_corner", "gate"]},
            {"id": "east", "position": [3.0, 0.3], "direction": "east", "compatible_with": ["wall_segment", "wall_corner", "gate"]}
        ],
        "tactical_notes": "Standard defensive wall."
    }"#;

    let component: Component = serde_json::from_str(json).unwrap();
    match component {
        Component::WallSegment(wall) => {
            assert_eq!(wall.variant_id, "stone_wall_3m_001");
            assert_eq!(wall.dimensions.length, 3.0);
            assert_eq!(wall.connection_points.len(), 2);
        }
        _ => panic!("Expected WallSegment"),
    }
}

#[test]
fn test_archer_tower_deserialize() {
    let json = r#"{
        "component_type": "archer_tower",
        "variant_id": "wooden_tower_8m_001",
        "display_name": "Wooden Archer Tower",
        "dimensions": {
            "base_width": 4.0,
            "base_depth": 4.0,
            "platform_height": 8.0,
            "platform_width": 5.0,
            "platform_depth": 5.0
        },
        "footprint": {
            "shape": "rectangle",
            "vertices": [[0, 0], [4.0, 0], [4.0, 4.0], [0, 4.0]],
            "origin": "center_base"
        },
        "firing_positions": [
            {
                "id": "pos_north",
                "position": [2.0, 4.5, 8.0],
                "firing_arc": {"center_angle": 0, "arc_width": 90},
                "elevation": 8.0,
                "cover_value": "full",
                "capacity": 1
            }
        ],
        "access": {
            "entry_point": [2.0, 0, 0],
            "entry_width": 1.0,
            "climb_time_seconds": 8
        },
        "properties": {
            "blocks_movement": true,
            "blocks_los_ground": true,
            "blocks_los_elevated": false,
            "total_capacity": 4,
            "provides_vision_bonus": 1.5,
            "destructible": true,
            "hp": 800,
            "material": "wood",
            "fire_vulnerable": true
        },
        "wall_connections": [],
        "tactical_notes": "Elevated firing platform."
    }"#;

    let component: Component = serde_json::from_str(json).unwrap();
    match component {
        Component::ArcherTower(tower) => {
            assert_eq!(tower.variant_id, "wooden_tower_8m_001");
            assert_eq!(tower.firing_positions.len(), 1);
            assert_eq!(tower.firing_positions[0].firing_arc.arc_width, 90.0);
        }
        _ => panic!("Expected ArcherTower"),
    }
}

#[test]
fn test_complete_360_coverage_passes() {
    let positions = vec![
        make_firing_position("north", 0.0, 90.0),
        make_firing_position("east", 90.0, 90.0),
        make_firing_position("south", 180.0, 90.0),
        make_firing_position("west", 270.0, 90.0),
    ];
    let errors = TacticalValidator::validate_firing_arcs(&positions);
    assert!(errors.is_empty(), "Complete 360° coverage should pass: {:?}", errors);
}

#[test]
fn test_incomplete_coverage_fails() {
    let positions = vec![
        make_firing_position("north", 0.0, 90.0),
        make_firing_position("east", 90.0, 90.0),
        // Missing south and west
    ];
    let errors = TacticalValidator::validate_firing_arcs(&positions);
    assert!(errors.iter().any(|e| matches!(e, ValidationError::FiringArcGap { .. })));
}

#[test]
fn test_arc_over_180_fails() {
    let positions = vec![
        make_firing_position("wide", 0.0, 200.0),
    ];
    let errors = TacticalValidator::validate_firing_arcs(&positions);
    assert!(errors.iter().any(|e| matches!(e, ValidationError::ArcTooWide { .. })));
}
