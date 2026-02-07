use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

use crate::file_ops::Navigator;
use crate::state::{FileEntry, SelectionManager, SortBy, SortOrder};

/// Represents a single pane in the dual-pane view
/// Contains all state needed for independent pane operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pane {
    /// Current path of this pane
    pub path: PathBuf,
    /// Previous path (for navigation back)
    pub previous_path: Option<PathBuf>,
    /// All files in the current directory
    pub files: Vec<FileEntry>,
    /// Indices of files to display (after filtering)
    pub displayed_indices: Vec<usize>,
    /// Currently selected file index
    pub selected_index: usize,
    /// Scroll offset for the file list
    pub scroll_offset: usize,
    /// Selection state specific to this pane
    pub selection: SelectionManager,
    /// Search query for this pane
    pub search_query: Option<String>,
}

impl Pane {
    /// Create a new pane with the given path
    pub fn new(path: PathBuf) -> Self {
        Pane {
            path,
            previous_path: None,
            files: Vec::new(),
            displayed_indices: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            selection: SelectionManager::new(),
            search_query: None,
        }
    }

    /// Refresh the file list for this pane
    pub fn refresh_file_list(&mut self, show_hidden: bool, sort_by: SortBy, sort_order: SortOrder) -> Result<()> {
        let navigator = Navigator::new();

        // Read directory
        match navigator.read_directory(&self.path) {
            Ok(mut files) => {
                // Sort files
                crate::state::files::sort_files(&mut files, sort_by, sort_order);
                self.files = files;

                // Filter files
                self.displayed_indices = crate::state::files::filter_files(
                    &self.files,
                    show_hidden,
                    self.search_query.as_deref(),
                );

                // Reset selection if needed
                self.selected_index = 0;
                self.scroll_offset = 0;

                // Clear selection for files not in current directory
                self.clear_old_selections();

                Ok(())
            }
            Err(e) => {
                // Return error but keep current state
                Err(e.into())
            }
        }
    }

    /// Change directory for this pane
    pub fn change_directory(&mut self, path: PathBuf) -> Result<()> {
        if path == self.path {
            return Ok(());
        }

        // Save current path as previous
        self.previous_path = Some(self.path.clone());

        // Navigate to new path
        if path.exists() {
            self.path = path.canonicalize().unwrap_or(path);
            self.selection.deselect_all();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Path does not exist"))
        }
    }

    /// Navigate to parent directory
    pub fn go_up(&mut self) -> Result<PathBuf> {
        if let Some(parent) = self.path.parent() {
            let parent_path = parent.to_path_buf();
            self.change_directory(parent_path.clone())?;
            Ok(parent_path)
        } else {
            Err(anyhow::anyhow!("Already at root"))
        }
    }

    /// Enter the selected directory
    pub fn enter_selected(&mut self) -> Result<()> {
        if self.is_empty_list() {
            return Ok(());
        }

        let current_index = self.displayed_indices.get(self.selected_index);
        if current_index.is_none() {
            return Ok(());
        }

        let file = &self.files[*current_index.unwrap()];
        let path = file.path.clone();
        let is_dir = file.is_dir;

        if is_dir {
            self.change_directory(path)?;
        }

        Ok(())
    }

    /// Check if the file list is empty
    pub fn is_empty_list(&self) -> bool {
        self.displayed_indices.is_empty()
    }

    /// Get the currently selected file
    pub fn get_selected_file(&self) -> Option<&FileEntry> {
        if self.is_empty_list() {
            return None;
        }
        self.displayed_indices
            .get(self.selected_index)
            .map(|&i| &self.files[i])
    }

    /// Move selection by delta
    pub fn move_selection(&mut self, delta: isize) {
        if self.is_empty_list() {
            return;
        }

        let new_index = if delta >= 0 {
            self.selected_index + delta as usize
        } else {
            self.selected_index.saturating_sub((-delta) as usize)
        };

        self.selected_index = new_index.min(self.displayed_indices.len() - 1);
        self.ensure_visible();
    }

    /// Navigate to a specific file index
    pub fn navigate_to_file_index(&mut self, index: usize) {
        if !self.displayed_indices.is_empty() {
            self.selected_index = index.min(self.displayed_indices.len() - 1);
            self.ensure_visible();
        }
    }

    /// Toggle selection for current file
    pub fn toggle_selection(&mut self) {
        if self.is_empty_list() {
            return;
        }

        if let Some(idx) = self.displayed_indices.get(self.selected_index) {
            let file = &self.files[*idx];
            self.selection.toggle(file.path.clone());
            self.selection.set_last_selected(self.selected_index);

            // Move to next item
            if self.selected_index < self.displayed_indices.len() - 1 {
                self.selected_index += 1;
                self.ensure_visible();
            }
        }
    }

    /// Select all files
    pub fn select_all(&mut self) {
        let all_paths: Vec<PathBuf> = self.displayed_indices
            .iter()
            .map(|i| self.files[*i].path.clone())
            .collect();
        self.selection.select_all(all_paths);
    }

    /// Update search query
    pub fn update_search(&mut self, query: Option<String>, show_hidden: bool) {
        self.search_query = query;

        // Re-filter the display
        self.displayed_indices = crate::state::files::filter_files(
            &self.files,
            show_hidden,
            self.search_query.as_deref(),
        );

        if !self.displayed_indices.is_empty() {
            self.selected_index = 0;
            self.scroll_offset = 0;
        }
    }

    /// Ensure selected item is visible
    pub fn ensure_visible(&mut self) {
        const LIST_HEIGHT: usize = 20; // Will be adjusted by UI

        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + LIST_HEIGHT {
            self.scroll_offset = self.selected_index - LIST_HEIGHT + 1;
        }
    }

    /// Clear selections for files not in current directory
    fn clear_old_selections(&mut self) {
        let current_paths: HashSet<PathBuf> = self.files
            .iter()
            .map(|f| f.path.clone())
            .collect();

        let selected = self.selection.get_selected();
        for path in selected {
            if !current_paths.contains(&path) {
                self.selection.deselect(&path);
            }
        }
    }

    /// Get display name for the pane
    pub fn display_name(&self) -> String {
        self.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("/")
            .to_string()
    }

    /// Get truncated path for display
    pub fn display_path(&self, max_length: usize) -> String {
        let path_str = self.path.display().to_string();
        if path_str.len() > max_length {
            let start = path_str.len() - max_length + 3;
            format!("...{}", &path_str[start..])
        } else {
            path_str
        }
    }
}

impl Default for Pane {
    fn default() -> Self {
        Pane::new(PathBuf::from("."))
    }
}

/// Enum to identify which pane is active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivePane {
    Left,
    Right,
}

impl ActivePane {
    /// Toggle to the other pane
    pub fn toggle(&self) -> Self {
        match self {
            ActivePane::Left => ActivePane::Right,
            ActivePane::Right => ActivePane::Left,
        }
    }
}
