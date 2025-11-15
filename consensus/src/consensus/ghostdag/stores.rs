use consensus_core::{Hash, BlueWorkType};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::collections::HashSet;

/// GHOSTDAG consensus data for a single block
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GhostdagData {
    /// Blue set - blocks considered blue for this block
    pub blue_set: HashSet<Hash>,
    /// Red set - blocks considered red for this block
    pub red_set: HashSet<Hash>,
    /// Blue score - number of blue blocks in the past
    pub blue_score: u64,

    /// Blue work - cumulative difficulty of blue blocks
    pub blue_work: BlueWorkType,

    /// Selected parent - parent with highest blue score
    pub selected_parent: Hash,

    /// Merge set size - number of parents
    pub merge_set_size: u64,

    /// Blues anticone sizes - for ordering
    pub blues_anticone_sizes: HashMap<Hash, u32>,

    /// Block height
    pub height: u64,
}

impl GhostdagData {
    pub fn new(selected_parent: Hash) -> Self {
        Self {
            blue_set: HashSet::new(),
            red_set: HashSet::new(),
            blue_score: 0,
            blue_work: BlueWorkType::from(0u64),
            selected_parent,
            merge_set_size: 0,
            blues_anticone_sizes: HashMap::new(),
            height: 0,
        }
    }

    pub fn with_blue_score(mut self, score: u64) -> Self {
        self.blue_score = score;
        self
    }
}

/// Thread-safe store for GHOSTDAG data
pub struct GhostdagStore {
    data: RwLock<HashMap<Hash, GhostdagData>>,
}

impl GhostdagStore {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert(&self, hash: Hash, data: GhostdagData) {
        let mut store = self.data.write().unwrap();
        store.insert(hash, data);
    }

    pub fn get(&self, hash: &Hash) -> Option<GhostdagData> {
        let store = self.data.read().unwrap();
        store.get(hash).cloned()
    }

    pub fn contains(&self, hash: &Hash) -> bool {
        let store = self.data.read().unwrap();
        store.contains_key(hash)
    }

    pub fn remove(&self, hash: &Hash) -> Option<GhostdagData> {
        let mut store = self.data.write().unwrap();
        store.remove(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ghostdag_data() {
        let parent = Hash::from_le_u64([1, 0, 0, 0]);
        let data = GhostdagData::new(parent).with_blue_score(5);
        assert_eq!(data.selected_parent, parent);
        assert_eq!(data.blue_score, 5);
        assert_eq!(data.merge_set_size, 0);
    }

    #[test]
    fn test_store_operations() {
        let store = GhostdagStore::new();
        let hash = Hash::from_le_u64([1, 0, 0, 0]);
        let parent = Hash::from_le_u64([0, 0, 0, 0]);
        let data = GhostdagData::new(parent);

    store.insert(hash, data.clone());
    assert!(store.contains(&hash));
    assert_eq!(store.get(&hash), Some(data.clone()));
    assert_eq!(store.remove(&hash), Some(data));
        assert!(!store.contains(&hash));
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let store = std::sync::Arc::new(GhostdagStore::new());
        let mut handles = vec![];

        for i in 0..10 {
            let store_clone = std::sync::Arc::clone(&store);
            let handle = thread::spawn(move || {
                let hash = Hash::from_le_u64([i as u64, 0, 0, 0]);
                let parent = Hash::from_le_u64([0, 0, 0, 0]);
                let data = GhostdagData::new(parent);
                store_clone.insert(hash, data);
                assert!(store_clone.contains(&hash));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
