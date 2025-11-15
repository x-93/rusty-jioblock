use std::sync::Arc;
use consensus_core::{Hash, BlueWorkType};
use super::stores::{GhostdagData, GhostdagStore};
use super::protocol::GhostdagProtocol;

pub struct GhostdagManager {
    protocol: Arc<GhostdagProtocol>,
    store: Arc<GhostdagStore>,
}

impl GhostdagManager {
    pub fn new(protocol: Arc<GhostdagProtocol>, store: Arc<GhostdagStore>) -> Self {
        Self { protocol, store }
    }

    pub fn init_genesis(&self, genesis_hash: Hash) {
        let mut data = GhostdagData::new(genesis_hash);
        data.blue_score = 0;
        data.blue_work = BlueWorkType::from(0u64);
        data.selected_parent = genesis_hash;
        data.height = 0;
        self.store.insert(genesis_hash, data);
    }

    pub fn get_ghostdag_data(&self, hash: &Hash) -> Option<GhostdagData> {
        self.store.get(hash)
    }

    pub fn add_block(&self, header: &consensus_core::header::Header) -> Result<GhostdagData, String> {
        let data = self.protocol.calculate_ghostdag(header)?;
        self.store.insert(header.hash, data.clone());
        Ok(data)
    }

    pub fn get_blue_score(&self, hash: &Hash) -> Option<u64> {
        self.store.get(hash).map(|d| d.blue_score)
    }

    pub fn get_selected_parent(&self, hash: &Hash) -> Option<Hash> {
        self.store.get(hash).map(|d| d.selected_parent)
    }

    pub fn get_virtual_ghostdag_data(&self, tips: Vec<Hash>) -> Result<GhostdagData, String> {
        let virtual_hash = Self::calculate_virtual_hash(&tips);
        let virtual_header = consensus_core::header::Header::from_precomputed_hash(virtual_hash, tips);
        self.protocol.calculate_ghostdag(&virtual_header)
    }

    pub fn calculate_virtual_hash(tips: &[Hash]) -> Hash {
        // Deterministic simple combiner: sort tips and add their last u64 parts into four u64 lanes
        let mut parts = [0u64; 4];
        let mut sorted = tips.to_vec();
        sorted.sort();
        for (i, tip) in sorted.iter().enumerate() {
            let bytes = tip.as_bytes();
            let last = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
            parts[i % 4] = parts[i % 4].wrapping_add(last);
        }
        Hash::from_le_u64(parts)
    }
}
