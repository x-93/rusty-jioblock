use std::collections::{HashMap, HashSet};

use crate::{
    constants::GHOSTDAG_K,
    errors::ConsensusError,
    header::Header,
    BlueWorkType,
    BlockHashMap,
    HashMapCustomHasher,
    KType,
};

use crate::Hash;

/// Represents GhostDAG data for a block
#[derive(Clone, Debug)]
pub struct GhostdagData {
    /// The blue blocks in this block's anticone
    pub blue_set: HashSet<Hash>,
    /// The red blocks in this block's anticone
    pub red_set: HashSet<Hash>,
    /// Blue anticone size for K-cluster selection
    pub blue_anticone_size: KType,
    /// Block's blue score (number of blue blocks)
    pub blue_score: u64,
    /// Accumulated proof of work of blue blocks
    pub blue_work: BlueWorkType,
    /// Selected parent with highest blue score
    pub selected_parent: Hash,
    /// Merkle root of block acceptance data
    pub mergeset_root: Hash,
}

/// Manages GhostDAG consensus calculations
pub struct GhostdagManager {
    /// GhostDAG data for each block
    block_data: BlockHashMap<GhostdagData>,
    /// Blue work for each block
    blue_work_store: BlockHashMap<BlueWorkType>,
}

impl GhostdagManager {
    /// Creates a new GhostDAG manager
    pub fn new() -> Self {
        Self {
            block_data: BlockHashMap::new(),
            blue_work_store: BlockHashMap::new(),
        }
    }

    /// Processes a new block header and calculates its GhostDAG data
    pub fn process_block(&mut self, header: &Header) -> Result<(), ConsensusError> {
        // Validate block has at least one parent
        if header.parents_by_level.is_empty() {
            return Err(ConsensusError::InvalidDagStructure);
        }

        let parents = header.direct_parents();
        if parents.is_empty() {
            return Err(ConsensusError::InvalidDagStructure);
        }

        // Get selected parent (highest blue score)
        let selected_parent = self.find_selected_parent(parents)?;

        // Calculate blue/red sets
        let (blue_set, red_set) = self.calculate_coloring(header, &selected_parent)?;

        // Calculate blue score and work
        let blue_score = self.calculate_blue_score(&blue_set)?;
        let blue_work = self.calculate_blue_work(&blue_set)?;

        // Calculate mergeset root
        let mergeset_root = self.calculate_mergeset_root(&blue_set, &red_set)?;

        // Store GhostDAG data
        let ghostdag_data = GhostdagData {
            blue_set: blue_set.clone(),
            red_set: red_set.clone(),
            blue_anticone_size: blue_set.len() as KType,
            blue_score,
            blue_work,
            selected_parent,
            mergeset_root,
        };

        self.block_data.insert(header.hash, ghostdag_data);
        self.blue_work_store.insert(header.hash, blue_work);

        Ok(())
    }

    /// Finds the selected parent (parent with highest blue score)
    fn find_selected_parent(&self, parents: &[Hash]) -> Result<Hash, ConsensusError> {
        let mut selected = parents[0];
        let mut max_score = 0u64;

        for parent in parents {
            if let Some(data) = self.block_data.get(parent) {
                if data.blue_score > max_score {
                    max_score = data.blue_score;
                    selected = *parent;
                }
            } else {
                return Err(ConsensusError::InvalidBlockParent);
            }
        }

        Ok(selected)
    }

    /// Calculates blue and red sets for a block using the GhostDAG K-cluster selection
    fn calculate_coloring(
        &self,
        header: &Header,
        selected_parent: &Hash,
    ) -> Result<(HashSet<Hash>, HashSet<Hash>), ConsensusError> {
        let mut blue_set = HashSet::new();
        let mut red_set = HashSet::new();
        let mut visited = HashSet::new();

        // Add all ancestors to visited set
        self.collect_ancestors(header, &mut visited)?;

        // Get anticone (blocks not in past or future)
        let anticone = self.get_anticone(header, &visited)?;

        // Calculate blue set based on K parameter
        for block in anticone {
            if self.get_anticone_size(&block, &blue_set)? <= GHOSTDAG_K {
                blue_set.insert(block);
            } else {
                red_set.insert(block);
            }
        }

        Ok((blue_set, red_set))
    }

    /// Calculates total blue score (number of blue blocks)
    fn calculate_blue_score(&self, blue_set: &HashSet<Hash>) -> Result<u64, ConsensusError> {
        let mut score = 0u64;

        for block in blue_set {
            if let Some(data) = self.block_data.get(block) {
                score += data.blue_score;
            } else {
                return Err(ConsensusError::InvalidDagStructure);
            }
        }

        Ok(score)
    }

    /// Calculates accumulated proof of work of blue blocks
    fn calculate_blue_work(&self, blue_set: &HashSet<Hash>) -> Result<BlueWorkType, ConsensusError> {
        let mut work = BlueWorkType::from(0u64);

        for block in blue_set {
            if let Some(block_work) = self.blue_work_store.get(block) {
                work += *block_work;
            } else {
                return Err(ConsensusError::InvalidDagStructure);
            }
        }

        Ok(work)
    }

    /// Calculates the size of a block's anticone relative to a set
    fn get_anticone_size(&self, block: &Hash, set: &HashSet<Hash>) -> Result<KType, ConsensusError> {
        let mut count = 0;

        if let Some(data) = self.block_data.get(block) {
            for blue_block in &data.blue_set {
                if set.contains(blue_block) {
                    count += 1;
                }
            }
            Ok(count)
        } else {
            Err(ConsensusError::InvalidDagStructure)
        }
    }

    /// Collects all ancestors of a block recursively
    fn collect_ancestors(&self, header: &Header, visited: &mut HashSet<Hash>) -> Result<(), ConsensusError> {
        if !visited.insert(header.hash) {
            return Ok(());
        }

        for parent in header.direct_parents() {
            if let Some(parent_data) = self.block_data.get(parent) {
                self.collect_ancestors(&Header::from_precomputed_hash(*parent, vec![parent_data.selected_parent]), visited)?;
            } else {
                return Err(ConsensusError::InvalidBlockParent);
            }
        }

        Ok(())
    }

    /// Gets blocks in the anticone (not in past or future)
    fn get_anticone(&self, header: &Header, past: &HashSet<Hash>) -> Result<HashSet<Hash>, ConsensusError> {
        let mut anticone = HashSet::new();

        // Add blocks from parents' blue sets that aren't in the past
        for parent in header.direct_parents() {
            if let Some(parent_data) = self.block_data.get(parent) {
                for block in &parent_data.blue_set {
                    if !past.contains(block) {
                        anticone.insert(*block);
                    }
                }
            } else {
                return Err(ConsensusError::InvalidBlockParent);
            }
        }

        Ok(anticone)
    }

    /// Calculates the merkle root of the block acceptance data
    fn calculate_mergeset_root(
        &self,
        blue_set: &HashSet<Hash>,
        red_set: &HashSet<Hash>,
    ) -> Result<Hash, ConsensusError> {
        // Combine blue and red sets
        let mut mergeset = Vec::new();
        mergeset.extend(blue_set.iter());
        mergeset.extend(red_set.iter());
        
        // Sort by hash for deterministic ordering
        mergeset.sort();

        // Calculate merkle root
        use crate::merkle::MerkleTree;
        let tree = MerkleTree::from_hashes(mergeset);
        Ok(tree.root())
    }
}