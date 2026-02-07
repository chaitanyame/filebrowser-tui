//! Bulk rename functionality with pattern support.
//!
//! Provides various renaming patterns including:
//! - Simple string replacement
//! - Numbered sequences
//! - Regex-based substitution
//! - Case transformations
//! - Extension manipulation

use anyhow::{Context, Result};
use regex::Regex;
use std::path::{Path, PathBuf};

/// Pattern types for bulk renaming operations.
#[derive(Debug, Clone, PartialEq)]
pub enum RenamePattern {
    /// Simple string replacement: replace all occurrences of `find` with `replace`
    SimpleReplace { find: String, replace: String },

    /// Numbered pattern: rename files using a template with `{n}` placeholder
    /// Example: "photo_{n}.jpg" produces "photo_1.jpg", "photo_2.jpg", etc.
    Numbered {
        template: String,
        start: usize,
        pad_width: usize,
    },

    /// Regex substitution with capture groups
    /// Example: r"(.*)\.(txt)" -> r"$1_backup.$2"
    Regex { pattern: String, replacement: String },

    /// Case transformation
    Case {
        transform: CaseTransform,
        scope: CaseScope,
    },

    /// Extension manipulation
    Extension {
        action: ExtensionAction,
        new_extension: Option<String>,
    },
}

/// Case transformation types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseTransform {
    Uppercase,
    Lowercase,
    TitleCase,
    SentenceCase,
    ToggleCase,
}

/// Scope for case transformation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseScope {
    NameOnly,
    ExtensionOnly,
    EntireName,
}

/// Extension manipulation actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionAction {
    Add,
    Remove,
    Replace,
    Keep,
}

/// Preview of a single rename operation.
#[derive(Debug, Clone)]
pub struct RenamePreview {
    /// Original file path
    pub old_path: PathBuf,
    /// New file path (after renaming)
    pub new_path: PathBuf,
    /// Whether this rename operation is valid
    pub is_valid: bool,
    /// Error message if invalid
    pub error: Option<String>,
    /// Whether this specific rename is accepted (for individual control)
    pub accepted: bool,
}

impl RenamePreview {
    /// Create a new rename preview.
    pub fn new(old_path: PathBuf, new_path: PathBuf) -> Self {
        let is_valid = Self::validate_rename(&old_path, &new_path);
        let error = if is_valid {
            None
        } else {
            Some("Invalid or dangerous rename".to_string())
        };

        Self {
            old_path,
            new_path,
            is_valid,
            error,
            accepted: true,
        }
    }

    /// Validate that a rename operation is safe.
    fn validate_rename(old_path: &Path, new_path: &Path) -> bool {
        // Don't allow empty names
        let new_name = match new_path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => return false,
        };

        if new_name.is_empty() {
            return false;
        }

        // Check for invalid characters
        if new_name.contains(|c| matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')) {
            return false;
        }

        // Don't allow renaming to same name
        if old_path.file_name() == new_path.file_name() {
            return false;
        }

        true
    }

    /// Get the old file name (without path).
    pub fn old_name(&self) -> String {
        self.old_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    /// Get the new file name (without path).
    pub fn new_name(&self) -> String {
        self.new_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }
}

/// Bulk renamer that handles pattern-based renaming.
pub struct BulkRenamer {
    pattern: RenamePattern,
    base_path: PathBuf,
}

impl BulkRenamer {
    /// Create a new bulk renamer with the given pattern.
    pub fn new(pattern: RenamePattern, base_path: PathBuf) -> Self {
        Self { pattern, base_path }
    }

    /// Generate a preview of all rename operations.
    pub fn preview(&self, files: &[PathBuf]) -> Vec<RenamePreview> {
        files
            .iter()
            .enumerate()
            .map(|(idx, file)| {
                let new_name = match self.apply_pattern(file, idx) {
                    Ok(name) => name,
                    Err(e) => {
                        return RenamePreview {
                            old_path: file.clone(),
                            new_path: file.clone(),
                            is_valid: false,
                            error: Some(e.to_string()),
                            accepted: true,
                        }
                    }
                };

                let new_path = self.base_path.join(&new_name);
                RenamePreview::new(file.clone(), new_path)
            })
            .collect()
    }

