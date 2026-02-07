use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: SystemTime,
    pub is_hidden: bool,
    pub is_system: bool,
    pub is_readonly: bool,
    pub is_symlink: bool,
}

impl FileEntry {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let metadata = fs::metadata(&path).ok();
        let symlink_metadata = fs::symlink_metadata(&path).ok();

        let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let is_symlink = symlink_metadata
            .as_ref()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);

        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        // Windows-specific attributes
        let is_hidden = is_hidden_windows(&path);
        let is_system = is_system_file_windows(&path);
        let is_readonly = metadata
            .as_ref()
            .map(|m| m.permissions().readonly())
            .unwrap_or(false);

        Ok(FileEntry {
            name,
            path,
            is_dir,
            size,
            modified,
            is_hidden,
            is_system,
            is_readonly,
            is_symlink,
        })
    }

    pub fn display_size(&self) -> String {
        if self.is_dir {
            return "<DIR>".to_string();
        }

        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size < KB {
            format!("{} B", self.size)
        } else if self.size < MB {
            format!("{:.1} KB", self.size as f64 / KB as f64)
        } else if self.size < GB {
            format!("{:.1} MB", self.size as f64 / MB as f64)
        } else {
            format!("{:.2} GB", self.size as f64 / GB as f64)
        }
    }

    pub fn display_modified(&self) -> String {
        use std::time::UNIX_EPOCH;

        let duration = self
            .modified
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        let datetime = chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
            .unwrap_or_default();

        datetime.format("%Y-%m-%d %H:%M").to_string()
    }

    pub fn extension(&self) -> Option<&str> {
        self.path.extension().and_then(|e| e.to_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortBy {
    Name,
    Size,
    Modified,
    Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

// Windows-specific hidden file check
#[cfg(windows)]
fn is_hidden_windows(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;

    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.starts_with('.') {
            return true;
        }
    }

    fs::metadata(path)
        .map(|m| (m.file_attributes() & 0x2) != 0) // FILE_ATTRIBUTE_HIDDEN
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn is_hidden_windows(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}

// Windows-specific system file check
#[cfg(windows)]
fn is_system_file_windows(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;

    fs::metadata(path)
        .map(|m| (m.file_attributes() & 0x4) != 0) // FILE_ATTRIBUTE_SYSTEM
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn is_system_file_windows(_path: &Path) -> bool {
    false
}

pub fn sort_files(files: &mut Vec<FileEntry>, sort_by: SortBy, order: SortOrder) {
    match sort_by {
        SortBy::Name => {
            files.sort_by(|a, b| {
                // Directories first
                if a.is_dir != b.is_dir {
                    return b.is_dir.cmp(&a.is_dir);
                }
                match order {
                    SortOrder::Ascending => {
                        a.name.to_lowercase().cmp(&b.name.to_lowercase())
                    }
                    SortOrder::Descending => {
                        b.name.to_lowercase().cmp(&a.name.to_lowercase())
                    }
                }
            });
        }
        SortBy::Size => {
            files.sort_by(|a, b| {
                if a.is_dir != b.is_dir {
                    return b.is_dir.cmp(&a.is_dir);
                }
                match order {
                    SortOrder::Ascending => a.size.cmp(&b.size),
                    SortOrder::Descending => b.size.cmp(&a.size),
                }
            });
        }
        SortBy::Modified => {
            files.sort_by(|a, b| {
                if a.is_dir != b.is_dir {
                    return b.is_dir.cmp(&a.is_dir);
                }
                match order {
                    SortOrder::Ascending => a.modified.cmp(&b.modified),
                    SortOrder::Descending => b.modified.cmp(&a.modified),
                }
            });
        }
        SortBy::Type => {
            files.sort_by(|a, b| {
                if a.is_dir != b.is_dir {
                    return b.is_dir.cmp(&a.is_dir);
                }
                match order {
                    SortOrder::Ascending => {
                        a.extension().cmp(&b.extension())
                    }
                    SortOrder::Descending => {
                        b.extension().cmp(&a.extension())
                    }
                }
            });
        }
    }
}

pub fn filter_files(
    files: &[FileEntry],
    show_hidden: bool,
    search_query: Option<&str>,
) -> Vec<usize> {
    let mut indices: Vec<usize> = Vec::new();

    for (i, file) in files.iter().enumerate() {
        // Filter hidden files
        if !show_hidden && (file.is_hidden || file.is_system) {
            continue;
        }

        // Filter by search query
        if let Some(query) = search_query {
            if !file.name.to_lowercase().contains(&query.to_lowercase()) {
                continue;
            }
        }

        indices.push(i);
    }

    indices
}

#[cfg(test)]
mod tests;
