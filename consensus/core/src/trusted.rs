use crypto_hashes::Hash;
use serde::{Deserialize, Serialize};

/// Trusted data configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedData {
    /// List of trusted peer IDs
    pub trusted_peers: Vec<String>,
    /// List of trusted block hashes
    pub trusted_blocks: Vec<Hash>,
}

impl TrustedData {
    /// Creates a new empty trusted data set
    pub fn new() -> Self {
        Self {
            trusted_peers: Vec::new(),
            trusted_blocks: Vec::new(),
        }
    }

    /// Adds a trusted peer
    pub fn add_trusted_peer(&mut self, peer_id: String) {
        if !self.trusted_peers.contains(&peer_id) {
            self.trusted_peers.push(peer_id);
        }
    }

    /// Adds a trusted block hash
    pub fn add_trusted_block(&mut self, block_hash: Hash) {
        if !self.trusted_blocks.contains(&block_hash) {
            self.trusted_blocks.push(block_hash);
        }
    }
}