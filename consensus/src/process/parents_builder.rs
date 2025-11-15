//! Parent block selection for new blocks
//!
//! This module implements the logic for selecting parent blocks when creating
//! new blocks in a DAG-based blockchain.

use consensus_core::block::Block;
use consensus_core::header::Header as BlockHeader;
use consensus_core::Hash;
use std::collections::HashSet;
use std::sync::Arc;

/// Parent block selection strategy
#[derive(Debug, Clone, Copy)]
pub enum ParentSelectionStrategy {
    /// Select all available tips
    AllTips,
    /// Select top N tips by blue score
    TopByBlueScore(usize),
    /// Select top N tips by timestamp
    TopByTimestamp(usize),
}

/// Parent builder for selecting block parents
pub struct ParentsBuilder {
    /// Maximum number of parents per block
    max_parents: usize,
    /// Selection strategy
    strategy: ParentSelectionStrategy,
}

impl ParentsBuilder {
    /// Create a new parents builder with default settings
    pub fn new() -> Self {
        Self {
            max_parents: 10,
            strategy: ParentSelectionStrategy::TopByBlueScore(10),
        }
    }

    /// Create a new parents builder with custom settings
    pub fn with_config(max_parents: usize, strategy: ParentSelectionStrategy) -> Self {
        Self {
            max_parents,
            strategy,
        }
    }

    /// Build parent list for a new block
    pub fn build_parents(&self, tips: &[Hash], blue_scores: &[(Hash, u64)]) -> Result<Vec<Hash>, String> {
        if tips.is_empty() {
            return Err("No tips available for parent selection".to_string());
        }

        match self.strategy {
            ParentSelectionStrategy::AllTips => {
                let mut parents = tips.to_vec();
                if parents.len() > self.max_parents {
                    parents.truncate(self.max_parents);
                }
                Ok(parents)
            }
            ParentSelectionStrategy::TopByBlueScore(n) => {
                self.select_by_blue_score(tips, blue_scores, n.min(self.max_parents))
            }
            ParentSelectionStrategy::TopByTimestamp(n) => {
                self.select_by_timestamp(tips, blue_scores, n.min(self.max_parents))
            }
        }
    }

    /// Select parents by blue score (highest first)
    fn select_by_blue_score(&self, tips: &[Hash], blue_scores: &[(Hash, u64)], count: usize) -> Result<Vec<Hash>, String> {
        let mut scored_tips: Vec<(Hash, u64)> = tips.iter()
            .filter_map(|tip| {
                blue_scores.iter()
                    .find(|(hash, _)| hash == tip)
                    .map(|(_, score)| (*tip, *score))
            })
            .collect();

        if scored_tips.is_empty() {
            return Err("No blue scores available for tips".to_string());
        }

        // Sort by blue score descending (highest first)
        scored_tips.sort_by(|a, b| b.1.cmp(&a.1));

        let parents: Vec<Hash> = scored_tips.into_iter()
            .take(count)
            .map(|(hash, _)| hash)
            .collect();

        Ok(parents)
    }

    /// Select parents by timestamp (most recent first)
    fn select_by_timestamp(&self, tips: &[Hash], timestamps: &[(Hash, u64)], count: usize) -> Result<Vec<Hash>, String> {
        let mut timed_tips: Vec<(Hash, u64)> = tips.iter()
            .filter_map(|tip| {
                timestamps.iter()
                    .find(|(hash, _)| hash == tip)
                    .map(|(_, time)| (*tip, *time))
            })
            .collect();

        if timed_tips.is_empty() {
            return Err("No timestamps available for tips".to_string());
        }

        // Sort by timestamp descending (most recent first)
        timed_tips.sort_by(|a, b| b.1.cmp(&a.1));

        let parents: Vec<Hash> = timed_tips.into_iter()
            .take(count)
            .map(|(hash, _)| hash)
            .collect();

        Ok(parents)
    }

