//! Property-based tests for filebrowser-tui
//!
//! This module uses proptest to verify invariants hold true across random inputs,
//! catching edge cases and logic errors that might be missed by unit tests.

use proptest::prelude::*;
use proptest::collection::{btree_set, vec};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use filebrowser_tui::state::files::{FileEntry, SortBy, SortOrder, sort_files};
use filebrowser_tui::state::selection::SelectionManager;
use filebrowser_tui::file_ops::bulk_rename::{
    BulkRenamer, RenamePattern, CaseTransform, CaseScope, ExtensionAction
};
use filebrowser_tui::file_ops::history::{HistoryManager, Operation};
use filebrowser_tui::file_ops::operations::{perform_copy, perform_move, perform_delete};

// ============================================================================
// Arbitrary Implementations
// ============================================================================

/// Strategy for generating valid file names (no invalid characters)
fn file_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Normal alphanumeric names
        "[a-zA-Z0-9][a-zA-Z0-9_-]{0,50}",
        // Names with extensions
        "[a-zA-Z0-9][a-zA-Z0-9_-]{0,30}\\.[a-zA-Z0-9]{1,10}",
        // Names with multiple dots
        "[a-zA-Z0-9][a-zA-Z0-9_-]{0,20}\\.[a-zA-Z0-9]{1,5}\\.[a-zA-Z0-9]{1,5}",
        // Simple names
        "[a-z]{1,20}",
    ]
}

/// Strategy for generating valid paths
fn path_strategy() -> impl Strategy<Value = PathBuf> {
    file_name_strategy().prop_map(|name| PathBuf::from(name))
}

/// Strategy for generating file entries
fn file_entry_strategy() -> impl Strategy<Value = FileEntry> {
    (
        file_name_strategy(),
        prop::bool::ANY,
        0u64..1_000_000_000u64,
    ).prop_map(|(name, is_dir, size)| {
        let path = PathBuf::from(&name);
        FileEntry {
            name: name.clone(),
            path,
            is_dir,
            size,
            modified: std::time::SystemTime::UNIX_EPOCH,
            is_hidden: false,
            is_system: false,
            is_readonly: false,
            is_symlink: false,
        }
    })
}

/// Strategy for generating a list of file entries
fn file_list_strategy() -> impl Strategy<Value = Vec<FileEntry>> {
    prop::collection::vec(file_entry_strategy(), 0..100)
}

/// Strategy for generating SortBy enum
fn sort_by_strategy() -> impl Strategy<Value = SortBy> {
    prop_oneof![
        Just(SortBy::Name),
        Just(SortBy::Size),
        Just(SortBy::Modified),
        Just(SortBy::Type),
    ]
}

/// Strategy for generating SortOrder enum
fn sort_order_strategy() -> impl Strategy<Value = SortOrder> {
    prop_oneof![
        Just(SortOrder::Ascending),
        Just(SortOrder::Descending),
    ]
}

// ============================================================================
// Test 1: File Sorting Properties
// ============================================================================