    /// Apply the rename pattern to a single file.
    fn apply_pattern(&self, path: &Path, index: usize) -> Result<String> {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid file name")?;

        let (stem, ext) = split_name_and_extension(file_name);

        let result = match &self.pattern {
            RenamePattern::SimpleReplace { find, replace } => {
                if find.is_empty() {
                    return Ok(file_name.to_string());
                }
                file_name.replace(find, replace)
            }

            RenamePattern::Numbered {
                template,
                start,
                pad_width,
            } => {
                let num = start + index;
                let num_str = if *pad_width > 0 {
                    format!("{:0width$}", num, width = pad_width)
                } else {
                    num.to_string()
                };

                template.replace("{n}", &num_str)
            }

            RenamePattern::Regex {
                pattern,
                replacement,
            } => {
                let regex = Regex::new(pattern)
                    .context("Invalid regex pattern")?;
                regex.replace(file_name, replacement.as_str()).to_string()
            }

            RenamePattern::Case { transform, scope } => {
                let (target_stem, target_ext) = match scope {
                    CaseScope::NameOnly => (stem, ext.as_deref()),
                    CaseScope::ExtensionOnly => ("", ext.map(|e| e.as_str()).as_deref()),
                    CaseScope::EntireName => (
                        if ext.is_some() { stem } else { file_name },
                        if ext.is_some() { None } else { ext.as_deref() },
                    ),
                };

                let transformed = apply_case_transform(target_stem, *transform);

                match scope {
                    CaseScope::NameOnly => {
                        if let Some(ext) = ext {
                            format!("{}.{}", transformed, ext)
                        } else {
                            transformed
                        }
                    }
                    CaseScope::ExtensionOnly => {
                        if stem.is_empty() {
                            transformed
                        } else {
                            format!("{}.{}", stem, transformed)
                        }
                    }
                    CaseScope::EntireName => {
                        if ext.is_some() {
                            transformed
                        } else {
                            format!("{}.{}", transformed, ext.unwrap_or(""))
                        }
                    }
                }
            }

            RenamePattern::Extension {
                action,
                new_extension,
            } => match action {
                ExtensionAction::Add => {
                    if ext.is_some() {
                        file_name.to_string()
                    } else if let Some(new_ext) = new_extension {
                        format!("{}.{}", stem, new_ext)
                    } else {
                        file_name.to_string()
                    }
                }
                ExtensionAction::Remove => stem.to_string(),
                ExtensionAction::Replace => {
                    if let Some(new_ext) = new_extension {
                        format!("{}.{}", stem, new_ext)
                    } else {
                        file_name.to_string()
                    }
                }
                ExtensionAction::Keep => file_name.to_string(),
            },
        };

        Ok(result)
    }

    /// Execute the bulk rename operation.
    pub fn execute(&self, previews: &mut [RenamePreview]) -> Result<usize> {
        let mut renamed_count = 0;

        // First pass: check for conflicts and invalid operations
        let new_paths: std::collections::HashSet<PathBuf> = previews
            .iter()
            .filter_map(|p| if p.accepted && p.is_valid { Some(&p.new_path) } else { None })
            .cloned()
            .collect();

        // Check for duplicates in target names
        let mut seen = std::collections::HashSet::new();
        for preview in &previews {
            if preview.accepted && preview.is_valid {
                if !seen.insert(&preview.new_path) {
                    return Err(anyhow::anyhow!(
                        "Duplicate target name: {}",
                        preview.new_name()
                    ));
                }
            }
        }

        // Execute renames
        for preview in previews.iter_mut() {
            if !preview.accepted || !preview.is_valid {
                continue;
            }

            // Skip if source doesn't exist
            if !preview.old_path.exists() {
                preview.is_valid = false;
                preview.error = Some("Source file not found".to_string());
                continue;
            }

            // Skip if target already exists (and is different from source)
            if preview.new_path.exists() && preview.old_path != preview.new_path {
                preview.is_valid = false;
                preview.error = Some("Target already exists".to_string());
                continue;
            }

            // Perform the rename
            match std::fs::rename(&preview.old_path, &preview.new_path) {
                Ok(_) => renamed_count += 1,
                Err(e) => {
                    preview.is_valid = false;
                    preview.error = Some(format!("Rename failed: {}", e));
                }
            }
        }

        Ok(renamed_count)
    }
}

