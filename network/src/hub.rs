use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc;
use crate::protowire::Message;
use crate::p2p::Peer;

pub struct Hub {
    peers: Arc<RwLock<HashMap<String, Arc<Peer>>>>,
}

impl Hub {
    pub fn new() -> Self {
        Self { peers: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn add_peer(&self, peer: Arc<Peer>) {
        self.peers.write().await.insert(peer.id.clone(), peer);
    }

    pub async fn broadcast(&self, msg: Message) {
        let peers = self.peers.read().await;
        for p in peers.values() {
            let _ = p.send_message(msg.clone()).await;
        }
    }
}
