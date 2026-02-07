//! Common test utilities for integration and snapshot testing
//!
//! Shared utilities for end-to-end tests including assertion helpers,
//! retry logic, and test helpers.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use filebrowser_tui::state::{FileEntry, SortBy, SortOrder};

// ============================================================================
// Original Test Utilities
// ============================================================================

/// Create a test file entry
pub fn make_test_file(
    name: &str,
    is_dir: bool,
    size: u64,
    is_hidden: bool,
) -> FileEntry {
    let mut path = PathBuf::from("/test");
    path.push(name);

    FileEntry {
        name: name.to_string(),
        path,
        is_dir,
        size,
        modified: SystemTime::UNIX_EPOCH,
        is_hidden,
        is_system: false,
        is_readonly: false,
        is_symlink: false,
    }
}

/// Create a set of test files
pub fn make_test_files() -> Vec<FileEntry> {
    vec![
        make_test_file("Documents", true, 0, false),
        make_test_file("Downloads", true, 0, false),
        make_test_file("Pictures", true, 0, false),
        make_test_file("Music", true, 0, false),
        make_test_file("Videos", true, 0, false),
        make_test_file(".hidden_dir", true, 0, true),
        make_test_file("file1.txt", false, 1024, false),
        make_test_file("file2.txt", false, 2048, false),
        make_test_file("file3.txt", false, 4096, false),
        make_test_file("image.png", false, 524288, false),
        make_test_file("document.pdf", false, 1048576, false),
        make_test_file(".hidden_file", false, 512, true),
        make_test_file("large_file.bin", false, 104857600, false),
    ]
}

/// Sort test files by name (ascending)
pub fn sort_test_files(files: &mut Vec<FileEntry>) {
    filebrowser_tui::state::files::sort_files(
        files,
        SortBy::Name,
        SortOrder::Ascending,
    );
}

// ============================================================================
// E2E Test Utilities
// ============================================================================

/// Retry configuration for operations that may need multiple attempts
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Delay between attempts
    pub delay: Duration,
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            delay: Duration::from_millis(100),
            exponential_backoff: true,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of attempts
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Set the delay between attempts
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Enable or disable exponential backoff
    pub fn with_exponential_backoff(mut self, enabled: bool) -> Self {
        self.exponential_backoff = enabled;
        self
    }

    /// Quick retry config for fast operations
    pub fn quick() -> Self {
        Self {
            max_attempts: 5,
            delay: Duration::from_millis(50),
            exponential_backoff: false,
        }
    }

    /// Slow retry config for slower operations
    pub fn slow() -> Self {
        Self {
            max_attempts: 10,
            delay: Duration::from_millis(500),
            exponential_backoff: true,
        }
    }
}

/// Retry an operation with the given configuration
///
/// # Arguments
///
/// * `operation` - Function to retry (should return Err to trigger retry)
/// * `config` - Retry configuration
///
/// # Example
///
/// ```no_run
/// # use anyhow::Result;
/// # use tests::common::{retry, RetryConfig};
/// # fn main() -> Result<()> {
/// let result = retry(
///     || {
///         // Some operation that might fail
///         Ok::<(), anyhow::Error>(())
///     },
///     RetryConfig::default()
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn retry<F, T>(mut operation: F, config: RetryConfig) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let mut attempt = 0;
    let mut delay = config.delay;

    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;
                if attempt >= config.max_attempts {
                    return Err(e.context(format!("Failed after {} attempts", attempt)));
                }

                if config.exponential_backoff {
                    delay *= 2;
                }

                std::thread::sleep(delay);
            }
        }
    }
}

/// Wait for a condition to become true with timeout
///
/// # Arguments
///
/// * `condition` - Function that returns true when condition is met
/// * `timeout` - Maximum time to wait
/// * `check_interval` - Time between checks
///
/// # Example
///
/// ```no_run
/// # use anyhow::Result;
/// # use tests::common::wait_for;
/// # use std::time::Duration;
/// # fn main() -> Result<()> {
/// let file_exists = wait_for(
///     || std::path::Path::new("/tmp/test.txt").exists(),
///     Duration::from_secs(5),
///     Duration::from_millis(100)
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn wait_for<F>(mut condition: F, timeout: Duration, check_interval: Duration) -> Result<bool>
where
    F: FnMut() -> bool,
{
    let start = Instant::now();

    while start.elapsed() < timeout {
        if condition() {
            return Ok(true);
        }
        std::thread::sleep(check_interval);
    }

    Ok(false)
}

