//! Battle system - tactical combat with fog of war and courier delays
//!
//! NOT Total War: terrain is dense, vision is scarce, control is delegated.

pub mod constants;
pub mod hex;
pub mod terrain;
pub mod battle_map;
pub mod planning;
pub mod execution;
pub mod courier;
pub mod resolution;

pub use hex::{BattleHexCoord, HexDirection};
pub use terrain::{BattleTerrain, TerrainFeature};
pub use battle_map::{BattleHex, BattleMap, VisibilityState, Objective};
