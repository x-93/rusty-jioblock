use serde::{Deserialize, Serialize};

/// Configuration for blocks per second (BPS) in consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpsConfig {
    /// Target number of blocks per second
    pub target_blocks_per_second: f64,
    /// Minimum blocks per second
    pub min_blocks_per_second: f64,
    /// Maximum blocks per second  
    pub max_blocks_per_second: f64,
}