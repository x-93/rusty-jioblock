//! Block-related routes

use axum::{
    Router,
    routing::get,
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::database::Database;
use crate::database::queries::BlockQueries;
use crate::models::PaginatedResponse;
use crate::error::Result;

#[derive(Deserialize)]
struct PaginationParams {
    page: Option<i32>,
    page_size: Option<i32>,
}

pub fn routes(database: Arc<Database>) -> Router {
    Router::new()
        .route("/blocks", get(list_blocks))
        .route("/blocks/:hash", get(get_block_by_hash))
        .route("/blocks/height/:height", get(get_block_by_height))
        .route("/blocks/recent", get(get_recent_blocks))
        .with_state(database)
}

#[axum::debug_handler]
async fn list_blocks(
    State(db): State<Arc<Database>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<crate::models::BlockSummary>>> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * page_size;

    let pool = Arc::new(db.pool().clone());
    let blocks = BlockQueries::list_recent(pool.clone(), page_size as i64, offset as i64).await?;
    let total = BlockQueries::count(pool).await?;
    let total_pages = (total as f64 / page_size as f64).ceil() as i32;

    Ok(Json(PaginatedResponse {
        data: blocks,
        total,
        page,
        page_size,
        total_pages,
    }))
}

#[axum::debug_handler]
async fn get_block_by_hash(
    State(db): State<Arc<Database>>,
    Path(hash): Path<String>,
) -> Result<Json<Option<crate::models::BlockSummary>>> {
    let pool = Arc::new(db.pool().clone());
    let block = BlockQueries::get_by_hash(pool, &hash).await?;
    Ok(Json(block))
}

#[axum::debug_handler]
async fn get_block_by_height(
    State(db): State<Arc<Database>>,
    Path(height): Path<i64>,
) -> Result<Json<Option<crate::models::BlockSummary>>> {
    let pool = Arc::new(db.pool().clone());
    let block = BlockQueries::get_by_height(pool, height).await?;
    Ok(Json(block))
}

#[axum::debug_handler]
async fn get_recent_blocks(
    State(db): State<Arc<Database>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<crate::models::BlockSummary>>> {
    let page_size = params.page_size.unwrap_or(10).min(50);
    let pool = Arc::new(db.pool().clone());
    let blocks = BlockQueries::list_recent(pool, page_size as i64, 0).await?;
    let total = blocks.len() as i64;

    Ok(Json(PaginatedResponse {
        data: blocks,
        total,
        page: 1,
        page_size,
        total_pages: 1,
    }))
}
