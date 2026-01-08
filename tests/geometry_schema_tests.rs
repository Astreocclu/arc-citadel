//! Tests for geometry schema deserialization

use arc_citadel::spatial::geometry_schema::*;

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
