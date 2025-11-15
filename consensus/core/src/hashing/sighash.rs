use crate::tx::Transaction;
use crate::Hash;

pub fn calc_transaction_sighash(_tx: &Transaction) -> Hash {
    // TODO: Implement real sighash calculation
    Hash::default()
}