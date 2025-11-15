use borsh::{BorshDeserialize, BorshSerialize};
use crate::Hash;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Size of a subnetwork ID in bytes
pub const SUBNETWORK_ID_SIZE: usize = 20;

/// Subnetwork ID for coinbase transactions
pub const SUBNETWORK_ID_COINBASE: SubnetworkId = SubnetworkId([0; SUBNETWORK_ID_SIZE]);

/// Represents a unique identifier for a subnetwork
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SubnetworkId([u8; SUBNETWORK_ID_SIZE]);

impl SubnetworkId {
    /// Creates a new SubnetworkId from raw bytes
    pub fn new(bytes: [u8; SUBNETWORK_ID_SIZE]) -> Self {
        Self(bytes)
    }

    /// Returns the underlying bytes
    pub fn as_bytes(&self) -> &[u8; SUBNETWORK_ID_SIZE] {
        &self.0
    }
}

impl Default for SubnetworkId {
    fn default() -> Self {
        SubnetworkId([0u8; SUBNETWORK_ID_SIZE])
    }
}

impl From<u64> for SubnetworkId {
    fn from(v: u64) -> Self {
        let mut bytes = [0u8; SUBNETWORK_ID_SIZE];
        let le = v.to_le_bytes();
        let copy_len = std::cmp::min(le.len(), SUBNETWORK_ID_SIZE);
        bytes[0..copy_len].copy_from_slice(&le[0..copy_len]);
        SubnetworkId(bytes)
    }
}

impl fmt::Display for SubnetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

/// Subnet configuration
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SubnetConfig {
    /// Gas limit for this subnet
    pub gas_limit: u64,
    /// Chain ID for this subnet
    pub chain_id: u64,
    /// Parent subnet hash if this is a child subnet
    pub parent_subnet: Option<Hash>,
}