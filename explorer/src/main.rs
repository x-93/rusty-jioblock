//! JIO Blockchain Explorer - Main entry point

use std::sync::Arc;
use tracing::{info, error};
use jio_explorer::{
    database::Database,
    api::ApiServer,
    indexer::IndexerService,
    error::Result,
    rpc_client::RpcClient,
};
use rpc_core::RpcApi;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting JIO Blockchain Explorer");

    // Database connection
    let database_path = std::env::var("DATABASE_URL")
        .map(|url| {
            if url.starts_with("sqlite:") {
                std::path::PathBuf::from(url.trim_start_matches("sqlite:"))
            } else {
                std::path::PathBuf::from(url)
            }
        })
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap()
                .join("jio_explorer.db")
        });

    info!("Database path: {:?}", database_path);
    let database = Arc::new(Database::new(&database_path).await?);
    info!("Connected to database");

    // Run migrations
    database.migrate().await?;
    info!("Database migrations completed");

    // Connect to JIOPad daemon via RPC
    let jiopad_url = std::env::var("JIOPAD_RPC_URL")
        .unwrap_or_else(|_| "ws://localhost:16110".to_string());
    info!("Connecting to JIOPad daemon at: {}", jiopad_url);

    let coordinator: Arc<dyn RpcApi> = Arc::new(RpcClient::new(&jiopad_url)
        .map_err(|e| jio_explorer::error::ExplorerError::Internal(format!("Failed to create RPC client: {}", e)))?);

    // Start indexer service
    let indexer = IndexerService::new(database.clone());
    let coordinator_clone = Arc::clone(&coordinator);
    tokio::spawn(async move {
        if let Err(e) = indexer.start(coordinator_clone).await {
            error!("Indexer error: {:?}", e);
        }
    });

    // Start API server
    let api_server = ApiServer::new(database.clone(), coordinator.clone(), 3000);
    info!("Starting API server on port 3000");
    api_server.start().await?;

    Ok(())
}
