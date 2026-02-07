use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::file_ops::{Clipboard, HistoryManager, Navigator, RenamePattern, SearchResult, WindowsDrives};
use crate::state::{ActivePane, Bookmarks, FileEntry, Pane, RenamePreview, SelectionManager, SortBy, SortOrder, Tab};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Command,
    Search,
    BulkRename,
    ContentSearch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub show_hidden: bool,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub show_preview: bool,
    pub preview_width_percent: u16,
    pub theme_color: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            show_hidden: false,
            sort_by: SortBy::Name,
            sort_order: SortOrder::Ascending,
            show_preview: false,
            preview_width_percent: 40,
            theme_color: "blue".to_string(),
        }
    }
}

pub struct App {
    // Tab management
    pub tabs: Vec<Tab>,
    pub current_tab: usize,

    // Dual-pane view
    pub split_view: bool,
    pub left_pane: Pane,
    pub right_pane: Pane,
    pub active_pane: ActivePane,
    // Current file listing (for current tab)
    pub all_files: Vec<FileEntry>,
    pub displayed_indices: Vec<usize>,
    pub selected_index: usize,
    pub scroll_offset: usize,

    // Paths and navigation
    pub previous_path: Option<PathBuf>,
    pub home_path: PathBuf,

    // State
    pub mode: Mode,
    pub command_input: String,
    pub search_query: Option<String>,
    pub message: Option<String>,
    pub message_level: MessageLevel,

    // Configuration
    pub config: Config,

    // Selection and operations
    pub selection: SelectionManager,
    pub clipboard: Option<Clipboard>,
    pub bookmarks: Bookmarks,
    pub drives: WindowsDrives,

    // Preview
    pub preview_content: Option<String>,
    pub preview_file: Option<PathBuf>,

    // Confirmation dialog
    pub confirm_dialog: Option<ConfirmDialog>,

    // Bulk rename mode
    pub rename_pattern: Option<RenamePattern>,
    pub rename_previews: Vec<RenamePreview>,
    pub rename_selected_index: usize,
    pub rename_pattern_input: String,

    // Content search mode
    pub content_search_results: Vec<SearchResult>,
    pub content_search_query: Option<String>,
    pub content_search_in_progress: bool,
    pub content_search_selected_index: usize,
    pub content_search_scroll_offset: usize,
    pub content_search_task: Option<Arc<Mutex<Option<tokio::task::JoinHandle<Result<Vec<SearchResult>>>>>>,

    // Undo/Redo history
    pub history: HistoryManager,
}

impl App {
    /// Get the current path of the active tab
    pub fn current_path(&self) -> &PathBuf {
        &self.tabs[self.current_tab].path
    }

    /// Set the current path of the active tab
    pub fn set_current_path(&mut self, path: PathBuf) {
        self.tabs[self.current_tab].path = path;
    }

    /// Get mutable reference to the active pane
    pub fn get_active_pane_mut(&mut self) -> &mut Pane {
        match self.active_pane {
            ActivePane::Left => &mut self.left_pane,
            ActivePane::Right => &mut self.right_pane,
        }
    }

    /// Get reference to the active pane
    pub fn get_active_pane(&self) -> &Pane {
        match self.active_pane {
            ActivePane::Left => &self.left_pane,
            ActivePane::Right => &self.right_pane,
        }
    }

    /// Get mutable reference to the inactive pane
    pub fn get_inactive_pane_mut(&mut self) -> &mut Pane {
        match self.active_pane {
            ActivePane::Left => &mut self.right_pane,
            ActivePane::Right => &mut self.left_pane,
        }
    }

    /// Get reference to the inactive pane
    pub fn get_inactive_pane(&self) -> &Pane {
        match self.active_pane {
            ActivePane::Left => &self.right_pane,
            ActivePane::Right => &self.left_pane,
        }
    }

    /// Toggle split view mode
    pub fn toggle_split_view(&mut self) -> Result<()> {
        self.split_view = !self.split_view;

        if self.split_view {
            // Initialize right pane with current path
            let current_path = self.current_path().clone();
            self.right_pane = Pane::new(current_path);
            let _ = self.right_pane.refresh_file_list(
                self.config.show_hidden,
                self.config.sort_by,
                self.config.sort_order,
            );
            self.set_message("Split view enabled (Tab to switch panes, Ctrl+P to disable)", MessageLevel::Info);
        } else {
            self.set_message("Split view disabled", MessageLevel::Info);
        }
        Ok(())
    }

    /// Switch active pane
    pub fn switch_active_pane(&mut self) {
        if !self.split_view {
            return;
        }

        self.active_pane = self.active_pane.toggle();
        self.set_message(
            format!("Active pane: {:?}", self.active_pane),
            MessageLevel::Info,
        );
    }

