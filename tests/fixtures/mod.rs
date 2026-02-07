//! Test fixtures for end-to-end testing
//!
//! Provides helper functions for creating test directory structures,
//! sample files, and cleanup utilities.

use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test fixture manager
///
/// Manages the lifecycle of test fixtures including creation and cleanup.
pub struct TestFixture {
    /// Temporary directory for this fixture
    temp_dir: TempDir,
    /// Path to the test directory
    pub path: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # use tests::fixtures::TestFixture;
    /// # fn main() -> Result<()> {
    /// let fixture = TestFixture::new()?;
    /// println!("Test directory: {:?}", fixture.path);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()
            .context("Failed to create temporary directory")?;

        let path = temp_dir.path().to_path_buf();

        Ok(Self {
            temp_dir,
            path,
        })
    }

    /// Create a new test fixture with a specific name
    pub fn with_name(name: &str) -> Result<Self> {
        let temp_dir = TempDir::with_prefix(name)
            .context("Failed to create temporary directory")?;

        let path = temp_dir.path().to_path_buf();

        Ok(Self {
            temp_dir,
            path,
        })
    }

    /// Create a standard test directory structure
    ///
    /// Creates a typical directory layout for testing:
    /// ```
    /// test_root/
    /// ├── dir1/
    /// │   ├── file1.txt
    /// │   └── file2.txt
    /// ├── dir2/
    /// │   └── subdir/
    /// │       └── file3.txt
    /// ├── empty_dir/
    /// ├── root_file.txt
    /// └── special.md
    /// ```
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # use tests::fixtures::TestFixture;
    /// # fn main() -> Result<()> {
    /// let fixture = TestFixture::new()?;
    /// fixture.create_standard_structure()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_standard_structure(&self) -> Result<()> {
        // Create directories
        self.create_dir("dir1")?;
        self.create_dir("dir2/subdir")?;
        self.create_dir("empty_dir")?;

        // Create files
        self.create_file("dir1/file1.txt", "Content of file1")?;
        self.create_file("dir1/file2.txt", "Content of file2")?;
        self.create_file("dir2/subdir/file3.txt", "Content of file3")?;
        self.create_file("root_file.txt", "Root file content")?;
        self.create_file("special.md", "# Markdown File\n\nWith **content**")?;

        Ok(())
    }

    /// Create a nested directory structure for testing navigation
    ///
    /// Creates a deep hierarchy:
    /// ```
    /// deep/
    /// ├── level1/
    /// │   ├── level2/
    /// │   │   └── level3/
    /// │   │       └── level4/
    /// │   │           └── deep_file.txt
    /// ```
    pub fn create_nested_structure(&self) -> Result<()> {
        let deep_path = self.path.join("deep/level1/level2/level3/level4");
        fs::create_dir_all(&deep_path)
            .context("Failed to create nested directories")?;

        self.create_file("deep/level1/level2/level3/level4/deep_file.txt", "Deep content")?;

        Ok(())
    }

    /// Create a large file structure for testing scrolling
    ///
    /// Creates a directory with many files to test scrolling behavior.
    pub fn create_large_structure(&self) -> Result<()> {
        self.create_dir("many_files")?;

        for i in 1..=100 {
            let filename = format!("many_files/file_{:03}.txt", i);
            self.create_file(&filename, &format!("Content of file {}", i))?;
        }

        Ok(())
    }

    /// Create a structure for testing search operations
    ///
    /// Creates files with various content patterns for search testing.
    pub fn create_search_structure(&self) -> Result<()> {
        self.create_dir("search_test")?;

        // Files with different extensions
        self.create_file("search_test/doc1.txt", "Search term here")?;
        self.create_file("search_test/doc2.txt", "Another document")?;
        self.create_file("search_test/notes.md", "# Notes\n\nSearch term in markdown")?;
        self.create_file("search_test/readme.md", "README content")?;
        self.create_file("search_test/data.json", r#"{"key": "search term"}"#)?;

        // Subdirectory with more files
        self.create_dir("search_test/subdir")?;
        self.create_file("search_test/subdir/nested.txt", "Nested search term")?;

        Ok(())
    }

    /// Create a structure for testing file operations
    ///
    /// Creates files and directories for testing copy, move, rename, delete.
    pub fn create_operations_structure(&self) -> Result<()> {
        // Source files for operations
        self.create_file("to_copy.txt", "File to copy")?;
        self.create_file("to_move.txt", "File to move")?;
        self.create_file("to_rename.txt", "File to rename")?;
        self.create_file("to_delete.txt", "File to delete")?;

        // Target directories
        self.create_dir("copy_dest")?;
        self.create_dir("move_dest")?;
        self.create_dir("rename_dest")?;

        // Files for bulk rename testing
        self.create_dir("bulk_rename_test")?;
        for i in 1..=10 {
            let filename = format!("bulk_rename_test/photo_{:02}.jpg", i);
            self.create_file(&filename, &format!("Photo {}", i))?;
        }

        Ok(())
    }

    /// Create a structure for testing tab management
    ///
    /// Creates multiple directories for tab testing.
    pub fn create_tab_structure(&self) -> Result<()> {
        for i in 1..=5 {
            let dir_name = format!("tab_dir_{}", i);
            self.create_dir(&dir_name)?;
            self.create_file(&format!("{}/file{}.txt", dir_name, i), &format!("Content {}", i))?;
        }

        Ok(())
    }

    /// Create a structure for testing split view
    ///
    /// Creates two independent directory structures.
    pub fn create_split_structure(&self) -> Result<()> {
        // Left pane structure
        self.create_dir("left_dir")?;
        self.create_file("left_dir/left1.txt", "Left file 1")?;
        self.create_file("left_dir/left2.txt", "Left file 2")?;

        // Right pane structure
        self.create_dir("right_dir")?;
        self.create_file("right_dir/right1.txt", "Right file 1")?;
        self.create_file("right_dir/right2.txt", "Right file 2")?;

        Ok(())
    }

    /// Create a structure for testing undo/redo
    ///
    /// Creates files and directories that can be modified.
    pub fn create_undo_structure(&self) -> Result<()> {
        self.create_dir("undo_test")?;
        self.create_file("undo_test/original.txt", "Original content")?;

        Ok(())
    }

    /// Create a directory
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to create (relative to fixture root)
    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let full_path = self.path.join(path);
        fs::create_dir_all(&full_path)
            .with_context(|| format!("Failed to create directory: {:?}", full_path))?;
        Ok(())
    }

    /// Create a file with content
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to create (relative to fixture root)
    /// * `content` - Content to write to the file
    pub fn create_file<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<()> {
        let full_path = self.path.join(path);

        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create parent directories: {:?}", parent))?;
            }
        }

        let mut file = File::create(&full_path)
            .with_context(|| format!("Failed to create file: {:?}", full_path))?;

        file.write_all(content.as_bytes())
            .with_context(|| format!("Failed to write to file: {:?}", full_path))?;

        Ok(())
    }

    /// Create an empty file
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to create (relative to fixture root)
    pub fn create_empty_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.create_file(path, "")
    }

    /// Create a file with specific size
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to create
    /// * `size_bytes` - Size of the file in bytes
    pub fn create_file_with_size<P: AsRef<Path>>(&self, path: P, size_bytes: usize) -> Result<()> {
        let content = "x".repeat(size_bytes);
        self.create_file(path, &content)
    }

    /// Create a binary file with specific content
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to create
    /// * `data` - Binary data to write
    pub fn create_binary_file<P: AsRef<Path>>(&self, path: P, data: &[u8]) -> Result<()> {
        let full_path = self.path.join(path);

        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create parent directories: {:?}", parent))?;
            }
        }

        let mut file = File::create(&full_path)
            .with_context(|| format!("Failed to create file: {:?}", full_path))?;

        file.write_all(data)
            .with_context(|| format!("Failed to write to file: {:?}", full_path))?;

        Ok(())
    }

    /// Append content to an existing file
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to append to
    /// * `content` - Content to append
    pub fn append_to_file<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<()> {
        let full_path = self.path.join(path);

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&full_path)
            .with_context(|| format!("Failed to open file for appending: {:?}", full_path))?;

        file.write_all(content.as_bytes())
            .with_context(|| format!("Failed to append to file: {:?}", full_path))?;

        Ok(())
    }

    /// Read file content
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to read
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let full_path = self.path.join(path);

        fs::read_to_string(&full_path)
            .with_context(|| format!("Failed to read file: {:?}", full_path))
    }

    /// Check if a file exists
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to check
    pub fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
        self.path.join(path).exists()
    }

    /// Get file metadata
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to query
    pub fn metadata<P: AsRef<Path>>(&self, path: P) -> Result<fs::Metadata> {
        let full_path = self.path.join(path);

        fs::metadata(&full_path)
            .with_context(|| format!("Failed to get metadata: {:?}", full_path))
    }

    /// Delete a file or directory
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to delete
    pub fn delete<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let full_path = self.path.join(path);

        if full_path.is_dir() {
            fs::remove_dir_all(&full_path)
                .with_context(|| format!("Failed to remove directory: {:?}", full_path))?;
        } else {
            fs::remove_file(&full_path)
                .with_context(|| format!("Failed to remove file: {:?}", full_path))?;
        }

        Ok(())
    }

    /// Rename a file or directory
    ///
    /// # Arguments
    ///
    /// * `from` - Current relative path
    /// * `to` - New relative path
    pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> Result<()> {
        let from_path = self.path.join(from);
        let to_path = self.path.join(to);

        fs::rename(&from_path, &to_path)
            .with_context(|| format!("Failed to rename {:?} to {:?}", from_path, to_path))?;

        Ok(())
    }

    /// Copy a file
    ///
    /// # Arguments
    ///
    /// * `from` - Source relative path
    /// * `to` - Destination relative path
    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> Result<()> {
        let from_path = self.path.join(from);
        let to_path = self.path.join(to);

        fs::copy(&from_path, &to_path)
            .with_context(|| format!("Failed to copy {:?} to {:?}", from_path, to_path))?;

        Ok(())
    }

    /// List directory contents
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to list
    pub fn list_dir<P: AsRef<Path>>(&self, path: P) -> Result<Vec<PathBuf>> {
        let full_path = self.path.join(path);

        let entries = fs::read_dir(&full_path)
            .with_context(|| format!("Failed to read directory: {:?}", full_path))?;

        let mut result = Vec::new();
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            result.push(entry.path());
        }

        Ok(result)
    }

    /// Count files in a directory (non-recursive)
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to count
    pub fn count_files<P: AsRef<Path>>(&self, path: P) -> Result<usize> {
        let entries = self.list_dir(path)?;
        Ok(entries.len())
    }

    /// Get the absolute path for a relative path
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path
    pub fn resolve_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.path.join(path)
    }

    /// Clean up all files in the fixture
    ///
    /// Removes all files and directories, keeping the fixture root.
    pub fn cleanup(&self) -> Result<()> {
        for entry in fs::read_dir(&self.path)
            .context("Failed to read fixture directory")?
        {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_dir() {
                fs::remove_dir_all(&path)
                    .with_context(|| format!("Failed to remove directory: {:?}", path))?;
            } else {
                fs::remove_file(&path)
                    .with_context(|| format!("Failed to remove file: {:?}", path))?;
            }
        }

        Ok(())
    }

    /// Create a fixture for content search testing
    ///
    /// Creates files with specific content patterns for testing content search.
    pub fn create_content_search_fixture(&self) -> Result<()> {
        self.create_dir("content_search")?;

        // Create files with various content patterns
        self.create_file("content_search/doc1.txt", "Hello World\nThis is a test")?;
        self.create_file("content_search/doc2.txt", "Another document\nWith different content")?;
        self.create_file("content_search/notes.md", "# Notes\n\nImportant: Remember this")?;
        self.create_file("content_search/code.rs", "fn main() {\n    println!(\"Hello\");\n}")?;

        // Create nested structure
        self.create_dir("content_search/nested")?;
        self.create_file("content_search/nested/deep.txt", "Deep content here")?;

        Ok(())
    }

    /// Create a fixture for bookmark testing
    ///
    /// Creates directories that can be bookmarked.
    pub fn create_bookmark_fixture(&self) -> Result<()> {
        self.create_dir("project/src")?;
        self.create_dir("project/tests")?;
        self.create_dir("project/docs")?;
        self.create_dir("downloads")?;
        self.create_dir("documents")?;

        Ok(())
    }

    /// Create a fixture for permission testing (Unix only)
    ///
    /// Creates files with different permissions.
    #[cfg(unix)]
    pub fn create_permission_fixture(&self) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        self.create_dir("permissions")?;

        // Read-only file
        let readonly_path = self.path.join("permissions/readonly.txt");
        self.create_file("permissions/readonly.txt", "Read only content")?;
        let mut perms = fs::metadata(&readonly_path)?.permissions();
        perms.set_mode(0o444);
        fs::set_permissions(&readonly_path, perms)?;

        // Executable file
        let exec_path = self.path.join("permissions/script.sh");
        self.create_file("permissions/script.sh", "#!/bin/bash\necho 'Hello'")?;
        let mut perms = fs::metadata(&exec_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&exec_path, perms)?;

        Ok(())
    }
}

