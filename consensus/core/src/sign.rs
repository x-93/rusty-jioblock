use crate::Hash;
use std::io::{Read, Write};

/// Digital signature information
#[derive(Debug, Clone)]
pub struct Signature {
    /// Public key (compressed format)
    pub public_key: [u8; 33],
    /// The R value of the signature
    pub r: [u8; 32],
    /// The S value of the signature
    pub s: [u8; 32],
}

impl Signature {
    /// Creates a new signature from its components
    pub fn new(public_key: [u8; 33], r: [u8; 32], s: [u8; 32]) -> Self {
        Self {
            public_key,
            r,
            s,
        }
    }

    /// Verifies this signature against a message hash
    pub fn verify(&self, _message_hash: &Hash) -> bool {
        // TODO: Implement actual signature verification
        true
    }
}

impl borsh::BorshSerialize for Signature {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.public_key)?;
        writer.write_all(&self.r)?;
        writer.write_all(&self.s)?;
        Ok(())
    }
}

impl borsh::BorshDeserialize for Signature {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let mut public_key = [0u8; 33];
        let mut r = [0u8; 32];
        let mut s = [0u8; 32];
        
        buf.read_exact(&mut public_key)?;
        buf.read_exact(&mut r)?;
        buf.read_exact(&mut s)?;

        Ok(Signature {
            public_key,
            r,
            s,
        })
    }
}