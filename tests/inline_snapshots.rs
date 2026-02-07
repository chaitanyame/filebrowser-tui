//! Inline snapshot tests for quick UI validation
//!
//! This module uses inline snapshots (stored directly in the test file)
//! for quick iteration during development. Inline snapshots are easier to
//! update during active development.
//!
//! To update inline snapshots:
//!   cargo insta test --accept --workspace

#![allow(clippy::unwrap_used)]

mod common;
mod visual_tester;

use std::path::PathBuf;

use common::{make_test_files, sort_test_files};
use filebrowser_tui::state::{ActivePane, App, Config, Mode, Pane, MessageLevel};
use visual_tester::VisualTester;

/// Quick inline snapshot test for basic file list view
#[test]
fn inline_basic_file_list() {
    let mut app = App::new().expect("Failed to create app");
    app.all_files = make_test_files();
    sort_test_files(&mut app.all_files);
    app.displayed_indices = (0..app.all_files.len()).collect();
    app.selected_index = 0;

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output, @r###"
    ┌────────────────────────────────────────────────────────────────────────────────┐
    │Tab 1 │ │                                                                      │
    ├────────────────────────────────────────────────────────────────────────────────┤
    │                                                                                │
    │  📁 Documents                                                                  │
    │  📁 Downloads                                                                  │
    │  📁 Music                                                                      │
    │  📁 Pictures                                                                   │
    │  📁 Videos                                                                    │
    │  📄 document.pdf                                                               │
    │  📄 file1.txt                                                                  │
    │  📄 file2.txt                                                                  │
    │  📄 file3.txt                                                                  │
    │  📄 image.png                                                                  │
    │  📄 large_file.bin                                                             │
    │                                                                                │
    │                                                                                │
    │                                                                                │
    │                                                                                │
    │                                                                                │
    │                                                                                │
    │                                                                                │
    │                                                                                │
    │                                                                                │
    ├────────────────────────────────────────────────────────────────────────────────┤
    │ ↑↓ Move │ Enter: Open │ Tab: New Tab │ ?: Help │ q: Quit                       │
    └────────────────────────────────────────────────────────────────────────────────┘
    "###);
}

/// Quick inline snapshot test for search mode
#[test]
fn inline_search_mode() {
    let mut app = App::new().expect("Failed to create app");
    app.all_files = make_test_files();
    sort_test_files(&mut app.all_files);
    app.mode = Mode::Search;
    app.search_query = Some("file".to_string());
    app.command_input = "file".to_string();
    app.displayed_indices = vec![6, 7, 8]; // Files matching "file"
    app.selected_index = 0;

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

/// Quick inline snapshot test for split view
#[test]
fn inline_split_view() {
    let mut app = App::new().expect("Failed to create app");
    app.all_files = make_test_files();
    sort_test_files(&mut app.all_files);
    app.split_view = true;
    app.active_pane = ActivePane::Left;

    app.left_pane = Pane::new(PathBuf::from("/left"));
    app.left_pane.files = make_test_files();
    sort_test_files(&mut app.left_pane.files);
    app.left_pane.displayed_indices = (0..app.left_pane.files.len()).collect();
    app.left_pane.selected_index = 0;

    app.right_pane = Pane::new(PathBuf::from("/right"));
    app.right_pane.files = make_test_files();
    sort_test_files(&mut app.right_pane.files);
    app.right_pane.displayed_indices = (0..app.right_pane.files.len()).collect();
    app.right_pane.selected_index = 2;

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}

/// Quick inline snapshot test for preview pane
#[test]
fn inline_preview_pane() {
    let mut app = App::new().expect("Failed to create app");
    app.all_files = make_test_files();
    sort_test_files(&mut app.all_files);
    app.config.show_preview = true;
    app.config.preview_width_percent = 40;
    app.selected_index = 6;

    if let Some(file) = app.get_selected_file() {
        app.preview_file = Some(file.path.clone());
        app.preview_content = Some(
            "Line 1: This is a preview\n\
             Line 2: Showing file contents\n\
             Line 3: In the preview pane\n\
             Line 4: With syntax highlighting\n\
             Line 5: And line numbers\n".to_string(),
        );
    }

    let tester = VisualTester::new();
    let output = tester.capture(&app).expect("Failed to capture output");

    insta::assert_snapshot!(output);
}
