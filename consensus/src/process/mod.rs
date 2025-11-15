//! Background processes for consensus
//!
//! This module provides background processes for mining, synchronization,
//! and block relay operations.

pub mod mining;
pub mod sync;
pub mod relay;

pub mod coinbase;

pub mod parents_builder;
pub mod past_median_time;
pub mod pruning;
pub mod pruning_proof;



pub use mining::MiningProcess;
pub use sync::SyncProcess;
pub use relay::RelayProcess;
