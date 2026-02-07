//! Unit tests for SelectionManager functionality.

use super::*;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_path(index: usize) -> PathBuf {
        PathBuf::from(format!("/test/file{}", index))
    }

    fn create_test_files(count: usize) -> Vec<super::super::FileEntry> {
        (0..count)
            .map(|i| super::super::FileEntry {
                name: format!("file{}", i),
                path: create_test_path(i),
                is_dir: false,
                size: 100,
                modified: std::time::SystemTime::UNIX_EPOCH,
                is_hidden: false,
                is_system: false,
                is_readonly: false,
                is_symlink: false,
            })
            .collect()
    }

    // ============================================================================
    // Basic Selection Tests
    // ============================================================================

    mod basic_selection {
        use super::*;

        #[test]
        fn test_new_manager_is_empty() {
            let manager = SelectionManager::new();

            assert!(manager.is_empty());
            assert_eq!(manager.count(), 0);
        }

        #[test]
        fn test_default_manager_is_empty() {
            let manager = SelectionManager::default();

            assert!(manager.is_empty());
            assert_eq!(manager.count(), 0);
        }

        #[test]
        fn test_toggle_single_file() {
            let mut manager = SelectionManager::new();
            let path = create_test_path(0);

            assert!(!manager.is_selected(&path));

            manager.toggle(path.clone());

            assert!(manager.is_selected(&path));
            assert_eq!(manager.count(), 1);
        }

        #[test]
        fn test_toggle_same_file_twice() {
            let mut manager = SelectionManager::new();
            let path = create_test_path(0);

            manager.toggle(path.clone());
            assert!(manager.is_selected(&path));

            manager.toggle(path.clone());
            assert!(!manager.is_selected(&path));
            assert!(manager.is_empty());
        }

        #[test]
        fn test_select_file() {
            let mut manager = SelectionManager::new();
            let path = create_test_path(0);

            manager.select(path.clone());

            assert!(manager.is_selected(&path));
            assert_eq!(manager.count(), 1);
        }

        #[test]
        fn test_select_same_file_multiple_times() {
            let mut manager = SelectionManager::new();
            let path = create_test_path(0);

            manager.select(path.clone());
            manager.select(path.clone());
            manager.select(path.clone());

            // HashSet prevents duplicates
            assert_eq!(manager.count(), 1);
        }

        #[test]
        fn test_deselect_file() {
            let mut manager = SelectionManager::new();
            let path = create_test_path(0);

            manager.select(path.clone());
            assert!(manager.is_selected(&path));

            manager.deselect(&path);
            assert!(!manager.is_selected(&path));
            assert!(manager.is_empty());
        }

        #[test]
        fn test_deselect_non_existent_file() {
            let mut manager = SelectionManager::new();
            let path = create_test_path(0);

            manager.deselect(&path);

            // Should not panic
            assert!(manager.is_empty());
        }

        #[test]
        fn test_is_selected_empty_manager() {
            let manager = SelectionManager::new();
            let path = create_test_path(0);

            assert!(!manager.is_selected(&path));
        }
    }

    // ============================================================================
    // Bulk Selection Tests
    // ============================================================================

    mod bulk_selection {
        use super::*;

        #[test]
        fn test_select_all_empty_list() {
            let mut manager = SelectionManager::new();
            let paths: Vec<PathBuf> = vec![];

            manager.select_all(paths);

            assert!(manager.is_empty());
        }

        #[test]
        fn test_select_all_multiple_files() {
            let mut manager = SelectionManager::new();
            let paths = vec![
                create_test_path(0),
                create_test_path(1),
                create_test_path(2),
            ];

            manager.select_all(paths.clone());

            assert_eq!(manager.count(), 3);
            for path in &paths {
                assert!(manager.is_selected(path));
            }
        }

        #[test]
        fn test_deselect_all() {
            let mut manager = SelectionManager::new();
            let paths = vec![
                create_test_path(0),
                create_test_path(1),
                create_test_path(2),
            ];

            manager.select_all(paths);
            assert_eq!(manager.count(), 3);

            manager.deselect_all();

            assert!(manager.is_empty());
            assert_eq!(manager.count(), 0);
        }

        #[test]
        fn test_deselect_all_when_empty() {
            let mut manager = SelectionManager::new();

            manager.deselect_all();

            // Should not panic
            assert!(manager.is_empty());
        }

        #[test]
        fn test_select_all_with_duplicates() {
            let mut manager = SelectionManager::new();
            let path = create_test_path(0);
            let paths = vec![
                path.clone(),
                path.clone(),
                create_test_path(1),
            ];

            manager.select_all(paths);

            // HashSet prevents duplicates
            assert_eq!(manager.count(), 2);
        }
    }

    // ============================================================================
    // Invert Selection Tests
    // ============================================================================

    mod invert_selection {
        use super::*;

        #[test]
        fn test_invert_empty_selection() {
            let mut manager = SelectionManager::new();
            let paths = vec![
                create_test_path(0),
                create_test_path(1),
                create_test_path(2),
            ];

            manager.invert(paths.clone());

            // All files should be selected
            assert_eq!(manager.count(), 3);
            for path in &paths {
                assert!(manager.is_selected(path));
            }
        }

        #[test]
        fn test_invert_partial_selection() {
            let mut manager = SelectionManager::new();
            let paths = vec![
                create_test_path(0),
                create_test_path(1),
                create_test_path(2),
                create_test_path(3),
            ];

            // Select first two
            manager.select(paths[0].clone());
            manager.select(paths[1].clone());

            manager.invert(paths.clone());

            // Last two should now be selected
            assert!(!manager.is_selected(&paths[0]));
            assert!(!manager.is_selected(&paths[1]));
            assert!(manager.is_selected(&paths[2]));
            assert!(manager.is_selected(&paths[3]));
        }

        #[test]
        fn test_invert_full_selection() {
            let mut manager = SelectionManager::new();
            let paths = vec![
                create_test_path(0),
                create_test_path(1),
                create_test_path(2),
            ];

            manager.select_all(paths.clone());
            assert_eq!(manager.count(), 3);

            manager.invert(paths);

            // All should be deselected
            assert!(manager.is_empty());
        }

        #[test]
        fn test_invert_twice() {
            let mut manager = SelectionManager::new();
            let paths = vec![
                create_test_path(0),
                create_test_path(1),
                create_test_path(2),
            ];

            manager.select(paths[0].clone());
            let initial_count = manager.count();

            manager.invert(paths.clone());
            manager.invert(paths);

            // Should return to original state
            assert_eq!(manager.count(), initial_count);
            assert!(manager.is_selected(&paths[0]));
            assert!(!manager.is_selected(&paths[1]));
            assert!(!manager.is_selected(&paths[2]));
        }

        #[test]
        fn test_invert_with_extra_files_not_in_list() {
            let mut manager = SelectionManager::new();
            let paths = vec![
                create_test_path(0),
                create_test_path(1),
            ];

            // Select a file that's not in the invert list
            manager.select(create_test_path(99));

            manager.invert(paths.clone());

            // The extra file should be deselected
            assert_eq!(manager.count(), 2);
            assert!(manager.is_selected(&paths[0]));
            assert!(manager.is_selected(&paths[1]));
        }
    }

    // ============================================================================
    // Get Selected Tests
    // ============================================================================

    mod get_selected {
        use super::*;

        #[test]
        fn test_get_selected_empty() {
            let manager = SelectionManager::new();

            let selected = manager.get_selected();

            assert!(selected.is_empty());
        }

        #[test]
        fn test_get_selected_multiple() {
            let mut manager = SelectionManager::new();
            let paths = vec![
                create_test_path(0),
                create_test_path(1),
                create_test_path(2),
            ];

            manager.select_all(paths.clone());

            let mut selected = manager.get_selected();
            selected.sort();

            let mut expected = paths.clone();
            expected.sort();

            assert_eq!(selected, expected);
        }

        #[test]
        fn test_get_selected_returns_new_vector() {
            let mut manager = SelectionManager::new();
            let path = create_test_path(0);

            manager.select(path.clone());

            let selected1 = manager.get_selected();
            let selected2 = manager.get_selected();

            // Should return separate vectors with same content
            assert_eq!(selected1, selected2);
        }

        #[test]
        fn test_modifying_returned_vector_doesnt_affect_manager() {
            let mut manager = SelectionManager::new();
            let path = create_test_path(0);

            manager.select(path.clone());

            let mut selected = manager.get_selected();
            selected.clear();

            // Manager should still have the file
            assert_eq!(manager.count(), 1);
        }
    }

    // ============================================================================
    // Last Selected Index Tests
    // ============================================================================

    mod last_selected {
        use super::*;

        #[test]
        fn test_set_last_selected() {
            let mut manager = SelectionManager::new();

            assert!(manager.get_last_selected().is_none());

            manager.set_last_selected(5);

            assert_eq!(manager.get_last_selected(), Some(5));
        }

        #[test]
        fn test_update_last_selected() {
            let mut manager = SelectionManager::new();

            manager.set_last_selected(3);
            manager.set_last_selected(7);

            assert_eq!(manager.get_last_selected(), Some(7));
        }

        #[test]
        fn test_last_selected_initially_none() {
            let manager = SelectionManager::new();

            assert!(manager.get_last_selected().is_none());
        }
    }

    // ============================================================================
    // Range Selection Tests
    // ============================================================================

    mod range_selection {
        use super::*;

        #[test]
        fn test_select_range_forward() {
            let mut manager = SelectionManager::new();
            let files = create_test_files(5);

            manager.select_range(1, 3, &files);

            assert_eq!(manager.count(), 3);
            assert!(manager.is_selected(&files[1].path));
            assert!(manager.is_selected(&files[2].path));
            assert!(manager.is_selected(&files[3].path));
        }

        #[test]
        fn test_select_range_backward() {
            let mut manager = SelectionManager::new();
            let files = create_test_files(5);

            manager.select_range(3, 1, &files);

            // Should select same range regardless of order
            assert_eq!(manager.count(), 3);
            assert!(manager.is_selected(&files[1].path));
            assert!(manager.is_selected(&files[2].path));
            assert!(manager.is_selected(&files[3].path));
        }

        #[test]
        fn test_select_range_single_item() {
            let mut manager = SelectionManager::new();
            let files = create_test_files(5);

            manager.select_range(2, 2, &files);

            assert_eq!(manager.count(), 1);
            assert!(manager.is_selected(&files[2].path));
        }

        #[test]
        fn test_select_range_from_zero() {
            let mut manager = SelectionManager::new();
            let files = create_test_files(5);

            manager.select_range(0, 2, &files);

            assert_eq!(manager.count(), 3);
            assert!(manager.is_selected(&files[0].path));
            assert!(manager.is_selected(&files[1].path));
            assert!(manager.is_selected(&files[2].path));
        }

        #[test]
        fn test_select_range_to_end() {
            let mut manager = SelectionManager::new();
            let files = create_test_files(5);

            manager.select_range(2, 4, &files);

            assert_eq!(manager.count(), 3);
            assert!(manager.is_selected(&files[2].path));
            assert!(manager.is_selected(&files[3].path));
            assert!(manager.is_selected(&files[4].path));
        }

        #[test]
        fn test_select_range_sets_last_selected() {
            let mut manager = SelectionManager::new();
            let files = create_test_files(5);

            manager.select_range(1, 3, &files);

            assert_eq!(manager.get_last_selected(), Some(3));
        }

        #[test]
        fn test_select_range_adds_to_existing() {
            let mut manager = SelectionManager::new();
            let files = create_test_files(10);

            // Select first range
            manager.select_range(0, 2, &files);

            // Select second range (non-contiguous)
            manager.select_range(5, 7, &files);

            assert_eq!(manager.count(), 6);
            assert!(manager.is_selected(&files[1].path));
            assert!(manager.is_selected(&files[6].path));
        }

        #[test]
        fn test_select_range_overlapping() {
            let mut manager = SelectionManager::new();
            let files = create_test_files(10);

            manager.select_range(0, 4, &files);
            manager.select_range(3, 7, &files);

            // HashSet prevents duplicates
            assert_eq!(manager.count(), 8);
        }

        #[test]
        fn test_select_range_out_of_bounds() {
            let mut manager = SelectionManager::new();
            let files = create_test_files(5);

            // Select beyond array bounds
            manager.select_range(3, 10, &files);

            // Should only select existing files
            assert_eq!(manager.count(), 2);
            assert!(manager.is_selected(&files[3].path));
            assert!(manager.is_selected(&files[4].path));
        }

        #[test]
        fn test_select_range_empty_file_list() {
            let mut manager = SelectionManager::new();
            let files: Vec<super::super::FileEntry> = vec![];

            manager.select_range(0, 5, &files);

            assert!(manager.is_empty());
        }
    }

    // ============================================================================
    // Edge Case Tests
    // ============================================================================

    mod edge_cases {
        use super::*;

        #[test]
        fn test_select_and_toggle_same_file() {
            let mut manager = SelectionManager::new();
            let path = create_test_path(0);

            manager.select(path.clone());
            assert!(manager.is_selected(&path));

            manager.toggle(path.clone());
            assert!(!manager.is_selected(&path));
        }

        #[test]
        fn test_count_accuracy() {
            let mut manager = SelectionManager::new();

            assert_eq!(manager.count(), 0);

            manager.select(create_test_path(0));
            assert_eq!(manager.count(), 1);

            manager.select(create_test_path(1));
            assert_eq!(manager.count(), 2);

            manager.deselect(&create_test_path(0));
            assert_eq!(manager.count(), 1);

            manager.deselect_all();
            assert_eq!(manager.count(), 0);
        }

        #[test]
        fn test_empty_vs_count_zero() {
            let mut manager = SelectionManager::new();

            assert!(manager.is_empty());
            assert_eq!(manager.count(), 0);

            manager.select(create_test_path(0));

            assert!(!manager.is_empty());
            assert_eq!(manager.count(), 1);
        }

        #[test]
        fn test_path_with_special_characters() {
            let mut manager = SelectionManager::new();
            let path = PathBuf::from("/test/file with spaces & special!.txt");

            manager.select(path.clone());

            assert!(manager.is_selected(&path));
            assert_eq!(manager.count(), 1);
        }

        #[test]
        fn test_path_absolute_and_relative() {
            let mut manager = SelectionManager::new();
            let absolute = PathBuf::from("/test/file.txt");
            let relative = PathBuf::from("test/file.txt");

            manager.select(absolute.clone());
            manager.select(relative.clone());

            // Different paths should both be selected
            assert_eq!(manager.count(), 2);
            assert!(manager.is_selected(&absolute));
            assert!(manager.is_selected(&relative));
        }

        #[test]
        fn test_concurrent_operations() {
            let mut manager = SelectionManager::new();
            let paths: Vec<PathBuf> =
                (0..10).map(|i| create_test_path(i)).collect();

            // Select all
            manager.select_all(paths.clone());
            assert_eq!(manager.count(), 10);

            // Invert
            manager.invert(paths.clone());
            assert_eq!(manager.count(), 0);

            // Select range
            let files = create_test_files(10);
            manager.select_range(2, 5, &files);
            assert_eq!(manager.count(), 4);

            // Deselect all
            manager.deselect_all();
            assert!(manager.is_empty());
        }
    }
}
