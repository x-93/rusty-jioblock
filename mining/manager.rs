//! Mining manager that coordinates workers and job distribution
//!
//! This module manages the lifecycle of mining workers, distributes mining jobs,
//! collects results, and coordinates difficulty adjustments.

use crate::difficulty::DifficultyManager;
use crate::job::{MinedBlock, MiningJob};
use crate::pow::Target;
use crate::worker::{MinerWorker, WorkerStats};
use consensus_core::block::Block;
use consensus_core::Hash;
use log;
use rpc_core::model::BlockTemplate;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::Instant;

/// Configuration for the mining manager
#[derive(Clone, Debug)]
pub struct MiningConfig {
    /// Number of worker threads
    pub num_workers: usize,
    /// Maximum time for a mining job before requesting new one (milliseconds)
    pub job_max_age_ms: u64,
}

impl Default for MiningConfig {
    fn default() -> Self {
        Self {
            num_workers: num_cpus::get(),
            job_max_age_ms: 30_000,
        }
    }
}

/// Result of mining a block
#[derive(Clone, Debug)]
pub struct MiningResult {
    pub job_id: u64,
    pub nonce: u64,
    pub block_hash: Hash,
    pub worker_id: usize,
    pub time_ms: u64,
    pub hash_rate: f64,
}

/// Manages mining operations
pub struct MiningManager {
    config: MiningConfig,
    /// Currently active mining job
    current_job: Arc<Mutex<Option<MiningJob>>>,
    /// Channels to send jobs to workers
    job_senders: Vec<Sender<MiningJob>>,
    /// Channel to receive mined blocks (mutex-wrapped so the manager is Sync)
    result_rx: Mutex<Receiver<MinedBlock>>,
    result_tx: Sender<MinedBlock>,
    /// Worker thread handles
    worker_threads: Vec<JoinHandle<()>>,
    /// Shutdown signal
    shutdown: Arc<AtomicBool>,
    /// Worker statistics
    worker_stats: Arc<Mutex<Vec<WorkerStats>>>,
    /// Difficulty manager
    difficulty_manager: DifficultyManager,
    /// Start time of mining session
    session_start: Instant,
}

impl MiningManager {
    /// Creates a new mining manager
    pub fn new(config: MiningConfig) -> Self {
    let (result_tx, result_rx) = mpsc::channel();
        let shutdown = Arc::new(AtomicBool::new(false));
        let mut job_senders = Vec::new();
        let mut worker_threads = Vec::new();

        // Initialize worker statistics
        let mut initial_stats = Vec::new();
        for _i in 0..config.num_workers {
            initial_stats.push(WorkerStats::default());
        }

        // Spawn worker threads
        for worker_id in 0..config.num_workers {
            let (job_tx, job_rx) = mpsc::channel();
            let result_tx_clone = result_tx.clone();
            let shutdown_clone = Arc::clone(&shutdown);

            let handle = thread::spawn(move || {
                let mut worker =
                    MinerWorker::new(worker_id, job_rx, result_tx_clone, shutdown_clone);
                worker.run();
            });

            job_senders.push(job_tx);
            worker_threads.push(handle);
        }

        Self {
            config,
            current_job: Arc::new(Mutex::new(None)),
            job_senders,
            result_rx: Mutex::new(result_rx),
            result_tx,
            worker_threads,
            shutdown,
            worker_stats: Arc::new(Mutex::new(initial_stats)),
            difficulty_manager: DifficultyManager::new(),
            session_start: Instant::now(),
        }
    }

    /// Starts the mining manager
    pub fn start(&mut self) {
        log::info!(
            "Mining manager started with {} workers",
            self.config.num_workers
        );
    }

    /// Updates the mining job for all workers
    pub fn update_job(&self, template: BlockTemplate) {
        let target = self
            .difficulty_manager
            .get_current_target()
            .unwrap_or_else(|| Target::from_bits(0x207fffff));

        let job = MiningJob::new(template, target);
        let job_id = job.job_id;

        log::debug!("Updating mining job {} across all workers", job_id);

        // Store as current job
        *self.current_job.lock().unwrap() = Some(job.clone());

        // Distribute job to all workers
        for (i, sender) in self.job_senders.iter().enumerate() {
            if let Err(e) = sender.send(job.clone()) {
                log::warn!("Failed to send job to worker {}: {}", i, e);
            }
        }
    }

