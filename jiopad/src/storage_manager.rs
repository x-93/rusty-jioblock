use crate::config::StorageConfig;
use consensus::consensus::storage::{ConsensusStorage, BlockStore as ConsensusBlockStore, UtxoSet};
use std::sync::Arc;
use std::path::Path;
use database::Database;
use database::stores::BlockStore as DbBlockStore;
use std::sync::Arc as StdArc;

/// Storage manager that coordinates all storage components
pub struct StorageManager {
    config: StorageConfig,
    consensus_storage: Arc<ConsensusStorage>,
}

impl StorageManager {
    /// Create a new storage manager
    pub async fn new(config: &StorageConfig) -> Result<Self, String> {
        // Create data directory if it doesn't exist
        if !config.data_dir.exists() {
            std::fs::create_dir_all(&config.data_dir)
                .map_err(|e| format!("Failed to create data directory: {}", e))?;
        }

    // Initialize consensus storage
    // Open persistent database and create DB-backed stores
    let db = StdArc::new(Database::open(&config.data_dir).map_err(|e| format!("Failed to open DB: {}", e))?);

    // Convert configured cache size (bytes) into a reasonable number of cache entries.
    // The config value is in bytes (default 512MB). The in-memory cache expects a
    // capacity in number of entries, so divide by an estimated average entry size
    // (4KB) to avoid massive pre-allocations. Also clamp to a sensible minimum.
    let cache_entries = std::cmp::max(1024usize, config.db_cache_size / 4096);

    // Create DB-backed block/header/UTXO stores
    let db_block_store = StdArc::new(DbBlockStore::new(db.clone(), cache_entries));
    let db_header_store = StdArc::new(database::stores::HeaderStore::new(db.clone(), cache_entries));
    let db_utxo_store = StdArc::new(database::stores::UtxoStore::new(db.clone(), cache_entries));

    let consensus_block_store = Arc::new(ConsensusBlockStore::new_with_db(db_block_store, Some(db_header_store)));
    let consensus_utxo = Arc::new(UtxoSet::new_with_db(db_utxo_store));

    let consensus_storage = Arc::new(ConsensusStorage::with_stores(consensus_block_store, consensus_utxo));

        Ok(Self {
            config: config.clone(),
            consensus_storage,
        })
    }

    /// Get consensus storage
    pub fn consensus_storage(&self) -> Arc<ConsensusStorage> {
        self.consensus_storage.clone()
    }

    /// Get block store
    pub fn block_store(&self) -> Arc<ConsensusBlockStore> {
        self.consensus_storage.block_store()
    }

    /// Get UTXO set
    pub fn utxo_set(&self) -> Arc<UtxoSet> {
        self.consensus_storage.utxo_set()
    }

    /// Get data directory
    pub fn data_dir(&self) -> &Path {
        &self.config.data_dir
    }

    /// Check if storage is ready
    pub fn is_ready(&self) -> bool {
        // Basic readiness check - in a real implementation,
        // this would check database connections, etc.
        true
    }
}
