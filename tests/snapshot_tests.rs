//! Snapshot tests for TUI UI rendering
//!
//! This module provides comprehensive snapshot testing for the file browser TUI.
//! Snapshots are stored in `tests/snapshots/` and can be reviewed/updated with:
//!   cargo insta test --review
//!   cargo insta review
//!   cargo insta accept

#![allow(clippy::unwrap_used)]

mod common;
mod visual_tester;

use std::path::PathBuf;

use common::{make_test_files, sort_test_files};
use filebrowser_tui::state::{
    ActivePane, App, ConfirmDialog, Config, FileEntry, Mode, Pane, SortBy, SortOrder,
    MessageLevel,
};
use visual_tester::VisualTester;

/// Helper to create a basic app for testing
fn create_test_app() -> App {
    let mut app = App::new().expect("Failed to create app");
    app.all_files = make_test_files();
    sort_test_files(&mut app.all_files);
    app.displayed_indices = (0..app.all_files.len()).collect();
    app.selected_index = 0;
    app.scroll_offset = 0;
    app
}

/// Helper to set up app with a custom path
fn create_app_with_path(path: PathBuf) -> App {
    let mut app = App::new().expect("Failed to create app");
    app.set_current_path(path);
    app.all_files = make_test_files();
    sort_test_files(&mut app.all_files);
    app.displayed_indices = (0..app.all_files.len()).collect();
    app.selected_index = 0;
    app.scroll_offset = 0;
    app
}

/// Helper to create app with specific files
fn create_app_with_files(files: Vec<FileEntry>) -> App {
    let mut app = App::new().expect("Failed to create app");
    let mut sorted_files = files;
    sort_test_files(&mut sorted_files);
    app.all_files = sorted_files;
    app.displayed_indices = (0..app.all_files.len()).collect();
    app.selected_index = 0;
    app.scroll_offset = 0;
    app
}

// ============================================================================
// Test Scenarios
// ============================================================================

#[test]
fn snapshot_empty_directory() {
    // Test rendering of an empty directory
    let mut app = App::new().expect("Failed to create app");
    app.all_files.clear();
    app.displayed_indices.clear();
    app.selected_index = 0;
    app.scroll_offset = 0;
    app.set_message("Directory is empty", MessageLevel::Info);

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output, @r###"
    ┌────────────────────────────────────────────────────────────────────────────────┐
    │ │ │                                                                            │
    └────────────────────────────────────────────────────────────────────────────────┘
    "###);
}

