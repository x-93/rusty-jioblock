use jiopad::config::Config;
use jiopad::storage_manager::StorageManager;
use jiopad::consensus_manager::ConsensusManager;
use std::sync::Arc;
use tokio::runtime::Runtime;
use std::fs;

#[test]
fn test_genesis_bootstrap() {
    // Create a temporary data dir under tmp
    let tmp_dir = std::env::temp_dir().join("jiopad_test_data");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    // Build a default config and override data_dir
    let mut config = Config::default();
    config.storage.data_dir = tmp_dir.clone();

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        // Create StorageManager
        let storage_manager = StorageManager::new(&config.storage).await.unwrap();

        // Create ConsensusManager using storage_manager and network config
        let consensus_manager = ConsensusManager::new(&config.consensus, Arc::new(storage_manager), &config.network).await.unwrap();

        // Check that block store contains the genesis block (block_count >= 1)
        let block_count = consensus_manager.storage().block_store().block_count();
        assert!(block_count >= 1, "Expected at least 1 block (genesis) in block store");

        // Check that UTXO set has at least 1 entry (coinbase output)
        let utxo_len = consensus_manager.storage().utxo_set().len();
        assert!(utxo_len >= 1, "Expected at least 1 UTXO after genesis bootstrap");
    });
}
