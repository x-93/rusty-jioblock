use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use log::{info, warn};

use consensus_core::{
    block::Block,
    Hash,
};
use consensus::process::mining::BlockTemplate;

// Use consensus header PoW validation so miner and node agree on PoW algorithm
use consensus_core::hashing::header as header_hashing;

/// Configuration for RPC-based miner
#[derive(Clone, Debug)]
pub struct RpcMinerConfig {
    /// Number of worker threads
    pub num_workers: usize,
    /// Mining address for coinbase
    pub mining_address: String,
    /// How often to refresh block template (ms)
    pub template_refresh_interval_ms: u64,
    /// Max iterations per template before checking for new one
    pub max_iterations: u64,
}

/// Mining statistics
#[derive(Clone, Debug)]
pub struct MiningStats {
    pub blocks_mined: u64,
    pub total_hashes: u64,
    pub hash_rate: f64, // H/s
    pub avg_time_per_block_ms: u64,
    pub uptime_ms: u64,
}

/// RPC-based miner that fetches templates and submits solved blocks
pub struct RpcMiner {
    config: RpcMinerConfig,
    current_template: Arc<Mutex<Option<BlockTemplate>>>,
    stats_blocks_mined: Arc<AtomicU64>,
    stats_total_hashes: Arc<AtomicU64>,
    stats_uptime_start: Instant,
    shutdown_flag: Arc<AtomicBool>,
    worker_threads: Vec<thread::JoinHandle<()>>,
    template_thread: Option<thread::JoinHandle<()>>,
    start_time: Option<Instant>,
    last_block_hash: Arc<Mutex<Option<String>>>,
}

impl RpcMiner {
    /// Create new RPC miner with configuration
    pub fn new(config: RpcMinerConfig) -> Self {
        Self {
            config,
            current_template: Arc::new(Mutex::new(None)),
            stats_blocks_mined: Arc::new(AtomicU64::new(0)),
            stats_total_hashes: Arc::new(AtomicU64::new(0)),
            stats_uptime_start: Instant::now(),
            shutdown_flag: Arc::new(AtomicBool::new(false)),
            worker_threads: Vec::new(),
            template_thread: None,
            start_time: None,
            last_block_hash: Arc::new(Mutex::new(None)),
        }
    }

    /// Start mining with provided RPC closure functions
    pub fn start_mining<F, S>(&mut self, get_template: F, submit_block: S)
    where
        F: Fn() -> Result<BlockTemplate, String> + Send + Sync + 'static,
        S: Fn(Block) -> Result<String, String> + Send + Sync + 'static,
    {
        self.start_time = Some(Instant::now());
        
        let get_template = Arc::new(get_template);
        let submit_block = Arc::new(submit_block);

        // Start template fetcher thread
        let template = self.current_template.clone();
        let get_tmpl = get_template.clone();
        let interval = self.config.template_refresh_interval_ms;
        let shutdown = self.shutdown_flag.clone();

        let template_handle = thread::spawn(move || {
            loop {
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }

                match get_tmpl() {
                    Ok(tmpl) => {
                        if let Ok(mut t) = template.lock() {
                            *t = Some(tmpl);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to fetch template: {}", e);
                    }
                }

                thread::sleep(Duration::from_millis(interval));
            }
        });

        self.template_thread = Some(template_handle);

        // Start worker threads
        let last_hash = self.last_block_hash.clone();
        for worker_id in 0..self.config.num_workers {
            let template = self.current_template.clone();
            let submit = submit_block.clone();
            let max_iter = self.config.max_iterations;
            let stats_blocks = self.stats_blocks_mined.clone();
            let stats_hashes = self.stats_total_hashes.clone();
            let shutdown = self.shutdown_flag.clone();

            let _last_hash = last_hash.clone();
            let worker = thread::spawn(move || {
                Self::worker_loop(
                    worker_id,
                    template,
                    submit,
                    max_iter,
                    stats_blocks,
                    stats_hashes,
                    shutdown,
                )
            });

            self.worker_threads.push(worker);
        }

        info!("RPC miner started with {} workers", self.config.num_workers);
    }

