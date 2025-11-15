//! Validation flow for block processing
//!
//! This module provides validation flow orchestration for blocks.

use consensus_core::block::Block;
use consensus_core::errors::ConsensusError;
use crate::consensus::validation::BlockValidator;
use std::sync::Arc;

/// Validation flow for blocks
pub struct ValidationFlow {
    block_validator: Arc<BlockValidator>,
}

impl ValidationFlow {
    /// Create a new validation flow
    pub fn new(block_validator: Arc<BlockValidator>) -> Self {
        Self {
            block_validator,
        }
    }

    /// Validate a block
    pub fn validate(&self, block: &Block) -> Result<(), ConsensusError> {
        self.block_validator.validate_block(block)
    }

    /// Validate a block without proof of work (for testing)
    #[cfg(test)]
    pub fn validate_without_pow(&self, block: &Block) -> Result<(), ConsensusError> {
        self.block_validator.validate_block_without_pow(block)
    }

    /// Validate multiple blocks
    pub fn validate_batch(&self, blocks: &[Block]) -> Vec<Result<(), ConsensusError>> {
        blocks.iter()
            .map(|block| self.validate(block))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::validation::{HeaderValidator, TransactionValidator};
    use consensus_core::{ZERO_HASH, BlueWorkType};
    use consensus_core::tx::{Transaction, TransactionOutput, ScriptPublicKey};
    use consensus_core::subnets::SUBNETWORK_ID_COINBASE;
    use consensus_core::constants::BLOCK_VERSION;

    fn create_test_block() -> Block {
        // Create a coinbase transaction
        let coinbase = Transaction::new(
            1,
            Vec::new(),
            vec![TransactionOutput::new(
                5000000000,
                ScriptPublicKey::from_vec(0, Vec::new()),
            )],
            0,
            SUBNETWORK_ID_COINBASE,
            0,
            Vec::new(),
        );

        let header = consensus_core::header::Header::new_finalized(
            BLOCK_VERSION,
            vec![],
            ZERO_HASH,
            ZERO_HASH,
            ZERO_HASH,
            1000,
            0x1f00ffff,
            0,
            0,
            BlueWorkType::from(0u64),
            0,
            ZERO_HASH,
        );
        Block::new(header, vec![coinbase])
    }

    #[test]
    fn test_validation_flow() {
        let header_validator = Arc::new(HeaderValidator::new());
        let tx_validator = Arc::new(TransactionValidator::new());
        let block_validator = Arc::new(BlockValidator::new(header_validator, tx_validator));
        
        let flow = ValidationFlow::new(block_validator);
        let block = create_test_block();

        // Should validate successfully for a basic block with coinbase
        // Note: We skip PoW validation in tests since test headers don't have valid PoW
        let result = flow.validate_without_pow(&block);
        assert!(result.is_ok(), "Block validation failed: {:?}", result.err());
    }
}

