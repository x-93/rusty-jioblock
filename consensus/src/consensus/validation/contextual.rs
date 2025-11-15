//! Contextual validation for consensus
//!
//! This module provides context-dependent validation rules that require
//! knowledge of the blockchain state, such as UTXO validation and
//! dependency checks.

use consensus_core::block::Block;
use consensus_core::tx::Transaction;
use consensus_core::errors::ConsensusError;
use consensus_core::constants::COINBASE_MATURITY;
use super::block_validator::BlockValidator;
use super::transaction_validator::{TransactionValidator, UtxoView};
use std::sync::Arc;

/// Contextual validator for consensus rules
pub struct ContextualValidator {
    block_validator: Arc<BlockValidator>,
    transaction_validator: Arc<TransactionValidator>,
}

impl ContextualValidator {
    /// Create a new contextual validator
    pub fn new(
        block_validator: Arc<BlockValidator>,
        transaction_validator: Arc<TransactionValidator>,
    ) -> Self {
        Self {
            block_validator,
            transaction_validator,
        }
    }

    /// Validate block in context with UTXO set
    pub fn validate_block_with_utxo(
        &self,
        block: &Block,
        utxo_view: &dyn UtxoView,
        current_daa_score: u64,
    ) -> Result<u64, ConsensusError> {
        // First do context-free validation
        self.block_validator.validate_block(block)?;

        // Validate all transactions with UTXO context
        let mut total_fees = 0u64;
        for (idx, tx) in block.transactions.iter().enumerate() {
            if idx == 0 {
                // Coinbase doesn't need UTXO validation
                continue;
            }

            let fee = self
                .transaction_validator
                .validate_transaction_with_utxo(tx, utxo_view, current_daa_score)?;
            total_fees += fee;
        }

        Ok(total_fees)
    }

    /// Validate transaction dependencies
    pub fn validate_transaction_dependencies(
        &self,
        transactions: &[Transaction],
        utxo_view: &dyn UtxoView,
    ) -> Result<(), ConsensusError> {
        // Check that all transaction inputs reference existing UTXOs
        for tx in transactions {
            if tx.is_coinbase() {
                continue;
            }

            for input in &tx.inputs {
                if utxo_view.get(&input.previous_outpoint).is_none() {
                    return Err(ConsensusError::InvalidUtxoReference);
                }
            }
        }

        Ok(())
    }

    /// Validate coinbase maturity for transaction inputs
    pub fn validate_coinbase_maturity(
        &self,
        tx: &Transaction,
        utxo_view: &dyn UtxoView,
        current_daa_score: u64,
    ) -> Result<(), ConsensusError> {
        if tx.is_coinbase() {
            return Ok(());
        }

        for input in &tx.inputs {
            if let Some(utxo) = utxo_view.get(&input.previous_outpoint) {
                if utxo.is_coinbase {
                    let maturity_age = current_daa_score.saturating_sub(utxo.block_daa_score);
                    if maturity_age < COINBASE_MATURITY {
                        return Err(ConsensusError::InvalidUtxoReference);
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate block reward matches expected
    pub fn validate_block_reward(
        &self,
        block: &Block,
        expected_reward: u64,
    ) -> Result<(), ConsensusError> {
        if block.transactions.is_empty() {
            return Err(ConsensusError::EmptyTransactionList);
        }

        let coinbase = &block.transactions[0];
        let total_output: u64 = coinbase.outputs.iter().map(|o| o.value).sum();

        if total_output > expected_reward {
            return Err(ConsensusError::InvalidCoinbaseTransaction);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::tx::UtxoEntry;
    use std::collections::HashMap;
    use consensus_core::tx::TransactionOutpoint;

    struct TestUtxoView {
        utxos: HashMap<TransactionOutpoint, UtxoEntry>,
    }

    impl TestUtxoView {
        fn new() -> Self {
            Self {
                utxos: HashMap::new(),
            }
        }

        fn add_utxo(&mut self, outpoint: TransactionOutpoint, entry: UtxoEntry) {
            self.utxos.insert(outpoint, entry);
        }
    }

    impl UtxoView for TestUtxoView {
        fn get(&self, outpoint: &TransactionOutpoint) -> Option<&UtxoEntry> {
            self.utxos.get(outpoint)
        }
    }

    #[test]
    fn test_validate_coinbase_maturity() {
        use consensus_core::tx::{TransactionInput, TransactionOutput, UtxoEntry, ScriptPublicKey};
        use consensus_core::Hash;
        use crate::consensus::validation::header_validator::HeaderValidator;

        let header_validator = Arc::new(HeaderValidator::new());
        let tx_validator = Arc::new(TransactionValidator::new());
        let block_validator = Arc::new(BlockValidator::new(
            header_validator.clone(),
            tx_validator.clone(),
        ));
        let contextual_validator = ContextualValidator::new(block_validator, tx_validator);

        let mut utxo_view = TestUtxoView::new();
        let outpoint = TransactionOutpoint::new(
            Hash::from_le_u64([1, 0, 0, 0]),
            0,
        );
        let utxo = UtxoEntry::new(
            5000,
            ScriptPublicKey::from_vec(0, Vec::new()),
            100, // block_daa_score
            true, // is_coinbase
        );
        utxo_view.add_utxo(outpoint, utxo);

        let input = TransactionInput::new(outpoint, Vec::new(), 0, 0);
        let output = TransactionOutput::new(
            3000,
            ScriptPublicKey::from_vec(0, Vec::new()),
        );
        // Use a non-coinbase subnetwork ID for regular transactions
        let mut subnet_bytes = [0u8; 20];
        subnet_bytes[0] = 1; // Make it non-zero to distinguish from coinbase
        let subnetwork_id = consensus_core::subnets::SubnetworkId::new(subnet_bytes);
        let tx = Transaction::new(
            1,
            vec![input],
            vec![output],
            0,
            subnetwork_id,
            0,
            Vec::new(),
        );

        // Current DAA score is 150, maturity is 100, so age is 50 < 100, should fail
        let result = contextual_validator.validate_coinbase_maturity(&tx, &utxo_view, 150);
        assert!(result.is_err());

        // Current DAA score is 250, maturity is 100, so age is 150 > 100, should pass
        let result = contextual_validator.validate_coinbase_maturity(&tx, &utxo_view, 250);
        assert!(result.is_ok());
    }
}