/// Assert that a condition becomes true within timeout
///
/// # Arguments
///
/// * `condition` - Function that returns true when condition is met
/// * `timeout` - Maximum time to wait
/// * `message` - Error message if condition never becomes true
///
/// # Panics
///
/// Panics if condition doesn't become true within timeout
pub fn assert_eventually<F>(mut condition: F, timeout: Duration, message: &str)
where
    F: FnMut() -> bool,
{
    let start = Instant::now();
    let check_interval = Duration::from_millis(50);

    while start.elapsed() < timeout {
        if condition() {
            return;
        }
        std::thread::sleep(check_interval);
    }

    panic!("Assertion failed: {} (timeout: {:?})", message, timeout);
}

/// Assert that a file exists
pub fn assert_file_exists<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if path.exists() {
        if path.is_file() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Path exists but is not a file: {:?}", path))
        }
    } else {
        Err(anyhow::anyhow!("File does not exist: {:?}", path))
    }
}

/// Assert that a file does not exist
pub fn assert_file_not_exists<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("File exists but shouldn't: {:?}", path))
    }
}

/// Assert that a directory exists
pub fn assert_dir_exists<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if path.exists() {
        if path.is_dir() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Path exists but is not a directory: {:?}", path))
        }
    } else {
        Err(anyhow::anyhow!("Directory does not exist: {:?}", path))
    }
}

/// Assert file content matches expected value
pub fn assert_file_content<P: AsRef<Path>>(path: P, expected: &str) -> Result<()> {
    let path = path.as_ref();
    let actual = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {:?}", path))?;

    if actual == expected {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "File content mismatch for {:?}\nExpected:\n{}\n\nActual:\n{}",
            path,
            expected,
            actual
        ))
    }
}

/// Assert file contains specific text
pub fn assert_file_contains<P: AsRef<Path>>(path: P, text: &str) -> Result<()> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {:?}", path))?;

    if content.contains(text) {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "File {:?} does not contain expected text: {:?}\nActual content:\n{}",
            path,
            text,
            content
        ))
    }
}

/// Measure execution time of a function
///
/// Returns the result and the duration
pub fn measure_time<F, T>(f: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

/// Assert that operation completes within timeout
///
/// # Panics
///
/// Panics if operation takes longer than timeout
pub fn assert_completes_within<F, T>(f: F, timeout: Duration, message: &str) -> T
where
    F: FnOnce() -> T,
{
    let (result, duration) = measure_time(f);

    if duration > timeout {
        panic!(
            "{}: took {:?} which exceeds timeout of {:?}",
            message, duration, timeout
        );
    }

    result
}

/// Get file size in bytes
pub fn file_size<P: AsRef<Path>>(path: P) -> Result<u64> {
    let path = path.as_ref();
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("Failed to get metadata for: {:?}", path))?;
    Ok(metadata.len())
}

/// Assert file size is within expected range
pub fn assert_file_size_between<P: AsRef<Path>>(
    path: P,
    min: u64,
    max: u64
) -> Result<()> {
    let size = file_size(path)?;

    if size >= min && size <= max {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "File size {} is not within range [{}, {}]",
            size,
            min,
            max
        ))
    }
}

/// Normalize path separators for cross-platform testing
pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Compare paths ignoring separator differences
pub fn paths_equal<P: AsRef<Path>, Q: AsRef<Path>>(path1: P, path2: Q) -> bool {
    normalize_path(&path1.as_ref().to_string_lossy())
        == normalize_path(&path2.as_ref().to_string_lossy())
}

