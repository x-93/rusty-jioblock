//! Block synchronization process
//!
//! This module implements block synchronization from peers,
//! including initial block download (IBD) and gap filling.

use crate::pipeline::BlockProcessor;
use crate::consensus::storage::BlockStore;
use crate::consensus::types::BlockStatus;
use consensus_core::block::Block;
use consensus_core::Hash;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

/// Block synchronization process
pub struct SyncProcess {
    processor: Arc<BlockProcessor>,
    block_store: Arc<BlockStore>,
    requested_blocks: std::sync::RwLock<HashSet<Hash>>,
    sync_queue: std::sync::RwLock<VecDeque<Hash>>,
}

impl SyncProcess {
    /// Create a new sync process
    pub fn new(processor: Arc<BlockProcessor>, block_store: Arc<BlockStore>) -> Self {
        Self {
            processor,
            block_store,
            requested_blocks: std::sync::RwLock::new(HashSet::new()),
            sync_queue: std::sync::RwLock::new(VecDeque::new()),
        }
    }

    /// Start initial block download
    pub fn start_ibd(&self, target_hashes: Vec<Hash>) -> Result<(), String> {
        let mut queue = self.sync_queue.write().unwrap();
        let mut requested = self.requested_blocks.write().unwrap();

        for hash in target_hashes {
            if !self.block_store.has_block(&hash) && !requested.contains(&hash) {
                queue.push_back(hash);
                requested.insert(hash);
            }
        }

        Ok(())
    }

    /// Process received block during sync
    pub fn process_sync_block(&self, block: Block) -> Result<BlockStatus, String> {
        let hash = block.header.hash;

        // Remove from requested set
        {
            let mut requested = self.requested_blocks.write().unwrap();
            requested.remove(&hash);
        }

        // Process the block
        let result = self.processor.process_block(block).map_err(|e| format!("{:?}", e))?;

        // If valid, check if we can request more blocks
        if result.status == BlockStatus::Valid {
            self.request_next_blocks()?;
        }

        Ok(result.status)
    }

    /// Get next blocks to request
    pub fn get_blocks_to_request(&self, max_count: usize) -> Vec<Hash> {
        let mut queue = self.sync_queue.write().unwrap();
        let mut result = Vec::new();

        while result.len() < max_count && !queue.is_empty() {
            if let Some(hash) = queue.pop_front() {
                result.push(hash);
            }
        }

        result
    }

    /// Check if sync is complete
    pub fn is_sync_complete(&self) -> bool {
        self.sync_queue.read().unwrap().is_empty() &&
        self.requested_blocks.read().unwrap().is_empty()
    }

    /// Request next blocks based on newly processed blocks
    fn request_next_blocks(&self) -> Result<(), String> {
        // In a real implementation, this would analyze the DAG to find
        // missing ancestors or descendants that need to be requested
        // For now, this is a placeholder
        Ok(())
    }

    /// Handle missing block during sync
    pub fn handle_missing_block(&self, hash: Hash) -> Result<(), String> {
        let mut requested = self.requested_blocks.write().unwrap();

        if !requested.contains(&hash) && !self.block_store.has_block(&hash) {
            let mut queue = self.sync_queue.write().unwrap();
            queue.push_back(hash);
            requested.insert(hash);
        }

        Ok(())
    }

    /// Get sync progress (0.0 to 1.0)
    pub fn get_sync_progress(&self) -> f64 {
        // Placeholder progress calculation
        // In real implementation, this would track downloaded vs total blocks
        let queue_len = self.sync_queue.read().unwrap().len();
        let requested_len = self.requested_blocks.read().unwrap().len();

        if queue_len + requested_len == 0 {
            1.0
        } else {
            // Simple heuristic: assume requested blocks are 50% complete
            0.5 / (queue_len + requested_len) as f64
        }
    }
}
