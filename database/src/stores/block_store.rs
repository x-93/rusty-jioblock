use crate::{Database, DbResult};
use crate::cache::WriteThroughCache;
use consensus_core::block::Block;
use consensus_core::Hash;
use std::sync::Arc;

pub struct BlockStore {
    db: Arc<Database>,
    cache: WriteThroughCache<Hash, Block>,
}

impl BlockStore {
    pub fn new(db: Arc<Database>, cache_size: usize) -> Self {
        Self { db, cache: WriteThroughCache::new(cache_size) }
    }

    pub fn put_block(&self, block: &Block) -> DbResult<()> {
        let hash = block.header.hash;
        let serialized = bincode::serialize(block)?;
        self.db.put(crate::db::CF_BLOCKS, hash.as_bytes(), &serialized)?;
        self.cache.insert(hash, block.clone());
        Ok(())
    }

    pub fn get_block(&self, hash: &Hash) -> DbResult<Option<Block>> {
        if let Some(b) = self.cache.get(hash) { return Ok(Some(b)); }
        if let Some(data) = self.db.get(crate::db::CF_BLOCKS, hash.as_bytes())? {
            let block: Block = bincode::deserialize(&data)?;
            self.cache.insert(*hash, block.clone());
            Ok(Some(block))
        } else { Ok(None) }
    }

    pub fn has_block(&self, hash: &Hash) -> DbResult<bool> {
        if self.cache.get(hash).is_some() { return Ok(true); }
        self.db.exists(crate::db::CF_BLOCKS, hash.as_bytes())
    }

    pub fn delete_block(&self, hash: &Hash) -> DbResult<()> {
        self.db.delete(crate::db::CF_BLOCKS, hash.as_bytes())?;
        self.cache.remove(hash);
        Ok(())
    }

    pub fn count(&self) -> DbResult<usize> {
        let mut count = 0usize;
        let iter = self.db.iterator(crate::db::CF_BLOCKS, rocksdb::IteratorMode::Start)?;
        for _ in iter { count += 1; }
        Ok(count)
    }
}
