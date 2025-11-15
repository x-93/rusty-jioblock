//! Consensus database interface
//!
//! This module provides a unified interface for consensus storage including
//! blocks, UTXOs, and consensus data.

use consensus_core::block::Block;
use consensus_core::header::Header;
use consensus_core::Hash;
use consensus_core::errors::ConsensusError;
use super::block_store::BlockStore;
use super::utxo_set::UtxoSet;
use std::sync::Arc;

/// Consensus storage coordinator
pub struct ConsensusStorage {
    block_store: Arc<BlockStore>,
    utxo_set: Arc<UtxoSet>,
}

impl ConsensusStorage {
    /// Create a new consensus storage
    pub fn new() -> Self {
        Self {
            block_store: Arc::new(BlockStore::new()),
            utxo_set: Arc::new(UtxoSet::new()),
        }
    }

    /// Create a new consensus storage with existing stores
    pub fn with_stores(block_store: Arc<BlockStore>, utxo_set: Arc<UtxoSet>) -> Self {
        Self {
            block_store,
            utxo_set,
        }
    }

    /// Get block store reference
    pub fn block_store(&self) -> Arc<BlockStore> {
        self.block_store.clone()
    }

    /// Get UTXO set reference
    pub fn utxo_set(&self) -> Arc<UtxoSet> {
        self.utxo_set.clone()
    }

    /// Store a block
    pub fn store_block(&self, block: Block) -> Result<(), ConsensusError> {
        self.block_store.store_block(block)
    }

    /// Store a header only
    pub fn store_header(&self, header: Header) -> Result<(), ConsensusError> {
        self.block_store.store_header(header)
    }

    /// Get a block by hash
    pub fn get_block(&self, hash: &Hash) -> Option<Block> {
        self.block_store.get_block(hash)
    }

    /// Get a header by hash
    pub fn get_header(&self, hash: &Hash) -> Option<Header> {
        self.block_store.get_header(hash)
    }

    /// Check if a block exists
    pub fn has_block(&self, hash: &Hash) -> bool {
        self.block_store.has_block(hash)
    }

    /// Check if a header exists
    pub fn has_header(&self, hash: &Hash) -> bool {
        self.block_store.has_header(hash)
    }

    /// Apply a block to the UTXO set
    pub fn apply_block(&self, block: &Block, block_daa_score: u64) -> Result<(), ConsensusError> {
        // Store block first
        self.block_store.store_block(block.clone())?;
        
        // Then apply to UTXO set
        self.utxo_set.apply_block(block, block_daa_score)
    }

    /// Get UTXO set
    pub fn utxo_set_ref(&self) -> Arc<UtxoSet> {
        self.utxo_set.clone()
    }
}

impl Default for ConsensusStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::{ZERO_HASH, BlueWorkType};

    fn create_test_block() -> Block {
        let header = Header::new_finalized(
            1,
            vec![],
            ZERO_HASH,
            ZERO_HASH,
            ZERO_HASH,
            1000,
            0x1f00ffff,
            0,
            0,
            BlueWorkType::from(0u64),
            0,
            ZERO_HASH,
        );
        Block::new(header, Vec::new())
    }

    #[test]
    fn test_store_block() {
        let storage = ConsensusStorage::new();
        let block = create_test_block();
        let hash = block.header.hash;

        storage.store_block(block.clone()).unwrap();
        assert!(storage.has_block(&hash));
    }

    #[test]
    fn test_apply_block() {
        let storage = ConsensusStorage::new();
        let block = create_test_block();

        storage.apply_block(&block, 100).unwrap();
        assert!(storage.has_block(&block.header.hash));
    }
}

