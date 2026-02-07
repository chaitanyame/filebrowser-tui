use anyhow::{Context, Result};
use regex::Regex;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::task::JoinHandle;

/// Represents a single search match result within a file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    /// Path to the file containing the match
    pub file_path: PathBuf,
    /// Line number (1-indexed) where the match was found
    pub line_number: usize,
    /// The full content of the line containing the match
    pub line_content: String,
    /// Byte index within line_content where the match starts
    pub match_start: usize,
    /// Byte index within line_content where the match ends (exclusive)
    pub match_end: usize,
}

impl SearchResult {
    /// Returns the matched text as a string slice
    pub fn matched_text(&self) -> &str {
        &self.line_content[self.match_start..self.match_end]
    }

    /// Returns the path relative to a base directory for display purposes
    pub fn relative_path(&self, base: &Path) -> String {
        self.file_path
            .strip_prefix(base)
            .unwrap_or(&self.file_path)
            .display()
            .to_string()
    }
}

/// Progress information for an ongoing search operation
#[derive(Debug, Clone)]
pub struct SearchProgress {
    /// Number of files searched so far
    pub files_searched: usize,
    /// Total number of files to search (if known)
    pub total_files: Option<usize>,
    /// Number of matches found so far
    pub matches_found: usize,
    /// Current file being searched (if any)
    pub current_file: Option<PathBuf>,
}

/// Configuration for content search operations
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Whether the search should be case-sensitive
    pub case_sensitive: bool,
    /// Whether to use regex pattern matching
    pub use_regex: bool,
    /// Maximum number of results to return (None for unlimited)
    pub max_results: Option<usize>,
    /// File extensions to include (None for all files)
    pub include_extensions: Option<Vec<String>>,
    /// File extensions to exclude
    pub exclude_extensions: Vec<String>,
    /// Maximum file size to search in bytes (None for unlimited)
    pub max_file_size: Option<u64>,
    /// Number of lines of context to include around matches
    pub context_lines: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig {
            case_sensitive: false,
            use_regex: false,
            max_results: None,
            include_extensions: None,
            exclude_extensions: vec![
                "exe".to_string(),
                "dll".to_string(),
                "so".to_string(),
                "dylib".to_string(),
                "bin".to_string(),
                "zip".to_string(),
                "tar".to_string(),
                "gz".to_string(),
                "7z".to_string(),
                "rar".to_string(),
                "pdf".to_string(),
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "mp3".to_string(),
                "mp4".to_string(),
                "wav".to_string(),
            ],
            max_file_size: Some(10 * 1024 * 1024), // 10 MB default
            context_lines: 0,
        }
    }
}

/// Async content searcher with progress reporting support
pub struct ContentSearcher {
    /// Search configuration
    config: SearchConfig,
    /// Cancellation token for stopping the search
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Callback for progress updates
    progress_callback: Option<Box<dyn Fn(SearchProgress) + Send + Sync>>,
}

impl ContentSearcher {
    /// Create a new ContentSearcher with default configuration
    pub fn new() -> Self {
        ContentSearcher {
            config: SearchConfig::default(),
            cancelled: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            progress_callback: None,
        }
    }

    /// Create a new ContentSearcher with custom configuration
    pub fn with_config(config: SearchConfig) -> Self {
        ContentSearcher {
            config,
            cancelled: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            progress_callback: None,
        }
    }

    /// Set the progress callback for search operations
    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(SearchProgress) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Search for a pattern in the given files
    /// Returns a handle to the async task that produces the results
    pub fn search_files(
        &self,
        files: Vec<PathBuf>,
        pattern: &str,
        base_path: PathBuf,
    ) -> JoinHandle<Result<Vec<SearchResult>>> {
        let pattern = pattern.to_string();
        let config = self.config.clone();
        let cancelled = self.cancelled.clone();
        let base = base_path.clone();
        let progress_callback = self.progress_callback.clone();

        tokio::spawn(async move {
            let mut results = Vec::new();
            let total_files = files.len();
            let mut files_searched = 0;
            let mut matches_found = 0;

            // Build the search pattern
            let search_regex = if config.use_regex {
                Regex::new(&pattern)
                    .with_context(|| format!("Invalid regex pattern: {}", pattern))?
            } else {
                // Escape special regex characters for literal search
                let escaped = regex::escape(&pattern);
                let flags = if config.case_sensitive { "" } else { "(?i)" };
                Regex::new(&format!("{}{}", flags, &escaped))
                    .with_context(|| format!("Failed to build regex from pattern: {}", pattern))?
            };

            for file_path in files {
                // Check for cancellation
                if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                // Report progress
                files_searched += 1;
                if let Some(ref callback) = progress_callback {
                    callback(SearchProgress {
                        files_searched,
                        total_files: Some(total_files),
                        matches_found,
                        current_file: Some(file_path.clone()),
                    });
                }

                // Skip if we've hit max results
                if let Some(max) = config.max_results {
                    if matches_found >= max {
                        break;
                    }
                }

                // Skip files that don't match extension filters
                if !self.should_search_file(&file_path, &config) {
                    continue;
                }

                // Check file size
                if let Ok(metadata) = tokio::fs::metadata(&file_path).await {
                    if let Some(max_size) = config.max_file_size {
                        if metadata.len() > max_size {
                            continue;
                        }
                    }
                    // Skip directories
                    if metadata.is_dir() {
                        continue;
                    }
                }

                // Search the file
                match Self::search_file(&file_path, &search_regex, &config).await {
                    Ok(mut file_results) => {
                        matches_found += file_results.len();
                        results.append(&mut file_results);
                    }
                    Err(e) => {
                        // Log error but continue searching other files
                        eprintln!("Error searching file {:?}: {}", file_path, e);
                    }
                }
            }

            // Final progress update
            if let Some(ref callback) = progress_callback {
                callback(SearchProgress {
                    files_searched,
                    total_files: Some(total_files),
                    matches_found,
                    current_file: None,
                });
            }

            Ok(results)
        })
    }

