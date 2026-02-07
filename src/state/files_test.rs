//! Unit tests for file sorting, filtering, and display functionality.

use super::*;
use std::path::PathBuf;
use std::time::SystemTime;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_file(name: &str, size: u64, is_dir: bool) -> FileEntry {
        FileEntry {
            name: name.to_string(),
            path: PathBuf::from(format!("/test/{}", name)),
            is_dir,
            size,
            modified: SystemTime::UNIX_EPOCH,
            is_hidden: name.starts_with('.'),
            is_system: false,
            is_readonly: false,
            is_symlink: false,
        }
    }

    fn create_file_with_date(
        name: &str,
        size: u64,
        is_dir: bool,
        modified: SystemTime,
    ) -> FileEntry {
        FileEntry {
            name: name.to_string(),
            path: PathBuf::from(format!("/test/{}", name)),
            is_dir,
            size,
            modified,
            is_hidden: name.starts_with('.'),
            is_system: false,
            is_readonly: false,
            is_symlink: false,
        }
    }

    // ============================================================================
    // File Sorting Tests
    // ============================================================================

    mod sort_tests {
        use super::*;

        #[test]
        fn test_sort_by_name_ascending() {
            let mut files = vec![
                create_test_file("zebra.txt", 100, false),
                create_test_file("apple.txt", 100, false),
                create_test_file("banana.txt", 100, false),
            ];

            sort_files(&mut files, SortBy::Name, SortOrder::Ascending);

            assert_eq!(files[0].name, "apple.txt");
            assert_eq!(files[1].name, "banana.txt");
            assert_eq!(files[2].name, "zebra.txt");
        }

        #[test]
        fn test_sort_by_name_descending() {
            let mut files = vec![
                create_test_file("zebra.txt", 100, false),
                create_test_file("apple.txt", 100, false),
                create_test_file("banana.txt", 100, false),
            ];

            sort_files(&mut files, SortBy::Name, SortOrder::Descending);

            assert_eq!(files[0].name, "zebra.txt");
            assert_eq!(files[1].name, "banana.txt");
            assert_eq!(files[2].name, "apple.txt");
        }

        #[test]
        fn test_sort_by_name_case_insensitive() {
            let mut files = vec![
                create_test_file("Zebra.txt", 100, false),
                create_test_file("apple.txt", 100, false),
                create_test_file("Banana.txt", 100, false),
            ];

            sort_files(&mut files, SortBy::Name, SortOrder::Ascending);

            assert_eq!(files[0].name, "apple.txt");
            assert_eq!(files[1].name, "Banana.txt");
            assert_eq!(files[2].name, "Zebra.txt");
        }

        #[test]
        fn test_sort_by_name_directories_first() {
            let mut files = vec![
                create_test_file("zebra.txt", 100, false),
                create_test_file("documents", 0, true),
                create_test_file("apple.txt", 100, false),
                create_test_file("downloads", 0, true),
            ];

            sort_files(&mut files, SortBy::Name, SortOrder::Ascending);

            assert!(files[0].is_dir);
            assert!(files[1].is_dir);
            assert!(!files[2].is_dir);
            assert!(!files[3].is_dir);
            assert_eq!(files[0].name, "documents");
            assert_eq!(files[1].name, "downloads");
            assert_eq!(files[2].name, "apple.txt");
            assert_eq!(files[3].name, "zebra.txt");
        }

        #[test]
        fn test_sort_by_size_ascending() {
            let mut files = vec![
                create_test_file("large.txt", 10000, false),
                create_test_file("small.txt", 100, false),
                create_test_file("medium.txt", 1000, false),
            ];

            sort_files(&mut files, SortBy::Size, SortOrder::Ascending);

            assert_eq!(files[0].size, 100);
            assert_eq!(files[1].size, 1000);
            assert_eq!(files[2].size, 10000);
        }

        #[test]
        fn test_sort_by_size_descending() {
            let mut files = vec![
                create_test_file("large.txt", 10000, false),
                create_test_file("small.txt", 100, false),
                create_test_file("medium.txt", 1000, false),
            ];

            sort_files(&mut files, SortBy::Size, SortOrder::Descending);

            assert_eq!(files[0].size, 10000);
            assert_eq!(files[1].size, 1000);
            assert_eq!(files[2].size, 100);
        }

        #[test]
        fn test_sort_by_size_directories_first() {
            let mut files = vec![
                create_test_file("large.txt", 10000, false),
                create_test_file("dir1", 0, true),
                create_test_file("small.txt", 100, false),
                create_test_file("dir2", 0, true),
            ];

            sort_files(&mut files, SortBy::Size, SortOrder::Ascending);

            assert!(files[0].is_dir);
            assert!(files[1].is_dir);
            assert_eq!(files[2].size, 100);
            assert_eq!(files[3].size, 10000);
        }

        #[test]
        fn test_sort_by_date_ascending() {
            let now = SystemTime::now();
            let earlier = now - std::time::Duration::from_secs(3600);
            let earliest = now - std::time::Duration::from_secs(7200);

            let mut files = vec![
                create_file_with_date("medium.txt", 100, false, earlier),
                create_file_with_date("newest.txt", 100, false, now),
                create_file_with_date("oldest.txt", 100, false, earliest),
            ];

            sort_files(&mut files, SortBy::Modified, SortOrder::Ascending);

            assert_eq!(files[0].name, "oldest.txt");
            assert_eq!(files[1].name, "medium.txt");
            assert_eq!(files[2].name, "newest.txt");
        }

        #[test]
        fn test_sort_by_date_descending() {
            let now = SystemTime::now();
            let earlier = now - std::time::Duration::from_secs(3600);
            let earliest = now - std::time::Duration::from_secs(7200);

            let mut files = vec![
                create_file_with_date("medium.txt", 100, false, earlier),
                create_file_with_date("newest.txt", 100, false, now),
                create_file_with_date("oldest.txt", 100, false, earliest),
            ];

            sort_files(&mut files, SortBy::Modified, SortOrder::Descending);

            assert_eq!(files[0].name, "newest.txt");
            assert_eq!(files[1].name, "medium.txt");
            assert_eq!(files[2].name, "oldest.txt");
        }

        #[test]
        fn test_sort_by_type_ascending() {
            let mut files = vec![
                create_test_file("document.txt", 100, false),
                create_test_file("image.png", 100, false),
                create_test_file("archive.tar.gz", 100, false),
                create_test_file("backup.txt", 100, false),
            ];

            sort_files(&mut files, SortBy::Type, SortOrder::Ascending);

            // Files with same extension should be grouped
            // gz comes before png, png comes before txt
            assert!(files[0].name.contains("tar.gz"));
            assert_eq!(files[2].name, "image.png");
            assert!(files[3].name.contains("txt"));
        }

        #[test]
        fn test_sort_by_type_descending() {
            let mut files = vec![
                create_test_file("document.txt", 100, false),
                create_test_file("image.png", 100, false),
                create_test_file("archive.tar.gz", 100, false),
            ];

            sort_files(&mut files, SortBy::Type, SortOrder::Descending);

            // Reverse order
            assert!(files[0].name.contains("txt"));
            assert_eq!(files[1].name, "image.png");
            assert!(files[2].name.contains("tar.gz"));
        }

        #[test]
        fn test_sort_by_type_no_extension() {
            let mut files = vec![
                create_test_file("README", 100, false),
                create_test_file("document.txt", 100, false),
                create_test_file("Makefile", 100, false),
            ];

            sort_files(&mut files, SortBy::Type, SortOrder::Ascending);

            // Files without extension come first
            assert_eq!(files[0].name, "README");
            assert_eq!(files[1].name, "Makefile");
            assert_eq!(files[2].name, "document.txt");
        }

        #[test]
        fn test_sort_empty_list() {
            let mut files: Vec<FileEntry> = vec![];

            sort_files(&mut files, SortBy::Name, SortOrder::Ascending);

            assert!(files.is_empty());
        }

        #[test]
        fn test_sort_single_item() {
            let mut files = vec![create_test_file("single.txt", 100, false)];

            sort_files(&mut files, SortBy::Name, SortOrder::Ascending);

            assert_eq!(files.len(), 1);
            assert_eq!(files[0].name, "single.txt");
        }
    }

    // ============================================================================
    // Filter Tests
    // ============================================================================

    mod filter_tests {
        use super::*;

        #[test]
        fn test_filter_files_no_filters() {
            let files = vec![
                create_test_file("file1.txt", 100, false),
                create_test_file("file2.txt", 100, false),
                create_test_file("file3.txt", 100, false),
            ];

            let indices = filter_files(&files, true, None);

            assert_eq!(indices.len(), 3);
            assert_eq!(indices, vec![0, 1, 2]);
        }

        #[test]
        fn test_filter_files_hide_hidden() {
            let files = vec![
                create_test_file("visible.txt", 100, false),
                create_test_file(".hidden", 100, false),
                create_test_file("normal.txt", 100, false),
            ];

            let indices = filter_files(&files, false, None);

            assert_eq!(indices.len(), 2);
            assert_eq!(indices, vec![0, 2]);
        }

        #[test]
        fn test_filter_files_show_hidden() {
            let files = vec![
                create_test_file("visible.txt", 100, false),
                create_test_file(".hidden", 100, false),
                create_test_file("normal.txt", 100, false),
            ];

            let indices = filter_files(&files, true, None);

            assert_eq!(indices.len(), 3);
            assert_eq!(indices, vec![0, 1, 2]);
        }

        #[test]
        fn test_filter_files_with_search_query() {
            let files = vec![
                create_test_file("document.txt", 100, false),
                create_test_file("image.png", 100, false),
                create_test_file("data.csv", 100, false),
            ];

            let indices = filter_files(&files, true, Some("doc"));

            assert_eq!(indices.len(), 1);
            assert_eq!(indices[0], 0);
        }

        #[test]
        fn test_filter_files_search_case_insensitive() {
            let files = vec![
                create_test_file("Document.txt", 100, false),
                create_test_file("document.txt", 100, false),
                create_test_file("DOCUMENT.TXT", 100, false),
            ];

            let indices = filter_files(&files, true, Some("doc"));

            assert_eq!(indices.len(), 3);
        }

        #[test]
        fn test_filter_files_search_empty_query() {
            let files = vec![
                create_test_file("file1.txt", 100, false),
                create_test_file("file2.txt", 100, false),
            ];

            let indices = filter_files(&files, true, Some(""));

            // Empty query should match all files
            assert_eq!(indices.len(), 2);
        }

        #[test]
        fn test_filter_files_combined_filters() {
            let files = vec![
                create_test_file("visible_doc.txt", 100, false),
                create_test_file(".hidden_doc.txt", 100, false),
                create_test_file("visible_img.png", 100, false),
                create_test_file("visible_data.txt", 100, false),
            ];

            // Hide hidden and search for "doc"
            let indices = filter_files(&files, false, Some("doc"));

            assert_eq!(indices.len(), 2);
            assert_eq!(indices, vec![0, 3]);
        }

        #[test]
        fn test_filter_files_no_matches() {
            let files = vec![
                create_test_file("file1.txt", 100, false),
                create_test_file("file2.txt", 100, false),
            ];

            let indices = filter_files(&files, true, Some("nonexistent"));

            assert_eq!(indices.len(), 0);
        }

        #[test]
        fn test_filter_files_system_files_hidden() {
            let mut files = vec![
                create_test_file("normal.txt", 100, false),
                create_test_file("system.txt", 100, false),
            ];

            files[1].is_system = true;

            let indices = filter_files(&files, false, None);

            assert_eq!(indices.len(), 1);
            assert_eq!(indices[0], 0);
        }

        #[test]
        fn test_filter_files_empty_list() {
            let files: Vec<FileEntry> = vec![];

            let indices = filter_files(&files, true, None);

            assert_eq!(indices.len(), 0);
        }
    }

    // ============================================================================
    // Display Size Tests
    // ============================================================================

    mod display_size_tests {
        use super::*;

        #[test]
        fn test_display_size_bytes() {
            let file = create_test_file("tiny.txt", 512, false);

            assert_eq!(file.display_size(), "512 B");
        }

        #[test]
        fn test_display_size_kilobytes() {
            let file = create_test_file("small.txt", 2048, false);

            assert_eq!(file.display_size(), "2.0 KB");
        }

        #[test]
        fn test_display_size_megabytes() {
            let file = create_test_file("medium.txt", 2_500_000, false);

            assert_eq!(file.display_size(), "2.4 MB");
        }

        #[test]
        fn test_display_size_gigabytes() {
            let file = create_test_file("large.bin", 1_500_000_000, false);

            assert_eq!(file.display_size(), "1.40 GB");
        }

        #[test]
        fn test_display_size_zero() {
            let file = create_test_file("empty.txt", 0, false);

            assert_eq!(file.display_size(), "0 B");
        }

        #[test]
        fn test_display_size_directory() {
            let dir = create_test_file("documents", 0, true);

            assert_eq!(dir.display_size(), "<DIR>");
        }

        #[test]
        fn test_display_size_boundary_kb() {
            let file = create_test_file("boundary.txt", 1023, false);

            assert_eq!(file.display_size(), "1023 B");

            let file2 = create_test_file("boundary2.txt", 1024, false);

            assert_eq!(file2.display_size(), "1.0 KB");
        }

        #[test]
        fn test_display_size_boundary_mb() {
            let file = create_test_file("boundary.txt", 1_048_575, false);

            assert!(file.display_size().contains("KB"));

            let file2 = create_test_file("boundary2.txt", 1_048_576, false);

            assert_eq!(file2.display_size(), "1.0 MB");
        }

        #[test]
        fn test_display_size_boundary_gb() {
            let file = create_test_file("boundary.txt", 1_073_741_823, false);

            assert!(file.display_size().contains("MB"));

            let file2 = create_test_file("boundary2.txt", 1_073_741_824, false);

            assert_eq!(file2.display_size(), "1.00 GB");
        }

        #[test]
        fn test_display_size_precision() {
            let file1 = create_test_file("file1.txt", 1_500_000, false);
            let file2 = create_test_file("file2.txt", 1_234_567_890, false);

            // MB shows 1 decimal place
            assert_eq!(file1.display_size(), "1.4 MB");

            // GB shows 2 decimal places
            assert_eq!(file2.display_size(), "1.15 GB");
        }
    }

    // ============================================================================
    // Hidden File Detection Tests
    // ============================================================================

    mod hidden_file_tests {
        use super::*;

        #[test]
        fn test_hidden_file_unix_style() {
            let file = create_test_file(".hidden", 100, false);

            assert!(file.is_hidden);
        }

        #[test]
        fn test_hidden_file_double_dot() {
            let file = create_test_file("..config", 100, false);

            assert!(file.is_hidden);
        }

        #[test]
        fn test_visible_file_normal() {
            let file = create_test_file("visible.txt", 100, false);

            assert!(!file.is_hidden);
        }

        #[test]
        fn test_visible_file_dot_in_middle() {
            let file = create_test_file("file.name.txt", 100, false);

            assert!(!file.is_hidden);
        }

        #[test]
        fn test_hidden_directory() {
            let dir = create_test_file(".secret", 0, true);

            assert!(dir.is_hidden);
        }

        #[test]
        fn test_file_entry_creation_preserves_hidden_flag() {
            let file = FileEntry {
                name: ".test".to_string(),
                path: PathBuf::from("/test/.test"),
                is_dir: false,
                size: 0,
                modified: SystemTime::UNIX_EPOCH,
                is_hidden: true,
                is_system: false,
                is_readonly: false,
                is_symlink: false,
            };

            assert!(file.is_hidden);
            assert_eq!(file.name, ".test");
        }
    }

    // ============================================================================
    // FileEntry Extension Tests
    // ============================================================================

    mod extension_tests {
        use super::*;

        #[test]
        fn test_extension_simple() {
            let file = create_test_file("document.txt", 100, false);

            assert_eq!(file.extension(), Some("txt"));
        }

        #[test]
        fn test_extension_multiple_dots() {
            let file = create_test_file("archive.tar.gz", 100, false);

            // Extension is everything after the last dot
            assert_eq!(file.extension(), Some("gz"));
        }

        #[test]
        fn test_extension_no_extension() {
            let file = create_test_file("README", 100, false);

            assert_eq!(file.extension(), None);
        }

        #[test]
        fn test_extension_dot_only() {
            let file = create_test_file(".hidden", 100, false);

            assert_eq!(file.extension(), None);
        }

        #[test]
        fn test_extension_empty_name() {
            let file = FileEntry {
                name: "".to_string(),
                path: PathBuf::from("/test/"),
                is_dir: false,
                size: 0,
                modified: SystemTime::UNIX_EPOCH,
                is_hidden: false,
                is_system: false,
                is_readonly: false,
                is_symlink: false,
            };

            assert_eq!(file.extension(), None);
        }

        #[test]
        fn test_extension_case_sensitive() {
            let file = create_test_file("image.PNG", 100, false);

            assert_eq!(file.extension(), Some("PNG"));
        }
    }
}
