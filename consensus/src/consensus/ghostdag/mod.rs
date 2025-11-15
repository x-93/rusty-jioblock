//! GHOSTDAG consensus implementation
//!
//! This module implements the GHOSTDAG protocol for BlockDAG consensus,
//! including blue set selection, parent ordering, and score calculation.

pub mod protocol;
pub mod stores;
pub mod manager;
#[cfg(test)]
mod integration_test;

pub use protocol::GhostdagProtocol;
pub use stores::{GhostdagData, GhostdagStore};
pub use manager::GhostdagManager;
