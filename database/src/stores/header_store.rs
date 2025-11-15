use crate::{Database, DbResult};
use crate::cache::WriteThroughCache;
use consensus_core::header::Header as BlockHeader;
use consensus_core::Hash;
use std::sync::Arc;

pub struct HeaderStore {
    db: Arc<Database>,
    cache: WriteThroughCache<Hash, BlockHeader>,
}

impl HeaderStore {
    pub fn new(db: Arc<Database>, cache_size: usize) -> Self {
        Self { db, cache: WriteThroughCache::new(cache_size) }
    }

    pub fn put_header(&self, header: &BlockHeader) -> DbResult<()> {
        let hash = header.hash;
        let serialized = bincode::serialize(header)?;
        self.db.put(crate::db::CF_HEADERS, hash.as_bytes(), &serialized)?;
        self.cache.insert(hash, header.clone());
        Ok(())
    }

    pub fn get_header(&self, hash: &Hash) -> DbResult<Option<BlockHeader>> {
        if let Some(h) = self.cache.get(hash) { return Ok(Some(h)); }
        if let Some(data) = self.db.get(crate::db::CF_HEADERS, hash.as_bytes())? {
            let header: BlockHeader = bincode::deserialize(&data)?;
            self.cache.insert(*hash, header.clone());
            Ok(Some(header))
        } else { Ok(None) }
    }

    pub fn has_header(&self, hash: &Hash) -> DbResult<bool> {
        if self.cache.get(hash).is_some() { return Ok(true); }
        self.db.exists(crate::db::CF_HEADERS, hash.as_bytes())
    }

    pub fn delete_header(&self, hash: &Hash) -> DbResult<()> {
        self.db.delete(crate::db::CF_HEADERS, hash.as_bytes())?;
        self.cache.remove(hash);
        Ok(())
    }

    pub fn count(&self) -> DbResult<usize> {
        let mut count = 0usize;
        let iter = self.db.iterator(crate::db::CF_HEADERS, rocksdb::IteratorMode::Start)?;
        for item in iter {
            let (_k, _v) = item?;
            count += 1;
        }
        Ok(count)
    }
}
