//! End-to-end tests for the TUI file browser
//!
//! These tests use the PTY-based testing framework to simulate real user interactions.
//!
//! Run with: cargo test --test e2e_tests

use std::path::PathBuf;
use std::time::Duration;

// Include the testing modules as part of this test crate
mod tui_tester;
mod fixtures;
mod common;

// Test helper to create a tester instance
fn create_tester() -> anyhow::Result<(tui_tester::TuiTester, fixtures::TestFixture)> {
    let fixture = fixtures::TestFixture::new()?;
    let tester = tui_tester::TuiTester::new(fixture.path.clone())?
        .with_terminal_size(40, 120)
        .with_verbose(std::env::var("TUI_TEST_VERBOSE").is_ok());

    Ok((tester, fixture))
}

// ============================================================================
// Navigation Tests
// ============================================================================

    #[test]
    #[ignore] // Requires compiled binary
    fn test_navigate_directories() {
        let (mut tester, fixture) = create_tester().unwrap();

        // Create test structure
        fixture.create_standard_structure().unwrap();

        // Launch the app
        tester.launch().unwrap();

        // Wait for initial screen
        tester.wait_for("dir1").unwrap();
        tester.assert_contains("dir2").unwrap();
        tester.assert_contains("root_file.txt").unwrap();

        // Navigate into dir1
        tester.send_keys("dir1").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();
        tester.wait_for("file1.txt").unwrap();
        tester.assert_contains("file2.txt").unwrap();

        // Navigate back up
        tester.send_special_key(tui_tester::SpecialKey::Escape).unwrap();
        tester.wait_for("root_file.txt").unwrap();

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_nested_navigation() {
        let (mut tester, fixture) = create_tester().unwrap();

        // Create nested structure
        fixture.create_nested_structure().unwrap();

        tester.launch().unwrap();

        // Navigate deep into hierarchy
        tester.wait_for("deep").unwrap();

        tester.send_keys("deep").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        tester.wait_for("level1").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Right).unwrap();

        // Continue navigation...
        for _ in 0..4 {
            tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();
        }

        tester.wait_for("deep_file.txt").unwrap();
        tester.assert_contains("level4").unwrap();

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_scroll_large_directory() {
        let (mut tester, fixture) = create_tester().unwrap();

        // Create large structure
        fixture.create_large_structure().unwrap();

        tester.launch().unwrap();

        tester.wait_for("many_files").unwrap();

        // Enter directory
        tester.send_keys("many_files").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Scroll down
        for _ in 0..10 {
            tester.send_special_key(tui_tester::SpecialKey::Down).unwrap();
        }

        // Should see files that were initially off-screen
        tester.wait_for("file_010.txt").unwrap();

        // Page down
        tester.send_special_key(tui_tester::SpecialKey::PageDown).unwrap();
        tester.wait_for("file_030.txt").unwrap();

        tester.quit(true).unwrap();
    }

// ============================================================================
// File Operations Tests
// ============================================================================

    #[test]
    #[ignore]
    fn test_create_file() {
        let (mut tester, fixture) = create_tester().unwrap();

        tester.launch().unwrap();

        // Enter command mode to create file
        tester.send_special_key(tui_tester::SpecialKey::Escape).unwrap();
        tester.send_keys(":touch new_file.txt").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Verify file was created
        common::assert_eventually(
            || fixture.exists("new_file.txt"),
            Duration::from_secs(2),
            "File should be created"
        );

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_create_directory() {
        let (mut tester, fixture) = create_tester().unwrap();

        tester.launch().unwrap();

        // Create directory via command
        tester.send_special_key(tui_tester::SpecialKey::Escape).unwrap();
        tester.send_keys(":mkdir new_dir").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Verify directory was created
        common::assert_eventually(
            || fixture.exists("new_dir"),
            Duration::from_secs(2),
            "Directory should be created"
        );

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_delete_file() {
        let (mut tester, fixture) = create_tester().unwrap();

        // Create file to delete
        fixture.create_file("to_delete.txt", "Delete me").unwrap();

        tester.launch().unwrap();

        // Navigate to file
        tester.wait_for("to_delete.txt").unwrap();

        // Delete the file (assuming 'd' is delete key)
        tester.send_keys("d").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap(); // Confirm

        // Verify file was deleted
        common::assert_eventually(
            || !fixture.exists("to_delete.txt"),
            Duration::from_secs(2),
            "File should be deleted"
        );

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_rename_file() {
        let (mut tester, fixture) = create_tester().unwrap();

        // Create file to rename
        fixture.create_file("old_name.txt", "Content").unwrap();

        tester.launch().unwrap();

        // Navigate to file and rename
        tester.wait_for("old_name.txt").unwrap();

        // Rename command (assuming 'r' is rename key)
        tester.send_keys("r").unwrap();
        tester.send_keys("new_name.txt").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Verify rename
        common::assert_eventually(
            || !fixture.exists("old_name.txt") && fixture.exists("new_name.txt"),
            Duration::from_secs(2),
            "File should be renamed"
        );

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_copy_file() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_operations_structure().unwrap();

        tester.launch().unwrap();

        // Navigate to file to copy
        tester.wait_for("to_copy.txt").unwrap();

        // Copy file (assuming F5 is copy)
        tester.send_special_key(tui_tester::SpecialKey::F(5)).unwrap();

        // Navigate to destination
        tester.send_keys("copy_dest").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Paste
        tester.send_keys("p").unwrap();

        // Verify copy
        common::assert_eventually(
            || fixture.exists("copy_dest/to_copy.txt"),
            Duration::from_secs(2),
            "File should be copied"
        );

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_move_file() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_operations_structure().unwrap();

        tester.launch().unwrap();

        // Navigate to file to move
        tester.wait_for("to_move.txt").unwrap();

        // Cut file
        tester.send_keys("x").unwrap();

        // Navigate to destination
        tester.send_keys("move_dest").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Paste (move)
        tester.send_keys("p").unwrap();

        // Verify move
        common::assert_eventually(
            || !fixture.exists("to_move.txt") && fixture.exists("move_dest/to_move.txt"),
            Duration::from_secs(2),
            "File should be moved"
        );

        tester.quit(true).unwrap();
    }

// ============================================================================
// Search and Filter Tests
// ============================================================================

    #[test]
    #[ignore]
    fn test_search_files() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_search_structure().unwrap();

        tester.launch().unwrap();

        // Enter search mode
        tester.send_special_key(tui_tester::SpecialKey::Escape).unwrap();
        tester.send_keys("/").unwrap();

        // Type search term
        tester.send_keys("doc").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Should filter to show only matching files
        tester.wait_for("doc1.txt").unwrap();
        tester.assert_contains("doc2.txt").unwrap();
        tester.assert_not_contains("readme.md").unwrap();

        // Clear search
        tester.send_special_key(tui_tester::SpecialKey::Escape).unwrap();

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_content_search() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_search_structure().unwrap();

        tester.launch().unwrap();

        // Enter content search mode (Ctrl+G)
        tester.send_special_key(tui_tester::SpecialKey::CtrlG).unwrap();

        // Type search term
        tester.send_keys("Search term").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Should find files containing the search term
        tester.wait_for("doc1.txt").unwrap();

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_filter_by_extension() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_search_structure().unwrap();

        tester.launch().unwrap();

        // Filter by .txt extension
        tester.send_special_key(tui_tester::SpecialKey::Escape).unwrap();
        tester.send_keys(":filter *.txt").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Should show only .txt files
        tester.assert_contains("doc1.txt").unwrap();
        tester.assert_not_contains("notes.md").unwrap();
        tester.assert_not_contains("data.json").unwrap();

        tester.quit(true).unwrap();
    }

// ============================================================================
// Tab Management Tests
// ============================================================================

    #[test]
    #[ignore]
    fn test_create_new_tab() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_tab_structure().unwrap();

        tester.launch().unwrap();

        // Create new tab (Ctrl+T)
        tester.send_special_key(tui_tester::SpecialKey::CtrlT).unwrap();

        // Should see tab indicator
        tester.wait_for("Tab 2").unwrap();

        // Navigate in new tab
        tester.send_keys("tab_dir_1").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        tester.wait_for("file1.txt").unwrap();

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_switch_tabs() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_tab_structure().unwrap();

        tester.launch().unwrap();

        // Create multiple tabs
        tester.send_special_key(tui_tester::SpecialKey::CtrlT).unwrap();
        tester.wait_for("Tab 2").unwrap();

        tester.send_special_key(tui_tester::SpecialKey::CtrlT).unwrap();
        tester.wait_for("Tab 3").unwrap();

        // Switch tabs using Tab key
        tester.send_special_key(tui_tester::SpecialKey::Tab).unwrap();
        // Should switch back to previous tab

        // Or use Ctrl+number
        tester.send_special_key(tui_tester::SpecialKey::CtrlT).unwrap(); // This is a placeholder
        tester.send_keys("1").unwrap();

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_close_tab() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_tab_structure().unwrap();

        tester.launch().unwrap();

        // Create and close tab
        tester.send_special_key(tui_tester::SpecialKey::CtrlT).unwrap();
        tester.wait_for("Tab 2").unwrap();

        tester.send_special_key(tui_tester::SpecialKey::CtrlW).unwrap();

        // Should return to single tab
        tester.wait_for_remove("Tab 2").unwrap();

        tester.quit(true).unwrap();
    }

// ============================================================================
// Split View Tests
// ============================================================================

    #[test]
    #[ignore]
    fn test_toggle_split_view() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_split_structure().unwrap();

        tester.launch().unwrap();

        // Toggle split view (Ctrl+P)
        tester.send_special_key(tui_tester::SpecialKey::CtrlP).unwrap();

        // Should see split panes
        tester.wait_for("left_dir").unwrap();
        tester.assert_contains("right_dir").unwrap();

        // Toggle off
        tester.send_special_key(tui_tester::SpecialKey::CtrlP).unwrap();

        tester.wait_for_remove("right_dir").unwrap();

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_switch_active_pane() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_split_structure().unwrap();

        tester.launch().unwrap();

        // Enable split view
        tester.send_special_key(tui_tester::SpecialKey::CtrlP).unwrap();
        tester.wait_for("left_dir").unwrap();

        // Switch to right pane (Tab in split view)
        tester.send_special_key(tui_tester::SpecialKey::Tab).unwrap();

        // Navigate in right pane
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();
        tester.wait_for("right1.txt").unwrap();

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_copy_between_panes() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_split_structure().unwrap();

        tester.launch().unwrap();

        // Enable split view
        tester.send_special_key(tui_tester::SpecialKey::CtrlP).unwrap();

        // In left pane, select file
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();
        tester.wait_for("left1.txt").unwrap();

        // Copy to other pane (F5 in split view)
        tester.send_special_key(tui_tester::SpecialKey::F(5)).unwrap();

        // Switch to right pane
        tester.send_special_key(tui_tester::SpecialKey::Tab).unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Verify file was copied
        tester.wait_for("left1.txt").unwrap();

        tester.quit(true).unwrap();
    }

// ============================================================================
// Undo/Redo Tests
// ============================================================================

    #[test]
    #[ignore]
    fn test_undo_delete() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_undo_structure().unwrap();

        tester.launch().unwrap();

        // Navigate to file and delete it
        tester.send_keys("undo_test").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();
        tester.wait_for("original.txt").unwrap();

        tester.send_keys("d").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Verify deleted
        common::assert_eventually(
            || !fixture.exists("undo_test/original.txt"),
            Duration::from_secs(2),
            "File should be deleted"
        );

        // Undo (Ctrl+U)
        tester.send_special_key(tui_tester::SpecialKey::CtrlU).unwrap();

        // Verify restored
        common::assert_eventually(
            || fixture.exists("undo_test/original.txt"),
            Duration::from_secs(2),
            "File should be restored"
        );

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_redo_after_undo() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_undo_structure().unwrap();

        tester.launch().unwrap();

        // Perform operation, undo, then redo
        tester.send_keys("undo_test").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();
        tester.wait_for("original.txt").unwrap();

        // Rename
        tester.send_keys("r").unwrap();
        tester.send_keys("renamed.txt").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Undo
        tester.send_special_key(tui_tester::SpecialKey::CtrlU).unwrap();

        // Redo (Ctrl+R)
        tester.send_special_key(tui_tester::SpecialKey::CtrlR).unwrap();

        // Verify rename is back
        common::assert_eventually(
            || !fixture.exists("undo_test/original.txt") && fixture.exists("undo_test/renamed.txt"),
            Duration::from_secs(2),
            "File should be renamed again"
        );

        tester.quit(true).unwrap();
    }

