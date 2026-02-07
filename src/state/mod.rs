mod app_state;
mod bookmarks;
mod files;
mod pane;
mod selection;
mod tab;

pub use app_state::{App, ConfirmDialog, MessageLevel, Mode};
pub use bookmarks::{Bookmark, Bookmarks};
pub use files::{FileEntry, SortBy, SortOrder};
pub use pane::{ActivePane, Pane};
pub use selection::SelectionManager;
pub use tab::Tab;

// Re-export RenamePreview from file_ops for convenience
pub use crate::file_ops::RenamePreview;

// Test modules
#[cfg(test)]
mod tests {
    // Integration tests that work with temp directories
    use super::*;
    use tempfile::TempDir;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::SystemTime;

    /// Helper to create a test file with content
    pub fn create_test_file(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    /// Helper to create a test directory
    pub fn create_test_dir(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        fs::create_dir_all(&path).unwrap();
        path
    }

    /// Helper to get a FileEntry for a path
    pub fn get_file_entry(path: PathBuf) -> FileEntry {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let metadata = fs::metadata(&path).ok();
        let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        FileEntry {
            name,
            path,
            is_dir,
            size,
            modified,
            is_hidden: false,
            is_system: false,
            is_readonly: false,
            is_symlink: false,
        }
    }
}
