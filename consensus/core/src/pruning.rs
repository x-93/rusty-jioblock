use serde::{Deserialize, Serialize};

/// Pruning point configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruningConfig {
    /// Number of blocks to keep before pruning point
    pub pruning_depth: u64,
    /// Interval between pruning operations
    pub pruning_interval: u64,
}