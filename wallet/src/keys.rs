use secp256k1::{Secp256k1, SecretKey, PublicKey};
use rand::{rngs::OsRng, RngCore};
use hmac::{Hmac, Mac};
use sha2::Sha256;


/// HD wallet key management (BIP32/BIP44 style)
#[derive(Clone)]
pub struct Keys {
    master_seed: [u8; 64],
    secp: Secp256k1<secp256k1::All>,
}

impl Keys {
    /// Create new keys from random seed
    pub fn new() -> Self {
        let mut rng = OsRng;
        let mut seed = [0u8; 64];
        rng.fill_bytes(&mut seed);

        Self {
            master_seed: seed,
            secp: Secp256k1::new(),
        }
    }

    /// Create keys from existing seed
    pub fn from_seed(seed: [u8; 64]) -> Self {
        Self {
            master_seed: seed,
            secp: Secp256k1::new(),
        }
    }

    /// Derive child key at path (simplified BIP32)
    pub fn derive_key(&self, path: &[u32]) -> Result<SecretKey, String> {
        let mut key = self.master_seed;
        let mut chain_code = self.master_seed[32..].to_vec();

        for &index in path {
            let mut data = vec![];
            data.extend_from_slice(&chain_code);
            data.extend_from_slice(&key[0..32]);
            data.extend_from_slice(&index.to_be_bytes());

            let hmac = Hmac::<Sha256>::new_from_slice(b"Bitcoin seed")
                .map_err(|e| format!("HMAC error: {}", e))?
                .chain_update(&data)
                .finalize()
                .into_bytes();

            let mut new_key = [0u8; 32];
            for i in 0..32 {
                new_key[i] = key[i] ^ hmac[i];
            }

            key[0..32].copy_from_slice(&new_key);
            chain_code = hmac[32..].to_vec();
        }

        SecretKey::from_slice(&key[0..32])
            .map_err(|e| format!("Invalid secret key: {}", e))
    }

    /// Get public key from secret key
    pub fn public_key(&self, secret_key: &SecretKey) -> PublicKey {
        PublicKey::from_secret_key(&self.secp, secret_key)
    }

    /// Generate new address (BIP44 path: m/44'/0'/0'/0/0)
    pub fn generate_address(&self) -> Result<(SecretKey, PublicKey), String> {
        let path = [44 + 0x80000000, 0 + 0x80000000, 0 + 0x80000000, 0, 0];
        let secret_key = self.derive_key(&path)?;
        let public_key = self.public_key(&secret_key);
        Ok((secret_key, public_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let keys = Keys::new();
        let (sk, pk) = keys.generate_address().unwrap();

        // Verify public key matches secret key
        let expected_pk = keys.public_key(&sk);
        assert_eq!(pk, expected_pk);
    }

    #[test]
    fn test_key_derivation() {
        let seed = [42u8; 64];
        let keys = Keys::from_seed(seed);
        let path = [0, 1, 2];
        let sk = keys.derive_key(&path).unwrap();

        // Should be deterministic
        let keys2 = Keys::from_seed(seed);
        let sk2 = keys2.derive_key(&path).unwrap();
        assert_eq!(sk, sk2);
    }
}
