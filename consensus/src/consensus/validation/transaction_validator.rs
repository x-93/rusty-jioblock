//! Transaction validation for consensus
//!
//! This module validates transactions including:
//! - Input/output validation
//! - Amount validation
//! - Fee calculation
//! - UTXO validation

use consensus_core::tx::{
    Transaction, TransactionOutpoint, UtxoEntry,
};
use consensus_core::errors::ConsensusError;
use consensus_core::constants::COINBASE_MATURITY;
use std::collections::HashSet;

/// Maximum transaction size in bytes
pub const MAX_TRANSACTION_SIZE: u64 = 1_000_000;

/// Maximum money supply (21 billion Jiocoins * 100 million sompi per Jiocoin)
pub const MAX_MONEY: u64 = 21_000_000_000 * 100_000_000;

/// Transaction validator for consensus rules
pub struct TransactionValidator {
    max_tx_size: u64,
    max_money: u64,
    coinbase_maturity: u64,
}

impl TransactionValidator {
    /// Create a new transaction validator with default parameters
    pub fn new() -> Self {
        Self {
            max_tx_size: MAX_TRANSACTION_SIZE,
            max_money: MAX_MONEY,
            coinbase_maturity: COINBASE_MATURITY,
        }
    }

    /// Create a new transaction validator with custom parameters
    pub fn with_params(max_tx_size: u64, max_money: u64, coinbase_maturity: u64) -> Self {
        Self {
            max_tx_size,
            max_money,
            coinbase_maturity,
        }
    }

    /// Validate transaction with context-free checks
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<(), ConsensusError> {
        // Check version >= 1
        if tx.version == 0 {
            return Err(ConsensusError::InvalidTransaction);
        }

        // Coinbase transactions are allowed to have empty inputs
        if tx.is_coinbase() {
            // Coinbase must have at least one output
            if tx.outputs.is_empty() {
                return Err(ConsensusError::InvalidTransaction);
            }
            // Coinbase validation is done at block level
            return Ok(());
        }

        // Check inputs not empty (except coinbase)
        if tx.inputs.is_empty() {
            return Err(ConsensusError::InvalidTransaction);
        }

        // Check outputs not empty
        if tx.outputs.is_empty() {
            return Err(ConsensusError::InvalidTransaction);
        }

        // Check no output value is zero (for non-coinbase)
        for output in &tx.outputs {
            if output.value == 0 {
                return Err(ConsensusError::InvalidTransaction);
            }
        }

        // Check total output doesn't overflow
        let total_output: u128 = tx.outputs.iter().map(|o| o.value as u128).sum();
        if total_output > self.max_money as u128 {
            return Err(ConsensusError::InvalidTransaction);
        }

        // Check transaction size <= max_size (approximate)
        let tx_size = self.estimate_transaction_size(tx);
        if tx_size > self.max_tx_size {
            return Err(ConsensusError::InvalidTransaction);
        }

        // Check no duplicate inputs
        let mut input_set = HashSet::new();
        for input in &tx.inputs {
            if !input_set.insert(input.previous_outpoint) {
                return Err(ConsensusError::InvalidTransaction);
            }
        }

        Ok(())
    }

    /// Validate transaction with UTXO context
    pub fn validate_transaction_with_utxo(
        &self,
        tx: &Transaction,
        utxo_view: &dyn UtxoView,
        current_daa_score: u64,
    ) -> Result<u64, ConsensusError> {
        // Context-free validation first
        self.validate_transaction(tx)?;

        // Coinbase transactions don't need UTXO validation
        if tx.is_coinbase() {
            return Ok(0);
        }

        // Validate all inputs reference existing UTXOs
        let mut total_input: u128 = 0;
        for input in &tx.inputs {
            let utxo = utxo_view
                .get(&input.previous_outpoint)
                .ok_or(ConsensusError::InvalidUtxoReference)?;

            // Check coinbase maturity
            if utxo.is_coinbase {
                let maturity_age = current_daa_score.saturating_sub(utxo.block_daa_score);
                if maturity_age < self.coinbase_maturity {
                    return Err(ConsensusError::InvalidUtxoReference);
                }
            }

            total_input += utxo.amount as u128;
        }

        // Calculate total output
        let total_output: u128 = tx.outputs.iter().map(|o| o.value as u128).sum();

        // Check total input >= total output
        if total_input < total_output {
            return Err(ConsensusError::InsufficientFunds);
        }

        // Calculate fee
        let fee = (total_input - total_output) as u64;

        Ok(fee)
    }

    /// Calculate transaction fee
    pub fn calculate_fee(
        &self,
        tx: &Transaction,
        utxo_view: &dyn UtxoView,
    ) -> Result<u64, ConsensusError> {
        if tx.is_coinbase() {
            return Ok(0);
        }

        let mut total_input: u128 = 0;
        for input in &tx.inputs {
            let utxo = utxo_view
                .get(&input.previous_outpoint)
                .ok_or(ConsensusError::InvalidUtxoReference)?;
            total_input += utxo.amount as u128;
        }

        let total_output: u128 = tx.outputs.iter().map(|o| o.value as u128).sum();

        if total_input < total_output {
            return Err(ConsensusError::InsufficientFunds);
        }

        Ok((total_input - total_output) as u64)
    }

