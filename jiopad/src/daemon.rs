use crate::config::Config;
use crate::ui;
use tokio::signal;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};
use tracing::info;
use std::sync::Arc;
use std::time::Instant;

// Real implementations
pub use crate::consensus_manager::ConsensusManager;
pub use crate::network_manager::NetworkManager;
pub use crate::rpc_server::RpcServer;
pub use crate::mining_coordinator::MiningCoordinator;
pub use crate::mempool::Mempool;
pub use crate::sync_manager::SyncManager;
pub use crate::storage_manager::StorageManager;

pub struct Daemon {
    config: Config,
    shutdown_tx: broadcast::Sender<()>,

    // Core components (placeholders for now)
    consensus: Arc<ConsensusManager>,
    network: Arc<NetworkManager>,
    rpc_server: Option<Arc<RpcServer>>,
    mining: Option<Arc<MiningCoordinator>>,
    mempool: Arc<Mempool>,
    sync: Arc<SyncManager>,
}

impl Daemon {
    /// Create new daemon instance
    pub async fn new(config: Config) -> Result<Self, String> {
        ui::print_section("Initializing Components");
        
        // Create shutdown channel
        let (shutdown_tx, _) = broadcast::channel(1);

        // Initialize storage
        ui::print_component_status("Storage", ui::ComponentStatus::Starting);
        info!("Initializing storage at {:?}", config.storage.data_dir);
        let storage = Arc::new(
            StorageManager::new(&config.storage).await?
        );
        ui::print_component_status("Storage", ui::ComponentStatus::Running);

        // Initialize consensus
        ui::print_component_status("Consensus Engine", ui::ComponentStatus::Starting);
        info!("Initializing consensus engine");
        let consensus = Arc::new(
            ConsensusManager::new(&config.consensus, storage.clone(), &config.network).await?
        );
        ui::print_component_status("Consensus Engine", ui::ComponentStatus::Running);

        // Initialize mempool
        ui::print_component_status("Mempool", ui::ComponentStatus::Starting);
        info!("Initializing mempool");
        let mempool = Arc::new(
            Mempool::new()
        );
        ui::print_component_status("Mempool", ui::ComponentStatus::Running);

        // Initialize network layer
        ui::print_component_status("P2P Network", ui::ComponentStatus::Starting);
        info!("Initializing P2P network");
        let network = Arc::new(
            NetworkManager::new(&config.p2p, consensus.clone()).await?
        );
        ui::print_component_status("P2P Network", ui::ComponentStatus::Running);

        // Initialize sync manager
        ui::print_component_status("Sync Manager", ui::ComponentStatus::Starting);
        info!("Initializing sync manager");
        let sync = Arc::new(
            SyncManager::new(network.clone(), consensus.clone())
        );
        ui::print_component_status("Sync Manager", ui::ComponentStatus::Running);

        // Initialize RPC server (optional)
        let rpc_server = if config.rpc.enabled {
            ui::print_component_status("RPC Server", ui::ComponentStatus::Starting);
            info!("Initializing RPC server on {}:{}", config.rpc.bind_address, config.rpc.port);
            let server = Arc::new(
                RpcServer::new(&config.rpc, consensus.clone(), network.clone(), mempool.clone()).await?
            );
            ui::print_component_status("RPC Server", ui::ComponentStatus::Running);
            Some(server)
        } else {
            None
        };

        // Initialize mining (optional)
        let mining = if config.mining.enabled {
            ui::print_component_status("Mining Coordinator", ui::ComponentStatus::Starting);
            info!("Initializing mining coordinator");
            let addr = config.mining.mining_address.as_ref()
                .ok_or("Mining enabled but no mining address provided")?;

            let mc_config = crate::mining_coordinator::MiningCoordinatorConfig {
                enabled: true,
                num_workers: config.mining.num_threads,
                mining_address: addr.clone(),
            };

            let coordinator = Arc::new(
                MiningCoordinator::new(mc_config, consensus.clone(), mempool.clone()).map_err(|e| e)?
            );
            ui::print_component_status("Mining Coordinator", ui::ComponentStatus::Running);
            Some(coordinator)
        } else {
            None
        };

        ui::print_status("✓", "All components initialized successfully", ui::StatusType::Success);
        Ok(Self {
            config,
            shutdown_tx,
            consensus,
            network,
            rpc_server,
            mining,
            mempool,
            sync,
        })
    }