// ============================================================================
// Bulk Rename Tests
// ============================================================================

    #[test]
    #[ignore]
    fn test_bulk_rename() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_operations_structure().unwrap();

        tester.launch().unwrap();

        // Navigate to bulk rename directory
        tester.send_keys("bulk_rename_test").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Enter bulk rename mode
        tester.send_keys("R").unwrap(); // Assuming 'R' is bulk rename key

        // Wait for bulk rename UI
        tester.wait_for("Bulk Rename").unwrap();

        // Select all files
        tester.send_keys("a").unwrap(); // Assuming 'a' is select all

        // Apply rename pattern
        tester.send_keys(":s/photo_/image_/").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Execute rename
        tester.send_keys("w").unwrap(); // Assuming 'w' is write

        // Verify renamed
        common::assert_eventually(
            || !fixture.exists("bulk_rename_test/photo_01.jpg") && fixture.exists("bulk_rename_test/image_01.jpg"),
            Duration::from_secs(2),
            "Files should be renamed"
        );

        tester.quit(true).unwrap();
    }

// ============================================================================
// Error Handling Tests
// ============================================================================

    #[test]
    #[ignore]
    fn test_delete_readonly_file_error() {
        let (mut tester, fixture) = create_tester().unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fixture.create_file("readonly.txt", "Read only").unwrap();
            let path = fixture.resolve_path("readonly.txt");
            let mut perms = std::fs::metadata(&path)?.permissions();
            perms.set_mode(0o444);
            std::fs::set_permissions(&path, perms)?;
        }

        tester.launch().unwrap();

        tester.wait_for("readonly.txt").unwrap();

        // Try to delete
        tester.send_keys("d").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Should see error message
        tester.wait_for("Error").unwrap();
        tester.assert_contains("Permission denied").unwrap();

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_invalid_directory() {
        let (mut tester, fixture) = create_tester().unwrap();

        tester.launch().unwrap();

        // Try to navigate to non-existent directory
        tester.send_keys("nonexistent_dir").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Should show error
        tester.wait_for("Error").unwrap();
        tester.assert_contains("not found").unwrap();

        tester.quit(true).unwrap();
    }

