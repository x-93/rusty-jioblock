#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::sync::Arc;
    use crate::Hash;
    use std::collections::HashSet;
    use crate::{BlockRelations, ReachabilityStore, DagTopology};

    #[test]
    fn test_dag_integration() {
        // Create components
        let relations = Arc::new(BlockRelations::new());
        let reachability = Arc::new(ReachabilityStore::new());
        let topology = DagTopology::new(relations.clone(), reachability.clone());

        // Add genesis
        let genesis = Hash::from_le_u64([0, 0, 0, 0]);
        relations.add_block(genesis, vec![], 0);
        reachability.init_genesis(genesis);

        // Add a chain of 3 blocks
        let block1 = Hash::from_le_u64([1, 0, 0, 0]);
        relations.add_block(block1, vec![genesis], 1);
        reachability.add_block(block1, vec![genesis]);

        let block2 = Hash::from_le_u64([2, 0, 0, 0]);
        relations.add_block(block2, vec![block1], 2);
        reachability.add_block(block2, vec![block1]);

        // Verify relationships
        assert!(relations.contains(&genesis));
        assert!(relations.contains(&block1));
        assert!(relations.contains(&block2));
        assert_eq!(relations.get_parents(&block1), Some(vec![genesis]));
        assert_eq!(relations.get_parents(&block2), Some(vec![block1]));
        assert_eq!(relations.get_children(&genesis), Some(HashSet::from([block1])));
        assert_eq!(relations.get_children(&block1), Some(HashSet::from([block2])));

        // Verify reachability
        assert!(reachability.is_ancestor_of(genesis, block1));
        assert!(reachability.is_ancestor_of(genesis, block2));
        assert!(reachability.is_ancestor_of(block1, block2));
        assert!(!reachability.is_ancestor_of(block1, genesis));

        // Verify topology
        assert_eq!(topology.get_tips(), vec![block2]);
        assert!(topology.is_tip(&block2));
        assert!(!topology.is_tip(&genesis));
        let chain = topology.get_selected_chain(&block2);
        assert_eq!(chain, vec![genesis, block1, block2]);
    }
}
