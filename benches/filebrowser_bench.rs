//! Comprehensive benchmarks for filebrowser-tui performance-critical operations.
//!
//! This benchmark suite measures performance across multiple dimensions:
//! - File listing operations at various scales
//! - Sorting performance with different criteria and algorithms
//! - Search operations (filename and content-based)
//! - UI rendering simulation for different list sizes
//! - File operations (copy, traversal)
//! - Bulk rename operations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use filebrowser_tui::file_ops::{
    bulk_rename::{BulkRenamer, RenamePattern},
    navigator::Navigator,
    search::{collect_files_in_directory, ContentSearcher, SearchConfig},
};
use filebrowser_tui::state::{files::sort_files, FileEntry, SortBy, SortOrder};
use std::path::PathBuf;
use tokio::runtime::Runtime;

mod mod;
use mod::{FixtureBuilder, create_mock_file_entries, create_sortable_entries};

/// Benchmark file listing operations
fn bench_file_listing(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_listing");

    // Benchmark directory reading with different file counts
    for file_count in [100, 1_000, 10_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("read_directory", file_count),
            file_count,
            |b, &count| {
                let fixture = FixtureBuilder::new().unwrap();
                let dir_path = fixture
                    .create_directory_with_files("bench_files", count, false)
                    .unwrap();
                let navigator = Navigator::new();

                b.iter(|| {
                    black_box(navigator.read_directory(black_box(&dir_path)))
                });
            },
        );
    }

    // Benchmark with hidden files
    group.bench_function("read_directory_with_hidden_1000", |b| {
        let fixture = FixtureBuilder::new().unwrap();
        let dir_path = fixture
            .create_directory_with_files("hidden_files", 1_000, true)
            .unwrap();
        let navigator = Navigator::new();

        b.iter(|| {
            black_box(navigator.read_directory(black_box(&dir_path)))
        });
    });

    group.finish();
}

