use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// The size of a MuHash in bytes
pub const MUHASH_SIZE: usize = 32;

/// Represents a empty MuHash value
pub const EMPTY_MUHASH: MuHash = MuHash([0; MUHASH_SIZE]);

/// MuHash implementation for efficient set membership verification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct MuHash([u8; MUHASH_SIZE]);

impl MuHash {
    /// Creates a new MuHash from bytes
    pub fn new(bytes: [u8; MUHASH_SIZE]) -> Self {
        Self(bytes)
    }

    /// Returns the bytes of the MuHash
    pub fn as_bytes(&self) -> &[u8; MUHASH_SIZE] {
        &self.0
    }

    /// Combines this MuHash with another one
    pub fn combine(&mut self, other: &MuHash) {
        // TODO: Implement actual MuHash combining logic
        // This is just a placeholder that XORs the bytes
        for i in 0..MUHASH_SIZE {
            self.0[i] ^= other.0[i];
        }
    }
}