    /// Submits a mined block (placeholder for integration with node)
    pub fn submit_block(&self, block: Block) {
        log::info!("Submitting mined block: {:?}", block.header);
        // This would integrate with the node's block acceptance logic
    }

    /// Collects mining results (non-blocking)
    pub fn collect_results(&self) -> Vec<MiningResult> {
        let mut results = Vec::new();
        loop {
            match self.result_rx.lock().unwrap().try_recv() {
                Ok(mined_block) => {
                    let result = MiningResult {
                        job_id: mined_block.job_id,
                        nonce: mined_block.nonce,
                        block_hash: mined_block.block_hash,
                        worker_id: mined_block.worker_id,
                        time_ms: mined_block.time_ms,
                        hash_rate: mined_block.hash_rate(),
                    };

                    // Update worker stats
                    if let Ok(mut stats) = self.worker_stats.lock() {
                        if mined_block.worker_id < stats.len() {
                            stats[mined_block.worker_id].update(&mined_block);
                        }
                    }

                    results.push(result);
                }
                Err(_) => break,
            }
        }

        results
    }

    /// Gets the current statistics for all workers
    pub fn get_worker_stats(&self) -> Vec<WorkerStats> {
        self.worker_stats
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Gets overall mining session statistics
    pub fn get_session_stats(&self) -> SessionStats {
        let stats = self.get_worker_stats();
        let session_duration_ms = self.session_start.elapsed().as_millis() as u64;

        let total_blocks = stats.iter().map(|s| s.blocks_mined).sum();
        let total_iterations = stats.iter().map(|s| s.total_iterations).sum();
        let total_time_ms = stats.iter().map(|s| s.total_time_ms).sum::<u64>();

        let overall_hash_rate = if session_duration_ms > 0 {
            (total_iterations as f64) / (session_duration_ms as f64 / 1000.0)
        } else {
            0.0
        };

        SessionStats {
            session_duration_ms,
            total_blocks,
            total_iterations,
            total_time_ms,
            overall_hash_rate,
            worker_count: self.config.num_workers,
            worker_stats: stats,
        }
    }

    /// Stops the mining manager and waits for workers to finish
    pub fn stop(&mut self) {
        log::info!("Stopping mining manager");

        // Signal shutdown
        self.shutdown.store(true, Ordering::Relaxed);

        // Wait for workers to finish
        for handle in self.worker_threads.drain(..) {
            match handle.join() {
                Ok(_) => log::debug!("Worker thread stopped gracefully"),
                Err(e) => log::error!("Worker thread panicked: {:?}", e),
            }
        }

        log::info!("Mining manager stopped");
    }

    /// Gets current job reference
    pub fn current_job(&self) -> Option<MiningJob> {
        self.current_job.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Returns the number of active workers
    pub fn worker_count(&self) -> usize {
        self.config.num_workers
    }
}

impl Drop for MiningManager {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Statistics for a mining session
#[derive(Clone, Debug)]
pub struct SessionStats {
    pub session_duration_ms: u64,
    pub total_blocks: u64,
    pub total_iterations: u64,
    pub total_time_ms: u64,
    pub overall_hash_rate: f64,
    pub worker_count: usize,
    pub worker_stats: Vec<WorkerStats>,
}

impl SessionStats {
    /// Formats session stats as a readable string
    pub fn format_summary(&self) -> String {
        format!(
            "Mining Session Stats:\n  Duration: {}ms\n  Total Blocks: {}\n  Total Iterations: {}\n  \
             Workers: {}\n  Overall Hash Rate: {:.2} MH/s",
            self.session_duration_ms,
            self.total_blocks,
            self.total_iterations,
            self.worker_count,
            self.overall_hash_rate / 1_000_000.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_manager_creation() {
        let config = MiningConfig {
            num_workers: 2,
            job_max_age_ms: 30_000,
        };
        let manager = MiningManager::new(config);
        assert_eq!(manager.worker_count(), 2);
    }

    #[test]
    fn test_mining_config_default() {
        let config = MiningConfig::default();
        assert!(config.num_workers > 0);
        assert_eq!(config.job_max_age_ms, 30_000);
    }

    #[test]
    fn test_session_stats_formatting() {
        let stats = SessionStats {
            session_duration_ms: 1000,
            total_blocks: 10,
            total_iterations: 10_000_000,
            total_time_ms: 5000,
            overall_hash_rate: 2_000_000.0,
            worker_count: 4,
            worker_stats: vec![],
        };
        let summary = stats.format_summary();
        assert!(summary.contains("Mining Session Stats"));
    }
}
