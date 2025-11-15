use crate::tx::TransactionOutpoint;
use crate::utxo::UtxoCollection;

/// Read-only view on an existing UTXO collection
pub struct UtxoView<'a> {
    inner: &'a UtxoCollection,
}

impl<'a> UtxoView<'a> {
    pub fn new(inner: &'a UtxoCollection) -> Self {
        Self { inner }
    }

    pub fn contains(&self, outpoint: &TransactionOutpoint) -> bool {
        self.inner.contains(outpoint)
    }

    pub fn get(&self, outpoint: &TransactionOutpoint) -> Option<&crate::tx::UtxoEntry> {
        self.inner.get(outpoint)
    }

    pub fn total_supply(&self) -> u128 {
        self.inner.total_supply()
    }
}
