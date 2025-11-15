use crate::Hash;
use crate::tx::Transaction;
use sha2::{Digest, Sha256};
use borsh::BorshSerialize;

pub fn calc_transaction_hash(tx: &Transaction) -> Hash {
    // Serialize transaction using Borsh then hash the bytes
    let ser = tx.try_to_vec().expect("transaction serialization");
    let result = Sha256::digest(&ser);
    Hash::try_from_slice(&result).expect("SHA256 output has correct length")
}