use serde::{Deserialize, Serialize};
use std::fmt;

/// Network type identifies the network a node is operating on
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkType {
    /// Main network
    Mainnet,
    /// Test network
    Testnet,
    /// Development network
    Devnet,
    /// Simnet for testing
    Simnet,
}

impl fmt::Display for NetworkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkType::Mainnet => write!(f, "mainnet"),
            NetworkType::Testnet => write!(f, "testnet"),
            NetworkType::Devnet => write!(f, "devnet"),
            NetworkType::Simnet => write!(f, "simnet"),
        }
    }
}

impl NetworkType {
    /// Returns an iterator over all NetworkType variants
    pub fn iter() -> impl Iterator<Item = NetworkType> {
        [
            NetworkType::Mainnet,
            NetworkType::Testnet,
            NetworkType::Devnet,
            NetworkType::Simnet,
        ]
        .into_iter()
    }
}