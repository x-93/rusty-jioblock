//! RPC API trait definitions

use async_trait::async_trait;
use consensus_core::{block::Block, tx::Transaction, Hash};
use crate::model::*;

/// Core RPC API trait defining all available RPC methods
#[async_trait]
pub trait RpcApi {
    // Blockchain methods
    async fn get_block_count(&self) -> Result<u64, RpcError>;
    async fn get_block(&self, hash: Hash) -> Result<Block, RpcError>;
    async fn get_block_dag_info(&self) -> Result<BlockDagInfo, RpcError>;
    async fn get_blocks(&self, low_hash: Option<Hash>, include_blocks: bool, include_transactions: bool) -> Result<GetBlocksResponse, RpcError>;

    // Network methods
    async fn get_peer_info(&self) -> Result<Vec<PeerInfo>, RpcError>;
    async fn add_peer(&self, address: String, is_permanent: bool) -> Result<(), RpcError>;
    async fn submit_block(&self, block: Block) -> Result<Hash, RpcError>;

    // Transaction methods
    async fn send_raw_transaction(&self, tx_hex: String, allow_high_fees: bool) -> Result<Hash, RpcError>;
    async fn get_mempool_info(&self) -> Result<MempoolInfo, RpcError>;
    async fn get_mempool_entries(&self, include_orphan_pool: bool, filter_transaction_pool: bool) -> Result<Vec<MempoolEntry>, RpcError>;

    // Mining methods
    async fn get_block_template(&self, pay_address: String, extra_data: Option<String>) -> Result<BlockTemplate, RpcError>;
    async fn submit_block_hex(&self, block_hex: String) -> Result<Hash, RpcError>;
    async fn get_mining_info(&self) -> Result<MiningInfo, RpcError>;

    // Wallet methods (integration with wallet crate)
    async fn estimate_network_hashes_per_second(&self, window_size: u32, start_hash: Option<Hash>) -> Result<u64, RpcError>;
    async fn get_balances(&self) -> Result<GetBalancesResponse, RpcError>;
    async fn get_virtual_selected_parent_blue_score(&self) -> Result<u64, RpcError>;
    
    // Additional methods for explorer
    async fn get_block_by_height(&self, height: u64) -> Result<Block, RpcError>;
    async fn get_transaction(&self, hash: Hash) -> Result<Transaction, RpcError>;
    async fn get_recent_blocks(&self, count: usize) -> Result<Vec<Block>, RpcError>;
    async fn get_dag_tips(&self) -> Result<Vec<Hash>, RpcError>;
    async fn get_block_children(&self, hash: Hash) -> Result<Vec<Hash>, RpcError>;
}

/// Notification API for streaming events
#[async_trait]
pub trait NotificationApi {
    async fn notify_block_added(&self) -> Result<(), RpcError>;
    async fn notify_virtual_selected_parent_chain_changed(&self) -> Result<(), RpcError>;
    async fn notify_finality_conflicts(&self) -> Result<(), RpcError>;
    async fn notify_virtual_daa_score_changed(&self) -> Result<(), RpcError>;
}
