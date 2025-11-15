use crate::tx::{TransactionOutpoint, Transaction, PopulatedTransaction};
use crate::errors::ConsensusError;

/// Trait for querying UTXO related data needed for validation and signing.
pub trait UtxoInquirer {
    /// Returns true if the outpoint exists
    fn contains(&self, outpoint: &TransactionOutpoint) -> bool;

    /// Returns the UTXO entry if present
    fn get(&self, outpoint: &TransactionOutpoint) -> Option<&crate::tx::UtxoEntry>;

    /// Returns whether an outpoint is spendable at the provided DAA score
    fn is_spendable(&self, outpoint: &TransactionOutpoint, current_daa_score: u64) -> Result<bool, ConsensusError>;

    /// Populate a transaction's UTXO entries. Coinbase txs are not populated.
    fn populate_transaction<'a>(&'a self, tx: &'a Transaction) -> Result<PopulatedTransaction<'a>, ConsensusError>;
}
