use std::sync::{Arc, Mutex};
use crate::consensus_manager::ConsensusManager;
use crate::network_manager::NetworkManager;
use crate::mempool::Mempool;
use crate::mining_coordinator::MiningCoordinator;
use crate::config::RpcConfig;
use rpc_wrpc::WrpcServer;
use rpc_core::RpcCoordinator;
use network::hub::Hub;
use tokio::task::JoinHandle;
use tracing::info;

/// RPC server that manages WebSocket and HTTP RPC endpoints
pub struct RpcServer {
    config: RpcConfig,
    server_handle: Mutex<Option<JoinHandle<Result<(), String>>>>,
    coordinator: Arc<RpcCoordinator>,
}

impl RpcServer {
    /// Create a new RPC server instance
    pub async fn new(cfg: &RpcConfig, consensus: Arc<ConsensusManager>, _network: Arc<NetworkManager>, mempool: Arc<Mempool>) -> Result<Self, String> {
        // Build minimal Hub for RPC coordinator (will not be fully integrated with NetworkManager yet)
        let hub = Arc::new(Hub::new());

        // Create RpcCoordinator using components from ConsensusManager and provided mempool
        let coordinator = Arc::new(RpcCoordinator::new(
            consensus.block_processor(),
            consensus.storage(),
            hub,
            mempool.clone() as Arc<dyn rpc_core::mempool::MempoolInterface>,
            None,
        ));

        Ok(Self {
            config: cfg.clone(),
            server_handle: Mutex::new(None),
            coordinator,
        })
    }

    /// Start the RPC server
    pub async fn start(&self) -> Result<(), String> {
        info!("RPC server configured for {}:{}", self.config.bind_address, self.config.port);
        // Start the wRPC server in a background task
        let wrpc = WrpcServer::new(self.coordinator.clone(), self.config.port);
        let handle = tokio::spawn(async move { wrpc.start().await });

        let mut guard = self.server_handle.lock().unwrap();
        *guard = Some(handle);

        Ok(())
    }

    /// Stop the RPC server
    pub async fn stop(&self) -> Result<(), String> {
        let mut handle = self.server_handle.lock().unwrap();
        if let Some(h) = handle.take() {
            h.abort();
            info!("RPC server stopped");
        }
        Ok(())
    }
}
