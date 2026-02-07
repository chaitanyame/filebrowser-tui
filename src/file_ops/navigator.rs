use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::state::FileEntry;

pub struct Navigator;

impl Navigator {
    pub fn new() -> Self {
        Navigator
    }

    pub fn read_directory(&self, path: &PathBuf) -> Result<Vec<FileEntry>> {
        let entries = fs::read_dir(path)
            .context(format!("Failed to read directory: {}", path.display()))?;

        let mut files = Vec::new();

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let file_path = entry.path();

            // Skip files we can't access
            if let Ok(file_entry) = FileEntry::from_path(file_path) {
                files.push(file_entry);
            }
        }

        Ok(files)
    }

    pub fn path_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    pub fn is_directory(&self, path: &Path) -> bool {
        path.is_dir()
    }

    pub fn get_parent(path: &PathBuf) -> Option<PathBuf> {
        path.parent().map(|p| p.to_path_buf())
    }

    pub fn resolve_path(&self, path: &Path) -> Result<PathBuf> {
        path.canonicalize()
            .context(format!("Failed to resolve path: {}", path.display()))
    }
}

impl Default for Navigator {
    fn default() -> Self {
        Self::new()
    }
}
