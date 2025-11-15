//! Block processing pipeline for consensus
//!
//! This module provides the block processing pipeline that orchestrates
//! validation, GHOSTDAG calculation, and state updates.

pub mod block_processor;
pub mod header_processor;
pub mod body_processor;
pub mod virtual_processor;
pub mod deps_manager;

pub mod flow;

pub use block_processor::BlockProcessor;
pub use header_processor::HeaderProcessor;
pub use body_processor::BodyProcessor;
pub use virtual_processor::VirtualProcessor;
pub use deps_manager::DepsManager;

