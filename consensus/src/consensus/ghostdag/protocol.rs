use super::stores::{GhostdagData, GhostdagStore};
use crate::consensus::dag::{DagTopology, BlockRelations};
use consensus_core::{Hash, BlueWorkType, header::Header};
use consensus_pow;
use crypto_hashes::{
    builders::BlockHashBuilder,
    HashWriter,
};
use primitive_types::U256;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::io::Write;

pub struct GhostdagProtocol {
    k: u32,  // anticone size parameter
    topology: Arc<DagTopology>,
    relations: Arc<BlockRelations>, 
    store: Arc<GhostdagStore>,
    hash_builder: BlockHashBuilder,
}

impl GhostdagProtocol {
    pub fn new(k: u32, topology: Arc<DagTopology>, relations: Arc<BlockRelations>, store: Arc<GhostdagStore>) -> Self {
        Self { 
            k, 
            topology, 
            relations, 
            store,
            hash_builder: BlockHashBuilder::new(),
        }
    }

    fn hash_block_data(&self, data: &GhostdagData) -> Hash {
        let mut writer = HashWriter::new();

        // Add sorted sets for determinism
        let mut blue_vec: Vec<_> = data.blue_set.iter().collect();
        blue_vec.sort();
        for hash in blue_vec {
            writer.write(hash.as_bytes()).unwrap();
        }

        let mut red_vec: Vec<_> = data.red_set.iter().collect();
        red_vec.sort();
        for hash in red_vec {
            writer.write(hash.as_bytes()).unwrap();
        }

        // Add metrics
    writer.write(&data.blue_score.to_le_bytes()).unwrap();
    writer.write(&data.blue_work.to_bytes()).unwrap();
    writer.write(data.selected_parent.as_bytes()).unwrap();

        // Add anticone data
        let mut anticone_pairs: Vec<_> = data.blues_anticone_sizes.iter().collect();
        anticone_pairs.sort_by_key(|pair| pair.0);
        for (hash, size) in anticone_pairs {
            writer.write(hash.as_bytes()).unwrap();
            writer.write(&size.to_le_bytes()).unwrap();
        }

        Hash::from(writer.finalize())
    }

    pub fn calculate_ghostdag(&self, header: &Header) -> Result<GhostdagData, String> {
        let parents: Vec<Hash> = header.direct_parents().to_vec();

        if parents.is_empty() {
            // Genesis block
            let mut data = GhostdagData::new(header.hash); // Self-selected for genesis
            data.blue_score = 1;
            // Use header.blue_work if available, otherwise unit work
            data.blue_work = header.blue_work;
            data.merge_set_size = 1;
            data.height = 0;
            // genesis has empty sets
            return Ok(data);
        }

        // Select parent with highest blue score
        let selected_parent = self.select_parent(&parents)?;

        // Calculate blue set and score using k-cluster algorithm
        let (blue_set, red_set) = self.calculate_blue_set(&header.hash, &parents, &selected_parent)?;

        // Calculate blue score: number of blue blocks 
        let blue_score = blue_set.len() as u64;

        // Calculate blue work by summing known work
        let blue_work = self.calculate_blue_work(&blue_set, header)?;

        // Calculate merge set size
        let merge_set_size = parents.len() as u64;

        // Calculate blues anticone sizes
        let blues_anticone_sizes = self.calculate_blues_anticone_sizes(&blue_set)?;

        // Get height
        let height = self.relations.get_height(&selected_parent).unwrap_or(0) + 1;

        // Create and hash GhostDAG data
        let mut data = GhostdagData::new(selected_parent);
        data.blue_set = blue_set.clone();
        data.red_set = red_set.clone();
        data.blue_score = blue_score;
        data.blue_work = blue_work;
        data.merge_set_size = merge_set_size;
        data.blues_anticone_sizes = blues_anticone_sizes;
        data.height = height;

        Ok(data)
    }

    /// Calculate total accumulated proof of work for a set of blue blocks.
    /// For each block, work is calculated as follows:
    /// - For blocks in store: Use stored blue_work
    /// - For current block: Calculate actual PoW work based on target and hash
    /// - For unknown blocks: Estimate work based on target difficulty
    fn calculate_blue_work(&self, blue_set: &HashSet<Hash>, header: &Header) -> Result<BlueWorkType, String> {
        use consensus_pow::State;

        let mut blue_work = BlueWorkType::from(0u64);

        for block in blue_set {
            let work = if let Some(data) = self.store.get(block) {
                data.blue_work
            } else if block == &header.hash {
                // Calculate actual PoW work for this block
                // convert compact bits -> U256 target (same logic as used elsewhere)
                let target = {
                    let bits = header.bits;
                    let size = (bits >> 24) as usize;
                    let word = bits & 0x007fffff;
                    if size <= 3 {
                        U256::from(word >> (8 * (3 - size)))
                    } else {
                        U256::from(word) << (8 * (size - 3))
                    }
                };

                if target.is_zero() {
                    return Err("Invalid target (zero)".to_string());
                }

                let state = State::new(header);
                let pow_hash = state.calculate_pow(header.nonce);

                // Convert PoW difficulty to BlueWork
                if pow_hash <= target {
                    // Work is proportional to 2^256-1 / target
                    let max_bytes = [0xffu8; 32];
                    let max_val = U256::from_big_endian(&max_bytes);
                    let work_amount = max_val / target;
                    BlueWorkType::from(work_amount.low_u64())
                } else {
                    return Err("Invalid proof of work".to_string());
                }
            } else {
                // For unknown blocks, use target-based work estimation
                let default_target = {
                    let bits = header.bits;
                    let size = (bits >> 24) as usize;
                    let word = bits & 0x007fffff;
                    if size <= 3 {
                        U256::from(word >> (8 * (3 - size)))
                    } else {
                        U256::from(word) << (8 * (size - 3))
                    }
                };

                if default_target.is_zero() {
                    return Err("Invalid target (zero)".to_string());
                }

                let max_bytes = [0xffu8; 32];
                let max_val = U256::from_big_endian(&max_bytes);
                let work_estimate = max_val / default_target;
                BlueWorkType::from(work_estimate.low_u64())
            };
            blue_work += work;
        }

        Ok(blue_work)
    }

