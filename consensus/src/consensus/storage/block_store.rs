//! Block store for consensus
//!
//! This module provides block storage and retrieval functionality.

use consensus_core::block::Block;
use consensus_core::header::Header;
use consensus_core::Hash;
use consensus_core::errors::ConsensusError;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use database::stores::BlockStore as DbBlockStore;
use database::stores::HeaderStore as DbHeaderStore;
use std::sync::Arc as StdArc;

/// Block store for consensus storage
pub struct BlockStore {
    blocks: Arc<RwLock<HashMap<Hash, Block>>>,
    headers: Arc<RwLock<HashMap<Hash, Header>>>,
    db_store: Option<StdArc<DbBlockStore>>,
    db_header_store: Option<StdArc<DbHeaderStore>>,
}

impl BlockStore {
    /// Create a new block store
    pub fn new() -> Self {
        Self {
            blocks: Arc::new(RwLock::new(HashMap::new())),
            headers: Arc::new(RwLock::new(HashMap::new())),
            db_store: None,
            db_header_store: None,
        }
    }

    /// Create a new block store backed by a database store
    pub fn new_with_db(db_store: StdArc<DbBlockStore>, header_store: Option<StdArc<DbHeaderStore>>) -> Self {
        Self {
            blocks: Arc::new(RwLock::new(HashMap::new())),
            headers: Arc::new(RwLock::new(HashMap::new())),
            db_store: Some(db_store),
            db_header_store: header_store,
        }
    }

    /// Check if this store is backed by a database
    pub fn has_db(&self) -> bool {
        self.db_store.is_some()
    }

    /// Store a block
    pub fn store_block(&self, block: Block) -> Result<(), ConsensusError> {
        let hash = block.header.hash;
        if let Some(db) = &self.db_store {
            db.put_block(&block).map_err(|e| ConsensusError::DatabaseError(e.to_string()))?;
            return Ok(());
        }
        let mut blocks = self.blocks.write().unwrap();
        blocks.insert(hash, block);
        Ok(())
    }

    /// Store a header only
    pub fn store_header(&self, header: Header) -> Result<(), ConsensusError> {
        let hash = header.hash;
        if let Some(hdb) = &self.db_header_store {
            hdb.put_header(&header).map_err(|e| ConsensusError::DatabaseError(e.to_string()))?;
            return Ok(());
        }
        let mut headers = self.headers.write().unwrap();
        headers.insert(hash, header);
        Ok(())
    }

    /// Get a block by hash
    pub fn get_block(&self, hash: &Hash) -> Option<Block> {
        if let Some(db) = &self.db_store {
            match db.get_block(hash) {
                Ok(opt) => return opt,
                Err(e) => {
                    // On DB error return None for now and log
                    eprintln!("DB get_block error: {}", e);
                    return None;
                }
            }
        }
        let blocks = self.blocks.read().unwrap();
        blocks.get(hash).cloned()
    }

    /// Get a header by hash
    pub fn get_header(&self, hash: &Hash) -> Option<Header> {
        if let Some(hdb) = &self.db_header_store {
            match hdb.get_header(hash) {
                Ok(opt) => return opt,
                Err(e) => { eprintln!("DB get_header error: {}", e); return None; }
            }
        }
        let headers = self.headers.read().unwrap();
        headers.get(hash).cloned()
    }

    /// Check if a block exists
    pub fn has_block(&self, hash: &Hash) -> bool {
        if let Some(db) = &self.db_store {
            match db.has_block(hash) {
                Ok(b) => return b,
                Err(e) => {
                    eprintln!("DB has_block error: {}", e);
                    return false;
                }
            }
        }
        let blocks = self.blocks.read().unwrap();
        blocks.contains_key(hash)
    }

    /// Check if a header exists
    pub fn has_header(&self, hash: &Hash) -> bool {
        if let Some(hdb) = &self.db_header_store {
            match hdb.has_header(hash) {
                Ok(b) => return b,
                Err(e) => { eprintln!("DB has_header error: {}", e); return false; }
            }
        }
        let headers = self.headers.read().unwrap();
        headers.contains_key(hash)
    }

    /// Remove a block
    pub fn remove_block(&self, hash: &Hash) -> Option<Block> {
        if let Some(db) = &self.db_store {
            if let Err(e) = db.delete_block(hash) {
                eprintln!("DB delete_block error: {}", e);
            }
            return None;
        }
        let mut blocks = self.blocks.write().unwrap();
        blocks.remove(hash)
    }

    /// Remove a header
    pub fn remove_header(&self, hash: &Hash) -> Option<Header> {
        if let Some(hdb) = &self.db_header_store {
            if let Err(e) = hdb.delete_header(hash) {
                eprintln!("DB delete_header error: {}", e);
            }
            return None;
        }
        let mut headers = self.headers.write().unwrap();
        headers.remove(hash)
    }

    /// Get number of stored blocks
    pub fn block_count(&self) -> usize {
        if let Some(db) = &self.db_store {
            match db.count() {
                Ok(c) => return c,
                Err(e) => eprintln!("DB count error: {}", e),
            }
        }
        let blocks = self.blocks.read().unwrap();
        blocks.len()
    }

    /// Get number of stored headers
    pub fn header_count(&self) -> usize {
        if let Some(hdb) = &self.db_header_store {
            match hdb.count() {
                Ok(c) => return c,
                Err(e) => eprintln!("DB header count error: {}", e),
            }
        }
        let headers = self.headers.read().unwrap();
        headers.len()
    }
}

impl Default for BlockStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::{ZERO_HASH, BlueWorkType};

    fn create_test_block() -> Block {
        let header = Header::new_finalized(
            1,
            vec![],
            ZERO_HASH,
            ZERO_HASH,
            ZERO_HASH,
            1000,
            0x1f00ffff,
            0,
            0,
            BlueWorkType::from(0u64),
            0,
            ZERO_HASH,
        );
        Block::new(header, Vec::new())
    }

    #[test]
    fn test_store_and_get_block() {
        let store = BlockStore::new();
        let block = create_test_block();
        let hash = block.header.hash;

        store.store_block(block.clone()).unwrap();
        let retrieved = store.get_block(&hash).unwrap();
        assert_eq!(retrieved.header.hash, hash);
    }

    #[test]
    fn test_store_and_get_header() {
        let store = BlockStore::new();
        let block = create_test_block();
        let header = block.header.clone();
        let hash = header.hash;

        store.store_header(header.clone()).unwrap();
        let retrieved = store.get_header(&hash).unwrap();
        assert_eq!(retrieved.hash, hash);
    }

    #[test]
    fn test_has_block() {
        let store = BlockStore::new();
        let block = create_test_block();
        let hash = block.header.hash;

        assert!(!store.has_block(&hash));
        store.store_block(block).unwrap();
        assert!(store.has_block(&hash));
    }
}

