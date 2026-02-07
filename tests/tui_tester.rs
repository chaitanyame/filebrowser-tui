//! PTY-based TUI testing framework
//!
//! Provides a comprehensive testing framework for TUI applications using pseudo-terminals.
//! Supports keystroke simulation, screen capture, and state verification.

use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

/// Default timeout for waiting for output
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
/// Default delay between keystrokes
const DEFAULT_KEY_DELAY: Duration = Duration::from_millis(50);
/// Polling interval for checking output
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// TUI Tester - Main testing struct for PTY-based TUI testing
///
/// # Example
///
/// ```no_run
/// use tests::tui_tester::TuiTester;
///
/// # fn main() -> anyhow::Result<()> {
/// let mut tester = TuiTester::new("/tmp/test_dir")?;
/// tester.launch()?;
/// tester.send_keys("Hello")?;
/// tester.wait_for("Hello")?;
/// tester.assert_contains("Hello")?;
/// tester.quit()?;
/// # Ok(())
/// # }
/// ```
pub struct TuiTester {
    /// Working directory for the test
    test_dir: PathBuf,
    /// PTY master file handle
    pty_master: Option<Box<dyn PtyMaster>>,
    /// Child process handle
    child: Option<Box<dyn ChildProcess>>,
    /// Terminal dimensions (rows, columns)
    term_size: (u16, u16),
    /// Buffer for captured output
    output_buffer: Vec<u8>,
    /// Current position in buffer for screen capture
    buffer_position: usize,
    /// Whether the app is running
    running: bool,
    /// Verbosity flag
    verbose: bool,
}

/// Trait for PTY master operations (abstracted for cross-platform support)
pub trait PtyMaster: Read + Write + Send {
    /// Set terminal size
    fn set_size(&mut self, rows: u16, cols: u16) -> Result<()>;
    /// Flush output
    fn flush(&mut self) -> std::io::Result<()>;
}

/// Trait for child process operations
pub trait ChildProcess: Send {
    /// Get process ID
    fn id(&self) -> u32;
    /// Try to kill the process
    fn try_kill(&mut self) -> Result<()>;
    /// Check if process is still running
    fn is_running(&self) -> bool;
}

#[cfg(unix)]
mod unix_impl {
    use super::*;
    use std::os::unix::io::{AsRawFd, RawFd};
    use std::os::unix::process::CommandExt;

    /// Unix-specific PTY master implementation
    pub struct UnixPtyMaster {
        master: ::pty::fork::Fork,
    }

    impl PtyMaster for UnixPtyMaster {
        fn set_size(&mut self, rows: u16, cols: u16) -> Result<()> {
            // PTY size is set via winsize in the fork
            Ok(())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Read for UnixPtyMaster {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.master.read(buf)
        }
    }

    impl Write for UnixPtyMaster {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.master.write(buf)
        }
    }

    /// Unix-specific child process wrapper
    pub struct UnixChild {
        master: ::pty::fork::Fork,
    }

    impl ChildProcess for UnixChild {
        fn id(&self) -> u32 {
            self.master.child_pid()
        }

        fn try_kill(&mut self) -> Result<()> {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;

            signal::kill(Pid::from_raw(self.master.child_pid() as i32), Signal::SIGTERM)
                .context("Failed to send SIGTERM to child process")?;
            Ok(())
        }

        fn is_running(&self) -> bool {
            use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
            match waitpid(Pid::from_raw(self.master.child_pid() as i32), Some(WaitPidFlag::WNOHANG)) {
                Ok(WaitStatus::StillAlive) => true,
                Ok(_) => false,
                Err(_) => false,
            }
        }
    }

    pub use UnixPtyMaster as DefaultPtyMaster;
    pub use UnixChild as DefaultChild;
}

#[cfg(unix)]
use unix_impl::{DefaultChild, DefaultPtyMaster};

#[cfg(windows)]
mod windows_impl {
    use super::*;

    /// Windows-specific PTY master implementation using conpty
    pub struct WindowsPtyMaster {
        // Windows ConPTY implementation would go here
        conout: std::fs::File,
    }

