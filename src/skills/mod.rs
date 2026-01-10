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
pub mod display;
pub mod domain;
pub mod learning;
pub mod library;
pub mod resolution;
pub mod species_mods;

pub use attention::{
    calculate_attention_budget, can_afford_attention, risks_fumble, FUMBLE_ATTENTION_THRESHOLD,
};
pub use chunk_id::ChunkId;
pub use context::{CombatContext, ContextTag};
pub use definitions::{get_chunk_definition, ChunkComponents, ChunkDefinition, CHUNK_LIBRARY};
pub use display::{
    compute_athleticism, compute_charisma, compute_combat, compute_craftsmanship,
    compute_leadership, compute_medicine, compute_scholarship, DisplayStat, SkillLevel,
};
pub use domain::ChunkDomain;
pub use learning::{calculate_encoding_depth, process_learning, process_learning_with_modifiers};
pub use library::{ChunkLibrary, DomainSummary, Experience, PersonalChunkState};
pub use resolution::{
    find_best_chunk, resolve_attack, resolve_crossbow_reload, resolve_defense, resolve_ranged_attack,
    resolve_riposte, ActionResult, ATTACK_CHUNKS, BOW_ATTACK_CHUNKS, CROSSBOW_ATTACK_CHUNKS,
    DEFENSE_CHUNKS, RANGED_ATTACK_CHUNKS, RIPOSTE_CHUNKS, THROWN_ATTACK_CHUNKS,
};
pub use species_mods::{DomainModifier, SpeciesChunkModifiers};
