//! Geometry validation for LLM-generated components

mod civilian;
mod composite;
mod connection;
mod geometric;
mod physical;
mod tactical;

pub use civilian::CivilianValidator;
pub use composite::{CompositeValidator, ValidationReport};
pub use connection::ConnectionValidator;
pub use geometric::GeometricValidator;
pub use physical::PhysicalValidator;
pub use tactical::TacticalValidator;

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
