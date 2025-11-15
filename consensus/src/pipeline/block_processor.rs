//! Block processor for consensus
//!
//! This module provides the main block processing logic that orchestrates
//! header processing, body processing, and state updates.

use consensus_core::block::Block;
use consensus_core::Hash;
use consensus_core::errors::ConsensusError;
use crate::consensus::types::BlockStatus;
use crate::pipeline::header_processor::HeaderProcessor;
use crate::pipeline::body_processor::BodyProcessor;
use crate::pipeline::virtual_processor::VirtualProcessor;
use crate::pipeline::deps_manager::DepsManager;
use crate::consensus::ghostdag::GhostdagManager;
use crate::consensus::storage::ConsensusStorage;
use std::sync::Arc;

/// Block processor for consensus
pub struct BlockProcessor {
    header_processor: Arc<HeaderProcessor>,
    body_processor: Arc<BodyProcessor>,
    virtual_processor: Arc<VirtualProcessor>,
    ghostdag_manager: Arc<GhostdagManager>,
    storage: Arc<ConsensusStorage>,
    deps_manager: Arc<DepsManager>,
}

impl BlockProcessor {
    /// Create a new block processor
    pub fn new(
        header_processor: Arc<HeaderProcessor>,
        body_processor: Arc<BodyProcessor>,
        virtual_processor: Arc<VirtualProcessor>,
        ghostdag_manager: Arc<GhostdagManager>,
        storage: Arc<ConsensusStorage>,
        deps_manager: Arc<DepsManager>,
    ) -> Self {
        Self {
            header_processor,
            body_processor,
            virtual_processor,
            ghostdag_manager,
            storage,
            deps_manager,
        }
    }

    /// Process a complete block
    pub fn process_block(&self, block: Block) -> Result<BlockProcessingResult, ConsensusError> {
        let hash = block.header.hash;

        // Check if block already exists
        if self.storage.has_block(&hash) {
            return Ok(BlockProcessingResult::already_exists(hash));
        }

        // Step 1: Process header
        let header_result = self.header_processor.process_header(block.header.clone())?;
        
        match header_result {
            crate::pipeline::header_processor::HeaderProcessingResult::Orphan(_) => {
                // Header is orphaned, store block as orphan
                self.deps_manager.add_orphan_block(block);
                return Ok(BlockProcessingResult::orphan(hash));
            }
            crate::pipeline::header_processor::HeaderProcessingResult::Invalid(hash, msg) => {
                return Ok(BlockProcessingResult::invalid(hash, msg));
            }
            crate::pipeline::header_processor::HeaderProcessingResult::AlreadyExists(_) => {
                return Ok(BlockProcessingResult::already_exists(hash));
            }
            crate::pipeline::header_processor::HeaderProcessingResult::Accepted { .. } => {
                // Header is valid, continue to body processing
            }
        }

        // Step 2: Process body (transactions)
        // Get DAA score from header (or calculate from GHOSTDAG data)
        let _ghostdag_data = self.ghostdag_manager.get_ghostdag_data(&hash)
            .ok_or_else(|| ConsensusError::Other("GHOSTDAG data not found".to_string()))?;
        
        // Use DAA score from header, or calculate from ghostdag data
        let daa_score = block.header.daa_score;
        
        let body_result = self.body_processor.process_body(&block, daa_score)?;

        match body_result {
            crate::pipeline::body_processor::BodyProcessingResult::AlreadyExists(_) => {
                return Ok(BlockProcessingResult::already_exists(hash));
            }
            crate::pipeline::body_processor::BodyProcessingResult::Accepted { total_fees, .. } => {
                // Block successfully processed
                Ok(BlockProcessingResult::valid(hash, total_fees))
            }
        }
    }

