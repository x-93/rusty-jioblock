//! Data models for the explorer

use serde::{Deserialize, Serialize};
use consensus_core::{block::Block, tx::Transaction, Hash};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlockSummary {
    pub hash: String,
    pub height: i64,
    pub timestamp: i64,
    pub tx_count: i32,
    pub size: i32,
    pub coinbase_value: i64,
    pub parent_count: i32,
    pub blue_score: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TransactionSummary {
    pub hash: String,
    pub block_hash: Option<String>,
    pub block_height: Option<i64>,
    pub timestamp: i64,
    pub input_count: i32,
    pub output_count: i32,
    pub value: i64,
    pub fee: Option<i64>,
    pub size: i32,
    pub is_coinbase: bool,
    pub is_confirmed: bool,
    pub confirmation_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AddressSummary {
    pub address: String,
    pub balance: i64,
    pub tx_count: i32,
    pub received_count: i32,
    pub sent_count: i32,
    pub total_received: i64,
    pub total_sent: i64,
    pub utxo_count: i32,
    pub first_seen: Option<i64>,
    pub last_seen: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub block_count: i64,
    pub tx_count: i64,
    pub address_count: i64,
    pub total_supply: i64,
    pub hashrate: Option<i64>,
    pub difficulty: Option<f64>,
    pub avg_block_time: Option<f64>,
    pub mempool_size: i32,
    pub mempool_bytes: i64,
    pub peer_count: i32,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub blocks: Vec<BlockSummary>,
    pub transactions: Vec<TransactionSummary>,
    pub addresses: Vec<AddressSummary>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    pub hash: String,
    pub is_confirmed: bool,
    pub block_hash: Option<String>,
    pub block_height: Option<i64>,
    pub confirmation_count: i32,
    pub is_in_mempool: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBalance {
    pub address: String,
    pub balance: i64,
    pub pending_balance: i64,
    pub utxo_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStats {
    pub total_blocks: i64,
    pub total_transactions: i64,
    pub total_value: i64,
    pub avg_block_time: f64,
    pub avg_tx_per_block: f64,
    pub avg_block_size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningInfo {
    pub network_hash_ps: i64,
    pub pooled_tx: i32,
    pub chain: String,
    pub warnings: String,
    pub difficulty: f64,
    pub blocks: i64,
    pub current_block_weight: Option<i32>,
    pub current_block_tx: Option<i32>,
    pub errors: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDagInfo {
    pub block_count: i64,
    pub tip_hashes: Vec<String>,
    pub difficulty: f64,
    pub network: String,
    pub virtual_parent_hashes: Vec<String>,
    pub pruning_point_hash: String,
}