    /// Search for a pattern in a single file
    async fn search_file(
        file_path: &PathBuf,
        pattern: &Regex,
        config: &SearchConfig,
    ) -> Result<Vec<SearchResult>> {
        let file = File::open(file_path).await
            .with_context(|| format!("Failed to open file: {}", file_path.display()))?;

        let reader = BufReader::new(file);
        let mut results = Vec::new();
        let mut lines = reader.lines();

        let mut line_number = 0;
        while let Some(line_result) = lines.next_line().await? {
            line_number += 1;

            // Check for cancellation
            if config.max_results.is_some() && results.len() >= config.max_results.unwrap() {
                break;
            }

            // Search for matches in this line
            for mat in pattern.find_iter(&line_result) {
                results.push(SearchResult {
                    file_path: file_path.clone(),
                    line_number,
                    line_content: line_result.clone(),
                    match_start: mat.start(),
                    match_end: mat.end(),
                });
            }
        }

        Ok(results)
    }

    /// Check if a file should be searched based on extension filters
    fn should_search_file(&self, file_path: &Path, config: &SearchConfig) -> bool {
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Check exclude list
        if config.exclude_extensions.contains(&extension) {
            return false;
        }

        // Check include list (if specified)
        if let Some(ref include) = config.include_extensions {
            include.iter().any(|e| e.to_lowercase() == extension)
        } else {
            true
        }
    }

    /// Cancel the ongoing search operation
    pub fn cancel(&self) {
        self.cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Reset the cancellation flag (allows reusing the searcher)
    pub fn reset(&self) {
        self.cancelled.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for ContentSearcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to search files synchronously (blocks until complete)
pub async fn search_files_sync(
    files: Vec<PathBuf>,
    pattern: &str,
    base_path: PathBuf,
    config: Option<SearchConfig>,
) -> Result<Vec<SearchResult>> {
    let searcher = if let Some(cfg) = config {
        ContentSearcher::with_config(cfg)
    } else {
        ContentSearcher::new()
    };

    let handle = searcher.search_files(files, pattern, base_path);
    handle.await?
}

/// Recursively collect all files in a directory (excluding directories themselves)
pub async fn collect_files_in_directory(
    dir_path: &Path,
    recursive: bool,
    max_depth: Option<usize>,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if !dir_path.exists() {
        return Ok(files);
    }

    let mut entries = tokio::fs::read_dir(dir_path).await
        .with_context(|| format!("Failed to read directory: {}", dir_path.display()))?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let file_type = entry.file_type().await?;

        if file_type.is_dir() && recursive {
            let should_recurse = if let Some(max) = max_depth {
                // Calculate depth by counting path separators
                let depth = path.components().count() - dir_path.components().count();
                depth < max
            } else {
                true
            };

            if should_recurse {
                let mut subdir_files = collect_files_in_directory(&path, recursive, max_depth).await?;
                files.append(&mut subdir_files);
            }
        } else if !file_type.is_dir() {
            files.push(path);
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_search_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        write(&test_file, b"Hello World\nThis is a test\nAnother line").unwrap();

        let pattern = Regex::new("test").unwrap();
        let results = ContentSearcher::search_file(&test_file, &pattern, &SearchConfig::default())
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_number, 2);
        assert_eq!(results[0].matched_text(), "test");
    }

    #[tokio::test]
    async fn test_collect_files() {
        let temp_dir = TempDir::new().unwrap();
        write(temp_dir.path().join("file1.txt"), b"content").unwrap();
        write(temp_dir.path().join("file2.rs"), b"content").unwrap();

        let files = collect_files_in_directory(temp_dir.path(), true, None).await.unwrap();
        assert_eq!(files.len(), 2);
    }
}
