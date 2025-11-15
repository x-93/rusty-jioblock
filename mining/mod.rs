//! Mining module for proof-of-work consensus
//!
//! This module implements the core mining functionality including proof-of-work
//! verification, mining job management, worker threads, and difficulty adjustment.
//! It integrates with the block template system, mempool for transaction selection,
//! and RPC models for communication.
//!
//! ## Module Organization
//!
//! - [`pow`]: Proof-of-Work hashing and verification logic using Blake3
//! - [`job`]: Mining job definitions and mined block results
//! - [`worker`]: Multithreaded worker implementation for mining operations
//! - [`manager`]: Coordinates multiple workers and manages mining sessions
//! - [`difficulty`]: Difficulty adjustment algorithm (DAA) similar to Kaspa

pub mod pow;
pub mod job;
pub mod worker;
pub mod manager;
pub mod difficulty;
pub mod rpc_miner;

#[cfg(test)]
pub mod tests;

// Re-export main types for easier access
pub use pow::{ProofOfWork, Target};
pub use job::{MiningJob, MinedBlock};
pub use worker::{MinerWorker, WorkerStats};
pub use manager::{MiningManager, MiningConfig, MiningResult, SessionStats};
pub use difficulty::{DifficultyManager, DifficultyConfig};
pub use rpc_miner::{RpcMiner, RpcMinerConfig, MiningStats};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::pow::{ProofOfWork, Target};
    pub use crate::job::{MiningJob, MinedBlock};
    pub use crate::worker::{MinerWorker, WorkerStats};
    pub use crate::manager::{MiningManager, MiningConfig, MiningResult, SessionStats};
    pub use crate::difficulty::{DifficultyManager, DifficultyConfig};
    pub use crate::rpc_miner::{RpcMiner, RpcMinerConfig, MiningStats};
}
