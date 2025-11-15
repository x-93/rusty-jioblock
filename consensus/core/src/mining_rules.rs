use serde::{Deserialize, Serialize};

/// Rules for mining new blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningRules {
    /// Whether ASIC mining is enabled
    pub enable_asic: bool,
    /// Minimum hash rate in H/s
    pub min_hash_rate: u64,
    /// Maximum number of mining threads
    pub max_threads: u32,
}