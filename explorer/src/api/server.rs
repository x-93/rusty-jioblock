//! API server implementation

use axum::{
    Router,
    http::Method,
};
use tower_http::cors::{CorsLayer, Any};
use std::sync::Arc;
use crate::database::Database;
use rpc_core::RpcApi;
use crate::api::routes;
use crate::error::Result;

pub struct ApiServer {
    database: Arc<Database>,
    rpc_client: Arc<dyn RpcApi>,
    port: u16,
}

impl ApiServer {
    pub fn new(database: Arc<Database>, rpc_client: Arc<dyn RpcApi>, port: u16) -> Self {
        Self { database, rpc_client, port }
    }

    pub fn router(&self) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(Any);

        Router::new()
            .nest("/api/v1", Router::new()
                .merge(routes::blocks::routes(self.database.clone()))
                .merge(routes::transactions::routes(self.database.clone()))
                .merge(routes::addresses::routes(self.database.clone()))
                .merge(routes::stats::routes(self.database.clone(), self.rpc_client.clone()))
                .merge(routes::search::routes(self.database.clone()))
            )
            .layer(cors)
    }

    pub async fn start(&self) -> Result<()> {
        let app = self.router();
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&addr).await
            .map_err(|e| crate::error::ExplorerError::Internal(format!("Failed to bind: {}", e)))?;

        tracing::info!("API server listening on {}", addr);

        axum::serve(listener, app).await
            .map_err(|e| crate::error::ExplorerError::Internal(format!("Server error: {}", e)))?;

        Ok(())
    }
}
