//! Block pruning management
//!
//! This module implements block pruning logic to manage blockchain storage
//! by removing old blocks that are no longer needed for consensus.

use consensus_core::block::Block;
use consensus_core::header::Header as BlockHeader;
use consensus_core::Hash;
use consensus_core::hashing;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};

/// Pruning configuration
#[derive(Debug, Clone)]
pub struct PruningConfig {
    /// Number of blocks to keep from the pruning point
    pub pruning_depth: u64,
    /// Maximum number of blocks to prune in a single operation
    pub max_pruning_batch: usize,
    /// Whether pruning is enabled
    pub enabled: bool,
}

impl Default for PruningConfig {
    fn default() -> Self {
        Self {
            pruning_depth: 1000,
            max_pruning_batch: 100,
            enabled: true,
        }
    }
}

/// Block pruning manager
pub struct PruningManager {
    /// Pruning configuration
    config: PruningConfig,
    /// Blocks that are candidates for pruning
    pruning_candidates: RwLock<HashSet<Hash>>,
    /// Blocks that must be kept (references by future blocks)
    protected_blocks: RwLock<HashSet<Hash>>,
    /// Current pruning point
    pruning_point: RwLock<Option<Hash>>,
}

impl PruningManager {
    /// Create a new pruning manager
    pub fn new(config: PruningConfig) -> Self {
        Self {
            config,
            pruning_candidates: RwLock::new(HashSet::new()),
            protected_blocks: RwLock::new(HashSet::new()),
            pruning_point: RwLock::new(None),
        }
    }

    /// Add a block to the pruning candidates
    pub fn add_pruning_candidate(&self, block_hash: Hash) {
        if self.config.enabled {
            let mut candidates = self.pruning_candidates.write().unwrap();
            candidates.insert(block_hash);
        }
    }

    /// Mark a block as protected (cannot be pruned)
    pub fn mark_block_protected(&self, block_hash: Hash) {
        let mut protected = self.protected_blocks.write().unwrap();
        protected.insert(block_hash);
    }

    /// Check if a block can be pruned
    pub fn can_prune_block(&self, block_hash: &Hash) -> bool {
        if !self.config.enabled {
            return false;
        }

        let candidates = self.pruning_candidates.read().unwrap();
        let protected = self.protected_blocks.read().unwrap();

        candidates.contains(block_hash) && !protected.contains(block_hash)
    }

    /// Get blocks ready for pruning
    pub fn get_blocks_to_prune(&self) -> Vec<Hash> {
        if !self.config.enabled {
            return vec![];
        }

        let candidates = self.pruning_candidates.read().unwrap();
        let protected = self.protected_blocks.read().unwrap();

        candidates
            .iter()
            .filter(|hash| !protected.contains(hash))
            .take(self.config.max_pruning_batch)
            .cloned()
            .collect()
    }

    /// Mark blocks as pruned (remove from candidates)
    pub fn mark_blocks_pruned(&self, block_hashes: &[Hash]) {
        let mut candidates = self.pruning_candidates.write().unwrap();
        for hash in block_hashes {
            candidates.remove(hash);
        }
    }

    /// Update the pruning point
    pub fn update_pruning_point(&self, new_pruning_point: Hash) {
        let mut pruning_point = self.pruning_point.write().unwrap();
        *pruning_point = Some(new_pruning_point);
    }

    /// Get the current pruning point
    pub fn get_pruning_point(&self) -> Option<Hash> {
        *self.pruning_point.read().unwrap()
    }

    /// Calculate new pruning point based on current DAG state
    pub fn calculate_pruning_point(&self, tips: &[Hash], block_depths: &HashMap<Hash, u64>) -> Result<Hash, String> {
        if tips.is_empty() {
            return Err("No tips available for pruning point calculation".to_string());
        }

        // Find the tip with the minimum depth (oldest tip)
        let mut min_depth = u64::MAX;
        let mut pruning_point = tips[0];

        for tip in tips {
            if let Some(depth) = block_depths.get(tip) {
                if *depth < min_depth {
                    min_depth = *depth;
                    pruning_point = *tip;
                }
            }
        }

        // For MVP return the selected pruning point (simplified behaviour)
        // In a production implementation this should ensure the pruning point is
        // at least `pruning_depth` blocks back and walk back the DAG accordingly.
        Ok(pruning_point)
    }

    /// Get pruning statistics
    pub fn get_stats(&self) -> PruningStats {
        let candidates = self.pruning_candidates.read().unwrap();
        let protected = self.protected_blocks.read().unwrap();

        PruningStats {
            pruning_candidates: candidates.len(),
            protected_blocks: protected.len(),
            pruning_point: *self.pruning_point.read().unwrap(),
            enabled: self.config.enabled,
        }
    }

    /// Clear all pruning candidates (useful for reset operations)
    pub fn clear_candidates(&self) {
        let mut candidates = self.pruning_candidates.write().unwrap();
        candidates.clear();
    }
}

