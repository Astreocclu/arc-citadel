//! Hierarchical chunking skill system
//!
//! Skill mastery is modeled through cognitive chunking - practiced actions
//! combine into larger chunks that execute with lower attention cost.
//!
//! A novice conscript thinks: "Grip. Stance. Swing. Don't drop it."
//! A master thinks: "Handle this flank." - thousands of micro-actions automatic.

pub mod chunk_id;
pub mod context;

pub use chunk_id::ChunkId;
pub use context::{CombatContext, ContextTag};
