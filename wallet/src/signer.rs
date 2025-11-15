use consensus_core::tx::{Transaction, TransactionInput};
use consensus_core::subnets::SubnetworkId;
use secp256k1::{Secp256k1, SecretKey, Message};
use sha2::{Sha256, Digest};
use crate::keys::Keys;

/// Transaction signer for creating digital signatures
pub struct Signer {
    keys: Keys,
    secp: Secp256k1<secp256k1::All>,
}

impl Signer {
    /// Create new signer
    pub fn new(keys: Keys) -> Self {
        Self {
            keys,
            secp: Secp256k1::new(),
        }
    }

    /// Sign transaction input
    pub fn sign_input(
        &self,
        tx: &Transaction,
        input_index: usize,
        secret_key: &SecretKey,
        sighash_type: u32,
    ) -> Result<Vec<u8>, String> {
        // Create sighash (simplified - real implementation needs proper sighash)
        let sighash = self.create_sighash(tx, input_index)?;

        // Sign the sighash
        let message = Message::from_slice(&sighash)
            .map_err(|e| format!("Invalid message: {}", e))?;

        let signature = self.secp.sign_ecdsa(&message, secret_key);

        // Serialize signature with sighash type
        let mut sig_bytes = signature.serialize_der().to_vec();
        sig_bytes.push(sighash_type as u8);

        Ok(sig_bytes)
    }

    /// Sign complete transaction
    pub fn sign_transaction(
        &self,
        mut tx: Transaction,
        secret_keys: &[SecretKey],
    ) -> Result<Transaction, String> {
        if tx.inputs.len() != secret_keys.len() {
            return Err("Number of inputs must match number of secret keys".to_string());
        }

        // Sign each input
        for (i, secret_key) in secret_keys.iter().enumerate() {
            let signature = self.sign_input(&tx, i, secret_key, 0x01)?; // SIGHASH_ALL

            // Create script_sig (simplified P2PKH)
            let public_key = self.keys.public_key(secret_key);
            let mut script_sig = vec![];
            script_sig.push(signature.len() as u8);
            script_sig.extend_from_slice(&signature);
            script_sig.push(public_key.serialize().len() as u8);
            script_sig.extend_from_slice(&public_key.serialize());

            tx.inputs[i].signature_script = script_sig;
        }

        Ok(tx)
    }

    /// Create sighash for transaction input (simplified)
    fn create_sighash(&self, tx: &Transaction, input_index: usize) -> Result<[u8; 32], String> {
        let mut hasher = Sha256::new();

        // Add version
        hasher.update(&tx.version.to_le_bytes());

        // Add input count
        hasher.update(&[tx.inputs.len() as u8]);

        // Add inputs (simplified - only the signing input)
        for (i, input) in tx.inputs.iter().enumerate() {
            if i == input_index {
                // For the input being signed, use the script_pub_key instead of script_sig
                hasher.update(&input.previous_outpoint.transaction_id.as_bytes());
                hasher.update(&input.previous_outpoint.index.to_le_bytes());
                // In real implementation, we'd use the script_pub_key from the UTXO
                hasher.update(&[0u8]); // Placeholder empty script
                hasher.update(&input.sequence.to_le_bytes());
            } else {
                // For other inputs, use empty script_sig
                hasher.update(&input.previous_outpoint.transaction_id.as_bytes());
                hasher.update(&input.previous_outpoint.index.to_le_bytes());
                hasher.update(&[0u8]); // Empty script_sig
                hasher.update(&input.sequence.to_le_bytes());
            }
        }

        // Add output count
        hasher.update(&[tx.outputs.len() as u8]);

        // Add outputs
        for output in &tx.outputs {
            hasher.update(&output.value.to_le_bytes());
            hasher.update(&[output.script_public_key.script().len() as u8]);
            hasher.update(output.script_public_key.script());
        }

        // Add lock_time
        hasher.update(&tx.lock_time.to_le_bytes());

        // Add sighash type (SIGHASH_ALL = 1)
        hasher.update(&[0x01]);

        let hash = hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash);
        Ok(result)
    }

    /// Verify signature
    pub fn verify_signature(
        &self,
        tx: &Transaction,
        input_index: usize,
        public_key: &secp256k1::PublicKey,
    ) -> Result<bool, String> {
        let sighash = self.create_sighash(tx, input_index)?;
        let message = Message::from_slice(&sighash)
            .map_err(|e| format!("Invalid message: {}", e))?;

        // Extract signature from script_sig (simplified)
        let script_sig = &tx.inputs[input_index].signature_script;
        if script_sig.len() < 2 {
            return Ok(false);
        }

        let sig_len = script_sig[0] as usize;
        if script_sig.len() < 1 + sig_len + 1 {
            return Ok(false);
        }

        let signature_bytes = &script_sig[1..1 + sig_len];
        // Remove the sighash type byte from the end
        let signature_bytes = &signature_bytes[..signature_bytes.len() - 1];
        let signature = secp256k1::ecdsa::Signature::from_der(signature_bytes)
            .map_err(|e| format!("Invalid signature: {}", e))?;

        Ok(self.secp.verify_ecdsa(&message, &signature, public_key).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::tx::{TransactionOutput, ScriptPublicKey, TransactionOutpoint};
    use consensus_core::Hash;

    #[test]
    fn test_signer_creation() {
        let keys = Keys::new();
        let signer = Signer::new(keys);
        // Just verify it creates without error
    }

    #[test]
    fn test_sign_and_verify() {
        let keys = Keys::new();
        let signer = Signer::new(keys.clone());

        // Create a simple transaction
        let mut tx = Transaction::new(
            1,
            vec![TransactionInput::new(
                TransactionOutpoint::new(Hash::from_le_u64([1, 0, 0, 0]), 0),
                vec![],
                0,
                0,
            )],
            vec![TransactionOutput::new(
                1000,
                ScriptPublicKey::from_vec(0, vec![0x76, 0xa9, 0x14, 0x88, 0xac]),
            )],
            0,
            SubnetworkId::from(0),
            0,
            vec![],
        );

        // Sign the transaction
        let (secret_key, public_key) = keys.generate_address().unwrap();
        let signed_tx = signer.sign_transaction(tx, &[secret_key]).unwrap();

        // Verify the signature
        let is_valid = signer.verify_signature(&signed_tx, 0, &public_key).unwrap();
        assert!(is_valid);
    }
}
