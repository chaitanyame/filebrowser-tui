# Performance Benchmarks

Comprehensive benchmark suite for filebrowser-tui performance-critical operations.

## Overview

This benchmark suite measures performance across multiple dimensions of the file browser:

- **File Listing**: Directory reading operations at various scales
- **Sorting**: Performance of different sort criteria and algorithms
- **Search**: Filename search and content search operations
- **UI Rendering**: Simulated rendering performance for different list sizes
- **File Operations**: Copy, traversal, and other file system operations
- **Bulk Rename**: Pattern-based rename preview generation and execution
- **Filtering**: File filtering with various criteria
- **Memory**: Memory allocation patterns and optimization opportunities

## Running Benchmarks

### Basic Usage

Run all benchmarks:
```bash
cargo bench
```

Run a specific benchmark group:
```bash
cargo bench --bench filebrowser_bench -- <group_name>
```

For example:
```bash
cargo bench --bench filebrowser_bench -- file_listing
cargo bench --bench filebrowser_bench -- sorting
cargo bench --bench filebrowser_bench -- search
```

### Advanced Options

Save benchmark results:
```bash
cargo bench -- --save-baseline main
```

Compare against a baseline:
```bash
cargo bench -- --baseline main
```

Generate plots (requires gnuplot):
```bash
cargo bench -- --plotting-backend gnuplot
```

Filter specific benchmarks:
```bash
cargo bench --bench filebrowser_bench -- 'file_listing/read_directory'
```

Output format options:
```bash
cargo bench -- --output-format bencher  # Alternative output formats
```

## Benchmark Groups

### 1. File Listing (`file_listing`)

Tests directory reading performance with varying file counts:

- `read_directory/100` - Directory with 100 files
- `read_directory/1000` - Directory with 1,000 files
- `read_directory/10000` - Directory with 10,000 files
- `read_directory_with_hidden_1000` - 1,000 files with hidden files

**Performance Goals**:
- 100 files: < 1ms
- 1,000 files: < 10ms
- 10,000 files: < 100ms

### 2. Sorting (`sorting`)

Tests file sorting performance with different criteria:

- `by_name/{count}` - Sort by filename
- `by_size/{count}` - Sort by file size
- `by_modified/{count}` - Sort by modification time
- `by_type/{count}` - Sort by file extension
- `by_name_ascending` vs `by_name_descending` - Order comparison

**Performance Goals**:
- 100 files: < 0.1ms
- 1,000 files: < 1ms
- 10,000 files: < 10ms

### 3. Search (`search`)

Tests search operation performance:

- `filename_search/{len}` - Filename search with various query lengths
- `content_search_files/{count}` - Content search in multiple files
- `content_search_large_file` - Search in large files (100K+ lines)
- `content_search_regex` - Regex-based content search

**Performance Goals**:
- Filename search (10K files): < 1ms
- Content search (100 files): < 100ms
- Large file search: < 50ms

### 4. UI Rendering (`ui_rendering`)

Simulates UI rendering performance:

- `render_file_list/{size}` - Render file lists of various sizes
- `render_split_view` - Dual-pane view rendering
- `render_tab_bar/{count}` - Tab bar rendering with many tabs

**Performance Goals**:
- 100 items: < 1ms
- 500 items: < 5ms
- Split view: < 2ms

### 5. File Operations (`file_operations`)

Tests file system operation performance:

- `copy_file/small` - Copy 1KB file
- `copy_file/medium` - Copy 1MB file
- `copy_file/large` - Copy 10MB file
- `traverse_directory/{depth}` - Directory traversal at various depths

**Performance Goals**:
- Small copy: < 1ms
- Medium copy: < 50ms
- Large copy: < 500ms
- Deep traversal (depth 5): < 100ms

### 6. Bulk Rename (`bulk_rename`)

Tests bulk rename operation performance:

- `preview_simple_replace/{count}` - Generate preview for simple replacement
- `preview_numbered/{count}` - Generate preview for numbered pattern
- `preview_regex/{count}` - Generate preview for regex pattern
- `preview_case_transform/{count}` - Generate preview for case transformation
- `parse_and_apply_patterns` - Pattern parsing and application

