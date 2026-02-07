use std::path::PathBuf;

/// Represents a single tab in the file browser
#[derive(Debug, Clone)]
pub struct Tab {
    /// Current path of this tab
    pub path: PathBuf,
    /// Currently selected file index in this tab
    pub selected_index: usize,
    /// Current scroll offset in this tab
    pub scroll_offset: usize,
    /// Display name for this tab
    pub display_name: String,
}

impl Tab {
    /// Create a new tab with the given path and display name
    pub fn new(path: PathBuf, display_name: String) -> Self {
        Tab {
            path,
            selected_index: 0,
            scroll_offset: 0,
            display_name,
        }
    }

    /// Create a new tab with a default display name based on the path
    pub fn from_path(path: PathBuf) -> Self {
        let display_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("/")
            .to_string();

        Self::new(path, display_name)
    }

    /// Update the display name based on the current path
    pub fn update_display_name(&mut self) {
        self.display_name = self
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("/")
            .to_string();
    }
}