// ============================================================================
// Performance Tests
// ============================================================================

    #[test]
    #[ignore]
    fn test_large_directory_performance() {
        let (mut tester, fixture) = create_tester().unwrap();

        // Create very large directory
        fixture.create_dir("perf_test").unwrap();
        for i in 1..=1000 {
            let filename = format!("perf_test/file_{:04}.txt", i);
            fixture.create_file(&filename, "Content").unwrap();
        }

        let start = std::time::Instant::now();

        tester.launch().unwrap();

        tester.wait_for("perf_test").unwrap();

        let load_time = start.elapsed();

        // Should load within reasonable time
        assert!(load_time < Duration::from_secs(5), "Directory took too long to load: {:?}", load_time);

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_navigation_performance() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_large_structure().unwrap();

        tester.launch().unwrap();

        tester.wait_for("many_files").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        let start = std::time::Instant::now();

        // Scroll through entire list
        for _ in 0..100 {
            tester.send_special_key(tui_tester::SpecialKey::Down).unwrap();
        }

        let scroll_time = start.elapsed();

        // Scrolling should be responsive
        assert!(scroll_time < Duration::from_secs(2), "Scrolling took too long: {:?}", scroll_time);

        tester.quit(true).unwrap();
    }

// ============================================================================
// Edge Cases Tests
// ============================================================================

    #[test]
    #[ignore]
    fn test_empty_directory() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_dir("empty").unwrap();

        tester.launch().unwrap();

        tester.wait_for("empty").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Should show empty message
        tester.wait_for("Empty").unwrap();

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_special_characters_in_filename() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_file("file with spaces.txt", "Content").unwrap();
        fixture.create_file("file-with-dashes.txt", "Content").unwrap();
        fixture.create_file("file_with_underscores.txt", "Content").unwrap();

        tester.launch().unwrap();

        tester.wait_for("file with spaces.txt").unwrap();
        tester.assert_contains("file-with-dashes.txt").unwrap();

        tester.quit(true).unwrap();
    }

    #[test]
    #[ignore]
    fn test_unicode_filenames() {
        let (mut tester, fixture) = create_tester().unwrap();

        fixture.create_file("файл.txt", "Content").unwrap();
        fixture.create_file("ファイル.txt", "Content").unwrap();
        fixture.create_file("파일.txt", "Content").unwrap();

        tester.launch().unwrap();

        tester.wait_for("файл.txt").unwrap();

        tester.quit(true).unwrap();
    }

// ============================================================================
// Integration Tests
// ============================================================================

    #[test]
    #[ignore]
    fn test_complete_workflow() {
        let (mut tester, fixture) = create_tester().unwrap();

        // Complete workflow: create, navigate, edit, cleanup
        fixture.create_standard_structure().unwrap();

        tester.launch().unwrap();

        // Navigate to directory
        tester.wait_for("dir1").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Create new file
        tester.send_special_key(tui_tester::SpecialKey::Escape).unwrap();
        tester.send_keys(":touch workflow_test.txt").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        // Verify
        common::assert_eventually(
            || fixture.exists("dir1/workflow_test.txt"),
            Duration::from_secs(2),
            "File should be created"
        );

        // Go back
        tester.send_special_key(tui_tester::SpecialKey::Escape).unwrap();

        // Create tab
        tester.send_special_key(tui_tester::SpecialKey::CtrlT).unwrap();

        // Navigate elsewhere
        tester.send_keys("dir2").unwrap();
        tester.send_special_key(tui_tester::SpecialKey::Enter).unwrap();

        tester.wait_for("subdir").unwrap();

        // Quit
        tester.quit(true).unwrap();
    }
