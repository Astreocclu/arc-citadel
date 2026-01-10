# LLM Geometry Generation Validation System

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a validation harness to test whether LLMs can generate geometrically valid building components and hex layouts for Arc Citadel's tactical combat system.

**Architecture:** Runtime validation pipeline with 5 validator categories (Geometric, Tactical, Connection, Physical, Civilian). JSON schemas parsed via serde into Rust structs, validated using the `geo` crate for polygon math. CLI test runner invokes DeepSeek API and reports pass/fail rates.

**Tech Stack:** Rust, serde_json, geo crate, tokio (async), existing LlmClient

---

## Task 1: Add geo Crate Dependency

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add geo dependency**

```toml
# Add after line 32 (after glam = "0.25")
geo = "0.28"
geo-types = "0.7"
```

**Step 2: Verify build**

Run: `cargo check`
Expected: Compiles successfully with new dependencies

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add geo crate for polygon validation"
```

---

## Task 2: Create Geometry Schema Module

**Files:**
- Create: `src/spatial/geometry_schema.rs`
- Modify: `src/spatial/mod.rs`

**Step 1: Write test for schema deserialization**

Create `tests/geometry_schema_tests.rs`:

```rust
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test geometry_schema_tests`
Expected: FAIL with "unresolved import"

**Step 3: Create geometry_schema.rs with structs**

Create `src/spatial/geometry_schema.rs`:

```rust
//! JSON schema definitions for LLM-generated geometry components
//!
//! These structs match the exact JSON format that LLMs will generate.
//! Used for parsing and validation of building components and hex layouts.

use serde::{Deserialize, Serialize};

/// Top-level component enum - discriminated by "component_type" field
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "component_type", rename_all = "snake_case")]
pub enum Component {
    WallSegment(WallSegment),
    ArcherTower(ArcherTower),
    TrenchSegment(TrenchSegment),
    Gate(Gate),
    StreetSegment(StreetSegment),
}

