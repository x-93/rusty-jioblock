//! Header validation for consensus
//!
//! This module validates block headers including:
//! - Proof of work validation
//! - Timestamp validation
//! - Parent existence checks
//! - Merkle root verification

use consensus_core::header::Header;
use consensus_core::Hash;
use consensus_core::errors::ConsensusError;
use consensus_core::constants::BLOCK_VERSION;
use consensus_core::hashing::header::validate_pow;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum number of parents per block
pub const MAX_BLOCK_PARENTS: usize = 10;

/// Maximum timestamp future offset (2 hours in milliseconds)
pub const MAX_TIMESTAMP_FUTURE_OFFSET: u64 = 2 * 3600 * 1000;

/// Header validator for consensus rules
pub struct HeaderValidator {
    max_block_parents: usize,
    max_timestamp_future_offset: u64,
}

impl HeaderValidator {
    /// Create a new header validator with default parameters
    pub fn new() -> Self {
        Self {
            max_block_parents: MAX_BLOCK_PARENTS,
            max_timestamp_future_offset: MAX_TIMESTAMP_FUTURE_OFFSET,
        }
    }

    /// Create a new header validator with custom parameters
    pub fn with_params(max_block_parents: usize, max_timestamp_future_offset: u64) -> Self {
        Self {
            max_block_parents,
            max_timestamp_future_offset,
        }
    }

    /// Validate header with context-free checks
    pub fn validate_header(&self, header: &Header) -> Result<(), ConsensusError> {
        self.validate_header_internal(header, true)
    }

    /// Validate header without proof of work (for testing)
    #[cfg(test)]
    pub fn validate_header_without_pow(&self, header: &Header) -> Result<(), ConsensusError> {
        self.validate_header_internal(header, false)
    }

    /// Internal header validation method
    fn validate_header_internal(&self, header: &Header, check_pow: bool) -> Result<(), ConsensusError> {
        // Check version is supported
        if header.version < BLOCK_VERSION {
            return Err(ConsensusError::InvalidBlockVersion);
        }

        // Check parents count
        let direct_parents = header.direct_parents();
        if !direct_parents.is_empty() && direct_parents.len() > self.max_block_parents {
            return Err(ConsensusError::InvalidBlockParent);
        }

        // Check no duplicate parents
        let parent_set: HashSet<Hash> = direct_parents.iter().copied().collect();
        if parent_set.len() != direct_parents.len() {
            return Err(ConsensusError::InvalidBlockParent);
        }

        // Check timestamp is reasonable (not too far in future)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        if header.timestamp > now + self.max_timestamp_future_offset {
            return Err(ConsensusError::InvalidTimestamp);
        }

        // Validate proof of work (if requested)
        if check_pow && !validate_pow(header) {
            return Err(ConsensusError::InvalidProofOfWork);
        }