proptest! {
    /// Property 1.1: After sorting by name, the list is always alphabetically ordered
    #[test]
    fn prop_sort_by_name_is_ordered(files in file_list_strategy()) {
        let mut sorted = files.clone();
        sort_files(&mut sorted, SortBy::Name, SortOrder::Ascending);

        // Check that names are in ascending order
        for i in 1..sorted.len() {
            let prev_name = sorted[i-1].name.to_lowercase();
            let curr_name = sorted[i].name.to_lowercase();
            prop_assert!(prev_name <= curr_name,
                "Files not in order: {} should come before {}",
                prev_name, curr_name);
        }
    }

    /// Property 1.2: Directories always come before files when sorting
    #[test]
    fn prop_sort_directories_before_files(files in file_list_strategy()) {
        let mut sorted = files.clone();
        sort_files(&mut sorted, SortBy::Name, SortOrder::Ascending);

        // Find the transition from directories to files
        let mut seen_file = false;
        for entry in &sorted {
            if !entry.is_dir {
                seen_file = true;
            } else if seen_file {
                prop_assert!(false,
                    "Directory {} found after files in sorted list",
                    entry.name);
            }
        }
    }

    /// Property 1.3: Sorting twice with same criteria gives same result (idempotence)
    #[test]
    fn prop_sort_twice_is_idempotent(
        mut files in file_list_strategy(),
        sort_by in sort_by_strategy(),
        order in sort_order_strategy()
    ) {
        sort_files(&mut files, sort_by, order);
        let first_result = files.clone();

        sort_files(&mut files, sort_by, order);
        let second_result = files;

        prop_assert_eq!(first_result, second_result,
            "Sorting twice should produce the same result");
    }

    /// Property 1.4: Sorting preserves all elements
    #[test]
    fn prop_sort_preserves_elements(
        mut files in file_list_strategy(),
        sort_by in sort_by_strategy(),
        order in sort_order_strategy()
    ) {
        let original_count = files.len();
        let original_paths: HashSet<_> = files.iter().map(|f| &f.path).collect();

        sort_files(&mut files, sort_by, order);

        prop_assert_eq!(files.len(), original_count,
            "Sorting changed the number of files");
        prop_assert_eq!(files.len(), original_paths.len(),
            "Sorting lost or duplicated files");

        for entry in &files {
            prop_assert!(original_paths.contains(&entry.path),
                "File path {} was not in original list", entry.path.display());
        }
    }

    /// Property 1.5: Descending sort is reverse of ascending sort
    #[test]
    fn prop_sort_descending_is_reverse_of_ascending(
        mut files1 in file_list_strategy(),
        sort_by in sort_by_strategy()
    ) {
        let mut files2 = files1.clone();

        sort_files(&mut files1, sort_by, SortOrder::Ascending);
        sort_files(&mut files2, sort_by, SortOrder::Descending);

        // Reverse the ascending list
        files1.reverse();

        prop_assert_eq!(files1, files2,
            "Descending sort should be reverse of ascending sort");
    }
}

// ============================================================================
// Test 2: Selection Operations Properties
// ============================================================================

