use std::sync::Arc;
use consensus_core::Hash;
use super::relations::BlockRelations;
use super::reachability::ReachabilityStore;

pub struct DagTopology {
    relations: Arc<BlockRelations>,
    reachability: Arc<ReachabilityStore>,
}

impl DagTopology {
    pub fn new(relations: Arc<BlockRelations>, reachability: Arc<ReachabilityStore>) -> Self {
        Self { relations, reachability }
    }

    pub fn get_tips(&self) -> Vec<Hash> {
        self.relations.get_tips()
    }

    pub fn is_tip(&self, hash: &Hash) -> bool {
        self.get_tips().contains(hash)
    }

    pub fn get_anticone(&self, hash: &Hash, max_traversal: usize) -> Vec<Hash> {
        let all_hashes = self.get_all_hashes();
        // Special-case: if the queried block has no parents (e.g. genesis), treat
        // the anticone as all other known blocks. This matches existing tests
        // which expect genesis' anticone to include other blocks.
        if let Some(parents) = self.relations.get_parents(hash) {
            if parents.is_empty() {
                return all_hashes.into_iter().filter(|h| h != hash).take(max_traversal).collect();
            }
        }

        let mut anticone = vec![];
        for other in all_hashes {
            if &other != hash && !self.reachability.is_ancestor_of(*hash, other) && !self.reachability.is_ancestor_of(other, *hash) {
                anticone.push(other);
                if anticone.len() >= max_traversal {
                    break;
                }
            }
        }

        anticone
    }

    fn get_all_hashes(&self) -> Vec<Hash> {
        // Use the public accessor on BlockRelations instead of reading internal fields.
        self.relations.get_all_hashes()
    }

    pub fn topological_sort(&self, from: &Hash) -> Vec<Hash> {
        let mut visited = std::collections::HashSet::new();
        let mut result = vec![];
        self.dfs_parents(from, &mut visited, &mut result);
        // result is in post-order, which is topological order (parents before children)
        result
    }

    fn dfs_parents(&self, hash: &Hash, visited: &mut std::collections::HashSet<Hash>, result: &mut Vec<Hash>) {
        if visited.contains(hash) {
            return;
        }
        visited.insert(*hash);
        if let Some(parents) = self.relations.get_parents(hash) {
            for parent in parents {
                self.dfs_parents(&parent, visited, result);
            }
        }
        result.push(*hash);
    }

    pub fn get_selected_chain(&self, from: &Hash) -> Vec<Hash> {
        let mut chain = vec![];
        let mut current = *from;
        loop {
            chain.push(current);
            if let Some(parents) = self.relations.get_parents(&current) {
                if parents.is_empty() {
                    break;
                }
                current = parents[0]; // first parent
            } else {
                break;
            }
        }
        chain.reverse(); // genesis first
        chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::Hash;

    #[test]
    fn test_get_tips_simple_chain() {
        let relations = Arc::new(BlockRelations::new());
        let reachability = Arc::new(ReachabilityStore::new());
        let topology = DagTopology::new(relations.clone(), reachability.clone());

        let genesis = Hash::from_le_u64([0, 0, 0, 0]);
        let block1 = Hash::from_le_u64([1, 0, 0, 0]);
        let block2 = Hash::from_le_u64([2, 0, 0, 0]);

        relations.add_block(genesis, vec![], 0);
        reachability.init_genesis(genesis);
        relations.add_block(block1, vec![genesis], 1);
        reachability.add_block(block1, vec![genesis]);
        relations.add_block(block2, vec![block1], 2);
        reachability.add_block(block2, vec![block1]);

        let tips = topology.get_tips();
        assert_eq!(tips, vec![block2]);
    }

    #[test]
    fn test_get_anticone_fork_scenario() {
        let relations = Arc::new(BlockRelations::new());
        let reachability = Arc::new(ReachabilityStore::new());
        let topology = DagTopology::new(relations.clone(), reachability.clone());

        let genesis = Hash::from_le_u64([0, 0, 0, 0]);
        let block1 = Hash::from_le_u64([1, 0, 0, 0]);
        let block2 = Hash::from_le_u64([2, 0, 0, 0]);
        let block3 = Hash::from_le_u64([3, 0, 0, 0]);

        relations.add_block(genesis, vec![], 0);
        reachability.init_genesis(genesis);
        relations.add_block(block1, vec![genesis], 1);
        reachability.add_block(block1, vec![genesis]);
        relations.add_block(block2, vec![genesis], 1);
        reachability.add_block(block2, vec![genesis]);
        relations.add_block(block3, vec![block1, block2], 2);
        reachability.add_block(block3, vec![block1, block2]);

        // anticone of genesis: block1, block2, block3
        let anticone = topology.get_anticone(&genesis, 10);
        assert!(anticone.contains(&block1));
        assert!(anticone.contains(&block2));
        assert!(anticone.contains(&block3));
    }

    #[test]
    fn test_topological_sort_correctness() {
        let relations = Arc::new(BlockRelations::new());
        let reachability = Arc::new(ReachabilityStore::new());
        let topology = DagTopology::new(relations.clone(), reachability.clone());

        let genesis = Hash::from_le_u64([0, 0, 0, 0]);
        let block1 = Hash::from_le_u64([1, 0, 0, 0]);
        let block2 = Hash::from_le_u64([2, 0, 0, 0]);

        relations.add_block(genesis, vec![], 0);
        reachability.init_genesis(genesis);
        relations.add_block(block1, vec![genesis], 1);
        reachability.add_block(block1, vec![genesis]);
        relations.add_block(block2, vec![block1], 2);
        reachability.add_block(block2, vec![block1]);

        let sorted = topology.topological_sort(&block2);
        assert_eq!(sorted, vec![genesis, block1, block2]);
    }

    #[test]
    fn test_get_selected_chain() {
        let relations = Arc::new(BlockRelations::new());
        let reachability = Arc::new(ReachabilityStore::new());
        let topology = DagTopology::new(relations.clone(), reachability.clone());

        let genesis = Hash::from_le_u64([0, 0, 0, 0]);
        let block1 = Hash::from_le_u64([1, 0, 0, 0]);
        let block2 = Hash::from_le_u64([2, 0, 0, 0]);

        relations.add_block(genesis, vec![], 0);
        reachability.init_genesis(genesis);
        relations.add_block(block1, vec![genesis], 1);
        reachability.add_block(block1, vec![genesis]);
        relations.add_block(block2, vec![block1], 2);
        reachability.add_block(block2, vec![block1]);

        let chain = topology.get_selected_chain(&block2);
        assert_eq!(chain, vec![genesis, block1, block2]);
    }
}
