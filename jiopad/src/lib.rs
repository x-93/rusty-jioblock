//! JIOPad - JIO Blockchain Full Node Daemon
//!
//! This crate implements the main daemon that orchestrates all blockchain components
//! to run a full JIO node, including consensus, networking, mining, and RPC services.

pub mod cli;
pub mod config;
pub mod daemon;
pub mod rpc_server;

pub use config::Config;
pub use daemon::Daemon;
pub use cli::Args;
pub mod consensus_manager;
pub mod storage_manager;
pub mod sync_manager;
pub mod mining_coordinator;
pub mod mempool;
pub mod network_manager;
pub mod ui;