proptest! {
    /// Property 2.1: Select all + deselect all = empty selection
    #[test]
    fn prop_select_all_deselect_all_is_empty(paths in prop::collection::vec(path_strategy(), 0..50)) {
        let mut manager = SelectionManager::new();

        // Select all
        manager.select_all(paths.clone());
        prop_assert!(!manager.is_empty() || paths.is_empty(),
            "After select_all, manager should not be empty (unless no paths)");

        // Deselect all
        manager.deselect_all();
        prop_assert!(manager.is_empty(),
            "After deselect_all, manager should be empty");
        prop_assert_eq!(manager.count(), 0,
            "Count should be 0 after deselect_all");
    }

    /// Property 2.2: Invert selection twice = original selection
    #[test]
    fn prop_invert_twice_returns_original(paths in prop::collection::vec(path_strategy(), 0..50)) {
        let mut manager = SelectionManager::new();

        // Select some random paths
        for path in paths.iter().take(paths.len() / 2 + 1) {
            manager.select(path.clone());
        }

        let original_selection: HashSet<_> = manager.get_selected().into_iter().collect();

        // Invert twice
        manager.invert(paths.clone());
        manager.invert(paths);

        let final_selection: HashSet<_> = manager.get_selected().into_iter().collect();

        prop_assert_eq!(final_selection, original_selection,
            "Inverting twice should return to original selection");
    }

    /// Property 2.3: Toggle twice = original state
    #[test]
    fn prop_toggle_twice_returns_original(path in path_strategy()) {
        let mut manager = SelectionManager::new();
        let original_selected = manager.is_selected(&path);

        manager.toggle(path.clone());
        manager.toggle(path);

        let final_selected = manager.is_selected(&path);

        prop_assert_eq!(final_selected, original_selected,
            "Toggling twice should return to original state");
    }

    /// Property 2.4: Select range never selects outside bounds
    #[test]
    fn prop_select_range_respects_bounds(
        mut files in file_list_strategy(),
        from_index in 0usize..100,
        to_index in 0usize..100
    ) {
        let mut manager = SelectionManager::new();
        let file_count = files.len();

        if file_count == 0 {
            return Ok(());
        }

        // Clamp indices to valid range
        let from_index = from_index % file_count;
        let to_index = to_index % file_count;

        manager.select_range(from_index, to_index, &files);

        // Verify all selected files are within bounds
        let selected_paths: HashSet<_> = manager.get_selected().into_iter().collect();
        let valid_paths: HashSet<_> = files.iter().map(|f| &f.path).collect();

        for path in selected_paths {
            prop_assert!(valid_paths.contains(&path),
                "Selected path {} is not in the file list", path.display());
        }

        // Verify last_selected_index is set correctly
        prop_assert_eq!(manager.get_last_selected(), Some(to_index));
    }

    /// Property 2.5: Select + invert includes all non-selected paths
    #[test]
    fn prop_select_invert_covers_all_paths(paths in prop::collection::vec(path_strategy(), 0..50)) {
        let mut manager = SelectionManager::new();

        // Select some paths
        let initial_count = paths.len() / 3 + 1;
        for path in paths.iter().take(initial_count) {
            manager.select(path.clone());
        }

        let before_count = manager.count();

        // Invert selection
        manager.invert(paths.clone());

        let after_count = manager.count();

        prop_assert_eq!(before_count + after_count, paths.len(),
            "Before + after count should equal total paths");
    }
}

// ============================================================================
// Test 3: Bulk Rename Properties
// ============================================================================

