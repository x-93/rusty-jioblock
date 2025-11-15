//! Coinbase transaction processing
//!
//! This module handles coinbase transaction creation, validation,
//! and reward calculation for the consensus process.

use consensus_core::tx::{Transaction, TransactionOutput, ScriptPublicKey};
use consensus_core::subnets;
use crate::consensus::types::ConsensusConfig;

/// Coinbase transaction processor
pub struct CoinbaseProcessor {
    config: ConsensusConfig,
}

impl CoinbaseProcessor {
    /// Create a new coinbase processor
    pub fn new(config: ConsensusConfig) -> Self {
        Self { config }
    }

    /// Create a coinbase transaction for a new block
    pub fn create_coinbase_transaction(
        &self,
        miner_address: &ScriptPublicKey,
        block_height: u64,
        fees: u64,
    ) -> Transaction {
        let reward = self.calculate_block_reward(block_height) + fees;

        let output = TransactionOutput {
            value: reward,
            script_public_key: miner_address.clone(),
        };

        Transaction::new(
            1,
            vec![], // Coinbase has no inputs
            vec![output],
            0,
            consensus_core::subnets::SUBNETWORK_ID_COINBASE,
            0,
            format!("Block {}", block_height).into_bytes(),
        )
    }

    /// Calculate block reward based on block height
    pub fn calculate_block_reward(&self, block_height: u64) -> u64 {
        // Simple halving every 210,000 blocks (like Bitcoin)
        let halvings = block_height / 210_000;
        let initial_reward = 50_000_000; // 50 coins in smallest unit

        if halvings >= 64 {
            0 // No more rewards after 64 halvings
        } else {
            initial_reward >> halvings // Divide by 2^halvings
        }
    }

    /// Validate coinbase transaction
    pub fn validate_coinbase(&self, coinbase: &Transaction, expected_reward: u64) -> Result<(), String> {
        // Must have no inputs
        if !coinbase.inputs.is_empty() {
            return Err("Coinbase transaction must have no inputs".to_string());
        }

        // Must have exactly one output
        if coinbase.outputs.len() != 1 {
            return Err("Coinbase transaction must have exactly one output".to_string());
        }

        // Output value must match expected reward
        if coinbase.outputs[0].value != expected_reward {
            return Err(format!(
                "Coinbase output value {} does not match expected reward {}",
                coinbase.outputs[0].value, expected_reward
            ));
        }

        // Must use coinbase subnetwork ID
        if coinbase.subnetwork_id != consensus_core::subnets::SUBNETWORK_ID_COINBASE {
            return Err("Coinbase transaction must use coinbase subnetwork ID".to_string());
        }

        Ok(())
    }

    /// Get coinbase maturity period
    pub fn get_coinbase_maturity(&self) -> u64 {
        self.config.coinbase_maturity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::subnets::SUBNETWORK_ID_COINBASE;

    #[test]
    fn test_calculate_block_reward() {
        let config = ConsensusConfig::default();
        let processor = CoinbaseProcessor::new(config);

        // Initial reward
        assert_eq!(processor.calculate_block_reward(0), 50_000_000);
        assert_eq!(processor.calculate_block_reward(209_999), 50_000_000);

        // First halving
        assert_eq!(processor.calculate_block_reward(210_000), 25_000_000);
        assert_eq!(processor.calculate_block_reward(419_999), 25_000_000);

        // Second halving
        assert_eq!(processor.calculate_block_reward(420_000), 12_500_000);

        // Zero after many halvings
        assert_eq!(processor.calculate_block_reward(13_440_000), 0);
    }

    #[test]
    fn test_create_coinbase_transaction() {
        let config = ConsensusConfig::default();
        let processor = CoinbaseProcessor::new(config);

    let miner_address = ScriptPublicKey::new(0, vec![1, 2, 3, 4].into());
        let coinbase = processor.create_coinbase_transaction(&miner_address, 100, 1000);

        assert!(coinbase.inputs.is_empty());
        assert_eq!(coinbase.outputs.len(), 1);
        assert_eq!(coinbase.outputs[0].value, 50_000_000 + 1000); // reward + fees
        assert_eq!(coinbase.outputs[0].script_public_key, miner_address);
        assert_eq!(coinbase.subnetwork_id, SUBNETWORK_ID_COINBASE);
        assert_eq!(coinbase.payload, b"Block 100");
    }

    #[test]
    fn test_validate_coinbase() {
        let config = ConsensusConfig::default();
        let processor = CoinbaseProcessor::new(config);

    let miner_address = ScriptPublicKey::new(0, vec![1, 2, 3, 4].into());
        let coinbase = processor.create_coinbase_transaction(&miner_address, 100, 1000);

        // Valid coinbase should pass
        assert!(processor.validate_coinbase(&coinbase, 50_001_000).is_ok());

        // Wrong reward should fail
        assert!(processor.validate_coinbase(&coinbase, 50_000_000).is_err());
    }
}
