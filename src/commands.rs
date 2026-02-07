use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

use crate::file_ops::{perform_copy, perform_delete, perform_mkdir, perform_move, perform_rename, Clipboard, ClipboardOp, Operation};
use crate::state::{App, ConfirmDialog, Mode};

/// Handle a key event and return true if the app should quit
pub fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<bool> {
    // Handle confirmation dialog
    if app.confirm_dialog.is_some() {
        return handle_confirm_dialog(app, key);
    }

    match app.mode {
        Mode::Normal => handle_normal_mode(app, key),
        Mode::Command => handle_command_mode(app, key),
        Mode::Search => handle_search_mode(app, key),
        Mode::ContentSearch => handle_content_search_mode(app, key),
        Mode::BulkRename => handle_bulk_rename_mode(app, key),
    }
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        // Quit
        KeyCode::Char('q') | KeyCode::Esc => return Ok(true),

        // Tab management
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.update_current_tab();
            app.new_tab();
            app.refresh_file_list()?;
        }
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.close_tab();
            app.refresh_file_list()?;
        }
        KeyCode::Char('1')..=KeyCode::Char('9') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let tab_num = match key.code {
                KeyCode::Char('1') => 0,
                KeyCode::Char('2') => 1,
                KeyCode::Char('3') => 2,
                KeyCode::Char('4') => 3,
                KeyCode::Char('5') => 4,
                KeyCode::Char('6') => 5,
                KeyCode::Char('7') => 6,
                KeyCode::Char('8') => 7,
                KeyCode::Char('9') => 8,
                _ => return Ok(false),
            };
            app.update_current_tab();
            app.switch_tab(tab_num);
            app.refresh_file_list()?;
        }
        KeyCode::Tab => {
            if app.split_view {
                // In split view, Tab switches between panes
                app.switch_active_pane();
            } else {
                // Otherwise, Tab switches between tabs
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    app.update_current_tab();
                    app.prev_tab();
                    app.refresh_file_list()?;
                } else {
                    app.update_current_tab();
                    app.next_tab();
                    app.refresh_file_list()?;
                }
            }
        }

        // Split view toggle
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.toggle_split_view()?;
            if app.split_view {
                // Refresh both panes
                let _ = app.left_pane.refresh_file_list(
                    app.config.show_hidden,
                    app.config.sort_by,
                    app.config.sort_order,
                );
                let _ = app.right_pane.refresh_file_list(
                    app.config.show_hidden,
                    app.config.sort_by,
                    app.config.sort_order,
                );
            } else {
                // Refresh single view
                app.refresh_file_list()?;
            }
        }

        // File transfer between panes (F5 = Copy, F6 = Move)
        KeyCode::F(5) => {
            if app.split_view {
                app.copy_to_other_pane()?;
            } else {
                // Normal refresh behavior when not in split view
                app.refresh_file_list()?;
                app.set_message("Refreshed".to_string(), crate::state::MessageLevel::Info);
            }
        }
        KeyCode::F(6) => {
            if app.split_view {
                app.move_to_other_pane()?;
            }
        }

        // Navigation
        KeyCode::Up | KeyCode::Char('k') => {
            app.move_selection(-1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.move_selection(1);
        }
        KeyCode::Left | KeyCode::Char('h') => {
            // Optionally go to parent or move to preview
            app.go_up()?;
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.enter_selected()?;
        }
        KeyCode::PageUp => {
            app.move_page(-1);
        }
        KeyCode::PageDown => {
            app.move_page(1);
        }
        KeyCode::Home => {
            app.selected_index = 0;
            app.scroll_offset = 0;
        }
        KeyCode::End => {
            if !app.displayed_indices.is_empty() {
                app.selected_index = app.displayed_indices.len() - 1;
                app.ensure_visible();
            }
        }
        KeyCode::Enter => {
            app.enter_selected()?;
        }
        KeyCode::Backspace => {
            app.go_up()?;
        }

        // Selection
        KeyCode::Char(' ') => {
            app.toggle_selection();
        }

        // View toggles
        KeyCode::Char('.') => {
            app.toggle_hidden()?;
        }
        KeyCode::Char('p') => {
            app.toggle_preview();
        }
        KeyCode::Char('P') => {
            app.toggle_sort(crate::state::SortBy::Size)?;
        }
        KeyCode::Char('T') => {
            app.toggle_sort(crate::state::SortBy::Type)?;
        }
        KeyCode::Char('D') => {
            app.toggle_sort(crate::state::SortBy::Modified)?;
        }
        KeyCode::Char('N') => {
            app.toggle_sort(crate::state::SortBy::Name)?;
        }

        // Refresh
        KeyCode::Char('L') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.refresh_file_list()?;
            app.set_message("Refreshed".to_string(), crate::state::MessageLevel::Info);
        }

        // Search
        KeyCode::Char('/') => {
            app.start_search();
        }
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.start_search();
        }
        KeyCode::Char('n') => {
            app.next_search_match();
        }
        KeyCode::Char('N') => {
            app.prev_search_match();
        }

        // Content search (grep-like)
        KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.start_content_search();
        }

        // Bulk rename
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.start_bulk_rename();
        }

        // Selection commands
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.select_all();
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.deselect_all();
        }
        KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.invert_selection();
        }

        // Clipboard operations
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if !app.selection.is_empty() {
                app.clipboard = Some(Clipboard {
                    operation: ClipboardOp::Copy,
                    sources: app.selection.get_selected(),
                });
                app.set_message(
                    format!("Copied {} item(s)", app.clipboard.as_ref().map(|c| c.sources.len()).unwrap_or(0)),
                    crate::state::MessageLevel::Success,
                );
            } else if let Some(file) = app.get_selected_file() {
                app.clipboard = Some(Clipboard {
                    operation: ClipboardOp::Copy,
                    sources: vec![file.path.clone()],
                });
                app.set_message("Copied 1 item".to_string(), crate::state::MessageLevel::Success);
            }
        }
        KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if !app.selection.is_empty() {
                app.clipboard = Some(Clipboard {
                    operation: ClipboardOp::Cut,
                    sources: app.selection.get_selected(),
                });
                app.set_message(
                    format!("Cut {} item(s)", app.clipboard.as_ref().map(|c| c.sources.len()).unwrap_or(0)),
                    crate::state::MessageLevel::Success,
                );
            } else if let Some(file) = app.get_selected_file() {
                app.clipboard = Some(Clipboard {
                    operation: ClipboardOp::Cut,
                    sources: vec![file.path.clone()],
                });
                app.set_message("Cut 1 item".to_string(), crate::state::MessageLevel::Success);
            }
        }
        KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(clipboard) = app.clipboard.clone() {
                for source in &clipboard.sources {
                    let target_name = source.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let target = app.current_path().join(&target_name);

                    match clipboard.operation {
                        ClipboardOp::Copy => {
                            if let Err(e) = perform_copy(source, &target) {
                                app.set_message(
                                    format!("Failed to copy: {}", e),
                                    crate::state::MessageLevel::Error,
                                );
                            } else {
                                // Record copy operation for undo
                                app.history.record(Operation::Copy {
                                    source: source.clone(),
                                    destination: target.clone(),
                                });
                            }
                        }
                        ClipboardOp::Cut => {
                            if let Err(e) = perform_move(source, &target) {
                                app.set_message(
                                    format!("Failed to move: {}", e),
                                    crate::state::MessageLevel::Error,
                                );
                            } else {
                                // Record move operation for undo
                                app.history.record(Operation::Move {
                                    original_path: source.clone(),
                                    new_path: target.clone(),
                                });
                            }
                        }
                    }
                }
                app.refresh_file_list()?;
                app.set_message("Pasted".to_string(), crate::state::MessageLevel::Success);
            }
        }

        // Undo/Redo operations
        KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                // Ctrl+Shift+Z or Ctrl+Y for redo
                if app.can_redo() {
                    app.redo()?;
                } else {
                    app.set_message("Nothing to redo".to_string(), crate::state::MessageLevel::Warning);
                }
            } else {
                // Ctrl+Z for undo
                if app.can_undo() {
                    app.undo()?;
                } else {
                    app.set_message("Nothing to undo".to_string(), crate::state::MessageLevel::Warning);
                }
            }
        }
        KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Ctrl+Y for redo (alternative)
            if app.can_redo() {
                app.redo()?;
            } else {
                app.set_message("Nothing to redo".to_string(), crate::state::MessageLevel::Warning);
            }
        }

        // Create directory
        KeyCode::F(7) => {
            app.mode = Mode::Command;
            app.command_input = "mkdir ".to_string();
            app.set_message("Enter directory name: ".to_string(), crate::state::MessageLevel::Info);
        }

        // Delete
        KeyCode::F(8) | KeyCode::Delete => {
            let files_to_delete: Vec<PathBuf> = if !app.selection.is_empty() {
                app.selection.get_selected()
            } else if let Some(file) = app.get_selected_file() {
                vec![file.path.clone()]
            } else {
                vec![]
            };

            if !files_to_delete.is_empty() {
                app.confirm_dialog = Some(ConfirmDialog::Delete {
                    files: files_to_delete,
                });
            }
        }

        // Rename
        KeyCode::F(2) => {
            if let Some(file) = app.get_selected_file().cloned() {
                let name = file.name.clone();
                app.mode = Mode::Command;
                app.command_input = format!("rename {}", name);
                app.set_message("Enter new name: ".to_string(), crate::state::MessageLevel::Info);
            }
        }

        // Bookmarks
        KeyCode::Char('m') => {
            if let Some(file) = app.get_selected_file() {
                let bookmark_name = format!("{}-{}", file.name, chrono::Utc::now().timestamp());
                app.bookmarks.add(bookmark_name, file.path.clone());
                let _ = app.bookmarks.save_to_config();
                app.set_message("Bookmark added".to_string(), crate::state::MessageLevel::Success);
            }
        }
        KeyCode::Char('`') => {
            if app.bookmarks.is_empty() {
                app.set_message("No bookmarks".to_string(), crate::state::MessageLevel::Warning);
            } else {
                // List bookmarks in status
                let count = app.bookmarks.len();
                app.set_message(
                    format!("Bookmarks: {} (use 0-9 to jump)", count),
                    crate::state::MessageLevel::Info,
                );
            }
        }
        KeyCode::Char(c) if c.is_ascii_digit() => {
            if let Some(path) = app.bookmarks.get_quick_slot(c) {
                app.change_directory(path.clone())?;
            }
        }

        // Home directory
        KeyCode::Char('~') => {
            app.change_directory(app.home_path.clone())?;
        }

        // Drive switching (Alt+Drive letter)
        KeyCode::Char(c) if c.is_ascii_alphabetic() && key.modifiers.contains(KeyModifiers::ALT) => {
            if let Some(drive_path) = app.drives.get_path_for_drive(c) {
                app.change_directory(drive_path)?;
            }
        }

        // Drive list
        KeyCode::F(9) => {
            let drive_list: Vec<String> = app.drives.drives
                .iter()
                .map(|d| format!("{}: ({:?})", d.letter, d.drive_type))
                .collect();
            app.set_message(
                format!("Drives: {}", drive_list.join(", ")),
                crate::state::MessageLevel::Info,
            );
        }

        _ => {}
    }

    Ok(false)
}

