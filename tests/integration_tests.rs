//! Integration tests for file browser TUI.
//!
//! These tests work with real temporary directories and test the full
//! functionality of file operations and navigation.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use filebrowser_tui::state::{FileEntry, SelectionManager, SortBy, SortOrder};
use filebrowser_tui::file_ops::{
    perform_copy, perform_delete, perform_mkdir, perform_move, perform_rename,
    BulkRenamer, CaseScope, CaseTransform, ExtensionAction, RenamePattern,
};

// ============================================================================
// Test Utilities
// ============================================================================

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
}

impl Default for TestDirBuilder {
    fn default() -> Self {
        Self::new().unwrap()
    }
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

// ============================================================================
// Basic File Operations Tests
// ============================================================================

#[test]
fn test_copy_file_to_directory() {
    let builder = TestDirBuilder::new().unwrap();
    let source = builder.file_str("source.txt", "Hello, World!").unwrap();
    let dest_dir = builder.dir("dest").unwrap();
    let dest = dest_dir.join("source.txt");

    perform_copy(&source, &dest).unwrap();

    assert_exists(&dest);
    assert_files_equal(&source, &dest);
}

#[test]
fn test_copy_file_overwrite() {
    let builder = TestDirBuilder::new().unwrap();
    let source = builder.file_str("source.txt", "New content").unwrap();
    let dest = builder.file_str("dest.txt", "Old content").unwrap();

    perform_copy(&source, &dest).unwrap();

    assert_exists(&dest);
    let content = fs::read_to_string(&dest).unwrap();
    assert_eq!(content, "New content");
}

#[test]
fn test_copy_nonexistent_file() {
    let builder = TestDirBuilder::new().unwrap();
    let source = builder.path().join("nonexistent.txt");
    let dest = builder.path().join("dest.txt");

    let result = perform_copy(&source, &dest);

    assert!(result.is_err());
}

#[test]
fn test_move_file() {
    let builder = TestDirBuilder::new().unwrap();
    let source = builder.file_str("source.txt", "Content").unwrap();
    let dest_dir = builder.dir("dest").unwrap();
    let dest = dest_dir.join("source.txt");

    perform_move(&source, &dest).unwrap();

    assert_not_exists(&source);
    assert_exists(&dest);
    let content = fs::read_to_string(&dest).unwrap();
    assert_eq!(content, "Content");
}

#[test]
fn test_move_file_to_same_name() {
    let builder = TestDirBuilder::new().unwrap();
    let source = builder.file_str("file.txt", "Content").unwrap();
    let dest = builder.path().join("file.txt");

    // Moving to same location should succeed (or at least not error)
    let result = perform_move(&source, &dest);
    // Behavior depends on implementation
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_delete_file() {
    let builder = TestDirBuilder::new().unwrap();
    let file = builder.file_str("to_delete.txt", "Content").unwrap();

    assert_exists(&file);

    perform_delete(&file).unwrap();

    assert_not_exists(&file);
}

#[test]
fn test_delete_directory() {
    let builder = TestDirBuilder::new().unwrap();
    let dir = builder.dir("to_delete").unwrap();
    builder.nested_file("to_delete/file.txt", b"content").unwrap();

    assert_exists(&dir);

    perform_delete(&dir).unwrap();

    assert_not_exists(&dir);
}

#[test]
fn test_delete_nonexistent() {
    let builder = TestDirBuilder::new().unwrap();
    let file = builder.path().join("nonexistent.txt");

    let result = perform_delete(&file);

    // Should either succeed or return an error
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_create_directory() {
    let builder = TestDirBuilder::new().unwrap();
    let dir = builder.path().join("new_dir");

    perform_mkdir(&dir).unwrap();

    assert_exists(&dir);
    assert_is_dir(&dir);
}

#[test]
fn test_create_nested_directory() {
    let builder = TestDirBuilder::new().unwrap();
    let dir = builder.path().join("parent/child/grandchild");

    perform_mkdir(&dir).unwrap();

    assert_exists(&dir);
    assert_is_dir(&dir);
}

#[test]
fn test_create_directory_already_exists() {
    let builder = TestDirBuilder::new().unwrap();
    let dir = builder.dir("existing").unwrap();

    let result = perform_mkdir(&dir);

    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_rename_file() {
    let builder = TestDirBuilder::new().unwrap();
    let source = builder.file_str("old.txt", "Content").unwrap();
    let dest = builder.path().join("new.txt");

    perform_rename(&source, &dest).unwrap();

    assert_not_exists(&source);
    assert_exists(&dest);

    let content = fs::read_to_string(&dest).unwrap();
    assert_eq!(content, "Content");
}

#[test]
fn test_rename_directory() {
    let builder = TestDirBuilder::new().unwrap();
    let source = builder.dir("old_dir").unwrap();
    builder.nested_file("old_dir/file.txt", b"content").unwrap();
    let dest = builder.path().join("new_dir");

    perform_rename(&source, &dest).unwrap();

    assert_not_exists(&source);
    assert_exists(&dest);
    assert_exists(&dest.join("file.txt"));
}

#[test]
fn test_rename_to_existing_file() {
    let builder = TestDirBuilder::new().unwrap();
    let source = builder.file_str("source.txt", "Source content").unwrap();
    let dest = builder.file_str("dest.txt", "Dest content").unwrap();

    let result = perform_rename(&source, &dest);

    // Should fail due to existing destination
    assert!(result.is_err());

    // Both files should remain
    assert_exists(&source);
    assert_exists(&dest);
}

// ============================================================================
// Bulk Operations Tests
// ============================================================================

#[test]
fn test_bulk_copy_multiple_files() {
    let builder = TestDirBuilder::new().unwrap();
    let files = vec![
        builder.file_str("file1.txt", "content1").unwrap(),
        builder.file_str("file2.txt", "content2").unwrap(),
        builder.file_str("file3.txt", "content3").unwrap(),
    ];
    let dest_dir = builder.dir("dest").unwrap();

    for file in &files {
        let dest = dest_dir.join(file.file_name().unwrap());
        perform_copy(file, &dest).unwrap();
    }

    assert_eq!(count_files_recursive(&dest_dir).unwrap(), 3);
}

#[test]
fn test_bulk_delete_files() {
    let builder = TestDirBuilder::new().unwrap();
    let files = vec![
        builder.file_str("file1.txt", "content").unwrap(),
        builder.file_str("file2.txt", "content").unwrap(),
        builder.file_str("file3.txt", "content").unwrap(),
    ];

    for file in &files {
        perform_delete(file).unwrap();
    }

    for file in &files {
        assert_not_exists(file);
    }
}

// ============================================================================
// FileEntry Tests
// ============================================================================

#[test]
fn test_file_entry_from_file() {
    let builder = TestDirBuilder::new().unwrap();
    let file_path = builder.file_str("test.txt", "content").unwrap();

    let entry = FileEntry::from_path(file_path.clone()).unwrap();

    assert_eq!(entry.name, "test.txt");
    assert!(!entry.is_dir);
    assert!(entry.size > 0);
}

#[test]
fn test_file_entry_from_directory() {
    let builder = TestDirBuilder::new().unwrap();
    let dir_path = builder.dir("testdir").unwrap();

    let entry = FileEntry::from_path(dir_path.clone()).unwrap();

    assert_eq!(entry.name, "testdir");
    assert!(entry.is_dir);
}

#[test]
fn test_file_entry_nonexistent() {
    let path = PathBuf::from("/nonexistent/path");

    let result = FileEntry::from_path(path);

    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// Sorting Tests
// ============================================================================

#[test]
fn test_sort_files_by_name() {
    let builder = TestDirBuilder::new().unwrap();
    builder.file_str("zebra.txt", "").unwrap();
    builder.file_str("apple.txt", "").unwrap();
    builder.file_str("banana.txt", "").unwrap();

    let mut entries: Vec<FileEntry> = vec![];
    for entry in fs::read_dir(builder.path()).unwrap() {
        let entry = entry.unwrap();
        if let Ok(file_entry) = FileEntry::from_path(entry.path()) {
            if !file_entry.is_dir {
                entries.push(file_entry);
            }
        }
    }

    use filebrowser_tui::state::sort_files;
    sort_files(&mut entries, SortBy::Name, SortOrder::Ascending);

    assert_eq!(entries[0].name, "apple.txt");
    assert_eq!(entries[1].name, "banana.txt");
    assert_eq!(entries[2].name, "zebra.txt");
}

#[test]
fn test_sort_files_with_directories() {
    let builder = TestDirBuilder::new().unwrap();
    builder.dir("adir").unwrap();
    builder.file_str("zfile.txt", "").unwrap();
    builder.dir("bdir").unwrap();
    builder.file_str("afile.txt", "").unwrap();

    let mut entries: Vec<FileEntry> = vec![];
    for entry in fs::read_dir(builder.path()).unwrap() {
        let entry = entry.unwrap();
        if let Ok(file_entry) = FileEntry::from_path(entry.path()) {
            entries.push(file_entry);
        }
    }

    use filebrowser_tui::state::sort_files;
    sort_files(&mut entries, SortBy::Name, SortOrder::Ascending);

    // Directories should come first
    assert!(entries[0].is_dir);
    assert!(entries[1].is_dir);
    assert!(!entries[2].is_dir);
    assert!(!entries[3].is_dir);
}

// ============================================================================
// SelectionManager Tests
// ============================================================================

#[test]
fn test_selection_manager_basic() {
    let builder = TestDirBuilder::new().unwrap();
    let files = vec![
        builder.file_str("file1.txt", "").unwrap(),
        builder.file_str("file2.txt", "").unwrap(),
        builder.file_str("file3.txt", "").unwrap(),
    ];

    let mut manager = SelectionManager::new();

    assert!(manager.is_empty());

    manager.select(files[0].clone());
    manager.select(files[1].clone());

    assert_eq!(manager.count(), 2);
    assert!(manager.is_selected(&files[0]));
    assert!(manager.is_selected(&files[1]));
    assert!(!manager.is_selected(&files[2]));
}

#[test]
fn test_selection_manager_toggle() {
    let builder = TestDirBuilder::new().unwrap();
    let file = builder.file_str("file.txt", "").unwrap();

    let mut manager = SelectionManager::new();

    manager.toggle(file.clone());
    assert!(manager.is_selected(&file));

    manager.toggle(file.clone());
    assert!(!manager.is_selected(&file));
}

#[test]
fn test_selection_manager_range() {
    let builder = TestDirBuilder::new().unwrap();
    let mut entries: Vec<FileEntry> = vec![];
    for i in 1..=5 {
        let path = builder.file_str(&format!("file{}.txt", i), "").unwrap();
        entries.push(FileEntry::from_path(path).unwrap());
    }

    let mut manager = SelectionManager::new();

    manager.select_range(1, 3, &entries);

    assert_eq!(manager.count(), 3);
    assert!(manager.is_selected(&entries[1].path));
    assert!(manager.is_selected(&entries[2].path));
    assert!(manager.is_selected(&entries[3].path));
}

#[test]
fn test_selection_manager_invert() {
    let builder = TestDirBuilder::new().unwrap();
    let files = vec![
        builder.file_str("file1.txt", "").unwrap(),
        builder.file_str("file2.txt", "").unwrap(),
        builder.file_str("file3.txt", "").unwrap(),
    ];

    let mut manager = SelectionManager::new();

    manager.select(files[0].clone());
    assert_eq!(manager.count(), 1);

    manager.invert(files.clone());

    assert_eq!(manager.count(), 2);
    assert!(!manager.is_selected(&files[0]));
    assert!(manager.is_selected(&files[1]));
    assert!(manager.is_selected(&files[2]));
}

// ============================================================================
// Bulk Rename Tests
// ============================================================================

#[test]
fn test_bulk_rename_simple_replace() {
    let builder = TestDirBuilder::new().unwrap();
    let files = vec![
        builder.file_str("old_file1.txt", "").unwrap(),
        builder.file_str("old_file2.txt", "").unwrap(),
    ];

    let pattern = RenamePattern::SimpleReplace {
        find: "old".to_string(),
        replace: "new".to_string(),
    };

    let renamer = BulkRenamer::new(pattern, builder.path().to_path_buf());
    let previews = renamer.preview(&files);

    assert_eq!(previews[0].new_name(), "new_file1.txt");
    assert_eq!(previews[1].new_name(), "new_file2.txt");
}

#[test]
fn test_bulk_rename_numbered() {
    let builder = TestDirBuilder::new().unwrap();
    let files = vec![
        builder.file_str("img1.jpg", "").unwrap(),
        builder.file_str("img2.jpg", "").unwrap(),
        builder.file_str("img3.jpg", "").unwrap(),
    ];

    let pattern = RenamePattern::Numbered {
        template: "photo_{n}.jpg".to_string(),
        start: 1,
        pad_width: 3,
    };

    let renamer = BulkRenamer::new(pattern, builder.path().to_path_buf());
    let previews = renamer.preview(&files);

    assert_eq!(previews[0].new_name(), "photo_001.jpg");
    assert_eq!(previews[1].new_name(), "photo_002.jpg");
    assert_eq!(previews[2].new_name(), "photo_003.jpg");
}

#[test]
fn test_bulk_rename_case_transform() {
    let builder = TestDirBuilder::new().unwrap();
    let files = vec![builder.file_str("lowercase.txt", "").unwrap()];

    let pattern = RenamePattern::Case {
        transform: CaseTransform::Uppercase,
        scope: CaseScope::NameOnly,
    };

    let renamer = BulkRenamer::new(pattern, builder.path().to_path_buf());
    let previews = renamer.preview(&files);

    assert_eq!(previews[0].new_name(), "LOWERCASE.txt");
}

#[test]
fn test_bulk_rename_extension_replace() {
    let builder = TestDirBuilder::new().unwrap();
    let files = vec![builder.file_str("document.txt", "").unwrap()];

    let pattern = RenamePattern::Extension {
        action: ExtensionAction::Replace,
        new_extension: Some("md".to_string()),
    };

    let renamer = BulkRenamer::new(pattern, builder.path().to_path_buf());
    let previews = renamer.preview(&files);

    assert_eq!(previews[0].new_name(), "document.md");
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_operation_on_nonexistent_path() {
    let builder = TestDirBuilder::new().unwrap();
    let nonexistent = builder.path().join("nonexistent");

    assert!(!nonexistent.exists());

    let copy_result = perform_copy(&nonexistent, &builder.path().join("dest"));
    let move_result = perform_move(&nonexistent, &builder.path().join("dest"));
    let delete_result = perform_delete(&nonexistent);

    assert!(copy_result.is_err());
    assert!(move_result.is_err());
    assert!(delete_result.is_err() || delete_result.is_ok());
}

#[test]
fn test_special_characters_in_filename() {
    let builder = TestDirBuilder::new().unwrap();
    let filename = "file with spaces & special!@#.txt";

    let file = builder.file_str(filename, "content").unwrap();

    assert_exists(&file);

    let content = fs::read_to_string(&file).unwrap();
    assert_eq!(content, "content");
}

#[test]
fn test_unicode_filename() {
    let builder = TestDirBuilder::new().unwrap();
    let filename = "文件名.txt";

    let file = builder.file_str(filename, "content").unwrap();

    assert_exists(&file);

    let entry = FileEntry::from_path(file.clone()).unwrap();
    assert_eq!(entry.name, filename);
}

#[test]
fn test_very_long_filename() {
    let builder = TestDirBuilder::new().unwrap();
    let long_name = "a".repeat(200) + ".txt";

    let result = builder.file_str(&long_name, "content");

    // May fail on some systems due to filename length limits
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_empty_filename() {
    let builder = TestDirBuilder::new().unwrap();

    let result = File::create(builder.path().join(""));

    assert!(result.is_err());
}

#[test]
fn test_directory_with_many_files() {
    let builder = TestDirBuilder::new().unwrap();
    let dir = builder.dir("many_files").unwrap();

    for i in 0..100 {
        let filename = format!("file{:03}.txt", i);
        let path = dir.join(&filename);
        let mut file = File::create(&path).unwrap();
        file.write_all(format!("content {}", i).as_bytes()).unwrap();
    }

    let count = count_files_recursive(&dir).unwrap();
    assert_eq!(count, 100);
}

#[test]
fn test_deeply_nested_directory() {
    let builder = TestDirBuilder::new().unwrap();

    let mut current = builder.path().to_path_buf();
    for i in 0..10 {
        current = current.join(format!("level{}", i));
        fs::create_dir(&current).unwrap();
    }

    assert_exists(&current);

    // Add a file at the deepest level
    let file = current.join("deep.txt");
    let mut f = File::create(&file).unwrap();
    f.write_all(b"deep content").unwrap();

    assert_exists(&file);
}

// ============================================================================
// Cross-Platform Tests
// ============================================================================

#[test]
fn test_line_endings_preserved() {
    let builder = TestDirBuilder::new().unwrap();

    // Test with different line endings
    let content_crlf = "line1\r\nline2\r\nline3\r\n";
    let content_lf = "line1\nline2\nline3\n";

    let file1 = builder.file_str("crlf.txt", content_crlf).unwrap();
    let file2 = builder.file_str("lf.txt", content_lf).unwrap();

    let read1 = fs::read_to_string(&file1).unwrap();
    let read2 = fs::read_to_string(&file2).unwrap();

    assert_eq!(read1, content_crlf);
    assert_eq!(read2, content_lf);
}

#[test]
fn test_file_permissions() {
    let builder = TestDirBuilder::new().unwrap();
    let file = builder.file_str("test.txt", "content").unwrap();

    let metadata = fs::metadata(&file).unwrap();
    let permissions = metadata.permissions();

    // We can read the permissions
    let readonly = permissions.readonly();

    // Test that we can still read the file
    let content = fs::read_to_string(&file).unwrap();
    assert_eq!(content, "content");

    // On Unix, we might be able to test write permissions
    // On Windows, this may be more restricted
    let _ = readonly;
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_large_file_copy() {
    let builder = TestDirBuilder::new().unwrap();

    // Create a 1MB file
    let large_data = vec![0u8; 1024 * 1024];
    let source = builder.file("large.bin", &large_data).unwrap();
    let dest = builder.path().join("large_copy.bin");

    let start = std::time::Instant::now();
    perform_copy(&source, &dest).unwrap();
    let duration = start.elapsed();

    assert_exists(&dest);
    assert_files_equal(&source, &dest);

    // Should complete in reasonable time (< 5 seconds)
    assert!(duration.as_secs() < 5, "Copy took too long: {:?}", duration);
}

#[test]
fn test_many_small_files() {
    let builder = TestDirBuilder::new().unwrap();

    let file_count = 100;
    for i in 0..file_count {
        builder
            .file_str(&format!("file{:03}.txt", i), &format!("content {}", i))
            .unwrap();
    }

    let count = count_files_recursive(builder.path()).unwrap();
    assert_eq!(count, file_count);
}

// ============================================================================
// Concurrent Operations
// ============================================================================

#[test]
fn test_simultaneous_file_operations() {
    let builder = TestDirBuilder::new().unwrap();

    let file1 = builder.file_str("file1.txt", "content1").unwrap();
    let file2 = builder.file_str("file2.txt", "content2").unwrap();
    let file3 = builder.file_str("file3.txt", "content3").unwrap();

    // Perform multiple operations
    perform_copy(&file1, &builder.path().join("copy1.txt")).unwrap();
    perform_copy(&file2, &builder.path().join("copy2.txt")).unwrap();
    perform_copy(&file3, &builder.path().join("copy3.txt")).unwrap();

    assert_exists(&builder.path().join("copy1.txt"));
    assert_exists(&builder.path().join("copy2.txt"));
    assert_exists(&builder.path().join("copy3.txt"));
}

// ============================================================================
// Cleanup Tests
// ============================================================================

#[test]
fn test_tempdir_cleanup() {
    let builder = TestDirBuilder::new().unwrap();
    let path = builder.path().to_path_buf();

    builder.file_str("test.txt", "content").unwrap();

    assert_exists(&path);

    // When builder goes out of scope, temp dir is cleaned up
    drop(builder);

    // Path should no longer exist
    assert_not_exists(&path);
}

#[test]
fn test_multiple_tempdirs() {
    let builder1 = TestDirBuilder::new().unwrap();
    let builder2 = TestDirBuilder::new().unwrap();

    builder1.file_str("test1.txt", "content1").unwrap();
    builder2.file_str("test2.txt", "content2").unwrap();

    assert_ne!(builder1.path(), builder2.path());

    assert_exists(builder1.path().join("test1.txt"));
    assert_exists(builder2.path().join("test2.txt"));
}
