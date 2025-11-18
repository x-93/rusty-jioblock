//! RPC client for connecting to JIOPad daemon

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use consensus_core::{block::Block, tx::Transaction, Hash};
use rpc_core::{RpcApi, RpcError, model::*};

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    #[serde(default)]
    result: serde_json::Value,
    #[serde(default)]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(default)]
    data: Option<serde_json::Value>,
}

pub struct RpcClient {
    url: String,
    next_id: Arc<Mutex<u64>>,
}



impl RpcClient {
    pub fn new(url: &str) -> Result<Self, RpcError> {
        Ok(Self {
            url: url.to_string(),
            next_id: Arc::new(Mutex::new(1)),
        })
    }

    async fn call_method(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, RpcError> {
        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| RpcError::Network(format!("WebSocket connection failed: {}", e)))?;

        let (mut write, mut read) = ws_stream.split();

        let id = {
            let mut next_id = self.next_id.lock().await;
            let id = *next_id;
            *next_id += 1;
            id
        };

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let request_json = serde_json::to_string(&request)
            .map_err(|e| RpcError::Internal(format!("Request serialization failed: {}", e)))?;

        write.send(Message::Text(request_json)).await
            .map_err(|e| RpcError::Network(format!("Send failed: {}", e)))?;

        // Read response
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    let response: JsonRpcResponse = serde_json::from_str(&text)
                        .map_err(|e| RpcError::Internal(format!("Response parsing failed: {}", e)))?;

                    if response.id != id {
                        continue; // Not our response
                    }

                    if let Some(error) = response.error {
                        return Err(RpcError::Internal(format!("RPC error {}: {}", error.code, error.message)));
                    }

                    return Ok(response.result);
                }
                Ok(Message::Close(_)) => break,
                Err(e) => return Err(RpcError::Network(format!("WebSocket error: {}", e))),
                _ => continue,
            }
        }

        Err(RpcError::Network("Connection closed without response".to_string()))
    }
}

#[async_trait]
impl RpcApi for RpcClient {
    async fn get_block_count(&self) -> Result<u64, RpcError> {
        let result = self.call_method("getBlockCount", serde_json::json!([])).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn get_block(&self, hash: Hash) -> Result<Block, RpcError> {
        let params = serde_json::json!([hash.to_string()]);
        let result = self.call_method("getBlock", params).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn get_block_dag_info(&self) -> Result<BlockDagInfo, RpcError> {
        let result = self.call_method("getBlockDagInfo", serde_json::json!([])).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn get_blocks(&self, low_hash: Option<Hash>, include_blocks: bool, include_transactions: bool) -> Result<GetBlocksResponse, RpcError> {
        let params = serde_json::json!({
            "lowHash": low_hash.map(|h| h.to_string()),
            "includeBlocks": include_blocks,
            "includeTransactions": include_transactions
        });
        let result = self.call_method("getBlocks", params).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn get_peer_info(&self) -> Result<Vec<PeerInfo>, RpcError> {
        let result = self.call_method("getPeerInfo", serde_json::json!([])).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn add_peer(&self, address: String, is_permanent: bool) -> Result<(), RpcError> {
        let params = serde_json::json!([address, is_permanent]);
        self.call_method("addPeer", params).await?;
        Ok(())
    }

    async fn submit_block(&self, block: Block) -> Result<Hash, RpcError> {
        let params = serde_json::json!([block]);
        let result = self.call_method("submitBlock", params).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn send_raw_transaction(&self, tx_hex: String, allow_high_fees: bool) -> Result<Hash, RpcError> {
        let params = serde_json::json!([tx_hex, allow_high_fees]);
        let result = self.call_method("sendRawTransaction", params).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn get_mempool_info(&self) -> Result<MempoolInfo, RpcError> {
        let result = self.call_method("getMempoolInfo", serde_json::json!([])).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn get_mempool_entries(&self, include_orphan_pool: bool, filter_transaction_pool: bool) -> Result<Vec<MempoolEntry>, RpcError> {
        let params = serde_json::json!([include_orphan_pool, filter_transaction_pool]);
        let result = self.call_method("getMempoolEntries", params).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn get_block_template(&self, pay_address: String, extra_data: Option<String>) -> Result<BlockTemplate, RpcError> {
        let params = serde_json::json!([pay_address, extra_data]);
        let result = self.call_method("getBlockTemplate", params).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn submit_block_hex(&self, block_hex: String) -> Result<Hash, RpcError> {
        let params = serde_json::json!([block_hex]);
        let result = self.call_method("submitBlockHex", params).await?;
        let hash_str: String = serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))?;
        let bytes = hex::decode(&hash_str).map_err(|e| RpcError::Internal(format!("Hex decode error: {}", e)))?;
        let array: [u8; 32] = bytes.try_into().map_err(|_| RpcError::Internal("Invalid hash length".to_string()))?;
        Ok(Hash::from(array))
    }

    async fn get_mining_info(&self) -> Result<MiningInfo, RpcError> {
        let result = self.call_method("getMiningInfo", serde_json::json!([])).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn estimate_network_hashes_per_second(&self, window_size: u32, start_hash: Option<Hash>) -> Result<u64, RpcError> {
        let params = serde_json::json!([window_size, start_hash.map(|h| h.to_string())]);
        let result = self.call_method("estimateNetworkHashesPerSecond", params).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn get_balances(&self) -> Result<GetBalancesResponse, RpcError> {
        let result = self.call_method("getBalances", serde_json::json!([])).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn get_virtual_selected_parent_blue_score(&self) -> Result<u64, RpcError> {
        let result = self.call_method("getVirtualSelectedParentBlueScore", serde_json::json!([])).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn get_block_by_height(&self, height: u64) -> Result<Block, RpcError> {
        let params = serde_json::json!([height]);
        let result = self.call_method("getBlockByHeight", params).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn get_transaction(&self, hash: Hash) -> Result<Transaction, RpcError> {
        let params = serde_json::json!([hash.to_string()]);
        let result = self.call_method("getTransaction", params).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn get_recent_blocks(&self, count: usize) -> Result<Vec<Block>, RpcError> {
        let params = serde_json::json!([count]);
        let result = self.call_method("getRecentBlocks", params).await?;
        serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))
    }

    async fn get_dag_tips(&self) -> Result<Vec<Hash>, RpcError> {
        let result = self.call_method("getDagTips", serde_json::json!([])).await?;
        let hash_strings: Vec<String> = serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))?;
        hash_strings.into_iter()
            .map(|s| {
                let bytes = hex::decode(&s).map_err(|e| RpcError::Internal(format!("Hex decode error: {}", e)))?;
                let array: [u8; 32] = bytes.try_into().map_err(|_| RpcError::Internal("Invalid hash length".to_string()))?;
                Ok(Hash::from(array))
            })
            .collect()
    }

    async fn get_block_children(&self, hash: Hash) -> Result<Vec<Hash>, RpcError> {
        let params = serde_json::json!([hash.to_string()]);
        let result = self.call_method("getBlockChildren", params).await?;
        let hash_strings: Vec<String> = serde_json::from_value(result).map_err(|e| RpcError::Internal(format!("Deserialization error: {}", e)))?;
        hash_strings.into_iter()
            .map(|s| {
                let bytes = hex::decode(&s).map_err(|e| RpcError::Internal(format!("Hex decode error: {}", e)))?;
                let array: [u8; 32] = bytes.try_into().map_err(|_| RpcError::Internal("Invalid hash length".to_string()))?;
                Ok(Hash::from(array))
            })
            .collect()
    }
}
