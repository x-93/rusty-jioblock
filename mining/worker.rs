//! Mining worker thread implementation
//!
//! This module implements the MinerWorker that runs in separate threads
//! and performs proof-of-work iterations on mining jobs.

use crate::job::{MinedBlock, MiningJob};
use crate::pow::ProofOfWork;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

/// A mining worker that processes jobs in a separate thread
#[derive(Debug)]
pub struct MinerWorker {
    /// Unique identifier for this worker
    pub id: usize,
    /// Receives mining jobs from the manager
    pub job_rx: Receiver<MiningJob>,
    /// Sends mined blocks back to the manager
    pub result_tx: Sender<MinedBlock>,
    /// Shared flag to signal shutdown
    pub shutdown: Arc<AtomicBool>,
}

impl MinerWorker {
    /// Creates a new mining worker
    pub fn new(
        id: usize,
        job_rx: Receiver<MiningJob>,
        result_tx: Sender<MinedBlock>,
        shutdown: Arc<AtomicBool>,
    ) -> Self {
        Self {
            id,
            job_rx,
            result_tx,
            shutdown,
        }
    }

    /// Runs the mining loop (blocking, should be run in a thread)
    pub fn run(&mut self) {
        log::info!("Worker {} started", self.id);

        loop {
            // Check shutdown flag
            if self.shutdown.load(Ordering::Relaxed) {
                log::info!("Worker {} shutting down", self.id);
                break;
            }

            // Try to receive a job with timeout
            match self.job_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(job) => {
                    log::debug!("Worker {} received job {}", self.id, job.job_id);
                    self.mine_job(&job);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Timeout is normal, just loop and check shutdown flag
                    continue;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    // Channel disconnected, shutdown worker
                    log::info!("Worker {} job channel disconnected", self.id);
                    break;
                }
            }
        }

        log::info!("Worker {} stopped", self.id);
    }

    /// Processes a single mining job
    fn mine_job(&mut self, job: &MiningJob) {
        let start_time = Instant::now();
        let mut nonce: u64 = 0;
        let mut iterations: u64 = 0;

        loop {
            // Check shutdown flag periodically
            if iterations % 1000 == 0 && self.shutdown.load(Ordering::Relaxed) {
                log::debug!("Worker {} interrupted mining job {}", self.id, job.job_id);
                return;
            }

            // Get header bytes with current nonce
            let header_bytes = job.header_with_nonce(nonce);

            // Check if this nonce produces valid PoW
            if ProofOfWork::is_valid_pow(&header_bytes, &job.target) {
                let time_ms = start_time.elapsed().as_millis() as u64;

                // Compute the final hash for the result
                let block_hash = ProofOfWork::compute_hash(&header_bytes);

                let mined_block = MinedBlock::new(
                    job.job_id,
                    self.id,
                    nonce,
                    block_hash,
                    iterations,
                    time_ms,
                );

                log::info!(
                    "Worker {} found block for job {} with nonce {} after {} iterations in {}ms (hash rate: {:.2} MH/s)",
                    self.id,
                    job.job_id,
                    nonce,
                    iterations,
                    time_ms,
                    mined_block.hash_rate() / 1_000_000.0
                );

                // Send result back to manager
                if let Err(e) = self.result_tx.send(mined_block) {
                    log::error!("Worker {} failed to send mined block: {}", self.id, e);
                    return;
                }

                return;
            }

            nonce = nonce.wrapping_add(1);
            iterations += 1;

            // Reset nonce if we've tried all values (very unlikely)
            if nonce == 0 {
                log::warn!("Worker {} wrapped nonce counter, restarting from 0", self.id);
            }
        }
    }
}

/// Statistics for a mining worker session
#[derive(Debug, Clone, Default)]
pub struct WorkerStats {
    /// Total blocks mined by this worker
    pub blocks_mined: u64,
    /// Total iterations performed
    pub total_iterations: u64,
    /// Total mining time in milliseconds
    pub total_time_ms: u64,
    /// Average hash rate in hashes per second
    pub average_hash_rate: f64,
}

impl WorkerStats {
    /// Updates statistics with a mined block
    pub fn update(&mut self, mined_block: &MinedBlock) {
        self.blocks_mined += 1;
        self.total_iterations += mined_block.iterations;
        self.total_time_ms += mined_block.time_ms;

        if self.total_time_ms > 0 {
            self.average_hash_rate = (self.total_iterations as f64) / (self.total_time_ms as f64 / 1000.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pow::Target;
    use consensus_core::Hash;
    use rpc_core::model::BlockTemplate;
    use std::sync::mpsc;

    fn create_test_job() -> MiningJob {
        let template = BlockTemplate {
            version: 1,
            parent_hashes: vec![Hash::default()],
            transactions: Vec::new(),
            coinbase_value: 5_000_000_000,
            bits: 0x207fffff,
            timestamp: 1000,
            pay_address: "test".to_string(),
            target: "0".to_string(),
        };
        MiningJob::new(template, Target::from_bits(0x207fffff))
    }

    #[test]
    fn test_worker_creation() {
        let (tx, _rx) = mpsc::channel();
        let (_job_tx, job_rx) = mpsc::channel();
        let shutdown = Arc::new(AtomicBool::new(false));

        let worker = MinerWorker::new(0, job_rx, tx, shutdown);
        assert_eq!(worker.id, 0);
    }

    #[test]
    fn test_worker_stats_update() {
        let mut stats = WorkerStats::default();
        let mined_block = MinedBlock::new(1, 0, 42, Hash::default(), 1_000_000, 1000);

        stats.update(&mined_block);
        assert_eq!(stats.blocks_mined, 1);
        assert_eq!(stats.total_iterations, 1_000_000);
        assert_eq!(stats.total_time_ms, 1000);
        assert!(stats.average_hash_rate > 900_000.0);
    }
}
