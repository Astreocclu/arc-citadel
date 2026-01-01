//! Hierarchy operations for polity parent-child relationships
//!
//! Provides queries for traversing the polity hierarchy tree.
//! All polities form a forest (collection of trees) where each tree
//! is rooted at a sovereign polity.

use crate::core::types::PolityId;
use crate::aggregate::polity::Polity;