    /// Process header only (for fast sync)
    pub fn process_header_only(&self, header: consensus_core::header::Header) -> Result<BlockStatus, ConsensusError> {
        let result = self.header_processor.process_header(header)?;
        
        match result {
            crate::pipeline::header_processor::HeaderProcessingResult::Accepted { .. } => {
                Ok(BlockStatus::HeaderOnly)
            }
            crate::pipeline::header_processor::HeaderProcessingResult::Orphan(_) => {
                Ok(BlockStatus::Orphan)
            }
            crate::pipeline::header_processor::HeaderProcessingResult::AlreadyExists(_) => {
                Ok(BlockStatus::Valid) // Already processed
            }
            crate::pipeline::header_processor::HeaderProcessingResult::Invalid(_, _) => {
                Ok(BlockStatus::Invalid)
            }
        }
    }

    /// Process orphan blocks that may now be valid
    pub fn process_orphans(&self) -> Vec<BlockProcessingResult> {
        let orphan_blocks = self.deps_manager.get_all_orphans();
        let mut results = Vec::new();

        for block in orphan_blocks {
            let hash = block.header.hash;
            // Try to process the orphan block
            match self.process_block(block.clone()) {
                Ok(result) => {
                    match result.status {
                        BlockStatus::Valid => {
                            // Successfully processed, remove from orphans
                            self.deps_manager.remove_orphan_block(&hash);
                            results.push(result);
                        }
                        BlockStatus::Orphan => {
                            // Still an orphan, keep it
                        }
                        BlockStatus::Invalid => {
                            // Invalid, remove from orphans
                            self.deps_manager.remove_orphan_block(&hash);
                            results.push(result);
                        }
                        BlockStatus::HeaderOnly => {
                            // Should not happen for full blocks
                        }
                    }
                }
                Err(_) => {
                    // Error processing, remove from orphans
                    self.deps_manager.remove_orphan_block(&hash);
                }
            }
        }

        results
    }

    /// Get virtual block data for mining
    pub fn get_virtual_block_data(&self, max_parents: usize) -> Result<crate::pipeline::virtual_processor::VirtualBlockData, String> {
        self.virtual_processor.get_virtual_block_data(max_parents)
    }

    /// Get ghostdag manager reference
    pub fn ghostdag_manager(&self) -> Arc<GhostdagManager> {
        self.ghostdag_manager.clone()
    }

    /// Get storage reference
    pub fn storage(&self) -> Arc<ConsensusStorage> {
        self.storage.clone()
    }
}

/// Result of block processing
#[derive(Debug, Clone)]
pub struct BlockProcessingResult {
    /// Block status
    pub status: BlockStatus,
    /// Block hash
    pub hash: Hash,
    /// Total fees collected (if valid)
    pub total_fees: Option<u64>,
    /// Error message (if invalid)
    pub error: Option<String>,
}

impl BlockProcessingResult {
    /// Create a valid result
    pub fn valid(hash: Hash, total_fees: u64) -> Self {
        Self {
            status: BlockStatus::Valid,
            hash,
            total_fees: Some(total_fees),
            error: None,
        }
    }

    /// Create an invalid result
    pub fn invalid(hash: Hash, error: String) -> Self {
        Self {
            status: BlockStatus::Invalid,
            hash,
            total_fees: None,
            error: Some(error),
        }
    }

    /// Create an orphan result
    pub fn orphan(hash: Hash) -> Self {
        Self {
            status: BlockStatus::Orphan,
            hash,
            total_fees: None,
            error: None,
        }
    }

    /// Create an already exists result
    pub fn already_exists(hash: Hash) -> Self {
        Self {
            status: BlockStatus::Valid, // Considered valid if already exists
            hash,
            total_fees: None,
            error: None,
        }
    }

    /// Check if the block was accepted
    pub fn is_valid(&self) -> bool {
        matches!(self.status, BlockStatus::Valid)
    }

    /// Check if the block is orphaned
    pub fn is_orphan(&self) -> bool {
        matches!(self.status, BlockStatus::Orphan)
    }
}

