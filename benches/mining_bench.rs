//! Mining performance benchmarks
//!
//! This module contains criterion-based benchmarks for the mining system,
//! measuring hashing throughput, mining time, and multi-threaded performance.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use mining::prelude::*;
use rpc_core::model::BlockTemplate;
use consensus_core::Hash;

fn create_template() -> BlockTemplate {
    BlockTemplate {
        version: 1,
        parent_hashes: vec![Hash::default()],
        transactions: Vec::new(),
        coinbase_value: 5_000_000_000,
        bits: 0x207fffff,
        timestamp: 1000,
        pay_address: "bench_address".to_string(),
        target: "0".to_string(),
    }
}

fn bench_pow_hash_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("pow_hash_computation");
    group.throughput(Throughput::Bytes(32));

    group.bench_function("blake3_hash_32bytes", |b| {
        let data = black_box(b"test_header_data_for_hashing");
        b.iter(|| ProofOfWork::compute_hash(data))
    });

    group.bench_function("blake3_hash_large_512bytes", |b| {
        let data = black_box(&[0u8; 512]);
        b.iter(|| ProofOfWork::compute_hash(data))
    });

    group.finish();
}

fn bench_pow_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("pow_validation");

    group.bench_function("is_valid_pow_easy_target", |b| {
        let header = black_box(b"block_header_data");
        let target = black_box(Target::from_bits(0x207fffff)); // Easy target
        b.iter(|| ProofOfWork::is_valid_pow(header, &target))
    });

    group.bench_function("is_valid_pow_hard_target", |b| {
        let header = black_box(b"block_header_data");
        let target = black_box(Target::from_bits(0x1d00ffff)); // Harder target
        b.iter(|| ProofOfWork::is_valid_pow(header, &target))
    });

    group.finish();
}

fn bench_mining_job_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("mining_job");

    group.bench_function("create_job", |b| {
        let template = black_box(create_template());
        let target = black_box(Target::from_bits(0x207fffff));
        b.iter(|| MiningJob::new(template.clone(), target))
    });

    group.bench_function("header_serialization_with_nonce", |b| {
        let job = black_box(MiningJob::new(create_template(), Target::from_bits(0x207fffff)));
        b.iter(|| job.header_with_nonce(black_box(12345)))
    });

    group.finish();
}

fn bench_target_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("target_operations");

    group.bench_function("target_from_bits", |b| {
        let bits = black_box(0x1d00ffff);
        b.iter(|| Target::from_bits(bits))
    });

    group.bench_function("target_to_bits", |b| {
        let target = black_box(Target::from_bits(0x1d00ffff));
        b.iter(|| target.to_bits())
    });

    group.finish();
}

fn bench_difficulty_adjustment(c: &mut Criterion) {
    let mut group = c.benchmark_group("difficulty_adjustment");

    group.bench_function("calculate_next_target", |b| {
        let block_times: Vec<u64> = (0..504)
            .map(|i| i as u64 * 1000)
            .collect();
        let block_times = black_box(block_times);

        b.iter(|| {
            DifficultyManager::calculate_next_target(&block_times, 1000)
        })
    });

    group.finish();
}

fn bench_mining_manager_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mining_manager");

    group.bench_function("manager_creation_2_workers", |b| {
        b.iter(|| {
            let config = black_box(MiningConfig {
                num_workers: 2,
                job_max_age_ms: 30_000,
            });
            MiningManager::new(config)
        })
    });

    group.bench_function("update_job", |b| {
        let manager = MiningManager::new(MiningConfig {
            num_workers: 2,
            job_max_age_ms: 30_000,
        });
        let template = black_box(create_template());
        b.iter(|| manager.update_job(template.clone()))
    });

    group.finish();
}

fn bench_mining_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("mining_scaling");
    group.sample_size(10); // Reduce sample size due to long-running operations

    for num_workers in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_workers),
            num_workers,
            |b, &num_workers| {
                let config = MiningConfig {
                    num_workers,
                    job_max_age_ms: 30_000,
                };
                let manager = black_box(MiningManager::new(config));

                b.iter(|| {
                    let stats = manager.get_session_stats();
                    // Simulate some work
                    black_box(stats)
                })
            },
        );
    }

    group.finish();
}

fn bench_hash_rate_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_rate_calculation");

    group.bench_function("calculate_hash_rate", |b| {
        let hashes = black_box(1_000_000_000);
        let duration_ms = black_box(1000);
        b.iter(|| ProofOfWork::calculate_hash_rate(hashes, duration_ms))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_pow_hash_computation,
    bench_pow_validation,
    bench_mining_job_creation,
    bench_target_operations,
    bench_difficulty_adjustment,
    bench_mining_manager_operations,
    bench_mining_scaling,
    bench_hash_rate_calculation,
);

criterion_main!(benches);
