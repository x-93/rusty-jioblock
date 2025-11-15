use crate::{Hash, HashWriter};
use std::io::Write;

/// Hash writer that follows POW block header hashing rules
#[derive(Clone)]
pub struct PowB3Hash {
    inner: HashWriter,
    #[allow(dead_code)] // Used implicitly in hash computation
    timestamp: u64,
}

impl PowB3Hash {
    pub fn new(pre_pow_hash: Hash, timestamp: u64) -> Self {
        let mut inner = HashWriter::new();
        inner.write_all(pre_pow_hash.as_bytes()).unwrap();
        inner.write_all(&timestamp.to_le_bytes()).unwrap();
        // Add padding
        inner.write_all(&[0; 32]).unwrap();

        Self { inner, timestamp }
    }

    pub fn finalize_with_nonce(&mut self, nonce: u64) -> Hash {
        // Write nonce at end
        self.inner.write_all(&nonce.to_le_bytes()).unwrap();
        Hash::from(self.inner.clone().finalize())
    }
}

/// Hash writer for the fast POW variant using FishHash
#[derive(Clone)]
pub struct PowFishHash {
    inner: HashWriter,
}

impl PowFishHash {
    pub fn new() -> Self {
        Self {
            inner: HashWriter::new(),
        }
    }

    pub fn finalize(&self) -> Hash {
        Hash::from(self.inner.clone().finalize())
    }
}

impl Write for PowFishHash {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_pow_b3_hash() {
        let pre_hash = Hash::from(hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"));
        let timestamp = 1234567890;
        let nonce = 42;

        let mut hasher = PowB3Hash::new(pre_hash, timestamp);
        let hash = hasher.finalize_with_nonce(nonce);

        // Hash should be deterministic
        let mut hasher2 = PowB3Hash::new(pre_hash, timestamp);
        let hash2 = hasher2.finalize_with_nonce(nonce);
        assert_eq!(hash, hash2);

        // Different nonce should give different hash
        let mut hasher3 = PowB3Hash::new(pre_hash, timestamp);
        let hash3 = hasher3.finalize_with_nonce(nonce + 1);
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_pow_fish_hash() {
        let mut hasher = PowFishHash::new();
        hasher.write_all(b"test data").unwrap();
        let hash = hasher.finalize();

        // Hash should be deterministic
        let mut hasher2 = PowFishHash::new();
        hasher2.write_all(b"test data").unwrap();
        let hash2 = hasher2.finalize();
        assert_eq!(hash, hash2);

        // Different data should give different hash
        let mut hasher3 = PowFishHash::new();
        hasher3.write_all(b"other data").unwrap();
        let hash3 = hasher3.finalize();
        assert_ne!(hash, hash3);
    }
}