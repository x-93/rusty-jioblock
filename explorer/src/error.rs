//! Error types for the explorer

use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Error, Debug)]
pub enum ExplorerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Cache error: {0}")]
    Cache(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, ExplorerError>;

impl From<rpc_core::RpcError> for ExplorerError {
    fn from(err: rpc_core::RpcError) -> Self {
        ExplorerError::Rpc(err.to_string())
    }
}

impl IntoResponse for ExplorerError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ExplorerError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            ExplorerError::Rpc(_) => (StatusCode::INTERNAL_SERVER_ERROR, "RPC error"),
            ExplorerError::Cache(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Cache error"),
            ExplorerError::Serialization(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Serialization error"),
            ExplorerError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error"),
            ExplorerError::NotFound(_) => (StatusCode::NOT_FOUND, "Not found"),
            ExplorerError::InvalidInput(_) => (StatusCode::BAD_REQUEST, "Invalid input"),
            ExplorerError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };

        let body = Json(json!({
            "error": error_message,
            "message": self.to_string(),
        }));

        (status, body).into_response()
    }
}

