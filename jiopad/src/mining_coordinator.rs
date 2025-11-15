//! Mining Coordinator - manages the mining process
//!
//! This module coordinates mining operations by managing mining threads,
//! distributing block templates, and collecting mined blocks.

use crate::consensus_manager::ConsensusManager;
use crate::mempool::Mempool;
use mining::prelude::*;
use rpc_core::model::BlockTemplate;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tracing::info;

/// Mining coordinator configuration
#[derive(Clone, Debug)]
pub struct MiningCoordinatorConfig {
    pub enabled: bool,
    pub num_workers: usize,
    pub mining_address: String,
}

impl Default for MiningCoordinatorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            num_workers: num_cpus::get(),
            mining_address: String::new(),
        }
    }
}

/// Manages the mining process
pub struct MiningCoordinator {
    config: MiningCoordinatorConfig,
    consensus: Arc<ConsensusManager>,
    mempool: Arc<Mempool>,
    mining_manager: Arc<Mutex<Option<MiningManager>>>,
    is_running: Arc<Mutex<bool>>,
}

impl MiningCoordinator {
    /// Creates a new mining coordinator
    pub fn new(
        config: MiningCoordinatorConfig,
        consensus: Arc<ConsensusManager>,
        mempool: Arc<Mempool>,
    ) -> Result<Self, String> {
        Ok(Self {
            config,
            consensus,
            mempool,
            mining_manager: Arc::new(Mutex::new(None)),
            is_running: Arc::new(Mutex::new(false)),
        })
    }

    /// Starts the mining coordinator
    pub async fn start(&mut self) -> Result<(), String> {
        if !self.config.enabled {
            info!("Mining is disabled");
            return Ok(());
        }

        info!("Mining coordinator starting with {} workers", self.config.num_workers);

        // Create mining configuration
        let mining_config = MiningConfig {
            num_workers: self.config.num_workers,
            job_max_age_ms: 30_000,
        };

        // Create and start mining manager
        let mut manager = MiningManager::new(mining_config);
        manager.start();

        *self.mining_manager.lock().unwrap() = Some(manager);
        *self.is_running.lock().unwrap() = true;

        info!("Mining coordinator started");
        Ok(())
    }

    /// Stops the mining coordinator
    pub async fn stop(&mut self) -> Result<(), String> {
        *self.is_running.lock().unwrap() = false;
        *self.mining_manager.lock().unwrap() = None;
        info!("Mining coordinator stopped");
        Ok(())
    }

    /// Updates the mining job with a new block template
    pub fn update_job(&self, template: BlockTemplate) -> Result<(), String> {
        if let Ok(manager_lock) = self.mining_manager.lock() {
            if let Some(manager) = manager_lock.as_ref() {
                manager.update_job(template);
                return Ok(());
            }
        }
        Err("Mining manager not initialized".to_string())
    }

    /// Collects and processes mined blocks
    pub fn collect_mined_blocks(&self) -> Result<Vec<MiningResult>, String> {
        if let Ok(manager_lock) = self.mining_manager.lock() {
            if let Some(manager) = manager_lock.as_ref() {
                let results = manager.collect_results();
                return Ok(results);
            }
        }
        Ok(Vec::new())
    }

    /// Gets the current mining session statistics
    pub fn get_mining_stats(&self) -> Result<SessionStats, String> {
        if let Ok(manager_lock) = self.mining_manager.lock() {
            if let Some(manager) = manager_lock.as_ref() {
                return Ok(manager.get_session_stats());
            }
        }
        Err("Mining manager not initialized".to_string())
    }

    /// Gets the current mining status
    pub fn is_mining(&self) -> bool {
        *self.is_running.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Gets the mining address
    pub fn mining_address(&self) -> &str {
        &self.config.mining_address
    }

    /// Gets the number of mining workers
    pub fn worker_count(&self) -> usize {
        self.config.num_workers
    }
}

