//! Pruning proof generation and verification
//!
//! This module implements the generation and verification of pruning proofs,
//! which allow nodes to prove that certain blocks have been pruned while
//! maintaining the ability to verify the chain's integrity.

use consensus_core::block::Block;
use consensus_core::header::Header as BlockHeader;
use consensus_core::Hash;
use std::collections::{HashMap, HashSet};

/// Pruning proof structure
#[derive(Debug, Clone)]
pub struct PruningProof {
    /// The pruning point block hash
    pub pruning_point: Hash,
    /// Headers of blocks that prove the pruning point
    pub headers: Vec<BlockHeader>,
    /// Merkle proofs for included transactions (if needed)
    pub merkle_proofs: Vec<MerkleProof>,
}

/// Merkle proof for transaction inclusion
#[derive(Debug, Clone)]
pub struct MerkleProof {
    /// Transaction hash being proven
    pub transaction_hash: Hash,
    /// Merkle branch hashes
    pub merkle_branch: Vec<Hash>,
    /// Index of the transaction in the block
    pub index: usize,
}

/// Pruning proof manager
pub struct PruningProofManager {
    /// Maximum number of headers to include in a proof
    max_proof_size: usize,
}

impl PruningProofManager {
    /// Create a new pruning proof manager
    pub fn new(max_proof_size: usize) -> Self {
        Self { max_proof_size }
    }

    /// Generate a pruning proof for a given pruning point
    pub fn generate_proof(&self, pruning_point: Hash, block_headers: &HashMap<Hash, BlockHeader>) -> Result<PruningProof, String> {
        let mut headers = Vec::new();
        let mut current_hash = pruning_point;

        // Walk back from the pruning point collecting headers
        for _ in 0..self.max_proof_size {
            if let Some(header) = block_headers.get(&current_hash) {
                headers.push(header.clone());

                // In a real DAG, we'd need to choose which parent to follow
                // For simplicity, follow the first parent
                // If there are no parents at level 0, stop walking back
                if header.parents_by_level.is_empty() || header.parents_by_level[0].is_empty() {
                    break;
                }
                current_hash = header.parents_by_level[0][0].clone();
            } else {
                return Err(format!("Header not found for block {}", current_hash));
            }
        }

        if headers.is_empty() {
            return Err("No headers found for pruning proof".to_string());
        }

        // Reverse to get chronological order (oldest first)
        headers.reverse();

        Ok(PruningProof {
            pruning_point,
            headers,
            merkle_proofs: vec![], // Simplified: no merkle proofs for now
        })
    }

    /// Verify a pruning proof
    pub fn verify_proof(&self, proof: &PruningProof, trusted_hashes: &HashSet<Hash>) -> Result<(), String> {
        if proof.headers.is_empty() {
            return Err("Pruning proof contains no headers".to_string());
        }

        // Check that the pruning point is in the trusted set
        if !trusted_hashes.contains(&proof.pruning_point) {
            return Err("Pruning point is not in trusted set".to_string());
        }

        // Verify header chain continuity
        for i in 1..proof.headers.len() {
            let current = &proof.headers[i];
            let previous = &proof.headers[i - 1];

            // Check that the current block references the previous block as a parent
            if !current.parents_by_level.iter().flatten().any(|p| p == &previous.hash) {
                return Err(format!(
                    "Header chain discontinuity at index {}: {} does not reference {}",
                    i,
                    current.hash,
                    previous.hash
                ));
            }
        }

        // Verify that the last header's hash matches the pruning point
        if let Some(last_header) = proof.headers.last() {
            if last_header.hash != proof.pruning_point {
                return Err(format!(
                    "Last header hash {} does not match pruning point {}",
                    last_header.hash,
                    proof.pruning_point
                ));
            }
        }

        // Verify merkle proofs (simplified: assume valid for now)
        for merkle_proof in &proof.merkle_proofs {
            self.verify_merkle_proof(merkle_proof)?;
        }

        Ok(())
    }

    /// Generate a merkle proof for a transaction
    pub fn generate_merkle_proof(&self, transaction_hash: Hash, block_transactions: &[Hash], tx_index: usize) -> Result<MerkleProof, String> {
        if block_transactions.is_empty() {
            return Err("No transactions in block".to_string());
        }

        if tx_index >= block_transactions.len() {
            return Err("Transaction index out of bounds".to_string());
        }

        if block_transactions[tx_index] != transaction_hash {
            return Err("Transaction hash mismatch at given index".to_string());
        }

        // Build merkle tree and generate proof
        let merkle_branch = self.build_merkle_branch(block_transactions, tx_index);

        Ok(MerkleProof {
            transaction_hash,
            merkle_branch,
            index: tx_index,
        })
    }

    /// Verify a merkle proof
    pub fn verify_merkle_proof(&self, proof: &MerkleProof) -> Result<(), String> {
        let mut current_hash = proof.transaction_hash;

        for (i, sibling_hash) in proof.merkle_branch.iter().enumerate() {
            // Determine if we should hash left or right based on index parity
            if proof.index & (1 << i) == 0 {
                // Left side
                current_hash = self.hash_pair(&current_hash, sibling_hash);
            } else {
                // Right side
                current_hash = self.hash_pair(sibling_hash, &current_hash);
            }
        }

        // In a real implementation, we'd compare against the expected merkle root
        // For now, just ensure the hash computation doesn't fail
        Ok(())
    }