/// Benchmark sorting operations
fn bench_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("sorting");

    // Test different sort criteria with varying file counts
    for file_count in [100, 1_000, 10_000].iter() {
        let mut entries = create_mock_file_entries(*file_count, false);

        // Sort by name
        group.bench_with_input(
            BenchmarkId::new("by_name", file_count),
            file_count,
            |b, _| {
                b.iter_batched(
                    || entries.clone(),
                    |mut data| {
                        sort_files(&mut data, SortBy::Name, SortOrder::Ascending);
                        black_box(data)
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );

        // Sort by size
        group.bench_with_input(
            BenchmarkId::new("by_size", file_count),
            file_count,
            |b, _| {
                b.iter_batched(
                    || entries.clone(),
                    |mut data| {
                        sort_files(&mut data, SortBy::Size, SortOrder::Descending);
                        black_box(data)
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );

        // Sort by modified time
        group.bench_with_input(
            BenchmarkId::new("by_modified", file_count),
            file_count,
            |b, _| {
                b.iter_batched(
                    || entries.clone(),
                    |mut data| {
                        sort_files(&mut data, SortBy::Modified, SortOrder::Descending);
                        black_box(data)
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );

        // Sort by type (extension)
        group.bench_with_input(
            BenchmarkId::new("by_type", file_count),
            file_count,
            |b, _| {
                b.iter_batched(
                    || entries.clone(),
                    |mut data| {
                        sort_files(&mut data, SortBy::Type, SortOrder::Ascending);
                        black_box(data)
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }

    // Compare ascending vs descending for the same criterion
    let mut entries = create_sortable_entries();
    group.bench_function("by_name_ascending", |b| {
        b.iter_batched(
            || entries.clone(),
            |mut data| {
                sort_files(&mut data, SortBy::Name, SortOrder::Ascending);
                black_box(data)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("by_name_descending", |b| {
        b.iter_batched(
            || entries.clone(),
            |mut data| {
                sort_files(&mut data, SortBy::Name, SortOrder::Descending);
                black_box(data)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark search operations
fn bench_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("search");

    // Filename search with various query lengths
    for query_len in [3, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("filename_search", query_len),
            query_len,
            |b, &len| {
                let entries = create_mock_file_entries(10_000, false);
                let query = "test".repeat(len / 4 + 1);
                let query = &query[..len.min(query.len())];

                b.iter(|| {
                    let mut results = Vec::new();
                    for entry in &entries {
                        if entry.name.to_lowercase().contains(&query.to_lowercase()) {
                            results.push(entry);
                        }
                    }
                    black_box(results)
                });
            },
        );
    }

    // Content search with varying file sizes
    let rt = Runtime::new().unwrap();

    for file_count in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("content_search_files", file_count),
            file_count,
            |b, &count| {
                let fixture = FixtureBuilder::new().unwrap();
                let dir_path = fixture
                    .create_searchable_files("search_bench", count, 100)
                    .unwrap();

                b.to_async(&rt).iter(|| async {
                    let files = collect_files_in_directory(&dir_path, true, None).await.unwrap();
                    let searcher = ContentSearcher::new();
                    let handle = searcher.search_files(
                        files.clone(),
                        "benchmark",
                        dir_path.clone(),
                    );
                    let results = handle.await.unwrap().unwrap();
                    black_box(results)
                });
            },
        );
    }

    // Large file content search
    group.bench_function("content_search_large_file", |b| {
        let fixture = FixtureBuilder::new().unwrap();
        let file_path = fixture
            .create_large_search_file("large_search.txt", 100_000)
            .unwrap();

        b.to_async(&rt).iter(|| async {
            let searcher = ContentSearcher::new();
            let handle = searcher.search_files(
                vec![file_path.clone()],
                "benchmark",
                fixture.base_path().to_path_buf(),
            );
            let results = handle.await.unwrap().unwrap();
            black_box(results)
        });
    });

    // Search with regex
    group.bench_function("content_search_regex", |b| {
        let fixture = FixtureBuilder::new().unwrap();
        let dir_path = fixture
            .create_searchable_files("regex_search", 100, 100)
            .unwrap();

        let config = SearchConfig {
            use_regex: true,
            case_sensitive: false,
            ..Default::default()
        };

        b.to_async(&rt).iter(|| async {
            let files = collect_files_in_directory(&dir_path, true, None).await.unwrap();
            let searcher = ContentSearcher::with_config(config.clone());
            let handle = searcher.search_files(
                files.clone(),
                r"bench.*mark",
                dir_path.clone(),
            );
            let results = handle.await.unwrap().unwrap();
            black_box(results)
        });
    });

    group.finish();
}

/// Benchmark UI rendering simulation
fn bench_ui_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("ui_rendering");

    // Simulate rendering different list sizes
    for list_size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("render_file_list", list_size),
            list_size,
            |b, &size| {
                let entries = create_mock_file_entries(size, false);

                b.iter(|| {
                    // Simulate rendering by processing each entry
                    let rendered: Vec<String> = entries
                        .iter()
                        .take(size)
                        .map(|e| {
                            format!(
                                "{:<40} {:>10} {}",
                                e.name,
                                e.display_size(),
                                e.display_modified()
                            )
                        })
                        .collect();
                    black_box(rendered)
                });
            },
        );
    }

    // Split view rendering (simulate dual pane)
    group.bench_function("render_split_view", |b| {
        let left_entries = create_mock_file_entries(100, false);
        let right_entries = create_mock_file_entries(100, false);

        b.iter(|| {
            let left_rendered: Vec<String> = left_entries
                .iter()
                .take(50)
                .map(|e| format!("{:<40} {:>10}", e.name, e.display_size()))
                .collect();

            let right_rendered: Vec<String> = right_entries
                .iter()
                .take(50)
                .map(|e| format!("{:<40} {:>10}", e.name, e.display_size()))
                .collect();

            black_box((left_rendered, right_rendered))
        });
    });

    // Tab bar rendering with many tabs
    for tab_count in [5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("render_tab_bar", tab_count),
            tab_count,
            |b, &count| {
                let tabs: Vec<String> = (0..count)
                    .map(|i| format!("Tab_{}", i))
                    .collect();

                b.iter(|| {
                    let tab_width = 20;
                    let total_width = 80;
                    let visible_tabs: Vec<&String> = tabs
                        .iter()
                        .take(total_width / tab_width)
                        .collect();

                    let rendered = visible_tabs
                        .iter()
                        .enumerate()
                        .map(|(i, tab)| {
                            let prefix = if i == 0 { "[" } else { " " };
                            format!("{}{:^18}]", prefix, tab)
                        })
                        .collect::<String>();

                    black_box(rendered)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark file operations
fn bench_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_operations");

    let rt = Runtime::new().unwrap();

    // Copy operations for different file sizes
    for size in ["small", "medium", "large"].iter() {
        group.bench_with_input(
            BenchmarkId::new("copy_file", size),
            size,
            |b, &_size| {
                let fixture = FixtureBuilder::new().unwrap();
                let dir_path = fixture.create_sized_files("copy_test").unwrap();

                let src_path = match *size {
                    "small" => dir_path.join("small.txt"),
                    "medium" => dir_path.join("medium.txt"),
                    "large" => dir_path.join("large.txt"),
                    _ => dir_path.join("small.txt"),
                };

                b.to_async(&rt).iter(|| async {
                    let dst_path = fixture.base_path().join("temp_copy.txt");
                    let _ = tokio::fs::copy(&src_path, &dst_path).await;
                    let _ = tokio::fs::remove_file(&dst_path).await;
                    black_box(())
                });
            },
        );
    }

    // Directory traversal speed
    for depth in [1, 3, 5].iter() {
        group.bench_with_input(
            BenchmarkId::new("traverse_directory", depth),
            depth,
            |b, &d| {
                let fixture = FixtureBuilder::new().unwrap();
                let dir_path = fixture
                    .create_nested_structure("traverse_test", d, 10)
                    .unwrap();

                b.to_async(&rt).iter(|| async {
                    let files = collect_files_in_directory(&dir_path, true, None).await.unwrap();
                    black_box(files)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark bulk rename operations
fn bench_bulk_rename(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_rename");

    // Preview generation for various item counts
    for item_count in [100, 1_000, 10_000].iter() {
        // Simple replace pattern
        group.bench_with_input(
            BenchmarkId::new("preview_simple_replace", item_count),
            item_count,
            |b, &count| {
                let fixture = FixtureBuilder::new().unwrap();
                let dir_path = fixture
                    .create_directory_with_files("rename_test", count, false)
                    .unwrap();

                let files: Vec<PathBuf> = dir_path
                    .read_dir()
                    .unwrap()
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .collect();

                let pattern = RenamePattern::SimpleReplace {
                    find: "file".to_string(),
                    replace: "document".to_string(),
                };
                let renamer = BulkRenamer::new(pattern, dir_path.clone());

                b.iter(|| {
                    black_box(renamer.preview(black_box(&files)))
                });
            },
        );

        // Numbered pattern
        group.bench_with_input(
            BenchmarkId::new("preview_numbered", item_count),
            item_count,
            |b, &count| {
                let fixture = FixtureBuilder::new().unwrap();
                let dir_path = fixture
                    .create_directory_with_files("numbered_test", count, false)
                    .unwrap();

                let files: Vec<PathBuf> = dir_path
                    .read_dir()
                    .unwrap()
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .collect();

                let pattern = RenamePattern::Numbered {
                    template: "item_{n}.txt".to_string(),
                    start: 1,
                    pad_width: 4,
                };
                let renamer = BulkRenamer::new(pattern, dir_path.clone());

                b.iter(|| {
                    black_box(renamer.preview(black_box(&files)))
                });
            },
        );

        // Regex pattern
        group.bench_with_input(
            BenchmarkId::new("preview_regex", item_count),
            item_count,
            |b, &count| {
                let fixture = FixtureBuilder::new().unwrap();
                let dir_path = fixture
                    .create_directory_with_files("regex_test", count, false)
                    .unwrap();

                let files: Vec<PathBuf> = dir_path
                    .read_dir()
                    .unwrap()
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .collect();

                let pattern = RenamePattern::Regex {
                    pattern: r"file_(\d+)".to_string(),
                    replacement: "doc_$1".to_string(),
                };
                let renamer = BulkRenamer::new(pattern, dir_path.clone());

                b.iter(|| {
                    black_box(renamer.preview(black_box(&files)))
                });
            },
        );

        // Case transformation
        group.bench_with_input(
            BenchmarkId::new("preview_case_transform", item_count),
            item_count,
            |b, &count| {
                let fixture = FixtureBuilder::new().unwrap();
                let dir_path = fixture
                    .create_directory_with_files("case_test", count, false)
                    .unwrap();

                let files: Vec<PathBuf> = dir_path
                    .read_dir()
                    .unwrap()
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .collect();

                use filebrowser_tui::file_ops::bulk_rename::{CaseTransform, CaseScope};
                let pattern = RenamePattern::Case {
                    transform: CaseTransform::Uppercase,
                    scope: CaseScope::NameOnly,
                };
                let renamer = BulkRenamer::new(pattern, dir_path.clone());

                b.iter(|| {
                    black_box(renamer.preview(black_box(&files)))
                });
            },
        );
    }

    // Pattern parsing and application
    group.bench_function("parse_and_apply_patterns", |b| {
        let patterns = vec![
            RenamePattern::SimpleReplace {
                find: "old".to_string(),
                replace: "new".to_string(),
            },
            RenamePattern::Numbered {
                template: "file_{n}.txt".to_string(),
                start: 1,
                pad_width: 3,
            },
            RenamePattern::Regex {
                pattern: r"(.*)\.txt".to_string(),
                replacement: "$1.md".to_string(),
            },
        ];

        let fixture = FixtureBuilder::new().unwrap();
        let dir_path = fixture
            .create_directory_with_files("pattern_test", 100, false)
            .unwrap();

        let files: Vec<PathBuf> = dir_path
            .read_dir()
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();

        b.iter(|| {
            for pattern in &patterns {
                let renamer = BulkRenamer::new(pattern.clone(), dir_path.clone());
                black_box(renamer.preview(&files));
            }
        });
    });

    group.finish();
}

/// Benchmark filtering operations
fn bench_filtering(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtering");

    // Filter with hidden files
    for file_count in [100, 1_000, 10_000].iter() {
        let entries_with_hidden = create_mock_file_entries(*file_count, true);
        let entries_no_hidden = create_mock_file_entries(*file_count, false);

        group.bench_with_input(
            BenchmarkId::new("show_hidden", file_count),
            file_count,
            |b, _| {
                b.iter(|| {
                    let filtered: Vec<_> = entries_with_hidden
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| !e.is_hidden && !e.is_system)
                        .map(|(i, _)| i)
                        .collect();
                    black_box(filtered)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("no_hidden", file_count),
            file_count,
            |b, _| {
                b.iter(|| {
                    let filtered: Vec<_> = entries_no_hidden
                        .iter()
                        .enumerate()
                        .map(|(i, _)| i)
                        .collect();
                    black_box(filtered)
                });
            },
        );
    }

    // Filter with search query
    group.bench_function("search_query_large", |b| {
        let entries = create_mock_file_entries(10_000, false);
        let query = "file_5";

        b.iter(|| {
            let filtered: Vec<_> = entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.name.to_lowercase().contains(&query.to_lowercase())
                })
                .map(|(i, _)| i)
                .collect();
            black_box(filtered)
        });
    });

    // Combined filters (hidden + search)
    group.bench_function("combined_filters", |b| {
        let entries = create_mock_file_entries(10_000, true);
        let query = "file_1";

        b.iter(|| {
            let filtered: Vec<_> = entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    !e.is_hidden && !e.is_system &&
                    e.name.to_lowercase().contains(&query.to_lowercase())
                })
                .map(|(i, _)| i)
                .collect();
            black_box(filtered)
        });
    });

    group.finish();
}

/// Benchmark memory allocations
fn bench_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    // FileEntry creation
    group.bench_function("create_file_entries_1000", |b| {
        b.iter(|| {
            let entries: Vec<FileEntry> = (0..1000)
                .map(|i| FileEntry {
                    name: format!("file_{}.txt", i),
                    path: PathBuf::from(format!("/tmp/test/file_{}.txt", i)),
                    is_dir: false,
                    size: 1024,
                    modified: std::time::SystemTime::UNIX_EPOCH,
                    is_hidden: false,
                    is_system: false,
                    is_readonly: false,
                    is_symlink: false,
                })
                .collect();
            black_box(entries)
        });
    });

    // Vec cloning for filtering operations
    group.bench_function("clone_large_vec", |b| {
        let entries = create_mock_file_entries(10_000, false);
        b.iter(|| {
            black_box(entries.clone())
        });
    });

    // String operations for display
    group.bench_function("format_display_strings_1000", |b| {
        let entries = create_mock_file_entries(1_000, false);
        b.iter(|| {
            let formatted: Vec<String> = entries
                .iter()
                .take(1000)
                .map(|e| e.display_size())
                .collect();
            black_box(formatted)
        });
    });

    group.finish();
}

/// Custom comparison benchmark: sorting algorithms
fn bench_sort_algorithms(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_algorithms");

    let mut entries = create_sortable_entries();

    // Standard library sort (used by sort_files)
    group.bench_function("std_sort", |b| {
        b.iter_batched(
            || entries.clone(),
            |mut data| {
                data.sort_by(|a, b| {
                    if a.is_dir != b.is_dir {
                        return b.is_dir.cmp(&a.is_dir);
                    }
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                });
                black_box(data)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Unstable sort (potentially faster)
    group.bench_function("std_sort_unstable", |b| {
        b.iter_batched(
            || entries.clone(),
            |mut data| {
                data.sort_unstable_by(|a, b| {
                    if a.is_dir != b.is_dir {
                        return b.is_dir.cmp(&a.is_dir);
                    }
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                });
                black_box(data)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Throughput benchmarks for operations that process data
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    // File listing throughput
    for size in [1024, 10_240, 102_400].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("bytes_per_second", size),
            size,
            |b, _| {
                let entries = create_mock_file_entries(1000, false);
                b.iter(|| {
                    let total_size: u64 = entries.iter().map(|e| e.size).sum();
                    black_box(total_size)
                });
            },
        );
    }

    // Items per second for filtering
    let mut group2 = c.benchmark_group("filter_throughput");
    group2.throughput(Throughput::Elements(10000));

    group2.bench_function("filter_10000_items", |b| {
        let entries = create_mock_file_entries(10_000, true);
        b.iter(|| {
            let filtered: Vec<_> = entries
                .iter()
                .filter(|e| !e.is_hidden)
                .collect();
            black_box(filtered)
        });
    });

    group2.finish();
    group.finish();
}

criterion_group!(
    benches,
    bench_file_listing,
    bench_sorting,
    bench_search,
    bench_ui_rendering,
    bench_file_operations,
    bench_bulk_rename,
    bench_filtering,
    bench_memory,
    bench_sort_algorithms,
    bench_throughput
);

criterion_main!(benches);