    /// Estimate transaction size in bytes
    fn estimate_transaction_size(&self, tx: &Transaction) -> u64 {
        // Base size
        let mut size = 8; // version + lock_time + subnetwork_id + gas

        // Inputs size
        for input in &tx.inputs {
            size += 36; // outpoint (32 + 4)
            size += 4; // sequence
            size += 1; // sig_op_count
            size += 4; // signature_script length (varint estimate)
            size += input.signature_script.len() as u64;
        }

        // Outputs size
        for output in &tx.outputs {
            size += 8; // value
            size += 2; // script_pubkey version
            size += 4; // script_pubkey length (varint estimate)
            size += output.script_public_key.script().len() as u64;
        }

        // Payload size
        size += 4; // payload length (varint estimate)
        size += tx.payload.len() as u64;

        size
    }
}

impl Default for TransactionValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for UTXO view operations
pub trait UtxoView {
    /// Get a UTXO entry by outpoint
    fn get(&self, outpoint: &TransactionOutpoint) -> Option<&UtxoEntry>;
}

// Implement UtxoView for consensus_core::utxo::UtxoView
impl<'a> UtxoView for consensus_core::utxo::UtxoView<'a> {
    fn get(&self, outpoint: &TransactionOutpoint) -> Option<&UtxoEntry> {
        self.get(outpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::tx::{TransactionInput, TransactionOutput, ScriptPublicKey};
    use consensus_core::Hash;
    use std::collections::HashMap;

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

    fn create_test_tx(inputs: Vec<TransactionInput>, outputs: Vec<TransactionOutput>) -> Transaction {
        // Use a non-coinbase subnetwork ID for regular transactions
        // Create a non-zero subnetwork ID (coinbase is all zeros)
        let mut subnet_bytes = [0u8; 20];
        subnet_bytes[0] = 1; // Make it non-zero to distinguish from coinbase
        let subnetwork_id = consensus_core::subnets::SubnetworkId::new(subnet_bytes);
        Transaction::new(
            1,
            inputs,
            outputs,
            0,
            subnetwork_id,
            0,
            Vec::new(),
        )
    }

    fn create_test_tx_with_hash(inputs: Vec<TransactionInput>, outputs: Vec<TransactionOutput>) -> (Transaction, Hash) {
        let tx = create_test_tx(inputs, outputs);
        let hash = tx.id();
        (tx, hash)
    }

    #[test]
    fn test_valid_transaction_passes() {
        let validator = TransactionValidator::new();
        let outpoint = TransactionOutpoint::new(
            Hash::from_le_u64([1, 0, 0, 0]),
            0,
        );
        let input = TransactionInput::new(outpoint, Vec::new(), 0, 0);
        let output = TransactionOutput::new(
            1000,
            ScriptPublicKey::from_vec(0, Vec::new()),
        );
        let tx = create_test_tx(vec![input], vec![output]);
        let result = validator.validate_transaction(&tx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_inputs_fails() {
        let validator = TransactionValidator::new();
        let output = TransactionOutput::new(
            1000,
            ScriptPublicKey::from_vec(0, Vec::new()),
        );
        let tx = create_test_tx(vec![], vec![output]);
        let result = validator.validate_transaction(&tx);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_output_fails() {
        let validator = TransactionValidator::new();
        let outpoint = TransactionOutpoint::new(
            Hash::from_le_u64([1, 0, 0, 0]),
            0,
        );
        let input = TransactionInput::new(outpoint, Vec::new(), 0, 0);
        let output = TransactionOutput::new(
            0,
            ScriptPublicKey::from_vec(0, Vec::new()),
        );
        let tx = create_test_tx(vec![input], vec![output]);
        let result = validator.validate_transaction(&tx);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_fee() {
        let validator = TransactionValidator::new();
        let mut utxo_view = TestUtxoView::new();
        
        let outpoint = TransactionOutpoint::new(
            Hash::from_le_u64([1, 0, 0, 0]),
            0,
        );
        let utxo = UtxoEntry::new(
            5000,
            ScriptPublicKey::from_vec(0, Vec::new()),
            100,
            false,
        );
        utxo_view.add_utxo(outpoint, utxo);

        let input = TransactionInput::new(outpoint, Vec::new(), 0, 0);
        let output = TransactionOutput::new(
            3000,
            ScriptPublicKey::from_vec(0, Vec::new()),
        );
        let tx = create_test_tx(vec![input], vec![output]);

        let fee = validator.calculate_fee(&tx, &utxo_view).unwrap();
        assert_eq!(fee, 2000);
    }
}

