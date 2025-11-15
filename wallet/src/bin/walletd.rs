use std::path::PathBuf;
use clap::{Parser, Subcommand};

use wallet::{Keys, Address, Keystore, TxBuilder, Signer};
use consensus::{ConsensusStorage, UtxoSet, BlockStore};
use consensus_core::tx::{TransactionOutpoint, UtxoEntry};
use std::collections::HashMap;
use wallet::keystore::{WalletData, AddressEntry};
use rand::RngCore;
use database::Database;
use database::stores::{BlockStore as DbBlockStore, HeaderStore as DbHeaderStore, UtxoStore as DbUtxoStore};
use std::sync::Arc;

/// Simple wallet management CLI for the `wallet` crate
#[derive(Parser)]
#[command(name = "walletd")]
struct Cli {
    /// Keystore file (default: wallet_keystore.json)
    #[arg(short, long, default_value = "wallet_keystore.json")]
    keystore: PathBuf,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new wallet and write an encrypted keystore
    Init {
        /// Password to encrypt the keystore
        #[arg(short, long)]
        password: String,
    },

    /// Generate and add a new address to the keystore
    NewAddress {
        /// Password to decrypt/encrypt the keystore
        #[arg(short, long)]
        password: String,
    },

    /// List addresses stored in keystore
    List {
        #[arg(short, long)]
        password: String,
    },

    /// Export master seed (hex). Warning: sensitive
    ExportSeed {
        #[arg(short, long)]
        password: String,
    },

    /// Import a raw seed (hex) and create keystore
    ImportSeed {
        /// hex seed (64 bytes -> 128 hex chars)
        #[arg(short, long)]
        seed_hex: String,
        #[arg(short, long)]
        password: String,
    },

    /// Create and sign a transaction
    SignTransaction {
        /// Recipient address
        #[arg(short, long)]
        to: String,
        /// Amount to send (in satoshis)
        #[arg(short, long)]
        amount: u64,
        /// Sender key index (default: 0)
        #[arg(long, default_value = "0")]
        from_index: u32,
        /// Password to decrypt keystore
        #[arg(short, long)]
        password: String,
    },

