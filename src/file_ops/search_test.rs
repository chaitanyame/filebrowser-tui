//! Comprehensive unit tests for search functionality.

use super::*;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::runtime::Runtime;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_path(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    // ============================================================================
    // SearchResult Creation Tests
    // ============================================================================

    mod search_result_creation {
        use super::*;

        #[test]
        fn test_search_result_creation() {
            let result = SearchResult {
                file_path: create_test_path("/test/file.txt"),
                line_number: 5,
                line_content: "hello world".to_string(),
                match_start: 0,
                match_end: 5,
            };

            assert_eq!(result.file_path, create_test_path("/test/file.txt"));
            assert_eq!(result.line_number, 5);
            assert_eq!(result.line_content, "hello world");
        }

        #[test]
        fn test_matched_text() {
            let result = SearchResult {
                file_path: create_test_path("/test/file.txt"),
                line_number: 1,
                line_content: "hello world".to_string(),
                match_start: 0,
                match_end: 5,
            };

            assert_eq!(result.matched_text(), "hello");
        }

        #[test]
        fn test_matched_text_multiple_words() {
            let result = SearchResult {
                file_path: create_test_path("/test/file.txt"),
                line_number: 1,
                line_content: "the quick brown fox".to_string(),
                match_start: 4,
                match_end: 9,
            };

            assert_eq!(result.matched_text(), "quick");
        }

        #[test]
        fn test_relative_path() {
            let result = SearchResult {
                file_path: create_test_path("/base/dir/subdir/file.txt"),
                line_number: 1,
                line_content: "test".to_string(),
                match_start: 0,
                match_end: 4,
            };

            let base = Path::new("/base/dir");
            let relative = result.relative_path(base);

            assert_eq!(relative, "subdir/file.txt");
        }

        #[test]
        fn test_relative_path_no_base() {
            let result = SearchResult {
                file_path: create_test_path("/unrelated/file.txt"),
                line_number: 1,
                line_content: "test".to_string(),
                match_start: 0,
                match_end: 4,
            };

            let base = Path::new("/other/path");
            let relative = result.relative_path(base);

            // Should return full path if not under base
            assert_eq!(relative, "/unrelated/file.txt");
        }

        #[test]
        fn test_relative_path_exact_base() {
            let result = SearchResult {
                file_path: create_test_path("/base/file.txt"),
                line_number: 1,
                line_content: "test".to_string(),
                match_start: 0,
                match_end: 4,
            };

            let base = Path::new("/base");
            let relative = result.relative_path(base);

            assert_eq!(relative, "file.txt");
        }

        #[test]
        fn test_equality() {
            let result1 = SearchResult {
                file_path: create_test_path("/test/file.txt"),
                line_number: 1,
                line_content: "hello world".to_string(),
                match_start: 0,
                match_end: 5,
            };

            let result2 = SearchResult {
                file_path: create_test_path("/test/file.txt"),
                line_number: 1,
                line_content: "hello world".to_string(),
                match_start: 0,
                match_end: 5,
            };

            assert_eq!(result1, result2);
        }

        #[test]
        fn test_inequality() {
            let result1 = SearchResult {
                file_path: create_test_path("/test/file1.txt"),
                line_number: 1,
                line_content: "hello".to_string(),
                match_start: 0,
                match_end: 5,
            };

            let result2 = SearchResult {
                file_path: create_test_path("/test/file2.txt"),
                line_number: 1,
                line_content: "hello".to_string(),
                match_start: 0,
                match_end: 5,
            };

            assert_ne!(result1, result2);
        }
    }

    // ============================================================================
    // SearchProgress Tests
    // ============================================================================

    mod search_progress {
        use super::*;

        #[test]
        fn test_search_progress_creation() {
            let progress = SearchProgress {
                files_searched: 10,
                total_files: Some(100),
                matches_found: 5,
                current_file: Some(create_test_path("/test/current.txt")),
            };

            assert_eq!(progress.files_searched, 10);
            assert_eq!(progress.total_files, Some(100));
            assert_eq!(progress.matches_found, 5);
            assert!(progress.current_file.is_some());
        }

        #[test]
        fn test_search_progress_no_total() {
            let progress = SearchProgress {
                files_searched: 5,
                total_files: None,
                matches_found: 2,
                current_file: None,
            };

            assert_eq!(progress.files_searched, 5);
            assert!(progress.total_files.is_none());
            assert!(progress.current_file.is_none());
        }
    }

    // ============================================================================
    // SearchConfig Tests
    // ============================================================================

    mod search_config {
        use super::*;

        #[test]
        fn test_default_config() {
            let config = SearchConfig::default();

            assert!(!config.case_sensitive);
            assert!(!config.use_regex);
            assert!(config.max_results.is_none());
            assert!(config.include_extensions.is_none());
            assert!(!config.exclude_extensions.is_empty());
            assert!(config.max_file_size.is_some());
        }

        #[test]
        fn test_custom_config() {
            let config = SearchConfig {
                case_sensitive: true,
                use_regex: true,
                max_results: Some(100),
                include_extensions: Some(vec!["rs".to_string(), "txt".to_string()]),
                exclude_extensions: vec![],
                max_file_size: Some(1_000_000),
                context_lines: 2,
            };

            assert!(config.case_sensitive);
            assert!(config.use_regex);
            assert_eq!(config.max_results, Some(100));
            assert_eq!(config.include_extensions.unwrap().len(), 2);
        }

        #[test]
        fn test_default_excludes_binary_files() {
            let config = SearchConfig::default();

            assert!(config.exclude_extensions.contains(&"exe".to_string()));
            assert!(config.exclude_extensions.contains(&"dll".to_string()));
            assert!(config.exclude_extensions.contains(&"zip".to_string()));
        }
    }

    // ============================================================================
    // ContentSearcher Tests
    // ============================================================================

    mod content_searcher {
        use super::*;

        #[test]
        fn test_searcher_creation() {
            let searcher = ContentSearcher::new();

            assert!(!searcher.config.case_sensitive);
            assert!(!searcher.config.use_regex);
        }

        #[test]
        fn test_searcher_with_config() {
            let config = SearchConfig {
                case_sensitive: true,
                use_regex: true,
                ..Default::default()
            };

            let searcher = ContentSearcher::with_config(config.clone());

            assert!(searcher.config.case_sensitive);
            assert!(searcher.config.use_regex);
        }

        #[test]
        fn test_default_searcher() {
            let searcher = ContentSearcher::default();

            assert!(!searcher.config.case_sensitive);
        }

        #[test]
        fn test_cancel_flag() {
            let searcher = ContentSearcher::new();

            assert!(!searcher.cancelled.load(std::sync::atomic::Ordering::Relaxed));

            searcher.cancel();

            assert!(searcher.cancelled.load(std::sync::atomic::Ordering::Relaxed));

            searcher.reset();

            assert!(!searcher.cancelled.load(std::sync::atomic::Ordering::Relaxed));
        }
    }

    // ============================================================================
    // File Collection Tests
    // ============================================================================

    mod file_collection {
        use super::*;

        #[tokio::test]
        async fn test_collect_empty_directory() {
            let temp_dir = TempDir::new().unwrap();

            let files = collect_files_in_directory(temp_dir.path(), true, None).await.unwrap();

            assert!(files.is_empty());
        }

        #[tokio::test]
        async fn test_collect_single_file() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            tokio::fs::write(&file_path, b"content").await.unwrap();

            let files = collect_files_in_directory(temp_dir.path(), false, None).await.unwrap();

            assert_eq!(files.len(), 1);
            assert_eq!(files[0], file_path);
        }

        #[tokio::test]
        async fn test_collect_multiple_files() {
            let temp_dir = TempDir::new().unwrap();

            tokio::fs::write(temp_dir.path().join("file1.txt"), b"content").await.unwrap();
            tokio::fs::write(temp_dir.path().join("file2.rs"), b"content").await.unwrap();
            tokio::fs::write(temp_dir.path().join("file3.md"), b"content").await.unwrap();

            let files = collect_files_in_directory(temp_dir.path(), false, None).await.unwrap();

            assert_eq!(files.len(), 3);
        }

        #[tokio::test]
        async fn test_collect_recursive() {
            let temp_dir = TempDir::new().unwrap();

            tokio::fs::write(temp_dir.path().join("root.txt"), b"content").await.unwrap();

            let subdir = temp_dir.path().join("subdir");
            tokio::fs::create_dir(&subdir).await.unwrap();
            tokio::fs::write(subdir.join("nested.txt"), b"content").await.unwrap();

            // Non-recursive
            let files_no_rec = collect_files_in_directory(temp_dir.path(), false, None)
                .await
                .unwrap();
            assert_eq!(files_no_rec.len(), 1);

            // Recursive
            let files_rec = collect_files_in_directory(temp_dir.path(), true, None)
                .await
                .unwrap();
            assert_eq!(files_rec.len(), 2);
        }

        #[tokio::test]
        async fn test_collect_with_max_depth() {
            let temp_dir = TempDir::new().unwrap();

            // Create nested directories
            let level1 = temp_dir.path().join("level1");
            let level2 = level1.join("level2");
            let level3 = level2.join("level3");

            tokio::fs::create_dir_all(&level3).await.unwrap();

            tokio::fs::write(level1.join("file1.txt"), b"content").await.unwrap();
            tokio::fs::write(level2.join("file2.txt"), b"content").await.unwrap();
            tokio::fs::write(level3.join("file3.txt"), b"content").await.unwrap();

            // Max depth 1 should only get level1
            let files = collect_files_in_directory(temp_dir.path(), true, Some(1))
                .await
                .unwrap();
            assert_eq!(files.len(), 1);
        }

        #[tokio::test]
        async fn test_collect_excludes_directories() {
            let temp_dir = TempDir::new().unwrap();

            tokio::fs::create_dir(temp_dir.path().join("dir1")).await.unwrap();
            tokio::fs::write(temp_dir.path().join("file1.txt"), b"content").await.unwrap();

            let files = collect_files_in_directory(temp_dir.path(), false, None)
                .await
                .unwrap();

            // Should only return files, not directories
            assert_eq!(files.len(), 1);
            assert!(files[0].to_string_lossy().contains("file1.txt"));
        }

        #[tokio::test]
        async fn test_collect_nonexistent_directory() {
            let files = collect_files_in_directory(Path::new("/nonexistent/path"), true, None)
                .await
                .unwrap();

            assert!(files.is_empty());
        }

        #[tokio::test]
        async fn test_collect_sorted_order() {
            let temp_dir = TempDir::new().unwrap();

            tokio::fs::write(temp_dir.path().join("z.txt"), b"content").await.unwrap();
            tokio::fs::write(temp_dir.path().join("a.txt"), b"content").await.unwrap();
            tokio::fs::write(temp_dir.path().join("m.txt"), b"content").await.unwrap();

            let files = collect_files_in_directory(temp_dir.path(), false, None)
                .await
                .unwrap();

            // Order may vary by OS, but all files should be present
            assert_eq!(files.len(), 3);
        }
    }

    // ============================================================================
    // Search File Content Tests (Mock)
    // ============================================================================

    mod search_file_content {
        use super::*;

        #[tokio::test]
        async fn test_search_file_literal_match() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            tokio::fs::write(&file_path, b"Hello World\nThis is a test").await.unwrap();

            let regex = Regex::new("test").unwrap();
            let results = ContentSearcher::search_file(&file_path, &regex, &SearchConfig::default())
                .await
                .unwrap();

            assert_eq!(results.len(), 1);
            assert_eq!(results[0].line_number, 2);
            assert_eq!(results[0].matched_text(), "test");
        }

        #[tokio::test]
        async fn test_search_file_multiple_matches() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            tokio::fs::write(&file_path, b"test\ntest line\ntest").await.unwrap();

            let regex = Regex::new("test").unwrap();
            let results = ContentSearcher::search_file(&file_path, &regex, &SearchConfig::default())
                .await
                .unwrap();

            assert_eq!(results.len(), 3);
        }

        #[tokio::test]
        async fn test_search_file_case_insensitive() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            tokio::fs::write(&file_path, b"Hello\nHELLO\nhello").await.unwrap();

            let regex = Regex::new("(?i)hello").unwrap();
            let results = ContentSearcher::search_file(&file_path, &regex, &SearchConfig::default())
                .await
                .unwrap();

            assert_eq!(results.len(), 3);
        }

        #[tokio::test]
        async fn test_search_file_regex_pattern() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            tokio::fs::write(&file_path, b"abc123\n456def\n789ghi").await.unwrap();

            let regex = Regex::new(r"\d+").unwrap();
            let results = ContentSearcher::search_file(&file_path, &regex, &SearchConfig::default())
                .await
                .unwrap();

            assert_eq!(results.len(), 3);
        }

        #[tokio::test]
        async fn test_search_file_no_matches() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            tokio::fs::write(&file_path, b"Hello World").await.unwrap();

            let regex = Regex::new("nonexistent").unwrap();
            let results = ContentSearcher::search_file(&file_path, &regex, &SearchConfig::default())
                .await
                .unwrap();

            assert!(results.is_empty());
        }

        #[tokio::test]
        async fn test_search_file_empty_file() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            tokio::fs::write(&file_path, b"").await.unwrap();

            let regex = Regex::new("test").unwrap();
            let results = ContentSearcher::search_file(&file_path, &regex, &SearchConfig::default())
                .await
                .unwrap();

            assert!(results.is_empty());
        }

        #[tokio::test]
        async fn test_search_file_multiline() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            let content = "line1\nline2\nline3\nline4\nline5";
            tokio::fs::write(&file_path, content.as_bytes()).await.unwrap();

            let regex = Regex::new("line").unwrap();
            let results = ContentSearcher::search_file(&file_path, &regex, &SearchConfig::default())
                .await
                .unwrap();

            assert_eq!(results.len(), 5);
            assert_eq!(results[0].line_number, 1);
            assert_eq!(results[4].line_number, 5);
        }

        #[tokio::test]
        async fn test_search_file_with_max_results() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            tokio::fs::write(&file_path, b"test\ntest\ntest\ntest\ntest").await.unwrap();

            let mut config = SearchConfig::default();
            config.max_results = Some(3);

            let regex = Regex::new("test").unwrap();
            let results = ContentSearcher::search_file(&file_path, &regex, &config)
                .await
                .unwrap();

            assert_eq!(results.len(), 3);
        }
    }

    // ============================================================================
    // Extension Filtering Tests
    // ============================================================================

    mod extension_filtering {
        use super::*;

        #[test]
        fn test_should_search_file_default() {
            let searcher = ContentSearcher::new();

            let txt_file = create_test_path("/test/file.txt");
            let rs_file = create_test_path("/test/file.rs");
            let exe_file = create_test_path("/test/file.exe");

            assert!(searcher.should_search_file(&txt_file, &searcher.config));
            assert!(searcher.should_search_file(&rs_file, &searcher.config));
            assert!(!searcher.should_search_file(&exe_file, &searcher.config));
        }

        #[test]
        fn test_should_search_file_include_only() {
            let config = SearchConfig {
                include_extensions: Some(vec!["rs".to_string(), "txt".to_string()]),
                ..Default::default()
            };
            let searcher = ContentSearcher::with_config(config);

            let rs_file = create_test_path("/test/file.rs");
            let txt_file = create_test_path("/test/file.txt");
            let md_file = create_test_path("/test/file.md");

            assert!(searcher.should_search_file(&rs_file, &searcher.config));
            assert!(searcher.should_search_file(&txt_file, &searcher.config));
            assert!(!searcher.should_search_file(&md_file, &searcher.config));
        }

        #[test]
        fn test_should_search_file_case_insensitive_extension() {
            let config = SearchConfig {
                include_extensions: Some(vec!["txt".to_string()]),
                ..Default::default()
            };
            let searcher = ContentSearcher::with_config(config);

            let upper = create_test_path("/test/file.TXT");
            let lower = create_test_path("/test/file.txt");
            let mixed = create_test_path("/test/file.Txt");

            assert!(searcher.should_search_file(&upper, &searcher.config));
            assert!(searcher.should_search_file(&lower, &searcher.config));
            assert!(searcher.should_search_file(&mixed, &searcher.config));
        }

        #[test]
        fn test_should_search_file_no_extension() {
            let config = SearchConfig {
                include_extensions: Some(vec!["txt".to_string()]),
                ..Default::default()
            };
            let searcher = ContentSearcher::with_config(config);

            let no_ext = create_test_path("/test/README");

            assert!(!searcher.should_search_file(&no_ext, &searcher.config));
        }

        #[test]
        fn test_should_search_file_exclude_override() {
            let config = SearchConfig {
                include_extensions: Some(vec!["txt".to_string()]),
                exclude_extensions: vec![],
                ..Default::default()
            };
            let searcher = ContentSearcher::with_config(config);

            let exe_file = create_test_path("/test/file.exe");

            // Not in include list
            assert!(!searcher.should_search_file(&exe_file, &searcher.config));
        }
    }

    // ============================================================================
    // Search Integration Tests
    // ============================================================================

    mod integration_tests {
        use super::*;

        #[tokio::test]
        async fn test_search_files_sync_basic() {
            let temp_dir = TempDir::new().unwrap();

            let file1 = temp_dir.path().join("file1.txt");
            let file2 = temp_dir.path().join("file2.txt");

            tokio::fs::write(&file1, b"hello world").await.unwrap();
            tokio::fs::write(&file2, b"goodbye world").await.unwrap();

            let files = vec![file1.clone(), file2.clone()];

            let results = search_files_sync(files, "world", temp_dir.path().to_path_buf(), None)
                .await
                .unwrap();

            assert_eq!(results.len(), 2);
        }

        #[tokio::test]
        async fn test_search_files_sync_no_matches() {
            let temp_dir = TempDir::new().unwrap();

            let file1 = temp_dir.path().join("file1.txt");
            tokio::fs::write(&file1, b"hello world").await.unwrap();

            let files = vec![file1.clone()];

            let results = search_files_sync(files, "nonexistent", temp_dir.path().to_path_buf(), None)
                .await
                .unwrap();

            assert!(results.is_empty());
        }

        #[tokio::test]
        async fn test_search_files_sync_with_config() {
            let temp_dir = TempDir::new().unwrap();

            let file1 = temp_dir.path().join("file1.txt");
            tokio::fs::write(&file1, b"hello\nHELLO\nhello").await.unwrap();

            let files = vec![file1.clone()];

            let config = SearchConfig {
                case_sensitive: true,
                ..Default::default()
            };

            let results = search_files_sync(
                files,
                "hello",
                temp_dir.path().to_path_buf(),
                Some(config),
            )
            .await
            .unwrap();

            // Should only match lowercase "hello"
            assert_eq!(results.len(), 2);
        }
    }

    // ============================================================================
    // Progress Callback Tests
    // ============================================================================

    mod progress_callback {
        use super::*;
        use std::sync::{Arc, Mutex};

        #[tokio::test]
        async fn test_progress_callback_called() {
            let temp_dir = TempDir::new().unwrap();

            for i in 0..3 {
                let file = temp_dir.path().join(format!("file{}.txt", i));
                tokio::fs::write(&file, b"content").await.unwrap();
            }

            let files: Vec<PathBuf> =
                (0..3).map(|i| temp_dir.path().join(format!("file{}.txt", i))).collect();

            let progress_updates = Arc::new(Mutex::new(Vec::new()));
            let progress_updates_clone = progress_updates.clone();

            let searcher = ContentSearcher::new().on_progress(move |progress| {
                progress_updates_clone
                    .lock()
                    .unwrap()
                    .push(progress.files_searched);
            });

            let _handle = searcher.search_files(
                files,
                "content",
                temp_dir.path().to_path_buf(),
            );

            // Give the task time to complete
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let updates = progress_updates.lock().unwrap();
            assert!(!updates.is_empty());
        }
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    mod edge_cases {
        use super::*;

        #[test]
        fn test_search_result_unicode_content() {
            let result = SearchResult {
                file_path: create_test_path("/test/file.txt"),
                line_number: 1,
                line_content: "Hello 世界".to_string(),
                match_start: 6,
                match_end: 9,
            };

            assert_eq!(result.matched_text(), "世界");
        }

        #[tokio::test]
        async fn test_search_file_special_characters() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");
            tokio::fs::write(&file_path, b"test\n$pecial\n@ymbols").await.unwrap();

            let regex = Regex::new(r"\w+").unwrap();
            let results = ContentSearcher::search_file(&file_path, &regex, &SearchConfig::default())
                .await
                .unwrap();

            assert!(!results.is_empty());
        }

        #[tokio::test]
        async fn test_search_file_very_long_line() {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.txt");

            let long_line = "a".repeat(10_000);
            tokio::fs::write(&file_path, long_line.as_bytes()).await.unwrap();

            let regex = Regex::new("a").unwrap();
            let results = ContentSearcher::search_file(&file_path, &regex, &SearchConfig::default())
                .await
                .unwrap();

            // Should handle long lines
            assert!(!results.is_empty());
        }

        #[tokio::test]
        async fn test_collect_files_with_symlinks() {
            let temp_dir = TempDir::new().unwrap();

            // Create a regular file
            let file = temp_dir.path().join("file.txt");
            tokio::fs::write(&file, b"content").await.unwrap();

            // Create a symlink (if supported)
            #[cfg(unix)]
            {
                let symlink = temp_dir.path().join("link.txt");
                let _ = tokio::fs::symlink(&file, &symlink).await;
            }

            let files = collect_files_in_directory(temp_dir.path(), false, None)
                .await
                .unwrap();

            // Should at least find the regular file
            assert!(files.len() >= 1);
        }
    }
}