    /// Copy selected files from active pane to inactive pane
    pub fn copy_to_other_pane(&mut self) -> Result<()> {
        if !self.split_view {
            self.set_message("Split view not enabled", MessageLevel::Warning);
            return Ok(());
        }

        let active = self.get_active_pane();
        let files_to_copy: Vec<PathBuf> = if !active.selection.is_empty() {
            active.selection.get_selected()
        } else if let Some(file) = active.get_selected_file() {
            vec![file.path.clone()]
        } else {
            vec![]
        };

        if files_to_copy.is_empty() {
            self.set_message("No files selected", MessageLevel::Warning);
            return Ok(());
        }

        let inactive = self.get_inactive_pane();
        let mut copied = 0;
        let target_dir = &inactive.path;

        for source in &files_to_copy {
            let target_name = source.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let target = target_dir.join(&target_name);

            match crate::file_ops::perform_copy(source, &target) {
                Ok(_) => copied += 1,
                Err(e) => {
                    self.set_message(
                        format!("Failed to copy {}: {}", source.display(), e),
                        MessageLevel::Error,
                    );
                }
            }
        }

        // Refresh the inactive pane
        let inactive = self.get_inactive_pane_mut();
        let _ = inactive.refresh_file_list(
            self.config.show_hidden,
            self.config.sort_by,
            self.config.sort_order,
        );

        self.set_message(
            format!("Copied {} item(s) to {}", copied, inactive.display_name()),
            MessageLevel::Success,
        );

        Ok(())
    }

    /// Move selected files from active pane to inactive pane
    pub fn move_to_other_pane(&mut self) -> Result<()> {
        if !self.split_view {
            self.set_message("Split view not enabled", MessageLevel::Warning);
            return Ok(());
        }

        let active = self.get_active_pane();
        let files_to_move: Vec<PathBuf> = if !active.selection.is_empty() {
            active.selection.get_selected()
        } else if let Some(file) = active.get_selected_file() {
            vec![file.path.clone()]
        } else {
            vec![]
        };

        if files_to_move.is_empty() {
            self.set_message("No files selected", MessageLevel::Warning);
            return Ok(());
        }

        let inactive = self.get_inactive_pane();
        let mut moved = 0;
        let target_dir = &inactive.path;

        for source in &files_to_move {
            let target_name = source.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let target = target_dir.join(&target_name);

            match crate::file_ops::perform_move(source, &target) {
                Ok(_) => moved += 1,
                Err(e) => {
                    self.set_message(
                        format!("Failed to move {}: {}", source.display(), e),
                        MessageLevel::Error,
                    );
                }
            }
        }

        // Refresh both panes
        let active = self.get_active_pane_mut();
        let _ = active.refresh_file_list(
            self.config.show_hidden,
            self.config.sort_by,
            self.config.sort_order,
        );

        let inactive = self.get_inactive_pane_mut();
        let _ = inactive.refresh_file_list(
            self.config.show_hidden,
            self.config.sort_by,
            self.config.sort_order,
        );

        self.set_message(
            format!("Moved {} item(s) to {}", moved, inactive.display_name()),
            MessageLevel::Success,
        );

        Ok(())
    }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub enum ConfirmDialog {
    Delete { files: Vec<PathBuf> },
    Overwrite { source: PathBuf, target: PathBuf },
}

impl App {
    pub fn new() -> Result<Self> {
        let home_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let current_path = std::env::current_dir().unwrap_or(home_path.clone());

        let config = Self::load_config()?;

        let bookmarks = Bookmarks::load_from_config()
            .unwrap_or_default();

        let drives = WindowsDrives::new();

        // Create initial tab
        let initial_tab = Tab::from_path(current_path.clone());
        // Create panes for split view
        let mut left_pane = Pane::new(current_path.clone());
        let _ = left_pane.refresh_file_list(config.show_hidden, config.sort_by, config.sort_order);
        let right_pane = Pane::new(current_path.clone());
            split_view: false,
            left_pane,
            right_pane,
            active_pane: ActivePane::Left,

        Ok(App {
            tabs: vec![initial_tab],
            current_tab: 0,
            all_files: Vec::new(),
            displayed_indices: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            previous_path: None,
            home_path,
            mode: Mode::Normal,
            command_input: String::new(),
            search_query: None,
            message: None,
            message_level: MessageLevel::Info,
            config,
            selection: SelectionManager::new(),
            clipboard: None,
            bookmarks,
            drives,
            preview_content: None,
            preview_file: None,
            confirm_dialog: None,
            rename_pattern: None,
            rename_previews: Vec::new(),
            rename_selected_index: 0,
            rename_pattern_input: String::new(),
            content_search_results: Vec::new(),
            content_search_query: None,
            content_search_in_progress: false,
            content_search_selected_index: 0,
            content_search_scroll_offset: 0,
            content_search_task: None,
            history: HistoryManager::new()?,
        })
    }

    /// Create a new tab with the current directory
    pub fn new_tab(&mut self) {
        let current_path = self.current_path().clone();
        let selected_index = self.selected_index;
        let scroll_offset = self.scroll_offset;

        // Create display name
        let display_name = current_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("/")
            .to_string();

        let new_tab = Tab {
            path: current_path,
            selected_index,
            scroll_offset,
            display_name,
        };

        self.tabs.push(new_tab);
        self.current_tab = self.tabs.len() - 1;

        self.set_message(
            format!("New tab created (Tab {})", self.current_tab + 1),
            MessageLevel::Info,
        );
    }

    /// Close the current tab
    pub fn close_tab(&mut self) {
        if self.tabs.len() <= 1 {
            self.set_message(
                "Cannot close the last tab".to_string(),
                MessageLevel::Warning,
            );
            return;
        }

        self.tabs.remove(self.current_tab);

        // Adjust current tab index if needed
        if self.current_tab >= self.tabs.len() {
            self.current_tab = self.tabs.len() - 1;
        }

        // Restore the state of the new current tab
        self.restore_tab_state();

        self.set_message(
            format!("Tab closed (now at Tab {})", self.current_tab + 1),
            MessageLevel::Info,
        );
    }