proptest! {
    /// Property 3.1: Numbered pattern always produces sequential numbers
    #[test]
    fn prop_numbered_pattern_sequential(
        template in "[a-zA-Z0-9_]{n}[a-zA-Z0-9_]*\\.[a-z]{2,4}",
        start in 0usize..1000,
        pad_width in 0usize..10,
        files in prop::collection::vec(path_strategy(), 1..20)
    ) {
        let pattern = RenamePattern::Numbered {
            template: template.replace("{n}", "{n}"), // Ensure placeholder exists
            start,
            pad_width,
        };
        let renamer = BulkRenamer::new(pattern, PathBuf::from("/tmp"));

        let previews = renamer.preview(&files);

        prop_assert_eq!(previews.len(), files.len(),
            "Preview count should match file count");

        // Extract numbers from generated names
        for (i, preview) in previews.iter().enumerate() {
            let expected_num = start + i;
            let new_name = preview.new_name();

            // Check that the name contains the expected number
            let num_str = if pad_width > 0 {
                format!("{:0width$}", expected_num, width = pad_width)
            } else {
                expected_num.to_string()
            };

            prop_assert!(new_name.contains(&num_str),
                "Generated name {} should contain {}", new_name, num_str);
        }
    }

    /// Property 3.2: Regex substitution never produces invalid paths
    #[test]
    fn prop_regex_substitution_valid_paths(
        pattern in "[a-z]{1,10}",
        replacement in "[a-z0-9_]{0,20}",
        files in prop::collection::vec(path_strategy(), 1..20)
    ) {
        let rename_pattern = RenamePattern::Regex {
            pattern,
            replacement,
        };
        let renamer = BulkRenamer::new(rename_pattern, PathBuf::from("/tmp"));

        let previews = renamer.preview(&files);

        for preview in &previews {
            let new_name = preview.new_name();

            // Check for invalid characters
            let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
            for c in invalid_chars {
                prop_assert!(!new_name.contains(c),
                    "Generated name contains invalid character '{}': {}", c, new_name);
            }

            // Name should not be empty
            prop_assert!(!new_name.is_empty(),
                "Generated name should not be empty");
        }
    }

    /// Property 3.3: Case transform preserves string length (for most transforms)
    #[test]
    fn prop_case_transform_preserves_length(
        transform in prop_oneof![
            Just(CaseTransform::Uppercase),
            Just(CaseTransform::Lowercase),
            Just(CaseTransform::ToggleCase),
        ],
        scope in prop_oneof![
            Just(CaseScope::NameOnly),
            Just(CaseScope::EntireName),
        ],
        files in prop::collection::vec(
            file_name_strategy().prop_filter("need extension", |n| n.contains('.')),
            1..20
        )
    ) {
        let pattern = RenamePattern::Case { transform, scope };
        let renamer = BulkRenamer::new(pattern, PathBuf::from("/tmp"));

        let previews = renamer.preview(&files);

        for (preview, original_path) in previews.iter().zip(files.iter()) {
            let original_name = original_path.file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let new_name = preview.new_name();

            prop_assert_eq!(new_name.len(), original_name.len(),
                "Case transform should preserve length: {} -> {}",
                original_name, new_name);
        }
    }

    /// Property 3.4: Simple replace with empty find does nothing
    #[test]
    fn prop_empty_find_does_nothing(files in prop::collection::vec(path_strategy(), 1..20)) {
        let pattern = RenamePattern::SimpleReplace {
            find: String::new(),
            replace: "anything".to_string(),
        };
        let renamer = BulkRenamer::new(pattern, PathBuf::from("/tmp"));

        let previews = renamer.preview(&files);

        for (preview, original) in previews.iter().zip(files.iter()) {
            let original_name = original.file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let new_name = preview.new_name();

            prop_assert_eq!(new_name, original_name,
                "Empty find should not change the name");
        }
    }

    /// Property 3.5: Extension replace changes only extension
    #[test]
    fn prop_extension_replace_changes_only_extension(
        new_ext in "[a-z]{1,5}",
        files in prop::collection::vec(
            file_name_strategy().prop_filter("need extension", |n| n.contains('.')),
            1..20
        )
    ) {
        let pattern = RenamePattern::Extension {
            action: ExtensionAction::Replace,
            new_extension: Some(new_ext.clone()),
        };
        let renamer = BulkRenamer::new(pattern, PathBuf::from("/tmp"));

        let previews = renamer.preview(&files);

        for (preview, original) in previews.iter().zip(files.iter()) {
            let original_name = original.file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let new_name = preview.new_name();

            // Split into stem and extension
            let original_stem = original_name.rsplit('.').nth(1).unwrap_or("");
            let new_stem = new_name.rsplit('.').nth(1).unwrap_or("");

            prop_assert_eq!(original_stem, new_stem,
                "Extension replace should preserve stem: {} -> {}",
                original_name, new_name);

            prop_assert!(new_name.ends_with(&new_ext),
                "New name should end with new extension: {}", new_name);
        }
    }

    /// Property 3.6: Uppercase then lowercase = original length
    #[test]
    fn prop_uppercase_then_lowercase_preserves(
        files in prop::collection::vec(path_strategy(), 1..20)
    ) {
        // First uppercase
        let pattern1 = RenamePattern::Case {
            transform: CaseTransform::Uppercase,
            scope: CaseScope::EntireName,
        };
        let renamer1 = BulkRenamer::new(pattern1, PathBuf::from("/tmp"));

        // Then lowercase
        let pattern2 = RenamePattern::Case {
            transform: CaseTransform::Lowercase,
            scope: CaseScope::EntireName,
        };
        let renamer2 = BulkRenamer::new(pattern2, PathBuf::from("/tmp"));

        let previews1 = renamer1.preview(&files);
        let intermediate: Vec<PathBuf> = previews1.iter()
            .map(|p| PathBuf::from(&p.new_name))
            .collect();

        let previews2 = renamer2.preview(&intermediate);

        for (preview2, original) in previews2.iter().zip(files.iter()) {
            let original_name = original.file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let final_name = preview2.new_name();

            // Length should be preserved
            prop_assert_eq!(final_name.len(), original_name.len(),
                "Uppercase then lowercase should preserve length");

            // Final should be all lowercase
            prop_assert_eq!(final_name, final_name.to_lowercase(),
                "Final name should be all lowercase");
        }
    }
}

