//! WebSocket server implementation

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use consensus_core::block::Block;
use crate::websocket::subscriptions::SubscriptionManager;
use crate::error::Result;

pub struct WSServer {
    subscription_manager: Arc<SubscriptionManager>,
    block_receiver: broadcast::Receiver<Block>,
}

impl WSServer {
    pub fn new(block_receiver: broadcast::Receiver<Block>) -> Self {
        Self {
            subscription_manager: Arc::new(SubscriptionManager::new()),
            block_receiver,
        }
    }
    
    pub async fn handle_connection(ws: WebSocketUpgrade, state: Arc<SubscriptionManager>) -> Response {
        ws.on_upgrade(|socket| Self::handle_socket(socket, state))
    }
    
    async fn handle_socket(socket: WebSocket, manager: Arc<SubscriptionManager>) {
        let (mut sender, mut receiver) = socket.split();
        let mut block_rx = manager.subscribe_blocks();
        
        // Handle incoming messages
        let manager_clone = manager.clone();
        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                if let Message::Text(text) = msg {
                    if let Ok(cmd) = serde_json::from_str::<WSCommand>(&text) {
                        match cmd {
                            WSCommand::Subscribe { channel } => {
                                manager_clone.subscribe(&channel).await;
                            }
                            WSCommand::Unsubscribe { channel } => {
                                manager_clone.unsubscribe(&channel).await;
                            }
                        }
                    }
                }
            }
        });
        
        // Handle outgoing messages (block broadcasts)
        let mut send_task = tokio::spawn(async move {
            while let Ok(block) = block_rx.recv().await {
                let event = WSEvent {
                    channel: "blocks:new".to_string(),
                    data: serde_json::json!({
                        "hash": block.header.hash.to_string(),
                        "height": 0, // TODO: Get height
                        "timestamp": block.header.timestamp,
                        "txCount": block.transactions.len(),
                    }),
                };
                
                if let Ok(json) = serde_json::to_string(&event) {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        });
        
        tokio::select! {
            _ = recv_task => {}
            _ = send_task => {}
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum WSCommand {
    Subscribe { channel: String },
    Unsubscribe { channel: String },
}

#[derive(serde::Serialize)]
struct WSEvent {
    channel: String,
    data: serde_json::Value,
}

