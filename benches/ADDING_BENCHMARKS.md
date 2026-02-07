# Guide: Adding New Benchmarks

This guide provides step-by-step instructions for adding new benchmarks to the filebrowser-tui suite.

## Quick Reference

When adding benchmarks for a new feature, follow these steps:

1. Create the benchmark function in `filebrowser_bench.rs`
2. Add it to `criterion_group!` macro
3. Document in `README.md`
4. Update performance goals
5. Run and verify

## Step-by-Step Example

Let's add a benchmark for a hypothetical "file compression" operation.

### Step 1: Define the Benchmark Function

Add to `benches/filebrowser_bench.rs`:

```rust
/// Benchmark file compression operations
fn bench_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression");

    // Test different file sizes
    for size in [1024, 10_240, 102_400].iter() {
        group.bench_with_input(
            BenchmarkId::new("compress_file", size),
            size,
            |b, &bytes| {
                let fixture = FixtureBuilder::new().unwrap();
                let file_path = fixture.create_test_file("test.txt", bytes).unwrap();

                b.iter(|| {
                    black_box(compress_file(black_box(&file_path)))
                });
            },
        );
    }

    // Test different compression levels
    for level in [1, 5, 9].iter() {
        group.bench_with_input(
            BenchmarkId::new("compress_level", level),
            level,
            |b, &lvl| {
                let fixture = FixtureBuilder::new().unwrap();
                let file_path = fixture
                    .create_test_file("test.txt", 10_240)
                    .unwrap();

                b.iter(|| {
                    black_box(compress_with_level(black_box(&file_path), lvl))
                });
            },
        );
    }

    group.finish();
}
```

### Step 2: Register with Criterion

Find the `criterion_group!` macro at the bottom of `filebrowser_bench.rs` and add your function:

```rust
criterion_group!(
    benches,
    bench_file_listing,
    bench_sorting,
    bench_search,
    // ... other benchmarks ...
    bench_compression,  // Add your new benchmark here
);

criterion_main!(benches);
```

### Step 3: Add Fixture (if needed)

If you need new test fixtures, add them to `benches/mod.rs`:

```rust
impl FixtureBuilder {
    /// Create a file with specific size for compression testing
    pub fn create_test_file(
        &self,
        file_name: &str,
        size: usize,
    ) -> anyhow::Result<PathBuf> {
        let file_path = self.base_path.join(file_name);
        let mut file = File::create(&file_path)?;

        // Create repetitive data for better compression
        let chunk = "test data for compression. ".as_bytes();
        let mut written = 0;
        while written < size {
            let to_write = chunk.len().min(size - written);
            file.write_all(&chunk[..to_write])?;
            written += to_write;
        }

        Ok(file_path)
    }
}
```

### Step 4: Document in README.md

Add to the "Benchmark Groups" section in `benches/README.md`:

```markdown
### 11. Compression (`compression`)

Tests file compression performance:

- `compress_file/{size}` - Compress files of various sizes
- `compress_level/{level}` - Compress with different compression levels

**Performance Goals**:
- Small file (1KB): < 1ms
- Medium file (10KB): < 5ms
- Large file (100KB): < 50ms
```

### Step 5: Run and Verify

```bash
# Run your new benchmark
cargo bench --bench filebrowser_bench -- compression

# Or use the helper script
./benches/run_benches.sh compression
```

## Common Patterns

### Pattern: Testing Multiple Configurations

```rust
fn bench_with_configs(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_test");

    for config in [
        Config::Fast,
        Config::Balanced,
        Config::Thorough,
    ] {
        group.bench_with_input(
            BenchmarkId::new("operation", config.name()),
            &config,
            |b, cfg| {
                b.iter(|| {
                    black_box(perform_operation(black_box(*cfg)))
                });
            },
        );
    }

    group.finish();
}
```

### Pattern: Comparing Implementations

```rust
fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("implementation_comparison");

    let data = create_test_data();

    group.bench_function("old_implementation", |b| {
        b.iter(|| {
            black_box(old_implementation(black_box(&data)))
        });
    });

    group.bench_function("new_implementation", |b| {
        b.iter(|| {
            black_box(new_implementation(black_box(&data)))
        });
    });

    group.finish();
}
```

### Pattern: Async Operations

```rust
fn bench_async_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_ops");
    let rt = Runtime::new().unwrap();

    group.bench_function("async_read", |b| {
        let fixture = FixtureBuilder::new().unwrap();
        let file_path = fixture.create_test_file("test.txt", 1024).unwrap();

        b.to_async(&rt).iter(|| async {
            let content = read_file_async(black_box(&file_path)).await.unwrap();
            black_box(content)
        });
    });

    group.finish();
}
```

### Pattern: Memory Allocations

```rust
fn bench_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocations");

    group.bench_function("vec_creation", |b| {
        b.iter(|| {
            let vec: Vec<u8> = (0..1000).map(|i| i as u8).collect();
            black_box(vec)
        });
    });

    group.bench_function("vec_reuse", |b| {
        let mut vec = Vec::with_capacity(1000);
        b.iter(|| {
            vec.clear();
            vec.extend(0..1000);
            black_box(&mut vec)
        });
    });

    group.finish();
}
```

