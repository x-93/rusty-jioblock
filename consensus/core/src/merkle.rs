use crate::Hash;

/// Represents a Merkle tree for transaction hashes
#[derive(Clone, Debug)]
pub struct MerkleTree {
    /// Nodes at each level of the tree (leaves at level 0)
    levels: Vec<Vec<Hash>>,
}

impl MerkleTree {
    /// Creates a new Merkle tree from a list of transaction hashes
    pub fn from_hashes(hashes: Vec<Hash>) -> Self {
        if hashes.is_empty() {
            return Self { levels: vec![] };
        }

        let mut levels = Vec::new();
        levels.push(hashes);

        while levels.last().unwrap().len() > 1 {
            let current_level = levels.last().unwrap();
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                let left = chunk[0];
                let right = if chunk.len() == 2 { chunk[1] } else { left };
                next_level.push(Self::hash_pair(&left, &right));
            }

            levels.push(next_level);
        }

        Self { levels }
    }

    /// Returns the Merkle root hash
    pub fn root(&self) -> Hash {
        if self.levels.is_empty() {
            Hash::default()
        } else {
            self.levels.last().unwrap()[0]
        }
    }

    /// Generates a Merkle proof for the transaction at the given index
    pub fn generate_proof(&self, tx_index: usize) -> Option<MerkleProof> {
        if self.levels.is_empty() || tx_index >= self.levels[0].len() {
            return None;
        }

        let mut proof = Vec::new();
        let mut pos = tx_index;

        for level in 0..self.levels.len() - 1 {
            let is_right = pos % 2 == 0;
            let sibling_pos = if is_right { pos + 1 } else { pos - 1 };

            if sibling_pos < self.levels[level].len() {
                proof.push(MerkleProofElement {
                    hash: self.levels[level][sibling_pos],
                    is_left: is_right, // If current is right, sibling is left
                });
            } else {
                // For odd-length levels, duplicate the last hash
                proof.push(MerkleProofElement {
                    hash: self.levels[level][pos],
                    is_left: is_right,
                });
            }

            pos /= 2;
        }

        Some(MerkleProof {
            proof_elements: proof,
            index: tx_index,
        })
    }

    /// Hashes two nodes together to create their parent
    fn hash_pair(left: &Hash, right: &Hash) -> Hash {
        use crate::hashing::double_sha256;
        
        let mut combined = Vec::with_capacity(64);
        combined.extend_from_slice(left.as_bytes());
        combined.extend_from_slice(right.as_bytes());
        
        double_sha256(&combined)
    }
}

/// A single element in a Merkle proof
#[derive(Clone, Debug)]
pub struct MerkleProofElement {
    /// The hash of the sibling node
    pub hash: Hash,
    /// Whether this is a left sibling
    pub is_left: bool,
}

/// A complete Merkle proof for a transaction
#[derive(Clone, Debug)]
pub struct MerkleProof {
    /// The proof elements (sibling hashes)
    pub proof_elements: Vec<MerkleProofElement>,
    /// The index of the transaction in the tree
    pub index: usize,
}

impl MerkleProof {
    /// Verifies the proof against a given transaction hash and Merkle root
    pub fn verify(&self, tx_hash: Hash, merkle_root: Hash) -> bool {
        let mut current = tx_hash;

        for element in &self.proof_elements {
            current = if element.is_left {
                MerkleTree::hash_pair(&current, &element.hash)
            } else {
                MerkleTree::hash_pair(&element.hash, &current)
            };
        }

        current == merkle_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree() {
        let tx_hashes = vec![
            Hash::from([1u8; 32]),
            Hash::from([2u8; 32]),
            Hash::from([3u8; 32]),
        ];

        let tree = MerkleTree::from_hashes(tx_hashes.clone());
        let root = tree.root();

        // Generate and verify proof for each transaction
        for (i, hash) in tx_hashes.iter().enumerate() {
            let proof = tree.generate_proof(i).unwrap();
            assert!(proof.verify(*hash, root));
        }
    }

    #[test]
    fn test_empty_tree() {
        let tree = MerkleTree::from_hashes(vec![]);
        assert_eq!(tree.root(), Hash::default());
    }

    #[test]
    fn test_single_tx() {
        let hash = Hash::from([1u8; 32]);
        let tree = MerkleTree::from_hashes(vec![hash]);
        assert_eq!(tree.root(), hash);
    }
}