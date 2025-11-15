use serde::{Deserialize, Serialize};

/// Network constants configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConstants {
    /// Number of target timespan blocks
    pub target_timespan_blocks: u32,
    /// Target time per block in seconds
    pub target_time_per_block: u32,
    /// Maximum target value for difficulty
    pub max_target: [u8; 32],
}