use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use consensus::{BlockProcessor, ConsensusStorage};
use consensus_core::{block::Block, tx::Transaction, Hash, BlockHashSet, HashMapCustomHasher};
use crate::api::RpcApi;
use crate::model::*;
use crate::mempool::MempoolInterface;
use network::Hub;
use wallet::Keys;


/// RPC Coordinator implementing the RpcApi trait
pub struct RpcCoordinator {
    processor: Arc<BlockProcessor>,
    storage: Arc<ConsensusStorage>,
    network: Arc<Hub>,
    mempool: Arc<dyn MempoolInterface>,
    wallet: Option<Arc<Keys>>,
    active_connections: Arc<RwLock<usize>>,
    peers: Arc<RwLock<HashMap<String, String>>>,
    recent_block_hashes: Arc<RwLock<BlockHashSet>>,
}

impl RpcCoordinator {
    pub fn new(
        processor: Arc<BlockProcessor>,
        storage: Arc<ConsensusStorage>,
        network: Arc<Hub>,
        mempool: Arc<dyn MempoolInterface>,
        wallet: Option<Arc<Keys>>,
    ) -> Self {
        Self {
            processor,
            storage,
            network,
            mempool,
            wallet,
            active_connections: Arc::new(RwLock::new(0)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            recent_block_hashes: Arc::new(RwLock::new(BlockHashSet::new())),
        }
    }

    // Helper methods for hex encoding/decoding
    fn decode_hex_to_block(&self, hex: &str) -> Result<Block, RpcError> {
        match hex::decode(hex) {
            Ok(bytes) => match bincode::deserialize::<Block>(&bytes) {
                Ok(block) => Ok(block),
                Err(e) => Err(RpcError::Rpc { code: -22, message: format!("Failed to deserialize block: {}", e) }),
            },
            Err(e) => Err(RpcError::Rpc { code: -22, message: format!("Failed to decode hex: {}", e) }),
        }
    }

    fn decode_hex_to_transaction(&self, hex: &str) -> Result<Transaction, RpcError> {
        match hex::decode(hex) {
            Ok(bytes) => match bincode::deserialize::<Transaction>(&bytes) {
                Ok(tx) => Ok(tx),
                Err(e) => Err(RpcError::Rpc { code: -22, message: format!("Failed to deserialize transaction: {}", e) }),
            },
            Err(e) => Err(RpcError::Rpc { code: -22, message: format!("Failed to decode hex: {}", e) }),
        }
    }

    fn encode_block_to_hex(&self, block: &Block) -> String {
        match bincode::serialize(block) {
            Ok(bytes) => hex::encode(&bytes),
            Err(_) => "".to_string(),
        }
    }

    fn get_virtual_daa_score(&self) -> u64 {
        // Get blue_score from virtual GHOSTDAG data (closest equivalent to DAA score)
        // If unavailable, fall back to zero.
        match self.processor.get_virtual_block_data(4) {
            Ok(vbd) => vbd.ghostdag_data.blue_score,
            Err(_) => 0,
        }
    }

    fn get_pruning_point_hash(&self) -> Hash {
        // Pruning point is typically the oldest block in our view of the DAG.
        // For now, return selected parent from virtual ghostdag data, or default if unavailable.
        match self.processor.get_virtual_block_data(4) {
            Ok(vbd) => vbd.ghostdag_data.selected_parent,
            Err(_) => Hash::default(),
        }
    }

    fn get_virtual_parent_hashes(&self) -> Vec<Hash> {
        match self.processor.get_virtual_block_data(4) {
            Ok(vbd) => vbd.parents,
            Err(_) => vec![],
        }
    }

    fn get_past_median_time(&self) -> u64 {
        // Past median time is calculated from selected parent blocks' timestamps
        // For now, use current Unix timestamp as a reasonable default
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn get_current_difficulty(&self) -> f64 {
        // Try to retrieve from the difficulty manager via processor or from virtual data
        // For now, use a reasonable default of 1.0 (represents relative difficulty)
        // In a full implementation, this would call the difficulty manager to compute
        // based on recent block times and current target.
        1.0
    }
}

#[async_trait::async_trait]
impl RpcApi for RpcCoordinator {
    async fn get_block_count(&self) -> Result<u64, RpcError> {
        let count = self.storage.block_store().block_count();
        Ok(count as u64)
    }

    async fn get_block(&self, hash: Hash) -> Result<Block, RpcError> {
        self.storage.get_block(&hash)
            .ok_or_else(|| RpcError::Rpc {
                code: -5,
                message: "Block not found".to_string(),
            })
    }

    async fn get_block_dag_info(&self) -> Result<BlockDagInfo, RpcError> {
        let tip_hashes = vec![]; // Tip tracking not implemented yet
        let virtual_parent_hashes = self.get_virtual_parent_hashes();
        let pruning_point_hash = self.get_pruning_point_hash();

        Ok(BlockDagInfo {
            block_count: self.get_block_count().await?,
            tip_hashes,
            difficulty: self.get_current_difficulty(),
            network: "testnet".to_string(), // default to testnet for this workspace
            virtual_parent_hashes,
            pruning_point_hash,
        })
    }

    async fn get_blocks(&self, _low_hash: Option<Hash>, _include_blocks: bool, _include_transactions: bool) -> Result<GetBlocksResponse, RpcError> {
        // Minimal implementation: return the requested block when low_hash is provided
        if let Some(low_hash) = _low_hash {
            if let Some(b) = self.storage.get_block(&low_hash) {
                return Ok(GetBlocksResponse { blocks: vec![b], next_block_hashes: vec![] });
            }
        }

        Ok(GetBlocksResponse { blocks: vec![], next_block_hashes: vec![] })
    }

    async fn get_peer_info(&self) -> Result<Vec<PeerInfo>, RpcError> {
        // Network hub integration not implemented yet
        Ok(vec![])
    }

    async fn add_peer(&self, _address: String, _is_permanent: bool) -> Result<(), RpcError> {
        // Peer addition not implemented; no-op for now
        Ok(())
    }

    async fn submit_block(&self, block: Block) -> Result<Hash, RpcError> {
        match self.processor.process_block(block) {
            Ok(result) => Ok(result.hash),
            Err(e) => Err(RpcError::Rpc {
                code: -25,
                message: format!("Block submission failed: {:?}", e),
            }),
        }
    }

    async fn send_raw_transaction(&self, tx_hex: String, _allow_high_fees: bool) -> Result<Hash, RpcError> {
        let tx = self.decode_hex_to_transaction(&tx_hex)?;

        // Add to mempool
        self.mempool.add_transaction(tx.clone()).map_err(|e| RpcError::Rpc {
            code: -25,
            message: format!("Transaction rejected: {}", e),
        })?;

        // Broadcast to network (best-effort)
        let message = network::protowire::Message::Transaction(tx.clone());
        self.network.broadcast(message).await;

        Ok(tx.hash())
    }

    async fn get_mempool_info(&self) -> Result<MempoolInfo, RpcError> {
        Ok(MempoolInfo {
            size: self.mempool.size(),
            bytes: 0,
        })
    }

    async fn get_mempool_entries(&self, _include_orphan_pool: bool, _filter_transaction_pool: bool) -> Result<Vec<MempoolEntry>, RpcError> {
        Ok(self.mempool.get_entries())
    }

    async fn get_block_template(&self, pay_address: String, _extra_data: Option<String>) -> Result<BlockTemplate, RpcError> {
        // Build a simple block template using virtual parents from the processor.
        // If the virtual parent data is not yet available (early startup), fall back
        // to genesis so external tools (miners) can still request templates.
        let transactions = self.mempool.get_all_transactions();
        let parent_hashes = match self.processor.get_virtual_block_data(4) {
            Ok(vbd) => vbd.parents,
            Err(_e) => {
                // This is normal when the chain is empty or just starting
                // Use genesis hash as parent for the first block
                vec![consensus_core::ZERO_HASH]
            }
        };

        // Try to construct a realistic coinbase transaction and merkle root.
        // Use the consensus coinbase processor with default config to compute reward.
        let config = consensus::ConsensusConfig::default();
        let coinbase_proc = consensus::process::coinbase::CoinbaseProcessor::new(config);

        // Build a ScriptPublicKey from the provided pay_address string (best-effort).
        let miner_spk = if pay_address.is_empty() {
            // Fallback to an empty script public key
            consensus_core::tx::ScriptPublicKey::new(0, Vec::new().into())
        } else {
            consensus_core::tx::ScriptPublicKey::new(0, pay_address.clone().into_bytes().into())
        };

        let block_height = self.get_virtual_daa_score();

        // Create coinbase tx with fees=0 (mempool fees not yet tracked)
        let coinbase_tx = coinbase_proc.create_coinbase_transaction(&miner_spk, block_height, 0);

        // Build full transaction list (coinbase first)
        let mut full_txs = Vec::with_capacity(1 + transactions.len());
        full_txs.push(coinbase_tx.clone());
        full_txs.extend(transactions.clone());

        // Compute a simple merkle root from the transactions
        // For now, just use the coinbase transaction hash as merkle root placeholder
        // A full implementation would build a proper merkle tree
        fn compute_merkle_root(txs: &[consensus_core::tx::Transaction]) -> consensus_core::Hash {
            if txs.is_empty() {
                return consensus_core::Hash::from_le_u64([0, 0, 0, 0]);
            }
            // For now, just use first transaction (coinbase) hash as placeholder
            // Real implementation would build proper merkle tree
            txs[0].hash()
        }

        let _merkle_root = compute_merkle_root(&full_txs);

        // Use a placeholder bits value for now (compact representation)
        // In production, this should come from the difficulty manager
        let bits: u32 = 0x1f00ffff;

        let coinbase_value = coinbase_tx.outputs.get(0).map(|o| o.value).unwrap_or(0);
        // Use milliseconds for better timestamp precision to ensure unique templates
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Log template details for debugging
        eprintln!(
            "[BlockTemplate] height={}, parents={}, txs={}, coinbase_value={}, bits={:08x}, timestamp={}",
            block_height,
            parent_hashes.len(),
            full_txs.len(),
            coinbase_value,
            bits,
            timestamp
        );

        Ok(BlockTemplate {
            version: 1,
            parent_hashes,
            transactions: full_txs,
            coinbase_value,
            bits,
            timestamp,
            pay_address,
            target: format!("{:08x}", bits),
        })
    }

    async fn estimate_network_hashes_per_second(&self, _window_size: u32, _start_hash: Option<Hash>) -> Result<u64, RpcError> {
        // Estimate network hashrate using difficulty and target time per block
        // Formula: hashrate ≈ (difficulty * 2^32) / target_time_per_block_secs
        // For now, use a conservative estimate based on default config
        let default_target_time = 1u64; // 1 second per block (from config)
        let difficulty = self.get_current_difficulty();
        
        // Estimate: assume current difficulty represents relative work
        let estimated_hashes = (difficulty as u64 * 1_000_000_000) / default_target_time;
        
        eprintln!("[NetworkHashrate] estimated={} H/s", estimated_hashes);
        Ok(estimated_hashes)
    }

    async fn get_balances(&self) -> Result<GetBalancesResponse, RpcError> {
        if let Some(_wallet) = &self.wallet {
            // TODO: Implement full wallet balance calculation
            // For now, return placeholder balances
            // Real implementation would:
            // 1. Get all addresses from wallet
            // 2. Query UTXO set for each address
            // 3. Sum spendable and pending UTXOs separately
            eprintln!("[Wallet] Returning placeholder balances (full UTXO integration pending)");
            Ok(GetBalancesResponse {
                available_balance: 0,
                pending_balance: 0,
            })
        } else {
            Err(RpcError::Rpc {
                code: -18,
                message: "Wallet not available".to_string(),
            })
        }
    }

    async fn get_virtual_selected_parent_blue_score(&self) -> Result<u64, RpcError> {
        Ok(self.get_virtual_daa_score())
    }

    async fn submit_block_hex(&self, block_hex: String) -> Result<Hash, RpcError> {
        let block = self.decode_hex_to_block(&block_hex)?;
        let block_hash = block.header.hash;
        
        // Check for duplicate block submission
        {
            let mut recent_hashes = self.recent_block_hashes.write().await;
            if recent_hashes.contains(&block_hash) {
                eprintln!("[submitBlockHex] Duplicate block submission detected: {}", block_hash);
                return Err(RpcError::Rpc {
                    code: -25,
                    message: format!("Duplicate block submission: {}", block_hash),
                });
            }
            // Keep only last 1000 block hashes to prevent memory growth
            if recent_hashes.len() > 1000 {
                recent_hashes.clear();
            }
            recent_hashes.insert(block_hash);
        }
        
        eprintln!("[submitBlockHex] Received block with hash: {}, nonce: {}, timestamp: {}", 
                  block_hash, block.header.nonce, block.header.timestamp);
        self.submit_block(block).await
    }

    async fn get_mining_info(&self) -> Result<MiningInfo, RpcError> {
        // For now, return placeholder data since mining coordinator integration is pending
        // In a full implementation, this would query the MiningCoordinator for real stats

        let network_hashrate = self.estimate_network_hashes_per_second(10, None).await.unwrap_or(1_000_000);

        Ok(MiningInfo {
            is_mining: false, // Placeholder - would check MiningCoordinator status
            current_hashrate: 0.0, // Placeholder - would get from MiningCoordinator
            network_hashrate,
            difficulty: self.get_current_difficulty(),
            blocks_mined: 0, // Placeholder - would get from MiningCoordinator
            total_mining_time_ms: 0, // Placeholder - would get from MiningCoordinator
            worker_count: 0, // Placeholder - would get from MiningCoordinator
            workers: vec![], // Placeholder - would get from MiningCoordinator
            mining_address: "".to_string(), // Placeholder - would get from MiningCoordinator
            current_template: None, // Could populate with current template info if available
        })
    }
    
    async fn get_block_by_height(&self, height: u64) -> Result<Block, RpcError> {
        // Get all blocks and find the one with matching DAA score (height)
        let all_blocks = self.storage.block_store().get_all_blocks();

        for block in all_blocks {
            if block.header.daa_score == height {
                return Ok(block);
            }
        }

        Err(RpcError::Rpc {
            code: -5,
            message: format!("Block at height {} not found", height),
        })
    }
    
    async fn get_transaction(&self, hash: Hash) -> Result<Transaction, RpcError> {
        // Try to get from mempool first
        let mempool_entries = self.mempool.get_entries();
        for entry in mempool_entries {
            if entry.transaction.hash() == hash {
                return Ok(entry.transaction);
            }
        }
        
        // Try to get from blocks
        // TODO: Implement transaction lookup from blocks
        Err(RpcError::Rpc {
            code: -5,
            message: "Transaction not found".to_string(),
        })
    }
    
    async fn get_recent_blocks(&self, count: usize) -> Result<Vec<Block>, RpcError> {
        // TODO: Implement recent blocks retrieval
        // For now, return empty vector
        Ok(vec![])
    }
    
    async fn get_dag_tips(&self) -> Result<Vec<Hash>, RpcError> {
        let virtual_parents = self.get_virtual_parent_hashes();
        Ok(virtual_parents)
    }
    
    async fn get_block_children(&self, hash: Hash) -> Result<Vec<Hash>, RpcError> {
        // TODO: Implement block children lookup
        // This requires maintaining a reverse index of parent->children
        Ok(vec![])
    }
}