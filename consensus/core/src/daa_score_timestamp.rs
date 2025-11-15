use serde::{Deserialize, Serialize};

/// DAA score and timestamp pair
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DaaScoreTimestamp {
    /// The DAA score of the block
    pub daa_score: u64,
    /// The timestamp of the block
    pub timestamp: u64,
}

impl DaaScoreTimestamp {
    /// Creates a new DaaScoreTimestamp
    pub fn new(daa_score: u64, timestamp: u64) -> Self {
        Self {
            daa_score,
            timestamp,
        }
    }
}