/// Pruning statistics
#[derive(Debug, Clone)]
pub struct PruningStats {
    /// Number of blocks that are candidates for pruning
    pub pruning_candidates: usize,
    /// Number of blocks that are protected from pruning
    pub protected_blocks: usize,
    /// Current pruning point
    pub pruning_point: Option<Hash>,
    /// Whether pruning is enabled
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_hash(i: u64) -> Hash {
        Hash::from_le_u64([i, 0, 0, 0])
    }

    #[test]
    fn test_pruning_manager_creation() {
        let config = PruningConfig::default();
        let manager = PruningManager::new(config.clone());

        let stats = manager.get_stats();
        assert_eq!(stats.pruning_candidates, 0);
        assert_eq!(stats.protected_blocks, 0);
        assert_eq!(stats.pruning_point, None);
        assert_eq!(stats.enabled, config.enabled);
    }

    #[test]
    fn test_add_pruning_candidate() {
        let manager = PruningManager::new(PruningConfig::default());
        let hash = create_test_hash(1);

        manager.add_pruning_candidate(hash);

        let stats = manager.get_stats();
        assert_eq!(stats.pruning_candidates, 1);
    }

    #[test]
    fn test_mark_block_protected() {
        let manager = PruningManager::new(PruningConfig::default());
        let hash = create_test_hash(1);

        manager.add_pruning_candidate(hash);
        manager.mark_block_protected(hash);

        assert!(!manager.can_prune_block(&hash));
    }

    #[test]
    fn test_can_prune_block() {
        let manager = PruningManager::new(PruningConfig::default());
        let hash1 = create_test_hash(1);
        let hash2 = create_test_hash(2);

        manager.add_pruning_candidate(hash1);
        manager.mark_block_protected(hash2);

        assert!(manager.can_prune_block(&hash1));
        assert!(!manager.can_prune_block(&hash2));
        assert!(!manager.can_prune_block(&create_test_hash(3))); // Not a candidate
    }

    #[test]
    fn test_get_blocks_to_prune() {
        let config = PruningConfig {
            max_pruning_batch: 2,
            ..Default::default()
        };
        let manager = PruningManager::new(config);

        let hash1 = create_test_hash(1);
        let hash2 = create_test_hash(2);
        let hash3 = create_test_hash(3);

        manager.add_pruning_candidate(hash1);
        manager.add_pruning_candidate(hash2);
        manager.add_pruning_candidate(hash3);
        manager.mark_block_protected(hash2); // Protect hash2

        let to_prune = manager.get_blocks_to_prune();
        assert_eq!(to_prune.len(), 2); // Should respect batch limit
        assert!(to_prune.contains(&hash1));
        assert!(to_prune.contains(&hash3));
        assert!(!to_prune.contains(&hash2)); // Protected
    }

    #[test]
    fn test_mark_blocks_pruned() {
        let manager = PruningManager::new(PruningConfig::default());
        let hash1 = create_test_hash(1);
        let hash2 = create_test_hash(2);

        manager.add_pruning_candidate(hash1);
        manager.add_pruning_candidate(hash2);

        manager.mark_blocks_pruned(&[hash1]);

        let stats = manager.get_stats();
        assert_eq!(stats.pruning_candidates, 1); // Only hash2 remains
    }

    #[test]
    fn test_update_pruning_point() {
        let manager = PruningManager::new(PruningConfig::default());
        let hash = create_test_hash(1);

        manager.update_pruning_point(hash);

        assert_eq!(manager.get_pruning_point(), Some(hash));
    }

    #[test]
    fn test_calculate_pruning_point() {
        let manager = PruningManager::new(PruningConfig::default());
        let tips = vec![create_test_hash(1), create_test_hash(2)];
        let mut depths = HashMap::new();
        depths.insert(tips[0], 100);
        depths.insert(tips[1], 50); // Older

        let pruning_point = manager.calculate_pruning_point(&tips, &depths).unwrap();
        assert_eq!(pruning_point, tips[1]); // Should choose the older tip
    }

    #[test]
    fn test_pruning_disabled() {
        let config = PruningConfig {
            enabled: false,
            ..Default::default()
        };
        let manager = PruningManager::new(config);
        let hash = create_test_hash(1);

        manager.add_pruning_candidate(hash);

        assert!(!manager.can_prune_block(&hash));
        assert!(manager.get_blocks_to_prune().is_empty());
    }

    #[test]
    fn test_clear_candidates() {
        let manager = PruningManager::new(PruningConfig::default());
        let hash = create_test_hash(1);

        manager.add_pruning_candidate(hash);
        assert_eq!(manager.get_stats().pruning_candidates, 1);

        manager.clear_candidates();
        assert_eq!(manager.get_stats().pruning_candidates, 0);
    }
}
