//! Main indexer service

use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};
use tracing::{info, error};
use consensus_core::{block::Block, Hash};
use crate::database::Database;
use crate::indexer::{block_indexer::BlockIndexer, transaction_indexer::TransactionIndexer, address_indexer::AddressIndexer};
use crate::error::Result;
use rpc_core::RpcApi;

pub struct IndexerService {
    database: Arc<Database>,
    block_indexer: BlockIndexer,
    tx_indexer: TransactionIndexer,
    address_indexer: AddressIndexer,
    block_sender: broadcast::Sender<Block>,
}

impl IndexerService {
    pub fn new(database: Arc<Database>) -> Self {
        let (block_sender, _) = broadcast::channel(100);

        Self {
            database: database.clone(),
            block_indexer: BlockIndexer::new(database.clone()),
            tx_indexer: TransactionIndexer::new(database.clone()),
            address_indexer: AddressIndexer::new(database.clone()),
            block_sender,
        }
    }
    
    pub fn block_sender(&self) -> broadcast::Sender<Block> {
        self.block_sender.clone()
    }
    
    pub async fn start(&self, coordinator: Arc<dyn RpcApi>) -> Result<()> {
        info!("Starting indexer service");

        // Start indexing loop
        let mut interval = interval(Duration::from_secs(5));
        let mut last_processed_height: i64 = -1;

        loop {
            interval.tick().await;

            // Get current block count from coordinator
            let block_count = match coordinator.get_block_count().await {
                Ok(count) => count as i64,
                Err(e) => {
                    error!("Failed to get block count: {:?}", e);
                    continue;
                }
            };

            // Process new blocks
            if block_count > last_processed_height + 1 {
                for height in (last_processed_height + 1)..block_count {
                    if let Err(e) = self.process_block_at_height(height, &coordinator).await {
                        error!("Failed to process block at height {}: {:?}", height, e);
                        break;
                    }
                    last_processed_height = height;
                }
            }
        }
    }
    
    async fn process_block_at_height(&self, height: i64, coordinator: &Arc<dyn RpcApi>) -> Result<()> {
        // Get block by height using the RPC method
        if let Ok(block) = coordinator.get_block_by_height(height as u64).await {
            // Index the block
            if let Err(e) = self.index_block(block).await {
                tracing::warn!("Failed to index block at height {}: {:?}", height, e);
            }
        } else {
            tracing::warn!("Failed to get block at height {}", height);
        }
        Ok(())
    }
    
    async fn index_block(&self, block: Block) -> Result<()> {
        info!("Indexing block: {}", block.header.hash);
        
        // Index block
        self.block_indexer.index(&block).await?;
        
        // Index transactions
        for tx in &block.transactions {
            self.tx_indexer.index(tx, Some(&block)).await?;
        }
        
        // Update addresses
        for tx in &block.transactions {
            self.address_indexer.update_from_transaction(tx).await?;
        }
        
        // Broadcast block event
        let _ = self.block_sender.send(block);
        
        Ok(())
    }
}

