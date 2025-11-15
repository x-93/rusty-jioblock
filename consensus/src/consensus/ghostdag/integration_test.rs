#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::{BlockRelations, ReachabilityStore, DagTopology, GhostdagStore, GhostdagProtocol, Hash};
    use consensus_core::header::Header;
    use std::sync::Arc;

    #[test]
    fn test_ghostdag_integration() {
        // Set up DAG components
        let relations = Arc::new(BlockRelations::new());
        let reachability = Arc::new(ReachabilityStore::new());
        let topology = Arc::new(DagTopology::new(relations.clone(), reachability.clone()));
        let store = Arc::new(GhostdagStore::new());
        let protocol = GhostdagProtocol::new(18, topology, relations.clone(), store.clone());

        // Add genesis
    let genesis = Hash::from_le_u64([0, 0, 0, 0]);
    relations.add_block(genesis, vec![], 0);
    reachability.init_genesis(genesis);
    let genesis_header = Header::from_precomputed_hash(genesis, vec![]);
    let genesis_data = protocol.calculate_ghostdag(&genesis_header).unwrap();
    store.insert(genesis, genesis_data);

        // Add a chain
    let block1 = Hash::from_le_u64([1, 0, 0, 0]);
        relations.add_block(block1, vec![genesis], 1);
        reachability.add_block(block1, vec![genesis]);
    let header1 = Header::from_precomputed_hash(block1, vec![genesis]);
    let block1_data = protocol.calculate_ghostdag(&header1).unwrap();
    store.insert(block1, block1_data.clone());

    let block2 = Hash::from_le_u64([2, 0, 0, 0]);
        relations.add_block(block2, vec![block1], 2);
        reachability.add_block(block2, vec![block1]);
    let header2 = Header::from_precomputed_hash(block2, vec![block1]);
    let block2_data = protocol.calculate_ghostdag(&header2).unwrap();
    store.insert(block2, block2_data.clone());

        // Verify chain properties
        assert_eq!(block1_data.selected_parent, genesis);
        assert_eq!(block1_data.height, 1);
        assert_eq!(block2_data.selected_parent, block1);
        assert_eq!(block2_data.height, 2);

        // Add a fork
    let block3 = Hash::from_le_u64([3, 0, 0, 0]);
        relations.add_block(block3, vec![genesis], 1);
        reachability.add_block(block3, vec![genesis]);
    let header3 = Header::from_precomputed_hash(block3, vec![genesis]);
    let block3_data = protocol.calculate_ghostdag(&header3).unwrap();
    store.insert(block3, block3_data.clone());

        // Merge block
    let merge = Hash::from_le_u64([4, 0, 0, 0]);
        relations.add_block(merge, vec![block2, block3], 2);
        reachability.add_block(merge, vec![block2, block3]);
    let merge_header = Header::from_precomputed_hash(merge, vec![block2, block3]);
    let merge_data = protocol.calculate_ghostdag(&merge_header).unwrap();
    store.insert(merge, merge_data.clone());

        // Verify merge
        assert!(merge_data.selected_parent == block2 || merge_data.selected_parent == block3);
        assert_eq!(merge_data.merge_set_size, 2);
        assert_eq!(merge_data.height, 2);
    }
}
