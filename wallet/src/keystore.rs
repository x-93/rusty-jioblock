use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use aes_gcm::{Aes256Gcm, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use argon2::Argon2;
use rand::rngs::OsRng;
use rand::RngCore;
use hex;

/// Encrypted keystore for storing wallet data
#[derive(Serialize, Deserialize)]
pub struct Keystore {
    version: u32,
    salt: [u8; 32],
    encrypted_data: Vec<u8>,
    nonce: [u8; 12],
}

#[derive(Serialize, Deserialize)]
pub struct WalletData {
    pub addresses: HashMap<String, AddressEntry>,
    pub master_seed: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct AddressEntry {
    pub path: Vec<u32>,
    pub public_key: Vec<u8>,
    pub label: Option<String>,
}

impl Keystore {
    /// Create new keystore
    pub fn new() -> Self {
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);

        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);

        Self {
            version: 1,
            salt,
            encrypted_data: Vec::new(),
            nonce,
        }
    }

    /// Load keystore from file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let data = fs::read(path)
            .map_err(|e| format!("Failed to read keystore: {}", e))?;

        serde_json::from_slice(&data)
            .map_err(|e| format!("Failed to parse keystore: {}", e))
    }

    /// Save keystore to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let data = serde_json::to_vec_pretty(self)
            .map_err(|e| format!("Failed to serialize keystore: {}", e))?;

        fs::write(path, data)
            .map_err(|e| format!("Failed to write keystore: {}", e))
    }

    /// Encrypt and store wallet data
    pub fn encrypt(&mut self, password: &str, wallet_data: &WalletData) -> Result<(), String> {
        let data_bytes = serde_json::to_vec(wallet_data)
            .map_err(|e| format!("Failed to serialize wallet data: {}", e))?;

        // Derive key from password using Argon2
        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(password.as_bytes(), &self.salt, &mut key)
            .map_err(|e| format!("Key derivation failed: {}", e))?;

        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Cipher creation failed: {:?}", e))?;
        let nonce = Nonce::from_slice(&self.nonce);

        self.encrypted_data = cipher.encrypt(nonce, data_bytes.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        Ok(())
    }

    /// Decrypt wallet data
    pub fn decrypt(&self, password: &str) -> Result<WalletData, String> {
        // Derive key from password
        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(password.as_bytes(), &self.salt, &mut key)
            .map_err(|e| format!("Key derivation failed: {}", e))?;

        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Cipher creation failed: {:?}", e))?;
        let nonce = Nonce::from_slice(&self.nonce);

        let decrypted = cipher.decrypt(nonce, self.encrypted_data.as_ref())
            .map_err(|e| "Decryption failed - wrong password or corrupted data".to_string())?;

        serde_json::from_slice(&decrypted)
            .map_err(|e| format!("Failed to parse decrypted data: {}", e))
    }

    /// List all addresses in wallet
    pub fn list_addresses(&self, password: &str) -> Result<Vec<(String, Vec<u32>)>, String> {
        let data = self.decrypt(password)?;
        Ok(data.addresses.iter().map(|(addr, entry)| (addr.clone(), entry.path.clone())).collect())
    }

    /// Export master seed as hex
    pub fn export_seed(&self, password: &str) -> Result<String, String> {
        let data = self.decrypt(password)?;
        Ok(hex::encode(&data.master_seed))
    }

    /// Add new address to encrypted keystore (add address, then re-encrypt)
    pub fn add_address_to_keystore(&mut self, password: &str, address: String, path: Vec<u32>, public_key: Vec<u8>) -> Result<(), String> {
        let mut data = self.decrypt(password)?;
        data.addresses.insert(address, AddressEntry {
            path,
            public_key,
            label: None,
        });
        self.encrypt(password, &data)?;
        Ok(())
    }

    /// Create wallet data from keys
    pub fn create_wallet_data(master_seed: [u8; 64]) -> WalletData {
        WalletData {
            addresses: HashMap::new(),
            master_seed: master_seed.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_keystore_encrypt_decrypt() {
        let mut keystore = Keystore::new();
        let password = "test_password";

        // Create test wallet data
        let master_seed = [42u8; 64];
        let wallet_data = Keystore::create_wallet_data(master_seed);

        // Encrypt
        keystore.encrypt(password, &wallet_data).unwrap();

        // Decrypt
        let decrypted = keystore.decrypt(password).unwrap();

        // Verify
        assert_eq!(decrypted.master_seed, master_seed.to_vec());
    }

    #[test]
    fn test_keystore_add_address() {
        let mut keystore = Keystore::new();
        let password = "test_password";

        // Create and encrypt
        let master_seed = [42u8; 64];
        let wallet_data = Keystore::create_wallet_data(master_seed);
        keystore.encrypt(password, &wallet_data).unwrap();

        // Add address via public method
        keystore.add_address_to_keystore(
            password,
            "test_address".to_string(),
            vec![44, 0, 0, 0, 0],
            vec![0x02, 0x03],
        ).unwrap();

        // List and verify
        let addresses = keystore.list_addresses(password).unwrap();
        assert!(addresses.iter().any(|(addr, _)| addr == "test_address"));
    }

    #[test]
    fn test_keystore_save_load() {
        let mut keystore = Keystore::new();
        let password = "test_password";

        // Create test data
        let master_seed = [123u8; 64];
        let wallet_data = Keystore::create_wallet_data(master_seed);
        keystore.encrypt(password, &wallet_data).unwrap();

        // Save to temp file
        let temp_file = NamedTempFile::new().unwrap();
        keystore.save(temp_file.path()).unwrap();

        // Load from file
        let loaded = Keystore::load(temp_file.path()).unwrap();

        // Decrypt and verify
        let decrypted = loaded.decrypt(password).unwrap();
        assert_eq!(decrypted.master_seed, master_seed.to_vec());
    }

    #[test]
    fn test_wrong_password() {
        let mut keystore = Keystore::new();
        let wallet_data = Keystore::create_wallet_data([0u8; 64]);
        keystore.encrypt("correct_password", &wallet_data).unwrap();

        // Try to decrypt with wrong password
        let result = keystore.decrypt("wrong_password");
        assert!(result.is_err());
    }
}
