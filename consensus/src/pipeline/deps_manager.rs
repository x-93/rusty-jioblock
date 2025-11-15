//! Dependency manager for orphan block handling
//!
//! This module manages orphan blocks (blocks with missing parents) and
//! resolves dependencies when parents become available.

use consensus_core::block::Block;
use consensus_core::header::Header;
use consensus_core::Hash;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Dependency manager for orphan blocks
pub struct DepsManager {
    /// Orphan blocks indexed by their hash
    orphans: Arc<RwLock<HashMap<Hash, Block>>>,
    /// Orphan headers indexed by their hash
    orphan_headers: Arc<RwLock<HashMap<Hash, Header>>>,
    /// Blocks waiting for specific parent hashes
    waiting_for_parents: Arc<RwLock<HashMap<Hash, Vec<Hash>>>>,
}

impl DepsManager {
    /// Create a new dependency manager
    pub fn new() -> Self {
        Self {
            orphans: Arc::new(RwLock::new(HashMap::new())),
            orphan_headers: Arc::new(RwLock::new(HashMap::new())),
            waiting_for_parents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add an orphan block
    pub fn add_orphan_block(&self, block: Block) {
        let hash = block.header.hash;
        let parents: Vec<Hash> = block.header.parents_by_level.iter()
            .flat_map(|level| level.iter().cloned())
            .collect();
        
        let mut orphans = self.orphans.write().unwrap();
        orphans.insert(hash, block);
        
        // Track which parents this block is waiting for
        let mut waiting = self.waiting_for_parents.write().unwrap();
        for parent in parents {
            waiting.entry(parent).or_insert_with(Vec::new).push(hash);
        }
    }

    /// Add an orphan header
    pub fn add_orphan_header(&self, header: Header) {
        let hash = header.hash;
        let parents: Vec<Hash> = header.parents_by_level.iter()
            .flat_map(|level| level.iter().cloned())
            .collect();
        
        let mut orphan_headers = self.orphan_headers.write().unwrap();
        orphan_headers.insert(hash, header);
        
        // Track which parents this header is waiting for
        let mut waiting = self.waiting_for_parents.write().unwrap();
        for parent in parents {
            waiting.entry(parent).or_insert_with(Vec::new).push(hash);
        }
    }

    /// Check if a block is an orphan
    pub fn is_orphan(&self, hash: &Hash) -> bool {
        let orphans = self.orphans.read().unwrap();
        orphans.contains_key(hash)
    }

    /// Check if a header is an orphan
    pub fn is_orphan_header(&self, hash: &Hash) -> bool {
        let orphan_headers = self.orphan_headers.read().unwrap();
        orphan_headers.contains_key(hash)
    }

    /// Get an orphan block
    pub fn get_orphan_block(&self, hash: &Hash) -> Option<Block> {
        let orphans = self.orphans.read().unwrap();
        orphans.get(hash).cloned()
    }

    /// Get an orphan header
    pub fn get_orphan_header(&self, hash: &Hash) -> Option<Header> {
        let orphan_headers = self.orphan_headers.read().unwrap();
        orphan_headers.get(hash).cloned()
    }

    /// Remove an orphan block
    pub fn remove_orphan_block(&self, hash: &Hash) -> Option<Block> {
        let mut orphans = self.orphans.write().unwrap();
        orphans.remove(hash)
    }

    /// Remove an orphan header
    pub fn remove_orphan_header(&self, hash: &Hash) -> Option<Header> {
        let mut orphan_headers = self.orphan_headers.write().unwrap();
        orphan_headers.remove(hash)
    }

    /// Get blocks that were waiting for a specific parent
    pub fn get_blocks_waiting_for(&self, parent_hash: &Hash) -> Vec<Hash> {
        let waiting = self.waiting_for_parents.read().unwrap();
        waiting.get(parent_hash).cloned().unwrap_or_default()
    }

    /// Remove waiting dependency
    pub fn remove_waiting_dependency(&self, parent_hash: &Hash, child_hash: &Hash) {
        let mut waiting = self.waiting_for_parents.write().unwrap();
        if let Some(children) = waiting.get_mut(parent_hash) {
            children.retain(|&h| h != *child_hash);
            if children.is_empty() {
                waiting.remove(parent_hash);
            }
        }
    }

    /// Check if all parents exist for a block
    pub fn all_parents_exist(&self, block: &Block, parent_checker: &dyn Fn(&Hash) -> bool) -> bool {
        for parent_level in &block.header.parents_by_level {
            for parent in parent_level {
                if !parent_checker(parent) {
                    return false;
                }
            }
        }
        true
    }

    /// Get all orphan blocks
    pub fn get_all_orphans(&self) -> Vec<Block> {
        let orphans = self.orphans.read().unwrap();
        orphans.values().cloned().collect()
    }

    /// Get all orphan headers
    pub fn get_all_orphan_headers(&self) -> Vec<Header> {
        let orphan_headers = self.orphan_headers.read().unwrap();
        orphan_headers.values().cloned().collect()
    }

    /// Get number of orphan blocks
    pub fn orphan_count(&self) -> usize {
        let orphans = self.orphans.read().unwrap();
        orphans.len()
    }

    /// Get number of orphan headers
    pub fn orphan_header_count(&self) -> usize {
        let orphan_headers = self.orphan_headers.read().unwrap();
        orphan_headers.len()
    }

    /// Clear all orphans (for testing)
    pub fn clear(&self) {
        let mut orphans = self.orphans.write().unwrap();
        orphans.clear();
        let mut orphan_headers = self.orphan_headers.write().unwrap();
        orphan_headers.clear();
        let mut waiting = self.waiting_for_parents.write().unwrap();
        waiting.clear();
    }
}

impl Default for DepsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::{ZERO_HASH, BlueWorkType};

    fn create_test_block(parents: Vec<Hash>) -> Block {
        let header = Header::new_finalized(
            1,
            vec![parents],
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
    fn test_add_orphan() {
        let deps = DepsManager::new();
        let parent = Hash::from_le_u64([1, 0, 0, 0]);
        let block = create_test_block(vec![parent]);
        let hash = block.header.hash;

        deps.add_orphan_block(block);
        assert!(deps.is_orphan(&hash));
        let retrieved = deps.get_orphan_block(&hash);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.as_ref().unwrap().header.hash, hash);
    }

    #[test]
    fn test_waiting_for_parents() {
        let deps = DepsManager::new();
        let parent = Hash::from_le_u64([1, 0, 0, 0]);
        let block = create_test_block(vec![parent]);
        let block_hash = block.header.hash;

        deps.add_orphan_block(block);
        let waiting = deps.get_blocks_waiting_for(&parent);
        assert!(waiting.contains(&block_hash));
    }

    #[test]
    fn test_all_parents_exist() {
        let deps = DepsManager::new();
        let parent1 = Hash::from_le_u64([1, 0, 0, 0]);
        let parent2 = Hash::from_le_u64([2, 0, 0, 0]);
        let block = create_test_block(vec![parent1, parent2]);

        let mut known_blocks = HashSet::new();
        known_blocks.insert(parent1);

        // Not all parents exist
        assert!(!deps.all_parents_exist(&block, &|h| known_blocks.contains(h)));

        // All parents exist
        known_blocks.insert(parent2);
        assert!(deps.all_parents_exist(&block, &|h| known_blocks.contains(h)));
    }
}

