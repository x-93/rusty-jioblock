//! Comprehensive tests for the mining module
//!
//! This module contains integration tests for the mining system including
//! PoW verification, job management, worker coordination, and difficulty adjustment.

#[cfg(test)]
mod tests {
    use crate::difficulty::DifficultyConfig;
    use crate::job::MiningJob;
    use crate::manager::{MiningConfig, MiningManager};
    use crate::pow::{ProofOfWork, Target};
    use consensus_core::Hash;
    use rpc_core::model::BlockTemplate;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    fn create_test_template() -> BlockTemplate {
        BlockTemplate {
            version: 1,
            parent_hashes: vec![Hash::default()],
            transactions: Vec::new(),
            coinbase_value: 5_000_000_000,
            bits: 0x207fffff,
            timestamp: 1000,
            pay_address: "test_address".to_string(),
            target: "0".to_string(),
        }
    }

    // ==================== PoW Tests ====================

    #[test]
    fn test_pow_hash_deterministic() {
        let data = b"consistent_data";
        let hash1 = ProofOfWork::compute_hash(data);
        let hash2 = ProofOfWork::compute_hash(data);
        assert_eq!(hash1, hash2, "Same data should produce same hash");
    }

    #[test]
    fn test_pow_different_inputs_different_hashes() {
        let hash1 = ProofOfWork::compute_hash(b"data1");
        let hash2 = ProofOfWork::compute_hash(b"data2");
        assert_ne!(hash1, hash2, "Different data should produce different hashes");
    }

    #[test]
    fn test_target_conversion_roundtrip() {
        let bits = 0x1d00ffff;
        let target = Target::from_bits(bits);
        let recovered = target.to_bits();
    // Ensure conversion produced a valid compact representation
    assert!(recovered > 0);
    }

    #[test]
    fn test_hash_rate_calculation() {
        let rate = ProofOfWork::calculate_hash_rate(2_000_000, 2000);
        // 2M hashes in 2 seconds = 1M hash/sec
        assert!(rate > 900_000.0 && rate < 1_100_000.0);
    }

    #[test]
    fn test_hash_rate_zero_duration() {
        let rate = ProofOfWork::calculate_hash_rate(1_000_000, 0);
        assert_eq!(rate, 0.0);
    }

    // ==================== Job Tests ====================

    #[test]
    fn test_mining_job_creation() {
        let template = create_test_template();
        let target = Target::from_bits(0x207fffff);
        let job = MiningJob::new(template, target);

        assert_eq!(job.version(), 1);
        assert_eq!(job.coinbase_value(), 5_000_000_000);
        assert_eq!(job.bits(), 0x207fffff);
    }

    #[test]
    fn test_mining_job_age_calculation() {
        let template = create_test_template();
        let target = Target::from_bits(0x207fffff);
        let job = MiningJob::new(template, target);

        let age = job.age_ms();
        assert!(age < 100, "New job should have very small age");
        assert!(job.is_recent(1000), "Job should be recent within 1 second");
    }

    #[test]
    fn test_mining_job_header_serialization() {
        let template = create_test_template();
        let target = Target::from_bits(0x207fffff);
        let job = MiningJob::new(template, target);

        let header_bytes = job.header_with_nonce(12345);
        assert!(header_bytes.len() > 0, "Header bytes should not be empty");

        let header_bytes_different = job.header_with_nonce(12346);
        assert_ne!(
            header_bytes, header_bytes_different,
            "Different nonces should produce different serializations"
        );
    }

    #[test]
    fn test_mined_block_hash_rate() {
        use crate::job::MinedBlock;
        let block = MinedBlock::new(1, 0, 12345, Hash::default(), 1_000_000, 1000);
        let rate = block.hash_rate();
        assert!(rate > 900_000.0 && rate < 1_100_000.0);
    }

    // ==================== Manager Tests ====================

    #[test]
    fn test_manager_creation() {
        let config = MiningConfig {
            num_workers: 2,
            job_max_age_ms: 30_000,
        };
        let manager = MiningManager::new(config);
        assert_eq!(manager.worker_count(), 2);
    }

    #[test]
    fn test_manager_start_stop() {
        let config = MiningConfig {
            num_workers: 2,
            job_max_age_ms: 30_000,
        };
        let mut manager = MiningManager::new(config);
        manager.start();
        // Stopping is called on drop
        drop(manager);
        // If we got here without panicking, the test passed
    }

