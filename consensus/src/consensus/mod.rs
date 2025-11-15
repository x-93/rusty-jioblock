//! Consensus module for BlockDAG-based blockchain
//!
//! This module implements the core consensus logic using GHOSTDAG algorithm,
//! including DAG management, reachability queries, and protocol rules.

pub mod dag;
pub mod ghostdag;
pub mod validation;
pub mod difficulty;
pub mod storage;
pub mod types;

pub use dag::{BlockRelations, DagTopology, Interval, ReachabilityStore};
pub use ghostdag::{GhostdagData, GhostdagProtocol, GhostdagStore, GhostdagManager};
pub use validation::{
    BlockValidator, HeaderValidator, TransactionValidator, ContextualValidator,
};
pub use difficulty::{DifficultyManager, DifficultyWindow};
pub use storage::{ConsensusStorage, UtxoSet, BlockStore};
pub use types::{BlockStatus, ConsensusConfig, BlockProcessingResult, ValidationResult};
