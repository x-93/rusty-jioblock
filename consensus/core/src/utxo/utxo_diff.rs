use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::tx::{TransactionOutpoint, UtxoEntry};

/// Represents the changes caused by applying a transaction to the UTXO set.
/// `spent` contains the previous UTXO entries that were consumed (for undo).
/// `created` lists the outpoints that were created by the transaction.
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct UtxoDiff {
    pub spent: Vec<(TransactionOutpoint, UtxoEntry)>,
    pub created: Vec<TransactionOutpoint>,
}

impl UtxoDiff {
    pub fn new() -> Self {
        Self { spent: Vec::new(), created: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::{TransactionOutpoint, ScriptPublicKey};

    #[test]
    fn diff_roundtrip() {
        let outpoint = TransactionOutpoint::new(Default::default(), 0);
        let entry = UtxoEntry::new(10, ScriptPublicKey::default(), 0, false);
        let mut d = UtxoDiff::new();
        d.spent.push((outpoint, entry));
        d.created.push(TransactionOutpoint::new(Default::default(), 1));
    let ser = d.try_to_vec().unwrap();
    let de: UtxoDiff = UtxoDiff::try_from_slice(&ser).unwrap();
        assert_eq!(d, de);
    }
}