    /// Validate that selected parents are valid
    pub fn validate_parents(&self, parents: &[Hash], tips: &[Hash]) -> Result<(), String> {
        if parents.is_empty() {
            return Err("No parents selected".to_string());
        }

        if parents.len() > self.max_parents {
            return Err(format!("Too many parents: {} > {}", parents.len(), self.max_parents));
        }

        // Check that all parents are in the current tips
        let tip_set: HashSet<Hash> = tips.iter().cloned().collect();
        for parent in parents {
            if !tip_set.contains(parent) {
                return Err(format!("Parent {} is not a current tip", parent));
            }
        }

        // Check for duplicates
        let parent_set: HashSet<Hash> = parents.iter().cloned().collect();
        if parent_set.len() != parents.len() {
            return Err("Duplicate parents found".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_hashes(count: usize) -> Vec<Hash> {
        (0..count)
            .map(|i| Hash::from_le_u64([i as u64, 0, 0, 0]))
            .collect()
    }

    #[test]
    fn test_parents_builder_creation() {
        let builder = ParentsBuilder::new();
        assert_eq!(builder.max_parents, 10);
    }

    #[test]
    fn test_parents_builder_with_config() {
        let builder = ParentsBuilder::with_config(5, ParentSelectionStrategy::AllTips);
        assert_eq!(builder.max_parents, 5);
    }

    #[test]
    fn test_build_parents_all_tips() {
        let builder = ParentsBuilder::with_config(5, ParentSelectionStrategy::AllTips);
        let tips = create_test_hashes(3);
        let blue_scores = vec![]; // Not used for AllTips

        let parents = builder.build_parents(&tips, &blue_scores).unwrap();
        assert_eq!(parents.len(), 3);
        assert_eq!(parents, tips);
    }

    #[test]
    fn test_build_parents_top_by_blue_score() {
        let builder = ParentsBuilder::with_config(2, ParentSelectionStrategy::TopByBlueScore(2));
        let tips = create_test_hashes(4);
        let blue_scores = vec![
            (tips[0], 10),
            (tips[1], 30),
            (tips[2], 20),
            (tips[3], 5),
        ];

        let parents = builder.build_parents(&tips, &blue_scores).unwrap();
        assert_eq!(parents.len(), 2);
        // Should be sorted by blue score: tips[1] (30), tips[2] (20)
        assert_eq!(parents[0], tips[1]);
        assert_eq!(parents[1], tips[2]);
    }

    #[test]
    fn test_build_parents_top_by_timestamp() {
        let builder = ParentsBuilder::with_config(2, ParentSelectionStrategy::TopByTimestamp(2));
        let tips = create_test_hashes(4);
        let timestamps = vec![
            (tips[0], 100),
            (tips[1], 400),
            (tips[2], 300),
            (tips[3], 200),
        ];

        let parents = builder.build_parents(&tips, &timestamps).unwrap();
        assert_eq!(parents.len(), 2);
        // Should be sorted by timestamp: tips[1] (400), tips[2] (300)
        assert_eq!(parents[0], tips[1]);
        assert_eq!(parents[1], tips[2]);
    }

    #[test]
    fn test_validate_parents_valid() {
        let builder = ParentsBuilder::with_config(3, ParentSelectionStrategy::AllTips);
        let tips = create_test_hashes(3);
        let parents = vec![tips[0], tips[1]];

        assert!(builder.validate_parents(&parents, &tips).is_ok());
    }

    #[test]
    fn test_validate_parents_empty() {
        let builder = ParentsBuilder::new();
        let tips = create_test_hashes(3);
        let parents = vec![];

        assert!(builder.validate_parents(&parents, &tips).is_err());
    }

    #[test]
    fn test_validate_parents_too_many() {
        let builder = ParentsBuilder::with_config(2, ParentSelectionStrategy::AllTips);
        let tips = create_test_hashes(4);
        let parents = vec![tips[0], tips[1], tips[2]];

        assert!(builder.validate_parents(&parents, &tips).is_err());
    }

    #[test]
    fn test_validate_parents_not_in_tips() {
        let builder = ParentsBuilder::new();
        let tips = create_test_hashes(3);
        let invalid_parent = Hash::from_le_u64([999, 0, 0, 0]);
        let parents = vec![tips[0], invalid_parent];

        assert!(builder.validate_parents(&parents, &tips).is_err());
    }

    #[test]
    fn test_validate_parents_duplicates() {
        let builder = ParentsBuilder::new();
        let tips = create_test_hashes(3);
        let parents = vec![tips[0], tips[0]];

        assert!(builder.validate_parents(&parents, &tips).is_err());
    }
}
