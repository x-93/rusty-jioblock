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

            // --- Consensus Storage Integration ---
            // 1. Open the database (path is hardcoded for now)
            let db_path = PathBuf::from("d:\\Jio-Block\\data");
            let db = Arc::new(Database::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?);

            // 2. Create database-backed store instances
            let db_block_store = Arc::new(DbBlockStore::new(db.clone(), 1024));
            let db_header_store = Arc::new(DbHeaderStore::new(db.clone(), 1024));
            let db_utxo_store = Arc::new(DbUtxoStore::new(db.clone(), 1024));

            // 3. Create consensus stores with the DB-backed stores
            let block_store = Arc::new(BlockStore::new_with_db(db_block_store, Some(db_header_store)));
            let utxo_set = Arc::new(UtxoSet::new_with_db(db_utxo_store));

            // 4. Create the main ConsensusStorage instance
            let consensus_storage = ConsensusStorage::with_stores(block_store, utxo_set);
            println!("Successfully initialized consensus storage from database.");
            // --- End of Integration ---

            // Now, you can use consensus_storage to query UTXOs
            let utxo_view = consensus_storage.utxo_set();
            let utxo_snapshot = utxo_view.snapshot();

            println!("Found {} total UTXOs in the database.", utxo_snapshot.len());

            let mut sender_utxos = Vec::new();
            for (outpoint, utxo_entry) in utxo_snapshot.iter() {
                if let Ok(addr) = Address::from_script_pub_key(&utxo_entry.script_public_key) {
                    if addr == sender_addr {
                        sender_utxos.push((outpoint.clone(), utxo_entry.clone()));
                    }
                }
            }

            println!("Found {} UTXOs for sender address {}:", sender_utxos.len(), sender_addr);
            for (outpoint, entry) in &sender_utxos {
                println!("  - Outpoint: {}:{}, Amount: {}", outpoint.transaction_id, outpoint.index, entry.amount);
            }

            // Convert sender_utxos to HashMap for TxBuilder
            let utxo_map: HashMap<TransactionOutpoint, UtxoEntry> = sender_utxos.into_iter().collect();

            // Use TxBuilder to construct the transaction
            let tx_builder = TxBuilder::send_to_address(
                &utxo_map,
                &sender_addr.to_string(),
                &to,
                amount,
                1, // fee rate (sompi per byte)
            ).map_err(|e| format!("Failed to build transaction: {}", e))?;

            println!("Transaction built successfully. Now signing...");

            // Build the transaction
            let unsigned_tx = tx_builder.build(&utxo_map)
                .map_err(|e| format!("Failed to finalize transaction build: {}", e))?;

            // Create signer
            let signer = Signer::new(keys);

            // Get secret keys for inputs.
            // This is simplified: it assumes all inputs are from the same `from_index` address.
            let mut secret_keys = Vec::new();
            for _ in 0..unsigned_tx.inputs.len() {
                secret_keys.push(sk.clone());
            }

            // Sign the transaction
            let signed_tx = signer.sign_transaction(unsigned_tx, &secret_keys)
                .map_err(|e| format!("Failed to sign transaction: {}", e))?;
            
            println!("Transaction signed successfully.");

            // Encode the signed transaction to hex
            let serialized_tx = bincode::serialize(&signed_tx)
                .map_err(|e| format!("Failed to serialize transaction: {}", e))?;
            let hex_tx = hex::encode(serialized_tx);

            println!("--- Signed Transaction (Hex) ---");
            println!("{}", hex_tx);
            println!("---------------------------------");
            
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

