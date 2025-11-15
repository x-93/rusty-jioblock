use crate::{block::Block, header::Header, subnets::SUBNETWORK_ID_COINBASE, tx::{Transaction, TransactionOutput, ScriptPublicKey}};
use crate::{Hash, ZERO_HASH};
use crate::merkle::MerkleTree;
use crate::constants::{INITIAL_BLOCK_REWARD, SOMPI_PER_JIO};

/// The constants uniquely representing the genesis block
#[derive(Clone, Debug)]
pub struct GenesisBlock {
    pub hash: Hash,
    pub version: u16,
    pub hash_merkle_root: Hash,
    pub utxo_commitment: Hash,
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u64,
    pub daa_score: u64,
    pub coinbase_payload: &'static [u8],
}

impl GenesisBlock {
    pub fn build_genesis_transactions(&self) -> Vec<Transaction> {
        // Create a coinbase transaction with a single output paying the initial block reward
        let reward = INITIAL_BLOCK_REWARD * SOMPI_PER_JIO;
        let output = TransactionOutput::new(
            reward,
            ScriptPublicKey::from_vec(0, Vec::new()),
        );
        vec![Transaction::new(0, Vec::new(), vec![output], 0, SUBNETWORK_ID_COINBASE, 0, self.coinbase_payload.to_vec())]
    }
}

impl From<&GenesisBlock> for Header {
    fn from(genesis: &GenesisBlock) -> Self {
        Header::new_finalized(
            genesis.version,
            Vec::new(),
            genesis.hash_merkle_root,
            ZERO_HASH,
            genesis.utxo_commitment,
            genesis.timestamp,
            genesis.bits,
            genesis.nonce,
            genesis.daa_score,
            0.into(),
            0,
            ZERO_HASH,
        )
    }
}

impl From<&GenesisBlock> for Block {
    fn from(genesis: &GenesisBlock) -> Self {
        Block::new(genesis.into(), genesis.build_genesis_transactions())
    }
}

impl From<(&Header, &'static [u8])> for GenesisBlock {
    fn from((header, payload): (&Header, &'static [u8])) -> Self {
        Self {
            hash: header.hash,
            version: header.version,
            hash_merkle_root: header.hash_merkle_root,
            utxo_commitment: header.utxo_commitment,
            timestamp: header.timestamp,
            bits: header.bits,
            nonce: header.nonce,
            daa_score: header.daa_score,
            coinbase_payload: payload,
        }
    }
}

// A simple default genesis for mainnet/dev purposes.
// Update these values to match the canonical genesis for each network.
pub fn default_genesis() -> GenesisBlock {
    // Deterministic canonical genesis generation
    static COINBASE_PAYLOAD: &[u8] = b"Jio deterministic genesis - 2025-11-12";

    // Build the coinbase transaction used for merkle root calculation
    let reward = INITIAL_BLOCK_REWARD * SOMPI_PER_JIO;
    let coinbase_tx = Transaction::new(
        0,
        Vec::new(),
        vec![TransactionOutput::new(reward, ScriptPublicKey::from_vec(0, Vec::new()))],
        0,
        SUBNETWORK_ID_COINBASE,
        0,
        COINBASE_PAYLOAD.to_vec(),
    );

    // Compute merkle root from the coinbase transaction hash
    let tx_hash = coinbase_tx.id();
    let merkle_root = MerkleTree::from_hashes(vec![tx_hash]).root();

    // Use the tx hash as a simple utxo commitment placeholder
    let utxo_commitment = tx_hash;

    use crate::header::Header;
    use crate::BlueWorkType;

    // Use the canonical testnet genesis values (timestamp/nonce/bits) so the
    // serialized genesis is stable for testnet and matches pre-mined values.
    let header = Header::new_finalized(
        1,                    // version
        Vec::new(),           // parents_by_level
        merkle_root,          // hash_merkle_root
        ZERO_HASH,            // accepted_id_merkle_root
        utxo_commitment,      // utxo_commitment
        1762971421786u64,     // timestamp (ms since epoch) - user-specified
        0x1f00_ffff,          // bits
        38922u64,             // nonce (user-specified)
        0,                    // daa_score
        BlueWorkType::from(0u64),
        0,
        ZERO_HASH,
    );

    GenesisBlock::from((&header, COINBASE_PAYLOAD))
}
