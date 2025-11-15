use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{BLOCK_VERSION, MAX_BLOCK_MASS},
    errors::ConsensusError,
    header::Header,
    hashing,
    tx::Transaction,
};
use crate::Hash;

/// Complete block structure including header and transactions
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    /// Block header containing metadata and parent information
    pub header: Header,
    /// List of transactions in the block
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Creates a new block with the given header and transactions
    pub fn new(header: Header, transactions: Vec<Transaction>) -> Self {
        Self { header, transactions }
    }

    /// Validates the block structure and basic rules
    pub fn validate(&self) -> Result<(), ConsensusError> {
        // Check block version
        if self.header.version != BLOCK_VERSION {
            return Err(ConsensusError::InvalidBlockVersion);
        }

        // Validate block mass (size)
        let mass = self.calculate_mass();
        if mass > MAX_BLOCK_MASS {
            return Err(ConsensusError::ExceedsMaxBlockMass);
        }

        // Validate merkle root matches transactions
        let merkle_root = self.calculate_merkle_root()?;
        if merkle_root != self.header.hash_merkle_root {
            return Err(ConsensusError::InvalidMerkleRoot);
        }

        // Validate proof of work
        if !self.validate_pow() {
            return Err(ConsensusError::InvalidProofOfWork);
        }

        // Validate there is exactly one coinbase transaction
        if !self.validate_coinbase() {
            return Err(ConsensusError::InvalidCoinbaseTransaction);
        }

        // Validate individual transactions
        for tx in &self.transactions {
            tx.validate()?;
        }

        Ok(())
    }

    /// Calculates the total mass (size) of the block
    pub fn calculate_mass(&self) -> u64 {
        // Header mass
        let mut mass = 
            std::mem::size_of::<Header>() as u64;

        // Add transaction masses
        for tx in &self.transactions {
            mass += tx.calculate_mass();
        }

        mass
    }

    /// Validates that the block meets proof of work requirements
    fn validate_pow(&self) -> bool {
        let hash = hashing::calculate_header_hash(&self.header);
        let hash_bytes = hash.as_bytes();
        let target = bits_to_target(self.header.bits);

        // Check if hash is below target (hash as big-endian uint256)
        for i in (0..32).rev() {
            if hash_bytes[i] < target[i] {
                return true;
            }
            if hash_bytes[i] > target[i] {
                return false;
            }
        }
        true
    }

    /// Validates coinbase transaction rules
    fn validate_coinbase(&self) -> bool {
        if self.transactions.is_empty() {
            return false;
        }

        // First transaction must be coinbase
        if !self.transactions[0].is_coinbase() {
            return false;
        }

        // No other transaction can be coinbase
        if self.transactions[1..].iter().any(|tx| tx.is_coinbase()) {
            return false;
        }

        true
    }

    /// Calculates the merkle root of the block's transactions
    fn calculate_merkle_root(&self) -> Result<Hash, ConsensusError> {
        use crate::merkle::MerkleTree;
        
        if self.transactions.is_empty() {
            return Err(ConsensusError::EmptyTransactionList);
        }

        let tx_hashes: Vec<_> = self
            .transactions
            .iter()
            .map(|tx| tx.hash())
            .collect();

        let merkle_tree = MerkleTree::from_hashes(tx_hashes);
        Ok(merkle_tree.root())
    }
}

/// Converts compact bits representation to target bytes
fn bits_to_target(bits: u32) -> [u8; 32] {
    let mut target = [0u8; 32];
    let exp = ((bits >> 24) & 0xff) as usize;
    let mantissa = bits & 0x00ff_ffff;
    
    if exp <= 32 {
        let offset = 32 - exp;
        target[offset - 3..offset].copy_from_slice(&mantissa.to_be_bytes()[1..]);
    }
    
    target
}