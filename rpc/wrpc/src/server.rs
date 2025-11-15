//! WebSocket RPC server for browser/web clients

use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info};
use rpc_core::RpcCoordinator;
use rpc_core::RpcApi;

#[derive(Debug, serde::Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, serde::Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, serde::Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

pub struct WrpcServer {
    coordinator: Arc<RpcCoordinator>,
    port: u16,
}

impl WrpcServer {
    pub fn new(coordinator: Arc<RpcCoordinator>, port: u16) -> Self {
        Self { coordinator, port }
    }

    pub async fn start(&self) -> Result<(), String> {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| format!("Failed to bind: {}", e))?;

        info!("wRPC server listening on {}", addr);

        loop {
            let (stream, _) = listener.accept().await
                .map_err(|e| format!("Accept error: {}", e))?;

            let coordinator = self.coordinator.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, coordinator).await {
                    error!("WebSocket error: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        stream: tokio::net::TcpStream,
        coordinator: Arc<RpcCoordinator>,
    ) -> Result<(), String> {
        let ws_stream = accept_async(stream).await
            .map_err(|e| format!("WebSocket handshake error: {}", e))?;

        let peer_addr = ws_stream.get_ref().peer_addr().ok();
        let (mut write, mut read) = ws_stream.split();

        while let Some(item) = read.next().await {
            match item {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => {
                            if let Some(addr) = peer_addr {
                                info!("Received WS message from {}: {}", addr, text);
                            }

                            // Handle request and reply; if handling fails, log and continue
                            match Self::handle_request(&text, &coordinator).await {
                                Ok(response) => {
                                    if let Err(e) = write.send(Message::Text(response)).await {
                                        error!("Write error: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("Request handling error: {}", e);
                                }
                            }
                        }
                        Message::Close(_) => break,
                        _ => { /* Ignore other message types */ }
                    }
                }
                Err(e) => {
                    // Client disconnected or protocol error; this is normal when clients close connections
                    // Only log at debug level to reduce noise
                    if let Some(addr) = peer_addr {
                        tracing::debug!("WebSocket client {} disconnected: {}", addr, e);
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_request(
        request: &str,
        coordinator: &Arc<RpcCoordinator>,
    ) -> Result<String, String> {
        // Parse JSON-RPC request
        let rpc_req: JsonRpcRequest = serde_json::from_str(request)
            .map_err(|e| format!("Invalid JSON-RPC request: {}", e))?;

        // Route to appropriate method
        let result = match rpc_req.method.as_str() {
            "getBlockCount" => {
                let count = coordinator.get_block_count().await
                    .map_err(|e| format!("getBlockCount error: {:?}", e))?;
                serde_json::json!(count)
            }
            "getBlock" => {
                let params = rpc_req.params.ok_or("Missing params")?;
                let hash_str = params.get("hash")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing hash parameter")?;

                // For now, return a placeholder - need to implement Hash parsing
                serde_json::json!({
                    "hash": hash_str,
                    "height": 0,
                    "timestamp": 0,
                    "transactions": []
                })
            }
            "getBlockDagInfo" => {
                let info = coordinator.get_block_dag_info().await
                    .map_err(|e| format!("getBlockDagInfo error: {:?}", e))?;
                serde_json::json!({
                    "blockCount": info.block_count,
                    "tipHashes": info.tip_hashes.iter().map(|h| h.to_string()).collect::<Vec<_>>(),
                    "difficulty": info.difficulty,
                    "network": info.network,
                    "virtualParentHashes": info.virtual_parent_hashes.iter().map(|h| h.to_string()).collect::<Vec<_>>(),
                    "pruningPointHash": info.pruning_point_hash.to_string()
                })
            }
            "getPeerInfo" => {
                let peers = coordinator.get_peer_info().await
                    .map_err(|e| format!("getPeerInfo error: {:?}", e))?;
                serde_json::json!(peers)
            }
            "getMempoolInfo" => {
                let info = coordinator.get_mempool_info().await
                    .map_err(|e| format!("getMempoolInfo error: {:?}", e))?;
                serde_json::json!({
                    "size": info.size,
                    "bytes": info.bytes
                })
            }
            "getBlockTemplate" => {
                // Return full JSON-serializable BlockTemplate from rpc_core::model
                // Use a default mining address if none provided
                let template = coordinator.get_block_template("1A1z7agoat3FwzZsQwtfTHtVtWWbnSFAZa".to_string(), None).await
                    .map_err(|e| format!("getBlockTemplate error: {:?}", e))?;
                serde_json::to_value(&template).map_err(|e| format!("Serialization error: {}", e))?
            }
            "submitBlockHex" => {
                // Expect params: { "blockHex": "..." }
                let params = rpc_req.params.as_ref().ok_or("Missing params")?;
                let hex = params.get("blockHex")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing blockHex parameter")?;

                let hash = coordinator.submit_block_hex(hex.to_string()).await
                    .map_err(|e| format!("submitBlockHex error: {:?}", e))?;

                serde_json::json!(hash.to_string())
            }
            "getMiningInfo" => {
                let info = coordinator.get_mining_info().await
                    .map_err(|e| format!("getMiningInfo error: {:?}", e))?;
                serde_json::to_value(&info).map_err(|e| format!("Serialization error: {}", e))?
            }
            _ => {
                return Err(format!("Unknown method: {}", rpc_req.method));
            }
        };

        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: rpc_req.id,
            result: Some(result),
            error: None,
        };

        serde_json::to_string(&response)
            .map_err(|e| format!("Serialization error: {}", e))
    }
}
