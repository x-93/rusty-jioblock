//! Transaction-related routes

use axum::{
    Router,
    routing::get,
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::database::Database;
use crate::database::queries::TransactionQueries;
use crate::models::PaginatedResponse;
use crate::error::Result;

#[derive(Deserialize)]
struct PaginationParams {
    page: Option<i32>,
    page_size: Option<i32>,
}

pub fn routes(database: Arc<Database>) -> Router {
    Router::new()
        .route("/transactions", get(list_transactions))
        .route("/transactions/:hash", get(get_transaction_by_hash))
        .route("/transactions/pending", get(get_pending_transactions))
        .with_state(database)
}

#[axum::debug_handler]
async fn list_transactions(
    State(db): State<Arc<Database>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<crate::models::TransactionSummary>>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * page_size;

    let pool = Arc::new(db.pool().clone());
    let txs = TransactionQueries::list_recent(pool.clone(), page_size as i64, offset as i64).await?;
    let total = TransactionQueries::count(pool).await?;

    Ok(Json(PaginatedResponse {
        data: txs,
        total,
        page,
        page_size,
        total_pages: (total as f64 / page_size as f64).ceil() as i32,
    }))
}

#[axum::debug_handler]
async fn get_transaction_by_hash(
    State(db): State<Arc<Database>>,
    Path(hash): Path<String>,
) -> Result<Json<Option<crate::models::TransactionSummary>>> {
    let pool = Arc::new(db.pool().clone());
    let tx = TransactionQueries::get_by_hash(pool, &hash).await?;
    Ok(Json(tx))
}

#[axum::debug_handler]
async fn get_pending_transactions(
    State(db): State<Arc<Database>>,
) -> Result<Json<Vec<crate::models::TransactionSummary>>> {
    let pool = Arc::new(db.pool().clone());
    let txs = TransactionQueries::list_pending(pool).await?;
    Ok(Json(txs))
}
