# Benchmark Implementation Guide

This document explains the implementation details of the filebrowser-tui benchmark suite, including design decisions, usage patterns, and best practices.

## Architecture

### File Structure

```
benches/
├── mod.rs                    # Benchmark utilities and fixtures
├── filebrowser_bench.rs     # Main benchmark suite
├── README.md                # User documentation
├── IMPLEMENTATION.md        # This file
├── run_benches.sh           # Helper script
└── .critter.toml           # Criterion configuration
```

### Module Structure

#### `mod.rs` - Fixtures and Utilities

This module provides test fixtures and utility functions:

- **`FixtureBuilder`**: Creates controlled test environments
  - Temporary directory management
  - File generation with specific properties
  - Nested directory structures
  - Sized files for I/O testing
  - Searchable content generation

- **Mock Data Functions**:
  - `create_mock_file_entries()`: Generate FileEntry objects
  - `create_sortable_entries()`: Create entries with known sort order

Design Principles:
- **Isolation**: Each benchmark gets a fresh temporary directory
- **Realism**: Fixtures mimic real-world file structures
- **Flexibility**: Configurable sizes and properties
- **Cleanup**: Automatic cleanup via `tempfile::TempDir`

#### `filebrowser_bench.rs` - Benchmark Suite

Organized into logical groups by operation type:

```rust
fn bench_group_name(c: &mut Criterion) {
    let mut group = c.benchmark_group("group_name");

    // Setup
    for size in [100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("operation", size),
            &size,
            |b, &s| {
                // Benchmark code
            },
        );
    }

    group.finish();
}
```

## Benchmark Patterns

### Pattern 1: Simple Iteration

For operations with minimal setup:

```rust
group.bench_function("name", |b| {
    b.iter(|| {
        black_box(operation_to_measure())
    });
});
```

**Use Cases**: Pure computations, simple lookups

### Pattern 2: Input Parameterization

For testing scalability:

```rust
for count in [100, 1_000, 10_000] {
    group.bench_with_input(
        BenchmarkId::new("operation", count),
        &count,
        |b, &c| {
            b.iter(|| {
                black_box(operation_with_size(c))
            });
        },
    );
}
```

**Use Cases**: Sorting, filtering, listing operations

### Pattern 3: Batched Operations

For operations with expensive setup:

```rust
b.iter_batched(
    || expensive_setup(),  // Setup function
    |data| {               // Benchmark function
        benchmark_operation(data);
        black_box(data)
    },
    criterion::BatchSize::LargeInput,  // Batch size hint
);
```

**Batch Size Options**:
- `SmallInput`: For small, cheap inputs (< 100 KB)
- `LargeInput`: For larger inputs (> 100 KB)
- `PerIteration`: For very expensive setups

**Use Cases**: File I/O, complex data structures

### Pattern 4: Async Operations

For async code:

```rust
let rt = Runtime::new().unwrap();

b.to_async(&rt).iter(|| async {
    let result = async_operation().await;
    black_box(result)
});
```

**Important Notes**:
- Create runtime once per benchmark group
- Don't recreate runtime in each iteration
- Use `black_box` on async results

### Pattern 5: Throughput Measurement

For measuring data processing rates:

```rust
group.throughput(Throughput::Bytes(size_in_bytes));
group.bench_function("name", |b| {
    b.iter(|| {
        process_data_of_known_size();
    });
});
```

**Throughput Types**:
- `Bytes`: For byte-based throughput
- `Elements`: For item-based throughput

## Common Benchmarks

### File Listing

```rust
fn bench_file_listing(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_listing");

    for file_count in [100, 1_000, 10_000] {
        group.bench_with_input(
            BenchmarkId::new("read_directory", file_count),
            &file_count,
            |b, &count| {
                let fixture = FixtureBuilder::new().unwrap();
                let dir_path = fixture
                    .create_directory_with_files("bench", count, false)
                    .unwrap();
                let navigator = Navigator::new();

                b.iter(|| {
                    black_box(navigator.read_directory(black_box(&dir_path)))
                });
            },
        );
    }

    group.finish();
}
```

**Key Points**:
- Fixture created outside the benchmark loop
- Directory structure created once
- Only the read operation is measured
- Use `black_box` to prevent optimization

### Sorting

```rust
fn bench_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("sorting");

    let mut entries = create_mock_file_entries(10_000, false);

    group.bench_function("by_name", |b| {
        b.iter_batched(
            || entries.clone(),
            |mut data| {
                sort_files(&mut data, SortBy::Name, SortOrder::Ascending);
                black_box(data)
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}
```

**Key Points**:
- Clone original data for each iteration
- Use batched setup for expensive clones
- Measure only the sort operation
- Test different sort criteria independently

### Search

