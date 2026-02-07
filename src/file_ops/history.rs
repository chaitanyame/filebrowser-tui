//! History management for undo/redo functionality
//!
//! This module provides the ability to track file operations and undo/redo them.
//! Each operation stores enough information to reverse the action.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Types of file operations that can be undone/redone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    /// Copy operation - stores source and destination paths
    Copy {
        source: PathBuf,
        destination: PathBuf,
    },
    /// Move operation - stores original and new locations
    Move {
        original_path: PathBuf,
        new_path: PathBuf,
    },
    /// Delete operation - stores the deleted path and optionally backed up content
    Delete {
        path: PathBuf,
        was_directory: bool,
        // For files, we store the content; for directories, we store structure
        backup_path: Option<PathBuf>,
    },
    /// Create directory operation - stores the created directory path
    CreateDir {
        path: PathBuf,
    },
    /// Rename operation - stores original and new names
    Rename {
        original_path: PathBuf,
        new_path: PathBuf,
    },
}

impl Operation {
    /// Get a human-readable description of the operation
    pub fn description(&self) -> String {
        match self {
            Operation::Copy { source, destination } => {
                format!(
                    "Copy: {} -> {}",
                    source.file_name().unwrap_or_default().to_string_lossy(),
                    destination.file_name().unwrap_or_default().to_string_lossy()
                )
            }
            Operation::Move { original_path, new_path } => {
                format!(
                    "Move: {} -> {}",
                    original_path.file_name().unwrap_or_default().to_string_lossy(),
                    new_path.file_name().unwrap_or_default().to_string_lossy()
                )
            }
            Operation::Delete { path, .. } => {
                format!(
                    "Delete: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                )
            }
            Operation::CreateDir { path } => {
                format!(
                    "CreateDir: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                )
            }
            Operation::Rename { original_path, new_path } => {
                format!(
                    "Rename: {} -> {}",
                    original_path.file_name().unwrap_or_default().to_string_lossy(),
                    new_path.file_name().unwrap_or_default().to_string_lossy()
                )
            }
        }
    }
}

/// A record in the history, containing the operation and timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub operation: Operation,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HistoryRecord {
    pub fn new(operation: Operation) -> Self {
        Self {
            operation,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Manages the undo/redo stack for file operations
pub struct HistoryManager {
    /// Stack of operations that can be undone (most recent first)
    undo_stack: Vec<HistoryRecord>,
    /// Stack of operations that can be redone (most recently undone first)
    redo_stack: Vec<HistoryRecord>,
    /// Maximum number of operations to keep in history
    max_history_size: usize,
    /// Temporary directory for backups during undo operations
    backup_dir: PathBuf,
}

impl HistoryManager {
    /// Create a new history manager with default settings
    pub fn new() -> Result<Self> {
        let backup_dir = Self::init_backup_dir()?;

        Ok(Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history_size: 100,
            backup_dir,
        })
    }

    /// Initialize the backup directory for storing deleted files
    fn init_backup_dir() -> Result<PathBuf> {
        let base_dir = if cfg!(windows) {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("filebrowser-tui")
        } else {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from(".cache"))
                .join("filebrowser-tui")
        };

        let backup_dir = base_dir.join("undo-backups");
        std::fs::create_dir_all(&backup_dir)
            .context("Failed to create backup directory")?;

        Ok(backup_dir)
    }

    /// Record a new operation for potential undo
    pub fn record(&mut self, operation: Operation) {
        self.undo_stack.push(HistoryRecord::new(operation));

        // Clear redo stack when new operation is performed
        self.redo_stack.clear();

        // Limit stack size
        if self.undo_stack.len() > self.max_history_size {
            self.undo_stack.remove(0);
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the description of the next undo operation
    pub fn peek_undo(&self) -> Option<String> {
        self.undo_stack
            .last()
            .map(|record| record.operation.description())
    }

    /// Get the description of the next redo operation
    pub fn peek_redo(&self) -> Option<String> {
        self.redo_stack
            .last()
            .map(|record| record.operation.description())
    }

    /// Undo the most recent operation
    pub fn undo(&mut self) -> Result<Operation> {
        let record = self.undo_stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("Nothing to undo"))?;

        // Create the inverse operation for redo
        let redo_operation = match &record.operation {
            Operation::Copy { source, destination } => {
                // Copy inverse: Delete the copy
                // For redo, we'll need to copy again
                Operation::Copy {
                    source: source.clone(),
                    destination: destination.clone(),
                }
            }
            Operation::Move { original_path, new_path } => {
                // Move inverse: Move back to original
                // For redo, move again to new path
                Operation::Move {
                    original_path: original_path.clone(),
                    new_path: new_path.clone(),
                }
            }
            Operation::Delete { path, was_directory, backup_path } => {
                // Delete inverse: Restore from backup
                // For redo, delete again
                Operation::Delete {
                    path: path.clone(),
                    was_directory: *was_directory,
                    backup_path: backup_path.clone(),
                }
            }
            Operation::CreateDir { path } => {
                // CreateDir inverse: Delete directory
                // For redo, create again
                Operation::CreateDir {
                    path: path.clone(),
                }
            }
            Operation::Rename { original_path, new_path } => {
                // Rename inverse: Rename back to original
                // For redo, rename to new name again
                Operation::Rename {
                    original_path: original_path.clone(),
                    new_path: new_path.clone(),
                }
            }
        };

        self.redo_stack.push(HistoryRecord::new(redo_operation));

        // Perform the actual undo
        self.perform_undo(&record.operation)?;

        Ok(record.operation)
    }

    /// Perform the actual undo of an operation
    fn perform_undo(&self, operation: &Operation) -> Result<()> {
        use std::fs;

        match operation {
            Operation::Copy { source, destination } => {
                // Undo copy: Delete the copied file/directory
                if destination.exists() {
                    if destination.is_dir() {
                        fs::remove_dir_all(destination)
                            .context(format!("Failed to undo copy by removing directory: {}", destination.display()))?;
                    } else {
                        fs::remove_file(destination)
                            .context(format!("Failed to undo copy by removing file: {}", destination.display()))?;
                    }
                }
                // Source still exists, nothing to restore
            }
            Operation::Move { original_path, new_path } => {
                // Undo move: Move back to original location
                if new_path.exists() {
                    // Ensure parent directory exists
                    if let Some(parent) = original_path.parent() {
                        if !parent.exists() {
                            fs::create_dir_all(parent)
                                .context(format!("Failed to create parent directory: {}", parent.display()))?;
                        }
                    }
                    fs::rename(new_path, original_path)
                        .context(format!("Failed to undo move: {} -> {}", new_path.display(), original_path.display()))?;
                }
            }
            Operation::Delete { path, was_directory, backup_path } => {
                // Undo delete: Restore from backup if available
                if let Some(backup) = backup_path {
                    if backup.exists() {
                        // Ensure parent directory exists
                        if let Some(parent) = path.parent() {
                            if !parent.exists() {
                                fs::create_dir_all(parent)
                                    .context(format!("Failed to create parent directory: {}", parent.display()))?;
                            }
                        }
                        self.restore_from_backup(backup, path, *was_directory)?;
                    }
                } else {
                    // No backup, but we can try to recreate an empty directory
                    if *was_directory {
                        fs::create_dir_all(path)
                            .context(format!("Failed to recreate directory: {}", path.display()))?;
                    }
                }
            }
            Operation::CreateDir { path } => {
                // Undo create directory: Remove it
                if path.exists() {
                    fs::remove_dir(path)
                        .context(format!("Failed to undo create directory: {}", path.display()))?;
                }
            }
            Operation::Rename { original_path, new_path } => {
                // Undo rename: Rename back to original
                if new_path.exists() {
                    fs::rename(new_path, original_path)
                        .context(format!("Failed to undo rename: {} -> {}", new_path.display(), original_path.display()))?;
                }
            }
        }

        Ok(())
    }

    /// Restore a file or directory from backup
    fn restore_from_backup(&self, backup: &PathBuf, target: &PathBuf, is_directory: bool) -> Result<()> {
        use std::fs;
        use std::io;

        if is_directory {
            // For directories, we need to recursively copy
            self.copy_directory_recursive(backup, target)?;
        } else {
            // For files, just copy
            fs::copy(backup, target)
                .context(format!("Failed to restore file from backup: {} -> {}", backup.display(), target.display()))?;
        }

        Ok(())
    }

    /// Recursively copy a directory
    fn copy_directory_recursive(&self, source: &PathBuf, target: &PathBuf) -> Result<()> {
        use std::fs;

        fs::create_dir_all(target)
            .context(format!("Failed to create target directory: {}", target.display()))?;

        for entry in fs::read_dir(source)
            .context(format!("Failed to read backup directory: {}", source.display()))?
        {
            let entry = entry.context("Failed to read directory entry")?;
            let source_path = entry.path();
            let file_name = entry.file_name();
            let target_path = target.join(&file_name);

            if source_path.is_dir() {
                self.copy_directory_recursive(&source_path, &target_path)?;
            } else {
                fs::copy(&source_path, &target_path)
                    .context(format!("Failed to copy backup file: {} -> {}", source_path.display(), target_path.display()))?;
            }
        }

        Ok(())
    }

    /// Redo the most recently undone operation
    pub fn redo(&mut self) -> Result<Operation> {
        let record = self.redo_stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("Nothing to redo"))?;

        // Create the inverse operation for undo
        let undo_operation = match &record.operation {
            Operation::Copy { source, destination } => {
                Operation::Copy {
                    source: source.clone(),
                    destination: destination.clone(),
                }
            }
            Operation::Move { original_path, new_path } => {
                Operation::Move {
                    original_path: original_path.clone(),
                    new_path: new_path.clone(),
                }
            }
            Operation::Delete { path, was_directory, backup_path } => {
                Operation::Delete {
                    path: path.clone(),
                    was_directory: *was_directory,
                    backup_path: backup_path.clone(),
                }
            }
            Operation::CreateDir { path } => {
                Operation::CreateDir {
                    path: path.clone(),
                }
            }
            Operation::Rename { original_path, new_path } => {
                Operation::Rename {
                    original_path: original_path.clone(),
                    new_path: new_path.clone(),
                }
            }
        };

        self.undo_stack.push(HistoryRecord::new(undo_operation));

        // Perform the actual redo
        self.perform_redo(&record.operation)?;

        Ok(record.operation)
    }

    /// Perform the actual redo of an operation
    fn perform_redo(&self, operation: &Operation) -> Result<()> {
        use crate::file_ops::operations::{perform_copy, perform_mkdir};
        use std::fs;

        match operation {
            Operation::Copy { source, destination } => {
                // Redo copy: Copy again
                perform_copy(source, destination)
                    .context(format!("Failed to redo copy: {} -> {}", source.display(), destination.display()))?;
            }
            Operation::Move { original_path, new_path } => {
                // Redo move: Move again
                fs::rename(original_path, new_path)
                    .context(format!("Failed to redo move: {} -> {}", original_path.display(), new_path.display()))?;
            }
            Operation::Delete { path, .. } => {
                // Redo delete: Delete again
                if path.exists() {
                    if path.is_dir() {
                        fs::remove_dir_all(path)
                            .context(format!("Failed to redo delete directory: {}", path.display()))?;
                    } else {
                        fs::remove_file(path)
                            .context(format!("Failed to redo delete file: {}", path.display()))?;
                    }
                }
            }
            Operation::CreateDir { path } => {
                // Redo create directory: Create again
                perform_mkdir(path)
                    .context(format!("Failed to redo create directory: {}", path.display()))?;
            }
            Operation::Rename { original_path, new_path } => {
                // Redo rename: Rename again
                fs::rename(original_path, new_path)
                    .context(format!("Failed to redo rename: {} -> {}", original_path.display(), new_path.display()))?;
            }
        }

        Ok(())
    }

    /// Create a backup of a file/directory before deletion
    pub fn create_backup(&self, path: &PathBuf) -> Result<PathBuf> {
        use std::fs;

        // Generate a unique backup identifier
        let timestamp = chrono::Utc::now().timestamp_micros();
        let random: u32 = rand::random();
        let backup_name = format!("{}_{}", timestamp, random);
        let backup_path = self.backup_dir.join(&backup_name);

        // Create the backup
        if path.is_dir() {
            self.copy_directory_recursive(path, &backup_path)?;
        } else {
            fs::copy(path, &backup_path)
                .context(format!("Failed to create backup: {} -> {}", path.display(), backup_path.display()))?;
        }

        Ok(backup_path)
    }

    /// Clean up old backups (call periodically to manage disk space)
    pub fn cleanup_old_backups(&self, max_age_hours: i64) -> Result<()> {
        use std::fs;

        let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(max_age_hours);

        for entry in fs::read_dir(&self.backup_dir)
            .context("Failed to read backup directory")?
        {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            // Extract timestamp from filename
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if let Some(timestamp_str) = name.split('_').next() {
                    if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                        let backup_time = chrono::DateTime::from_timestamp(timestamp / 1_000_000, 0)
                            .unwrap_or(chrono::Utc::now());

                        if backup_time < cutoff_time {
                            // Clean up this backup
                            if path.is_dir() {
                                let _ = fs::remove_dir_all(&path);
                            } else {
                                let _ = fs::remove_file(&path);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Clear all history (both undo and redo stacks)
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Get the number of undo operations available
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of redo operations available
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history_size: 100,
            backup_dir: PathBuf::from("."),
        })
    }
}

#[cfg(test)]
mod tests;
