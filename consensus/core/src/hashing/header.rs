use crate::Hash;
use crate::header::Header;
use crate::constants::{BLOCK_VERSION_KHASHV1, BLOCK_VERSION_KHASHV2};
use crypto_hashes::pow_hash::PowB3Hash;
use primitive_types::U256;

/// Converts difficulty bits to target
fn bits_to_target(bits: u32) -> U256 {
    let exponent = (bits >> 24) as usize;
    let mantissa = bits & 0x00ffffff;

    if exponent <= 3 {
        U256::from(mantissa >> (8 * (3 - exponent)))
    } else {
        U256::from(mantissa) << (8 * (exponent - 3))
    }
}

/// Computes the hash of a block header
pub fn calculate_header_hash(header: &Header) -> Hash {
    // Serialize the header without the hash field
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&header.version.to_le_bytes());
    
    // Serialize parents
    for parent_level in &header.parents_by_level {
        for parent in parent_level {
            bytes.extend_from_slice(parent.as_bytes());
        }
    }
    
    // Add other header fields
    bytes.extend_from_slice(header.hash_merkle_root.as_bytes());
    bytes.extend_from_slice(header.accepted_id_merkle_root.as_bytes());
    bytes.extend_from_slice(header.utxo_commitment.as_bytes());
    bytes.extend_from_slice(&header.timestamp.to_le_bytes());
    bytes.extend_from_slice(&header.bits.to_le_bytes());
    bytes.extend_from_slice(&header.nonce.to_le_bytes());
    bytes.extend_from_slice(&header.daa_score.to_le_bytes());
    bytes.extend_from_slice(&header.blue_work.to_bytes());
    bytes.extend_from_slice(&header.blue_score.to_le_bytes());
    bytes.extend_from_slice(header.pruning_point.as_bytes());

    // Double SHA256 the entire header
    let hash = super::double_sha256(&bytes);
    // `double_sha256` already returns a `Hash`
    hash
}

/// Computes the proof of work hash for a block header based on its version
pub fn calculate_pow_hash(header: &Header) -> Hash {
    match header.version {
        v if v == BLOCK_VERSION_KHASHV1 => {
            // For KHASHV1, use PowB3Hash
            // First calculate the pre-pow hash (header hash without nonce)
            let mut pre_pow_bytes = Vec::new();
            pre_pow_bytes.extend_from_slice(&header.version.to_le_bytes());
            for parent_level in &header.parents_by_level {
                for parent in parent_level {
                    pre_pow_bytes.extend_from_slice(parent.as_bytes());
                }
            }
            pre_pow_bytes.extend_from_slice(header.hash_merkle_root.as_bytes());
            pre_pow_bytes.extend_from_slice(header.accepted_id_merkle_root.as_bytes());
            pre_pow_bytes.extend_from_slice(header.utxo_commitment.as_bytes());
            pre_pow_bytes.extend_from_slice(&header.timestamp.to_le_bytes());
            pre_pow_bytes.extend_from_slice(&header.bits.to_le_bytes());
            // Note: nonce is not included in pre-pow hash
            pre_pow_bytes.extend_from_slice(&header.daa_score.to_le_bytes());
            pre_pow_bytes.extend_from_slice(&header.blue_work.to_bytes());
            pre_pow_bytes.extend_from_slice(&header.blue_score.to_le_bytes());
            pre_pow_bytes.extend_from_slice(header.pruning_point.as_bytes());
            
            let pre_pow_hash = super::double_sha256(&pre_pow_bytes);
            
            // Create PowB3Hash with pre-pow hash and timestamp
            let mut pow_hasher = PowB3Hash::new(pre_pow_hash, header.timestamp);
            // Finalize with the nonce to obtain the PoW hash
            let result = pow_hasher.finalize_with_nonce(header.nonce);
            
            // `finalize_with_nonce` already returns a `Hash`
            result
        }
        v if v == BLOCK_VERSION_KHASHV2 => {
            // For KHASHV2, use matrix-based hashing
            let mut hash_input = header.hash.as_bytes().to_vec();
            hash_input.extend_from_slice(&header.nonce.to_le_bytes());

            // Use double SHA256 for now, will be replaced with matrix hash
            let hash = super::double_sha256(&hash_input);
            hash
        }
        _ => {
            // For unknown versions, fallback to header hash
            header.hash
        }
    }
}

