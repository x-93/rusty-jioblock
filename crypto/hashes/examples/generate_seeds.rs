// Seed Generator for JIO Network
// Run with: cargo run --release --example generate_seeds

use sha3::digest::{ExtendableOutput, XofReader};
use sha3::CShake256;

fn generate_seed(domain: &[u8]) -> [u8; 32] {
    let hasher = CShake256::from_core(sha3::CShake256Core::new(domain));
    let mut seed = [0u8; 32];
    hasher.finalize_xof().read(&mut seed);
    seed
}

fn generate_keccak_state(domain: &[u8]) -> [u64; 25] {
    let mut state_bytes = [0u8; 200];
    let hasher = CShake256::from_core(sha3::CShake256Core::new(domain));
    hasher.finalize_xof().read(&mut state_bytes);
    
    let mut state = [0u64; 25];
    for i in 0..25 {
        let bytes = &state_bytes[i*8..(i+1)*8];
        state[i] = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
    }
    state
}

fn bytes_to_hex_array(data: &[u8; 32]) -> String {
    data.iter()
        .map(|b| format!("0x{:02x}", b))
        .collect::<Vec<_>>()
        .join(", ")
}

fn state_to_rust_format(state: &[u64; 25]) -> String {
    let mut result = String::from("[\n");
    for (i, val) in state.iter().enumerate() {
        if i % 5 == 0 {
            result.push_str("    ");
        }
        result.push_str(&format!("0x{:016x}, ", val));
        if (i + 1) % 5 == 0 {
            result.push('\n');
        }
    }
    result.push(']');
    result
}

fn main() {
    println!("================================================================================");
    println!("JIO NETWORK SEED GENERATOR");
    println!("================================================================================\n");
    
    // Define domain strings for JIO network
    let domains = vec![
        ("FishHash", b"JIO_FishHashSeed".to_vec()),
        ("ProofOfWork", b"JIO_ProofOfWorkHash".to_vec()),
        ("HeavyHash", b"JIO_HeavyHash".to_vec()),
    ];
    
    // Generate FishHash seed
    println!("1. FISHHASH SEED (32 bytes)");
    println!("--------------------------------------------------------------------------------");
    let fishhash_domain = b"JIO_FishHashSeed";
    let fishhash_seed = generate_seed(fishhash_domain);
    println!("Domain: {}", String::from_utf8_lossy(fishhash_domain));
    println!("Seed: ");
    for (i, byte) in fishhash_seed.iter().enumerate() {
        if i % 8 == 0 {
            print!("    ");
        }
        print!("0x{:02x}, ", byte);
        if (i + 1) % 8 == 0 {
            println!();
        }
    }
    if fishhash_seed.len() % 8 != 0 {
        println!();
    }
    println!("]);");
    
    // Generate PowHash initial state
    println!("\n\n2. POWHASH INITIAL STATE (25 u64s)");
    println!("--------------------------------------------------------------------------------");
    let powhash_domain = b"JIO_ProofOfWorkHash";
    let powhash_state = generate_keccak_state(powhash_domain);
    println!("Domain: {}", String::from_utf8_lossy(powhash_domain));
    println!("First u64: 0x{:016x}\n", powhash_state[0]);
    println!("Rust code for PowHash INITIAL_STATE:");
    println!("impl PowHash {{");
    println!("    #[rustfmt::skip]");
    println!("    const INITIAL_STATE: [u64; 25] = {}", state_to_rust_format(&powhash_state));
    println!("}}");
    
    // Generate HeavyHash initial state
    println!("\n\n3. HEAVYHASH INITIAL STATE (25 u64s)");
    println!("--------------------------------------------------------------------------------");
    let heavyhash_domain = b"JIO_HeavyHash";
    let heavyhash_state = generate_keccak_state(heavyhash_domain);
    println!("Domain: {}", String::from_utf8_lossy(heavyhash_domain));
    println!("First u64: 0x{:016x}\n", heavyhash_state[0]);
    println!("Rust code for KHeavyHash INITIAL_STATE:");
    println!("impl KHeavyHash {{");
    println!("    #[rustfmt::skip]");
    println!("    const INITIAL_STATE: [u64; 25] = {}", state_to_rust_format(&heavyhash_state));
    println!("}}");
    
    // Summary
    println!("\n\n================================================================================");
    println!("SUMMARY");
    println!("================================================================================");
    println!("\nDomains used:");
    for (name, domain) in &domains {
        println!("  {:15} = {}", name, String::from_utf8_lossy(domain));
    }
    
    println!("\n✓ All seeds generated successfully!");
    println!("\nNext steps:");
    println!("  1. Copy the Rust code above to src/pow_hashers.rs");
    println!("  2. Run 'cargo test' to verify the seeds work correctly");
    println!("  3. Document these domains in your network specification");
}