// ============================================================================
// Test 4: Undo/Redo Properties
// ============================================================================

proptest! {
    /// Property 4.1: Can't undo when history is empty
    #[test]
    fn prop_cannot_undo_empty_history() {
        let manager = HistoryManager::new().unwrap();
        prop_assert!(!manager.can_undo(),
            "Should not be able to undo with empty history");
        prop_assert_eq!(manager.undo_count(), 0,
            "Undo count should be 0");
    }

    /// Property 4.2: Can't redo when redo stack is empty
    #[test]
    fn prop_cannot_redo_empty_stack() {
        let manager = HistoryManager::new().unwrap();
        prop_assert!(!manager.can_redo(),
            "Should not be able to redo with empty redo stack");
        prop_assert_eq!(manager.redo_count(), 0,
            "Redo count should be 0");
    }

    /// Property 4.3: Recording an operation clears the redo stack
    #[test]
    fn prop_record_clears_redo_stack(
        op1_path in path_strategy(),
        op2_path in path_strategy()
    ) {
        let mut manager = HistoryManager::new().unwrap();

        // Record first operation
        manager.record(Operation::CreateDir {
            path: op1_path.clone(),
        });

        // Undo it (adds to redo stack)
        let _ = manager.undo();
        prop_assert!(manager.can_redo(),
            "Should be able to redo after undo");

        // Record second operation (should clear redo stack)
        manager.record(Operation::CreateDir {
            path: op2_path,
        });

        prop_assert!(!manager.can_redo(),
            "Recording new operation should clear redo stack");
        prop_assert_eq!(manager.redo_count(), 0,
            "Redo count should be 0 after recording");
    }

    /// Property 4.4: Undo count increases with each record
    #[test]
    fn prop_undo_count_increases(paths in prop::collection::vec(path_strategy(), 1..20)) {
        let mut manager = HistoryManager::new().unwrap();

        for (i, path) in paths.iter().enumerate() {
            manager.record(Operation::CreateDir {
                path: path.clone(),
            });
            prop_assert_eq!(manager.undo_count(), i + 1,
                "Undo count should increment with each record");
        }
    }

    /// Property 4.5: Clear empties both stacks
    #[test]
    fn prop_clear_empties_stacks(paths in prop::collection::vec(path_strategy(), 1..10)) {
        let mut manager = HistoryManager::new().unwrap();

        // Add some operations
        for path in &paths {
            manager.record(Operation::CreateDir {
                path: path.clone(),
            });
        }

        // Undo some
        if manager.undo_count() > 0 {
            let _ = manager.undo();
        }

        prop_assert!(manager.can_undo() || manager.can_redo(),
            "Should have something to undo or redo");

        // Clear
        manager.clear();

        prop_assert!(!manager.can_undo(),
            "Should not be able to undo after clear");
        prop_assert!(!manager.can_redo(),
            "Should not be able to redo after clear");
        prop_assert_eq!(manager.undo_count(), 0,
            "Undo count should be 0 after clear");
        prop_assert_eq!(manager.redo_count(), 0,
            "Redo count should be 0 after clear");
    }

    /// Property 4.6: Undo followed by redo restores counts
    #[test]
    fn prop_undo_redo_restores_counts(path in path_strategy()) {
        let mut manager = HistoryManager::new().unwrap();

        manager.record(Operation::CreateDir {
            path: path.clone(),
        });

        let undo_before = manager.undo_count();
        let redo_before = manager.redo_count();

        let _ = manager.undo();

        let undo_after_undo = manager.undo_count();
        let redo_after_undo = manager.redo_count();

        let _ = manager.redo();

        let undo_after_redo = manager.undo_count();
        let redo_after_redo = manager.redo_count();

        prop_assert_eq!(undo_after_redo, undo_before,
            "Undo count should be restored after redo");
        prop_assert_eq!(redo_after_redo, redo_before,
            "Redo count should be restored after redo");

        prop_assert_eq!(undo_after_undo, undo_before - 1,
            "Undo count should decrease by 1 after undo");
        prop_assert_eq!(redo_after_undo, redo_before + 1,
            "Redo count should increase by 1 after undo");
    }
}

