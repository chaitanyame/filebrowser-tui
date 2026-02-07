//! Test utilities and helpers for integration tests.
//!
//! This module provides common fixtures, setup helpers, and utilities
//! used across integration tests.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Builder pattern for creating test directory structures
pub struct TestDirBuilder {
    temp_dir: TempDir,
}

impl TestDirBuilder {
    /// Create a new test directory builder
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            temp_dir: TempDir::new()?,
        })
    }

    /// Get the temporary directory path
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Add a file with content
    pub fn file(&self, name: &str, content: &[u8]) -> std::io::Result<PathBuf> {
        let path = self.temp_dir.path().join(name);
        let mut file = File::create(&path)?;
        file.write_all(content)?;
        Ok(path)
    }

    /// Add a file with string content
    pub fn file_str(&self, name: &str, content: &str) -> std::io::Result<PathBuf> {
        self.file(name, content.as_bytes())
    }

    /// Add a directory
    pub fn dir(&self, name: &str) -> std::io::Result<PathBuf> {
        let path = self.temp_dir.path().join(name);
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    /// Add a nested directory structure
    pub fn nested_dir(&self, path: &str) -> std::io::Result<PathBuf> {
        let full_path = self.temp_dir.path().join(path);
        fs::create_dir_all(&full_path)?;
        Ok(full_path)
    }

    /// Add a file in a nested path
    pub fn nested_file(&self, path: &str, content: &[u8]) -> std::io::Result<PathBuf> {
        if let Some(parent) = PathBuf::from(path).parent() {
            self.nested_dir(parent.to_str().unwrap())?;
        }
        let full_path = self.temp_dir.path().join(path);
        let mut file = File::create(&full_path)?;
        file.write_all(content)?;
        Ok(full_path)
    }

    /// Get the TempDir to control its lifetime
    pub fn into_temp_dir(self) -> TempDir {
        self.temp_dir
    }
}

impl Default for TestDirBuilder {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// Create a standard test directory structure
pub fn create_standard_test_structure() -> TestDirBuilder {
    let builder = TestDirBuilder::new().unwrap();

    // Create some files
    builder.file_str("document.txt", "Hello, World!").unwrap();
    builder.file_str("image.png", &"\0".repeat(100)).unwrap();
    builder.file_str("data.csv", "name,value\nAlice,100\n").unwrap();
    builder.file_str("README.md", "# Test Project\n").unwrap();

    // Create directories
    builder.dir("documents").unwrap();
    builder.dir("downloads").unwrap();
    builder.dir("projects").unwrap();

    // Create nested structure
    builder.nested_file("documents/work/report.txt", "Work report content").unwrap();
    builder.nested_file("documents/personal/notes.txt", "Personal notes").unwrap();
    builder.nested_file("projects/rust/main.rs", "fn main() {}").unwrap();

    builder
}

/// Create a large test directory for performance testing
pub fn create_large_test_structure(file_count: usize) -> TestDirBuilder {
    let builder = TestDirBuilder::new().unwrap();

    for i in 0..file_count {
        let dir_num = i % 10;
        let file_num = i;
        builder
            .nested_file(
                &format!("dir{}/file{:04}.txt", dir_num, file_num),
                format!("Content of file {}", i).as_bytes(),
            )
            .unwrap();
    }

    builder
}

/// Create a test directory with various file types
pub fn create_mixed_file_types_structure() -> TestDirBuilder {
    let builder = TestDirBuilder::new().unwrap();

    // Text files
    builder.file_str("readme.txt", "Readme content").unwrap();
    builder.file_str("license.md", "License text").unwrap();

    // Code files
    builder.file_str("main.rs", "fn main() {}").unwrap();
    builder.file_str("script.py", "print('hello')").unwrap();

    // Data files
    builder.file_str("data.json", "{\"key\": \"value\"}").unwrap();
    builder.file_str("config.toml", "[settings]").unwrap();

    // Binary-like files
    builder.file("binary.dat", &[0x00, 0x01, 0x02, 0x03]).unwrap();

    // Hidden files
    builder.file_str(".hidden", "hidden content").unwrap();
    builder.file_str(".config", "config data").unwrap();

    builder
}

/// Assert that two files have the same content
pub fn assert_files_equal(path1: &Path, path2: &Path) -> std::io::Result<()> {
    let content1 = fs::read(path1)?;
    let content2 = fs::read(path2)?;

    assert_eq!(
        content1, content2,
        "Files differ: {} vs {}",
        path1.display(),
        path2.display()
    );

    Ok(())
}

/// Assert that a path exists
pub fn assert_exists(path: &Path) {
    assert!(
        path.exists(),
        "Path does not exist: {}",
        path.display()
    );
}

/// Assert that a path does not exist
pub fn assert_not_exists(path: &Path) {
    assert!(
        !path.exists(),
        "Path should not exist: {}",
        path.display()
    );
}

/// Assert that a path is a directory
pub fn assert_is_dir(path: &Path) {
    assert!(
        path.is_dir(),
        "Path is not a directory: {}",
        path.display()
    );
}

/// Assert that a path is a file
pub fn assert_is_file(path: &Path) {
    assert!(
        path.is_file(),
        "Path is not a file: {}",
        path.display()
    );
}

/// Count files in a directory recursively
pub fn count_files_recursive(dir: &Path) -> std::io::Result<usize> {
    let mut count = 0;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            count += count_files_recursive(&path)?;
        } else {
            count += 1;
        }
    }
    Ok(count)
}

/// Get all file paths in a directory recursively
pub fn collect_files_recursive(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.append(&mut collect_files_recursive(&path)?);
        } else {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let builder = TestDirBuilder::new().unwrap();

        builder.file_str("test.txt", "content").unwrap();

        assert_exists(builder.path().join("test.txt"));
    }

    #[test]
    fn test_builder_nested() {
        let builder = TestDirBuilder::new().unwrap();

        builder
            .nested_file("a/b/c/test.txt", b"content")
            .unwrap();

        assert_exists(builder.path().join("a/b/c/test.txt"));
    }

    #[test]
    fn test_standard_structure() {
        let builder = create_standard_test_structure();

        assert_exists(builder.path().join("document.txt"));
        assert_exists(builder.path().join("documents"));
        assert_exists(builder.path().join("documents/work/report.txt"));
    }

    #[test]
    fn test_large_structure() {
        let builder = create_large_test_structure(100);

        let count = count_files_recursive(builder.path()).unwrap();
        assert_eq!(count, 100);
    }

    #[test]
    fn test_count_files() {
        let builder = TestDirBuilder::new().unwrap();

        builder.file_str("file1.txt", "content").unwrap();
        builder.file_str("file2.txt", "content").unwrap();
        builder.dir("subdir").unwrap();

        let count = count_files_recursive(builder.path()).unwrap();
        assert_eq!(count, 2);
    }
}