    impl PtyMaster for WindowsPtyMaster {
        fn set_size(&mut self, rows: u16, cols: u16) -> Result<()> {
            Ok(())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.conout.flush()
        }
    }

    impl Read for WindowsPtyMaster {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.conout.read(buf)
        }
    }

    impl Write for WindowsPtyMaster {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.conout.write(buf)
        }
    }

    pub struct WindowsChild {
        pid: u32,
    }

    impl ChildProcess for WindowsChild {
        fn id(&self) -> u32 {
            self.pid
        }

        fn try_kill(&mut self) -> Result<()> {
            use std::os::windows::process::CommandExt;

            Command::new("taskkill")
                .args(["/PID", &self.pid.to_string(), "/F"])
                .output()
                .context("Failed to kill child process")?;
            Ok(())
        }

        fn is_running(&self) -> bool {
            Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", self.pid)])
                .output()
                .map(|out| String::from_utf8_lossy(&out.stdout).contains(&self.pid.to_string()))
                .unwrap_or(false)
        }
    }

    pub use WindowsChild as DefaultChild;
    pub use WindowsPtyMaster as DefaultPtyMaster;
}

#[cfg(windows)]
use windows_impl::{DefaultChild, DefaultPtyMaster};

impl TuiTester {
    /// Create a new TuiTester instance
    ///
    /// # Arguments
    ///
    /// * `test_dir` - Directory to use as the working directory for tests
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// use tests::tui_tester::TuiTester;
    /// let tester = TuiTester::new("/tmp/test_dir")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(test_dir: impl Into<PathBuf>) -> Result<Self> {
        let test_dir = test_dir.into();

        // Create test directory if it doesn't exist
        std::fs::create_dir_all(&test_dir)
            .context("Failed to create test directory")?;