    /// Switch to a specific tab by index
    pub fn switch_tab(&mut self, tab_index: usize) {
        if tab_index < self.tabs.len() {
            self.current_tab = tab_index;
            self.restore_tab_state();
            self.set_message(
                format!("Switched to Tab {}", tab_index + 1),
                MessageLevel::Info,
            );
        } else {
            self.set_message(
                format!("Tab {} does not exist", tab_index + 1),
                MessageLevel::Warning,
            );
        }
    }

    /// Switch to the next tab
    pub fn next_tab(&mut self) {
        let next_index = (self.current_tab + 1) % self.tabs.len();
        self.switch_tab(next_index);
    }

    /// Switch to the previous tab
    pub fn prev_tab(&mut self) {
        let prev_index = if self.current_tab == 0 {
            self.tabs.len() - 1
        } else {
            self.current_tab - 1
        };
        self.switch_tab(prev_index);
    }

    /// Save the current state to the active tab
    pub fn update_current_tab(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.current_tab) {
            tab.path = self.current_path().clone();
            tab.selected_index = self.selected_index;
            tab.scroll_offset = self.scroll_offset;
            tab.update_display_name();
        }
    }

    /// Restore the state from the current tab
    fn restore_tab_state(&mut self) {
        if let Some(tab) = self.tabs.get(self.current_tab) {
            self.selected_index = tab.selected_index;
            self.scroll_offset = tab.scroll_offset;
        }
    }

    pub fn refresh_file_list(&mut self) -> Result<()> {
        let navigator = Navigator::new();
        let current_path = self.current_path().clone();

        // Read directory
        match navigator.read_directory(&current_path) {
            Ok(mut files) => {
                // Sort files
                crate::state::files::sort_files(
                    &mut files,
                    self.config.sort_by,
                    self.config.sort_order,
                );

                self.all_files = files;

                // Filter files
                self.displayed_indices = crate::state::files::filter_files(
                    &self.all_files,
                    self.config.show_hidden,
                    self.search_query.as_deref(),
                );

                // Reset selection if needed
                self.selected_index = 0;
                self.scroll_offset = 0;

                // Clear selection for files not in current directory
                self.clear_old_selections();

                // Update preview if enabled
                if self.config.show_preview {
                    self.update_preview()?;
                }

                Ok(())
            }
            Err(e) => {
                self.set_message(
                    format!("Failed to read directory: {}", e),
                    MessageLevel::Error,
                );
                Ok(())
            }
        }
    }

    pub fn change_directory(&mut self, path: PathBuf) -> Result<()> {
        let current_path = self.current_path().clone();

        if path == current_path {
            return Ok(());
        }

        // Save current path as previous
        self.previous_path = Some(current_path);

        // Navigate to new path
        if path.exists() {
            let canonical_path = path.canonicalize().unwrap_or(path);
            self.set_current_path(canonical_path);
            self.refresh_file_list()?;
            self.selection.deselect_all();
            Ok(())
        } else {
            self.set_message("Path does not exist", MessageLevel::Error);
            Ok(())
        }
    }

    pub fn go_up(&mut self) -> Result<()> {
        let current_path = self.current_path().clone();

        if let Some(parent) = current_path.parent() {
            let parent_path = parent.to_path_buf();
            self.change_directory(parent_path)?;
            // Try to select the previous directory
            if let Some(prev) = &self.previous_path {
                if let Some(prev_name) = prev.file_name() {
                    if let Some(idx) = self
                        .displayed_indices
                        .iter()
                        .position(|&i| self.all_files[i].name == prev_name.to_string_lossy().to_string())
                    {
                        self.selected_index = idx;
                        self.ensure_visible();
                    }
                }
            }
        }
        Ok(())
    }

    pub fn enter_selected(&mut self) -> Result<()> {
        if self.is_empty_list() {
            return Ok(());
        }

        let current_index = self.displayed_indices.get(self.selected_index);
        if current_index.is_none() {
            return Ok(());
        }

        let file = &self.all_files[*current_index.unwrap()];
        let path = file.path.clone();
        let is_dir = file.is_dir;

        if is_dir {
            self.change_directory(path)?;
        } else {
            // Open file with default Windows application
            self.open_file_with_default_app(&path)?;
        }

        Ok(())
    }

    pub fn open_file_with_default_app(&mut self, path: &PathBuf) -> Result<()> {
        #[cfg(windows)]
        {
            use std::process::Command;

            let path_str = path.to_string_lossy().to_string();
            let result = Command::new("cmd")
                .args(["/c", "start", "", "", &path_str])
                .spawn();

            match result {
                Ok(_) => {
                    self.set_message(
                        format!("Opened: {}", path.file_name().unwrap_or_default().to_string_lossy()),
                        MessageLevel::Success,
                    );
                }
                Err(e) => {
                    self.set_message(
                        format!("Failed to open file: {}", e),
                        MessageLevel::Error,
                    );
                }
            }
        }

        #[cfg(not(windows))]
        {
            use std::process::Command;

            let result = Command::new("xdg-open")
                .arg(path)
                .spawn();

            match result {
                Ok(_) => {
                    self.set_message(
                        format!("Opened: {}", path.file_name().unwrap_or_default().to_string_lossy()),
                        MessageLevel::Success,
                    );
                }
                Err(e) => {
                    self.set_message(
                        format!("Failed to open file: {}", e),
                        MessageLevel::Error,
                    );
                }
            }
        }

        Ok(())
    }