    /// Encode signed transaction to hex for broadcasting
    EncodeTransaction {
        /// Transaction JSON (or path to file)
        #[arg(short, long)]
        tx_json: String,
    },
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.cmd {
        Commands::Init { password } => {
            // Generate a new random master seed
            let mut seed = [0u8; 64];
            rand::rngs::OsRng.fill_bytes(&mut seed);

            // Create keys and get default address
            let keys = Keys::from_seed(seed);
            let addr_mgr = Address::new(keys.clone());
            let addr = addr_mgr.generate_new().map_err(|e| format!("Failed to generate address: {}", e))?;
            
            // Get public key
            let (_sk, pk) = keys.generate_address().map_err(|e| format!("Failed to derive key: {}", e))?;

            // Create and encrypt keystore with first address
            let mut ks = Keystore::new();
            let mut data = Keystore::create_wallet_data(seed);
            data.addresses.insert(addr.clone(), AddressEntry {
                path: vec![44 + 0x8000_0000, 0 + 0x8000_0000, 0 + 0x8000_0000, 0, 0],
                public_key: pk.serialize().to_vec(),
                label: None,
            });
            ks.encrypt(&password, &data).map_err(|e| format!("Encrypt failed: {}", e))?;
            ks.save(&cli.keystore).map_err(|e| format!("Save failed: {}", e))?;

            println!("Created keystore at {}", cli.keystore.display());
            println!("Initial address: {}", addr);
            Ok(())
        }

        Commands::NewAddress { password } => {
            // Load keystore
            let mut ks = Keystore::load(&cli.keystore).map_err(|e| format!("Failed to load keystore: {}", e))?;
            
            // Get current address count to determine next index
            let current_addresses = ks.list_addresses(&password).map_err(|e| format!("Failed to list addresses: {}", e))?;
            let next_index = current_addresses.len() as u32;

            // Decrypt to get seed
            let data = ks.decrypt(&password).map_err(|e| format!("Failed to decrypt: {}", e))?;
            if data.master_seed.len() != 64 {
                return Err("Master seed in keystore is not 64 bytes".to_string());
            }
            let mut seed = [0u8; 64];
            seed.copy_from_slice(&data.master_seed[..64]);

            // Create Keys from seed
            let keys = Keys::from_seed(seed);

            // Derive key at m/44'/0'/0'/0/index
            let path = vec![44u32 + 0x8000_0000, 0u32 + 0x8000_0000, 0u32 + 0x8000_0000, 0, next_index];
            let sk = keys.derive_key(&path).map_err(|e| format!("derive_key failed: {}", e))?;
            let pk = keys.public_key(&sk);
            let addr = Address::from_public_key(&pk);

            // Add address to keystore
            ks.add_address_to_keystore(&password, addr.clone(), path, pk.serialize().to_vec())
                .map_err(|e| format!("Failed to add address: {}", e))?;

            println!("Added address: {}", addr);
            Ok(())
        }

        Commands::List { password } => {
            let ks = Keystore::load(&cli.keystore).map_err(|e| format!("Failed to load keystore: {}", e))?;
            let addresses = ks.list_addresses(&password).map_err(|e| format!("Failed to list: {}", e))?;
            println!("Addresses in {}:", cli.keystore.display());
            for (addr, path) in addresses {
                println!("- {} (path: {:?})", addr, path);
            }
            Ok(())
        }

        Commands::ExportSeed { password } => {
            let ks = Keystore::load(&cli.keystore).map_err(|e| format!("Failed to load keystore: {}", e))?;
            let hex = ks.export_seed(&password).map_err(|e| format!("Failed to export: {}", e))?;
            println!("Master seed (hex) WARNING: keep secret: {}", hex);
            Ok(())
        }

        Commands::ImportSeed { seed_hex, password } => {
            let bytes = hex::decode(&seed_hex).map_err(|e| format!("Invalid hex seed: {}", e))?;
            if bytes.len() != 64 {
                return Err("Seed must be exactly 64 bytes (128 hex chars)".to_string());
            }
            let mut seed = [0u8; 64];
            seed.copy_from_slice(&bytes);

            let mut ks = Keystore::new();
            let data = Keystore::create_wallet_data(seed);
            ks.encrypt(&password, &data).map_err(|e| format!("Encrypt failed: {}", e))?;
            ks.save(&cli.keystore).map_err(|e| format!("Save failed: {}", e))?;
            println!("Imported seed and saved keystore to {}", cli.keystore.display());
            Ok(())
        }

        Commands::SignTransaction { to, amount, from_index, password } => {
            // Load keystore
            let ks = Keystore::load(&cli.keystore).map_err(|e| format!("Failed to load keystore: {}", e))?;
            
            // Decrypt to get seed
            let data = ks.decrypt(&password).map_err(|e| format!("Failed to decrypt: {}", e))?;
            if data.master_seed.len() != 64 {
                return Err("Master seed in keystore is not 64 bytes".to_string());
            }
            let mut seed = [0u8; 64];
            seed.copy_from_slice(&data.master_seed[..64]);

            // Create Keys from seed
            let keys = Keys::from_seed(seed);

            // Derive sender key
            let path = vec![44u32 + 0x8000_0000, 0u32 + 0x8000_0000, 0u32 + 0x8000_0000, 0, from_index];
            let sk = keys.derive_key(&path).map_err(|e| format!("derive_key failed: {}", e))?;
            let pk = keys.public_key(&sk);
            let sender_addr = Address::from_public_key(&pk);

            // Validate recipient address
            if !Address::validate(&to) {
                return Err(format!("Invalid recipient address: {}", to));
            }

            // Create transaction (minimal example - no UTXO inputs for now)
            println!("Sender: {}", sender_addr);
            println!("Recipient: {}", to);
            println!("Amount: {} satoshis", amount);
            println!("Transaction signing not fully implemented (requires UTXO set)");
            println!("Would need to:");
            println!("  1. Query UTXO set for sender funds");
            println!("  2. Select UTXOs to spend");
            println!("  3. Create transaction with inputs/outputs");
            println!("  4. Sign with secret key");
            
            Ok(())
        }

        Commands::EncodeTransaction { tx_json } => {
            // This would typically take a JSON transaction and encode it to hex bincode
            println!("Transaction JSON: {}", tx_json);
            println!("Encoding transaction to hex (bincode serialization)");
            println!("Note: Transaction encoding requires a serialized Transaction struct from consensus_core");
            Ok(())
        }
    }
}

// Re-export internal keystore types used for initialization only

