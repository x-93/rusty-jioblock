use tempfile::TempDir;
use std::sync::Arc;

use database::Database;
use database::stores::{BlockStore as DbBlockStore, HeaderStore as DbHeaderStore, UtxoStore as DbUtxoStore};
use consensus::consensus::storage::{ConsensusStorage, BlockStore as ConsensusBlockStore, UtxoSet};
use consensus_core::{ZERO_HASH, BlueWorkType};
use consensus_core::header::Header;
use consensus_core::block::Block;

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
fn test_persistence_block_and_utxo() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path();

    // Open DB and create stores
    let db = Arc::new(Database::open(path).expect("open db"));
    let db_block = Arc::new(DbBlockStore::new(db.clone(), 16));
    let db_header = Arc::new(DbHeaderStore::new(db.clone(), 16));
    let db_utxo = Arc::new(DbUtxoStore::new(db.clone(), 16));

    // Create consensus storage with DB-backed stores
    let cs = ConsensusStorage::with_stores(
        Arc::new(ConsensusBlockStore::new_with_db(db_block.clone(), Some(db_header.clone()))),
        Arc::new(UtxoSet::new_with_db(db_utxo.clone())),
    );

    let block = create_test_block();
    let hash = block.header.hash;

    // Apply block (stores block and updates utxo set)
    cs.apply_block(&block, 100).expect("apply block");

    // Ensure the block exists
    assert!(cs.has_block(&hash));

    drop(cs);
    drop(db_block);
    drop(db_header);
    drop(db_utxo);
    drop(db);

    // Reopen DB and stores
    let db2 = Arc::new(Database::open(path).expect("open db2"));
    let db_block2 = Arc::new(DbBlockStore::new(db2.clone(), 16));
    let db_header2 = Arc::new(DbHeaderStore::new(db2.clone(), 16));
    let db_utxo2 = Arc::new(DbUtxoStore::new(db2.clone(), 16));

    let cs2 = ConsensusStorage::with_stores(
        Arc::new(ConsensusBlockStore::new_with_db(db_block2.clone(), Some(db_header2.clone()))),
        Arc::new(UtxoSet::new_with_db(db_utxo2.clone())),
    );

    // Block should be persisted
    assert!(cs2.has_block(&hash));
}
