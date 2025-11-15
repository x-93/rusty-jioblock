use crate::{Database, DbResult};
use crate::cache::WriteThroughCache;
use consensus_core::Hash;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostdagData {
    pub blue_score: u64,
    pub blue_work: u128,
    pub selected_parent: Hash,
    pub merge_set_size: u64,
    pub blues_anticone_sizes: HashMap<Hash, u32>,
    pub height: u64,
}

pub struct GhostdagStore {
    db: Arc<Database>,
    cache: WriteThroughCache<Hash, GhostdagData>,
}

impl GhostdagStore {
    pub fn new(db: Arc<Database>, cache_size: usize) -> Self {
        Self { db, cache: WriteThroughCache::new(cache_size) }
    }

    pub fn put_ghostdag_data(&self, hash: &Hash, data: &GhostdagData) -> DbResult<()> {
        let serialized = bincode::serialize(data)?;
        self.db.put(crate::db::CF_GHOSTDAG, hash.as_bytes(), &serialized)?;
        self.cache.insert(*hash, data.clone());
        Ok(())
    }

    pub fn get_ghostdag_data(&self, hash: &Hash) -> DbResult<Option<GhostdagData>> {
        if let Some(d) = self.cache.get(hash) { return Ok(Some(d)); }
        if let Some(bytes) = self.db.get(crate::db::CF_GHOSTDAG, hash.as_bytes())? {
            let data: GhostdagData = bincode::deserialize(&bytes)?;
            self.cache.insert(*hash, data.clone());
            Ok(Some(data))
        } else { Ok(None) }
    }

    pub fn has_ghostdag_data(&self, hash: &Hash) -> DbResult<bool> {
        if self.cache.get(hash).is_some() { return Ok(true); }
        self.db.exists(crate::db::CF_GHOSTDAG, hash.as_bytes())
    }

    pub fn get_blue_score(&self, hash: &Hash) -> DbResult<Option<u64>> {
        Ok(self.get_ghostdag_data(hash)?.map(|d| d.blue_score))
    }
}