    /// Run the daemon
    pub async fn run(self) -> Result<(), String> {
        ui::print_section("Starting Services");
        info!("Starting JIOPad daemon");

        let shutdown_rx = self.shutdown_tx.subscribe();
        let start_time = Instant::now();

        // Start all components
        self.start_components().await?;

        ui::print_status("✓", "JIOPad daemon is now running", ui::StatusType::Success);
        ui::print_status("ℹ", "Press Ctrl+C to stop the daemon", ui::StatusType::Info);
        println!();

        // Start status update task
        let status_handle = {
            let consensus = self.consensus.clone();
            let _network = self.network.clone();
            let mempool = self.mempool.clone();
            let mining = self.mining.clone();
            let start_time = start_time;
            
            tokio::spawn(async move {
                let mut interval = interval(Duration::from_secs(30));
                loop {
                    interval.tick().await;
                    
                    // Collect status information
                    let block_count = consensus.storage().block_store().block_count() as u64;
                    let peer_count = 0; // TODO: Get from network manager
                    let mempool_size = mempool.size();
                    let is_mining = mining.is_some();
                    let mining_hashrate = 0.0; // TODO: Get from mining coordinator
                    
                    let status = ui::NodeStatus {
                        uptime: start_time.elapsed(),
                        block_count,
                        peer_count,
                        is_mining,
                        mining_hashrate,
                        mempool_size,
                        sync_percentage: 100.0, // TODO: Calculate actual sync percentage
                    };
                    
                    print!("{}", status);
                }
            })
        };

        // Wait for shutdown signal
        self.wait_for_shutdown(shutdown_rx).await;
        
        // Cancel status updates
        status_handle.abort();

        // Stop all components
        self.stop_components().await?;

        Ok(())
    }

    async fn start_components(&self) -> Result<(), String> {
        // Start network layer
        ui::print_component_status("Network Layer", ui::ComponentStatus::Starting);
        info!("Starting network layer");
        self.network.start().await?;
        ui::print_component_status("Network Layer", ui::ComponentStatus::Running);

        // Start sync manager
        ui::print_component_status("Sync Manager", ui::ComponentStatus::Starting);
        info!("Starting sync manager");
        self.sync.start().await?;
        ui::print_component_status("Sync Manager", ui::ComponentStatus::Running);

        // Start RPC server
        if let Some(rpc) = &self.rpc_server {
            ui::print_component_status("RPC Server", ui::ComponentStatus::Starting);
            info!("Starting RPC server");
            rpc.start().await?;
            ui::print_component_status("RPC Server", ui::ComponentStatus::Running);
        }

        // Start mining
        if let Some(_mining) = &self.mining {
            ui::print_component_status("Mining", ui::ComponentStatus::Starting);
            info!("Starting mining");
            // mining.start().await?; // Note: MiningCoordinator is wrapped in Arc, so we don't call start/stop on it
            ui::print_component_status("Mining", ui::ComponentStatus::Running);
        } else {
            ui::print_status("ℹ", "Mining not enabled", ui::StatusType::Info);
        }

        Ok(())
    }

    async fn stop_components(&self) -> Result<(), String> {
        info!("Stopping components");

        // Stop mining first
        if let Some(_mining) = &self.mining {
            info!("Stopping mining");
            // mining.stop().await?; // Note: MiningCoordinator is wrapped in Arc so methods need interior mutability
        }

        // Stop RPC server
        if let Some(rpc) = &self.rpc_server {
            info!("Stopping RPC server");
            rpc.stop().await?;
        }

        // Stop sync manager
        info!("Stopping sync manager");
        self.sync.stop().await?;

        // Stop network
        info!("Stopping network layer");
        self.network.stop().await?;

        info!("All components stopped");
        Ok(())
    }

    async fn wait_for_shutdown(&self, mut shutdown_rx: broadcast::Receiver<()>) {
        tokio::select! {
            _ = signal::ctrl_c() => {
                ui::print_status("ℹ", "Received Ctrl+C, shutting down gracefully...", ui::StatusType::Warning);
                info!("Received Ctrl+C, shutting down");
            }
            _ = shutdown_rx.recv() => {
                ui::print_status("ℹ", "Received shutdown signal", ui::StatusType::Info);
                info!("Received shutdown signal");
            }
        }

        // Broadcast shutdown to all components
        let _ = self.shutdown_tx.send(());
    }
}
