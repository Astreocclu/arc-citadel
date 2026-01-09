//! Unified Parameterized Blueprint System
//!
//! This module provides a flexible blueprint system for defining game entities
//! with parameterized expressions. Blueprints can reference other blueprints,
//! use expressions for computed values, and support inheritance.

pub mod expression;
pub mod instance;
pub mod schema;

pub use expression::{BinOp, EvalError, Expr, ParseError, UnaryOp};
pub use instance::*;
pub use schema::*;