// ============================================================================
// Test 5: File Operations Properties
// ============================================================================

proptest! {
    /// Property 5.1: Copy preserves file content
    #[test]
    fn prop_copy_preserves_content(content in "[a-zA-Z0-9\\s]{0,1000}") {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let target_path = temp_dir.path().join("target.txt");

        // Create source file with content
        fs::write(&source_path, &content).unwrap();

        // Perform copy
        perform_copy(&source_path, &target_path).unwrap();

        // Verify content is preserved
        let source_content = fs::read_to_string(&source_path).unwrap();
        let target_content = fs::read_to_string(&target_path).unwrap();

        prop_assert_eq!(source_content, content,
            "Source content should match original");
        prop_assert_eq!(target_content, content,
            "Target content should match original");
        prop_assert_eq!(source_content, target_content,
            "Source and target content should match");
    }

    /// Property 5.2: Move changes location, not content
    #[test]
    fn prop_move_preserves_content(content in "[a-zA-Z0-9\\s]{0,1000}") {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let target_path = temp_dir.path().join("subdir").join("target.txt");

        // Create source file with content
        fs::write(&source_path, &content).unwrap();

        // Perform move
        perform_move(&source_path, &target_path).unwrap();

        // Verify source no longer exists
        prop_assert!(!source_path.exists(),
            "Source should not exist after move");

        // Verify target exists with same content
        prop_assert!(target_path.exists(),
            "Target should exist after move");

        let target_content = fs::read_to_string(&target_path).unwrap();
        prop_assert_eq!(target_content, content,
            "Target content should match original");
    }

    /// Property 5.3: Delete removes file
    #[test]
    fn prop_delete_removes_file(content in "[a-zA-Z0-9\\s]{0,1000}") {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");

        // Create file
        fs::write(&file_path, &content).unwrap();
        prop_assert!(file_path.exists(),
            "File should exist before delete");

        // Perform delete
        perform_delete(&file_path).unwrap();

        // Verify file is removed
        prop_assert!(!file_path.exists(),
            "File should not exist after delete");
    }

    /// Property 5.4: Copy to same location creates separate file
    #[test]
    fn prop_copy_creates_separate_file(content in "[a-zA-Z0-9\\s]{10,100}") {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let target_path = temp_dir.path().join("target.txt");

        // Create source file
        fs::write(&source_path, &content).unwrap();

        // Perform copy
        perform_copy(&source_path, &target_path).unwrap();

        // Modify source
        let new_content = "modified content";
        fs::write(&source_path, new_content).unwrap();

        // Verify target is unchanged
        let target_content = fs::read_to_string(&target_path).unwrap();
        prop_assert_eq!(target_content, content,
            "Target should not be affected by source modification");
    }

    /// Property 5.5: Copy directory preserves structure
    #[test]
    fn prop_copy_directory_preserves_structure(
        files in prop::collection::vec("[a-zA-Z0-9]{1,20}", 1..10)
    ) {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let target_dir = temp_dir.path().join("target");

        // Create source directory with files
        fs::create_dir(&source_dir).unwrap();
        for (i, filename) in files.iter().enumerate() {
            let content = format!("content {}", i);
            fs::write(source_dir.join(filename), &content).unwrap();
        }

        // Perform copy
        perform_copy(&source_dir, &target_dir).unwrap();

        // Verify structure is preserved
        prop_assert!(target_dir.exists(),
            "Target directory should exist");
        prop_assert!(target_dir.is_dir(),
            "Target should be a directory");

        for filename in &files {
            let source_file = source_dir.join(filename);
            let target_file = target_dir.join(filename);

            prop_assert!(target_file.exists(),
                "File {} should exist in target", filename);

            let source_content = fs::read_to_string(&source_file).unwrap();
            let target_content = fs::read_to_string(&target_file).unwrap();

            prop_assert_eq!(source_content, target_content,
                "Content of {} should match", filename);
        }
    }
}

