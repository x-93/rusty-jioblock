use crate::consensus_manager::ConsensusManager;
use crate::network_manager::NetworkManager;
use consensus::process::sync::SyncProcess;
use consensus::pipeline::BlockProcessor;
use consensus_core::block::Block;
use consensus_core::Hash;
use std::sync::Arc;

/// Sync manager that handles block synchronization
pub struct SyncManager {
    sync_process: Arc<SyncProcess>,
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new(network: Arc<NetworkManager>, consensus: Arc<ConsensusManager>) -> Self {
        let sync_process = Arc::new(SyncProcess::new(
            consensus.block_processor(),
            consensus.storage().block_store(),
        ));

        Self { sync_process }
    }

    /// Start synchronization
    pub async fn start(&self) -> Result<(), String> {
        // In a real implementation, this would start IBD or ongoing sync
        // For now, just mark as started
        Ok(())
    }

    /// Stop synchronization
    pub async fn stop(&self) -> Result<(), String> {
        // Cleanup sync state
        Ok(())
    }

    /// Process a block received during sync
    pub async fn process_sync_block(&self, block: Block) -> Result<(), String> {
        let status = self.sync_process.process_sync_block(block)
            .map_err(|e| format!("Failed to process sync block: {}", e))?;

        match status {
            consensus::consensus::types::BlockStatus::Valid => Ok(()),
            consensus::consensus::types::BlockStatus::Invalid => Err("Block validation failed".to_string()),
            consensus::consensus::types::BlockStatus::Orphan => Err("Block is orphaned".to_string()),
            consensus::consensus::types::BlockStatus::HeaderOnly => Err("Block is header-only".to_string()),
        }
    }

    /// Check if sync is complete
    pub fn is_sync_complete(&self) -> bool {
        self.sync_process.is_sync_complete()
    }

    /// Get sync progress (0.0 to 1.0)
    pub fn get_sync_progress(&self) -> f64 {
        self.sync_process.get_sync_progress()
    }
}
