//! Hierarchical chunking skill system
//!
//! Skill mastery is modeled through cognitive chunking - practiced actions
//! combine into larger chunks that execute with lower attention cost.
//!
//! A novice conscript thinks: "Grip. Stance. Swing. Don't drop it."
//! A master thinks: "Handle this flank." - thousands of micro-actions automatic.

pub mod action_mapping;
pub mod attention;
pub mod chunk_id;
pub mod context;
pub mod definitions;
pub mod domain;
pub mod integration;
pub mod learning;
pub mod library;
pub mod resolution;
pub mod spawn_loadouts;
pub mod history;

pub use action_mapping::{action_requires_skill, get_chunks_for_action};
pub use attention::{calculate_attention_budget, can_afford_attention, risks_fumble, FUMBLE_ATTENTION_THRESHOLD};
pub use chunk_id::ChunkId;
pub use context::{CombatContext, ContextTag};
pub use definitions::{get_chunk_definition, ChunkComponents, ChunkDefinition, CHUNK_LIBRARY};
pub use domain::ChunkDomain;
pub use learning::{calculate_encoding_depth, process_learning};
pub use library::{ChunkLibrary, Experience, PersonalChunkState};
pub use resolution::{
    find_best_chunk, resolve_attack, resolve_defense, resolve_riposte, ActionResult,
    ATTACK_CHUNKS, DEFENSE_CHUNKS, RIPOSTE_CHUNKS,
};
pub use integration::{
    record_action_experience, refresh_attention, skill_check, spend_attention, SkillCheckResult,
    SkillFailure,
};
pub use spawn_loadouts::generate_spawn_chunks;
