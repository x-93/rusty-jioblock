use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::constants::COINBASE_MATURITY;
use crate::tx::{PopulatedTransaction, Transaction, TransactionOutpoint, UtxoEntry};
use crate::errors::ConsensusError;
use crate::utxo::UtxoDiff;

/// A simple in-memory UTXO collection.
/// This is intentionally minimal and can be replaced by a DB-backed implementation later.
#[derive(Debug, Default, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct UtxoCollection {
    utxos: HashMap<TransactionOutpoint, UtxoEntry>,
}

impl UtxoCollection {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self { utxos: HashMap::new() }
    }

    /// Returns true if the outpoint exists in the set
    pub fn contains(&self, outpoint: &TransactionOutpoint) -> bool {
        self.utxos.contains_key(outpoint)
    }

    /// Get a reference to a UTXO entry
    pub fn get(&self, outpoint: &TransactionOutpoint) -> Option<&UtxoEntry> {
        self.utxos.get(outpoint)
    }

    /// Insert a new UTXO entry (overwrites if exists)
    pub fn insert(&mut self, outpoint: TransactionOutpoint, entry: UtxoEntry) {
        self.utxos.insert(outpoint, entry);
    }

    /// Remove and return an entry
    pub fn remove(&mut self, outpoint: &TransactionOutpoint) -> Option<UtxoEntry> {
        self.utxos.remove(outpoint)
    }

    /// Returns number of UTXOs
    pub fn len(&self) -> usize {
        self.utxos.len()
    }

    /// Returns true if empty
    pub fn is_empty(&self) -> bool {
        self.utxos.is_empty()
    }

    /// Returns total supply implied by UTXOs (sum of all utxo amounts)
    pub fn total_supply(&self) -> u128 {
        self.utxos.values().map(|e| e.amount as u128).sum()
    }

    /// Returns total supply implied by coinbase UTXOs (sum of coinbase utxo amounts)
    pub fn total_coinbase_supply(&self) -> u128 {
        self.utxos.values().filter(|e| e.is_coinbase).map(|e| e.amount as u128).sum()
    }

    /// Checks whether an outpoint is spendable under the provided `current_daa_score`.
    /// For normal outputs this is true. For coinbase outputs, this checks the `COINBASE_MATURITY`.
    pub fn is_spendable(&self, outpoint: &TransactionOutpoint, current_daa_score: u64) -> Result<bool, ConsensusError> {
        match self.get(outpoint) {
            Some(entry) => {
                if entry.is_coinbase {
                    Ok(current_daa_score >= entry.block_daa_score.saturating_add(COINBASE_MATURITY))
                } else {
                    Ok(true)
                }
            }
            None => Err(ConsensusError::InvalidUtxoReference),
        }
    }

    /// Populate a `PopulatedTransaction` using current UTXO set. For non-coinbase txs this will
    /// return a `PopulatedTransaction` containing the referenced UTXO entries. Coinbase txs are
    /// expected to be handled separately (they don't reference existing UTXOs).
    pub fn populate_transaction<'a>(&'a self, tx: &'a Transaction) -> Result<PopulatedTransaction<'a>, ConsensusError> {
        if tx.is_coinbase() {
            return Err(ConsensusError::InvalidTransaction);
        }

        let mut entries = Vec::with_capacity(tx.inputs.len());
        for input in &tx.inputs {
            let outpoint = &input.previous_outpoint;
            match self.get(outpoint) {
                Some(entry) => entries.push(entry.clone()),
                None => return Err(ConsensusError::InvalidUtxoReference),
            }
        }

        Ok(PopulatedTransaction::new(tx, entries))
    }

    /// Applies a single transaction to the UTXO set.
    /// - `current_daa_score` is used to enforce coinbase maturity for spent inputs.
    /// - `block_daa_score` is stored on created outputs.
    /// Returns a `UtxoDiff` suitable for undoing the operation.
    pub fn apply_transaction(&mut self, tx: &Transaction, current_daa_score: u64, block_daa_score: u64) -> Result<UtxoDiff, ConsensusError> {
        let mut diff = UtxoDiff::new();

        // Remove spent inputs (non-coinbase)
        if !tx.is_coinbase() {
            // check for duplicate inputs
            let mut seen = std::collections::HashSet::new();
            for input in &tx.inputs {
                let outpoint = input.previous_outpoint;
                if !seen.insert(outpoint) {
                    return Err(ConsensusError::DoubleSpend);
                }

                // ensure exists and spendable
                let entry = self.remove(&outpoint).ok_or(ConsensusError::InvalidUtxoReference)?;
                // if coinbase ensure maturity
                if entry.is_coinbase {
                    if current_daa_score < entry.block_daa_score.saturating_add(COINBASE_MATURITY) {
                        // restore removed entry before erroring
                        self.insert(outpoint, entry);
                        return Err(ConsensusError::InvalidTransaction);
                    }
                }

                diff.spent.push((outpoint, entry));
            }
        }

        // Add outputs as new UTXOs
        for (index, output) in tx.outputs.iter().enumerate() {
            let outpoint = TransactionOutpoint::new(tx.id(), index as u32);
            let entry = UtxoEntry::new(output.value, output.script_public_key.clone(), block_daa_score, tx.is_coinbase());
            self.insert(outpoint, entry);
            diff.created.push(outpoint);
        }

        Ok(diff)
    }

    /// Apply all transactions of a block in order. Returns the vector of diffs, one per transaction.
    /// `current_daa_score` is used to check maturity for spends; `block_daa_score` is the daa score
    /// assigned to outputs created by these transactions.
    pub fn apply_block(&mut self, txs: &[Transaction], current_daa_score: u64, block_daa_score: u64) -> Result<Vec<UtxoDiff>, ConsensusError> {
        let mut diffs = Vec::with_capacity(txs.len());
        for tx in txs {
            let diff = self.apply_transaction(tx, current_daa_score, block_daa_score)?;
            diffs.push(diff);
        }
        Ok(diffs)
    }

    /// Rollback a previously applied `UtxoDiff` (undo a transaction): re-inserts spent entries and
    /// removes created outpoints. The operation is best-effort and will return an error if it cannot
    /// restore the state (which should not happen when undoing a previously applied diff).
    pub fn rollback(&mut self, diff: UtxoDiff) -> Result<(), ConsensusError> {
        // remove created
        for outpoint in diff.created {
            self.remove(&outpoint);
        }

        // restore spent entries
        for (outpoint, entry) in diff.spent {
            self.insert(outpoint, entry);
        }

        Ok(())
    }
}