## Checklist

Before submitting a new benchmark:

- [ ] Function follows naming convention: `bench_<operation_name>`
- [ ] Uses `black_box` to prevent optimizations
- [ ] Setup is done outside the benchmark loop
- [ ] Fixtures properly clean up resources
- [ ] Added to `criterion_group!` macro
- [ ] Documented in `README.md`
- [ ] Performance goals specified
- [ ] Benchmark runs successfully
- [ ] Results are reasonable (verify with cargo bench)

## Naming Conventions

### Benchmark Groups
- Use lowercase with underscores: `file_listing`, `bulk_rename`
- Match the operation being tested
- Keep names descriptive but concise

### Benchmark IDs
```rust
BenchmarkId::new("operation_name", parameter)
// Examples:
BenchmarkId::new("read_directory", 1000)
BenchmarkId::new("compress_file", "level_9")
BenchmarkId::new("sort_by", "name")
```

### Function Names
```rust
fn bench_<operation>(c: &mut Criterion) {
    // Examples:
    fn bench_file_listing(c: &mut Criterion)
    fn bench_sorting(c: &mut Criterion)
    fn bench_compression(c: &mut Criterion)
}
```

## Performance Goals Template

When documenting performance goals in README.md:

```markdown
### X. Operation Name (`group_name`)

Tests description of what operations:

- `benchmark_id` - Description of what this tests
- `benchmark_id2` - Description of what this tests

**Performance Goals**:
- Small operation: < Xms
- Medium operation: < Yms
- Large operation: < Zms
```

## Testing Your Benchmarks

### Local Testing

```bash
# Quick test (fewer samples)
cargo bench --bench filebrowser_bench -- your_group -- --sample-size 10

# With output
cargo bench --bench filebrowser_bench -- your_group -- --output-format quiet

# Save baseline
cargo bench --bench filebrowser_bench -- your_group -- --save-baseline test
```

### Verification

After adding a benchmark, verify:

1. **It runs**: `cargo bench` completes without errors
2. **Results are stable**: Run multiple times, variance is low
3. **Times are reasonable**: Compare to similar operations
4. **No regressions**: Compare to baseline after changes

## Common Mistakes

### Mistake 1: Setup Inside Loop

```rust
// BAD: Setup runs every iteration
b.iter(|| {
    let data = create_expensive_data();  // Don't do this!
    process(&data)
})

// GOOD: Setup runs once
let data = create_expensive_data();
b.iter(|| {
    process(black_box(&data))
})
```

### Mistake 2: No Black Box

```rust
// BAD: Compiler might optimize away
b.iter(|| {
    let result = compute();
    result  // Unused result
})

// GOOD: Result is preserved
b.iter(|| {
    black_box(compute())
})
```

### Mistake 3: Wrong Batch Size

```rust
// BAD: Using SmallInput for large data
b.iter_batched(
    || create_large_data(),  // 10MB
    |data| process(data),
    criterion::BatchSize::SmallInput,  // Wrong!
)

// GOOD: Using LargeInput for large data
b.iter_batched(
    || create_large_data(),  // 10MB
    |data| process(data),
    criterion::BatchSize::LargeInput,  // Correct!
)
```

## Example: Complete Addition

Here's a complete example of adding a "file hashing" benchmark:

```rust
// In filebrowser_bench.rs

/// Benchmark file hashing operations
fn bench_file_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_hashing");

    // Test with different file sizes
    for size_in_kb in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("sha256_hash", size_in_kb),
            size_in_kb,
            |b, &kb| {
                let fixture = FixtureBuilder::new().unwrap();
                let file_path = fixture
                    .create_test_file("hash_test.txt", kb * 1024)
                    .unwrap();

                b.iter(|| {
                    black_box(hash_file_sha256(black_box(&file_path)))
                });
            },
        );
    }

    // Compare different hash algorithms
    let fixture = FixtureBuilder::new().unwrap();
    let file_path = fixture
        .create_test_file("compare.txt", 10 * 1024)
        .unwrap();

    group.bench_function("sha256", |b| {
        b.iter(|| black_box(hash_file_sha256(black_box(&file_path))));
    });

    group.bench_function("md5", |b| {
        b.iter(|| black_box(hash_file_md5(black_box(&file_path))));
    });

    group.finish();
}

// In criterion_group! macro
criterion_group!(
    benches,
    // ... existing benchmarks ...
    bench_file_hashing,  // Add here
);
```

## Resources

- [Criterion.rs Book](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- Internal: `benches/IMPLEMENTATION.md` for detailed patterns

## Support

If you encounter issues while adding benchmarks:

1. Check existing benchmarks for similar patterns
2. Review `benches/IMPLEMENTATION.md` for detailed examples
3. Run `cargo bench -- --help` for Criterion options
4. Check Criterion documentation for advanced features
