//! Unified Parameterized Blueprint System
//!
//! This module provides a flexible blueprint system for defining game entities
//! with parameterized expressions. Blueprints can reference other blueprints,
//! use expressions for computed values, and support inheritance.

pub mod construction;
pub mod damage;
pub mod expression;
pub mod instance;
pub mod registry;
pub mod schema;

pub use construction::{apply_work, get_labor_cap, get_required_materials};
pub use damage::{apply_damage, find_damage_state, DamageResult};
pub use expression::{BinOp, EvalError, Expr, ParseError, UnaryOp};
pub use instance::*;
pub use registry::*;
pub use schema::*;