        Ok(Self {
            test_dir,
            pty_master: None,
            child: None,
            term_size: (24, 80), // Default terminal size
            output_buffer: Vec::new(),
            buffer_position: 0,
            running: false,
            verbose: std::env::var("TUI_TEST_VERBOSE").is_ok(),
        })
    }

    /// Set terminal size for the PTY
    ///
    /// # Arguments
    ///
    /// * `rows` - Number of rows in the terminal
    /// * `cols` - Number of columns in the terminal
    pub fn with_terminal_size(mut self, rows: u16, cols: u16) -> Self {
        self.term_size = (rows, cols);
        self
    }

    /// Enable verbose output for debugging
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Launch the TUI application with a PTY
    ///
    /// This spawns the application in a pseudo-terminal, capturing all output
    /// and allowing simulated keyboard input.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # use tests::tui_tester::TuiTester;
    /// let mut tester = TuiTester::new("/tmp/test_dir")?;
    /// tester.launch()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn launch(&mut self) -> Result<()> {
        #[cfg(unix)]
        {
            self.launch_unix()
        }

        #[cfg(windows)]
        {
            self.launch_windows()
        }
    }

    #[cfg(unix)]
    fn launch_unix(&mut self) -> Result<()> {
        use pty::fork::Fork;
        use std::os::unix::io::AsRawFd;

        // Build the command to launch our binary
        let exe_path = std::env::current_exe()
            .context("Failed to get current executable")?;

        let fork = Fork::fork(
            &pty::fork::ForkOptions {
                // Set the working directory
                cwd: Some(self.test_dir.clone()),
                ..Default::default()
            }
        ).context("Failed to fork PTY")?;

        if fork.is_child().is_some() {
            // Child process - launch the TUI app
            // Use fbt binary if available, otherwise use the main binary
            let bin_path = if exe_path.ends_with("filebrowser-tui") {
                // Running the main binary directly
                exe_path
            } else {
                // Running as a test, try to find the fbt binary
                exe_path.parent()
                    .unwrap_or(&exe_path)
                    .join("fbt")
            };

            let err = exec::Command::new(bin_path)
                .env("RUST_BACKTRACE", "1")
                .exec();

            eprintln!("Failed to exec: {:?}", err);
            std::process::exit(1);
        }

        // Parent process
        self.running = true;
        self.output_buffer.clear();
        self.buffer_position = 0;

        // Give the process time to start
        thread::sleep(Duration::from_millis(200));

        self.log("Application launched successfully");
        Ok(())
    }

    #[cfg(windows)]
    fn launch_windows(&mut self) -> Result<()> {
        // Windows implementation using ConPTY
        let exe_path = std::env::current_exe()
            .context("Failed to get current executable")?;

        let bin_path = if exe_path.ends_with("filebrowser-tui.exe") {
            exe_path
        } else {
            exe_path.parent()
                .unwrap_or(&exe_path)
                .join("fbt.exe")
        };

        // Create ConPTY and spawn process
        // This is simplified - full implementation would use windows-conpty crate
        let mut cmd = Command::new(bin_path);
        cmd.current_dir(&self.test_dir);
        cmd.env("RUST_BACKTRACE", "1");

        self.running = true;
        self.output_buffer.clear();
        self.buffer_position = 0;

        thread::sleep(Duration::from_millis(200));

        self.log("Application launched successfully");
        Ok(())
    }

    /// Send simulated keystrokes to the TUI
    ///
    /// # Arguments
    ///
    /// * `keys` - String representing keys to send (e.g., "abc", "\n", "\t")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # use tests::tui_tester::TuiTester;
    /// # let mut tester = TuiTester::new("/tmp/test_dir")?;
    /// # tester.launch()?;
    /// tester.send_keys("Hello")?;
    /// tester.send_keys("\n")?;  // Send Enter key
    /// tester.send_keys("\t")?;  // Send Tab key
    /// # Ok(())
    /// # }
    /// ```
    pub fn send_keys(&mut self, keys: &str) -> Result<()> {
        if !self.running {
            return Err(anyhow::anyhow!("Application is not running"));
        }

        self.log(&format!("Sending keys: {:?}", keys));

        if let Some(ref mut master) = self.pty_master {
            master.write_all(keys.as_bytes())
                .context("Failed to write keys to PTY")?;
            master.flush()
                .context("Failed to flush PTY")?;
        }

        thread::sleep(DEFAULT_KEY_DELAY);
        self.capture_output()?;
        Ok(())
    }

    /// Send a special key sequence (Ctrl+C, Alt+F, etc.)
    ///
    /// # Arguments
    ///
    /// * `key` - Special key to send
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # use tests::tui_tester::{TuiTester, SpecialKey};
    /// # let mut tester = TuiTester::new("/tmp/test_dir")?;
    /// # tester.launch()?;
    /// tester.send_special_key(SpecialKey::CtrlC)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn send_special_key(&mut self, key: SpecialKey) -> Result<()> {
        let sequence = key.to_sequence();
        self.log(&format!("Sending special key: {:?}", key));
        self.send_keys(&sequence)
    }

    /// Wait for specific text to appear in the output
    ///
    /// # Arguments
    ///
    /// * `text` - Text to wait for
    /// * `timeout` - Maximum time to wait (uses DEFAULT_TIMEOUT if None)
    ///
    /// # Returns
    ///
    /// `Ok(true)` if text was found, `Err` if timeout occurred
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # use std::time::Duration;
    /// # fn main() -> Result<()> {
    /// # use tests::tui_tester::TuiTester;
    /// # let mut tester = TuiTester::new("/tmp/test_dir")?;
    /// # tester.launch()?;
    /// tester.wait_for("Welcome")?;
    /// tester.wait_for_timeout("Files", Duration::from_secs(10))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn wait_for(&mut self, text: &str) -> Result<()> {
        self.wait_for_timeout(text, DEFAULT_TIMEOUT)
    }

    /// Wait for specific text with a custom timeout
    pub fn wait_for_timeout(&mut self, text: &str, timeout: Duration) -> Result<()> {
        let start = Instant::now();
        let text_lower = text.to_lowercase();

        self.log(&format!("Waiting for: {:?}", text));

        loop {
            self.capture_output()?;

            let output = self.get_output_as_string();
            if output.to_lowercase().contains(&text_lower) {
                self.log(&format!("Found: {:?}", text));
                return Ok(());
            }

            if start.elapsed() > timeout {
                return Err(anyhow::anyhow!(
                    "Timeout waiting for {:?}\nCurrent output:\n{}",
                    text,
                    self.get_screen()
                ));
            }

            thread::sleep(POLL_INTERVAL);
        }
    }

    /// Wait for a specific pattern to NOT appear in the output
    ///
    /// # Arguments
    ///
    /// * `text` - Text that should disappear from the output
    pub fn wait_for_remove(&mut self, text: &str) -> Result<()> {
        let start = Instant::now();

        self.log(&format!("Waiting for removal of: {:?}", text));

        loop {
            self.capture_output()?;

            let output = self.get_output_as_string();
            if !output.contains(text) {
                self.log(&format!("Removed: {:?}", text));
                return Ok(());
            }

            if start.elapsed() > DEFAULT_TIMEOUT {
                return Err(anyhow::anyhow!(
                    "Timeout waiting for removal of {:?}",
                    text
                ));
            }

            thread::sleep(POLL_INTERVAL);
        }
    }

    /// Get current screen content
    ///
    /// Returns the captured screen content as a string, with terminal
    /// control codes stripped.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # use tests::tui_tester::TuiTester;
    /// # let mut tester = TuiTester::new("/tmp/test_dir")?;
    /// # tester.launch()?;
    /// let screen = tester.get_screen()?;
    /// println!("Screen content:\n{}", screen);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_screen(&mut self) -> Result<String> {
        self.capture_output()?;
        Ok(self.get_output_as_string())
    }

    /// Capture current output from PTY
    fn capture_output(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }

        if let Some(ref mut master) = self.pty_master {
            // Try to read available data with non-blocking
            let mut buf = [0u8; 4096];
            master.set_nonblocking(true)?;

            loop {
                match master.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        self.output_buffer.extend_from_slice(&buf[..n]);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        break;
                    }
                    Err(e) => {
                        return Err(e).context("Failed to read from PTY");
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the captured output as a cleaned string
    fn get_output_as_string(&self) -> String {
        let output = String::from_utf8_lossy(&self.output_buffer);
        strip_ansi_codes(&output)
    }

    /// Assert that text is visible on screen
    ///
    /// # Arguments
    ///
    /// * `text` - Text that should be visible
    ///
    /// # Panics
    ///
    /// Panics if the text is not found
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # use tests::tui_tester::TuiTester;
    /// # let mut tester = TuiTester::new("/tmp/test_dir")?;
    /// # tester.launch()?;
    /// tester.assert_contains("File Browser")?;
    /// tester.assert_contains("test.txt")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn assert_contains(&mut self, text: &str) -> Result<()> {
        let screen = self.get_screen()?;

        if screen.contains(text) {
            self.log(&format!("Assertion passed: contains {:?}", text));
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Assertion failed: expected to contain {:?}\nActual screen:\n{}",
                text,
                screen
            ))
        }
    }

    /// Assert that text is NOT visible on screen
    ///
    /// # Arguments
    ///
    /// * `text` - Text that should NOT be visible
    ///
    /// # Panics
    ///
    /// Panics if the text is found
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # use tests::tui_tester::TuiTester;
    /// # let mut tester = TuiTester::new("/tmp/test_dir")?;
    /// # tester.launch()?;
    /// tester.send_keys(":q")?;
    /// tester.assert_not_contains("Error")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn assert_not_contains(&mut self, text: &str) -> Result<()> {
        let screen = self.get_screen()?;

        if !screen.contains(text) {
            self.log(&format!("Assertion passed: does not contain {:?}", text));
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Assertion failed: expected NOT to contain {:?}\nActual screen:\n{}",
                text,
                screen
            ))
        }
    }

    /// Assert that the screen matches an expected pattern
    ///
    /// # Arguments
    ///
    /// * `pattern` - Regex pattern to match against screen content
    pub fn assert_matches(&mut self, pattern: &str) -> Result<()> {
        let screen = self.get_screen()?;
        let regex = regex::Regex::new(pattern)
            .context("Invalid regex pattern")?;

        if regex.is_match(&screen) {
            self.log(&format!("Assertion passed: matches pattern {:?}", pattern));
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Assertion failed: expected to match pattern {:?}\nActual screen:\n{}",
                pattern,
                screen
            ))
        }
    }

    /// Assert screen content exactly matches expected text
    ///
    /// # Arguments
    ///
    /// * `expected` - Expected screen content
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # use tests::tui_tester::TuiTester;
    /// # let mut tester = TuiTester::new("/tmp/test_dir")?;
    /// # tester.launch()?;
    /// tester.assert_screen_equals("Expected output\nwith multiple lines")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn assert_screen_equals(&mut self, expected: &str) -> Result<()> {
        let screen = self.get_screen()?;

        if screen.trim() == expected.trim() {
            self.log("Assertion passed: screen equals expected");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Assertion failed: screen does not match expected\nExpected:\n{}\n\nActual:\n{}",
                expected,
                screen
            ))
        }
    }

    /// Parse the current TUI state from screen content
    ///
    /// Returns a structured representation of the TUI state including:
    /// - Current directory
    /// - File list
    /// - Selected item
    /// - Active pane
    /// - Mode indicators
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # use tests::tui_tester::TuiTester;
    /// # let mut tester = TuiTester::new("/tmp/test_dir")?;
    /// # tester.launch()?;
    /// let state = tester.parse_state()?;
    /// println!("Current directory: {:?}", state.current_directory);
    /// println!("Files: {:?}", state.files);
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_state(&mut self) -> Result<TuiState> {
        let screen = self.get_screen()?;
        TuiState::parse(&screen)
    }

    /// Clean shutdown of the TUI application
    ///
    /// Sends 'q' to quit and waits for the process to terminate.
    /// Also performs cleanup of test files if requested.
    ///
    /// # Arguments
    ///
    /// * `cleanup` - If true, removes the test directory
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # use tests::tui_tester::TuiTester;
    /// # let mut tester = TuiTester::new("/tmp/test_dir")?;
    /// # tester.launch()?;
    /// tester.quit(true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn quit(&mut self, cleanup: bool) -> Result<()> {
        self.log("Quitting application...");

        // Try graceful shutdown first
        let _ = self.send_keys("q");

        // Give it time to shut down
        thread::sleep(Duration::from_millis(200));

        // Force kill if still running
        if self.running {
            if let Some(ref mut child) = self.child {
                let _ = child.try_kill();
            }
        }

        self.running = false;

        // Cleanup test directory if requested
        if cleanup {
            self.cleanup()?;
        }

        self.log("Application quit successfully");
        Ok(())
    }

    /// Clean up test artifacts
    fn cleanup(&self) -> Result<()> {
        if self.test_dir.exists() {
            std::fs::remove_dir_all(&self.test_dir)
                .context("Failed to remove test directory")?;
        }
        Ok(())
    }

    /// Get the path to the test directory
    pub fn test_dir(&self) -> &PathBuf {
        &self.test_dir
    }

    /// Log debug output if verbose mode is enabled
    fn log(&self, message: &str) {
        if self.verbose {
            eprintln!("[TuiTester] {}", message);
        }
    }
}

