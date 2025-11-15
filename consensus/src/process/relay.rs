//! Block relay process
//!
//! This module implements block and transaction relay to peers,
//! including announcement protocols and peer management.

use consensus_core::block::Block;
use consensus_core::tx::Transaction;
use consensus_core::Hash;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Block relay process
pub struct RelayProcess {
    /// Known blocks that have been announced
    announced_blocks: RwLock<HashSet<Hash>>,
    /// Known transactions that have been announced
    announced_transactions: RwLock<HashSet<Hash>>,
    /// Connected peers with their connection state
    peers: RwLock<Vec<PeerInfo>>,
    /// Maximum number of peers to connect to
    max_peers: usize,
    /// Minimum number of peers to maintain
    min_peers: usize,
}

/// Peer information
#[derive(Clone, Debug)]
pub struct PeerInfo {
    /// Peer ID
    pub id: String,
    /// Peer address
    pub address: String,
    /// Last seen timestamp
    pub last_seen: u64,
}

impl RelayProcess {
    /// Create a new relay process with default peer limits
    pub fn new() -> Self {
        Self::with_limits(8, 3) // Default: max 8 peers, min 3 peers
    }

    /// Create a new relay process with custom peer limits
    pub fn with_limits(max_peers: usize, min_peers: usize) -> Self {
        Self {
            announced_blocks: RwLock::new(HashSet::new()),
            announced_transactions: RwLock::new(HashSet::new()),
            peers: RwLock::new(Vec::new()),
            max_peers,
            min_peers,
        }
    }

    /// Announce a new block to all peers
    pub fn announce_block(&self, block: &Block) -> Result<(), String> {
        let hash = block.header.hash;

        // Check if already announced
        {
            let announced = self.announced_blocks.read().unwrap();
            if announced.contains(&hash) {
                return Ok(()); // Already announced
            }
        }

        // Add to announced set
        {
            let mut announced = self.announced_blocks.write().unwrap();
            announced.insert(hash);
        }

        // Send block announcement to all connected peers
        let peers = self.peers.read().unwrap();
        for peer in peers.iter() {
            self.send_block_announcement_to_peer(&peer.id, &hash)?;
        }

        Ok(())
    }

    /// Announce a new transaction to all peers
    pub fn announce_transaction(&self, transaction: &Transaction) -> Result<(), String> {
        let hash = transaction.hash();

        // Check if already announced
        {
            let announced = self.announced_transactions.read().unwrap();
            if announced.contains(&hash) {
                return Ok(()); // Already announced
            }
        }

        // Add to announced set
        {
            let mut announced = self.announced_transactions.write().unwrap();
            announced.insert(hash);
        }

        // Send transaction announcement to all connected peers
        let peers = self.peers.read().unwrap();
        for peer in peers.iter() {
            self.send_transaction_announcement_to_peer(&peer.id, &hash)?;
        }

        Ok(())
    }

    /// Handle incoming block announcement from peer
    pub fn handle_block_announcement(&self, peer_id: &str, block_hash: Hash) -> Result<(), String> {
        // Check if we already have this block announced
        {
            let announced = self.announced_blocks.read().unwrap();
            if announced.contains(&block_hash) {
                return Ok(()); // Already know about this block
            }
        }

        // Check if we already have this block in our store
        // In a real implementation, this would check the block store
        let have_block = self.check_have_block(&block_hash);

        if !have_block {
            // Request the block from the peer
            self.send_block_request_to_peer(peer_id, &block_hash)?;
        } else {
            // We have the block, mark it as announced to avoid re-processing
            let mut announced = self.announced_blocks.write().unwrap();
            announced.insert(block_hash);
        }

        Ok(())
    }

    /// Handle incoming transaction announcement from peer
    pub fn handle_transaction_announcement(&self, peer_id: &str, tx_hash: Hash) -> Result<(), String> {
        // Check if we already have this transaction announced
        {
            let announced = self.announced_transactions.read().unwrap();
            if announced.contains(&tx_hash) {
                return Ok(()); // Already know about this transaction
            }
        }

        // Check if we already have this transaction in our mempool
        // In a real implementation, this would check the mempool
        let have_tx = self.check_have_transaction(&tx_hash);

        if !have_tx {
            // Request the transaction from the peer
            self.send_transaction_request_to_peer(peer_id, &tx_hash)?;
        } else {
            // We have the transaction, mark it as announced to avoid re-processing
            let mut announced = self.announced_transactions.write().unwrap();
            announced.insert(tx_hash);
        }

        Ok(())
    }

