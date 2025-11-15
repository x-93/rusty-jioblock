//! Header processor for consensus
//!
//! This module processes block headers independently of block bodies,
//! enabling fast header-only synchronization.

use consensus_core::header::Header;
use consensus_core::Hash;
use consensus_core::errors::ConsensusError;
use crate::consensus::validation::HeaderValidator;
use crate::consensus::ghostdag::GhostdagManager;
use crate::consensus::storage::BlockStore;
use crate::consensus::difficulty::DifficultyManager;
use crate::pipeline::deps_manager::DepsManager;
use std::sync::Arc;

/// Header processor for header-only processing
pub struct HeaderProcessor {
    header_validator: Arc<HeaderValidator>,
    ghostdag_manager: Arc<GhostdagManager>,
    block_store: Arc<BlockStore>,
    difficulty_manager: Arc<DifficultyManager>,
    deps_manager: Arc<DepsManager>,
}

impl HeaderProcessor {
    /// Create a new header processor
    pub fn new(
        header_validator: Arc<HeaderValidator>,
        ghostdag_manager: Arc<GhostdagManager>,
        block_store: Arc<BlockStore>,
        difficulty_manager: Arc<DifficultyManager>,
        deps_manager: Arc<DepsManager>,
    ) -> Self {
        Self {
            header_validator,
            ghostdag_manager,
            block_store,
            difficulty_manager,
            deps_manager,
        }
    }

    /// Process a header
    pub fn process_header(&self, header: Header) -> Result<HeaderProcessingResult, ConsensusError> {
        let hash = header.hash;

        // Check if header already exists
        if self.block_store.has_header(&hash) {
            return Ok(HeaderProcessingResult::AlreadyExists(hash));
        }

        // Validate header
        self.header_validator.validate_header(&header)?;

        // Check if all parents exist
        let all_parents_exist = self.check_parents_exist(&header);
        if !all_parents_exist {
            // Add as orphan header
            self.deps_manager.add_orphan_header(header);
            return Ok(HeaderProcessingResult::Orphan(hash));
        }

        // Calculate GHOSTDAG data
        let ghostdag_data = self.ghostdag_manager.add_block(&header)
            .map_err(|e| ConsensusError::Other(format!("GHOSTDAG calculation failed: {}", e)))?;

        // Update difficulty window (calculate_next_difficulty adds block to window)
        let _ = self.difficulty_manager.calculate_next_difficulty(&header);

        // Store header
        self.block_store.store_header(header.clone())?;

        // Check if any orphan blocks/headers were waiting for this header
        let waiting_blocks = self.deps_manager.get_blocks_waiting_for(&hash);
        for waiting_hash in waiting_blocks {
            self.deps_manager.remove_waiting_dependency(&hash, &waiting_hash);
        }

        Ok(HeaderProcessingResult::Accepted {
            hash,
            ghostdag_data,
        })
    }

    /// Check if all parents of a header exist
    fn check_parents_exist(&self, header: &Header) -> bool {
        for parent_level in &header.parents_by_level {
            for parent in parent_level {
                if !self.block_store.has_header(parent) && !self.block_store.has_block(parent) {
                    return false;
                }
            }
        }
        true
    }

    /// Process orphan headers that may now be valid
    pub fn process_orphan_headers(&self) -> Vec<HeaderProcessingResult> {
        let orphan_headers = self.deps_manager.get_all_orphan_headers();
        let mut results = Vec::new();

        for header in orphan_headers {
            let hash = header.hash;
            // Try to process the orphan header
            if let Ok(result) = self.process_header(header.clone()) {
                match result {
                    HeaderProcessingResult::Accepted { .. } => {
                        // Successfully processed, remove from orphans
                        self.deps_manager.remove_orphan_header(&hash);
                        results.push(result);
                    }
                    HeaderProcessingResult::Orphan(_) => {
                        // Still an orphan, keep it
                    }
                    HeaderProcessingResult::AlreadyExists(_) => {
                        // Already exists, remove from orphans
                        self.deps_manager.remove_orphan_header(&hash);
                        results.push(result);
                    }
                    HeaderProcessingResult::Invalid(_, _) => {
                        // Invalid, remove from orphans
                        self.deps_manager.remove_orphan_header(&hash);
                        results.push(result);
                    }
                }
            }
        }

        results
    }
}

/// Result of header processing
#[derive(Debug, Clone)]
pub enum HeaderProcessingResult {
    /// Header was accepted and processed
    Accepted {
        hash: Hash,
        ghostdag_data: crate::consensus::ghostdag::GhostdagData,
    },
    /// Header is orphaned (missing parents)
    Orphan(Hash),
    /// Header already exists
    AlreadyExists(Hash),
    /// Header is invalid
    Invalid(Hash, String),
}

impl HeaderProcessingResult {
    /// Get the hash from the result
    pub fn hash(&self) -> Hash {
        match self {
            HeaderProcessingResult::Accepted { hash, .. } => *hash,
            HeaderProcessingResult::Orphan(hash) => *hash,
            HeaderProcessingResult::AlreadyExists(hash) => *hash,
            HeaderProcessingResult::Invalid(hash, _) => *hash,
        }
    }

    /// Check if the header was accepted
    pub fn is_accepted(&self) -> bool {
        matches!(self, HeaderProcessingResult::Accepted { .. })
    }

    /// Check if the header is orphaned
    pub fn is_orphan(&self) -> bool {
        matches!(self, HeaderProcessingResult::Orphan(_))
    }
}