#[test]
fn snapshot_directory_with_files() {
    // Test rendering of a directory with multiple files
    let app = create_test_app();

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_selected_file() {
    // Test rendering with a file selected
    let mut app = create_test_app();
    app.selected_index = 3;
    app.scroll_offset = 0;

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_search_mode() {
    // Test rendering in search mode
    let mut app = create_test_app();
    app.mode = Mode::Search;
    app.search_query = Some("file".to_string());
    app.command_input = "file".to_string();
    app.displayed_indices = vec![6, 7, 8]; // Indices of files matching "file"
    app.selected_index = 0;
    app.scroll_offset = 0;

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_split_view() {
    // Test rendering of split view (dual-pane mode)
    let mut app = create_test_app();
    app.split_view = true;
    app.active_pane = ActivePane::Left;

    // Setup left pane
    app.left_pane = Pane::new(PathBuf::from("/test/left"));
    app.left_pane.files = make_test_files();
    sort_test_files(&mut app.left_pane.files);
    app.left_pane.displayed_indices = (0..app.left_pane.files.len()).collect();
    app.left_pane.selected_index = 0;
    app.left_pane.scroll_offset = 0;

    // Setup right pane
    app.right_pane = Pane::new(PathBuf::from("/test/right"));
    app.right_pane.files = make_test_files();
    sort_test_files(&mut app.right_pane.files);
    app.right_pane.displayed_indices = (0..app.right_pane.files.len()).collect();
    app.right_pane.selected_index = 2;
    app.right_pane.scroll_offset = 0;

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_multiple_tabs() {
    // Test rendering with multiple tabs
    let mut app = create_test_app();

    // Add additional tabs
    app.new_tab();
    app.tabs[1].path = PathBuf::from("/home/user/documents");
    app.tabs[1].display_name = "documents".to_string();

    app.new_tab();
    app.tabs[2].path = PathBuf::from("/home/user/downloads");
    app.tabs[2].display_name = "downloads".to_string();

    app.current_tab = 1;

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_preview_pane() {
    // Test rendering with preview pane enabled
    let mut app = create_test_app();
    app.config.show_preview = true;
    app.config.preview_width_percent = 40;
    app.selected_index = 6; // Select a file

    if let Some(file) = app.get_selected_file() {
        app.preview_file = Some(file.path.clone());
        app.preview_content = Some("Line 1 of file content\nLine 2 of file content\nLine 3 of file content\n".to_string());
    }

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_bulk_rename_preview() {
    // Test rendering of bulk rename mode with preview
    let mut app = create_test_app();
    app.mode = Mode::BulkRename;
    app.rename_pattern_input = "file,new_file".to_string();

    // Setup rename previews
    use filebrowser_tui::file_ops::RenamePreview;
    app.rename_previews = vec![
        RenamePreview {
            original_path: PathBuf::from("/test/file1.txt"),
            new_name: "new_file1.txt".to_string(),
            accepted: true,
        },
        RenamePreview {
            original_path: PathBuf::from("/test/file2.txt"),
            new_name: "new_file2.txt".to_string(),
            accepted: true,
        },
        RenamePreview {
            original_path: PathBuf::from("/test/file3.txt"),
            new_name: "new_file3.txt".to_string(),
            accepted: false,
        },
    ];
    app.rename_selected_index = 0;

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_confirmation_dialog() {
    // Test rendering of confirmation dialog
    let mut app = create_test_app();
    app.confirm_dialog = Some(ConfirmDialog::Delete {
        files: vec![
            PathBuf::from("/test/file1.txt"),
            PathBuf::from("/test/file2.txt"),
        ],
    });

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

// ============================================================================
// Additional Edge Cases
// ============================================================================

#[test]
fn snapshot_long_filename() {
    // Test rendering with very long filenames
    let long_name = "this_is_a_very_long_filename_that_exceeds_normal_width.txt";
    let files = vec![
        common::make_test_file(long_name, false, 1024, false),
        common::make_test_file("normal.txt", false, 512, false),
    ];
    let mut app = create_app_with_files(files);
    app.selected_index = 0;

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_many_files_scrolled() {
    // Test rendering with scrolling
    let mut files = Vec::new();
    for i in 0..50 {
        files.push(common::make_test_file(&format!("file{:02}.txt", i), false, i * 100, false));
    }

    let mut app = create_app_with_files(files);
    app.selected_index = 25;
    app.scroll_offset = 15;

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_with_message() {
    // Test rendering with status message
    let mut app = create_test_app();
    app.set_message("3 files selected", MessageLevel::Info);

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_with_error_message() {
    // Test rendering with error message
    let mut app = create_test_app();
    app.set_message("Permission denied", MessageLevel::Error);

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_with_warning_message() {
    // Test rendering with warning message
    let mut app = create_test_app();
    app.set_message("File already exists", MessageLevel::Warning);

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_with_success_message() {
    // Test rendering with success message
    let mut app = create_test_app();
    app.set_message("File copied successfully", MessageLevel::Success);

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_hidden_files_shown() {
    // Test rendering with hidden files visible
    let mut app = create_test_app();
    app.config.show_hidden = true;
    // Re-filter to include hidden files
    app.displayed_indices = (0..app.all_files.len()).collect();

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_command_mode() {
    // Test rendering in command mode
    let mut app = create_test_app();
    app.mode = Mode::Command;
    app.command_input = ":cd /home/user".to_string();

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_content_search_mode() {
    // Test rendering in content search mode
    let mut app = create_test_app();
    app.mode = Mode::ContentSearch;
    app.content_search_query = Some("search_term".to_string());
    app.command_input = "search_term".to_string();

    // Add mock search results
    use filebrowser_tui::file_ops::SearchResult;
    app.content_search_results = vec![
        SearchResult {
            file_path: PathBuf::from("/test/file1.txt"),
            line_number: 5,
            line_content: "This line contains search_term".to_string(),
        },
        SearchResult {
            file_path: PathBuf::from("/test/file2.txt"),
            line_number: 10,
            line_content: "Another line with search_term here".to_string(),
        },
    ];
    app.content_search_selected_index = 0;
    app.content_search_scroll_offset = 0;

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_narrow_terminal() {
    // Test rendering on narrow terminal (60 columns)
    let app = create_test_app();

    let tester = VisualTester::with_size(60, 24);
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_wide_terminal() {
    // Test rendering on wide terminal (120 columns)
    let app = create_test_app();

    let tester = VisualTester::with_size(120, 30);
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_split_view_different_paths() {
    // Test split view with different paths in each pane
    let mut app = create_test_app();
    app.split_view = true;
    app.active_pane = ActivePane::Right;

    // Setup left pane with one set of files
    app.left_pane = Pane::new(PathBuf::from("/home/user/documents"));
    let mut left_files = vec![
        common::make_test_file("report.docx", false, 24576, false),
        common::make_test_file("presentation.pptx", false, 5324800, false),
        common::make_test_file("spreadsheet.xlsx", false, 32768, false),
    ];
    sort_test_files(&mut left_files);
    app.left_pane.files = left_files;
    app.left_pane.displayed_indices = vec![0, 1, 2];
    app.left_pane.selected_index = 1;
    app.left_pane.scroll_offset = 0;

    // Setup right pane with different files
    app.right_pane = Pane::new(PathBuf::from("/home/user/downloads"));
    let mut right_files = vec![
        common::make_test_file("archive.zip", false, 10485760, false),
        common::make_test_file("installer.exe", false, 52428800, false),
    ];
    sort_test_files(&mut right_files);
    app.right_pane.files = right_files;
    app.right_pane.displayed_indices = vec![0, 1];
    app.right_pane.selected_index = 0;
    app.right_pane.scroll_offset = 0;

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_overwrite_dialog() {
    // Test rendering of overwrite confirmation dialog
    let mut app = create_test_app();
    app.confirm_dialog = Some(ConfirmDialog::Overwrite {
        source: PathBuf::from("/test/source.txt"),
        target: PathBuf::from("/test/target.txt"),
    });

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}
