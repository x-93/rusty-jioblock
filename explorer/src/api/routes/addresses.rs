//! Address-related routes

use axum::{
    Router,
    routing::get,
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::database::Database;
use crate::database::queries::AddressQueries;
use crate::models::PaginatedResponse;
use crate::error::Result;

#[derive(Deserialize)]
struct PaginationParams {
    page: Option<i32>,
    page_size: Option<i32>,
}

pub fn routes(database: Arc<Database>) -> Router {
    Router::new()
        .route("/addresses/:address", get(get_address))
        .route("/addresses/:address/transactions", get(get_address_transactions))
        .with_state(database)
}

#[axum::debug_handler]
async fn get_address(
    State(db): State<Arc<Database>>,
    Path(address): Path<String>,
) -> Result<Json<Option<crate::models::AddressSummary>>> {
    let pool = Arc::new(db.pool().clone());
    let addr = AddressQueries::get_summary(pool.clone(), &address).await?;
    Ok(Json(addr))
}

#[axum::debug_handler]
async fn get_address_transactions(
    State(db): State<Arc<Database>>,
    Path(address): Path<String>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<crate::models::TransactionSummary>>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * page_size;

    let pool = Arc::new(db.pool().clone());
    let txs = AddressQueries::get_transactions(pool, &address, page_size as i64, offset as i64).await?;
    let total = txs.len() as i64; // TODO: Get actual count

    Ok(Json(PaginatedResponse {
        data: txs,
        total,
        page,
        page_size,
        total_pages: (total as f64 / page_size as f64).ceil() as i32,
    }))
}