```rust
fn bench_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("search");
    let rt = Runtime::new().unwrap();

    group.bench_function("content_search", |b| {
        let fixture = FixtureBuilder::new().unwrap();
        let dir_path = fixture
            .create_searchable_files("search", 100, 100)
            .unwrap();

        b.to_async(&rt).iter(|| async {
            let files = collect_files_in_directory(&dir_path, true, None)
                .await
                .unwrap();
            let searcher = ContentSearcher::new();
            let handle = searcher.search_files(
                files.clone(),
                "pattern",
                dir_path.clone(),
            );
            let results = handle.await.unwrap().unwrap();
            black_box(results)
        });
    });

    group.finish();
}
```

**Key Points**:
- Use runtime for async operations
- File collection happens in benchmark
- Only search operation is measured
- Handle async results properly

## Best Practices

### 1. Use `black_box` Correctly

```rust
// Good: Prevents compiler optimization
b.iter(|| {
    let result = expensive_computation(input);
    black_box(result)
});

// Bad: Compiler might optimize away
b.iter(|| {
    expensive_computation(input)
});
```

### 2. Setup Outside Benchmark Loop

```rust
// Good: Setup done once
let fixture = FixtureBuilder::new().unwrap();
let data = create_test_data();
b.iter(|| {
    process(&data)
});

// Bad: Setup runs in every iteration
b.iter(|| {
    let fixture = FixtureBuilder::new().unwrap();
    let data = create_test_data();
    process(&data)
});
```

### 3. Appropriate Batch Sizes

```rust
// For small inputs (< 100 KB)
criterion::BatchSize::SmallInput

// For large inputs (> 100 KB)
criterion::BatchSize::LargeInput

// When setup is very expensive
criterion::BatchSize::PerIteration
```

### 4. Realistic Test Data

```rust
// Good: Mimics real-world scenarios
let entries = create_mock_file_entries(10_000, true);
// Includes: Various sizes, types, hidden files, directories

// Bad: Too artificial
let entries = vec![FileEntry::default(); 10_000];
// All identical, doesn't test real conditions
```

### 5. Measure Hot Paths

Focus on code that runs frequently:

- Directory listing (common operation)
- Sorting (happens on every directory change)
- Filtering (runs on search)
- Rendering (happens on every frame)

## Performance Target Guidelines

Based on typical user expectations:

| Operation | Scale | Target Time | Rationale |
|-----------|-------|-------------|-----------|
| File listing | 1,000 files | < 10ms | Instant feel |
| File listing | 10,000 files | < 100ms | Acceptable delay |
| Sorting | 10,000 items | < 10ms | Unnoticeable |
| Filtering | 10,000 items | < 1ms | Real-time response |
| Search (filename) | 10,000 files | < 1ms | Instant feedback |
| Search (content) | 100 files | < 100ms | Quick scan |
| Copy file | 10 MB | < 500ms | Reasonable wait |
| Rename preview | 10,000 items | < 100ms | Quick preview |

## Troubleshooting

### Issue: Inconsistent Results

**Symptoms**: Large variance between runs

**Solutions**:
1. Increase measurement time:
   ```toml
   [measurements]
   measurement_time = 10.0
   ```

2. Close background applications
3. Use consistent system state
4. Increase sample size:
   ```toml
   sample_size = 200
   ```

### Issue: "Too Few Iterations"

**Symptoms**: Error about insufficient iterations

**Solutions**:
1. Increase sample size
2. Reduce input size
3. Increase measurement time

### Issue: Outliers

**Symptoms**: Some iterations much slower

**Solutions**:
1. Check for background processes
2. Verify disk cache effects
3. Consider warm-up iterations
4. Use median instead of mean for reporting

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Benchmarks

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  bench:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run benchmarks
        run: cargo bench -- --save-baseline ci
      - name: Store baseline
        uses: actions/upload-artifact@v3
        with:
          name: criterion-baseline
          path: target/criterion
```

### Conditional Benchmarking

```bash
# Only run full benchmarks on main branch
if [[ $GITHUB_REF == "refs/heads/main" ]]; then
  cargo bench
else
  cargo bench -- quick
fi
```

## Extending the Suite

### Adding a New Benchmark Group

1. **Create the function**:
   ```rust
   fn bench_new_operation(c: &mut Criterion) {
       let mut group = c.benchmark_group("new_operation");

       // Add benchmarks

       group.finish();
   }
   ```

2. **Register with criterion_group**:
   ```rust
   criterion_group!(
       benches,
       bench_file_listing,
       bench_new_operation  // Add here
   );
   ```

3. **Document in README.md**:
   - Add to benchmark groups section
   - Specify performance goals
   - Add usage examples

### Adding Fixtures

When adding new fixtures to `mod.rs`:

1. **Make it realistic**: Mimic actual use cases
2. **Make it configurable**: Allow size/property variations
3. **Document behavior**: Explain what it creates
4. **Clean up properly**: Use RAII for resources

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Benchmarking Best Practices](https://github.com/rust-lang/rustc-dev-guide/blob/master/benchmarking/benchmarking.md)
