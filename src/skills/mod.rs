//! Hierarchical chunking skill system
//!
//! Skill mastery is modeled through cognitive chunking - practiced actions
//! combine into larger chunks that execute with lower attention cost.
//!
//! A novice conscript thinks: "Grip. Stance. Swing. Don't drop it."
//! A master thinks: "Handle this flank." - thousands of micro-actions automatic.

pub mod attention;
pub mod chunk_id;
pub mod context;
pub mod definitions;
pub mod library;

pub use attention::{calculate_attention_budget, can_afford_attention, risks_fumble, FUMBLE_ATTENTION_THRESHOLD};
pub use chunk_id::ChunkId;
pub use context::{CombatContext, ContextTag};
pub use definitions::{get_chunk_definition, ChunkComponents, ChunkDefinition, CHUNK_LIBRARY};
pub use library::{ChunkLibrary, Experience, PersonalChunkState};
