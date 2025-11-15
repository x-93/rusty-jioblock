use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use consensus_core::config::genesis as core_genesis;
use hex::encode as hex_encode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub network: NetworkConfig,
    pub consensus: ConsensusConfig,
    pub storage: StorageConfig,
    pub rpc: RpcConfig,
    pub mining: MiningConfig,
    pub p2p: P2PConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub network_id: String,
    pub genesis_hash: String,
    pub genesis_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub ghostdag_k: u32,
    pub max_block_parents: usize,
    pub target_time_per_block: u64,
    pub difficulty_window_size: u64,
    pub max_block_size: u64,
    pub coinbase_maturity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub db_cache_size: usize,
    pub enable_pruning: bool,
    pub pruning_depth: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    pub enabled: bool,
    pub bind_address: String,
    pub port: u16,
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningConfig {
    pub enabled: bool,
    pub mining_address: Option<String>,
    pub num_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    pub listen_address: String,
    pub port: u16,
    pub max_peers: usize,
    pub bootstrap_peers: Vec<String>,
    pub enable_upnp: bool,
}

impl Config {
    /// Load configuration from file if it exists, otherwise use defaults
    pub fn load(path: &Path) -> Result<Self, String> {
        // Try to load from file, but fall back to defaults if file doesn't exist
        if path.exists() {
            let content = fs::read_to_string(path)
                .map_err(|e| format!("Failed to read config file: {}", e))?;

            let config: Config = toml::from_str(&content)
                .map_err(|e| format!("Failed to parse config: {}", e))?;

            Ok(config)
        } else {
            // Use defaults if file not found
            Ok(Config::default())
        }
    }

    /// Load default configuration for network
    pub fn for_network(network: &str) -> Result<Self, String> {
        let mut config = Config::default();
        
        match network {
            "mainnet" => {
                config.network.network_id = "mainnet".to_string();
            }
            "testnet" => {
                config.network.network_id = "testnet".to_string();
            }
            "devnet" => {
                config.network.network_id = "devnet".to_string();
            }
            _ => return Err(format!("Unknown network: {}", network)),
        }

        Ok(config)
    }

    /// Override config with CLI arguments
    pub fn apply_cli_overrides(&mut self, args: &crate::cli::Args) {
        if let Some(data_dir) = &args.data_dir {
            self.storage.data_dir = data_dir.clone();
        }

        if let Some(rpc_port) = args.rpc_port {
            self.rpc.port = rpc_port;
        }

        if let Some(p2p_port) = args.p2p_port {
            self.p2p.port = p2p_port;
        }

        if args.no_rpc {
            self.rpc.enabled = false;
        }

        if args.enable_mining {
            self.mining.enabled = true;
            self.mining.mining_address = args.mining_address.clone();
        }

        if let Some(peers) = &args.bootstrap_peers {
            self.p2p.bootstrap_peers = peers.split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        // Compute deterministic genesis hash from consensus core default genesis so config matches runtime
        let genesis = core_genesis::default_genesis();
        let genesis_hash_hex = hex_encode(genesis.hash.as_bytes());

        Self {
            network: NetworkConfig {
                network_id: "mainnet".to_string(),
                genesis_hash: genesis_hash_hex,
                genesis_timestamp: genesis.timestamp,
            },
            consensus: ConsensusConfig {
                ghostdag_k: 18,
                max_block_parents: 10,
                target_time_per_block: 1,
                difficulty_window_size: 2641,
                max_block_size: 1_000_000,
                coinbase_maturity: 100,
            },
            storage: StorageConfig {
                data_dir: PathBuf::from("./data"),
                db_cache_size: 512 * 1024 * 1024, // 512 MB
                enable_pruning: false,
                pruning_depth: 10000,
            },
            rpc: RpcConfig {
                enabled: true,
                bind_address: "127.0.0.1".to_string(),
                port: 16110,
                max_connections: 100,
            },
            mining: MiningConfig {
                enabled: false,
                mining_address: None,
                num_threads: 1,
            },
            p2p: P2PConfig {
                listen_address: "0.0.0.0".to_string(),
                port: 16111,
                max_peers: 50,
                bootstrap_peers: vec![],
                enable_upnp: true,
            },
        }
    }
}
