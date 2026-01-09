//! World objects and spatial identification

pub mod blocking;
pub mod loader;
pub mod objects;
pub mod placement;
pub mod spatial_id;

pub use blocking::{BlockedCells, BlockingState};
pub use loader::{LoadError, PlacementLoader};
pub use objects::WorldObjects;
pub use placement::{ObjectState, PlacedByJson, Placement, PlacementFile, PlacementMetadata};
pub use spatial_id::SpatialId;