/// Test context manager
///
/// Provides setup/teardown for test fixtures
pub struct TestContext {
    /// Name of the test
    pub test_name: String,
    /// Test fixtures directory
    fixtures_dir: PathBuf,
    /// Cleanup flag
    cleanup_on_drop: bool,
}

impl TestContext {
    /// Create a new test context
    pub fn new(test_name: &str) -> Self {
        let fixtures_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("test_fixtures")
            .join(test_name);

        Self {
            test_name: test_name.to_string(),
            fixtures_dir,
            cleanup_on_drop: true,
        }
    }

    /// Get the fixtures directory path
    pub fn fixtures_dir(&self) -> &Path {
        &self.fixtures_dir
    }

    /// Create the fixtures directory
    pub fn setup(&self) -> Result<()> {
        std::fs::create_dir_all(&self.fixtures_dir)
            .with_context(|| format!("Failed to create fixtures directory: {:?}", self.fixtures_dir))?;
        Ok(())
    }

    /// Clean up the fixtures directory
    pub fn teardown(&self) -> Result<()> {
        if self.fixtures_dir.exists() {
            std::fs::remove_dir_all(&self.fixtures_dir)
                .with_context(|| format!("Failed to remove fixtures directory: {:?}", self.fixtures_dir))?;
        }
        Ok(())
    }

    /// Disable automatic cleanup on drop
    pub fn retain_on_drop(&mut self) {
        self.cleanup_on_drop = false;
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        if self.cleanup_on_drop {
            let _ = self.teardown();
        }
    }
}

/// Helper to run a test with automatic cleanup
///
/// # Example
///
/// ```no_run
/// # use anyhow::Result;
/// # use tests::common::with_test_context;
/// # fn main() -> Result<()> {
/// with_test_context("my_test", |ctx| {
///     // Use ctx.fixtures_dir() for test files
///     Ok(())
/// })?;
/// # Ok(())
/// # }
/// ```
pub fn with_test_context<F>(test_name: &str, f: F) -> Result<()>
where
    F: FnOnce(&TestContext) -> Result<()>,
{
    let ctx = TestContext::new(test_name);
    ctx.setup()?;

    // Ensure cleanup happens even if test fails
    let result = f(&ctx);

    let cleanup_result = ctx.teardown();

    result.and(cleanup_result)
}

/// Get environment variable or default value
pub fn env_var_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Check if running in CI environment
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("CONTINUOUS_INTEGRATION").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
        || std::env::var("TRAVIS").is_ok()
}

/// Get timeout multiplier for slower CI environments
pub fn timeout_multiplier() -> f64 {
    if is_ci() {
        3.0 // Triple timeout in CI
    } else {
        1.0
    }
}

/// Adjust duration by CI timeout multiplier
pub fn adjust_timeout(duration: Duration) -> Duration {
    Duration::from_millis((duration.as_millis() as f64 * timeout_multiplier()) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_success() {
        let mut attempts = 0;
        let result = retry(
            || {
                attempts += 1;
                if attempts < 3 {
                    Err(anyhow::anyhow!("Not yet"))
                } else {
                    Ok("Success")
                }
            },
            RetryConfig::default()
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(attempts, 3);
    }

    #[test]
    fn test_retry_failure() {
        let result = retry(
            || Err::<(), _>(anyhow::anyhow!("Always fails")),
            RetryConfig::default().with_max_attempts(2)
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_wait_for() {
        let start = Instant::now();
        let result = wait_for(
            || start.elapsed() > Duration::from_millis(100),
            Duration::from_secs(1),
            Duration::from_millis(10)
        );

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("foo\\bar\\baz"), "foo/bar/baz");
        assert_eq!(normalize_path("foo/bar/baz"), "foo/bar/baz");
    }

    #[test]
    fn test_paths_equal() {
        assert!(paths_equal("foo/bar", "foo\\bar"));
        assert!(!paths_equal("foo/bar", "foo/baz"));
    }

    #[test]
    fn test_adjust_timeout() {
        let duration = Duration::from_millis(100);
        let adjusted = adjust_timeout(duration);

        if is_ci() {
            assert!(adjusted >= duration);
        } else {
            assert_eq!(adjusted, duration);
        }
    }
}