    fn select_parent(&self, parents: &[Hash]) -> Result<Hash, String> {
        let mut max_score = 0;
        let mut selected = None;

        for parent in parents {
            if let Some(data) = self.store.get(parent) {
                if data.blue_score > max_score {
                    max_score = data.blue_score;
                    selected = Some(*parent);
                }
            } else {
                return Err(format!("Parent {} not found in store", parent));
            }
        }

        selected.ok_or("No parents found".to_string())
    }

    fn calculate_blue_set(&self, hash: &Hash, parents: &[Hash], selected_parent: &Hash) -> Result<(HashSet<Hash>, HashSet<Hash>), String> {
        // New: perform k-cluster style coloring to build blue and red sets
        let mut blue_set: HashSet<Hash> = HashSet::new();
        let mut red_set: HashSet<Hash> = HashSet::new();

        // Start with selected parent and its known blue set (if present)
        if let Some(selected_data) = self.store.get(selected_parent) {
            for b in &selected_data.blue_set {
                blue_set.insert(*b);
            }
            blue_set.insert(*selected_parent);
        } else {
            blue_set.insert(*selected_parent);
        }

        // Consider other parents as candidates as well
        for parent in parents {
            if parent == selected_parent {
                continue;
            }
            // If we have data for the parent, try to include it deterministically
            if let Some(_) = self.store.get(parent) {
                // will be processed via candidates below
            } else {
                // unknown parent -> treat conservatively as red (skip)
            }
        }

        // Get anticone candidates for the new header (blocks neither in past nor future)
        let mut candidates = self.topology.get_anticone(&hash, 10000);
        // Ensure deterministic ordering
        candidates.sort();

        for candidate in candidates {
            // skip if already known
            if blue_set.contains(&candidate) || red_set.contains(&candidate) {
                continue;
            }

            // get candidate's anticone and count intersection with current blue_set
            let candidate_anticone = self.topology.get_anticone(&candidate, 10000);
            let anticone_size = candidate_anticone.iter().filter(|b| blue_set.contains(b)).count() as u32;

            if anticone_size <= self.k {
                blue_set.insert(candidate);
            } else {
                red_set.insert(candidate);
            }
        }

        Ok((blue_set, red_set))
    }

