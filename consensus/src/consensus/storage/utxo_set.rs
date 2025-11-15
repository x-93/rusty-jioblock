//! UTXO set management for consensus
//!
//! This module provides UTXO set management including adding, removing,
//! and querying UTXOs.

use consensus_core::block::Block;
use consensus_core::tx::{
    TransactionOutpoint, UtxoEntry,
};
use consensus_core::errors::ConsensusError;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use database::stores::UtxoStore as DbUtxoStore;
use std::sync::Arc as StdArc;

/// UTXO set for consensus storage
pub struct UtxoSet {
    utxos: Arc<RwLock<HashMap<TransactionOutpoint, UtxoEntry>>>,
    current_daa_score: Arc<RwLock<u64>>,
    db_store: Option<StdArc<DbUtxoStore>>,
}

impl UtxoSet {
    /// Create a new UTXO set
    pub fn new() -> Self {
        Self {
            utxos: Arc::new(RwLock::new(HashMap::new())),
            current_daa_score: Arc::new(RwLock::new(0)),
            db_store: None,
        }
    }

    /// Create a new UTXO set backed by a DB-backed UtxoStore
    pub fn new_with_db(db_store: StdArc<DbUtxoStore>) -> Self {
        Self {
            utxos: Arc::new(RwLock::new(HashMap::new())),
            current_daa_score: Arc::new(RwLock::new(0)),
            db_store: Some(db_store),
        }
    }

    /// Add a UTXO entry
    pub fn add_utxo(&self, outpoint: TransactionOutpoint, entry: UtxoEntry) -> Result<(), ConsensusError> {
        if let Some(db) = &self.db_store {
            db.put_utxo(&outpoint, &entry).map_err(|e| ConsensusError::DatabaseError(e.to_string()))?;
            return Ok(());
        }
        let mut utxos = self.utxos.write().unwrap();
        utxos.insert(outpoint, entry);
        Ok(())
    }

    /// Remove a UTXO entry
    pub fn remove_utxo(&self, outpoint: &TransactionOutpoint) -> Option<UtxoEntry> {
        if let Some(db) = &self.db_store {
            // For DB-backed store, try to fetch the entry first so we can
            // return the removed UTXO (callers expect Some on success).
            match db.get_utxo(outpoint) {
                Ok(opt) => {
                    if opt.is_some() {
                        if let Err(e) = db.delete_utxo(outpoint) {
                            eprintln!("DB delete_utxo error: {}", e);
                        }
                    }
                    return opt;
                }
                Err(e) => {
                    eprintln!("DB get_utxo error: {}", e);
                    return None;
                }
            }
        }
        let mut utxos = self.utxos.write().unwrap();
        utxos.remove(outpoint)
    }

    /// Get a UTXO entry
    pub fn get_utxo(&self, outpoint: &TransactionOutpoint) -> Option<UtxoEntry> {
        if let Some(db) = &self.db_store {
            match db.get_utxo(outpoint) {
                Ok(opt) => return opt,
                Err(e) => { eprintln!("DB get_utxo error: {}", e); return None; }
            }
        }
        let utxos = self.utxos.read().unwrap();
        utxos.get(outpoint).cloned()
    }

    /// Check if a UTXO exists
    pub fn contains(&self, outpoint: &TransactionOutpoint) -> bool {
        if let Some(db) = &self.db_store {
            match db.has_utxo(outpoint) {
                Ok(b) => return b,
                Err(e) => { eprintln!("DB has_utxo error: {}", e); return false; }
            }
        }
        let utxos = self.utxos.read().unwrap();
        utxos.contains_key(outpoint)
    }

    /// Apply a block to the UTXO set
    pub fn apply_block(&self, block: &Block, block_daa_score: u64) -> Result<(), ConsensusError> {
        // Update current daa score
        let mut current_daa_score = self.current_daa_score.write().unwrap();
        *current_daa_score = block_daa_score;

        // Process all transactions in the block
        for tx in block.transactions.iter() {
            // Remove inputs (spent UTXOs)
            if !tx.is_coinbase() {
                for input in &tx.inputs {
                    // If DB-backed, let remove_utxo attempt deletion; otherwise, operate on in-memory map
                    if self.remove_utxo(&input.previous_outpoint).is_none() {
                        return Err(ConsensusError::InvalidUtxoReference);
                    }
                }
            }

            // Add outputs (new UTXOs)
            for (output_index, output) in tx.outputs.iter().enumerate() {
                let outpoint = TransactionOutpoint::new(tx.id(), output_index as u32);
                let entry = UtxoEntry::new(
                    output.value,
                    output.script_public_key.clone(),
                    block_daa_score,
                    tx.is_coinbase(),
                );
                self.add_utxo(outpoint, entry)?;
            }
        }

        Ok(())
    }

