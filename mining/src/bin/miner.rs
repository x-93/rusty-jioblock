use clap::Parser;
use mining::prelude::*;
use log::{info, warn};
use tungstenite::{connect, Message};
use url::Url;
use serde_json::json;
use serde_json::Value;
use hex;
use bincode;

use consensus::process::mining::BlockTemplate as ConsensusBlockTemplate;
use consensus_core::header::Header;
use consensus_core::block::Block;
use consensus_core::Hash;
use consensus_core::merkle::MerkleTree;

/// RPC-based blockchain miner
#[derive(Parser, Debug)]
#[command(name = "jio-miner")]
#[command(about = "RPC-based miner for Jio blockchain", long_about = None)]
struct Args {
    /// RPC server address (host:port)
    #[arg(short, long, default_value = "127.0.0.1:16110")]
    rpc_addr: String,

    /// Mining address (coinbase recipient)
    #[arg(short, long)]
    mining_address: Option<String>,

    /// Number of worker threads
    #[arg(short, long)]
    workers: Option<usize>,

    /// Block template refresh interval (milliseconds)
    #[arg(long, default_value = "5000")]
    template_refresh_ms: u64,

    /// Max iterations per template
    #[arg(long, default_value = "1000000000")]
    max_iterations: u64,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(args.log_level.parse()?)
        .init();

    info!("Jio RPC Miner starting...");
    info!("RPC Address: {}", args.rpc_addr);

    // Create miner configuration
    let config = RpcMinerConfig {
        num_workers: args.workers.unwrap_or_else(num_cpus::get),
        mining_address: args.mining_address.unwrap_or_else(|| "1A1z7agoat3FwzZsQwtfTHtVtWWbnSFAZa".to_string()),
        template_refresh_interval_ms: args.template_refresh_ms,
        max_iterations: args.max_iterations,
    };

    info!("Miner config: {} workers, address: {}", config.num_workers, config.mining_address);

    // Create miner
    let mut miner = RpcMiner::new(config);

    // RPC client helpers (synchronous WebSocket JSON-RPC)
    let rpc_addr = args.rpc_addr.clone();

    let get_template = move || -> Result<ConsensusBlockTemplate, String> {
        let url = format!("ws://{}", rpc_addr);
        let url_parsed = Url::parse(&url).map_err(|e| e.to_string())?;
        let (mut socket, _response) = connect(url_parsed).map_err(|e| format!("WS connect error: {}", e))?;

        let req = json!({ "jsonrpc": "2.0", "id": 1, "method": "getBlockTemplate", "params": null });
        socket.write_message(Message::Text(req.to_string())).map_err(|e| e.to_string())?;

        // Read response
        loop {
            let msg = socket.read_message().map_err(|e| e.to_string())?;
            if let Message::Text(txt) = msg {
                let v: Value = serde_json::from_str(&txt).map_err(|e| e.to_string())?;
                if let Some(result) = v.get("result") {
                    // Parse into rpc_core::model::BlockTemplate then convert to consensus BlockTemplate
                    let rpc_tmpl: rpc_core::model::BlockTemplate = serde_json::from_value(result.clone()).map_err(|e| e.to_string())?;
                    // Map rpc_core::model::BlockTemplate -> consensus::process::mining::BlockTemplate
                    // Note: rpc_tmpl.transactions are full Transaction objects; reuse them
                    // Compute merkle root using consensus merkle routine from transaction hashes
                    let tx_hashes: Vec<Hash> = rpc_tmpl.transactions.iter().map(|tx| tx.hash()).collect();
                    let merkle_root = if tx_hashes.is_empty() {
                        Default::default()
                    } else {
                        MerkleTree::from_hashes(tx_hashes).root()
                    };

                    let header = Header::new_finalized(
                        rpc_tmpl.version as u16,
                        vec![rpc_tmpl.parent_hashes.clone()],
                        merkle_root,
                        Default::default(),
                        Default::default(),
                        rpc_tmpl.timestamp,
                        rpc_tmpl.bits,
                        0,
                        0,
                        0.into(),
                        0,
                        Default::default(),
                    );

                    let tmpl = ConsensusBlockTemplate {
                        header,
                        transactions: rpc_tmpl.transactions,
                        coinbase_reward: rpc_tmpl.coinbase_value,
                    };

                    return Ok(tmpl);
                }
            }
        }
    };

    let submit_addr = args.rpc_addr.clone();
    let submit_block = move |block: Block| -> Result<String, String> {
        // Serialize block with bincode and send as hex via submitBlockHex
        let bytes = bincode::serialize(&block).map_err(|e| e.to_string())?;
        let hex_str = hex::encode(bytes);

        let url = format!("ws://{}", submit_addr);
        let url_parsed = Url::parse(&url).map_err(|e| e.to_string())?;
        let (mut socket, _response) = connect(url_parsed).map_err(|e| format!("WS connect error: {}", e))?;

        let req = json!({ "jsonrpc": "2.0", "id": 1, "method": "submitBlockHex", "params": { "blockHex": hex_str } });
        socket.write_message(Message::Text(req.to_string())).map_err(|e| e.to_string())?;

        loop {
            let msg = socket.read_message().map_err(|e| e.to_string())?;
            if let Message::Text(txt) = msg {
                let v: Value = serde_json::from_str(&txt).map_err(|e| e.to_string())?;
                if let Some(result) = v.get("result") {
                    // result is a hash string, return as-is
                    let hash_str = result.as_str().ok_or("Invalid hash result")?;
                    return Ok(hash_str.to_string());
                }
            }
        }
    };

    // Start mining
    miner.start_mining(get_template, submit_block);

    // Monitor mining statistics every 10 seconds
    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
        let stats = miner.get_stats();
        info!(
            "Mining stats - Blocks: {}, Hash rate: {:.2} H/s, Avg time/block: {} ms, Uptime: {} s",
            stats.blocks_mined,
            stats.hash_rate,
            stats.avg_time_per_block_ms,
            stats.uptime_ms / 1000
        );
    }
}