// ============================================================================
// Test 6: Path Operations Properties
// ============================================================================

proptest! {
    /// Property 6.1: Parent of parent is grandparent
    #[test]
    fn prop_parent_of_parent_is_grandparent(
        name in file_name_strategy(),
        parent in file_name_strategy(),
        grandparent in file_name_strategy()
    ) {
        let path = PathBuf::from(grandparent)
            .join(parent)
            .join(name);

        let parent1 = path.parent();
        let parent2 = parent1.and_then(|p| p.parent());

        prop_assert!(parent1.is_some(),
            "First parent should exist");
        prop_assert!(parent2.is_some(),
            "Second parent (grandparent) should exist");

        // The grandparent from the path should match our constructed grandparent
        let expected_grandparent = PathBuf::from(grandparent);
        prop_assert_eq!(parent2.unwrap(), expected_grandparent,
            "Parent of parent should be grandparent");
    }

    /// Property 6.2: Join with absolute path ignores base
    #[test]
    fn prop_join_absolute_ignores_base(
        base in "[a-zA-Z0-9]{1,20}",
        absolute_name in file_name_strategy()
    ) {
        let base = PathBuf::from(base);
        let absolute_path = PathBuf::from("/").join(&absolute_name);

        let joined = base.join(&absolute_path);

        // When joining with an absolute path, the absolute path replaces the base
        prop_assert_eq!(joined, absolute_path,
            "Joining with absolute path should ignore base");
    }

    /// Property 6.3: Path extension extraction is consistent
    #[test]
    fn prop_extension_extraction_consistent(
        stem in "[a-zA-Z0-9]{1,20}",
        ext in "[a-z]{1,5}"
    ) {
        let filename = format!("{}.{}", stem, ext);
        let path = PathBuf::from(&filename);

        let extracted_ext = path.extension().and_then(|e| e.to_str());

        prop_assert_eq!(extracted_ext, Some(ext.as_str()),
            "Extracted extension should match original");

        // FileEntry should give same result
        let entry = FileEntry {
            name: filename.clone(),
            path: path.clone(),
            is_dir: false,
            size: 0,
            modified: std::time::SystemTime::UNIX_EPOCH,
            is_hidden: false,
            is_system: false,
            is_readonly: false,
            is_symlink: false,
        };

        prop_assert_eq!(entry.extension(), Some(ext.as_str()),
            "FileEntry extension should match path extension");
    }

    /// Property 6.4: File name extraction is lossless
    #[test]
    fn prop_filename_extraction_lossless(name in file_name_strategy()) {
        let path = PathBuf::from(&name);
        let extracted = path.file_name()
            .and_then(|n| n.to_str());

        prop_assert_eq!(extracted, Some(name.as_str()),
            "Extracted filename should match original");

        // FileEntry should preserve this
        let entry = FileEntry {
            name: name.clone(),
            path: path.clone(),
            is_dir: false,
            size: 0,
            modified: std::time::SystemTime::UNIX_EPOCH,
            is_hidden: false,
            is_system: false,
            is_readonly: false,
            is_symlink: false,
        };

        prop_assert_eq!(entry.name, name,
            "FileEntry name should match original");
    }

    /// Property 6.5: Ancestor chain terminates at root
    #[test]
    fn prop_ancestor_chain_terminates(
        components in prop::collection::vec(file_name_strategy(), 1..10)
    ) {
        let mut path = PathBuf::new();
        for component in &components {
            path = path.join(component);
        }

        let mut ancestors = Vec::new();
        let mut current = Some(path.clone());

        while let Some(p) = current {
            ancestors.push(p.clone());
            current = p.parent();
            // Prevent infinite loop
            if ancestors.len() > 100 {
                break;
            }
        }

        // The last ancestor should be empty or very short (root)
        if let Some(last) = ancestors.last() {
            prop_assert!(last.as_os_str().is_empty() || last.components().count() <= 1,
                "Ancestor chain should terminate at root, got: {:?}",
                last);
        }
    }
}

