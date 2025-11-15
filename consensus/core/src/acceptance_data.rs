use crate::header::Header;
use crate::Hash;
use serde::{Deserialize, Serialize};

/// AcceptanceData contains block acceptance data used in various consensus validations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceData {
    /// The header of the accepted block
    pub header: Header,
    /// Hash of the block
    pub hash: Hash,
    /// Block blue score
    pub blue_score: u64,
    /// Acceptance index in DAG
    pub acceptance_index: u64,
}