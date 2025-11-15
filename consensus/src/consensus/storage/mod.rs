//! Storage module for consensus
//!
//! This module provides storage interfaces for blocks, UTXOs, and consensus data.

pub mod consensus_db;
pub mod utxo_set;
pub mod block_store;

pub use consensus_db::ConsensusStorage;
pub use utxo_set::UtxoSet;
pub use block_store::BlockStore;

