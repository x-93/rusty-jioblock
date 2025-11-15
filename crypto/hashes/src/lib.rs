pub mod builders;
pub mod hasher;
pub mod merkle;
pub mod pow_hash;

// Re-export commonly used types
pub use hasher::{Hashable, HashError, HashWriter, double_sha256, sha256};
pub use pow_hash::{PowB3Hash, PowFishHash};

use sha2::Digest;
use std::fmt;
use std::hash::Hash as StdHash;
use std::ops::{Deref, DerefMut};

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
pub const HASH_SIZE: usize = 32;
/// A 32-byte hash wrapper used across the project.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    /// Create a hash from a 32-byte array
    pub fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns raw bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Creates a zeroed hash
    pub fn zeroed() -> Self {
        Self([0u8; 32])
    }

    /// Const constructor returning a zeroed Hash. Can be used in const contexts.
    pub const fn zeroed_const() -> Self {
        Self([0u8; 32])
    }

    /// Constructs a hash from four little-endian u64s (used in tests)
    pub const fn from_le_u64(parts: [u64; 4]) -> Self {
        let mut bytes = [0u8; 32];
        let mut i = 0;
        while i < 4 {
            let part = parts[i];
            bytes[i * 8] = (part & 0xFF) as u8;
            bytes[i * 8 + 1] = ((part >> 8) & 0xFF) as u8;
            bytes[i * 8 + 2] = ((part >> 16) & 0xFF) as u8;
            bytes[i * 8 + 3] = ((part >> 24) & 0xFF) as u8;
            bytes[i * 8 + 4] = ((part >> 32) & 0xFF) as u8;
            bytes[i * 8 + 5] = ((part >> 40) & 0xFF) as u8;
            bytes[i * 8 + 6] = ((part >> 48) & 0xFF) as u8;
            bytes[i * 8 + 7] = ((part >> 56) & 0xFF) as u8;
            i += 1;
        }
        Self(bytes)
    }

    /// Tries to create a Hash from a slice of bytes
    pub fn try_from_slice(slice: &[u8]) -> Result<Self, std::array::TryFromSliceError> {
        let array: [u8; 32] = slice.try_into()?;
        Ok(Self(array))
    }

    /// Creates a hash from a single u64 word (for compatibility)
    pub fn from_u64_word(word: u64) -> Self {
        Self::from_le_u64([word, 0, 0, 0])
    }
}

// Implement From and conversions
impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Hash::from(bytes)
    }
}

impl From<Hash> for [u8; 32] {
    fn from(h: Hash) -> Self {
        h.0
    }
}

impl From<Vec<u8>> for Hash {
    fn from(vec: Vec<u8>) -> Self {
        let array: [u8; 32] = vec.try_into().expect("Vec must be exactly 32 bytes");
        Self(array)
    }
}


impl TryFrom<&[u8]> for Hash {
    type Error = std::array::TryFromSliceError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let array: [u8; 32] = slice.try_into()?;
        Ok(Self(array))
    }
}

// Implement AsRef<[u8]> for Hash
impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

// Implement Display and Debug for Hash
impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({})", hex::encode(self.0))
    }
}

// Implement std::hash::Hash for Hash
impl StdHash for Hash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // use the last u64 as a fast hasher source
        let mut le = [0u8; 8];
        le.copy_from_slice(&self.0[24..32]);
        let v = u64::from_le_bytes(le);
        v.hash(state);
    }
}

impl Deref for Hash {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Hash {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl hasher::Hashable for Hash {
    fn hash_into(&self, state: &mut sha2::Sha256) {
        state.update(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::Hash;

    #[test]
    fn from_le_u64_roundtrip() {
        let h = Hash::from_le_u64([1, 2, 3, 4]);
        let bytes = h.as_bytes();
        assert_eq!(&bytes[0..8], &1u64.to_le_bytes());
        assert_eq!(&bytes[8..16], &2u64.to_le_bytes());
        assert_eq!(&bytes[24..32], &4u64.to_le_bytes());
    }
}
