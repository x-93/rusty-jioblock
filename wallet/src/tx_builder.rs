use consensus_core::{
    tx::{Transaction, TransactionInput, TransactionOutput, TransactionOutpoint, ScriptPublicKey},
    constants::SOMPI_PER_JIO,
    subnets::SubnetworkId,
    Hash,
};
use std::collections::HashMap;

/// Transaction builder for creating and signing transactions
pub struct TxBuilder {
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
    fee_rate: u64, // sompi per byte
}

impl TxBuilder {
    /// Create new transaction builder
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            fee_rate: 1, // default 1 sompi per byte
        }
    }

    /// Set fee rate
    pub fn fee_rate(mut self, rate: u64) -> Self {
        self.fee_rate = rate;
        self
    }

    /// Add input
    pub fn add_input(mut self, outpoint: TransactionOutpoint, script_sig: Vec<u8>) -> Self {
        let input = TransactionInput::new(outpoint, script_sig, 0, 0);
        self.inputs.push(input);
        self
    }

    /// Add output
    pub fn add_output(mut self, value: u64, script_pub_key: ScriptPublicKey) -> Self {
        let output = TransactionOutput::new(value, script_pub_key);
        self.outputs.push(output);
        self
    }

    /// Build transaction
    pub fn build(self, utxos: &HashMap<TransactionOutpoint, consensus_core::tx::UtxoEntry>) -> Result<Transaction, String> {
        if self.inputs.is_empty() {
            return Err("No inputs specified".to_string());
        }
        if self.outputs.is_empty() {
            return Err("No outputs specified".to_string());
        }

        // Calculate total input and output amounts
        let total_input: u128 = self.inputs.iter()
            .map(|input| utxos.get(&input.previous_outpoint).map_or(0, |utxo| utxo.amount as u128))
            .sum();
        let total_output: u128 = self.outputs.iter()
            .map(|o| o.value as u128)
            .sum();

        if total_output > total_input {
            return Err("Insufficient funds".to_string());
        }

        // Estimate transaction size and fee
        let estimated_size = self.estimate_size();
        let fee = estimated_size as u128 * self.fee_rate as u128;

        if total_output + fee > total_input {
            return Err("Insufficient funds for fee".to_string());
        }

        // Create transaction
        Ok(Transaction::new(
            1, // version
            self.inputs,
            self.outputs,
            0, // lock_time
            SubnetworkId::from(0), // subnetwork_id
            0, // gas
            vec![], // payload
        ))
    }

    /// Estimate transaction size in bytes
    fn estimate_size(&self) -> usize {
        // Rough estimation
        let input_size = self.inputs.len() * 150; // ~150 bytes per input
        let output_size = self.outputs.len() * 34; // ~34 bytes per output
        let overhead = 10; // version, lock_time, etc.

        overhead + input_size + output_size
    }

    /// Calculate minimum fee for transaction
    pub fn calculate_min_fee(&self) -> u64 {
        (self.estimate_size() as u64 * self.fee_rate).max(1)
    }

    /// Create transaction to send amount to address
    pub fn send_to_address(
        utxos: &HashMap<TransactionOutpoint, consensus_core::tx::UtxoEntry>,
        from_address: &str,
        to_address: &str,
        amount: u64,
        fee_rate: u64,
    ) -> Result<Self, String> {
        // Find spendable UTXOs for the from_address
        let mut available_utxos = Vec::new();
        let mut total_available = 0u128;

        for (outpoint, entry) in utxos {
            // In real implementation, check if UTXO belongs to from_address
            // For now, assume all UTXOs are spendable
            if entry.amount > 0 {
                available_utxos.push((outpoint.clone(), entry.clone()));
                total_available += entry.amount as u128;
            }
        }

        if total_available < amount as u128 {
            return Err("Insufficient balance".to_string());
        }

        // Select UTXOs (simplified - just take first one that covers)
        let mut selected_utxos = Vec::new();
        let mut selected_amount = 0u128;

        for (outpoint, entry) in available_utxos {
            selected_utxos.push((outpoint, entry.clone()));
            selected_amount += entry.amount as u128;
            if selected_amount >= amount as u128 {
                break;
            }
        }

        // Create transaction builder
        let mut builder = TxBuilder::new().fee_rate(fee_rate);

        // Add inputs
        for (outpoint, _) in &selected_utxos {
            builder = builder.add_input(outpoint.clone(), vec![]); // script_sig will be filled by signer
        }

        // Add output to recipient
        let to_script = crate::address::Address::to_script_pub_key(to_address)?;
        builder = builder.add_output(amount, to_script);

        // Add change output if necessary
        let estimated_fee = builder.calculate_min_fee() as u128;
        let change_amount = selected_amount - amount as u128 - estimated_fee;

        if change_amount > 0 {
            let change_script = crate::address::Address::to_script_pub_key(from_address)?;
            builder = builder.add_output(change_amount as u64, change_script);
        }

        Ok(builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_transaction_builder() {
        let builder = TxBuilder::new()
            .fee_rate(10)
            .add_output(1000, ScriptPublicKey::from_vec(0, vec![0x76, 0xa9, 0x14, 0x88, 0xac]));

        assert_eq!(builder.fee_rate, 10);
        assert_eq!(builder.outputs.len(), 1);
    }

    #[test]
    fn test_estimate_size() {
        let builder = TxBuilder::new()
            .add_input(TransactionOutpoint::new(Hash::from_le_u64([1, 0, 0, 0]), 0), vec![])
            .add_output(1000, ScriptPublicKey::from_vec(0, vec![0x76, 0xa9, 0x14, 0x88, 0xac]));

        let size = builder.estimate_size();
        assert!(size > 0);
    }

    #[test]
    fn test_min_fee() {
        let builder = TxBuilder::new().fee_rate(5);
        let fee = builder.calculate_min_fee();
        assert!(fee >= 5); // At least 1 byte * 5 sompi/byte
    }
}