impl crate::utxo::UtxoInquirer for UtxoCollection {
    fn contains(&self, outpoint: &TransactionOutpoint) -> bool {
        self.contains(outpoint)
    }

    fn get(&self, outpoint: &TransactionOutpoint) -> Option<&crate::tx::UtxoEntry> {
        self.get(outpoint)
    }

    fn is_spendable(&self, outpoint: &TransactionOutpoint, current_daa_score: u64) -> Result<bool, ConsensusError> {
        self.is_spendable(outpoint, current_daa_score)
    }

    fn populate_transaction<'a>(&'a self, tx: &'a Transaction) -> Result<PopulatedTransaction<'a>, ConsensusError> {
        self.populate_transaction(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::{Transaction, TransactionInput, TransactionOutput, TransactionOutpoint, ScriptPublicKey};
    use crate::subnets::SUBNETWORK_ID_COINBASE;

    #[test]
    fn test_basic_apply_and_rollback() {
        let mut set = UtxoCollection::new();

        // create a funding tx (coinbase style)
        let funding_tx = Transaction::new(1, vec![], vec![TransactionOutput::new(50, ScriptPublicKey::default())], 0, SUBNETWORK_ID_COINBASE, 0, vec![]);
        // manually insert its outputs (simulate coinbase created at daa_score 0)
        let outpoint = TransactionOutpoint::new(funding_tx.id(), 0);
        let entry = UtxoEntry::new(50, ScriptPublicKey::default(), 0, true);
        set.insert(outpoint, entry.clone());

        // create spending tx
        let input = TransactionInput::new(outpoint, vec![], 0, 0);
        let spending_tx = Transaction::new(1, vec![input], vec![TransactionOutput::new(50, ScriptPublicKey::default())], 0, 0.into(), 0, vec![]);

        // cannot spend coinbase before maturity
        let res = set.apply_transaction(&spending_tx, 0, 1);
        assert!(res.is_err()); // Coinbase maturity check enforces that coinbase cannot be spent before maturity

        // apply with sufficient daa score
        let res = set.apply_transaction(&spending_tx, 100, 1).unwrap();
        assert_eq!(res.spent.len(), 1);
        assert_eq!(res.created.len(), 1);

        // rollback
        set.rollback(res).unwrap();
        assert!(set.contains(&outpoint));
        // Verify the entry was properly restored
        assert_eq!(set.get(&outpoint).unwrap().amount, 50);
    }
}