/// Split a file name into stem (without extension) and extension.
fn split_name_and_extension(name: &str) -> (&str, Option<&str>) {
    if let Some(pos) = name.rfind('.') {
        let (stem, ext) = name.split_at(pos);
        (stem, Some(&ext[1..]))
    } else {
        (name, None)
    }
}

/// Apply a case transformation to a string.
fn apply_case_transform(s: &str, transform: CaseTransform) -> String {
    match transform {
        CaseTransform::Uppercase => s.to_uppercase(),
        CaseTransform::Lowercase => s.to_lowercase(),
        CaseTransform::TitleCase => {
            s.split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => {
                            first.to_uppercase().collect::<String>() + chars.as_str()
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
        CaseTransform::SentenceCase => {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str()
                }
            }
        }
        CaseTransform::ToggleCase => {
            s.chars()
                .map(|c| {
                    if c.is_uppercase() {
                        c.to_lowercase().collect::<String>()
                    } else if c.is_lowercase() {
                        c.to_uppercase().collect::<String>()
                    } else {
                        c.to_string()
                    }
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_name_and_extension() {
        assert_eq!(split_name_and_extension("file.txt"), ("file", Some("txt")));
        assert_eq!(split_name_and_extension("archive.tar.gz"), ("archive", Some("tar.gz")));
        assert_eq!(split_name_and_extension("noext"), ("noext", None));
        assert_eq!(split_name_and_extension(".hidden"), ("", Some("hidden")));
    }

    #[test]
    fn test_simple_replace() {
        let pattern = RenamePattern::SimpleReplace {
            find: "old".to_string(),
            replace: "new".to_string(),
        };
        let renamer = BulkRenamer::new(pattern, PathBuf::from("/tmp"));

        let files = vec![
            PathBuf::from("/tmp/old_file.txt"),
            PathBuf::from("/tmp/another_old.txt"),
        ];

        let previews = renamer.preview(&files);
        assert_eq!(previews[0].new_name(), "new_file.txt");
        assert_eq!(previews[1].new_name(), "another_new.txt");
    }

    #[test]
    fn test_numbered_pattern() {
        let pattern = RenamePattern::Numbered {
            template: "photo_{n}.jpg".to_string(),
            start: 1,
            pad_width: 3,
        };
        let renamer = BulkRenamer::new(pattern, PathBuf::from("/tmp"));

        let files = vec![
            PathBuf::from("/tmp/img1.jpg"),
            PathBuf::from("/tmp/img2.jpg"),
            PathBuf::from("/tmp/img3.jpg"),
        ];

        let previews = renamer.preview(&files);
        assert_eq!(previews[0].new_name(), "photo_001.jpg");
        assert_eq!(previews[1].new_name(), "photo_002.jpg");
        assert_eq!(previews[2].new_name(), "photo_003.jpg");
    }

    #[test]
    fn test_case_transform() {
        let pattern = RenamePattern::Case {
            transform: CaseTransform::Uppercase,
            scope: CaseScope::NameOnly,
        };
        let renamer = BulkRenamer::new(pattern, PathBuf::from("/tmp"));

        let files = vec![PathBuf::from("/tmp/lowercase.txt")];
        let previews = renamer.preview(&files);
        assert_eq!(previews[0].new_name(), "LOWERCASE.txt");
    }

    #[test]
    fn test_extension_replace() {
        let pattern = RenamePattern::Extension {
            action: ExtensionAction::Replace,
            new_extension: Some("md".to_string()),
        };
        let renamer = BulkRenamer::new(pattern, PathBuf::from("/tmp"));

        let files = vec![PathBuf::from("/tmp/document.txt")];
        let previews = renamer.preview(&files);
        assert_eq!(previews[0].new_name(), "document.md");
    }
}