// ============================================================================
// WALL SEGMENT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallSegment {
    pub variant_id: String,
    pub display_name: String,
    pub dimensions: WallDimensions,
    pub footprint: Footprint,
    pub properties: WallProperties,
    pub connection_points: Vec<ConnectionPoint>,
    pub tactical_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallDimensions {
    pub length: f32,
    pub height: f32,
    pub thickness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallProperties {
    pub blocks_movement: bool,
    pub blocks_los: bool,
    pub provides_cover: CoverLevel,
    pub cover_direction: String,
    pub destructible: bool,
    pub hp: u32,
    pub material: String,
}

// ============================================================================
// ARCHER TOWER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcherTower {
    pub variant_id: String,
    pub display_name: String,
    pub dimensions: TowerDimensions,
    pub footprint: Footprint,
    pub firing_positions: Vec<FiringPosition>,
    pub access: TowerAccess,
    pub properties: TowerProperties,
    pub wall_connections: Vec<WallConnection>,
    pub tactical_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerDimensions {
    pub base_width: f32,
    pub base_depth: f32,
    pub platform_height: f32,
    pub platform_width: f32,
    pub platform_depth: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiringPosition {
    pub id: String,
    /// [x, y, z] position relative to component origin
    pub position: [f32; 3],
    pub firing_arc: FiringArc,
    pub elevation: f32,
    pub cover_value: CoverLevel,
    pub capacity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiringArc {
    /// Center angle in degrees (0 = north, 90 = east, 180 = south, 270 = west)
    pub center_angle: f32,
    /// Total arc width in degrees
    pub arc_width: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerAccess {
    pub entry_point: [f32; 3],
    pub entry_width: f32,
    pub climb_time_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerProperties {
    pub blocks_movement: bool,
    pub blocks_los_ground: bool,
    pub blocks_los_elevated: bool,
    pub total_capacity: u32,
    pub provides_vision_bonus: f32,
    pub destructible: bool,
    pub hp: u32,
    pub material: String,
    pub fire_vulnerable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallConnection {
    pub position: [f32; 2],
    pub direction: Direction,
    pub compatible_with: Vec<String>,
}

// ============================================================================
// TRENCH SEGMENT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrenchSegment {
    pub variant_id: String,
    pub display_name: String,
    pub dimensions: TrenchDimensions,
    pub footprint: Footprint,
    pub zones: Vec<TrenchZone>,
    pub cover_positions: Vec<CoverPosition>,
    pub properties: TrenchProperties,
    pub connection_points: Vec<ConnectionPoint>,
    pub tactical_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrenchDimensions {
    pub length: f32,
    pub width: f32,
    pub depth: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrenchZone {
    pub id: String,
    pub polygon: Vec<[f32; 2]>,
    pub elevation: f32,
    pub properties: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrenchProperties {
    pub blocks_movement_cavalry: bool,
    pub blocks_movement_infantry: bool,
    pub movement_cost_multiplier: f32,
    pub blocks_los: bool,
    pub provides_cover_inside: CoverLevel,
    pub provides_concealment: bool,
    pub indestructible: bool,
}

// ============================================================================
// GATE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate {
    pub variant_id: String,
    pub display_name: String,
    pub dimensions: GateDimensions,
    pub footprint: Footprint,
    pub states: GateStates,
    pub properties: GateProperties,
    pub connection_points: Vec<ConnectionPoint>,
    pub tactical_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateDimensions {
    pub width: f32,
    pub height: f32,
    pub thickness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateStates {
    pub open: GateState,
    pub closed: GateState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateState {
    pub blocks_movement: bool,
    pub blocks_los: bool,
    pub provides_cover: CoverLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateProperties {
    pub gate_type: String,
    pub fortification_level: String,
    pub destructible: bool,
    pub hp: u32,
    pub open_time_seconds: u32,
}

// ============================================================================
// STREET SEGMENT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreetSegment {
    pub variant_id: String,
    pub display_name: String,
    pub dimensions: StreetDimensions,
    pub footprint: Footprint,
    pub military_properties: StreetMilitaryProperties,
    pub civilian_properties: StreetCivilianProperties,
    pub connection_points: Vec<StreetConnection>,
    pub tactical_notes: String,
    pub economic_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreetDimensions {
    pub length: f32,
    pub width: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreetMilitaryProperties {
    pub provides_cover: CoverLevel,
    pub blocks_los: bool,
    pub movement_cost: f32,
    pub cavalry_charge_viable: bool,
    pub chokepoint: bool,
    pub ambush_risk: String,
    pub defensibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreetCivilianProperties {
    pub pedestrian_capacity: u32,
    pub cart_lanes: u32,
    pub market_stall_slots: u32,
    pub allows_gatherings: bool,
    pub drainage: String,
    pub fire_lane: bool,
    pub prestige_modifier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreetConnection {
    pub id: String,
    pub position: [f32; 2],
    pub direction: Direction,
    pub width: f32,
}

// ============================================================================
// SHARED TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footprint {
    pub shape: String,
    pub vertices: Vec<[f32; 2]>,
    pub origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoint {
    pub id: String,
    pub position: [f32; 2],
    pub direction: Direction,
    pub compatible_with: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverPosition {
    pub id: String,
    pub position: [f32; 3],
    pub cover_value: CoverLevel,
    pub cover_direction: String,
    pub capacity: u32,
    pub stance: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverLevel {
    None,
    Partial,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    North,
    East,
    South,
    West,
}

// ============================================================================
// HEX LAYOUT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexLayout {
    pub layout_type: String,
    pub variant_id: String,
    pub display_name: String,
    pub hex_size: u32,
    pub zones: Vec<HexZone>,
    pub features: Vec<HexFeature>,
    pub cover_positions: Vec<HexCoverPosition>,
    pub connections: HexConnections,
    pub elevation_map: ElevationMap,
    pub los_blockers: Vec<LosBlocker>,
    pub tactical_notes: String,
    pub ambush_points: Vec<AmbushPoint>,
    pub patrol_routes: Vec<PatrolRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexZone {
    pub id: String,
    pub polygon: Vec<[f32; 2]>,
    pub military_properties: ZoneMilitaryProperties,
    pub civilian_properties: ZoneCivilianProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneMilitaryProperties {
    pub terrain_type: String,
    pub movement_cost: f32,
    pub provides_cover: CoverLevel,
    #[serde(default)]
    pub hazards: Vec<String>,
    pub defensibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneCivilianProperties {
    pub function: String,
    #[serde(default)]
    pub worker_capacity: u32,
    #[serde(default)]
    pub storage_capacity: u32,
    #[serde(default)]
    pub throughput_capacity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexFeature {
    pub id: String,
    #[serde(rename = "type")]
    pub feature_type: String,
    pub position: [f32; 2],
    pub footprint: FeatureFootprint,
    pub properties: FeatureProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFootprint {
    pub shape: String,
    #[serde(default)]
    pub size: Option<[f32; 2]>,
    #[serde(default)]
    pub radius: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureProperties {
    pub blocks_movement: bool,
    #[serde(default)]
    pub blocks_los: bool,
    pub provides_cover: CoverLevel,
    #[serde(default)]
    pub cover_height: f32,
    #[serde(default)]
    pub indestructible: bool,
    #[serde(default)]
    pub destructible: bool,
    #[serde(default)]
    pub movement_cost_multiplier: f32,
    #[serde(default)]
    pub interaction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexCoverPosition {
    pub id: String,
    pub position: [f32; 2],
    pub cover_value: CoverLevel,
    pub cover_direction: String,
    pub provided_by: String,
    pub stance: String,
    pub capacity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexConnections {
    pub north: HexConnection,
    pub south: HexConnection,
    pub east: HexConnection,
    pub west: HexConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexConnection {
    #[serde(rename = "type")]
    pub connection_type: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub position: Option<[f32; 2]>,
    #[serde(default)]
    pub obstacle: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationMap {
    pub default: f32,
    pub zones: std::collections::HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LosBlocker {
    #[serde(rename = "type")]
    pub blocker_type: String,
    pub vertices: Vec<[f32; 2]>,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbushPoint {
    pub position: [f32; 2],
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatrolRoute {
    pub id: String,
    pub waypoints: Vec<[f32; 2]>,
}

// ============================================================================
// TEST RESULT CONTAINER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryTestResult {
    pub test_run_id: String,
    pub model: String,
    pub timestamp: String,
    pub wall_segments: Vec<WallSegment>,
    pub archer_towers: Vec<ArcherTower>,
    pub trenches: Vec<TrenchSegment>,
    pub gates: Vec<Gate>,
    pub street_segments: Vec<StreetSegment>,
    pub hex_layouts: HexLayoutCollection,
    pub validation_results: ValidationResults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexLayoutCollection {
    pub dwarven_forge: Vec<HexLayout>,
    pub human_tavern: Vec<HexLayout>,
    pub elven_glade: Vec<HexLayout>,
    pub defensive_outpost: Vec<HexLayout>,
    pub forest_clearing: Vec<HexLayout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    pub total_components: u32,
    pub passed_geometric: u32,
    pub passed_tactical: u32,
    pub passed_connection: u32,
    pub passed_physical: u32,
    pub passed_civilian: u32,
    pub failed_components: Vec<FailedComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedComponent {
    pub id: String,
    pub failure_reason: String,
}
```

**Step 4: Update mod.rs to export new module**

Add to `src/spatial/mod.rs`:

```rust
pub mod geometry_schema;
```

**Step 5: Run test to verify it passes**

Run: `cargo test --test geometry_schema_tests`
Expected: PASS (2 tests)

**Step 6: Commit**

```bash
git add src/spatial/geometry_schema.rs src/spatial/mod.rs tests/geometry_schema_tests.rs
git commit -m "feat: add geometry schema structs for LLM validation"
```

---

## Task 3: Implement Geometric Validator

**Files:**
- Create: `src/spatial/validation/mod.rs`
- Create: `src/spatial/validation/geometric.rs`
- Modify: `src/spatial/mod.rs`

**Step 1: Write failing test for polygon validation**

Add to `tests/geometry_schema_tests.rs`:

```rust
use arc_citadel::spatial::validation::{GeometricValidator, ValidationError};

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
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test geometry_schema_tests test_valid_polygon`
Expected: FAIL with "unresolved import"

**Step 3: Create validation module structure**

Create `src/spatial/validation/mod.rs`:

```rust
//! Geometry validation for LLM-generated components

mod geometric;

pub use geometric::GeometricValidator;

/// Validation error types
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    InsufficientVertices { count: usize, minimum: usize },
    SelfIntersecting { description: String },
    InvalidWinding { expected: Winding },
    OutOfBounds { coordinate: [f32; 2], bounds: [f32; 2] },
    PolygonOverlap { zone1: String, zone2: String },
    FiringArcGap { missing_degrees: f32 },
    FiringArcOverlap { positions: Vec<String> },
    ArcTooWide { position_id: String, width: f32, max: f32 },
    ConnectionMisaligned { point1: String, point2: String, distance: f32 },
    FeatureOutsideZone { feature_id: String },
    InvalidCoverPosition { position_id: String, reason: String },
    PhysicalImplausible { description: String },
    CivilianCapacityExceeded { zone_id: String, claimed: u32, max: u32 },
    InsufficientWidth { component_id: String, width: f32, required: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Winding {
    CounterClockwise,
    Clockwise,
}
```

Create `src/spatial/validation/geometric.rs`:

```rust
//! Geometric validation: polygon validity, winding, bounds, overlaps

use super::{ValidationError, Winding};
use geo::{Contains, Intersects, LineString, Polygon};
use geo::algorithm::winding_order::Winding as GeoWinding;
use geo::algorithm::is_convex::IsConvex;

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

        let line_string = LineString::from(coords.clone());

        // Check for self-intersection by creating polygon and checking validity
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
    fn segments_intersect(
        a1: (f64, f64),
        a2: (f64, f64),
        b1: (f64, f64),
        b2: (f64, f64),
    ) -> bool {
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
```

**Step 4: Update mod.rs to export validation module**

Add to `src/spatial/mod.rs`:

```rust
pub mod validation;
```

**Step 5: Run test to verify it passes**

Run: `cargo test --test geometry_schema_tests`
Expected: PASS (5 tests)

**Step 6: Commit**

```bash
git add src/spatial/validation/ src/spatial/mod.rs tests/geometry_schema_tests.rs
git commit -m "feat: add geometric validator for polygon validation"
```

---

## Task 4: Implement Tactical Validator (Firing Arcs)

**Files:**
- Create: `src/spatial/validation/tactical.rs`
- Modify: `src/spatial/validation/mod.rs`

**Step 1: Write failing test for firing arc validation**

Add to `tests/geometry_schema_tests.rs`:

```rust
use arc_citadel::spatial::validation::TacticalValidator;
use arc_citadel::spatial::geometry_schema::{FiringPosition, FiringArc, CoverLevel};

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
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test geometry_schema_tests test_complete_360`
Expected: FAIL with "unresolved import"

**Step 3: Create tactical.rs**

Create `src/spatial/validation/tactical.rs`:

```rust
//! Tactical validation: firing arcs, cover positions, chokepoints

use super::ValidationError;
use crate::spatial::geometry_schema::FiringPosition;

pub struct TacticalValidator;

impl TacticalValidator {
    /// Validate that firing positions cover 360° without gaps and each arc ≤ 180°
    pub fn validate_firing_arcs(positions: &[FiringPosition]) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if positions.is_empty() {
            errors.push(ValidationError::FiringArcGap { missing_degrees: 360.0 });
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
    pub fn validate_chokepoint_width(width: f32, is_marked_chokepoint: bool) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // If marked as chokepoint but too wide, that's an error
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
        let intervals = TacticalValidator::arcs_to_intervals(&[
            crate::spatial::geometry_schema::FiringPosition {
                id: "north".into(),
                position: [0.0, 0.0, 0.0],
                firing_arc: crate::spatial::geometry_schema::FiringArc {
                    center_angle: 0.0,
                    arc_width: 90.0,
                },
                elevation: 8.0,
                cover_value: crate::spatial::geometry_schema::CoverLevel::Full,
                capacity: 1,
            }
        ]);
        // Should produce two intervals: [315, 360) and [0, 45)
        assert_eq!(intervals.len(), 2);
    }
}
```

**Step 4: Update mod.rs**

Add to `src/spatial/validation/mod.rs`:

```rust
mod tactical;
pub use tactical::TacticalValidator;
```

**Step 5: Run test to verify it passes**

Run: `cargo test --test geometry_schema_tests`
Expected: PASS (8 tests)

**Step 6: Commit**

```bash
git add src/spatial/validation/tactical.rs src/spatial/validation/mod.rs tests/geometry_schema_tests.rs
git commit -m "feat: add tactical validator for firing arc coverage"
```

---

## Task 5: Implement Civilian Validator

**Files:**
- Create: `src/spatial/validation/civilian.rs`
- Modify: `src/spatial/validation/mod.rs`

**Step 1: Write failing test for civilian validation**

Add to `tests/geometry_schema_tests.rs`:

```rust
use arc_citadel::spatial::validation::CivilianValidator;
use arc_citadel::spatial::geometry_schema::{StreetSegment, StreetDimensions, StreetCivilianProperties, StreetMilitaryProperties, Footprint, StreetConnection, CoverLevel};

fn make_street(width: f32, pedestrian_capacity: u32, cart_lanes: u32, market_stalls: u32) -> StreetSegment {
    StreetSegment {
        variant_id: "test_street".into(),
        display_name: "Test Street".into(),
        dimensions: StreetDimensions { length: 10.0, width },
        footprint: Footprint {
            shape: "rectangle".into(),
            vertices: vec![[0.0, 0.0], [10.0, 0.0], [10.0, width], [0.0, width]],
            origin: "center_base".into(),
        },
        military_properties: StreetMilitaryProperties {
            provides_cover: CoverLevel::None,
            blocks_los: false,
            movement_cost: 1.0,
            cavalry_charge_viable: width >= 6.0,
            chokepoint: width < 4.0,
            ambush_risk: "low".into(),
            defensibility: "poor".into(),
        },
        civilian_properties: StreetCivilianProperties {
            pedestrian_capacity,
            cart_lanes,
            market_stall_slots: market_stalls,
            allows_gatherings: true,
            drainage: "good".into(),
            fire_lane: true,
            prestige_modifier: 1.0,
        },
        connection_points: vec![],
        tactical_notes: "Test".into(),
        economic_notes: "Test".into(),
    }
}

#[test]
fn test_street_cart_lanes_width_valid() {
    let street = make_street(6.0, 30, 2, 0);
    let errors = CivilianValidator::validate_street(&street);
    assert!(errors.is_empty(), "6m street with 2 cart lanes should pass");
}

#[test]
fn test_street_cart_lanes_too_narrow() {
    let street = make_street(2.0, 10, 2, 0);
    let errors = CivilianValidator::validate_street(&street);
    assert!(errors.iter().any(|e| matches!(e, ValidationError::InsufficientWidth { .. })));
}

#[test]
fn test_market_stalls_exceed_length() {
    // 10m street, but claiming 10 stalls at 1.5m each = 15m needed
    let street = make_street(8.0, 50, 2, 10);
    let errors = CivilianValidator::validate_street(&street);
    assert!(!errors.is_empty(), "Too many market stalls for street length");
}

#[test]
fn test_cavalry_on_narrow_street() {
    // 4m street claiming cavalry viable
    let mut street = make_street(4.0, 20, 1, 0);
    street.military_properties.cavalry_charge_viable = true;
    let errors = CivilianValidator::validate_street(&street);
    assert!(!errors.is_empty(), "Cavalry should not be viable on 4m street");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test geometry_schema_tests test_street`
Expected: FAIL with "unresolved import"

**Step 3: Create civilian.rs**

Create `src/spatial/validation/civilian.rs`:

```rust
//! Civilian validation: capacity, width requirements, economic plausibility

use super::ValidationError;
use crate::spatial::geometry_schema::{StreetSegment, HexZone};

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
```

**Step 4: Update mod.rs**

Add to `src/spatial/validation/mod.rs`:

```rust
mod civilian;
pub use civilian::CivilianValidator;
```

**Step 5: Run test to verify it passes**

Run: `cargo test --test geometry_schema_tests`
Expected: PASS (12 tests)

**Step 6: Commit**

```bash
git add src/spatial/validation/civilian.rs src/spatial/validation/mod.rs tests/geometry_schema_tests.rs
git commit -m "feat: add civilian validator for capacity and width checks"
```

---

## Task 6: Implement Connection Validator

**Files:**
- Create: `src/spatial/validation/connection.rs`
- Modify: `src/spatial/validation/mod.rs`

**Step 1: Write failing test**

Add to `tests/geometry_schema_tests.rs`:

```rust
use arc_citadel::spatial::validation::ConnectionValidator;
use arc_citadel::spatial::geometry_schema::ConnectionPoint;

#[test]
fn test_aligned_connections_pass() {
    let p1 = ConnectionPoint {
        id: "east".into(),
        position: [3.0, 0.3],
        direction: arc_citadel::spatial::geometry_schema::Direction::East,
        compatible_with: vec!["wall_segment".into()],
    };
    let p2 = ConnectionPoint {
        id: "west".into(),
        position: [3.05, 0.3], // Within 0.1m tolerance
        direction: arc_citadel::spatial::geometry_schema::Direction::West,
        compatible_with: vec!["wall_segment".into()],
    };
    let errors = ConnectionValidator::validate_alignment(&p1, &p2, 0.1);
    assert!(errors.is_empty());
}

#[test]
fn test_misaligned_connections_fail() {
    let p1 = ConnectionPoint {
        id: "east".into(),
        position: [3.0, 0.3],
        direction: arc_citadel::spatial::geometry_schema::Direction::East,
        compatible_with: vec!["wall_segment".into()],
    };
    let p2 = ConnectionPoint {
        id: "west".into(),
        position: [3.5, 0.3], // 0.5m off - exceeds tolerance
        direction: arc_citadel::spatial::geometry_schema::Direction::West,
        compatible_with: vec!["wall_segment".into()],
    };
    let errors = ConnectionValidator::validate_alignment(&p1, &p2, 0.1);
    assert!(errors.iter().any(|e| matches!(e, ValidationError::ConnectionMisaligned { .. })));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test geometry_schema_tests test_aligned`
Expected: FAIL

**Step 3: Create connection.rs**

Create `src/spatial/validation/connection.rs`:

```rust
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
```

**Step 4: Update mod.rs**

Add to `src/spatial/validation/mod.rs`:

```rust
mod connection;
pub use connection::ConnectionValidator;
```

**Step 5: Run test to verify it passes**

Run: `cargo test --test geometry_schema_tests`
Expected: PASS (14 tests)

**Step 6: Commit**

```bash
git add src/spatial/validation/connection.rs src/spatial/validation/mod.rs tests/geometry_schema_tests.rs
git commit -m "feat: add connection validator for alignment checks"
```

---

## Task 7: Implement Physical Validator

**Files:**
- Create: `src/spatial/validation/physical.rs`
- Modify: `src/spatial/validation/mod.rs`

**Step 1: Write failing test**

Add to `tests/geometry_schema_tests.rs`:

```rust
use arc_citadel::spatial::validation::PhysicalValidator;

#[test]
fn test_platform_clearance_valid() {
    let errors = PhysicalValidator::validate_platform_clearance(8.0, 2.0);
    assert!(errors.is_empty(), "8m platform with 2m clearance should pass");
}

#[test]
fn test_platform_clearance_insufficient() {
    let errors = PhysicalValidator::validate_platform_clearance(1.5, 2.0);
    assert!(!errors.is_empty(), "1.5m platform should fail 2m clearance check");
}

#[test]
fn test_trench_depth_survivable() {
    let errors = PhysicalValidator::validate_trench_depth(1.2);
    assert!(errors.is_empty(), "1.2m trench should be survivable");
}

#[test]
fn test_trench_depth_too_deep() {
    let errors = PhysicalValidator::validate_trench_depth(2.5);
    assert!(!errors.is_empty(), "2.5m trench requires ladder");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test geometry_schema_tests test_platform`
Expected: FAIL

**Step 3: Create physical.rs**

Create `src/spatial/validation/physical.rs`:

```rust
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
    pub fn validate_grounded(position: [f32; 3], expected_z: f32, tolerance: f32) -> Vec<ValidationError> {
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
```

**Step 4: Update mod.rs**

Add to `src/spatial/validation/mod.rs`:

```rust
mod physical;
pub use physical::PhysicalValidator;
```

**Step 5: Run test to verify it passes**

Run: `cargo test --test geometry_schema_tests`
Expected: PASS (18 tests)

**Step 6: Commit**

```bash
git add src/spatial/validation/physical.rs src/spatial/validation/mod.rs tests/geometry_schema_tests.rs
git commit -m "feat: add physical validator for plausibility checks"
```

---

## Task 8: Create Composite Validator

**Files:**
- Create: `src/spatial/validation/composite.rs`
- Modify: `src/spatial/validation/mod.rs`

**Step 1: Write failing test**

Add to `tests/geometry_schema_tests.rs`:

```rust
use arc_citadel::spatial::validation::CompositeValidator;

#[test]
fn test_composite_validator_wall_segment() {
    let json = r#"{
        "component_type": "wall_segment",
        "variant_id": "stone_wall_3m_001",
        "display_name": "Stone Wall Section",
        "dimensions": { "length": 3.0, "height": 2.0, "thickness": 0.6 },
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
            {"id": "west", "position": [0, 0.3], "direction": "west", "compatible_with": ["wall_segment"]},
            {"id": "east", "position": [3.0, 0.3], "direction": "east", "compatible_with": ["wall_segment"]}
        ],
        "tactical_notes": "Standard defensive wall."
    }"#;

    let component: arc_citadel::spatial::geometry_schema::Component = serde_json::from_str(json).unwrap();
    let report = CompositeValidator::validate_component(&component);

    assert!(report.is_valid, "Valid wall segment should pass: {:?}", report.errors);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test geometry_schema_tests test_composite`
Expected: FAIL

**Step 3: Create composite.rs**

Create `src/spatial/validation/composite.rs`:

```rust
//! Composite validator that runs all validation checks

use super::{
    ValidationError, GeometricValidator, TacticalValidator, CivilianValidator,
    ConnectionValidator, PhysicalValidator,
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

pub struct CompositeValidator;

impl CompositeValidator {
    /// Validate a single component
    pub fn validate_component(component: &Component) -> ValidationReport {
        let mut report = ValidationReport::new();

        match component {
            Component::WallSegment(wall) => {
                // Geometric: footprint polygon
                report.add_geometric_errors(
                    GeometricValidator::validate_polygon(&wall.footprint.vertices)
                );

                // Physical: wall height plausible
                if wall.dimensions.height > 10.0 {
                    report.add_physical_errors(vec![ValidationError::PhysicalImplausible {
                        description: format!("Wall height {}m > 10m max", wall.dimensions.height),
                    }]);
                }
            }

            Component::ArcherTower(tower) => {
                // Geometric: footprint polygon
                report.add_geometric_errors(
                    GeometricValidator::validate_polygon(&tower.footprint.vertices)
                );

                // Tactical: firing arcs sum to 360
                report.add_tactical_errors(
                    TacticalValidator::validate_firing_arcs(&tower.firing_positions)
                );

                // Physical: platform clearance
                report.add_physical_errors(
                    PhysicalValidator::validate_platform_clearance(tower.dimensions.platform_height, 2.0)
                );
            }

            Component::TrenchSegment(trench) => {
                // Geometric: footprint polygon
                report.add_geometric_errors(
                    GeometricValidator::validate_polygon(&trench.footprint.vertices)
                );

                // Geometric: interior zone polygons
                for zone in &trench.zones {
                    report.add_geometric_errors(
                        GeometricValidator::validate_polygon(&zone.polygon)
                    );
                }

                // Physical: trench depth
                report.add_physical_errors(
                    PhysicalValidator::validate_trench_depth(trench.dimensions.depth)
                );
            }

            Component::Gate(gate) => {
                // Geometric: footprint polygon
                report.add_geometric_errors(
                    GeometricValidator::validate_polygon(&gate.footprint.vertices)
                );
            }

            Component::StreetSegment(street) => {
                // Geometric: footprint polygon
                report.add_geometric_errors(
                    GeometricValidator::validate_polygon(&street.footprint.vertices)
                );

                // Civilian: capacity and width checks
                report.add_civilian_errors(
                    CivilianValidator::validate_street(street)
                );
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
            report.add_geometric_errors(
                GeometricValidator::validate_polygon(&zone.polygon)
            );

            // Geometric: bounds check
            report.add_geometric_errors(
                GeometricValidator::validate_bounds(&zone.polygon, hex_size, hex_size)
            );

            // Civilian: worker capacity
            let area = CivilianValidator::polygon_area(&zone.polygon);
            report.add_civilian_errors(
                CivilianValidator::validate_zone_capacity(zone, area)
            );
        }

        // Check zone overlaps
        for i in 0..layout.zones.len() {
            for j in (i + 1)..layout.zones.len() {
                report.add_geometric_errors(
                    GeometricValidator::validate_no_overlap(
                        &layout.zones[i].id,
                        &layout.zones[i].polygon,
                        &layout.zones[j].id,
                        &layout.zones[j].polygon,
                    )
                );
            }
        }

        // Validate features are in bounds
        for feature in &layout.features {
            if feature.position[0] < 0.0 || feature.position[0] > hex_size
                || feature.position[1] < 0.0 || feature.position[1] > hex_size
            {
                report.add_geometric_errors(vec![ValidationError::OutOfBounds {
                    coordinate: feature.position,
                    bounds: [hex_size, hex_size],
                }]);
            }
        }

        // Validate hex connections
        report.add_connection_errors(
            ConnectionValidator::validate_hex_connection_position(
                "north",
                layout.connections.north.position,
                hex_size,
            )
        );
        report.add_connection_errors(
            ConnectionValidator::validate_hex_connection_position(
                "south",
                layout.connections.south.position,
                hex_size,
            )
        );
        report.add_connection_errors(
            ConnectionValidator::validate_hex_connection_position(
                "east",
                layout.connections.east.position,
                hex_size,
            )
        );
        report.add_connection_errors(
            ConnectionValidator::validate_hex_connection_position(
                "west",
                layout.connections.west.position,
                hex_size,
            )
        );

        report
    }
}
```

**Step 4: Update mod.rs**

Add to `src/spatial/validation/mod.rs`:

```rust
mod composite;
pub use composite::{CompositeValidator, ValidationReport};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --test geometry_schema_tests`
Expected: PASS (19 tests)

**Step 6: Commit**

```bash
git add src/spatial/validation/composite.rs src/spatial/validation/mod.rs tests/geometry_schema_tests.rs
git commit -m "feat: add composite validator for full validation pipeline"
```

---

## Task 9: Create Test Runner CLI

**Files:**
- Create: `src/bin/geometry_test.rs`

**Step 1: Create the CLI binary**

Create `src/bin/geometry_test.rs`:

```rust
//! CLI tool for running LLM geometry generation tests
//!
//! Usage:
//!   cargo run --bin geometry_test -- --model deepseek --output results.json

use arc_citadel::llm::client::LlmClient;
use arc_citadel::spatial::geometry_schema::*;
use arc_citadel::spatial::validation::{CompositeValidator, ValidationReport};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct Args {
    model: String,
    output: PathBuf,
    prompt_file: Option<PathBuf>,
}

fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    let mut model = "deepseek".to_string();
    let mut output = PathBuf::from("geometry_test_results.json");
    let mut prompt_file = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--model" | "-m" => {
                i += 1;
                if i < args.len() {
                    model = args[i].clone();
                }
            }
            "--output" | "-o" => {
                i += 1;
                if i < args.len() {
                    output = PathBuf::from(&args[i]);
                }
            }
            "--prompt" | "-p" => {
                i += 1;
                if i < args.len() {
                    prompt_file = Some(PathBuf::from(&args[i]));
                }
            }
            _ => {}
        }
        i += 1;
    }

    Args { model, output, prompt_file }
}

const DEFAULT_PROMPT: &str = r#"Generate geometry components for a tactical strategy game. Output valid JSON matching the schemas exactly.

Generate:
- 10 wall_segment variants
- 10 archer_tower variants
- 10 trench_segment variants
- 10 gate variants
- 10 street_segment variants
- 5 hex layouts each for: dwarven_forge, human_tavern, elven_glade, defensive_outpost, forest_clearing

Requirements:
- All coordinates in meters
- Polygon vertices counter-clockwise
- Tower firing arcs must sum to 360°
- Street cavalry_charge_viable only if width >= 6m
- All positions within hex bounds [0, 100]

Output a single JSON object with all components."#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args();

    println!("=== LLM Geometry Generation Test ===");
    println!("Model: {}", args.model);
    println!("Output: {}", args.output.display());

    // Load prompt
    let prompt = if let Some(path) = &args.prompt_file {
        fs::read_to_string(path)?
    } else {
        DEFAULT_PROMPT.to_string()
    };

    // Create LLM client
    let client = LlmClient::from_env()?;

    println!("\nSending generation request to LLM...");
    let response = client.complete(
        "You are a geometry generator for a tactical strategy game. Output only valid JSON.",
        &prompt,
    ).await?;

    println!("Received response ({} chars)", response.len());

    // Extract JSON from response
    let json_start = response.find('{').unwrap_or(0);
    let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
    let json_str = &response[json_start..json_end];

    // Parse response
    println!("\nParsing response...");
    let result: GeometryTestResult = match serde_json::from_str(json_str) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to parse JSON: {}", e);
            eprintln!("Raw response:\n{}", &response[..response.len().min(2000)]);
            return Err(e.into());
        }
    };

    // Validate all components
    println!("\nValidating components...");
    let mut total = 0u32;
    let mut passed_geometric = 0u32;
    let mut passed_tactical = 0u32;
    let mut passed_connection = 0u32;
    let mut passed_physical = 0u32;
    let mut passed_civilian = 0u32;
    let mut failed: Vec<FailedComponent> = Vec::new();

    // Helper to process validation report
    let mut process_report = |id: &str, report: ValidationReport| {
        total += 1;
        if report.passed_geometric { passed_geometric += 1; }
        if report.passed_tactical { passed_tactical += 1; }
        if report.passed_connection { passed_connection += 1; }
        if report.passed_physical { passed_physical += 1; }
        if report.passed_civilian { passed_civilian += 1; }
        if !report.is_valid {
            failed.push(FailedComponent {
                id: id.to_string(),
                failure_reason: format!("{:?}", report.errors),
            });
        }
    };

    // Validate wall segments
    for wall in &result.wall_segments {
        let report = CompositeValidator::validate_component(&Component::WallSegment(wall.clone()));
        process_report(&wall.variant_id, report);
    }

    // Validate archer towers
    for tower in &result.archer_towers {
        let report = CompositeValidator::validate_component(&Component::ArcherTower(tower.clone()));
        process_report(&tower.variant_id, report);
    }

    // Validate trenches
    for trench in &result.trenches {
        let report = CompositeValidator::validate_component(&Component::TrenchSegment(trench.clone()));
        process_report(&trench.variant_id, report);
    }

    // Validate gates
    for gate in &result.gates {
        let report = CompositeValidator::validate_component(&Component::Gate(gate.clone()));
        process_report(&gate.variant_id, report);
    }

    // Validate streets
    for street in &result.street_segments {
        let report = CompositeValidator::validate_component(&Component::StreetSegment(street.clone()));
        process_report(&street.variant_id, report);
    }

    // Validate hex layouts
    for layout in &result.hex_layouts.dwarven_forge {
        let report = CompositeValidator::validate_hex_layout(layout);
        process_report(&layout.variant_id, report);
    }
    for layout in &result.hex_layouts.human_tavern {
        let report = CompositeValidator::validate_hex_layout(layout);
        process_report(&layout.variant_id, report);
    }
    for layout in &result.hex_layouts.elven_glade {
        let report = CompositeValidator::validate_hex_layout(layout);
        process_report(&layout.variant_id, report);
    }
    for layout in &result.hex_layouts.defensive_outpost {
        let report = CompositeValidator::validate_hex_layout(layout);
        process_report(&layout.variant_id, report);
    }
    for layout in &result.hex_layouts.forest_clearing {
        let report = CompositeValidator::validate_hex_layout(layout);
        process_report(&layout.variant_id, report);
    }

    // Build final results
    let final_results = ValidationResults {
        total_components: total,
        passed_geometric,
        passed_tactical,
        passed_connection,
        passed_physical,
        passed_civilian,
        failed_components: failed,
    };

    // Print summary
    println!("\n=== Validation Results ===");
    println!("Total components: {}", total);
    println!("Passed geometric: {} ({:.1}%)", passed_geometric, 100.0 * passed_geometric as f64 / total as f64);
    println!("Passed tactical:  {} ({:.1}%)", passed_tactical, 100.0 * passed_tactical as f64 / total as f64);
    println!("Passed connection: {} ({:.1}%)", passed_connection, 100.0 * passed_connection as f64 / total as f64);
    println!("Passed physical:  {} ({:.1}%)", passed_physical, 100.0 * passed_physical as f64 / total as f64);
    println!("Passed civilian:  {} ({:.1}%)", passed_civilian, 100.0 * passed_civilian as f64 / total as f64);

    let all_passed = final_results.failed_components.is_empty();
    let pass_rate = if total > 0 {
        100.0 * (total - final_results.failed_components.len() as u32) as f64 / total as f64
    } else {
        0.0
    };

    println!("\nOverall pass rate: {:.1}%", pass_rate);
    if pass_rate >= 70.0 {
        println!("RESULT: LLM geometry generation is VIABLE");
    } else if pass_rate >= 40.0 {
        println!("RESULT: Need validation + correction loop");
    } else {
        println!("RESULT: Fall back to procedural generation");
    }

    // Save results
    let output_data = serde_json::json!({
        "test_run_id": result.test_run_id,
        "model": result.model,
        "timestamp": result.timestamp,
        "validation_results": final_results,
        "raw_components": {
            "wall_segments": result.wall_segments.len(),
            "archer_towers": result.archer_towers.len(),
            "trenches": result.trenches.len(),
            "gates": result.gates.len(),
            "street_segments": result.street_segments.len(),
            "hex_layouts": {
                "dwarven_forge": result.hex_layouts.dwarven_forge.len(),
                "human_tavern": result.hex_layouts.human_tavern.len(),
                "elven_glade": result.hex_layouts.elven_glade.len(),
                "defensive_outpost": result.hex_layouts.defensive_outpost.len(),
                "forest_clearing": result.hex_layouts.forest_clearing.len(),
            }
        }
    });

    fs::write(&args.output, serde_json::to_string_pretty(&output_data)?)?;
    println!("\nResults saved to: {}", args.output.display());

    Ok(())
}
```

**Step 2: Verify it builds**

Run: `cargo build --bin geometry_test`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/bin/geometry_test.rs
git commit -m "feat: add geometry test CLI runner"
```

---

## Task 10: Create DeepSeek Prompt File

**Files:**
- Create: `data/prompts/geometry_generation.txt`

**Step 1: Create prompts directory and file**

Run: `mkdir -p data/prompts`

Create `data/prompts/geometry_generation.txt` with the full prompt from the user's original specification (the complete JSON schemas and generation instructions).

**Step 2: Commit**

```bash
git add data/prompts/geometry_generation.txt
git commit -m "feat: add geometry generation prompt for DeepSeek"
```

---

## Task 11: Run the Test

**Step 1: Set up environment**

```bash
export LLM_API_KEY="your-deepseek-api-key"
export LLM_API_URL="https://api.deepseek.com/v1/chat/completions"
export LLM_MODEL="deepseek-chat"
```

**Step 2: Run the test**

```bash
cargo run --bin geometry_test -- --prompt data/prompts/geometry_generation.txt --output geometry_results.json
```

**Step 3: Review results**

The output will show:
- Pass rates for each validation category
- Overall viability assessment
- Detailed failures in `geometry_results.json`

**Step 4: Commit results**

```bash
git add geometry_results.json
git commit -m "test: run initial geometry generation test with DeepSeek"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Add geo crate | Cargo.toml |
| 2 | Schema structs | src/spatial/geometry_schema.rs |
| 3 | Geometric validator | src/spatial/validation/geometric.rs |
| 4 | Tactical validator | src/spatial/validation/tactical.rs |
| 5 | Civilian validator | src/spatial/validation/civilian.rs |
| 6 | Connection validator | src/spatial/validation/connection.rs |
| 7 | Physical validator | src/spatial/validation/physical.rs |
| 8 | Composite validator | src/spatial/validation/composite.rs |
| 9 | Test runner CLI | src/bin/geometry_test.rs |
| 10 | DeepSeek prompt | data/prompts/geometry_generation.txt |
| 11 | Run test | (manual execution) |

**Total estimated steps:** ~50 bite-sized actions

**Success criteria:** 70%+ pass rate indicates LLM geometry generation is viable.
