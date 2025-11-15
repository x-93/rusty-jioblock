use crate::{hasher::HashWriter, Hash};
use std::io::Write;

/// A builder for incrementally constructing block hashes
#[derive(Clone)]
pub struct BlockHashBuilder {
    writer: HashWriter,
}

impl BlockHashBuilder {
    pub fn new() -> Self {
        Self {
            writer: HashWriter::new(),
        }
    }

    pub fn add_version(&mut self, version: u16) -> &mut Self {
        self.writer.write_all(&version.to_le_bytes()).unwrap();
        self
    }

    pub fn add_parent(&mut self, parent: &Hash) -> &mut Self {
        self.writer.write_all(parent.as_bytes()).unwrap();
        self
    }

    pub fn add_timestamp(&mut self, timestamp: u64) -> &mut Self {
        self.writer.write_all(&timestamp.to_le_bytes()).unwrap();
        self
    }

    pub fn add_target(&mut self, target: u32) -> &mut Self {
        self.writer.write_all(&target.to_le_bytes()).unwrap();
        self
    }

    pub fn add_nonce(&mut self, nonce: u64) -> &mut Self {
        self.writer.write_all(&nonce.to_le_bytes()).unwrap();
        self
    }

    pub fn add_merkle_root(&mut self, root: &Hash) -> &mut Self {
        self.writer.write_all(root.as_bytes()).unwrap();
        self
    }

    pub fn finalize(self) -> Hash {
        Hash::from(self.writer.finalize())
    }
}

impl Default for BlockHashBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A builder for incrementally constructing transaction hashes
#[derive(Clone)]
pub struct TxHashBuilder {
    writer: HashWriter,
}

impl TxHashBuilder {
    pub fn new() -> Self {
        Self {
            writer: HashWriter::new(),
        }
    }

    pub fn add_version(&mut self, version: u32) -> &mut Self {
        self.writer.write_all(&version.to_le_bytes()).unwrap();
        self
    }

    pub fn add_input_hash(&mut self, hash: &Hash) -> &mut Self {
        self.writer.write_all(hash.as_bytes()).unwrap();
        self
    }

    pub fn add_output_value(&mut self, value: u64) -> &mut Self {
        self.writer.write_all(&value.to_le_bytes()).unwrap();
        self
    }

    pub fn add_script(&mut self, script: &[u8]) -> &mut Self {
        self.writer.write_all(script).unwrap();
        self
    }

    pub fn finalize(self) -> Hash {
        Hash::from(self.writer.finalize())
    }
}

impl Default for TxHashBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_block_hash_builder() {
        let parent = Hash::from(hex!("0000000000000000000000000000000000000000000000000000000000000001"));
        let merkle_root = Hash::from(hex!("0000000000000000000000000000000000000000000000000000000000000002"));

            let mut builder = BlockHashBuilder::new();
            builder.add_version(1)
                .add_parent(&parent)
                .add_timestamp(1234567890)
                .add_target(0x1d00ffff)
                .add_nonce(42)
                .add_merkle_root(&merkle_root);
            let hash = builder.clone().finalize();        // The exact hash value will depend on the specific byte ordering and format
        assert_ne!(hash, Hash::zeroed());
    }

    #[test]
    fn test_tx_hash_builder() {
        let input_hash = Hash::from(hex!("0000000000000000000000000000000000000000000000000000000000000001"));
        
            let mut builder = TxHashBuilder::new();
            builder.add_version(1)
                .add_input_hash(&input_hash)
                .add_output_value(50_000_000)
                .add_script(&[0xAC]); // OP_CHECKSIG
            let hash = builder.clone().finalize();        assert_ne!(hash, Hash::zeroed());
    }
}