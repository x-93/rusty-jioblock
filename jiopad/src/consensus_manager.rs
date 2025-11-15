use crate::config::ConsensusConfig;
use crate::storage_manager::StorageManager;
use consensus::consensus::types::ConsensusConfig as CoreConsensusConfig;
use consensus::consensus::storage::ConsensusStorage;
use consensus::consensus::ghostdag::{GhostdagManager, GhostdagProtocol, stores::GhostdagStore};
use consensus::consensus::difficulty::DifficultyManager;
use consensus::consensus::validation::{BlockValidator, HeaderValidator, TransactionValidator, ContextualValidator};
use consensus::pipeline::{BlockProcessor, HeaderProcessor, BodyProcessor, VirtualProcessor, DepsManager};
use consensus::consensus::dag::{BlockRelations, ReachabilityStore, DagTopology};
use consensus_core::{Hash, ZERO_HASH};
use consensus_core::config::genesis as core_genesis;
use std::sync::Arc;

/// Consensus manager that coordinates all consensus components
pub struct ConsensusManager {
    config: ConsensusConfig,
    block_processor: Arc<BlockProcessor>,
    ghostdag_manager: Arc<GhostdagManager>,
    difficulty_manager: Arc<DifficultyManager>,
    storage: Arc<ConsensusStorage>,
    dag_topology: Arc<DagTopology>,
    virtual_processor: Arc<VirtualProcessor>,
}

impl ConsensusManager {
    /// Create a new consensus manager
    pub async fn new(config: &ConsensusConfig, storage: Arc<StorageManager>, network_config: &crate::config::NetworkConfig) -> Result<Self, String> {
        // Convert config to core consensus config
        let core_config = CoreConsensusConfig {
            ghostdag_k: config.ghostdag_k,
            max_block_parents: config.max_block_parents,
            target_time_per_block: config.target_time_per_block,
            difficulty_window_size: config.difficulty_window_size,
            max_block_size: config.max_block_size,
            coinbase_maturity: config.coinbase_maturity,
        };

    // Get consensus storage from the provided StorageManager (so bootstrap uses the persistent manager)
    let consensus_storage = storage.consensus_storage();

        // Bootstrap genesis block into storage if empty
        // If there are no blocks stored yet, construct the default genesis and persist it.
        if consensus_storage.block_store().block_count() == 0 {
            // Build default genesis from consensus core
            let genesis_block = core_genesis::default_genesis();
            let genesis_block: consensus_core::block::Block = (&genesis_block).into();
            // store as the first block and apply to UTXO set with daa score 0
            let _ = consensus_storage.apply_block(&genesis_block, genesis_block.header.daa_score);
        }

        // Initialize DAG components
        let block_relations = Arc::new(BlockRelations::new());
        let reachability_store = Arc::new(ReachabilityStore::new());
        let dag_topology = Arc::new(DagTopology::new(block_relations.clone(), reachability_store.clone()));

        // Initialize GHOSTDAG components
        let ghostdag_store = Arc::new(GhostdagStore::new());
        let ghostdag_protocol = Arc::new(GhostdagProtocol::new(
            core_config.ghostdag_k,
            dag_topology.clone(),
            block_relations.clone(),
            ghostdag_store.clone(),
        ));
        let ghostdag_manager = Arc::new(GhostdagManager::new(ghostdag_protocol.clone(), ghostdag_store.clone()));

        // Initialize genesis block
        let genesis_hash = if network_config.genesis_hash == "0000000000000000000000000000000000000000000000000000000000000000" {
            ZERO_HASH
        } else {
            Hash::try_from_slice(&hex::decode(&network_config.genesis_hash).unwrap_or(vec![0; 32])[..32]).unwrap_or(ZERO_HASH)
        };
        reachability_store.init_genesis(genesis_hash);
        ghostdag_manager.init_genesis(genesis_hash);

        // Initialize difficulty manager
        let difficulty_manager = Arc::new(DifficultyManager::new());

        // Initialize validators
        let transaction_validator = Arc::new(TransactionValidator::new());
        let header_validator = Arc::new(HeaderValidator::new());
        let block_validator = Arc::new(BlockValidator::new(header_validator.clone(), transaction_validator.clone()));
        let contextual_validator = Arc::new(ContextualValidator::new(block_validator.clone(), transaction_validator.clone()));

        // Initialize dependency manager
        let deps_manager = Arc::new(DepsManager::new());

        // Initialize processors
        let header_processor = Arc::new(HeaderProcessor::new(
            header_validator,
            ghostdag_manager.clone(),
            consensus_storage.block_store(),
            difficulty_manager.clone(),
            deps_manager.clone(),
        ));

        let body_processor = Arc::new(BodyProcessor::new(
            block_validator,
            contextual_validator,
            consensus_storage.block_store(),
            consensus_storage.utxo_set(),
        ));

        let virtual_processor = Arc::new(VirtualProcessor::new(
            ghostdag_manager.clone(),
            consensus_storage.block_store(),
        ));

        let block_processor = Arc::new(BlockProcessor::new(
            header_processor,
            body_processor,
            virtual_processor.clone(),
            ghostdag_manager.clone(),
            consensus_storage.clone(),
            deps_manager,
        ));

        Ok(Self {
            config: config.clone(),
            block_processor,
            ghostdag_manager,
            difficulty_manager,
            storage: consensus_storage,
            dag_topology,
            virtual_processor,
        })
    }

    /// Get block processor
    pub fn block_processor(&self) -> Arc<BlockProcessor> {
        self.block_processor.clone()
    }

    /// Get ghostdag manager
    pub fn ghostdag_manager(&self) -> Arc<GhostdagManager> {
        self.ghostdag_manager.clone()
    }

    /// Get difficulty manager
    pub fn difficulty_manager(&self) -> Arc<DifficultyManager> {
        self.difficulty_manager.clone()
    }

    /// Get storage
    pub fn storage(&self) -> Arc<ConsensusStorage> {
        self.storage.clone()
    }

    /// Get DAG topology
    pub fn dag_topology(&self) -> Arc<DagTopology> {
        self.dag_topology.clone()
    }

    /// Get current DAA score
    pub fn current_daa_score(&self) -> u64 {
        // In a real implementation, this would track the current DAA score
        // For now, return a placeholder
        0
    }

    /// Get virtual processor
    pub fn virtual_processor(&self) -> Arc<VirtualProcessor> {
        self.virtual_processor.clone()
    }
}