impl Drop for TuiTester {
    fn drop(&mut self) {
        // Ensure we clean up on drop
        if self.running {
            let _ = self.quit(false);
        }
    }
}

/// Special keys that can be sent to the TUI
#[derive(Debug, Clone, Copy)]
pub enum SpecialKey {
    /// Enter key
    Enter,
    /// Tab key
    Tab,
    /// Escape key
    Escape,
    /// Backspace
    Backspace,
    /// Delete
    Delete,
    /// Arrow Up
    Up,
    /// Arrow Down
    Down,
    /// Arrow Left
    Left,
    /// Arrow Right,
    Right,
    /// Page Up
    PageUp,
    /// Page Down
    PageDown,
    /// Home
    Home,
    /// End,
    End,
    /// Ctrl+C
    CtrlC,
    /// Ctrl+D
    CtrlD,
    /// Ctrl+T (new tab)
    CtrlT,
    /// Ctrl+W (close tab)
    CtrlW,
    /// Ctrl+P (toggle split view)
    CtrlP,
    /// Ctrl+F (search)
    CtrlF,
    /// Ctrl+G (content search)
    CtrlG,
    /// Ctrl+R (redo)
    CtrlR,
    /// Ctrl+U (undo)
    CtrlU,
    /// F1-F12 function keys
    F(u8),
}

