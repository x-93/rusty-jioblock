use crate::Hash;
use serde::{Deserialize, Serialize};

/// API types for consensus operations
pub mod consensus {
    use super::*;

    /// Block validation result
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ValidationResult {
        /// Whether the block is valid
        pub is_valid: bool,
        /// Validation error message if any
        pub error: Option<String>,
    }

    /// Block submission result
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SubmitBlockResult {
        /// Block hash
        pub block_hash: Hash,
        /// Whether submission was successful
        pub success: bool,
        /// Error message if submission failed
        pub error: Option<String>,
    }
}