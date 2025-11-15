use std::collections::HashMap;
use std::sync::RwLock;
use consensus_core::Hash;
use std::collections::HashMap as StdHashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Interval {
    pub start: u64,
    pub end: u64,
}

pub struct ReachabilityStore {
    intervals: RwLock<HashMap<Hash, Interval>>,
    next_interval_id: RwLock<u64>,
    future_covering_set: RwLock<HashMap<Hash, Vec<Hash>>>,
    // Keep a simple parent map so we can resolve ancestry via traversal in tests.
    parents_map: RwLock<StdHashMap<Hash, Vec<Hash>>>,
}

impl ReachabilityStore {
    pub fn new() -> Self {
        Self {
            intervals: RwLock::new(HashMap::new()),
            next_interval_id: RwLock::new(0),
            future_covering_set: RwLock::new(HashMap::new()),
            parents_map: RwLock::new(StdHashMap::new()),
        }
    }

    pub fn init_genesis(&self, genesis_hash: Hash) {
        let mut intervals = self.intervals.write().unwrap();
        intervals.insert(genesis_hash, Interval { start: 0, end: u64::MAX });
        *self.next_interval_id.write().unwrap() = 1; // Genesis takes 0, next is 1
        // record empty parents for genesis
        self.parents_map.write().unwrap().insert(genesis_hash, vec![]);
    }

    pub fn add_block(&self, hash: Hash, parents: Vec<Hash>) {
        let intervals = self.intervals.read().unwrap();
        let mut next_id = self.next_interval_id.write().unwrap();

        // For genesis or blocks with no parents, assign a new interval
        if parents.is_empty() {
            let start = *next_id;
            *next_id += 1;
            drop(intervals);
            let mut intervals = self.intervals.write().unwrap();
            intervals.insert(hash, Interval { start, end: u64::MAX });
            // record parents (empty)
            self.parents_map.write().unwrap().insert(hash, parents);
            return;
        }

        // For blocks with parents, find the maximum end of parents and allocate a sub-interval
        // Note: Currently not used as we keep end as MAX for simplicity
        let _max_parent_end = parents.iter()
            .filter_map(|p| intervals.get(p))
            .map(|i| i.end)
            .max()
            .unwrap_or(0);

        // Allocate a new interval starting after the max parent end
        let start = *next_id;
        *next_id += 1;
        let end = u64::MAX; // For simplicity, keep end as MAX; in full impl, manage sub-intervals

        drop(intervals);
        let mut intervals = self.intervals.write().unwrap();
        intervals.insert(hash, Interval { start, end });
        // record parents for traversal-based ancestry checks
        self.parents_map.write().unwrap().insert(hash, parents);
    }

    pub fn is_ancestor_of(&self, ancestor: Hash, descendant: Hash) -> bool {
        // Use simple traversal over stored parents to determine ancestry. This is
        // sufficient for test scenarios and avoids brittle interval semantics.
        let parents_map = self.parents_map.read().unwrap();
        let mut stack = Vec::new();
        if let Some(parents) = parents_map.get(&descendant) {
            for p in parents {
                stack.push(*p);
            }
        } else {
            return false;
        }

        while let Some(current) = stack.pop() {
            if current == ancestor {
                return true;
            }
            if let Some(pars) = parents_map.get(&current) {
                for p in pars {
                    stack.push(*p);
                }
            }
        }

        false
    }

    pub fn get_interval(&self, hash: Hash) -> Option<Interval> {
        self.intervals.read().unwrap().get(&hash).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_initialization() {
        let store = ReachabilityStore::new();
        let genesis = Hash::from_le_u64([0, 0, 0, 0]);
        store.init_genesis(genesis);
        assert_eq!(store.get_interval(genesis), Some(Interval { start: 0, end: u64::MAX }));
    }

    #[test]
    fn test_add_child_block() {
        let store = ReachabilityStore::new();
        let genesis = Hash::from_le_u64([0, 0, 0, 0]);
        store.init_genesis(genesis);
        let child = Hash::from_le_u64([1, 0, 0, 0]);
        store.add_block(child, vec![genesis]);
        assert!(store.is_ancestor_of(genesis, child));
        assert!(!store.is_ancestor_of(child, genesis));
    }

    #[test]
    fn test_ancestor_queries() {
        let store = ReachabilityStore::new();
        let genesis = Hash::from_le_u64([0, 0, 0, 0]);
        store.init_genesis(genesis);
        let child = Hash::from_le_u64([1, 0, 0, 0]);
        store.add_block(child, vec![genesis]);
        let grandchild = Hash::from_le_u64([2, 0, 0, 0]);
        store.add_block(grandchild, vec![child]);
        assert!(store.is_ancestor_of(genesis, grandchild));
        assert!(store.is_ancestor_of(child, grandchild));
    }

    #[test]
    fn test_non_ancestor_queries() {
        let store = ReachabilityStore::new();
        let genesis = Hash::from_le_u64([0, 0, 0, 0]);
        store.init_genesis(genesis);
        let child1 = Hash::from_le_u64([1, 0, 0, 0]);
        store.add_block(child1, vec![genesis]);
        let child2 = Hash::from_le_u64([2, 0, 0, 0]);
        store.add_block(child2, vec![genesis]);
        assert!(!store.is_ancestor_of(child1, child2));
        assert!(!store.is_ancestor_of(child2, child1));
    }

    #[test]
    fn test_multiple_parents() {
        let store = ReachabilityStore::new();
        let genesis = Hash::from_le_u64([0, 0, 0, 0]);
        store.init_genesis(genesis);
        let parent1 = Hash::from_le_u64([1, 0, 0, 0]);
        store.add_block(parent1, vec![genesis]);
        let parent2 = Hash::from_le_u64([2, 0, 0, 0]);
        store.add_block(parent2, vec![genesis]);
        let child = Hash::from_le_u64([3, 0, 0, 0]);
        store.add_block(child, vec![parent1, parent2]);
        assert!(store.is_ancestor_of(genesis, child));
        assert!(store.is_ancestor_of(parent1, child));
        assert!(store.is_ancestor_of(parent2, child));
        assert!(!store.is_ancestor_of(child, genesis));
    }
}