    pub fn navigate_to_file_index(&mut self, index: usize) {
        if !self.displayed_indices.is_empty() {
            self.selected_index = index.min(self.displayed_indices.len() - 1);
            self.ensure_visible();

            // Update preview if enabled
            if self.config.show_preview {
                let _ = self.update_preview();
            }
        }
    }

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

        if self.config.show_preview {
            let _ = self.update_preview();
        }
    }

    pub fn move_page(&mut self, direction: isize) {
        // Page size will be determined by UI, using default 20 for now
        let page_size = 20;
        self.move_selection(direction * page_size);
    }

    pub fn toggle_selection(&mut self) {
        if self.is_empty_list() {
            return;
        }

        if let Some(idx) = self.displayed_indices.get(self.selected_index) {
            let file = &self.all_files[*idx];
            self.selection.toggle(file.path.clone());
            self.selection.set_last_selected(self.selected_index);

            // Move to next item
            if self.selected_index < self.displayed_indices.len() - 1 {
                self.selected_index += 1;
                self.ensure_visible();
            }
        }
    }

    pub fn select_all(&mut self) {
        let all_paths: Vec<PathBuf> = self.displayed_indices
            .iter()
            .map(|i| self.all_files[*i].path.clone())
            .collect();
        self.selection.select_all(all_paths);
        self.set_message(
            format!("Selected {} items", self.selection.count()),
            MessageLevel::Info,
        );
    }

    pub fn deselect_all(&mut self) {
        self.selection.deselect_all();
        self.set_message("Deselected all items", MessageLevel::Info);
    }

    pub fn invert_selection(&mut self) {
        let all_paths: Vec<PathBuf> = self.displayed_indices
            .iter()
            .map(|i| self.all_files[*i].path.clone())
            .collect();
        self.selection.invert(all_paths);
        self.set_message(
            format!("Selected {} items", self.selection.count()),
            MessageLevel::Info,
        );
    }

    pub fn toggle_hidden(&mut self) -> Result<()> {
        self.config.show_hidden = !self.config.show_hidden;
        self.refresh_file_list()?;
        self.set_message(
            format!("Hidden files: {}",
                if self.config.show_hidden { "shown" } else { "hidden" }),
            MessageLevel::Info,
        );
        Ok(())
    }

    pub fn toggle_sort(&mut self, sort_by: SortBy) -> Result<()> {
        if self.config.sort_by == sort_by {
            // Toggle order if same sort
            self.config.sort_order = match self.config.sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::Ascending,
            };
        } else {
            self.config.sort_by = sort_by;
            self.config.sort_order = SortOrder::Ascending;
        }
        self.refresh_file_list()?;
        self.set_message(
            format!("Sort by: {:?} ({:?})", self.config.sort_by, self.config.sort_order),
            MessageLevel::Info,
        );
        Ok(())
    }

    pub fn set_message(&mut self, message: impl Into<String>, level: MessageLevel) {
        self.message = Some(message.into());
        self.message_level = level;
    }

    pub fn clear_message(&mut self) {
        self.message = None;
        self.message_level = MessageLevel::Info;
    }

    pub fn is_empty_list(&self) -> bool {
        self.displayed_indices.is_empty()
    }

    pub fn get_selected_file(&self) -> Option<&FileEntry> {
        if self.is_empty_list() {
            return None;
        }
        self.displayed_indices
            .get(self.selected_index)
            .map(|&i| &self.all_files[i])
    }

    pub fn ensure_visible(&mut self) {
        const LIST_HEIGHT: usize = 20; // Will be adjusted by UI

        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + LIST_HEIGHT {
            self.scroll_offset = self.selected_index - LIST_HEIGHT + 1;
        }
    }

    fn clear_old_selections(&mut self) {
        let current_paths: HashSet<PathBuf> = self.all_files
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

    fn update_preview(&mut self) -> Result<()> {
        let selected = self.get_selected_file().cloned();
        if let Some(file) = selected {
            if !file.is_dir {
                self.preview_file = Some(file.path.clone());
                self.preview_content = self.load_file_preview(&file.path)?;
            } else {
                self.preview_file = None;
                self.preview_content = None;
            }
        } else {
            self.preview_file = None;
            self.preview_content = None;
        }
        Ok(())
    }

    fn load_file_preview(&self, path: &PathBuf) -> Result<Option<String>> {
        use std::fs::File;
        use std::io::Read;

        // Try to read as text (first 500 lines, max 50KB)
        match File::open(path) {
            Ok(mut file) => {
                let mut buffer = Vec::with_capacity(50 * 1024);
                if file.read_to_end(&mut buffer).is_ok() {
                    // Try to decode as UTF-8
                    match String::from_utf8_lossy(&buffer[..buffer.len().min(50 * 1024)]).to_string() {
                        content if content.len() < 50 * 1024 => {
                            // Get first 500 lines
                            let lines: Vec<&str> = content.lines().take(500).collect();
                            Ok(Some(lines.join("\n")))
                        }
                        _ => Ok(Some("[Binary or too large file]".to_string())),
                    }
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }

    pub fn toggle_preview(&mut self) {
        self.config.show_preview = !self.config.show_preview;
        if !self.config.show_preview {
            self.preview_content = None;
            self.preview_file = None;
        } else {
            let _ = self.update_preview();
        }
        self.set_message(
            format!("Preview: {}", if self.config.show_preview { "on" } else { "off" }),
            MessageLevel::Info,
        );
    }

    pub fn start_search(&mut self) {
        self.mode = Mode::Search;
        self.search_query = Some(String::new());
        self.command_input = String::new();
    }

    pub fn update_search(&mut self) {
        self.search_query = if self.command_input.is_empty() {
            None
        } else {
            Some(self.command_input.clone())
        };

        // Re-filter the display
        self.displayed_indices = crate::state::files::filter_files(
            &self.all_files,
            self.config.show_hidden,
            self.search_query.as_deref(),
        );

        if !self.displayed_indices.is_empty() {
            self.selected_index = 0;
            self.scroll_offset = 0;
        }
    }

    pub fn exit_search(&mut self) {
        self.mode = Mode::Normal;
        self.search_query = None;
        self.command_input.clear();
        self.refresh_file_list().ok();
    }

    pub fn next_search_match(&mut self) {
        if self.search_query.is_some() && !self.displayed_indices.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.displayed_indices.len();
            self.ensure_visible();
        }
    }

    pub fn prev_search_match(&mut self) {
        if self.search_query.is_some() && !self.displayed_indices.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.displayed_indices.len() - 1
            } else {
                self.selected_index - 1
            };
            self.ensure_visible();
        }
    }

    fn load_config() -> Result<Config> {
        // Try to load from config file
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            return serde_json::from_str(&content)
                .context("Failed to parse config file");
        }

        Ok(Config::default())
    }

    pub fn save_config(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let content = serde_json::to_string_pretty(&self.config)
            .context("Failed to serialize config")?;
        std::fs::write(&config_path, content)
            .context("Failed to write config file")?;

        Ok(())
    }

    #[cfg(windows)]
    fn config_path() -> Result<PathBuf> {
        let appdata = std::env::var("APPDATA")
            .context("Failed to get APPDATA environment variable")?;
        Ok(PathBuf::from(appdata).join("filebrowser-tui").join("config.json"))
    }

    #[cfg(not(windows))]
    fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .context("Failed to get HOME environment variable")?;
        Ok(PathBuf::from(home).join(".config").join("filebrowser-tui").join("config.json"))
    }

    /// Undo the most recent file operation
    pub fn undo(&mut self) -> Result<()> {
        match self.history.undo() {
            Ok(op) => {
                self.refresh_file_list()?;
                self.set_message(
                    format!("Undo: {}", op.description()),
                    MessageLevel::Success,
                );
                Ok(())
            }
            Err(e) => {
                self.set_message(
                    format!("Undo failed: {}", e),
                    MessageLevel::Error,
                );
                Err(e)
            }
        }
    }

    /// Redo the most recently undone operation
    pub fn redo(&mut self) -> Result<()> {
        match self.history.redo() {
            Ok(op) => {
                self.refresh_file_list()?;
                self.set_message(
                    format!("Redo: {}", op.description()),
                    MessageLevel::Success,
                );
                Ok(())
            }
            Err(e) => {
                self.set_message(
                    format!("Redo failed: {}", e),
                    MessageLevel::Error,
                );
                Err(e)
            }
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Get description of next undo operation
    pub fn peek_undo(&self) -> Option<String> {
        self.history.peek_undo()
    }

    /// Get description of next redo operation
    pub fn peek_redo(&self) -> Option<String> {
        self.history.peek_redo()
    }
}

    // Content search methods

    /// Start content search mode
    pub fn start_content_search(&mut self) {
        self.mode = Mode::ContentSearch;
        self.content_search_query = Some(String::new());
        self.command_input = String::new();
        self.content_search_results.clear();
        self.content_search_selected_index = 0;
        self.content_search_scroll_offset = 0;
        self.content_search_in_progress = false;
        self.content_search_task = None;
    }

    /// Cancel the ongoing content search
    pub fn cancel_content_search(&mut self) {
        self.content_search_in_progress = false;
        self.content_search_task = None;
        self.content_search_results.clear();
        self.content_search_query = None;
        self.mode = Mode::Normal;
        self.set_message("Content search cancelled", MessageLevel::Info);
    }

    /// Exit content search mode and return to normal
    pub fn exit_content_search(&mut self) {
        self.mode = Mode::Normal;
        self.content_search_query = None;
        self.command_input.clear();
        self.content_search_in_progress = false;
        self.content_search_task = None;
    }

    /// Execute the content search with the given query
    pub fn execute_content_search(&mut self) {
        let query = self.command_input.clone();
        if query.is_empty() {
            self.set_message("Search query cannot be empty", MessageLevel::Warning);
            return;
        }

        self.content_search_query = Some(query.clone());
        self.content_search_results.clear();
        self.content_search_selected_index = 0;
        self.content_search_scroll_offset = 0;
        self.content_search_in_progress = true;

        // Collect files to search from current directory
        let current_path = self.current_path().clone();

        // Spawn async search task
        let task = tokio::spawn(async move {
            crate::file_ops::collect_files_in_directory(&current_path, true, None)
                .await
        });

        // Store the task handle
        self.content_search_task = Some(Arc::new(Mutex::new(Some(task))));

        self.set_message(
            format!("Searching for '{}'...", query),
            MessageLevel::Info,
        );
    }

    /// Update content search results (called from async task completion)
    pub fn update_content_search_results(&mut self, results: Vec<SearchResult>) {
        self.content_search_results = results;
        self.content_search_in_progress = false;
        self.content_search_task = None;

        let count = self.content_search_results.len();
        if count > 0 {
            self.set_message(
                format!("Found {} match(es)", count),
                MessageLevel::Success,
            );
        } else {
            self.set_message("No matches found", MessageLevel::Info);
        }
    }

    /// Navigate to the next content search result
    pub fn next_content_search_result(&mut self) {
        if self.content_search_results.is_empty() {
            return;
        }

        self.content_search_selected_index = (self.content_search_selected_index + 1) % self.content_search_results.len();
        self.ensure_content_search_visible();
    }

    /// Navigate to the previous content search result
    pub fn prev_content_search_result(&mut self) {
        if self.content_search_results.is_empty() {
            return;
        }

        self.content_search_selected_index = if self.content_search_selected_index == 0 {
            self.content_search_results.len() - 1
        } else {
            self.content_search_selected_index - 1
        };
        self.ensure_content_search_visible();
    }

    /// Get the currently selected content search result
    pub fn get_selected_content_search_result(&self) -> Option<&SearchResult> {
        self.content_search_results.get(self.content_search_selected_index)
    }

    /// Ensure the selected result is visible in the search results panel
    fn ensure_content_search_visible(&mut self) {
        const RESULTS_HEIGHT: usize = 15; // Will be adjusted by UI

        if self.content_search_selected_index < self.content_search_scroll_offset {
            self.content_search_scroll_offset = self.content_search_selected_index;
        } else if self.content_search_selected_index >= self.content_search_scroll_offset + RESULTS_HEIGHT {
            self.content_search_scroll_offset = self.content_search_selected_index - RESULTS_HEIGHT + 1;
        }
    }

    /// Open the file at the selected content search result
    pub fn open_content_search_result(&mut self) -> Result<()> {
        if let Some(result) = self.get_selected_content_search_result() {
            // Navigate to the file's directory and select the file
            let file_path = result.file_path.clone();
            if let Some(parent_dir) = file_path.parent() {
                self.change_directory(parent_dir.to_path_buf())?;

                // Try to select the file in the listing
                if let Some(file_name) = file_path.file_name() {
                    let file_name_str = file_name.to_string_lossy().to_string();
                    if let Some(idx) = self
                        .displayed_indices
                        .iter()
                        .position(|&i| self.all_files[i].name == file_name_str)
                    {
                        self.selected_index = idx;
                        self.ensure_visible();
                    }
                }

                self.set_message(
                    format!("Opened: {}", file_name.unwrap_or_default().to_string_lossy()),
                    MessageLevel::Success,
                );
            }
        }
        Ok(())
    }

    // Content search methods

    /// Start content search mode
    pub fn start_content_search(&mut self) {
        self.mode = Mode::ContentSearch;
        self.content_search_query = Some(String::new());
        self.command_input = String::new();
        self.content_search_results.clear();
        self.content_search_selected_index = 0;
        self.content_search_scroll_offset = 0;
        self.content_search_in_progress = false;
        self.content_search_task = None;
    }

    /// Cancel the ongoing content search
    pub fn cancel_content_search(&mut self) {
        self.content_search_in_progress = false;
        self.content_search_task = None;
        self.content_search_results.clear();
        self.content_search_query = None;
        self.mode = Mode::Normal;
        self.set_message("Content search cancelled", MessageLevel::Info);
    }

    /// Exit content search mode and return to normal
    pub fn exit_content_search(&mut self) {
        self.mode = Mode::Normal;
        self.content_search_query = None;
        self.command_input.clear();
        self.content_search_in_progress = false;
        self.content_search_task = None;
    }

    /// Execute the content search with the given query
    pub fn execute_content_search(&mut self) {
        let query = self.command_input.clone();
        if query.is_empty() {
            self.set_message("Search query cannot be empty", MessageLevel::Warning);
            return;
        }

        self.content_search_query = Some(query.clone());
        self.content_search_results.clear();
        self.content_search_selected_index = 0;
        self.content_search_scroll_offset = 0;
        self.content_search_in_progress = true;

        // Collect files to search from current directory
        let current_path = self.current_path().clone();

        // Spawn async search task
        let task = tokio::spawn(async move {
            crate::file_ops::collect_files_in_directory(&current_path, true, None)
                .await
        });

        // Store the task handle
        self.content_search_task = Some(Arc::new(Mutex::new(Some(task))));

        self.set_message(
            format!("Searching for '{}'...", query),
            MessageLevel::Info,
        );
    }

    /// Update content search results (called from async task completion)
    pub fn update_content_search_results(&mut self, results: Vec<SearchResult>) {
        self.content_search_results = results;
        self.content_search_in_progress = false;
        self.content_search_task = None;

        let count = self.content_search_results.len();
        if count > 0 {
            self.set_message(
                format!("Found {} match(es)", count),
                MessageLevel::Success,
            );
        } else {
            self.set_message("No matches found", MessageLevel::Info);
        }
    }

    /// Navigate to the next content search result
    pub fn next_content_search_result(&mut self) {
        if self.content_search_results.is_empty() {
            return;
        }

        self.content_search_selected_index = (self.content_search_selected_index + 1) % self.content_search_results.len();
        self.ensure_content_search_visible();
    }

    /// Navigate to the previous content search result
    pub fn prev_content_search_result(&mut self) {
        if self.content_search_results.is_empty() {
            return;
        }

        self.content_search_selected_index = if self.content_search_selected_index == 0 {
            self.content_search_results.len() - 1
        } else {
            self.content_search_selected_index - 1
        };
        self.ensure_content_search_visible();
    }

    /// Get the currently selected content search result
    pub fn get_selected_content_search_result(&self) -> Option<&SearchResult> {
        self.content_search_results.get(self.content_search_selected_index)
    }

    /// Ensure the selected result is visible in the search results panel
    fn ensure_content_search_visible(&mut self) {
        const RESULTS_HEIGHT: usize = 15; // Will be adjusted by UI

        if self.content_search_selected_index < self.content_search_scroll_offset {
            self.content_search_scroll_offset = self.content_search_selected_index;
        } else if self.content_search_selected_index >= self.content_search_scroll_offset + RESULTS_HEIGHT {
            self.content_search_scroll_offset = self.content_search_selected_index - RESULTS_HEIGHT + 1;
        }
    }

    /// Open the file at the selected content search result
    pub fn open_content_search_result(&mut self) -> Result<()> {
        if let Some(result) = self.get_selected_content_search_result() {
            // Navigate to the file's directory and select the file
            let file_path = result.file_path.clone();
            if let Some(parent_dir) = file_path.parent() {
                self.change_directory(parent_dir.to_path_buf())?;

                // Try to select the file in the listing
                if let Some(file_name) = file_path.file_name() {
                    let file_name_str = file_name.to_string_lossy().to_string();
                    if let Some(idx) = self
                        .displayed_indices
                        .iter()
                        .position(|&i| self.all_files[i].name == file_name_str)
                    {
                        self.selected_index = idx;
                        self.ensure_visible();
                    }
                }

                self.set_message(
                    format!("Opened: {}", file_name.unwrap_or_default().to_string_lossy()),
                    MessageLevel::Success,
                );
            }
        }
        Ok(())
    }

    // Bulk rename methods

    /// Start bulk rename mode with selected or all files
    pub fn start_bulk_rename(&mut self) {
        let files_to_rename: Vec<PathBuf> = if !self.selection.is_empty() {
            self.selection.get_selected()
        } else if let Some(file) = self.get_selected_file() {
            vec![file.path.clone()]
        } else {
            self.set_message("No files selected", MessageLevel::Warning);
            return;
        };

        if files_to_rename.is_empty() {
            self.set_message("No files to rename", MessageLevel::Warning);
            return;
        }

        self.mode = Mode::BulkRename;
        self.rename_previews = Vec::new();
        self.rename_selected_index = 0;
        self.rename_pattern_input = String::new();
        self.command_input.clear();

        self.set_message(
            format!("Bulk rename: {} file(s). Enter pattern.", files_to_rename.len()),
            MessageLevel::Info,
        );
    }

    /// Update rename preview based on current pattern input
    pub fn update_rename_preview(&mut self) -> Result<()> {
        let pattern = self.parse_rename_pattern()?;
        let files_to_rename: Vec<PathBuf> = if !self.selection.is_empty() {
            self.selection.get_selected()
        } else if let Some(file) = self.get_selected_file() {
            vec![file.path.clone()]
        } else {
            return Ok(());
        };

        let renamer = crate::file_ops::BulkRenamer::new(pattern, self.current_path().clone());
        self.rename_previews = renamer.preview(&files_to_rename);

        Ok(())
    }

    /// Parse the pattern input into a RenamePattern
    fn parse_rename_pattern(&self) -> Result<RenamePattern> {
        let input = self.rename_pattern_input.trim();

        if input.is_empty() {
            return Err(anyhow::anyhow!("Empty pattern"));
        }

        // Detect pattern type
        if input.starts_with("s/") || input.starts_with("regex/") {
            // Regex pattern: s/find/replace/ or regex/find/replace/
            let parts: Vec<&str> = input.split('/').collect();
            if parts.len() >= 3 {
                let pattern = parts[1].to_string();
                let replacement = if parts.len() > 3 {
                    parts[2..].join("/")
                } else {
                    parts[2].to_string()
                };
                return Ok(RenamePattern::Regex { pattern, replacement });
            }
        } else if input.contains("{n}") {
            // Numbered pattern
            let (template, rest) = input.split_once(',').unwrap_or((input, "1,0"));
            let params: Vec<&str> = rest.trim().split(',').collect();
            let start = params.get(0).and_then(|s| s.parse().ok()).unwrap_or(1);
            let pad_width = params.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            return Ok(RenamePattern::Numbered {
                template: template.to_string(),
                start,
                pad_width,
            });
        } else if input.starts_with("case:") {
            // Case transformation
            let rest = &input[5..];
            let (transform, scope) = if let Some(idx) = rest.find(',') {
                let transform_str = &rest[..idx];
                let scope_str = rest[idx + 1..].trim();
                (
                    parse_case_transform(transform_str)?,
                    parse_case_scope(scope_str)?,
                )
            } else {
                (parse_case_transform(rest)?, crate::file_ops::CaseScope::EntireName)
            };
            return Ok(RenamePattern::Case { transform, scope });
        } else if input.starts_with("ext:") {
            // Extension manipulation
            let rest = &input[4..];
            let (action, ext) = if let Some(idx) = rest.find(',') {
                let action_str = &rest[..idx];
                let ext_str = rest[idx + 1..].trim();
                (
                    parse_extension_action(action_str)?,
                    if ext_str.is_empty() { None } else { Some(ext_str.to_string()) },
                )
            } else {
                return Err(anyhow::anyhow!("Invalid extension pattern"));
            };
            return Ok(RenamePattern::Extension {
                action,
                new_extension: ext,
            });
        } else if let Some(idx) = input.find(',') {
            // Simple replace: find,replace
            let find = input[..idx].to_string();
            let replace = input[idx + 1..].to_string();
            return Ok(RenamePattern::SimpleReplace { find, replace });
        } else {
            // Just a find string, replace with empty
            return Ok(RenamePattern::SimpleReplace {
                find: input.to_string(),
                replace: String::new(),
            });
        }

        Err(anyhow::anyhow!("Invalid pattern format"))
    }

    /// Execute the bulk rename operation
    pub fn execute_bulk_rename(&mut self) -> Result<()> {
        if self.rename_previews.is_empty() {
            self.set_message("No renames to execute", MessageLevel::Warning);
            return Ok(());
        }

        let pattern = self.rename_pattern.take().ok_or_else(|| anyhow::anyhow!("No pattern set"))?;
        let renamer = crate::file_ops::BulkRenamer::new(pattern, self.current_path().clone());

        let mut previews = self.rename_previews.clone();
        let renamed = renamer.execute(&mut previews)?;

        // Update the previews with the results
        self.rename_previews = previews;

        // Refresh file list and exit rename mode
        self.refresh_file_list()?;
        self.mode = Mode::Normal;
        self.rename_previews.clear();
        self.rename_pattern_input.clear();

        self.set_message(
            format!("Renamed {} file(s)", renamed),
            MessageLevel::Success,
        );

        Ok(())
    }

    /// Cancel bulk rename mode
    pub fn cancel_bulk_rename(&mut self) {
        self.mode = Mode::Normal;
        self.rename_previews.clear();
        self.rename_pattern = None;
        self.rename_pattern_input.clear();
        self.set_message("Bulk rename cancelled", MessageLevel::Info);
    }

    /// Toggle acceptance of the currently selected rename preview
    pub fn toggle_rename_acceptance(&mut self) {
        if let Some(preview) = self.rename_previews.get_mut(self.rename_selected_index) {
            preview.accepted = !preview.accepted;
        }
    }

    /// Navigate in the rename preview list
    pub fn move_rename_selection(&mut self, delta: isize) {
        if self.rename_previews.is_empty() {
            return;
        }

        let new_index = if delta >= 0 {
            self.rename_selected_index + delta as usize
        } else {
            self.rename_selected_index.saturating_sub((-delta) as usize)
        };

        self.rename_selected_index = new_index.min(self.rename_previews.len() - 1);
    }
}

