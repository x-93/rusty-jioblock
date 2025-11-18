//! JIO Blockchain Explorer Backend
//!
//! This crate provides the backend services for the JIO blockchain explorer,
//! including REST API, WebSocket server, and indexing service.

pub mod api;
pub mod indexer;
pub mod database;
pub mod models;
pub mod websocket;
pub mod cache;
pub mod error;

pub use error::{ExplorerError, Result};

// Type alias for database pool
pub type DbPool = sqlx::SqlitePool;

pub mod rpc_client;
