//! Search routes

use axum::{
    Router,
    routing::get,
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::database::Database;
use crate::database::queries::{BlockQueries, TransactionQueries, AddressQueries};
use crate::error::Result;

#[derive(Deserialize)]
struct SearchParams {
    q: String,
}

pub fn routes(database: Arc<Database>) -> Router {
    Router::new()
        .route("/search", get(search))
        .with_state(database)
}

#[axum::debug_handler]
async fn search(
    State(db): State<Arc<Database>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<crate::models::SearchResults>> {
    let query = params.q.trim();
    if query.is_empty() {
        return Ok(Json(crate::models::SearchResults {
            blocks: vec![],
            transactions: vec![],
            addresses: vec![],
            total: 0,
        }));
    }

    let pool = Arc::new(db.pool().clone());

    // Search blocks by hash
    let blocks = if query.len() >= 10 {
        BlockQueries::get_by_hash(pool.clone(), query).await?
            .map(|b| vec![b])
            .unwrap_or_default()
    } else {
        vec![]
    };

    // Search transactions by hash
    let transactions = if query.len() >= 10 {
        TransactionQueries::get_by_hash(pool.clone(), query).await?
            .map(|t| vec![t])
            .unwrap_or_default()
    } else {
        vec![]
    };

    // Search addresses
    let addresses = AddressQueries::get_summary(pool, query).await?
        .map(|a| vec![a])
        .unwrap_or_default();

    let total = blocks.len() + transactions.len() + addresses.len();

    Ok(Json(crate::models::SearchResults {
        blocks,
        transactions,
        addresses,
        total,
    }))
}
