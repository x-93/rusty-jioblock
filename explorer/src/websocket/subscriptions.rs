//! Subscription management for WebSocket

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use consensus_core::block::Block;

pub struct SubscriptionManager {
    block_sender: broadcast::Sender<Block>,
    subscriptions: Arc<RwLock<HashMap<String, usize>>>,
}

impl SubscriptionManager {
    pub fn new() -> Self {
        let (block_sender, _) = broadcast::channel(100);
        Self {
            block_sender,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn subscribe_blocks(&self) -> broadcast::Receiver<Block> {
        self.block_sender.subscribe()
    }
    
    pub async fn subscribe(&self, channel: &str) {
        let mut subs = self.subscriptions.write().await;
        *subs.entry(channel.to_string()).or_insert(0) += 1;
    }
    
    pub async fn unsubscribe(&self, channel: &str) {
        let mut subs = self.subscriptions.write().await;
        if let Some(count) = subs.get_mut(channel) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                subs.remove(channel);
            }
        }
    }
    
    pub fn broadcast_block(&self, block: Block) {
        let _ = self.block_sender.send(block);
    }
}

