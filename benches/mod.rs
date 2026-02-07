//! Benchmark utilities and fixtures for filebrowser-tui performance tests.
//!
//! This module provides common setup/teardown functionality and test fixtures
//! for benchmarking various operations in the file browser.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tempfile::TempDir;

/// Test fixture manager for creating controlled test environments
pub struct FixtureBuilder {
    temp_dir: TempDir,
    base_path: PathBuf,
}

impl FixtureBuilder {
    /// Create a new fixture builder with a temporary directory
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path().to_path_buf();
        Ok(Self { temp_dir, base_path })
    }

    /// Get the base path for the fixture
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Create a directory with a specified number of files
    pub fn create_directory_with_files(
        &self,
        dir_name: &str,
        file_count: usize,
        include_hidden: bool,
    ) -> anyhow::Result<PathBuf> {
        let dir_path = self.base_path.join(dir_name);
        fs::create_dir_all(&dir_path)?;

        let hidden_ratio = if include_hidden { 10 } else { 0 };

        for i in 0..file_count {
            let is_hidden = include_hidden && i % hidden_ratio == 0;
            let file_name = if is_hidden {
                format!(".hidden_file_{}.txt", i)
            } else {
                format!("file_{}.txt", i)
            };

            let file_path = dir_path.join(&file_name);
            let mut file = File::create(&file_path)?;
            writeln!(file, "Test file content for file {}", i)?;
        }

        Ok(dir_path)
    }

    /// Create a nested directory structure
    pub fn create_nested_structure(
        &self,
        base_name: &str,
        depth: usize,
        files_per_dir: usize,
    ) -> anyhow::Result<PathBuf> {
        let base_path = self.base_path.join(base_name);
        self.create_nested_level(&base_path, depth, files_per_dir)?;
        Ok(base_path)
    }

    fn create_nested_level(
        &self,
        path: &Path,
        remaining_depth: usize,
        files_per_dir: usize,
    ) -> anyhow::Result<()> {
        fs::create_dir_all(path)?;

        // Create files at this level
        for i in 0..files_per_dir {
            let file_path = path.join(format!("file_{}.txt", i));
            let mut file = File::create(&file_path)?;
            writeln!(file, "Content at level {}", remaining_depth)?;
        }

        // Create subdirectories if we haven't reached max depth
        if remaining_depth > 0 {
            for i in 0..3 {
                let sub_path = path.join(format!("subdir_{}", i));
                self.create_nested_level(&sub_path, remaining_depth - 1, files_per_dir)?;
            }
        }

        Ok(())
    }

    /// Create files of varying sizes for copy benchmarking
    pub fn create_sized_files(&self, dir_name: &str) -> anyhow::Result<PathBuf> {
        let dir_path = self.base_path.join(dir_name);
        fs::create_dir_all(&dir_path)?;

        // Small file (1KB)
        let small_path = dir_path.join("small.txt");
        let mut small_file = File::create(&small_path)?;
        let small_content = "x".repeat(1024);
        small_file.write_all(small_content.as_bytes())?;

        // Medium file (1MB)
        let medium_path = dir_path.join("medium.txt");
        let mut medium_file = File::create(&medium_path)?;
        let medium_content = "y".repeat(1024 * 1024);
        medium_file.write_all(medium_content.as_bytes())?;

        // Large file (10MB)
        let large_path = dir_path.join("large.txt");
        let mut large_file = File::create(&large_path)?;
        let large_content = b"z".repeat(10 * 1024 * 1024);
        large_file.write_all(&large_content)?;

        Ok(dir_path)
    }

    /// Create files with searchable content
    pub fn create_searchable_files(
        &self,
        dir_name: &str,
        file_count: usize,
        lines_per_file: usize,
    ) -> anyhow::Result<PathBuf> {
        let dir_path = self.base_path.join(dir_name);
        fs::create_dir_all(&dir_path)?;

        let search_terms = vec!["benchmark", "performance", "test", "optimize", "speed"];

        for i in 0..file_count {
            let file_path = dir_path.join(format!("search_{}.txt", i));
            let mut file = File::create(&file_path)?;

            for j in 0..lines_per_file {
                let term = search_terms[j % search_terms.len()];
                writeln!(file, "Line {}: This is a {} with content", j, term)?;
            }
        }

        Ok(dir_path)
    }

    /// Create a large file for content search testing
    pub fn create_large_search_file(&self, file_name: &str, line_count: usize) -> anyhow::Result<PathBuf> {
        let file_path = self.base_path.join(file_name);
        let mut file = File::create(&file_path)?;

        for i in 0..line_count {
            writeln!(file, "Line {}: Searching for benchmark patterns in large files", i)?;
        }

        Ok(file_path)
    }

    /// Consume the fixture and return the TempDir to prevent cleanup
    /// (useful for debugging)
    #[allow(dead_code)]
    pub fn into_temp_dir(self) -> TempDir {
        self.temp_dir
    }

    /// Get the underlying TempDir reference
    pub fn temp_dir(&self) -> &TempDir {
        &self.temp_dir
    }
}