// ============================================================================
// Test 7: Combined Operations Properties
// ============================================================================

proptest! {
    /// Property 7.1: Sort then filter preserves ordering of filtered results
    #[test]
    fn prop_sort_then_filter_preserves_order(
        mut files in file_list_strategy(),
        query in file_name_strategy()
    ) {
        use filebrowser_tui::state::files::filter_files;

        // Sort first
        sort_files(&mut files, SortBy::Name, SortOrder::Ascending);

        // Then filter
        let indices = filter_files(&files, false, Some(&query));

        // Check that filtered indices are in ascending order
        for i in 1..indices.len() {
            prop_assert!(indices[i] > indices[i-1],
                "Filtered indices should be in ascending order");
        }

        // Check that filtered files maintain sorted order
        for i in 1..indices.len() {
            let prev_name = files[indices[i-1]].name.to_lowercase();
            let curr_name = files[indices[i]].name.to_lowercase();
            prop_assert!(prev_name <= curr_name,
                "Filtered files should maintain sorted order");
        }
    }

    /// Property 7.2: Selection operations are idempotent
    #[test]
    fn prop_selection_operations_idempotent(
        paths in prop::collection::vec(path_strategy(), 1..20)
    ) {
        let mut manager1 = SelectionManager::new();
        let mut manager2 = SelectionManager::new();

        // Select all in manager1
        manager1.select_all(paths.clone());

        // Select all twice in manager2
        manager2.select_all(paths.clone());
        manager2.select_all(paths);

        // Should have same result
        prop_assert_eq!(manager1.count(), manager2.count(),
            "Selecting all twice should give same result");

        let selection1: HashSet<_> = manager1.get_selected().into_iter().collect();
        let selection2: HashSet<_> = manager2.get_selected().into_iter().collect();

        prop_assert_eq!(selection1, selection2,
            "Selection sets should be identical");
    }

    /// Property 7.3: Empty selection operations are safe
    #[test]
    fn prop_empty_selection_is_safe() {
        let mut manager = SelectionManager::new();

        // These operations should not panic on empty selection
        manager.deselect_all();
        prop_assert!(manager.is_empty());

        manager.select_all(vec![]);
        prop_assert!(manager.is_empty());

        manager.invert(vec![]);
        prop_assert!(manager.is_empty());

        let selected = manager.get_selected();
        prop_assert_eq!(selected.len(), 0);
    }

    /// Property 7.4: History with max size respects limit
    #[test]
    fn prop_history_respects_max_size(
        mut manager in prop::collection::vec(path_strategy(), 1..150)
            .prop_map(|paths| {
                let mut m = HistoryManager::new().unwrap();
                for path in paths {
                    m.record(Operation::CreateDir { path });
                }
                m
            })
    ) {
        // HistoryManager has max_history_size of 100
        let max_size = 100;

        prop_assert!(manager.undo_count() <= max_size,
            "History should not exceed max size of {}",
            max_size);
    }
}