    /// Add a new peer
    pub fn add_peer(&self, peer_info: PeerInfo) {
        let mut peers = self.peers.write().unwrap();
        peers.push(peer_info);
    }

    /// Remove a peer
    pub fn remove_peer(&self, peer_id: &str) {
        let mut peers = self.peers.write().unwrap();
        peers.retain(|p| p.id != peer_id);
    }

    /// Get list of connected peers
    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.read().unwrap().clone()
    }

    /// Get relay statistics
    pub fn get_stats(&self) -> RelayStats {
        let announced_blocks = self.announced_blocks.read().unwrap().len();
        let announced_txs = self.announced_transactions.read().unwrap().len();
        let peer_count = self.peers.read().unwrap().len();

        RelayStats {
            announced_blocks,
            announced_transactions: announced_txs,
            connected_peers: peer_count,
        }
    }

    /// Send block announcement to a specific peer
    fn send_block_announcement_to_peer(&self, peer_id: &str, block_hash: &Hash) -> Result<(), String> {
        // In a real implementation, this would send a network message
        // For now, simulate the network call
        println!("Sending block announcement {} to peer {}", block_hash, peer_id);
        Ok(())
    }

    /// Send transaction announcement to a specific peer
    fn send_transaction_announcement_to_peer(&self, peer_id: &str, tx_hash: &Hash) -> Result<(), String> {
        // In a real implementation, this would send a network message
        // For now, simulate the network call
        println!("Sending transaction announcement {} to peer {}", tx_hash, peer_id);
        Ok(())
    }

    /// Send block request to a specific peer
    fn send_block_request_to_peer(&self, peer_id: &str, block_hash: &Hash) -> Result<(), String> {
        // In a real implementation, this would send a network message
        // For now, simulate the network call
        println!("Sending block request {} to peer {}", block_hash, peer_id);
        Ok(())
    }

    /// Send transaction request to a specific peer
    fn send_transaction_request_to_peer(&self, peer_id: &str, tx_hash: &Hash) -> Result<(), String> {
        // In a real implementation, this would send a network message
        // For now, simulate the network call
        println!("Sending transaction request {} to peer {}", tx_hash, peer_id);
        Ok(())
    }

    /// Check if we have a block (placeholder for block store integration)
    fn check_have_block(&self, _block_hash: &Hash) -> bool {
        // In a real implementation, this would query the block store
        false // Placeholder: assume we don't have the block
    }

    /// Check if we have a transaction (placeholder for mempool integration)
    fn check_have_transaction(&self, _tx_hash: &Hash) -> bool {
        // In a real implementation, this would query the mempool
        false // Placeholder: assume we don't have the transaction
    }

    /// Check if we need more peers
    pub fn needs_more_peers(&self) -> bool {
        let peer_count = self.peers.read().unwrap().len();
        peer_count < self.min_peers
    }

    /// Check if we can accept more peers
    pub fn can_accept_more_peers(&self) -> bool {
        let peer_count = self.peers.read().unwrap().len();
        peer_count < self.max_peers
    }

    /// Update peer last seen timestamp
    pub fn update_peer_timestamp(&self, peer_id: &str) {
        let mut peers = self.peers.write().unwrap();
        if let Some(peer) = peers.iter_mut().find(|p| p.id == peer_id) {
            peer.last_seen = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
    }
}

/// Relay statistics
#[derive(Debug, Clone)]
pub struct RelayStats {
    /// Number of blocks announced
    pub announced_blocks: usize,
    /// Number of transactions announced
    pub announced_transactions: usize,
    /// Number of connected peers
    pub connected_peers: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
use consensus_core::block::Block;
use consensus_core::header::Header as BlockHeader;
use consensus_core::tx::Transaction;

    fn create_test_block() -> Block {
        Block {
            header: BlockHeader::from_precomputed_hash(
                Hash::from_le_u64([1, 0, 0, 0]),
                vec![],
            ),
            transactions: vec![],
        }
    }

