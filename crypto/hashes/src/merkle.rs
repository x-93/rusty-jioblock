use crate::{hasher::{Hashable, HashWriter}, Hash};
use std::io::Write;

/// Merkle tree implementation for hashing multiple items
pub struct MerkleTree {
    leaves: Vec<Hash>,
    levels: Vec<Vec<Hash>>,
}

impl MerkleTree {
    /// Create a new Merkle tree from a set of items that implement Hashable
    pub fn new<T: Hashable>(items: &[T]) -> Self {
        let mut leaves = Vec::with_capacity(items.len());
        for item in items {
            let mut writer = HashWriter::new();
            writer.hash_object(item);
            leaves.push(Hash::from(writer.finalize()));
        }
        Self::from_leaves(leaves)
    }

    /// Create a Merkle tree directly from leaf hashes
    pub fn from_leaves(leaves: Vec<Hash>) -> Self {
        let mut tree = Self {
            leaves: leaves.clone(),
            levels: vec![leaves],
        };
        tree.build_tree();
        tree
    }

    /// Build the internal nodes of the Merkle tree
    fn build_tree(&mut self) {
        let mut current_level = self.levels[0].clone();
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                let mut writer = HashWriter::new();
                match chunk {
                    [left, right] => {
                        writer.write_all(left.as_bytes()).unwrap();
                        writer.write_all(right.as_bytes()).unwrap();
                    }
                    [single] => {
                        writer.write_all(single.as_bytes()).unwrap();
                        writer.write_all(single.as_bytes()).unwrap(); // Duplicate last node if odd
                    }
                    _ => unreachable!(),
                }
                next_level.push(Hash::from(writer.finalize()));
            }
            self.levels.push(next_level.clone());
            current_level = next_level;
        }
    }

    /// Get the Merkle root hash
    pub fn root(&self) -> Hash {
        self.levels.last().unwrap()[0]
    }

    /// Create a Merkle proof for the item at the given index
    pub fn create_proof(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaves.len() {
            return None;
        }

        let mut proof = Vec::new();
        let mut current_index = index;

        for level in &self.levels[..self.levels.len() - 1] {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < level.len() {
                proof.push(level[sibling_index]);
            } else {
                proof.push(level[current_index]); // Use same hash if no sibling
            }

            current_index /= 2;
        }

        Some(MerkleProof {
            proof_hashes: proof,
            index,
        })
    }
}

/// Merkle proof for verifying inclusion
pub struct MerkleProof {
    proof_hashes: Vec<Hash>,
    index: usize,
}

impl MerkleProof {
    /// Verify that the given hash exists in the tree with the given root
    pub fn verify(&self, hash: Hash, root: Hash) -> bool {
        let mut current = hash;
        let mut current_index = self.index;

        for sibling in &self.proof_hashes {
            let mut writer = HashWriter::new();
            if current_index % 2 == 0 {
                writer.write_all(current.as_bytes()).unwrap();
                writer.write_all(sibling.as_bytes()).unwrap();
            } else {
                writer.write_all(sibling.as_bytes()).unwrap();
                writer.write_all(current.as_bytes()).unwrap();
            }
            current = Hash::from(writer.finalize());
            current_index /= 2;
        }

        current == root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_empty_tree() {
        let tree = MerkleTree::from_leaves(vec![]);
        assert_eq!(tree.leaves.len(), 0);
        assert_eq!(tree.levels.len(), 1);
    }

    #[test]
    fn test_single_leaf() {
        let hash = Hash::from(hex!("0000000000000000000000000000000000000000000000000000000000000001"));
        let tree = MerkleTree::from_leaves(vec![hash]);
        assert_eq!(tree.root(), hash);
    }

    #[test]
    fn test_two_leaves() {
        let hash1 = Hash::from(hex!("0000000000000000000000000000000000000000000000000000000000000001"));
        let hash2 = Hash::from(hex!("0000000000000000000000000000000000000000000000000000000000000002"));
        let tree = MerkleTree::from_leaves(vec![hash1, hash2]);

        let mut writer = HashWriter::new();
        writer.write_all(hash1.as_bytes()).unwrap();
        writer.write_all(hash2.as_bytes()).unwrap();
        let expected_root = Hash::from(writer.finalize());

        assert_eq!(tree.root(), expected_root);
    }

    #[test]
    fn test_proof_verification() {
        let hashes = vec![
            Hash::from(hex!("0000000000000000000000000000000000000000000000000000000000000001")),
            Hash::from(hex!("0000000000000000000000000000000000000000000000000000000000000002")),
            Hash::from(hex!("0000000000000000000000000000000000000000000000000000000000000003")),
            Hash::from(hex!("0000000000000000000000000000000000000000000000000000000000000004")),
        ];

        let tree = MerkleTree::from_leaves(hashes.clone());
        let root = tree.root();

        // Verify proof for each leaf
        for (i, hash) in hashes.iter().enumerate() {
            let proof = tree.create_proof(i).unwrap();
            assert!(proof.verify(*hash, root));
        }
    }
}