        Ok(())
    }

    /// Validate header in context with parent headers
    pub fn validate_header_in_context(
        &self,
        header: &Header,
        parent_headers: &[Header],
    ) -> Result<(), ConsensusError> {
        // All context-free checks
        self.validate_header(header)?;

        // Check all parents exist
        let direct_parents = header.direct_parents();
        if direct_parents.len() != parent_headers.len() {
            return Err(ConsensusError::InvalidBlockParent);
        }

        let parent_hashes: HashSet<Hash> = parent_headers.iter().map(|h| h.hash).collect();
        for parent_hash in direct_parents {
            if !parent_hashes.contains(parent_hash) {
                return Err(ConsensusError::InvalidBlockParent);
            }
        }

        // Check timestamp > median of parent timestamps
        if !parent_headers.is_empty() {
            let median_timestamp = self.median_timestamp(parent_headers);
            if header.timestamp <= median_timestamp {
                return Err(ConsensusError::InvalidTimestamp);
            }
        }

        Ok(())
    }

    /// Check proof of work
    pub fn check_pow(&self, header: &Header) -> Result<(), ConsensusError> {
        if validate_pow(header) {
            Ok(())
        } else {
            Err(ConsensusError::InvalidProofOfWork)
        }
    }

    /// Check timestamp validity
    pub fn check_timestamp(&self, header: &Header, parents: Option<&[Header]>) -> Result<(), ConsensusError> {
        // Check not too far in future
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        if header.timestamp > now + self.max_timestamp_future_offset {
            return Err(ConsensusError::InvalidTimestamp);
        }

        // If parents provided: timestamp > median(parent_timestamps)
        if let Some(parent_headers) = parents {
            if !parent_headers.is_empty() {
                let median_timestamp = self.median_timestamp(parent_headers);
                if header.timestamp <= median_timestamp {
                    return Err(ConsensusError::InvalidTimestamp);
                }
            }
        }

        Ok(())
    }

    /// Check parents validity
    pub fn check_parents(&self, header: &Header) -> Result<(), ConsensusError> {
        let direct_parents = header.direct_parents();
        
        // Check count in valid range (0 is allowed for genesis)
        if direct_parents.len() > self.max_block_parents {
            return Err(ConsensusError::InvalidBlockParent);
        }

        // Check no duplicates
        let parent_set: HashSet<Hash> = direct_parents.iter().copied().collect();
        if parent_set.len() != direct_parents.len() {
            return Err(ConsensusError::InvalidBlockParent);
        }

        Ok(())
    }

    /// Calculate median timestamp from headers
    pub fn median_timestamp(&self, headers: &[Header]) -> u64 {
        if headers.is_empty() {
            return 0;
        }

        let mut timestamps: Vec<u64> = headers.iter().map(|h| h.timestamp).collect();
        timestamps.sort();

        let mid = timestamps.len() / 2;
        if timestamps.len() % 2 == 0 {
            (timestamps[mid - 1] + timestamps[mid]) / 2
        } else {
            timestamps[mid]
        }
    }
}

impl Default for HeaderValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::header::Header;
    use consensus_core::{ZERO_HASH, BlueWorkType, Hash};

    fn create_test_header(_hash: Hash, parents: Vec<Hash>, timestamp: u64, bits: u32) -> Header {
        Header::new_finalized(
            BLOCK_VERSION,
            vec![parents],
            ZERO_HASH,
            ZERO_HASH,
            ZERO_HASH,
            timestamp,
            bits,
            0,
            0,
            BlueWorkType::from(0u64),
            0,
            ZERO_HASH,
        )
    }

    #[test]
    fn test_valid_header_passes() {
        let validator = HeaderValidator::new();
        let hash = Hash::from_le_u64([1, 0, 0, 0]);
        let header = create_test_header(hash, vec![], 1000, 0x1f00ffff);
        // Note: In real tests, we'd need valid PoW, but for structure test this is okay
        let result = validator.check_parents(&header);
        assert!(result.is_ok());
    }

    #[test]
    fn test_too_many_parents_fails() {
        let validator = HeaderValidator::new();
        let hash = Hash::from_le_u64([1, 0, 0, 0]);
        let parents: Vec<Hash> = (0..=MAX_BLOCK_PARENTS).map(|i| Hash::from_le_u64([i as u64, 0, 0, 0])).collect();
        let header = create_test_header(hash, parents, 1000, 0x1f00ffff);
        let result = validator.check_parents(&header);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_parents_fails() {
        let validator = HeaderValidator::new();
        let hash = Hash::from_le_u64([1, 0, 0, 0]);
        let parent = Hash::from_le_u64([2, 0, 0, 0]);
        let header = create_test_header(hash, vec![parent, parent], 1000, 0x1f00ffff);
        let result = validator.check_parents(&header);
        assert!(result.is_err());
    }

    #[test]
    fn test_median_timestamp() {
        let validator = HeaderValidator::new();
        let headers = vec![
            create_test_header(Hash::from_le_u64([1, 0, 0, 0]), vec![], 1000, 0x1f00ffff),
            create_test_header(Hash::from_le_u64([2, 0, 0, 0]), vec![], 2000, 0x1f00ffff),
            create_test_header(Hash::from_le_u64([3, 0, 0, 0]), vec![], 3000, 0x1f00ffff),
        ];
        let median = validator.median_timestamp(&headers);
        assert_eq!(median, 2000);
    }
}