    fn create_test_transaction() -> Transaction {
        // Use the consensus_core Transaction constructor
        Transaction::new(
            1,
            vec![],
            vec![],
            0,
            consensus_core::subnets::SubnetworkId::default(),
            0,
            vec![],
        )
    }

    #[test]
    fn test_relay_process_creation() {
        let relay = RelayProcess::new();
        assert!(relay.needs_more_peers()); // Should need peers initially
        assert!(relay.can_accept_more_peers()); // Should accept peers initially
    }

    #[test]
    fn test_relay_process_with_limits() {
        let relay = RelayProcess::with_limits(5, 2);
        assert!(relay.needs_more_peers());
        assert!(relay.can_accept_more_peers());
    }

    #[test]
    fn test_add_and_remove_peers() {
        let relay = RelayProcess::new();

        // Add peers
        let peer1 = PeerInfo {
            id: "peer1".to_string(),
            address: "127.0.0.1:8333".to_string(),
            last_seen: 1234567890,
        };
        let peer2 = PeerInfo {
            id: "peer2".to_string(),
            address: "127.0.0.1:8334".to_string(),
            last_seen: 1234567891,
        };

        relay.add_peer(peer1.clone());
        relay.add_peer(peer2.clone());

        let peers = relay.get_peers();
        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0].id, "peer1");
        assert_eq!(peers[1].id, "peer2");

        // Remove a peer
        relay.remove_peer("peer1");
        let peers = relay.get_peers();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].id, "peer2");
    }

    #[test]
    fn test_announce_block() {
        let relay = RelayProcess::new();
        let block = create_test_block();

        // Should succeed even without peers
        let result = relay.announce_block(&block);
        assert!(result.is_ok());

        // Check stats
        let stats = relay.get_stats();
        assert_eq!(stats.announced_blocks, 1);
        assert_eq!(stats.announced_transactions, 0);
    }

    #[test]
    fn test_announce_transaction() {
        let relay = RelayProcess::new();
        let tx = create_test_transaction();

        // Should succeed even without peers
        let result = relay.announce_transaction(&tx);
        assert!(result.is_ok());

        // Check stats
        let stats = relay.get_stats();
        assert_eq!(stats.announced_blocks, 0);
        assert_eq!(stats.announced_transactions, 1);
    }

    #[test]
    fn test_handle_block_announcement() {
        let relay = RelayProcess::new();
        let block_hash = Hash::from_le_u64([1, 2, 3, 4]);

        // Should succeed
        let result = relay.handle_block_announcement("peer1", block_hash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_transaction_announcement() {
        let relay = RelayProcess::new();
        let tx_hash = Hash::from_le_u64([5, 6, 7, 8]);

        // Should succeed
        let result = relay.handle_transaction_announcement("peer1", tx_hash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_peer_limits() {
        let relay = RelayProcess::with_limits(2, 1);

        // Initially needs peers
        assert!(relay.needs_more_peers());
        assert!(relay.can_accept_more_peers());

        // Add one peer
        let peer = PeerInfo {
            id: "peer1".to_string(),
            address: "127.0.0.1:8333".to_string(),
            last_seen: 1234567890,
        };
        relay.add_peer(peer);

        // Still needs more peers (min_peers = 1, we have 1, but min is 1 so needs_more should be false? Wait)
        // Wait, needs_more_peers checks if peer_count < min_peers
        // With 1 peer and min_peers = 1, needs_more_peers should be false
        assert!(!relay.needs_more_peers());
        assert!(relay.can_accept_more_peers()); // Can accept up to 2

        // Add second peer
        let peer2 = PeerInfo {
            id: "peer2".to_string(),
            address: "127.0.0.1:8334".to_string(),
            last_seen: 1234567891,
        };
        relay.add_peer(peer2);

        // Now at max peers
        assert!(!relay.needs_more_peers());
        assert!(!relay.can_accept_more_peers());
    }

    #[test]
    fn test_update_peer_timestamp() {
        let relay = RelayProcess::new();
        let peer = PeerInfo {
            id: "peer1".to_string(),
            address: "127.0.0.1:8333".to_string(),
            last_seen: 1234567890,
        };
        relay.add_peer(peer);

        // Update timestamp
        relay.update_peer_timestamp("peer1");

        let peers = relay.get_peers();
        assert_eq!(peers.len(), 1);
        assert!(peers[0].last_seen > 1234567890); // Should be updated
    }
}
