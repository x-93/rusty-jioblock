//! Validation module for consensus
//!
//! This module provides validation for blocks, headers, and transactions,
//! including context-free validation and contextual validation with UTXO set.

pub mod block_validator;
pub mod header_validator;
pub mod transaction_validator;
pub mod contextual;

pub use block_validator::BlockValidator;
pub use header_validator::HeaderValidator;
pub use transaction_validator::TransactionValidator;
pub use contextual::ContextualValidator;