/// Builder for creating custom test fixtures
pub struct FixtureBuilder {
    fixture: TestFixture,
}

impl FixtureBuilder {
    /// Create a new fixture builder
    pub fn new() -> Result<Self> {
        Ok(Self {
            fixture: TestFixture::new()?,
        })
    }

    /// Add a directory
    pub fn with_dir(mut self, path: &str) -> Result<Self> {
        self.fixture.create_dir(path)?;
        Ok(self)
    }

    /// Add a file with content
    pub fn with_file(mut self, path: &str, content: &str) -> Result<Self> {
        self.fixture.create_file(path, content)?;
        Ok(self)
    }

    /// Build the fixture
    pub fn build(self) -> TestFixture {
        self.fixture
    }
}

impl Default for FixtureBuilder {
    fn default() -> Self {
        Self::new().expect("Failed to create fixture builder")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_creation() {
        let fixture = TestFixture::new().unwrap();
        assert!(fixture.path.exists());
    }

    #[test]
    fn test_standard_structure() {
        let fixture = TestFixture::new().unwrap();
        fixture.create_standard_structure().unwrap();

        assert!(fixture.exists("dir1"));
        assert!(fixture.exists("dir2"));
        assert!(fixture.exists("root_file.txt"));
        assert!(fixture.exists("dir1/file1.txt"));
    }

    #[test]
    fn test_file_creation() {
        let fixture = TestFixture::new().unwrap();
        fixture.create_file("test.txt", "Hello, World!").unwrap();

        assert!(fixture.exists("test.txt"));
        let content = fixture.read_file("test.txt").unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_fixture_builder() {
        let fixture = FixtureBuilder::new()
            .unwrap()
            .with_dir("custom_dir").unwrap()
            .with_file("custom_dir/file.txt", "Content").unwrap()
            .build();

        assert!(fixture.exists("custom_dir/file.txt"));
    }

    #[test]
    fn test_file_operations() {
        let fixture = TestFixture::new().unwrap();
        fixture.create_file("original.txt", "Original content").unwrap();

        // Test copy
        fixture.copy("original.txt", "copied.txt").unwrap();
        assert!(fixture.exists("copied.txt"));

        // Test rename
        fixture.rename("original.txt", "renamed.txt").unwrap();
        assert!(!fixture.exists("original.txt"));
        assert!(fixture.exists("renamed.txt"));

        // Test delete
        fixture.delete("renamed.txt").unwrap();
        assert!(!fixture.exists("renamed.txt"));
    }
}
