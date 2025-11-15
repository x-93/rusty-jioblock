use sha2::{Digest, Sha256};
use std::io::Write;

pub trait Hashable {
    fn hash_into(&self, state: &mut Sha256);
}

#[derive(Debug)]
pub enum HashError {
    EncodingError(std::io::Error),
    DecodingError(&'static str),
}

impl std::fmt::Display for HashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashError::EncodingError(e) => write!(f, "Hash encoding error: {}", e),
            HashError::DecodingError(msg) => write!(f, "Hash decoding error: {}", msg),
        }
    }
}

impl std::error::Error for HashError {}

/// Compute SHA256(SHA256(data))
pub fn double_sha256(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(&first);
    second.into()
}

/// Compute SHA256(data)
pub fn sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

/// HashWriter wraps a Sha256 hasher to implement Write trait
#[derive(Clone)]
pub struct HashWriter(Sha256);

impl HashWriter {
    pub fn new() -> Self {
        Self(Sha256::new())
    }

    pub fn finalize(self) -> [u8; 32] {
        self.0.finalize().into()
    }

    pub fn hash_object<T: Hashable>(&mut self, obj: &T) {
        obj.hash_into(&mut self.0);
    }
}

impl Write for HashWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Default for HashWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_double_sha256() {
        let data = b"hello";
        let hash = double_sha256(data);
        assert_eq!(
            hash,
            hex!("9595c9df90075148eb06860365df33584b75bff782a510c6cd4883a419833d50")
        );
    }

    #[test]
    fn test_hash_writer() {
        let mut writer = HashWriter::new();
        writer.write_all(b"hello").unwrap();
        let hash = writer.finalize();
        assert_eq!(
            hash,
            hex!("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824")
        );
    }
}