impl Default for FixtureBuilder {
    fn default() -> Self {
        Self::new().expect("Failed to create FixtureBuilder")
    }
}

/// Create a vector of mock FileEntry objects for testing
pub fn create_mock_file_entries(count: usize, include_hidden: bool) -> Vec<filebrowser_tui::state::FileEntry> {
    let mut entries = Vec::with_capacity(count);

    let hidden_ratio = if include_hidden { 10 } else { 0 };

    for i in 0..count {
        let is_hidden = include_hidden && i % hidden_ratio == 0;
        let file_name = if is_hidden {
            format!(".hidden_{}.txt", i)
        } else {
            format!("file_{}.txt", i)
        };

        entries.push(filebrowser_tui::state::FileEntry {
            name: file_name.clone(),
            path: PathBuf::from(format!("/tmp/test/{}", file_name)),
            is_dir: i % 20 == 0, // Every 20th item is a directory
            size: (i * 1024) as u64,
            modified: SystemTime::UNIX_EPOCH,
            is_hidden,
            is_system: false,
            is_readonly: false,
            is_symlink: false,
        });
    }

    entries
}

/// Create mock FileEntry objects with specific sort characteristics
pub fn create_sortable_entries() -> Vec<filebrowser_tui::state::FileEntry> {
    vec![
        filebrowser_tui::state::FileEntry {
            name: "zebra.txt".to_string(),
            path: PathBuf::from("/tmp/zebra.txt"),
            is_dir: false,
            size: 9999,
            modified: SystemTime::UNIX_EPOCH,
            is_hidden: false,
            is_system: false,
            is_readonly: false,
            is_symlink: false,
        },
        filebrowser_tui::state::FileEntry {
            name: "apple.txt".to_string(),
            path: PathBuf::from("/tmp/apple.txt"),
            is_dir: false,
            size: 100,
            modified: SystemTime::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(3600)).unwrap(),
            is_hidden: false,
            is_system: false,
            is_readonly: false,
            is_symlink: false,
        },
        filebrowser_tui::state::FileEntry {
            name: "middle.txt".to_string(),
            path: PathBuf::from("/tmp/middle.txt"),
            is_dir: true,
            size: 5000,
            modified: SystemTime::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(1800)).unwrap(),
            is_hidden: false,
            is_system: false,
            is_readonly: false,
            is_symlink: false,
        },
        filebrowser_tui::state::FileEntry {
            name: "first.rs".to_string(),
            path: PathBuf::from("/tmp/first.rs"),
            is_dir: false,
            size: 50,
            modified: SystemTime::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(7200)).unwrap(),
            is_hidden: false,
            is_system: false,
            is_readonly: false,
            is_symlink: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_builder_creation() {
        let fixture = FixtureBuilder::new().unwrap();
        assert!(fixture.base_path().exists());
    }

    #[test]
    fn test_create_directory_with_files() {
        let fixture = FixtureBuilder::new().unwrap();
        let dir_path = fixture
            .create_directory_with_files("test_dir", 10, false)
            .unwrap();

        assert!(dir_path.exists());
        assert_eq!(dir_path.read_dir().unwrap().count(), 10);
    }

    #[test]
    fn test_create_directory_with_hidden_files() {
        let fixture = FixtureBuilder::new().unwrap();
        let dir_path = fixture
            .create_directory_with_files("test_hidden", 20, true)
            .unwrap();

        let entries: Vec<_> = dir_path.read_dir().unwrap().collect();
        assert_eq!(entries.len(), 20);

        // Check that we have hidden files (starting with '.')
        let hidden_count = entries
            .iter()
            .filter(|e| {
                e.as_ref()
                    .ok()
                    .and_then(|e| e.file_name().to_str())
                    .map(|n| n.starts_with('.'))
                    .unwrap_or(false)
            })
            .count();
        assert!(hidden_count > 0);
    }

    #[test]
    fn test_mock_entries_creation() {
        let entries = create_mock_file_entries(100, true);
        assert_eq!(entries.len(), 100);

        let hidden_count = entries.iter().filter(|e| e.is_hidden).count();
        assert!(hidden_count > 0);
    }
}