/// Validates whether the block's proof of work meets the required target difficulty
pub fn validate_pow(header: &Header) -> bool {
    let pow_hash = calculate_pow_hash(header);
    let target = bits_to_target(header.bits);
    
    // Convert hash to U256 for comparison
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(pow_hash.as_bytes());
    let pow_hash_num = U256::from_big_endian(&bytes);
    
    pow_hash_num <= target
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlueWorkType;

    #[test]
    fn test_header_hash() {
        let header = Header::new_finalized(
            1,
            vec![vec![Hash::default()]],
            Hash::default(),
            Hash::default(),
            Hash::default(),
            1699545600000,
            0x1f00ffff,
            0,
            0,
            BlueWorkType::from(0u64),
            0,
            Hash::default(),
        );
        
        let hash = calculate_header_hash(&header);
        assert_ne!(hash, Hash::default());
    }

    #[test]
    fn test_pow_hash_v1() {
        let header = Header::new_finalized(
            BLOCK_VERSION_KHASHV1,
            vec![vec![Hash::default()]],
            Hash::default(),
            Hash::default(),
            Hash::default(),
            1699545600000,
            0x1f00ffff,
            123456,  // Test nonce
            0,
            BlueWorkType::from(0u64),
            0,
            Hash::default(),
        );

        let pow_hash = calculate_pow_hash(&header);
        assert_ne!(pow_hash, Hash::default());
    }

    #[test]
    fn test_pow_hash_v2() {
        let header = Header::new_finalized(
            BLOCK_VERSION_KHASHV2,
            vec![vec![Hash::default()]],
            Hash::default(),
            Hash::default(),
            Hash::default(),
            1699545600000,
            0x1f00ffff,
            123456,  // Test nonce
            0,
            BlueWorkType::from(0u64),
            0,
            Hash::default(),
        );

        let pow_hash = calculate_pow_hash(&header);
        assert_ne!(pow_hash, Hash::default());
    }

    #[test]
    fn test_validate_pow() {
        // Use a very easy target that should always pass
        let header = Header::new_finalized(
            BLOCK_VERSION_KHASHV1,
            vec![vec![Hash::default()]],
            Hash::default(),
            Hash::default(),
            Hash::default(),
            1699545600000,
            0x21FFFFFF, // Very easy target (exponent 33, mantissa 0xFFFFFF)
            0,
            0,
            BlueWorkType::from(0u64),
            0,
            Hash::default(),
        );

        let pow_hash = calculate_pow_hash(&header);
        let target = bits_to_target(header.bits);
        println!("PoW hash: {:?}", pow_hash);
        println!("Target: {:?}", target);

        // For easy target, the hash should be less than or equal to target
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(pow_hash.as_bytes());
        let pow_hash_num = U256::from_big_endian(&bytes);
        println!("PoW hash num: {:?}", pow_hash_num);

        let is_valid = validate_pow(&header);
        assert!(is_valid, "PoW validation failed with easy target");
    }
}

/// Computes the header hash while allowing override of the nonce and timestamp fields.
/// Used by PoW routines which need the pre-pow hash (with time/nonce zeroed or overridden).
pub fn hash_override_nonce_time(header: &Header, nonce_override: u64, time_override: u64) -> Hash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&header.version.to_le_bytes());
    
    // Serialize parents
    for parent_level in &header.parents_by_level {
        for parent in parent_level {
            bytes.extend_from_slice(parent.as_bytes());
        }
    }

    // Add other header fields with overrides for time and nonce
    bytes.extend_from_slice(header.hash_merkle_root.as_bytes());
    bytes.extend_from_slice(header.accepted_id_merkle_root.as_bytes());
    bytes.extend_from_slice(header.utxo_commitment.as_bytes());
    bytes.extend_from_slice(&time_override.to_le_bytes());
    bytes.extend_from_slice(&header.bits.to_le_bytes());
    bytes.extend_from_slice(&nonce_override.to_le_bytes());
    bytes.extend_from_slice(&header.daa_score.to_le_bytes());
    bytes.extend_from_slice(&header.blue_work.to_bytes());
    bytes.extend_from_slice(&header.blue_score.to_le_bytes());
    bytes.extend_from_slice(header.pruning_point.as_bytes());

    super::double_sha256(&bytes)
}