fn handle_command_mode(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Enter => {
            let input = app.command_input.trim();

            if let Some(dir_name) = input.strip_prefix("mkdir ") {
                if !dir_name.is_empty() {
                    let target = app.current_path().join(dir_name);
                    match perform_mkdir(&target) {
                        Ok(_) => {
                            // Record create directory operation for undo
                            app.history.record(Operation::CreateDir {
                                path: target.clone(),
                            });
                            app.set_message(
                                format!("Created: {}", dir_name),
                                crate::state::MessageLevel::Success,
                            );
                            app.refresh_file_list()?;
                        }
                        Err(e) => {
                            app.set_message(
                                format!("Failed to create directory: {}", e),
                                crate::state::MessageLevel::Error,
                            );
                        }
                    }
                }
            } else if let Some(new_name) = input.strip_prefix("rename ") {
                if !new_name.is_empty() {
                    if let Some(file) = app.get_selected_file() {
                        let original_path = file.path.clone();
                        match perform_rename(&file.path, new_name) {
                            Ok(new_path) => {
                                // Record rename operation for undo
                                app.history.record(Operation::Rename {
                                    original_path: original_path.clone(),
                                    new_path: new_path.clone(),
                                });
                                app.set_message(
                                    format!("Renamed to {}", new_name),
                                    crate::state::MessageLevel::Success,
                                );
                                app.refresh_file_list()?;
                            }
                            Err(e) => {
                                app.set_message(
                                    format!("Failed to rename: {}", e),
                                    crate::state::MessageLevel::Error,
                                );
                            }
                        }
                    }
                }
            }

            app.mode = Mode::Normal;
            app.command_input.clear();
        }
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.command_input.clear();
        }
        KeyCode::Char(c) => {
            app.command_input.push(c);
        }
        KeyCode::Backspace => {
            app.command_input.pop();
        }
        _ => {}
    }

    Ok(false)
}