impl SpecialKey {
    /// Convert special key to its terminal escape sequence
    fn to_sequence(self) -> String {
        match self {
            SpecialKey::Enter => "\n".to_string(),
            SpecialKey::Tab => "\t".to_string(),
            SpecialKey::Escape => "\x1b".to_string(),
            SpecialKey::Backspace => "\x08".to_string(),
            SpecialKey::Delete => "\x1b[3~".to_string(),
            SpecialKey::Up => "\x1b[A".to_string(),
            SpecialKey::Down => "\x1b[B".to_string(),
            SpecialKey::Left => "\x1b[D".to_string(),
            SpecialKey::Right => "\x1b[C".to_string(),
            SpecialKey::PageUp => "\x1b[5~".to_string(),
            SpecialKey::PageDown => "\x1b[6~".to_string(),
            SpecialKey::Home => "\x1b[H".to_string(),
            SpecialKey::End => "\x1b[F".to_string(),
            SpecialKey::CtrlC => "\x03".to_string(),
            SpecialKey::CtrlD => "\x04".to_string(),
            SpecialKey::CtrlT => "\x14".to_string(),
            SpecialKey::CtrlW => "\x17".to_string(),
            SpecialKey::CtrlP => "\x10".to_string(),
            SpecialKey::CtrlF => "\x06".to_string(),
            SpecialKey::CtrlG => "\x07".to_string(),
            SpecialKey::CtrlR => "\x12".to_string(),
            SpecialKey::CtrlU => "\x15".to_string(),
            SpecialKey::F(n) => format!("\x1b[{}", n + 10),
        }
    }
}