**Performance Goals**:
- 100 items: < 1ms
- 1,000 items: < 10ms
- 10,000 items: < 100ms

### 7. Filtering (`filtering`)

Tests file filtering performance:

- `show_hidden/{count}` - Filter with hidden files
- `no_hidden/{count}` - Filter without hidden files
- `search_query_large` - Filter with search query
- `combined_filters` - Combined filtering operations

**Performance Goals**:
- 10K items with filters: < 1ms

### 8. Memory (`memory`)

Tests memory allocation patterns:

- `create_file_entries_1000` - FileEntry creation overhead
- `clone_large_vec` - Vec cloning for filtering
- `format_display_strings_1000` - Display string formatting

**Performance Goals**:
- Minimize allocations in hot paths
- Reuse buffers where possible

### 9. Sort Algorithms (`sort_algorithms`)

Compares different sorting algorithms:

- `std_sort` - Standard library stable sort
- `std_sort_unstable` - Standard library unstable sort

**Performance Notes**:
- Unstable sort is typically 10-20% faster
- Consider using unstable sort if stability is not required

### 10. Throughput (`throughput`)

Measures throughput for data processing:

- `bytes_per_second/{size}` - Bytes processed per second
- `filter_throughput/filter_10000_items` - Items filtered per second

## Interpreting Results

Benchmark results are saved to `target/criterion/` directory:

```
target/criterion/
├── file_listing/
│   ├── read_directory/
│   │   ├── new/
│   │   │   ├── sample.json
│   │   │   ├── tukey.json
│   │   │   └── ...
│   │   └── change/
│   └── ...
├── report/
│   └── index.html
```

Open `target/criterion/report/index.html` in a browser for detailed results and visualizations.

## Continuous Benchmarking

For CI/CD integration:

```bash
# Save baseline in CI
cargo bench -- --save-baseline ci

# Compare locally against CI baseline
cargo bench -- --baseline ci
```

## Writing New Benchmarks

When adding new benchmarks:

1. **Use `black_box`**: Prevent compiler optimizations from eliminating code
   ```rust
   black_box(input_data)
   ```

2. **Batch large inputs**: Use appropriate batch sizes
   ```rust
   b.iter_batched(
       || setup_function(),
       |data| benchmark_function(data),
       criterion::BatchSize::LargeInput,
   );
   ```

3. **Use meaningful IDs**: Make benchmark names descriptive
   ```rust
   BenchmarkId::new("operation_name", parameter)
   ```

4. **Consider async**: Use runtime for async operations
   ```rust
   let rt = Runtime::new().unwrap();
   b.to_async(&rt).iter(|| async { ... });
   ```

5. **Document goals**: Add performance goals to this README

## Performance Optimization Checklist

Before optimizing, verify:

- [ ] Profile with `cargo flamegraph` or similar tools
- [ ] Identify actual bottlenecks (don't guess)
- [ ] Measure before and after changes
- [ ] Consider algorithmic improvements first
- [ ] Only optimize hot paths
- [ ] Document trade-offs

## Common Optimizations

### Sorting
- Use `sort_unstable` when stability is not required
- Consider pre-computing sort keys for complex criteria

### Filtering
- Use iterator chains instead of intermediate allocations
- Consider filtering in place when safe

### Search
- Index frequently searched fields
- Use specialized data structures (tries, hash maps)

### Memory
- Reuse allocations with `Vec::clear()` instead of dropping
- Use `Cow` for conditionally owned data
- Consider arena allocation for many small objects

## Troubleshooting

### Benchmarks are too slow
- Reduce input sizes in fixture creation
- Use `--sample-size` to reduce iterations
- Focus on specific benchmark groups

### Results are inconsistent
- Ensure no background processes are using CPU/disk
- Increase `--warm-up-time` for JIT-compiled effects
- Use `--measurement-time` for longer measurements

### "Too few iterations" error
- Increase `--sample-size` (default: 100)
- The operation may be too fast to benchmark accurately

## Contributing

When contributing performance improvements:

1. Add benchmarks for new code paths
2. Update this README with new benchmarks
3. Include baseline comparisons in PR description
4. Document any trade-offs or limitations

## Resources

- [Criterion User Guide](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Flamegraph Guide](https://github.com/flamegraph-rs/flamegraph)
