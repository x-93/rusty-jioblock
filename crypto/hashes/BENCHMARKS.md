# JIO Network Hashing Benchmarks

Comprehensive benchmarking suite for the JIO network PoW algorithms using Criterion.

## Running Benchmarks

### Run all benchmarks
```bash
cd crypto/hashes
cargo bench --bench bench
```

### Run specific benchmark group
```bash
# Compare all algorithms
cargo bench --bench bench algorithm_comparison

# PowB3Hash only
cargo bench --bench bench powb3hash

# PowHash only
cargo bench --bench bench powhash

# KHeavyHash only
cargo bench --bench bench heavyhash

# Throughput tests
cargo bench --bench bench throughput

# Different input tests
cargo bench --bench bench different_inputs
```

### Generate HTML reports
```bash
cargo bench --bench bench -- --verbose
```

The HTML reports will be in: `target/criterion/`

## Benchmark Categories

### 1. Individual Algorithm Benchmarks

#### PowB3Hash (BLAKE3-based)
- `PowB3Hash::new` - Time to initialize hasher
- `PowB3Hash::finalize` - Time to finalize with nonce
- `PowB3Hash::full_flow` - Complete hash computation

#### PowHash (Keccak-based cSHAKE256)
- `PowHash::new` - Time to initialize hasher
- `PowHash::finalize_with_nonce` - Time to finalize with nonce
- `PowHash::full_flow` - Complete hash computation

#### KHeavyHash (Keccak-based cSHAKE256)
- `KHeavyHash::hash` - Single hash computation

### 2. Comparative Benchmarks

`algorithm_comparison` - Side-by-side performance comparison:
- PowB3Hash full flow
- PowHash full flow
- KHeavyHash single hash

### 3. Throughput Benchmarks

`throughput_*` - Measures hashes per second:
- `PowB3Hash_throughput_1000` - 1000 sequential hashes
- `PowHash_throughput_1000` - 1000 sequential hashes
- `KHeavyHash_throughput_1000` - 1000 sequential hashes

### 4. Input Variation Benchmarks

`different_inputs` - Tests with different seed values (0, 42, 128, 255):
- Shows consistency across different inputs
- Helps identify input-dependent performance variations

## Benchmark Configuration

```rust
Criterion::default()
    .sample_size(100)              // 100 samples per benchmark
    .warm_up_time(Duration::from_secs(1))  // 1 second warm-up
```

## Expected Performance

### Typical Results (Reference)

Run on modern hardware, results will vary:

```
PowB3Hash::new                   ~5-10 µs
PowB3Hash::finalize              ~50-100 µs
PowB3Hash::full_flow            ~55-110 µs

PowHash::new                     ~1-2 µs
PowHash::finalize_with_nonce    ~10-20 µs
PowHash::full_flow              ~11-22 µs

KHeavyHash::hash                ~10-20 µs

Algorithm Comparison (relative speeds):
  PowHash     1.0x (baseline)
  KHeavyHash  1.2x
  PowB3Hash   3-5x (more computationally intensive)
```

## Output Format

Criterion generates detailed reports:

```
PowHash::full_flow              time:   [11.523 ms 11.634 ms 11.759 ms]
                        change: [-2.4321% -1.2345% +0.1234%]
                        Performance has improved.

Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
```

## Interpreting Results

1. **Time**: The mean time per iteration
   - Lower is better
   - Shown with confidence interval (first and last values)

2. **Change**: Relative performance change from last run
   - `[-X% +Y%]` represents the 95% confidence interval
   - Green = improvement, Red = regression

3. **Outliers**: Statistical outliers in measurements
   - Some variation is normal
   - High percentage of outliers may indicate system noise

## Performance Tips

### For Accurate Benchmarking

1. **Close other applications** - Reduces system noise
2. **Run multiple times** - Criterion takes multiple samples
3. **Use same hardware** - Compare results on consistent systems
4. **Run with release mode** - Cargo automatically uses release mode for benchmarks

### Optimization Strategies

If you want to optimize further:

1. **Profile the hot path**:
   ```bash
   cargo bench --bench bench -- --profile-time=10
   ```

2. **Check assembly**:
   ```bash
   cargo asm crypto_hashes::pow_hashers::PowHash::new
   ```

3. **Monitor cache behavior**:
   - Watch for memory allocation patterns
   - Keccak works on 25 u64 elements (200 bytes)
   - Should fit in L1 cache on most modern CPUs

## Customizing Benchmarks

To add a new benchmark:

```rust
fn bench_my_function(c: &mut Criterion) {
    c.bench_function("my_benchmark", |b| {
        let input = black_box(create_test_hash(42));
        b.iter(|| {
            my_function(black_box(input))
        });
    });
}

// Add to criterion_group! macro
```

## Key Concepts

### black_box()
- Prevents compiler optimizations from skewing results
- Ensures the benchmark measures real work
- Always wrap inputs with `black_box()`

### iter_batched()
- For setup/teardown scenarios
- Useful when you need to initialize state

### BatchSize
- `SmallInput` - Default, includes setup in timing
- `LargeInput` - Amortizes setup cost
- `NumericBatches(N)` - Custom batch size

## Continuous Integration

Add to your CI pipeline:

```yaml
# Example GitHub Actions
- name: Run benchmarks
  run: cargo bench --bench bench -- --output-format bencher
  
- name: Store benchmark result
  uses: benchmark-action/github-action@v1
  with:
    tool: 'cargo'
    output-file-path: target/criterion/output.txt
```

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [BLAKE3 Performance](https://blake3.io/)
- [Keccak Performance](https://keccak.team/)

## Troubleshooting

### Benchmarks too slow
- Reduce `sample_size` in Criterion config
- Use smaller sample sizes: `c.bench_function().sample_size(10)`

### Inconsistent results
- Close background applications
- Disable CPU frequency scaling if possible
- Run on bare metal instead of VM

### Compilation errors
- Ensure `criterion` is in `[dev-dependencies]`
- Check `[[bench]]` section in `Cargo.toml` has `harness = false`

## Performance Monitoring

Track performance over time:

```bash
# Run benchmarks and save baseline
cargo bench --bench bench -- --save-baseline main

# Run again and compare
cargo bench --bench bench -- --baseline main
```

Criterion stores baseline data in `target/criterion/`