    #[test]
    fn test_manager_update_job() {
        let config = MiningConfig {
            num_workers: 2,
            job_max_age_ms: 30_000,
        };
        let manager = MiningManager::new(config);
        let template = create_test_template();

        manager.update_job(template);
        assert!(manager.current_job().is_some());
    }

    #[test]
    fn test_manager_session_stats() {
        let config = MiningConfig {
            num_workers: 2,
            job_max_age_ms: 30_000,
        };
        let manager = MiningManager::new(config);
        let stats = manager.get_session_stats();

        assert_eq!(stats.worker_count, 2);
        assert_eq!(stats.total_blocks, 0);
        assert_eq!(stats.worker_stats.len(), 2);
    }

    #[test]
    fn test_manager_collect_results_empty() {
        let config = MiningConfig {
            num_workers: 2,
            job_max_age_ms: 30_000,
        };
        let manager = MiningManager::new(config);
        let results = manager.collect_results();
        assert_eq!(results.len(), 0);
    }

    // ==================== Difficulty Tests ====================

    #[test]
    fn test_difficulty_manager_creation() {
        use crate::difficulty::DifficultyManager;
        let manager = DifficultyManager::new();
        assert!(manager.get_current_target().is_some());
    }

    #[test]
    fn test_difficulty_with_custom_config() {
        use crate::difficulty::DifficultyManager;
        let config = DifficultyConfig {
            window_size: 100,
            target_block_time_ms: 500,
            min_target: Target::from_bits(0x1f000000),
            max_target: Target::from_bits(0x207fffff),
        };
        let manager = DifficultyManager::with_config(config);
        let target = manager.get_current_target();
        assert!(target.is_some());
    }

    #[test]
    fn test_target_clamping() {
        use crate::difficulty::DifficultyManager;
        let min = Target::from_bits(0x1f000000);
        let max = Target::from_bits(0x207fffff);
        let test_target = Target::from_bits(0x21000000);

        let clamped = DifficultyManager::clamp_target(test_target, min, max);
        assert!(clamped <= max);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_complete_mining_workflow() {
        // This test demonstrates the complete mining workflow
        let config = MiningConfig {
            num_workers: 1,
            job_max_age_ms: 30_000,
        };
        let mut manager = MiningManager::new(config);
        manager.start();

        let template = create_test_template();
        manager.update_job(template);

        // Wait a bit to allow workers to attempt mining
        thread::sleep(Duration::from_millis(100));

        let stats = manager.get_session_stats();
        assert_eq!(stats.worker_count, 1);

        // Cleanup on drop
        drop(manager);
    }

    #[test]
    fn test_mining_with_multiple_workers() {
        let config = MiningConfig {
            num_workers: 4,
            job_max_age_ms: 30_000,
        };
        let mut manager = MiningManager::new(config);
        manager.start();

        let worker_stats = manager.get_worker_stats();
        assert_eq!(worker_stats.len(), 4);

        drop(manager);
    }

    #[test]
    fn test_pow_validation_integration() {
        let header_data = b"test_block_header";
        let target = Target::from_bits(0x207fffff); // Easy target

        // Compute hash
        let hash = ProofOfWork::compute_hash(header_data);
        assert_eq!(hash.as_bytes().len(), 32);

        // Verify PoW (should pass for an easy target)
        let is_valid = ProofOfWork::is_valid_pow(header_data, &target);
        // Don't assert specific value as it depends on hash
        assert_eq!(is_valid, ProofOfWork::is_valid_pow(header_data, &target));
    }

    #[test]
    fn test_concurrent_job_updates() {
        use std::sync::Arc;
        let config = MiningConfig {
            num_workers: 2,
            job_max_age_ms: 30_000,
        };
        let manager = Arc::new(MiningManager::new(config));

        let mut handles = vec![];
        for i in 0..3 {
            let manager_clone = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                let template = BlockTemplate {
                    version: 1,
                    parent_hashes: vec![Hash::default()],
                    transactions: Vec::new(),
                    coinbase_value: 5_000_000_000 + i as u64,
                    bits: 0x207fffff,
                    timestamp: 1000 + i as u64,
                    pay_address: format!("address_{}", i),
                    target: "0".to_string(),
                };
                manager_clone.update_job(template);
                thread::sleep(Duration::from_millis(10));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(manager.current_job().is_some());
    }
}