    /// Worker thread mining loop
    fn worker_loop(
        worker_id: usize,
        current_template: Arc<Mutex<Option<BlockTemplate>>>,
        submit_block: Arc<impl Fn(Block) -> Result<String, String>>,
        max_iterations: u64,
        blocks_mined: Arc<AtomicU64>,
        total_hashes: Arc<AtomicU64>,
        shutdown: Arc<AtomicBool>,
    ) {
        let mut nonce = worker_id as u64;
        let mut local_hash_count = 0u64;

        while !shutdown.load(Ordering::Relaxed) {
            // Get current template
            let template_opt = match current_template.lock() {
                Ok(guard) => guard.clone(),
                Err(_) => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
            };

            if let Some(template) = template_opt {
                // Mine on this template
                for _ in 0..max_iterations {
                    if shutdown.load(Ordering::Relaxed) {
                        break;
                    }

                    // Create header with nonce
                    let mut header = template.header.clone();
                    header.nonce = nonce;
                    // Recalculate header hash with new nonce
                    header.finalize();
                    nonce = nonce.wrapping_add(1);
                    local_hash_count += 1;

                    // Check PoW using consensus header hashing to ensure miner/validator parity
                    if header_hashing::validate_pow(&header) {
                        // Found valid block!
                        let block = Block::new(header.clone(), template.transactions.clone());
                        
                        // Log the actual block hash for debugging
                        log::info!(
                            "Worker {} found valid block with hash: {}, nonce: {}",
                            worker_id,
                            block.header.hash,
                            header.nonce
                        );

                        match submit_block(block) {
                            Ok(_hash_str) => {
                                info!("Worker {} mined block and submitted", worker_id);
                                blocks_mined.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(e) => {
                                warn!("Worker {} failed to submit block: {}", worker_id, e);
                            }
                        }
                    }

                    // Update stats periodically
                    if local_hash_count % 10000 == 0 {
                        total_hashes.fetch_add(local_hash_count, Ordering::Relaxed);
                        local_hash_count = 0;
                    }
                }

                // Yield to template refresh thread
                thread::sleep(Duration::from_millis(1));
            } else {
                // No template yet, wait
                thread::sleep(Duration::from_millis(100));
            }
        }

        // Final update of stats
        if local_hash_count > 0 {
            total_hashes.fetch_add(local_hash_count, Ordering::Relaxed);
        }
    }

    /// Get current mining statistics
    pub fn get_stats(&self) -> MiningStats {
        let uptime_ms = self.stats_uptime_start.elapsed().as_millis() as u64;
        let blocks_mined = self.stats_blocks_mined.load(Ordering::Relaxed);
        let total_hashes = self.stats_total_hashes.load(Ordering::Relaxed);

        let hash_rate = if uptime_ms > 0 {
            (total_hashes as f64) / (uptime_ms as f64 / 1000.0)
        } else {
            0.0
        };

        let avg_time_per_block_ms = if blocks_mined > 0 {
            uptime_ms / blocks_mined
        } else {
            0
        };

        MiningStats {
            blocks_mined,
            total_hashes,
            hash_rate,
            avg_time_per_block_ms,
            uptime_ms,
        }
    }

    /// Gracefully shutdown the miner
    pub fn shutdown(&mut self) {
        info!("Shutting down RPC miner...");
        self.shutdown_flag.store(true, Ordering::Relaxed);

        // Wait for template thread
        if let Some(handle) = self.template_thread.take() {
            let _ = handle.join();
        }

        // Wait for all worker threads
        for handle in self.worker_threads.drain(..) {
            let _ = handle.join();
        }

        let final_stats = self.get_stats();
        info!("Miner shutdown - Blocks: {}, Total hashes: {}, Hash rate: {:.2} H/s, Uptime: {}ms",
            final_stats.blocks_mined,
            final_stats.total_hashes,
            final_stats.hash_rate,
            final_stats.uptime_ms
        );
    }
}

impl Drop for RpcMiner {
    fn drop(&mut self) {
        if !self.shutdown_flag.load(Ordering::Relaxed) {
            self.shutdown();
        }
    }
}
