//! Statistics routes

use axum::{
    Router,
    routing::get,
    extract::State,
    Json,
};
use std::sync::Arc;
use crate::database::Database;

use crate::error::Result;
use rpc_core::RpcApi;

#[derive(Clone)]
pub struct StatsState {
    pub database: Arc<Database>,
    pub rpc_client: Arc<dyn RpcApi>,
}

pub fn routes(database: Arc<Database>, rpc_client: Arc<dyn RpcApi>) -> Router {
    let state = StatsState { database, rpc_client };
    Router::new()
        .route("/stats/network", get(get_network_stats))
        .route("/stats/mining", get(get_mining_stats))
        .route("/stats/blockdag", get(get_blockdag_stats))
        .with_state(state)
}

#[axum::debug_handler]
async fn get_network_stats(
    State(state): State<StatsState>,
) -> Result<Json<crate::models::NetworkStats>> {
    let pool = Arc::new(state.database.pool().clone());

    // Get basic stats from database
    let block_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM blocks"
    )
    .fetch_one(&*pool)
    .await?;

    let tx_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM transactions"
    )
    .fetch_one(&*pool)
    .await?;

    let address_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM addresses"
    )
    .fetch_one(&*pool)
    .await?;

    let total_supply = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(coinbase_value), 0) FROM blocks"
    )
    .fetch_one(&*pool)
    .await?;

    let mempool_size = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM mempool_transactions"
    )
    .fetch_one(&*pool)
    .await?;

    let mempool_bytes = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(size), 0) FROM mempool_transactions"
    )
    .fetch_one(&*pool)
    .await?;

    Ok(Json(crate::models::NetworkStats {
        block_count,
        tx_count,
        address_count,
        total_supply,
        hashrate: None, // TODO: Calculate from recent blocks
        difficulty: None, // TODO: Get from latest block
        avg_block_time: None, // TODO: Calculate from block timestamps
        mempool_size: mempool_size as i32,
        mempool_bytes,
        peer_count: 0, // TODO: Get from network
        timestamp: chrono::Utc::now().timestamp(),
    }))
}

#[axum::debug_handler]
async fn get_mining_stats(
    State(state): State<StatsState>,
) -> Result<Json<crate::models::MiningInfo>> {
    // Get mining info from RPC
    match state.rpc_client.get_mining_info().await {
        Ok(mining_info) => Ok(Json(crate::models::MiningInfo {
            network_hash_ps: mining_info.network_hashrate as i64,
            pooled_tx: 0, // TODO: Get from RPC if available
            chain: "mainnet".to_string(),
            warnings: "".to_string(),
            difficulty: mining_info.difficulty,
            blocks: mining_info.blocks_mined as i64,
            current_block_weight: None,
            current_block_tx: None,
            errors: None,
        })),
        Err(e) => {
            tracing::warn!("Failed to get mining info from RPC: {:?}", e);
            // Return default values if RPC fails
            Ok(Json(crate::models::MiningInfo {
                network_hash_ps: 0,
                pooled_tx: 0,
                chain: "mainnet".to_string(),
                warnings: "".to_string(),
                difficulty: 1.0,
                blocks: 0,
                current_block_weight: None,
                current_block_tx: None,
                errors: Some(format!("RPC error: {}", e)),
            }))
        }
    }
}

#[axum::debug_handler]
async fn get_blockdag_stats(
    State(state): State<StatsState>,
) -> Result<Json<crate::models::BlockDagInfo>> {
    // Get blockDAG info from RPC
    match state.rpc_client.get_block_dag_info().await {
        Ok(blockdag_info) => Ok(Json(crate::models::BlockDagInfo {
            block_count: blockdag_info.block_count as i64,
            tip_hashes: blockdag_info.tip_hashes.into_iter().map(|h| h.to_string()).collect(),
            difficulty: blockdag_info.difficulty,
            network: blockdag_info.network,
            virtual_parent_hashes: blockdag_info.virtual_parent_hashes.into_iter().map(|h| h.to_string()).collect(),
            pruning_point_hash: blockdag_info.pruning_point_hash.to_string(),
        })),
        Err(e) => {
            tracing::warn!("Failed to get blockDAG info from RPC: {:?}", e);
            // Return default values if RPC fails
            Ok(Json(crate::models::BlockDagInfo {
                block_count: 0,
                tip_hashes: vec![],
                difficulty: 1.0,
                network: "mainnet".to_string(),
                virtual_parent_hashes: vec![],
                pruning_point_hash: "".to_string(),
            }))
        }
    }
}