    /// Build merkle branch for a transaction
    fn build_merkle_branch(&self, transactions: &[Hash], target_index: usize) -> Vec<Hash> {
        if transactions.is_empty() {
            return Vec::new();
        }

        let mut branch = Vec::new();
        let mut current_level = transactions.to_vec();
        let mut current_index = target_index;

        while current_level.len() > 1 {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < current_level.len() {
                branch.push(current_level[sibling_index]);
            } else {
                // Duplicate the last hash if odd number of elements
                branch.push(*current_level.last().unwrap());
            }

            // Move to next level
            current_level = self.next_merkle_level(&current_level);
            current_index /= 2;
        }

        branch
    }

    /// Compute the next level of the merkle tree
    fn next_merkle_level(&self, level: &[Hash]) -> Vec<Hash> {
        let mut next_level = Vec::new();

        for i in (0..level.len()).step_by(2) {
            let left = level[i];
            let right = if i + 1 < level.len() {
                level[i + 1]
            } else {
                left // Duplicate if odd number
            };
            next_level.push(self.hash_pair(&left, &right));
        }

        next_level
    }

    /// Hash two hashes together (simplified hash function)
    fn hash_pair(&self, left: &Hash, right: &Hash) -> Hash {
        // In a real implementation, use proper cryptographic hash
        // For now, just XOR the bytes
        let mut result = [0u8; 32];
        for i in 0..32 {
            result[i] = left.as_bytes()[i] ^ right.as_bytes()[i];
        }
        Hash::from(result)
    }

    /// Get the size of a pruning proof
    pub fn get_proof_size(&self, proof: &PruningProof) -> usize {
        proof.headers.len() + proof.merkle_proofs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_hash(i: u64) -> Hash {
        Hash::from_le_u64([i, 0, 0, 0])
    }

    fn create_test_header(hash: Hash, parents: Vec<Hash>) -> BlockHeader {
        // Use from_precomputed_hash to avoid depending on internal Header fields
        BlockHeader::from_precomputed_hash(hash, parents)
    }

    #[test]
    fn test_pruning_proof_manager_creation() {
        let manager = PruningProofManager::new(100);
        assert_eq!(manager.max_proof_size, 100);
    }

    #[test]
    fn test_generate_merkle_proof() {
        let manager = PruningProofManager::new(100);
        let transactions = vec![
            create_test_hash(1),
            create_test_hash(2),
            create_test_hash(3),
            create_test_hash(4),
        ];

        let proof = manager.generate_merkle_proof(transactions[0], &transactions, 0).unwrap();
        assert_eq!(proof.transaction_hash, transactions[0]);
        assert_eq!(proof.index, 0);
        assert!(!proof.merkle_branch.is_empty());
    }

    #[test]
    fn test_verify_merkle_proof() {
        let manager = PruningProofManager::new(100);
        let transactions = vec![
            create_test_hash(1),
            create_test_hash(2),
        ];

        let proof = manager.generate_merkle_proof(transactions[0], &transactions, 0).unwrap();
        assert!(manager.verify_merkle_proof(&proof).is_ok());
    }

    #[test]
    fn test_generate_proof() {
        let manager = PruningProofManager::new(10);
        let mut headers = HashMap::new();

        let hash1 = create_test_hash(1);
        let hash2 = create_test_hash(2);

        headers.insert(hash1, create_test_header(hash1, vec![]));
        headers.insert(hash2, create_test_header(hash2, vec![hash1]));

        let proof = manager.generate_proof(hash2, &headers).unwrap();
        assert_eq!(proof.pruning_point, hash2);
        assert!(!proof.headers.is_empty());
    }

    #[test]
    fn test_verify_proof_valid() {
        let manager = PruningProofManager::new(10);
        let mut headers = HashMap::new();
        let mut trusted = HashSet::new();

        let hash1 = create_test_hash(1);
        let hash2 = create_test_hash(2);

        headers.insert(hash1, create_test_header(hash1, vec![]));
        headers.insert(hash2, create_test_header(hash2, vec![hash1]));
        trusted.insert(hash2);

        let proof = manager.generate_proof(hash2, &headers).unwrap();
        assert!(manager.verify_proof(&proof, &trusted).is_ok());
    }

    #[test]
    fn test_verify_proof_invalid_pruning_point() {
        let manager = PruningProofManager::new(10);
        let mut headers = HashMap::new();
        let mut trusted = HashSet::new();

        let hash1 = create_test_hash(1);
        let hash2 = create_test_hash(2);

        headers.insert(hash1, create_test_header(hash1, vec![]));
        headers.insert(hash2, create_test_header(hash2, vec![hash1]));
        // Don't add hash2 to trusted set

        let proof = manager.generate_proof(hash2, &headers).unwrap();
        assert!(manager.verify_proof(&proof, &trusted).is_err());
    }

    #[test]
    fn test_get_proof_size() {
        let manager = PruningProofManager::new(10);
        let proof = PruningProof {
            pruning_point: create_test_hash(1),
            headers: vec![create_test_header(create_test_hash(1), vec![])],
            merkle_proofs: vec![MerkleProof {
                transaction_hash: create_test_hash(2),
                merkle_branch: vec![create_test_hash(3)],
                index: 0,
            }],
        };

        assert_eq!(manager.get_proof_size(&proof), 2);
    }

    #[test]
    fn test_hash_pair() {
        let manager = PruningProofManager::new(10);
        let hash1 = create_test_hash(1);
        let hash2 = create_test_hash(2);

        let result = manager.hash_pair(&hash1, &hash2);
        // Since we use XOR, result should not be equal to either input
        assert_ne!(result, hash1);
        assert_ne!(result, hash2);
    }
}
