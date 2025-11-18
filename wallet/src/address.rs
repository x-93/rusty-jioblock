use consensus_core::tx::ScriptPublicKey;
use ripemd::Ripemd160;
use sha2::{Sha256, Digest};
use crate::keys::Keys;

/// Wallet address management
pub struct Address {
    keys: Keys,
}

impl Address {
    /// Create new address manager
    pub fn new(keys: Keys) -> Self {
        Self { keys }
    }

    /// Generate new address from public key
    pub fn from_public_key(public_key: &secp256k1::PublicKey) -> String {
        // Get compressed public key bytes
        let pubkey_bytes = public_key.serialize();

        // SHA256 hash
        let sha256_hash = Sha256::digest(&pubkey_bytes);

        // RIPEMD160 hash
        let ripemd_hash = Ripemd160::digest(&sha256_hash);

        // Add version byte (0x00 for mainnet)
        let mut versioned_payload = vec![0x00];
        versioned_payload.extend_from_slice(&ripemd_hash);

        // Double SHA256 for checksum
        let checksum = Sha256::digest(&Sha256::digest(&versioned_payload));

        // Add first 4 bytes of checksum
        versioned_payload.extend_from_slice(&checksum[0..4]);

        // Base58 encode
        bs58::encode(&versioned_payload).into_string()
    }

    /// Generate new address
    pub fn generate_new(&self) -> Result<String, String> {
        let (_, public_key) = self.keys.generate_address()?;
        Ok(Self::from_public_key(&public_key))
    }

    /// Validate address format
    pub fn validate(address: &str) -> bool {
        // Basic validation - check if valid base58 and correct length
        if let Ok(decoded) = bs58::decode(address).into_vec() {
            decoded.len() >= 21 // version + payload + checksum
        } else {
            false
        }
    }

    /// Get script public key for address
    pub fn to_script_pub_key(address: &str) -> Result<ScriptPublicKey, String> {
        if !Self::validate(address) {
            return Err("Invalid address format".to_string());
        }

        let decoded = bs58::decode(address).into_vec()
            .map_err(|e| format!("Base58 decode error: {}", e))?;

        if decoded.len() < 21 {
            return Err("Address too short".to_string());
        }

        // Extract payload (without version and checksum)
        let payload = &decoded[1..decoded.len()-4];

        // Create P2PKH script
        let mut script = vec![0x76, 0xa9, 0x14]; // OP_DUP OP_HASH160 PUSH(20)
        script.extend_from_slice(payload);
        script.extend_from_slice(&[0x88, 0xac]); // OP_EQUALVERIFY OP_CHECKSIG

        Ok(ScriptPublicKey::from_vec(0, script))
    }

    /// Get address from script public key
    pub fn from_script_pub_key(script: &ScriptPublicKey) -> Result<String, String> {
        let script_bytes = script.script();
        // Check for P2PKH script structure
        if script_bytes.len() == 25 &&
           script_bytes[0] == 0x76 && // OP_DUP
           script_bytes[1] == 0xa9 && // OP_HASH160
           script_bytes[2] == 0x14 && // PUSH(20)
           script_bytes[23] == 0x88 && // OP_EQUALVERIFY
           script_bytes[24] == 0xac { // OP_CHECKSIG

            let pubkey_hash = &script_bytes[3..23];

            // Add version byte (0x00 for mainnet)
            let mut versioned_payload = vec![0x00];
            versioned_payload.extend_from_slice(pubkey_hash);

            // Double SHA256 for checksum
            let checksum = Sha256::digest(&Sha256::digest(&versioned_payload));

            // Add first 4 bytes of checksum
            versioned_payload.extend_from_slice(&checksum[0..4]);

            // Base58 encode
            Ok(bs58::encode(&versioned_payload).into_string())
        } else {
            Err("Not a standard P2PKH script".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::Secp256k1;

    #[test]
    fn test_address_generation() {
        let keys = Keys::new();
        let address = Address::new(keys);
        let addr_str = address.generate_new().unwrap();

        // Should be valid base58
        assert!(Address::validate(&addr_str));
    }

    #[test]
    fn test_address_validation() {
        // Valid address format (placeholder)
        assert!(Address::validate("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"));

        // Invalid format
        assert!(!Address::validate("invalid"));
        assert!(!Address::validate(""));
    }

    #[test]
    fn test_script_pub_key() {
        let keys = Keys::new();
        let secp = Secp256k1::new();
        let (_, pk) = keys.generate_address().unwrap();
        let addr = Address::from_public_key(&pk);
        let script = Address::to_script_pub_key(&addr).unwrap();

        // Should have P2PKH script structure
        assert_eq!(script.script().len(), 25); // P2PKH script length
        assert_eq!(script.script()[0], 0x76); // OP_DUP
        assert_eq!(script.script()[1], 0xa9); // OP_HASH160
        assert_eq!(script.script()[2], 0x14); // PUSH(20)
    }
}
