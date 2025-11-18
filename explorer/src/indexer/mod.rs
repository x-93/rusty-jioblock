//! Indexer service for continuously indexing blockchain data

pub mod service;
pub mod block_indexer;
pub mod transaction_indexer;
pub mod address_indexer;

pub use service::IndexerService;

