use crate::{Database, DbResult};
use crate::cache::WriteThroughCache;
use consensus_core::tx::{TransactionOutpoint, UtxoEntry};
use std::sync::Arc;

pub struct UtxoStore {
    db: Arc<Database>,
    cache: WriteThroughCache<TransactionOutpoint, UtxoEntry>,
}

impl UtxoStore {
    pub fn new(db: Arc<Database>, cache_size: usize) -> Self {
        Self { db, cache: WriteThroughCache::new(cache_size) }
    }

    pub fn put_utxo(&self, outpoint: &TransactionOutpoint, entry: &UtxoEntry) -> DbResult<()> {
        let key = Self::outpoint_to_key(outpoint);
        let serialized = bincode::serialize(entry)?;
        self.db.put(crate::db::CF_UTXOS, &key, &serialized)?;
        self.cache.insert(outpoint.clone(), entry.clone());
        Ok(())
    }

    pub fn get_utxo(&self, outpoint: &TransactionOutpoint) -> DbResult<Option<UtxoEntry>> {
        if let Some(e) = self.cache.get(outpoint) { return Ok(Some(e)); }
        let key = Self::outpoint_to_key(outpoint);
        if let Some(data) = self.db.get(crate::db::CF_UTXOS, &key)? {
            let entry: UtxoEntry = bincode::deserialize(&data)?;
            self.cache.insert(outpoint.clone(), entry.clone());
            Ok(Some(entry))
        } else { Ok(None) }
    }

    pub fn delete_utxo(&self, outpoint: &TransactionOutpoint) -> DbResult<()> {
        let key = Self::outpoint_to_key(outpoint);
        self.db.delete(crate::db::CF_UTXOS, &key)?;
        self.cache.remove(outpoint);
        Ok(())
    }

    pub fn has_utxo(&self, outpoint: &TransactionOutpoint) -> DbResult<bool> {
        if self.cache.get(outpoint).is_some() { return Ok(true); }
        let key = Self::outpoint_to_key(outpoint);
        self.db.exists(crate::db::CF_UTXOS, &key)
    }

    pub fn count(&self) -> DbResult<usize> {
        let mut count = 0usize;
        let iter = self.db.iterator(crate::db::CF_UTXOS, rocksdb::IteratorMode::Start)?;
        for _ in iter { count += 1; }
        Ok(count)
    }

    /// Sum amounts of all UTXO entries in the DB (returns total as u128)
    pub fn sum_amounts(&self) -> DbResult<u128> {
        let mut total: u128 = 0;
        let iter = self.db.iterator(crate::db::CF_UTXOS, rocksdb::IteratorMode::Start)?;
        for item in iter {
            let (_k, value) = item?;
            let entry: UtxoEntry = bincode::deserialize(&value)?;
            total = total.saturating_add(entry.amount as u128);
        }
        Ok(total)
    }

    fn outpoint_to_key(outpoint: &TransactionOutpoint) -> Vec<u8> {
        let mut key = outpoint.transaction_id.as_bytes().to_vec();
        key.extend_from_slice(&outpoint.index.to_le_bytes());
        key
    }
}
