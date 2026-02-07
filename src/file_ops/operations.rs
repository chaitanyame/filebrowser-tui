use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardOp {
    Copy,
    Cut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clipboard {
    pub operation: ClipboardOp,
    pub sources: Vec<PathBuf>,
}

pub fn perform_copy(source: &Path, target: &Path) -> Result<()> {
    if source.is_dir() {
        copy_directory(source, target)?;
    } else {
        fs::copy(source, target)
            .context(format!("Failed to copy {} to {}",
                source.display(), target.display()))?;
    }
    Ok(())
}

pub fn perform_move(source: &Path, target: &Path) -> Result<()> {
    fs::rename(source, target)
        .context(format!("Failed to move {} to {}",
            source.display(), target.display()))?;
    Ok(())
}

pub fn perform_delete(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
            .context(format!("Failed to delete directory: {}", path.display()))?;
    } else {
        fs::remove_file(path)
            .context(format!("Failed to delete file: {}", path.display()))?;
    }
    Ok(())
}

pub fn perform_mkdir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .context(format!("Failed to create directory: {}", path.display()))?;
    Ok(())
}

pub fn perform_rename(source: &Path, new_name: &str) -> Result<PathBuf> {
    let target = if let Some(parent) = source.parent() {
        parent.join(new_name)
    } else {
        PathBuf::from(new_name)
    };

    fs::rename(source, &target)
        .context(format!("Failed to rename {} to {}",
            source.display(), target.display()))?;

    Ok(target)
}

fn copy_directory(source: &Path, target: &Path) -> Result<()> {
    // Create target directory
    fs::create_dir_all(target)
        .context(format!("Failed to create target directory: {}", target.display()))?;

    // Copy contents
    for entry in fs::read_dir(source)
        .context(format!("Failed to read source directory: {}", source.display()))?
    {
        let entry = entry.context("Failed to read directory entry")?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        if source_path.is_dir() {
            copy_directory(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)
                .context(format!("Failed to copy {} to {}",
                    source_path.display(), target_path.display()))?;
        }
    }

    Ok(())
}

pub fn calculate_size(path: &Path) -> Result<u64> {
    if path.is_file() {
        return Ok(fs::metadata(path)?.len());
    }

    let mut total = 0u64;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            total += calculate_size(&entry_path)?;
        } else {
            total += entry.metadata()?.len();
        }
    }
    Ok(total)
}
