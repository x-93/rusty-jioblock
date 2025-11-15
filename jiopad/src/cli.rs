use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "jiopad")]
#[command(about = "JIO blockchain full node daemon", long_about = None)]
pub struct Args {
    /// Path to configuration file (optional, uses defaults if not provided)
    #[arg(short, long)]
    pub config_path: Option<PathBuf>,

    /// Data directory
    #[arg(short, long)]
    pub data_dir: Option<PathBuf>,

    /// Network (mainnet, testnet, devnet)
    #[arg(short, long)]
    pub network: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    pub log_level: String,

    /// Enable mining
    #[arg(long)]
    pub enable_mining: bool,

    /// Mining address (required if mining enabled)
    #[arg(long)]
    pub mining_address: Option<String>,

    /// RPC server port
    #[arg(long)]
    pub rpc_port: Option<u16>,

    /// P2P listen port
    #[arg(long)]
    pub p2p_port: Option<u16>,

    /// Bootstrap peers (comma-separated)
    #[arg(long)]
    pub bootstrap_peers: Option<String>,

    /// Disable RPC server
    #[arg(long)]
    pub no_rpc: bool,

    /// Run as archive node (keep full history)
    #[arg(long)]
    pub archive: bool,
}

pub fn parse_args() -> Args {
    Args::parse()
}
