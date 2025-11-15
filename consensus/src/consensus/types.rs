//! Consensus-specific types
//!
//! This module defines types used throughout the consensus module.

use consensus_core::Hash;

/// Block status in the consensus pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockStatus {
    /// Block is valid and accepted
    Valid,
    /// Block is invalid
    Invalid,
    /// Block is orphaned (missing parents)
    Orphan,
    /// Block header is valid but body not yet processed
    HeaderOnly,
}

/// Validation result
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Validation passed
    Valid,
    /// Validation failed with error
    Invalid(String),
}

/// Block processing result
#[derive(Debug, Clone)]
pub struct BlockProcessingResult {
    /// Block status
    pub status: BlockStatus,
    /// Block hash
    pub hash: Hash,
    /// Error message if processing failed
    pub error: Option<String>,
}

impl BlockProcessingResult {
    /// Create a successful result
    pub fn success(hash: Hash) -> Self {
        Self {
            status: BlockStatus::Valid,
            hash,
            error: None,
        }
    }

    /// Create a failed result
    pub fn failure(hash: Hash, error: String) -> Self {
        Self {
            status: BlockStatus::Invalid,
            hash,
            error: Some(error),
        }
    }

    /// Create an orphan result
    pub fn orphan(hash: Hash) -> Self {
        Self {
            status: BlockStatus::Orphan,
            hash,
            error: None,
        }
    }
}

/// Consensus configuration
#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    /// GHOSTDAG K parameter
    pub ghostdag_k: u32,
    /// Maximum number of parents per block
    pub max_block_parents: usize,
    /// Target time per block (in seconds)
    pub target_time_per_block: u64,
    /// Difficulty adjustment window size
    pub difficulty_window_size: u64,
    /// Maximum block size (bytes)
    pub max_block_size: u64,
    /// Coinbase maturity (blocks)
    pub coinbase_maturity: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            ghostdag_k: 18,
            max_block_parents: 10,
            target_time_per_block: 1,
            difficulty_window_size: 2641,
            max_block_size: 1_000_000,
            coinbase_maturity: 100,
        }
    }
}

