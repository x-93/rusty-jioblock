//! Mining job definitions and management
//!
//! This module defines the MiningJob structure that encapsulates all necessary
//! information for a mining worker thread to perform proof-of-work computations.

use crate::pow::Target;
use rpc_core::model::BlockTemplate;
use consensus_core::tx::Transaction;
use consensus_core::Hash;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a mining job that workers process
#[derive(Clone, Debug)]
pub struct MiningJob {
    /// The block template to mine
    pub template: BlockTemplate,
    /// The difficulty target to meet
    pub target: Target,
    /// Timestamp when this job was created
    pub job_timestamp: u64,
    /// Job identifier for tracking
    pub job_id: u64,
}

impl MiningJob {
    /// Creates a new MiningJob
    ///
    /// # Arguments
    /// * `template` - The block template containing transactions and metadata
    /// * `target` - The difficulty target
    ///
    /// # Returns
    /// A new MiningJob instance
    pub fn new(template: BlockTemplate, target: Target) -> Self {
        Self {
            template,
            target,
            job_timestamp: current_timestamp(),
            job_id: generate_job_id(),
        }
    }

    /// Creates a MiningJob with explicit timestamp and ID
    pub fn with_metadata(template: BlockTemplate, target: Target, timestamp: u64, job_id: u64) -> Self {
        Self {
            template,
            target,
            job_timestamp: timestamp,
            job_id,
        }
    }

    /// Returns whether this job is still recent (not older than max_age_ms)
    pub fn is_recent(&self, max_age_ms: u64) -> bool {
        let now = current_timestamp();
        now.saturating_sub(self.job_timestamp) < max_age_ms
    }

    /// Gets the age of the job in milliseconds
    pub fn age_ms(&self) -> u64 {
        current_timestamp().saturating_sub(self.job_timestamp)
    }

    /// Returns the version number from the template
    pub fn version(&self) -> u32 {
        self.template.version
    }

    /// Returns the coinbase value
    pub fn coinbase_value(&self) -> u64 {
        self.template.coinbase_value
    }

    /// Returns the number of transactions in the template
    pub fn transaction_count(&self) -> usize {
        self.template.transactions.len()
    }

    /// Returns a reference to the transactions
    pub fn transactions(&self) -> &[Transaction] {
        &self.template.transactions
    }

    /// Returns the timestamp from the template
    pub fn template_timestamp(&self) -> u64 {
        self.template.timestamp
    }

    /// Creates a header with the given nonce
    ///
    /// # Arguments
    /// * `nonce` - The nonce value to test
    ///
    /// # Returns
    /// A serialized header as bytes with the nonce set
    pub fn header_with_nonce(&self, nonce: u64) -> Vec<u8> {
        // Serialize the header including all template data with the nonce
        // Format: version || parents || merkle_root || timestamp || bits || nonce
        let mut header_bytes = Vec::new();

        // Version (4 bytes)
        header_bytes.extend_from_slice(&self.template.version.to_le_bytes());

        // Parent hashes (variable length)
        header_bytes.extend_from_slice(&(self.template.parent_hashes.len() as u32).to_le_bytes());
        for parent_hash in &self.template.parent_hashes {
            header_bytes.extend_from_slice(parent_hash.as_bytes());
        }

        // Timestamp (8 bytes)
        header_bytes.extend_from_slice(&self.template.timestamp.to_le_bytes());

        // Bits/difficulty (4 bytes)
        header_bytes.extend_from_slice(&self.template.bits.to_le_bytes());

        // Nonce (8 bytes)
        header_bytes.extend_from_slice(&nonce.to_le_bytes());

        header_bytes
    }

    /// Gets the difficulty bits
    pub fn bits(&self) -> u32 {
        self.template.bits
    }
}

/// Mined block result that workers send back
#[derive(Clone, Debug)]
pub struct MinedBlock {
    /// The job that produced this block
    pub job_id: u64,
    /// The worker ID that mined this block
    pub worker_id: usize,
    /// The nonce that produced valid PoW
    pub nonce: u64,
    /// The hash of the mined block
    pub block_hash: Hash,
    /// Number of iterations performed to find this block
    pub iterations: u64,
    /// Time taken to mine in milliseconds
    pub time_ms: u64,
}

impl MinedBlock {
    /// Creates a new MinedBlock result
    pub fn new(
        job_id: u64,
        worker_id: usize,
        nonce: u64,
        block_hash: Hash,
        iterations: u64,
        time_ms: u64,
    ) -> Self {
        Self {
            job_id,
            worker_id,
            nonce,
            block_hash,
            iterations,
            time_ms,
        }
    }

    /// Calculates the hash rate for this mining result
    pub fn hash_rate(&self) -> f64 {
        if self.time_ms == 0 {
            return 0.0;
        }
        (self.iterations as f64) / (self.time_ms as f64 / 1000.0)
    }
}

/// Gets the current timestamp in milliseconds since UNIX_EPOCH
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Generates a unique job ID
fn generate_job_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::Hash as ConsensusHash;

    fn create_test_template() -> BlockTemplate {
        BlockTemplate {
            version: 1,
            parent_hashes: vec![ConsensusHash::default()],
            transactions: Vec::new(),
            coinbase_value: 5_000_000_000,
            bits: 0x207fffff,
            timestamp: 1000,
            pay_address: "test".to_string(),
            target: "0".to_string(),
        }
    }

    #[test]
    fn test_mining_job_creation() {
        let template = create_test_template();
        let target = Target::from_bits(0x207fffff);
        let job = MiningJob::new(template, target);

        assert_eq!(job.version(), 1);
        assert_eq!(job.coinbase_value(), 5_000_000_000);
        assert!(job.is_recent(10000));
    }

    #[test]
    fn test_mined_block_hash_rate() {
        let block = MinedBlock::new(1, 0, 12345, ConsensusHash::default(), 1_000_000, 1000);
        let rate = block.hash_rate();
        assert!(rate > 900_000.0 && rate < 1_100_000.0);
    }

    #[test]
    fn test_job_age_calculation() {
        let template = create_test_template();
        let target = Target::from_bits(0x207fffff);
        let job = MiningJob::new(template, target);

        let age = job.age_ms();
        assert!(age < 100); // Should be very recent
    }

    #[test]
    fn test_header_with_nonce() {
        let template = create_test_template();
        let target = Target::from_bits(0x207fffff);
        let job = MiningJob::new(template, target);

        let header_bytes = job.header_with_nonce(42);
        assert!(header_bytes.len() > 0);
    }
}
