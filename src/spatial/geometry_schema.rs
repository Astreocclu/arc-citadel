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
