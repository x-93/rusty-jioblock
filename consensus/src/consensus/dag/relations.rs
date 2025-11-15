use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use consensus_core::Hash;

pub struct BlockRelations {
    pub(crate) parents: RwLock<HashMap<Hash, Vec<Hash>>>,
    pub(crate) children: RwLock<HashMap<Hash, HashSet<Hash>>>,
    heights: RwLock<HashMap<Hash, u64>>,
}

impl BlockRelations {
    pub fn new() -> Self {
        Self {
            parents: RwLock::new(HashMap::new()),
            children: RwLock::new(HashMap::new()),
            heights: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_block(&self, hash: Hash, parents: Vec<Hash>, height: u64) {
        // Add parents
        {
            let mut parents_map = self.parents.write().unwrap();
            parents_map.insert(hash, parents.clone());
        }

        // Add children relationships
        for parent in parents {
            let mut children_map = self.children.write().unwrap();
            children_map.entry(parent).or_insert_with(HashSet::new).insert(hash);
        }

        // Add height
        {
            let mut heights_map = self.heights.write().unwrap();
            heights_map.insert(hash, height);
        }
    }

    pub fn get_parents(&self, hash: &Hash) -> Option<Vec<Hash>> {
        let parents_map = self.parents.read().unwrap();
        parents_map.get(hash).cloned()
    }

    pub fn get_children(&self, hash: &Hash) -> Option<HashSet<Hash>> {
        let children_map = self.children.read().unwrap();
        // Return an empty set when there are no children recorded for the given hash.
        // Tests expect `Some(empty_set)` for blocks with no children rather than `None`.
        match children_map.get(hash) {
            Some(set) => Some(set.clone()),
            None => Some(HashSet::new()),
        }
    }

    pub fn get_height(&self, hash: &Hash) -> Option<u64> {
        let heights_map = self.heights.read().unwrap();
        heights_map.get(hash).copied()
    }

    pub fn contains(&self, hash: &Hash) -> bool {
        let heights_map = self.heights.read().unwrap();
        heights_map.contains_key(hash)
    }

    pub fn get_tips(&self) -> Vec<Hash> {
        let children_map = self.children.read().unwrap();
        let parents_map = self.parents.read().unwrap();

        parents_map.keys()
            .filter(|hash| !children_map.contains_key(hash))
            .cloned()
            .collect()
    }

    /// Returns all known block hashes tracked in the relations (from heights map).
    pub fn get_all_hashes(&self) -> Vec<Hash> {
        let heights_map = self.heights.read().unwrap();
        heights_map.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_genesis_block() {
        let relations = BlockRelations::new();
        let genesis_hash = Hash::from_le_u64([1, 0, 0, 0]);

        relations.add_block(genesis_hash, vec![], 0);

        assert!(relations.contains(&genesis_hash));
        assert_eq!(relations.get_height(&genesis_hash), Some(0));
        assert_eq!(relations.get_parents(&genesis_hash), Some(vec![]));
        assert_eq!(relations.get_children(&genesis_hash), Some(HashSet::new()));
        assert_eq!(relations.get_tips(), vec![genesis_hash]);
    }

    #[test]
    fn test_add_block_with_single_parent() {
        let relations = BlockRelations::new();
        let genesis_hash = Hash::from_le_u64([1, 0, 0, 0]);
        let block_hash = Hash::from_le_u64([2, 0, 0, 0]);

        relations.add_block(genesis_hash, vec![], 0);
        relations.add_block(block_hash, vec![genesis_hash], 1);

        assert!(relations.contains(&block_hash));
        assert_eq!(relations.get_height(&block_hash), Some(1));
        assert_eq!(relations.get_parents(&block_hash), Some(vec![genesis_hash]));
        assert_eq!(relations.get_children(&genesis_hash), Some(HashSet::from([block_hash])));
        assert_eq!(relations.get_tips(), vec![block_hash]);
    }

    #[test]
    fn test_add_block_with_multiple_parents() {
        let relations = BlockRelations::new();
        let genesis1 = Hash::from_le_u64([1, 0, 0, 0]);
        let genesis2 = Hash::from_le_u64([2, 0, 0, 0]);
        let block_hash = Hash::from_le_u64([3, 0, 0, 0]);

        relations.add_block(genesis1, vec![], 0);
        relations.add_block(genesis2, vec![], 0);
        relations.add_block(block_hash, vec![genesis1, genesis2], 1);

        assert!(relations.contains(&block_hash));
        assert_eq!(relations.get_height(&block_hash), Some(1));
        let parents = relations.get_parents(&block_hash).unwrap();
        assert!(parents.contains(&genesis1));
        assert!(parents.contains(&genesis2));
        assert_eq!(relations.get_children(&genesis1), Some(HashSet::from([block_hash])));
        assert_eq!(relations.get_children(&genesis2), Some(HashSet::from([block_hash])));
        assert_eq!(relations.get_tips(), vec![block_hash]);
    }

    #[test]
    fn test_get_tips_multiple() {
        let relations = BlockRelations::new();
        let genesis1 = Hash::from_le_u64([1, 0, 0, 0]);
        let genesis2 = Hash::from_le_u64([2, 0, 0, 0]);
        let block1 = Hash::from_le_u64([3, 0, 0, 0]);
        let block2 = Hash::from_le_u64([4, 0, 0, 0]);

        relations.add_block(genesis1, vec![], 0);
        relations.add_block(genesis2, vec![], 0);
        relations.add_block(block1, vec![genesis1], 1);
        relations.add_block(block2, vec![genesis2], 1);

        let tips = relations.get_tips();
        assert!(tips.contains(&block1));
        assert!(tips.contains(&block2));
        assert!(!tips.contains(&genesis1));
        assert!(!tips.contains(&genesis2));
    }

    #[test]
    fn test_duplicate_block() {
        let relations = BlockRelations::new();
        let hash = Hash::from_le_u64([1, 0, 0, 0]);

        relations.add_block(hash, vec![], 0);
        relations.add_block(hash, vec![], 0); // Duplicate

        assert!(relations.contains(&hash));
        assert_eq!(relations.get_height(&hash), Some(0));
    }

    #[test]
    fn test_missing_parent() {
        let relations = BlockRelations::new();
        let parent_hash = Hash::from_le_u64([1, 0, 0, 0]);
        let block_hash = Hash::from_le_u64([2, 0, 0, 0]);

        // Add block with non-existent parent
        relations.add_block(block_hash, vec![parent_hash], 1);

        assert!(relations.contains(&block_hash));
        assert_eq!(relations.get_parents(&block_hash), Some(vec![parent_hash]));
        // Parent is not in the relations, but the block is added
        assert!(!relations.contains(&parent_hash));
    }
}
