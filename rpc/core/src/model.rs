//! RPC data models and types

use serde::{Deserialize, Serialize};
use thiserror::Error;
use consensus_core::{block::Block, tx::Transaction, Hash};

/// RPC error type
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum RpcError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("RPC error {code}: {message}")]
    Rpc { code: i32, message: String },
}

/// Block DAG information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDagInfo {
    pub block_count: u64,
    pub tip_hashes: Vec<Hash>,
    pub difficulty: f64,
    pub network: String,
    pub virtual_parent_hashes: Vec<Hash>,
    pub pruning_point_hash: Hash,
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub address: String,
    pub last_ping_duration: Option<u64>,
    pub is_connected: bool,
    pub version: u32,
    pub user_agent: String,
    pub advertised_protocol_version: u32,
    pub time_offset: i64,
    pub is_ibd_peer: bool,
}

/// Mempool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolInfo {
    pub size: usize,
    pub bytes: u64,
}

/// Mempool entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolEntry {
    pub fee: u64,
    pub transaction: Transaction,
    pub is_orphan: bool,
}

/// Block template for mining
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTemplate {
    pub version: u32,
    pub parent_hashes: Vec<Hash>,
    pub transactions: Vec<Transaction>,
    pub coinbase_value: u64,
    pub bits: u32,
    pub timestamp: u64,
    pub pay_address: String,
    pub target: String,
}

/// Get blocks response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBlocksResponse {
    pub blocks: Vec<Block>,
    pub next_block_hashes: Vec<Hash>,
}

/// Get balances response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBalancesResponse {
    pub available_balance: u64,
    pub pending_balance: u64,
}

/// Transaction output with address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOutput {
    pub transaction_id: Hash,
    pub index: u32,
    pub script_public_key: ScriptPublicKey,
    pub value: u64,
    pub is_spent: bool,
}

/// Script public key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptPublicKey {
    pub version: u16,
    pub script: Vec<u8>,
}

/// UTXO entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoEntry {
    pub amount: u64,
    pub script_public_key: ScriptPublicKey,
    pub block_daa_score: u64,
    pub is_coinbase: bool,
}

/// Fee estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimate {
    pub priority_bucket: FeeEstimateBucket,
    pub normal_buckets: Vec<FeeEstimateBucket>,
}

/// Fee estimate bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimateBucket {
    pub feerate: f64,
    pub estimated_seconds: f64,
}

/// Network connection info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnectionInfo {
    pub p2p_connections: usize,
    pub rpc_connections: usize,
    pub uas_connections: usize,
}

/// Consensus info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusInfo {
    pub virtual_daa_score: u64,
    pub block_count: u64,
    pub header_count: u64,
    pub tip_hashes: Vec<Hash>,
    pub difficulty: f64,
    pub past_median_time: u64,
    pub virtual_parent_hashes: Vec<Hash>,
    pub pruning_point_hash: Hash,
    pub virtual_daa_score_timestamp: u64,
}

/// Mining information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningInfo {
    /// Current mining status
    pub is_mining: bool,
    /// Current hashrate in H/s
    pub current_hashrate: f64,
    /// Network hashrate estimate in H/s
    pub network_hashrate: u64,
    /// Current difficulty
    pub difficulty: f64,
    /// Number of blocks mined in current session
    pub blocks_mined: u64,
    /// Total mining time in milliseconds
    pub total_mining_time_ms: u64,
    /// Number of active mining workers
    pub worker_count: usize,
    /// Individual worker statistics
    pub workers: Vec<WorkerInfo>,
    /// Mining address
    pub mining_address: String,
    /// Current block template info
    pub current_template: Option<BlockTemplateInfo>,
}

/// Worker information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    pub id: usize,
    pub blocks_mined: u64,
    pub hashrate: f64,
    pub total_iterations: u64,
    pub uptime_ms: u64,
    pub efficiency: f64, // percentage
}

/// Block template information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTemplateInfo {
    pub height: u64,
    pub timestamp: u64,
    pub coinbase_value: u64,
    pub transaction_count: usize,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    pub hash: String,
    pub is_confirmed: bool,
    pub block_hash: Option<String>,
    pub block_height: Option<u64>,
    pub confirmation_count: u32,
    pub is_in_mempool: bool,
}

/// Address balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBalance {
    pub address: String,
    pub balance: u64,
    pub pending_balance: u64,
    pub utxo_count: u32,
}

/// Address summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressSummary {
    pub address: String,
    pub balance: u64,
    pub tx_count: u64,
    pub received_count: u64,
    pub sent_count: u64,
    pub total_received: u64,
    pub total_sent: u64,
    pub utxo_count: u32,
    pub first_seen: Option<u64>,
    pub last_seen: Option<u64>,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub block_count: u64,
    pub tx_count: u64,
    pub address_count: u64,
    pub total_supply: u64,
    pub hashrate: Option<u64>,
    pub difficulty: Option<f64>,
    pub avg_block_time: Option<f64>,
    pub mempool_size: usize,
    pub mempool_bytes: u64,
    pub peer_count: usize,
}

/// Search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub blocks: Vec<String>, // Block hashes
    pub transactions: Vec<String>, // Transaction hashes
    pub addresses: Vec<String>, // Addresses
    pub total: usize,
}