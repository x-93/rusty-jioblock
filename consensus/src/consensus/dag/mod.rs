//! DAG (Directed Acyclic Graph) management for BlockDAG consensus
//!
//! This module provides:
//! - Block relationship tracking (parents/children)
//! - O(1) reachability queries (ancestor checks)
//! - DAG topology operations (tips, anticone, ordering)

pub mod relations;
pub mod reachability;
pub mod topology;
#[cfg(test)]
mod integration_test;

pub use relations::BlockRelations;
pub use reachability::{ReachabilityStore, Interval};
pub use topology::DagTopology;
