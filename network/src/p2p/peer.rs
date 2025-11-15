use std::net::SocketAddr;
use std::sync::Arc;
use crate::protowire::Message;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum PeerState {
    Connecting,
    Connected,
    Ready,
    Disconnected,
}

#[derive(Clone)]
pub struct Peer {
    pub id: String,
    pub address: SocketAddr,
    pub tx: mpsc::Sender<Message>,
}

impl Peer {
    pub fn new(id: String, address: SocketAddr, tx: mpsc::Sender<Message>) -> Self {
        Self { id, address, tx }
    }

    pub async fn send_message(&self, msg: Message) -> Result<(), String> {
        self.tx.send(msg).await.map_err(|e| format!("send failed: {}", e))
    }
}
