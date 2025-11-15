use serde::{Deserialize, Serialize};

/// Legacy/simple network parameters (kept for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkParams {
    /// The name of the network (e.g. "mainnet", "testnet", "devnet")
    pub network: String,
    /// Network magic number to identify network messages
    pub network_id: u32,
    /// Initial block subsidy in sompi
    pub block_subsidy: u64,
    /// Initial block difficulty target
    pub initial_difficulty: u32,
}

/// Consensus parameters used by various subsystems (mass, mempool, etc.)
///
/// This struct contains only the fields required by the current codebase.
/// If you need additional consensus parameters, add them here.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Params {
    /// Network identifier (string), kept for convenience
    pub network: String,
    /// Numeric network id
    pub network_id: u32,
    /// Block subsidy in sompis
    pub block_subsidy: u64,
    /// Initial difficulty target
    pub initial_difficulty: u32,

    /* Fields required by mass calculator */
    /// Mass charged per transaction byte
    pub mass_per_tx_byte: u64,
    /// Mass charged per script pubkey byte
    pub mass_per_script_pub_key_byte: u64,
    /// Mass charged per signature operation
    pub mass_per_sig_op: u64,
    /// Storage mass parameter (storm parameter)
    pub storage_mass_parameter: u64,
}