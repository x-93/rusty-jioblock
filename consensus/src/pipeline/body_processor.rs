//! Body processor for consensus
//!
//! This module processes block bodies (transactions) and updates the UTXO set.

use consensus_core::block::Block;
use consensus_core::Hash;
use consensus_core::errors::ConsensusError;
use crate::consensus::validation::{BlockValidator, ContextualValidator};
use crate::consensus::storage::{BlockStore, UtxoSet};
use crate::consensus::validation::transaction_validator::UtxoView;
use std::sync::Arc;
use std::collections::HashMap;

/// Body processor for transaction processing
pub struct BodyProcessor {
    block_validator: Arc<BlockValidator>,
    contextual_validator: Arc<ContextualValidator>,
    block_store: Arc<BlockStore>,
    utxo_set: Arc<UtxoSet>,
}

impl BodyProcessor {
    /// Create a new body processor
    pub fn new(
        block_validator: Arc<BlockValidator>,
        contextual_validator: Arc<ContextualValidator>,
        block_store: Arc<BlockStore>,
        utxo_set: Arc<UtxoSet>,
    ) -> Self {
        Self {
            block_validator,
            contextual_validator,
            block_store,
            utxo_set,
        }
    }

    /// Process block body (transactions)
    pub fn process_body(&self, block: &Block, block_daa_score: u64) -> Result<BodyProcessingResult, ConsensusError> {
        let hash = block.header.hash;

        // Check if block already exists
        if self.block_store.has_block(&hash) {
            return Ok(BodyProcessingResult::AlreadyExists(hash));
        }

        // Validate block structure
        self.block_validator.validate_block(block)?;

        // Create UTXO view from current UTXO set snapshot
        let utxo_snapshot = self.utxo_set.snapshot();
        let utxo_view = SnapshotUtxoView::new(utxo_snapshot);

        // Validate block with UTXO context
        let total_fees = self.contextual_validator.validate_block_with_utxo(
            block,
            &utxo_view,
            block_daa_score,
        )?;

        // Apply block to UTXO set
        self.utxo_set.apply_block(block, block_daa_score)?;

        // Store block
        self.block_store.store_block(block.clone())?;

        Ok(BodyProcessingResult::Accepted {
            hash,
            total_fees,
        })
    }

    /// Validate block body without applying it
    pub fn validate_body(&self, block: &Block, block_daa_score: u64) -> Result<u64, ConsensusError> {
        // Validate block structure
        self.block_validator.validate_block(block)?;

        // Create UTXO view from current UTXO set snapshot
        let utxo_snapshot = self.utxo_set.snapshot();
        let utxo_view = SnapshotUtxoView::new(utxo_snapshot);

        // Validate block with UTXO context
        let total_fees = self.contextual_validator.validate_block_with_utxo(
            block,
            &utxo_view,
            block_daa_score,
        )?;

        Ok(total_fees)
    }
}

/// Result of body processing
#[derive(Debug, Clone)]
pub enum BodyProcessingResult {
    /// Body was accepted and processed
    Accepted {
        hash: Hash,
        total_fees: u64,
    },
    /// Body already exists
    AlreadyExists(Hash),
}

impl BodyProcessingResult {
    /// Get the hash from the result
    pub fn hash(&self) -> Hash {
        match self {
            BodyProcessingResult::Accepted { hash, .. } => *hash,
            BodyProcessingResult::AlreadyExists(hash) => *hash,
        }
    }

    /// Check if the body was accepted
    pub fn is_accepted(&self) -> bool {
        matches!(self, BodyProcessingResult::Accepted { .. })
    }
}

/// UTXO view implementation for snapshot
struct SnapshotUtxoView {
    snapshot: HashMap<consensus_core::tx::TransactionOutpoint, consensus_core::tx::UtxoEntry>,
}

impl SnapshotUtxoView {
    fn new(snapshot: HashMap<consensus_core::tx::TransactionOutpoint, consensus_core::tx::UtxoEntry>) -> Self {
        Self { snapshot }
    }
}

impl UtxoView for SnapshotUtxoView {
    fn get(&self, outpoint: &consensus_core::tx::TransactionOutpoint) -> Option<&consensus_core::tx::UtxoEntry> {
        self.snapshot.get(outpoint)
    }
}

