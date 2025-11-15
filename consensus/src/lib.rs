//! Consensus library for BlockDAG-based blockchain
//!
//! This library implements the core consensus logic using GHOSTDAG algorithm,
//! including DAG management, reachability queries, and protocol rules.

pub mod consensus;
pub mod pipeline;
pub mod process;

// Re-export key types for easier access
pub use consensus_core::Hash;
pub use consensus::dag::{BlockRelations, ReachabilityStore, DagTopology};
pub use consensus::ghostdag::{GhostdagData, GhostdagStore, GhostdagProtocol, GhostdagManager};
pub use consensus::validation::{
    BlockValidator, HeaderValidator, TransactionValidator, ContextualValidator,
};
pub use consensus::difficulty::{DifficultyManager, DifficultyWindow};
pub use consensus::storage::{ConsensusStorage, UtxoSet, BlockStore};
pub use consensus::types::{BlockStatus, ConsensusConfig, BlockProcessingResult, ValidationResult};

// Re-export pipeline types
pub use pipeline::{
    BlockProcessor, HeaderProcessor, BodyProcessor, VirtualProcessor, DepsManager,
};
pub use pipeline::flow::{ProcessQueue, ValidationFlow};
