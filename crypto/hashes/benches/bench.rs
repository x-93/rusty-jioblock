// Benchmarks for JIO Network Hashing Algorithms
// Run with: cargo bench --bench bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use crypto_hashes::{Hash, pow_hashers::{PowB3Hash, PowHash, KHeavyHash}};

// ============================================================================
// Test Data Setup
// ============================================================================

fn create_test_hash(value: u8) -> Hash {
    Hash::from_bytes([value; 32])
}

fn create_test_timestamp() -> u64 {
    1234567890
}

fn create_test_nonce() -> u64 {
    9876543210
}

// ============================================================================
// PowB3Hash Benchmarks
// ============================================================================

fn bench_powb3hash_new(c: &mut Criterion) {
    c.bench_function("PowB3Hash::new", |b| {
        let pre_pow_hash = black_box(create_test_hash(42));
        let timestamp = black_box(create_test_timestamp());
        
        b.iter(|| {
            PowB3Hash::new(black_box(pre_pow_hash), black_box(timestamp))
        });
    });
}

fn bench_powb3hash_finalize(c: &mut Criterion) {
    c.bench_function("PowB3Hash::finalize_with_nonce", |b| {
        let pre_pow_hash = black_box(create_test_hash(42));
        let timestamp = black_box(create_test_timestamp());
        let nonce = black_box(create_test_nonce());
        
        b.iter_batched(
            || PowB3Hash::new(black_box(pre_pow_hash), black_box(timestamp)),
            |hasher| hasher.finalize_with_nonce(black_box(nonce)),
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_powb3hash_full(c: &mut Criterion) {
    c.bench_function("PowB3Hash::full_flow", |b| {
        let pre_pow_hash = black_box(create_test_hash(42));
        let timestamp = black_box(create_test_timestamp());
        let nonce = black_box(create_test_nonce());
        
        b.iter(|| {
            let hasher = PowB3Hash::new(black_box(pre_pow_hash), black_box(timestamp));
            hasher.finalize_with_nonce(black_box(nonce))
        });
    });
}

// ============================================================================
// PowHash Benchmarks (cSHAKE256-based Keccak)
// ============================================================================

fn bench_powhash_new(c: &mut Criterion) {
    c.bench_function("PowHash::new", |b| {
        let pre_pow_hash = black_box(create_test_hash(42));
        let timestamp = black_box(create_test_timestamp());
        
        b.iter(|| {
            PowHash::new(black_box(pre_pow_hash), black_box(timestamp))
        });
    });
}

fn bench_powhash_finalize_with_nonce(c: &mut Criterion) {
    c.bench_function("PowHash::finalize_with_nonce", |b| {
        let pre_pow_hash = black_box(create_test_hash(42));
        let timestamp = black_box(create_test_timestamp());
        let nonce = black_box(create_test_nonce());
        
        b.iter_batched(
            || PowHash::new(black_box(pre_pow_hash), black_box(timestamp)),
            |hasher| hasher.finalize_with_nonce(black_box(nonce)),
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_powhash_full(c: &mut Criterion) {
    c.bench_function("PowHash::full_flow", |b| {
        let pre_pow_hash = black_box(create_test_hash(42));
        let timestamp = black_box(create_test_timestamp());
        let nonce = black_box(create_test_nonce());
        
        b.iter(|| {
            let hasher = PowHash::new(black_box(pre_pow_hash), black_box(timestamp));
            hasher.finalize_with_nonce(black_box(nonce))
        });
    });
}

// ============================================================================
// KHeavyHash Benchmarks (cSHAKE256-based Keccak)
// ============================================================================

fn bench_heavyhash_hash(c: &mut Criterion) {
    c.bench_function("KHeavyHash::hash", |b| {
        let input = black_box(create_test_hash(42));
        b.iter(|| KHeavyHash::hash(black_box(input)));
    });
}

// ============================================================================
// Comparative Benchmarks
// ============================================================================

fn bench_algorithm_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("algorithm_comparison");
    
    let pre_pow_hash = black_box(create_test_hash(42));
    let timestamp = black_box(create_test_timestamp());
    let nonce = black_box(create_test_nonce());
    let input = black_box(create_test_hash(42));
    
    group.bench_with_input(BenchmarkId::new("PowB3Hash", "full"), &0, |b, _| {
        b.iter(|| {
            let hasher = PowB3Hash::new(black_box(pre_pow_hash), black_box(timestamp));
            hasher.finalize_with_nonce(black_box(nonce))
        });
    });
    
    group.bench_with_input(BenchmarkId::new("PowHash", "full"), &0, |b, _| {
        b.iter(|| {
            let hasher = PowHash::new(black_box(pre_pow_hash), black_box(timestamp));
            hasher.finalize_with_nonce(black_box(nonce))
        });
    });
    
    group.bench_with_input(BenchmarkId::new("KHeavyHash", "single"), &0, |b, _| {
        b.iter(|| KHeavyHash::hash(black_box(input)));
    });
    
    group.finish();
}

// ============================================================================
// Throughput Benchmarks
// ============================================================================

fn bench_throughput_powb3hash(c: &mut Criterion) {
    c.bench_function("PowB3Hash_throughput_1000", |b| {
        let pre_pow_hash = black_box(create_test_hash(42));
        let timestamp = black_box(create_test_timestamp());
        
        b.iter(|| {
            for i in 0..1000 {
                let hasher = PowB3Hash::new(black_box(pre_pow_hash), black_box(timestamp));
                let _ = hasher.finalize_with_nonce(black_box(i as u64));
            }
        });
    });
}

fn bench_throughput_powhash(c: &mut Criterion) {
    c.bench_function("PowHash_throughput_1000", |b| {
        let pre_pow_hash = black_box(create_test_hash(42));
        let timestamp = black_box(create_test_timestamp());
        
        b.iter(|| {
            for i in 0..1000 {
                let hasher = PowHash::new(black_box(pre_pow_hash), black_box(timestamp));
                let _ = hasher.finalize_with_nonce(black_box(i as u64));
            }
        });
    });
}

fn bench_throughput_heavyhash(c: &mut Criterion) {
    c.bench_function("KHeavyHash_throughput_1000", |b| {
        let input = black_box(create_test_hash(42));
        
        b.iter(|| {
            for _ in 0..1000 {
                let _ = KHeavyHash::hash(black_box(input));
            }
        });
    });
}

// ============================================================================
// Latency Benchmarks (Multiple Input Sizes)
// ============================================================================

fn bench_different_inputs(c: &mut Criterion) {
    let mut group = c.benchmark_group("different_inputs");
    
    for seed_value in [0u8, 42, 128, 255].iter() {
        let pre_pow_hash = black_box(create_test_hash(*seed_value));
        let timestamp = black_box(create_test_timestamp());
        let nonce = black_box(create_test_nonce());
        
        group.bench_with_input(
            BenchmarkId::new("PowB3Hash", format!("seed_{}", seed_value)),
            &0,
            |b, _| {
                b.iter(|| {
                    let hasher = PowB3Hash::new(black_box(pre_pow_hash), black_box(timestamp));
                    hasher.finalize_with_nonce(black_box(nonce))
                });
            },
        );
    }
    
    for seed_value in [0u8, 42, 128, 255].iter() {
        let pre_pow_hash = black_box(create_test_hash(*seed_value));
        let timestamp = black_box(create_test_timestamp());
        let nonce = black_box(create_test_nonce());
        
        group.bench_with_input(
            BenchmarkId::new("PowHash", format!("seed_{}", seed_value)),
            &0,
            |b, _| {
                b.iter(|| {
                    let hasher = PowHash::new(black_box(pre_pow_hash), black_box(timestamp));
                    hasher.finalize_with_nonce(black_box(nonce))
                });
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// Criterion Configuration and Main
// ============================================================================

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .warm_up_time(std::time::Duration::from_secs(1));
    targets = 
        bench_powb3hash_new,
        bench_powb3hash_finalize,
        bench_powb3hash_full,
        bench_powhash_new,
        bench_powhash_finalize_with_nonce,
        bench_powhash_full,
        bench_heavyhash_hash,
        bench_algorithm_comparison,
        bench_throughput_powb3hash,
        bench_throughput_powhash,
        bench_throughput_heavyhash,
        bench_different_inputs
);

criterion_main!(benches);