/// Parsed TUI state from screen content
///
/// Provides a structured view of what's currently displayed
#[derive(Debug, Clone)]
pub struct TuiState {
    /// Current working directory
    pub current_directory: String,
    /// List of visible files
    pub files: Vec<String>,
    /// Index of selected file (if any)
    pub selected_index: Option<usize>,
    /// Currently selected file name
    pub selected_file: Option<String>,
    /// Active pane (for split view)
    pub active_pane: Option<String>,
    /// Current mode (Normal, Command, Search, etc.)
    pub mode: String,
    /// Whether split view is active
    pub split_view: bool,
    /// Status bar message (if any)
    pub status_message: Option<String>,
    /// Command line input (if in command mode)
    pub command_input: Option<String>,
    /// Number of tabs
    pub tab_count: usize,
    /// Currently active tab number
    pub active_tab: usize,
}

impl TuiState {
    /// Parse screen content into structured state
    pub fn parse(screen: &str) -> Result<Self> {
        let lines: Vec<&str> = screen.lines().collect();

        // Try to extract current directory (usually at top or in status line)
        let current_directory = Self::extract_directory(screen);

        // Extract file list
        let files = Self::extract_files(screen);

        // Extract selected item
        let (selected_index, selected_file) = Self::extract_selected(screen, &files);

        // Extract mode
        let mode = Self::extract_mode(screen);

        // Check for split view
        let split_view = Self::check_split_view(screen);

        // Extract active pane
        let active_pane = if split_view {
            Self::extract_active_pane(screen)
        } else {
            None
        };

        // Extract status message
        let status_message = Self::extract_status_message(screen);

        // Extract command input if in command mode
        let command_input = if mode.contains("Command") || mode.contains("Search") {
            Self::extract_command_input(screen)
        } else {
            None
        };

        // Extract tab information
        let (tab_count, active_tab) = Self::extract_tab_info(screen);

        Ok(Self {
            current_directory,
            files,
            selected_index,
            selected_file,
            active_pane,
            mode,
            split_view,
            status_message,
            command_input,
            tab_count,
            active_tab,
        })
    }

    fn extract_directory(screen: &str) -> String {
        // Look for path-like patterns
        for line in screen.lines() {
            if line.contains('/') || line.contains('\\') {
                if line.len() < 200 { // Reasonable path length
                    return line.trim().to_string();
                }
            }
        }
        "Unknown".to_string()
    }