    /// Revert a block from the UTXO set
    pub fn revert_block(&self, block: &Block) -> Result<(), ConsensusError> {
        // Process transactions in reverse order
        for tx in block.transactions.iter().rev() {
            // Remove outputs (revert new UTXOs)
            for (output_index, _) in tx.outputs.iter().enumerate() {
                let outpoint = TransactionOutpoint::new(tx.id(), output_index as u32);
                if self.remove_utxo(&outpoint).is_none() {
                    return Err(ConsensusError::InvalidUtxoReference);
                }
            }

            // Add inputs back (restore spent UTXOs)
            if !tx.is_coinbase() {
                // Note: We'd need to restore the original UTXO entries here
                // This is a simplified version - in practice, we'd need to store
                // the previous state or reconstruct from the block's acceptance data
            }
        }

        Ok(())
    }

    /// Get total supply from UTXO set
    pub fn total_supply(&self) -> u128 {
        if let Some(db) = &self.db_store {
            match db.sum_amounts() {
                Ok(total) => return total,
                Err(e) => eprintln!("DB sum_amounts error: {}", e),
            }
        }
        let utxos = self.utxos.read().unwrap();
        utxos.values().map(|e| e.amount as u128).sum()
    }

    /// Get number of UTXOs
    pub fn len(&self) -> usize {
        if let Some(db) = &self.db_store {
            match db.count() {
                Ok(c) => return c,
                Err(e) => eprintln!("DB count error: {}", e),
            }
        }
        let utxos = self.utxos.read().unwrap();
        utxos.len()
    }

    /// Check if UTXO set is empty
    pub fn is_empty(&self) -> bool {
        if let Some(db) = &self.db_store {
            match db.count() {
                Ok(c) => return c == 0,
                Err(e) => { eprintln!("DB count error: {}", e); return true; }
            }
        }
        let utxos = self.utxos.read().unwrap();
        utxos.is_empty()
    }

    /// Get current DAA score
    pub fn current_daa_score(&self) -> u64 {
        let current_daa_score = self.current_daa_score.read().unwrap();
        *current_daa_score
    }

    /// Create a snapshot of all UTXOs as a HashMap for validation
    /// Note: This clones all UTXOs, so it should be used sparingly
    /// This allows UtxoSet to be used with validators that require UtxoView trait
    pub fn snapshot(&self) -> std::collections::HashMap<TransactionOutpoint, UtxoEntry> {
        // If DB-backed, we cannot cheaply clone all UTXOs; fall back to in-memory snapshot
        let utxos = self.utxos.read().unwrap();
        utxos.clone()
    }
}

impl Default for UtxoSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::header::Header;
    use consensus_core::{Hash, ZERO_HASH, BlueWorkType};
    use consensus_core::tx::{Transaction, TransactionOutput, ScriptPublicKey};
    use consensus_core::subnets::SUBNETWORK_ID_COINBASE;

    fn create_test_block(txs: Vec<Transaction>) -> Block {
        let header = Header::new_finalized(
            1,
            vec![],
            ZERO_HASH,
            ZERO_HASH,
            ZERO_HASH,
            1000,
            0x1f00ffff,
            0,
            0,
            BlueWorkType::from(0u64),
            0,
            ZERO_HASH,
        );
        Block::new(header, txs)
    }

    #[test]
    fn test_add_and_get_utxo() {
        let utxo_set = UtxoSet::new();
        let outpoint = TransactionOutpoint::new(Hash::from_le_u64([1, 0, 0, 0]), 0);
        let entry = UtxoEntry::new(
            5000,
            ScriptPublicKey::from_vec(0, Vec::new()),
            100,
            false,
        );

        utxo_set.add_utxo(outpoint, entry.clone()).unwrap();
        let retrieved = utxo_set.get_utxo(&outpoint).unwrap();
        assert_eq!(retrieved.amount, entry.amount);
    }

    #[test]
    fn test_apply_block() {
        let utxo_set = UtxoSet::new();
        let coinbase = Transaction::new(
            1,
            Vec::new(),
            vec![TransactionOutput::new(
                5000000000,
                ScriptPublicKey::from_vec(0, Vec::new()),
            )],
            0,
            SUBNETWORK_ID_COINBASE,
            0,
            Vec::new(),
        );
        let block = create_test_block(vec![coinbase]);

        utxo_set.apply_block(&block, 100).unwrap();
        assert_eq!(utxo_set.len(), 1);
    }
}