fn handle_search_mode(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Enter => {
            app.exit_search();
        }
        KeyCode::Esc => {
            app.exit_search();
        }
        KeyCode::Char(c) => {
            app.command_input.push(c);
            app.update_search();
        }
        KeyCode::Backspace => {
            app.command_input.pop();
            app.update_search();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.prev_search_match();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_search_match();
        }
        _ => {}
    }

    Ok(false)
}

fn handle_confirm_dialog(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let Some(dialog) = app.confirm_dialog.take() {
                match dialog {
                    ConfirmDialog::Delete { files } => {
                        let mut deleted = 0;
                        for path in &files {
                            let was_directory = path.is_dir();
                            // Create backup before deleting if possible
                            let backup_path = if path.exists() {
                                match app.history.create_backup(path) {
                                    Ok(backup) => Some(backup),
                                    Err(_) => None,
                                }
                            } else {
                                None
                            };

                            if perform_delete(path).is_ok() {
                                deleted += 1;
                                // Record delete operation for undo
                                app.history.record(Operation::Delete {
                                    path: path.clone(),
                                    was_directory,
                                    backup_path,
                                });
                            }
                        }
                        app.refresh_file_list()?;
                        app.set_message(
                            format!("Deleted {} item(s)", deleted),
                            crate::state::MessageLevel::Success,
                        );
                    }
                    ConfirmDialog::Overwrite { source, target } => {
                        // Handle overwrite
                        let _ = perform_copy(&source, &target);
                        app.refresh_file_list()?;
                    }
                }
            }
        }
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
            app.confirm_dialog = None;
        }
        _ => {}
    }

    Ok(false)
}

