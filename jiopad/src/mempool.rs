use consensus_core::tx::Transaction;
use consensus_core::Hash;
use rpc_core::{MempoolInterface, model::MempoolEntry};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Memory pool for pending transactions
pub struct Mempool {
    transactions: Arc<RwLock<HashMap<Hash, Transaction>>>,
    max_size: usize,
}

impl Mempool {
    /// Create a new mempool
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
            max_size: 50000, // Default max size
        }
    }

    /// Add a transaction to the mempool
    pub fn add_transaction(&self, tx: Transaction) -> Result<(), String> {
        let hash = tx.hash();
        let mut transactions = self.transactions.write().unwrap();

        // Check if already exists
        if transactions.contains_key(&hash) {
            return Err("Transaction already in mempool".to_string());
        }

        // Check size limit
        if transactions.len() >= self.max_size {
            return Err("Mempool is full".to_string());
        }

        // Basic validation (placeholder - would do full validation)
        if tx.inputs.is_empty() && !tx.is_coinbase() {
            return Err("Transaction has no inputs".to_string());
        }

        transactions.insert(hash, tx);
        Ok(())
    }

    /// Remove a transaction from the mempool
    pub fn remove_transaction(&self, hash: &Hash) -> Option<Transaction> {
        let mut transactions = self.transactions.write().unwrap();
        transactions.remove(hash)
    }

    /// Get a transaction by hash
    pub fn get_transaction(&self, hash: &Hash) -> Option<Transaction> {
        let transactions = self.transactions.read().unwrap();
        transactions.get(hash).cloned()
    }

    /// Get all transactions
    pub fn get_all_transactions(&self) -> Vec<Transaction> {
        let transactions = self.transactions.read().unwrap();
        transactions.values().cloned().collect()
    }

    /// Get mempool size
    pub fn size(&self) -> usize {
        let transactions = self.transactions.read().unwrap();
        transactions.len()
    }

    /// Clear the mempool
    pub fn clear(&self) {
        let mut transactions = self.transactions.write().unwrap();
        transactions.clear();
    }

    /// Check if transaction exists in mempool
    pub fn contains(&self, hash: &Hash) -> bool {
        let transactions = self.transactions.read().unwrap();
        transactions.contains_key(hash)
    }
}

/// Implement the MempoolInterface trait for Mempool
impl MempoolInterface for Mempool {
    fn add_transaction(&self, tx: Transaction) -> Result<(), String> {
        let hash = tx.hash();
        let mut transactions = self.transactions.write().unwrap();

        // Check if already exists
        if transactions.contains_key(&hash) {
            return Err("Transaction already in mempool".to_string());
        }

        // Check size limit
        if transactions.len() >= self.max_size {
            return Err("Mempool is full".to_string());
        }

        // Basic validation (placeholder - would do full validation)
        if tx.inputs.is_empty() && !tx.is_coinbase() {
            return Err("Transaction has no inputs".to_string());
        }

        transactions.insert(hash, tx);
        Ok(())
    }

    fn remove_transaction(&self, tx_id: &str) -> Result<(), String> {
        // Parse tx_id as hash (placeholder implementation)
        Err("Not implemented".to_string())
    }

    fn size(&self) -> usize {
        let transactions = self.transactions.read().unwrap();
        transactions.len()
    }

    fn get_all_transactions(&self) -> Vec<Transaction> {
        let transactions = self.transactions.read().unwrap();
        transactions.values().cloned().collect()
    }

    fn get_entries(&self) -> Vec<MempoolEntry> {
        let transactions = self.transactions.read().unwrap();
        transactions.values().map(|tx| {
            MempoolEntry {
                fee: 0, // TODO: Calculate actual fee
                transaction: tx.clone(),
                is_orphan: false,
            }
        }).collect()
    }
}