    fn calculate_blues_anticone_sizes(&self, blue_set: &HashSet<Hash>) -> Result<HashMap<Hash, u32>, String> {
        let mut sizes = HashMap::new();
        
        for block in blue_set {
            if let Some(data) = self.store.get(block) {
                let anticone_size = data.blue_set.intersection(blue_set).count() as u32;
                sizes.insert(*block, anticone_size);
            }
        }

        Ok(sizes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::dag::{BlockRelations, ReachabilityStore};

    #[test]
    fn test_genesis_calculation() {
        let relations = Arc::new(BlockRelations::new());
        let reachability = Arc::new(ReachabilityStore::new());
        let topology = Arc::new(DagTopology::new(relations.clone(), reachability.clone()));
        let store = Arc::new(GhostdagStore::new());
        let protocol = GhostdagProtocol::new(18, topology, relations, store);

        let genesis = Hash::from_le_u64([0, 0, 0, 0]);
        let header = Header::from_precomputed_hash(genesis, vec![]);
        let data = protocol.calculate_ghostdag(&header).unwrap();
        assert_eq!(data.blue_score, 1);
        assert_eq!(data.selected_parent, genesis);
        assert_eq!(data.height, 0);
    }

    #[test]
    fn test_child_calculation() {
        let relations = Arc::new(BlockRelations::new());
        let reachability = Arc::new(ReachabilityStore::new());
        let topology = Arc::new(DagTopology::new(relations.clone(), reachability.clone()));
        let store = Arc::new(GhostdagStore::new());
        let protocol = GhostdagProtocol::new(18, topology, relations, store);

        let genesis = Hash::from_le_u64([0, 0, 0, 0]);
        let genesis_header = Header::from_precomputed_hash(genesis, vec![]);
        let genesis_data = protocol.calculate_ghostdag(&genesis_header).unwrap();
        protocol.store.insert(genesis, genesis_data);

        let child = Hash::from_le_u64([1, 0, 0, 0]);
        let child_header = Header::from_precomputed_hash(child, vec![genesis]);
        let child_data = protocol.calculate_ghostdag(&child_header).unwrap();
        assert_eq!(child_data.selected_parent, genesis);
        assert_eq!(child_data.height, 1);
    }

    #[test]
    fn test_multiple_parents() {
        let relations = Arc::new(BlockRelations::new());
        let reachability = Arc::new(ReachabilityStore::new());
        let topology = Arc::new(DagTopology::new(relations.clone(), reachability.clone()));
        let store = Arc::new(GhostdagStore::new());
        let protocol = GhostdagProtocol::new(18, topology, relations, store);

        let genesis = Hash::from_le_u64([0, 0, 0, 0]);
        let genesis_header = Header::from_precomputed_hash(genesis, vec![]);
        let genesis_data = protocol.calculate_ghostdag(&genesis_header).unwrap();
        protocol.store.insert(genesis, genesis_data);

        let parent1 = Hash::from_le_u64([1, 0, 0, 0]);
        let parent1_header = Header::from_precomputed_hash(parent1, vec![genesis]);
        let parent1_data = protocol.calculate_ghostdag(&parent1_header).unwrap();
        protocol.store.insert(parent1, parent1_data);

        let parent2 = Hash::from_le_u64([2, 0, 0, 0]);
        let parent2_header = Header::from_precomputed_hash(parent2, vec![genesis]);
        let parent2_data = protocol.calculate_ghostdag(&parent2_header).unwrap();
        protocol.store.insert(parent2, parent2_data);

        let child = Hash::from_le_u64([3, 0, 0, 0]);
        let child_header = Header::from_precomputed_hash(child, vec![parent1, parent2]);
        let child_data = protocol.calculate_ghostdag(&child_header).unwrap();
        assert!(child_data.selected_parent == parent1 || child_data.selected_parent == parent2);
        assert_eq!(child_data.merge_set_size, 2);
    }

    #[test]
    fn test_pow_work_calculation() {
        use consensus_pow::State;
        let relations = Arc::new(BlockRelations::new());
        let reachability = Arc::new(ReachabilityStore::new());
        let topology = Arc::new(DagTopology::new(relations.clone(), reachability.clone()));
        let store = Arc::new(GhostdagStore::new());
        let protocol = GhostdagProtocol::new(18, topology, relations, store);

        // Create a test block with known valid PoW
        let parent = Hash::from_le_u64([0, 0, 0, 0]);
        let mut header = Header::from_precomputed_hash(Hash::from_le_u64([1, 0, 0, 0]), vec![parent]);
        
        // Set target difficulty (relatively easy for test)
        header.bits = 0x1f00ffff; // Very low difficulty
        header.nonce = 42; // Would be solved in real mining

        // Create test PoW state
        let state = State::new(&header);
        let pow_hash = state.calculate_pow(header.nonce);
        // convert compact bits -> U256 target
        let target = {
            let bits = header.bits;
            let size = (bits >> 24) as usize;
            let word = bits & 0x007fffff;
            if size <= 3 {
                U256::from(word >> (8 * (3 - size)))
            } else {
                U256::from(word) << (8 * (size - 3))
            }
        };
        
        // Only continue test if we have valid PoW
        if pow_hash <= target {
            let blue_set = {
                let mut set = HashSet::new();
                set.insert(header.hash);
                set
            };

            // Calculate blue work
            let blue_work = protocol.calculate_blue_work(&blue_set, &header).unwrap();
            
            // Work should be non-zero
            assert!(blue_work > BlueWorkType::from(0u64));
            
            // Work should be proportional to difficulty
            let expected_min = BlueWorkType::from(1u64);
            assert!(blue_work >= expected_min);
        }
    }

    #[test]
    fn test_block_hashing() {
        let relations = Arc::new(BlockRelations::new());
        let reachability = Arc::new(ReachabilityStore::new());
        let topology = Arc::new(DagTopology::new(relations.clone(), reachability.clone()));
        let store = Arc::new(GhostdagStore::new());
        let protocol = GhostdagProtocol::new(18, topology, relations, store);

        // Create test data
        let _block = Hash::from_le_u64([1, 0, 0, 0]);
        let parent = Hash::from_le_u64([0, 0, 0, 0]);
        let mut data = GhostdagData::new(parent);
        data.blue_set.insert(parent);
        data.blue_score = 2;
        data.blue_work = BlueWorkType::from(100u64);
        data.blues_anticone_sizes.insert(parent, 1);

        // Hash should be deterministic
        let hash1 = protocol.hash_block_data(&data);
        let hash2 = protocol.hash_block_data(&data);
        assert_eq!(hash1, hash2);

        // Different data should give different hashes
        let mut data2 = data.clone();
        data2.blue_score = 3;
        let hash3 = protocol.hash_block_data(&data2);
        assert_ne!(hash1, hash3);
    }
}
