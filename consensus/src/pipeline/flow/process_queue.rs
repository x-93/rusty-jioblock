//! Process queue for block processing
//!
//! This module provides a queue for managing block processing order.

use consensus_core::block::Block;
use consensus_core::Hash;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

/// Process queue for blocks
pub struct ProcessQueue {
    queue: Arc<RwLock<VecDeque<Block>>>,
    pending: Arc<RwLock<std::collections::HashSet<Hash>>>,
}

impl ProcessQueue {
    /// Create a new process queue
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            pending: Arc::new(RwLock::new(std::collections::HashSet::new())),
        }
    }

    /// Add a block to the queue
    pub fn enqueue(&self, block: Block) {
        let hash = block.header.hash;
        let mut queue = self.queue.write().unwrap();
        let mut pending = self.pending.write().unwrap();
        
        if !pending.contains(&hash) {
            queue.push_back(block);
            pending.insert(hash);
        }
    }

    /// Remove and return the next block from the queue
    pub fn dequeue(&self) -> Option<Block> {
        let mut queue = self.queue.write().unwrap();
        let mut pending = self.pending.write().unwrap();
        
        if let Some(block) = queue.pop_front() {
            let hash = block.header.hash;
            pending.remove(&hash);
            Some(block)
        } else {
            None
        }
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        let queue = self.queue.read().unwrap();
        queue.is_empty()
    }

    /// Get the number of blocks in the queue
    pub fn len(&self) -> usize {
        let queue = self.queue.read().unwrap();
        queue.len()
    }

    /// Check if a block is pending
    pub fn is_pending(&self, hash: &Hash) -> bool {
        let pending = self.pending.read().unwrap();
        pending.contains(hash)
    }

    /// Clear the queue
    pub fn clear(&self) {
        let mut queue = self.queue.write().unwrap();
        let mut pending = self.pending.write().unwrap();
        queue.clear();
        pending.clear();
    }
}

impl Default for ProcessQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::{ZERO_HASH, BlueWorkType};

    fn create_test_block() -> Block {
        let header = consensus_core::header::Header::new_finalized(
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
    fn test_enqueue_dequeue() {
        let queue = ProcessQueue::new();
        let block = create_test_block();
        let hash = block.header.hash;

        assert!(queue.is_empty());
        queue.enqueue(block.clone());
        assert!(!queue.is_empty());
        assert!(queue.is_pending(&hash));

        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.header.hash, hash);
        assert!(queue.is_empty());
        assert!(!queue.is_pending(&hash));
    }

    #[test]
    fn test_duplicate_enqueue() {
        let queue = ProcessQueue::new();
        let block = create_test_block();

        queue.enqueue(block.clone());
        queue.enqueue(block.clone()); // Should not add duplicate

        assert_eq!(queue.len(), 1);
    }
}