/// Parse case transformation string
fn parse_case_transform(s: &str) -> Result<crate::file_ops::CaseTransform> {
    match s.trim().to_lowercase().as_str() {
        "upper" | "uppercase" => Ok(crate::file_ops::CaseTransform::Uppercase),
        "lower" | "lowercase" => Ok(crate::file_ops::CaseTransform::Lowercase),
        "title" | "titlecase" => Ok(crate::file_ops::CaseTransform::TitleCase),
        "sentence" | "sentencecase" => Ok(crate::file_ops::CaseTransform::SentenceCase),
        "toggle" | "togglecase" => Ok(crate::file_ops::CaseTransform::ToggleCase),
        _ => Err(anyhow::anyhow!("Invalid case transform: {}", s)),
    }
}

/// Parse case scope string
fn parse_case_scope(s: &str) -> Result<crate::file_ops::CaseScope> {
    match s.trim().to_lowercase().as_str() {
        "name" => Ok(crate::file_ops::CaseScope::NameOnly),
        "ext" | "extension" => Ok(crate::file_ops::CaseScope::ExtensionOnly),
        "all" | "entire" => Ok(crate::file_ops::CaseScope::EntireName),
        _ => Err(anyhow::anyhow!("Invalid case scope: {}", s)),
    }
}

/// Parse extension action string
fn parse_extension_action(s: &str) -> Result<crate::file_ops::ExtensionAction> {
    match s.trim().to_lowercase().as_str() {
        "add" => Ok(crate::file_ops::ExtensionAction::Add),
        "remove" | "rm" => Ok(crate::file_ops::ExtensionAction::Remove),
        "replace" => Ok(crate::file_ops::ExtensionAction::Replace),
        "keep" => Ok(crate::file_ops::ExtensionAction::Keep),
        _ => Err(anyhow::anyhow!("Invalid extension action: {}", s)),
    }
}
