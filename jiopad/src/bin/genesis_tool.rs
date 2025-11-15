use clap::Parser;
use consensus_core::config::genesis as core_genesis;
use consensus_core::block::Block;
use consensus_core::header::Header;
use consensus_pow::State as PowState;
use std::fs;
use std::time::Instant;

/// Small genesis tool that can print, mine and serialize the deterministic genesis.
#[derive(Parser, Debug)]
#[command(name = "genesis_tool")]
struct Opts {
	/// Print bincode-serialized genesis as hex
	#[arg(long)]
	hex: bool,

	/// Mine for a valid PoW nonce (one-shot miner). This will search nonces until a valid one is found or max-iterations reached.
	#[arg(long)]
	mine: bool,

	/// Output serialized genesis to this file (binary). If omitted, prints to stdout only.
	#[arg(long)]
	out: Option<String>,

	/// Override bits (compact representation). Accepts decimal or 0x-prefixed hex.
	#[arg(long)]
	bits: Option<String>,

	/// Override timestamp (milliseconds)
	#[arg(long)]
	timestamp: Option<u64>,

	/// Maximum nonce/iterations to try when mining (default: 10_000_000)
	#[arg(long, default_value_t = 10_000_000u64)]
	max_iterations: u64,
}

fn parse_bits(s: &str) -> Option<u32> {
	if s.starts_with("0x") || s.starts_with("0X") {
		u32::from_str_radix(&s[2..], 16).ok()
	} else {
		s.parse::<u32>().ok()
	}
}

fn main() {
	let opts = Opts::parse();

	let mut genesis = core_genesis::default_genesis();

	// Build a mutable header from the genesis and allow overrides
	let mut header: Header = (&genesis).into();

	if let Some(bits_str) = opts.bits.as_deref() {
		if let Some(b) = parse_bits(bits_str) {
			header.bits = b;
		} else {
			eprintln!("Failed to parse bits '{}', ignoring", bits_str);
		}
	}

	// Apply provided timestamp override; otherwise keep the genesis default timestamp.
	if let Some(ts) = opts.timestamp {
		header.timestamp = ts;
	}

	// Finalize header now that overrides are applied. Do NOT reset nonce here;
	// keep the default genesis nonce unless the user is mining (which will reset it).
	header.finalize();

	// Optionally mine: search for a nonce satisfying PoW
	if opts.mine {
		println!("Starting one-shot mine (max_iterations={})...", opts.max_iterations);
		let start = Instant::now();
		// start mining from nonce=0
		header.nonce = 0;
		header.finalize();
		let state = PowState::new(&header);
		let mut found = false;
		let mut pow_value = None;
		let mut nonce = 0u64;
		while nonce < opts.max_iterations {
			let (ok, pow) = state.check_pow(nonce);
			if ok {
				header.nonce = nonce;
				header.finalize();
				found = true;
				pow_value = Some(pow);
				break;
			}
			nonce = nonce.wrapping_add(1);
			if nonce % 1_000_000 == 0 {
				let elapsed = start.elapsed().as_secs_f64();
				let rate = nonce as f64 / elapsed.max(1e-6);
				println!("Tried {} nonces (rate {:.2} kH/s)", nonce, rate / 1000.0);
			}
		}

		let elapsed = start.elapsed();
		if found {
			println!("Found valid nonce {} in {:.2}s", header.nonce, elapsed.as_secs_f64());
			if let Some(pow) = pow_value {
				let mut be = [0u8; 32];
				pow.to_big_endian(&mut be);
				println!("PoW value: 0x{}", hex::encode(&be));
			}
		} else {
			eprintln!("Failed to find a valid nonce within {} iterations (elapsed {:.2}s)", opts.max_iterations, elapsed.as_secs_f64());
		}
	}

	// Rebuild a GenesisBlock-like block for serialization/printing
	let block = Block::new(header.clone(), genesis.build_genesis_transactions());

	println!("Genesis hash: {}", hex::encode(block.header.hash.as_bytes()));
	println!("Merkle root: {}", hex::encode(block.header.hash_merkle_root.as_bytes()));
	println!("UTXO commitment: {}", hex::encode(block.header.utxo_commitment.as_bytes()));
	println!("Timestamp: {}", block.header.timestamp);
	println!("Bits: 0x{:08x}", block.header.bits);
	println!("Nonce: {}", block.header.nonce);
	println!("Coinbase payload (utf8): {}", String::from_utf8_lossy(genesis.coinbase_payload));

	if opts.hex || opts.out.is_some() {
		match bincode::serialize(&block) {
			Ok(bytes) => {
				println!("Serialized genesis (hex): {}", hex::encode(&bytes));
				if let Some(path) = opts.out.as_deref() {
					if let Err(e) = fs::write(path, &bytes) {
						eprintln!("Failed to write genesis to {}: {}", path, e);
					} else {
						println!("Wrote serialized genesis to {}", path);
					}
				} else if opts.mine {
					// Default place to store a freshly mined genesis so it's easy to find in the repo
					let default_path = "jiopad/genesis.bin";
					if let Err(e) = fs::write(default_path, &bytes) {
						eprintln!("Failed to write default-genesis to {}: {}", default_path, e);
					} else {
						println!("Wrote serialized genesis to {}", default_path);
					}
				}
			}
			Err(e) => eprintln!("Failed to serialize genesis block: {}", e),
		}
	}
}