fn handle_content_search_mode(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Enter => {
            if app.content_search_query.is_some() && app.command_input.is_empty() {
                // Search has been executed, Enter opens the selected result
                app.open_content_search_result()?;
            } else {
                // Execute the search
                app.execute_content_search();
            }
        }
        KeyCode::Esc => {
            app.exit_content_search();
        }
        KeyCode::Char(c) => {
            app.command_input.push(c);
        }
        KeyCode::Backspace => {
            app.command_input.pop();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.prev_content_search_result();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_content_search_result();
        }
        KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Start new search
            app.start_content_search();
        }
        _ => {}
    }

    Ok(false)
}

fn handle_bulk_rename_mode(app: &mut App, key: KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Enter => {
            // Apply the bulk rename
            if let Err(e) = app.execute_bulk_rename() {
                app.set_message(
                    format!("Rename failed: {}", e),
                    crate::state::MessageLevel::Error,
                );
            }
        }
        KeyCode::Esc => {
            app.cancel_bulk_rename();
        }
        KeyCode::Char(' ') => {
            // Toggle acceptance of selected item
            app.toggle_rename_acceptance();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.move_rename_selection(-1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.move_rename_selection(1);
        }
        KeyCode::Tab => {
            // Switch to pattern editing mode (toggle between preview and pattern input)
            // For now, just update the preview when Tab is pressed
            if let Err(e) = app.update_rename_preview() {
                app.set_message(
                    format!("Pattern error: {}", e),
                    crate::state::MessageLevel::Error,
                );
            } else {
                // Store the pattern for execution
                let _ = app.parse_rename_pattern().map(|pattern| {
                    app.rename_pattern = Some(pattern);
                });
            }
        }
        KeyCode::Char(c) => {
            app.rename_pattern_input.push(c);
            // Update preview in real-time
            if let Err(_) = app.update_rename_preview() {
                // Pattern might be incomplete, ignore error
            }
        }
        KeyCode::Backspace => {
            app.rename_pattern_input.pop();
            // Update preview in real-time
            if let Err(_) = app.update_rename_preview() {
                // Pattern might be incomplete, ignore error
            }
        }
        _ => {}
    }

    Ok(false)
}