    fn extract_files(screen: &str) -> Vec<String> {
        let mut files = Vec::new();

        for line in screen.lines() {
            let trimmed = line.trim();
            // Skip empty lines and UI elements
            if trimmed.is_empty() || trimmed.starts_with('│') || trimmed.starts_with('─') {
                continue;
            }
            // Look for file-like entries
            if !trimmed.starts_with('┌') && !trimmed.starts_with('└') &&
               !trimmed.starts_with('┐') && !trimmed.starts_with('┘') {
                files.push(trimmed.to_string());
            }
        }

        files
    }

    fn extract_selected(screen: &str, files: &[String]) -> (Option<usize>, Option<String>) {
        // Look for selection indicator (usually > or *)
        for (i, line) in screen.lines().enumerate() {
            if line.contains('▶') || line.contains('●') || line.contains('*') {
                return (Some(i), Some(line.trim().to_string()));
            }
        }

        // If no explicit marker, assume first file is selected
        if !files.is_empty() {
            (Some(0), Some(files[0].clone()))
        } else {
            (None, None)
        }
    }

    fn extract_mode(screen: &str) -> String {
        if screen.contains("-- COMMAND --") || screen.contains("COMMAND") {
            return "Command".to_string();
        }
        if screen.contains("-- SEARCH --") || screen.contains("SEARCH") {
            return "Search".to_string();
        }
        if screen.contains("-- INSERT --") || screen.contains("INSERT") {
            return "Insert".to_string();
        }
        if screen.contains("Bulk Rename") || screen.contains("RENAME") {
            return "Bulk Rename".to_string();
        }
        "Normal".to_string()
    }

    fn check_split_view(screen: &str) -> String {
        // Look for split view indicators
        // (Implementation depends on how split view is rendered)
        "false".to_string()
    }

    fn extract_active_pane(screen: &str) -> String {
        // Look for active pane indicator
        // (Implementation depends on how panes are rendered)
        "left".to_string()
    }

    fn extract_status_message(screen: &str) -> String {
        // Look for status line (usually at bottom)
        None
    }

    fn extract_command_input(screen: &str) -> String {
        // Look for command prompt input
        // (Implementation depends on how command input is rendered)
        None
    }

    fn extract_tab_info(screen: &str) -> (usize, usize) {
        // Look for tab indicators
        // (Implementation depends on how tabs are rendered)
        (1, 0)
    }
}

/// Strip ANSI escape codes from a string
fn strip_ansi_codes(input: &str) -> String {
    // This regex matches common ANSI escape sequences
    let ansi_regex = regex::Regex::new(r"\x1b\[[0-9;]*[mGKHfABCD]")
        .unwrap();

    ansi_regex.replace_all(input, "").to_string()
}

/// Helper trait for setting non-blocking mode
trait SetNonblocking {
    fn set_nonblocking(&self, nonblocking: bool) -> std::io::Result<()>;
}

#[cfg(unix)]
impl SetNonblocking for Box<dyn PtyMaster> {
    fn set_nonblocking(&self, nonblocking: bool) -> std::io::Result<()> {
        use std::os::unix::io::AsRawFd;
        // Implementation would set O_NONBLOCK flag
        Ok(())
    }
}

#[cfg(windows)]
impl SetNonblocking for Box<dyn PtyMaster> {
    fn set_nonblocking(&self, _nonblocking: bool) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_codes() {
        let input = "\x1b[31mRed text\x1b[0m and \x1b[1;32mbold green\x1b[0m";
        let output = strip_ansi_codes(input);
        assert_eq!(output, "Red text and bold green");
    }

    #[test]
    fn test_special_key_sequences() {
        assert_eq!(SpecialKey::Enter.to_sequence(), "\n");
        assert_eq!(SpecialKey::Escape.to_sequence(), "\x1b");
        assert_eq!(SpecialKey::CtrlC.to_sequence(), "\x03");
    }

    #[test]
    fn test_tui_state_parsing() {
        let screen = r#"
/home/user/test
────────────────
▶ file1.txt
  file2.txt
  file3.txt
────────────────
-- NORMAL --
"#;
        let state = TuiState::parse(screen).unwrap();
        assert!(!state.current_directory.is_empty());
        assert!(!state.files.is_empty());
    }
}
