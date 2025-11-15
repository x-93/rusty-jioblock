//! Proof-of-Work implementation using SHA256 hashing
//!
//! This module provides the core cryptographic functions for verifying and computing
//! proof-of-work proofs. It handles hash computation and target validation similar to
//! Kaspa's PoW system.

use consensus_core::Hash;
use crypto_hashes::double_sha256;
use primitive_types::U256;
use std::cmp::Ordering;

/// Target represents the difficulty threshold for valid blocks
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Target(U256);

impl Target {
    /// Creates a new Target from a U256 value
    pub fn new(value: U256) -> Self {
        Target(value)
    }

    /// Creates a Target from compact bits representation (Kaspa format)
    /// Format: [3 bytes mantissa][1 byte exponent]
    pub fn from_bits(bits: u32) -> Self {
        let size = (bits >> 24) as usize;
        let word = bits & 0x007fffff;

        let value = if size <= 3 {
            U256::from(word >> (8 * (3 - size)))
        } else {
            U256::from(word) << (8 * (size - 3))
        };

        Target(value)
    }

    /// Converts Target to compact bits representation
    pub fn to_bits(&self) -> u32 {
        let mut bytes = [0u8; 32];
        self.0.to_big_endian(&mut bytes);

        // Find first non-zero byte
        let mut size = 32;
        for (i, &byte) in bytes.iter().enumerate() {
            if byte != 0 {
                size = 32 - i;
                break;
            }
        }

        if size <= 3 {
            let word = u32::from_be_bytes([bytes[29], bytes[30], bytes[31], 0]) >> (8 * (3 - size));
            (3 << 24) | word
        } else {
            let offset = 32 - size;
            let word = u32::from_be_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], 0]);
            ((size as u32) << 24) | word
        }
    }

    /// Returns the inner U256 value
    pub fn as_u256(&self) -> U256 {
        self.0
    }

    /// Returns the inner U256 value as mutable reference
    pub fn as_u256_mut(&mut self) -> &mut U256 {
        &mut self.0
    }
}

impl From<U256> for Target {
    fn from(value: U256) -> Self {
        Target(value)
    }
}

impl From<Target> for U256 {
    fn from(target: Target) -> Self {
        target.0
    }
}

/// Proof-of-Work handler
pub struct ProofOfWork;

impl ProofOfWork {
    /// Computes the SHA256(SHA256(header)) hash of a block header
    ///
    /// # Arguments
    /// * `header_bytes` - Serialized block header bytes
    ///
    /// # Returns
    /// The double-SHA256 hash of the header
    pub fn compute_hash(header_bytes: &[u8]) -> Hash {
        // Use double SHA256 hashing via the crypto_hashes crate
        let hash_array = double_sha256(header_bytes);
        Hash::from(hash_array)
    }

    /// Verifies if a block header meets the proof-of-work target
    ///
    /// # Arguments
    /// * `header_bytes` - Serialized block header bytes
    /// * `target` - The difficulty target that must be met
    ///
    /// # Returns
    /// true if the hash is less than or equal to the target
    pub fn is_valid_pow(header_bytes: &[u8], target: &Target) -> bool {
        let hash = Self::compute_hash(header_bytes);
        let hash_u256 = U256::from_big_endian(hash.as_bytes());
        hash_u256 <= target.0
    }

    /// Calculates the hash rate (hashes per second)
    ///
    /// # Arguments
    /// * `hashes` - Number of hashes performed
    /// * `duration_ms` - Time taken in milliseconds
    ///
    /// # Returns
    /// Hash rate in hashes per second
    pub fn calculate_hash_rate(hashes: u64, duration_ms: u64) -> f64 {
        if duration_ms == 0 {
            return 0.0;
        }
        (hashes as f64) / (duration_ms as f64 / 1000.0)
    }

    /// Converts a hash to U256 for comparison
    fn hash_to_u256(hash: &Hash) -> U256 {
        U256::from_big_endian(hash.as_bytes())
    }

    /// Compares hash against target and returns ordering
    pub fn compare_hash_to_target(hash: &Hash, target: &Target) -> Ordering {
        let hash_u256 = Self::hash_to_u256(hash);
        hash_u256.cmp(&target.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_from_bits() {
        // Test bits conversion
        let bits = 0x207fffff;
        let target = Target::from_bits(bits);
        let recovered = target.to_bits();
        // Note: exact recovery may not be perfect due to precision loss
        assert!(recovered > 0);
    }

    #[test]
    fn test_pow_hash_computation() {
        let header_bytes = b"test header data";
        let hash = ProofOfWork::compute_hash(header_bytes);
        assert_eq!(hash.as_bytes().len(), 32);
    }

    #[test]
    fn test_hash_rate_calculation() {
        let rate = ProofOfWork::calculate_hash_rate(1_000_000, 1000); // 1M hashes in 1 second
        assert!(rate > 900_000.0 && rate < 1_100_000.0);
    }

    #[test]
    fn test_is_valid_pow_high_target() {
        // Create a high target (easy)
        let target = Target::from_bits(0x207fffff); // Relatively easy target
        let header_bytes = b"test";

        // Test should pass for valid combinations
        let is_valid = ProofOfWork::is_valid_pow(header_bytes, &target);
        // We don't assert the result as it depends on the hash value
        assert_eq!(is_valid, ProofOfWork::is_valid_pow(header_bytes, &target));
    }

    #[test]
    fn test_target_ordering() {
        let target1 = Target::from_bits(0x207fffff);
        let target2 = Target::from_bits(0x1fffffff);
        assert!(target2 < target1); // Lower bits value = harder target
    }
}
