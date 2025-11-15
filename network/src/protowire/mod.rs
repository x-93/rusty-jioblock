use bincode;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use consensus_core::block::Block;
use consensus_core::tx::Transaction;
use consensus_core::Hash;

pub const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024; // 16 MiB

/// Protowire message used by the network crate. Uses consensus_core's Block/Transaction/Hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Ping { nonce: u64 },
    Pong { nonce: u64 },
    Transaction(Transaction),
    Block(Block),
    InvBlock { hashes: Vec<Hash> },
    RequestBlocks { hashes: Vec<Hash> },
}

pub async fn write_frame(stream: &mut TcpStream, msg: &Message) -> Result<(), String> {
    let payload = bincode::serialize(msg).map_err(|e| format!("serialize: {}", e))?;
    if payload.len() > MAX_FRAME_SIZE {
        return Err("frame too large".into());
    }
    let len = payload.len() as u32;
    stream.write_u32_le(len).await.map_err(|e| e.to_string())?;
    stream.write_all(&payload).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn read_frame(stream: &mut TcpStream) -> Result<Message, String> {
    let len = stream.read_u32_le().await.map_err(|e| e.to_string())? as usize;
    if len > MAX_FRAME_SIZE {
        return Err("frame too large".into());
    }
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await.map_err(|e| e.to_string())?;
    let msg: Message = bincode::deserialize(&buf).map_err(|e| format!("deserialize: {}", e))?;
    Ok(msg)
}
