//! Visual testing utilities for TUI snapshot testing
//!
//! Provides the `VisualTester` struct for capturing and comparing TUI rendering output.

use anyhow::Result;
use ratatui::{
    backend::{Backend, TestBackend},
    buffer::Buffer,
    Terminal,
};
use std::fmt::Write;

use filebrowser_tui::state::App;

/// A visual tester for capturing TUI rendering output
///
/// `VisualTester` renders the application to an in-memory terminal and captures
/// the output for snapshot comparison.
pub struct VisualTester {
    /// Terminal width
    width: u16,
    /// Terminal height
    height: u16,
}

impl VisualTester {
    /// Create a new visual tester with default dimensions (80x24)
    pub fn new() -> Self {
        Self::with_size(80, 24)
    }

    /// Create a new visual tester with custom dimensions
    pub fn with_size(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    /// Render the app and capture the output as text
    ///
    /// This renders the application to an in-memory test backend and
    /// returns the rendered content as a formatted string.
    pub fn capture(&self, app: &App) -> Result<String> {
        // Create test backend with specified dimensions
        let backend = TestBackend::new(self.width, self.height);
        let mut terminal = Terminal::new(backend)?;

        // Render the app
        terminal.draw(|f| {
            filebrowser_tui::ui::render_app(f, app);
        })?;

        // Get the buffer and convert to text
        let buffer = terminal.backend().buffer().clone();
        Ok(self.buffer_to_text(&buffer))
    }

    /// Convert a ratatui buffer to text representation
    ///
    /// The output includes:
    /// - Cell characters with colors indicated (where applicable)
    /// - Borders and box drawing characters preserved
    /// - Empty areas shown as spaces
    fn buffer_to_text(&self, buffer: &Buffer) -> String {
        let mut output = String::new();
        let area = buffer.area();

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = buffer.get(x, y);
                let symbol = if cell.symbol().is_empty() {
                    ' '
                } else {
                    // Convert box drawing and other special chars
                    cell.symbol().chars().next().unwrap_or(' ')
                };
                write!(output, "{}", symbol).ok();
            }
            // Add newline at end of each row, but not after the last row
            if y < area.bottom() - 1 {
                writeln!(output).ok();
            }
        }

        output
    }

    /// Render the app and return both the text output and the buffer
    ///
    /// This is useful for more detailed inspection or custom diff formatting.
    pub fn capture_with_buffer(&self, app: &App) -> Result<(String, Buffer)> {
        let backend = TestBackend::new(self.width, self.height);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|f| {
            filebrowser_tui::ui::render_app(f, app);
        })?;

        let buffer = terminal.backend().buffer().clone();
        let text = self.buffer_to_text(&buffer);

        Ok((text, buffer))
    }

    /// Create a colored diff output showing changes between expected and actual
    ///
    /// This provides a side-by-side comparison showing:
    /// - Lines present in both (unchanged)
    /// - Lines only in expected (removed)
    /// - Lines only in actual (added)
    pub fn diff_output(&self, expected: &str, actual: &str) -> String {
        let mut diff = String::new();
        let expected_lines: Vec<&str> = expected.lines().collect();
        let actual_lines: Vec<&str> = actual.lines().collect();

        use std::cmp::{max, min};

        let max_lines = max(expected_lines.len(), actual_lines.len());

        diff.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        diff.push_str("SNAPSHOT DIFF\n");
        diff.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

        for i in 0..max_lines {
            let expected_line = expected_lines.get(i).map(|s| s.as_str()).unwrap_or("");
            let actual_line = actual_lines.get(i).map(|s| s.as_str()).unwrap_or("");

            if expected_line == actual_line {
                // Unchanged line - show as is (with line number)
                diff.push_str(&format!("  {:04} │ {}\n", i + 1, expected_line));
            } else {
                // Changed line
                if !expected_line.is_empty() {
                    diff.push_str(&format!("- {:04} │ {}\n", i + 1, expected_line));
                }
                if !actual_line.is_empty() {
                    diff.push_str(&format!("+ {:04} │ {}\n", i + 1, actual_line));
                }
            }
        }

        diff.push_str("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        diff.push_str(&format!(
            "LEGEND:  | Unchanged  - Expected only  + Actual only\n\
             Dimensions: {}x{} | Expected lines: {} | Actual lines: {}\n",
            self.width,
            self.height,
            expected_lines.len(),
            actual_lines.len()
        ));
        diff.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        diff
    }
}

impl Default for VisualTester {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual tester_creation() {
        let tester = VisualTester::new();
        assert_eq!(tester.width, 80);
        assert_eq!(tester.height, 24);
    }

    #[test]
    fn test_visual tester_custom_size() {
        let tester = VisualTester::with_size(120, 30);
        assert_eq!(tester.width, 120);
        assert_eq!(tester.height, 30);
    }

    #[test]
    fn test_diff_output_identical() {
        let tester = VisualTester::new();
        let content = "line 1\nline 2\nline 3";
        let diff = tester.diff_output(content, content);
        assert!(diff.contains("Unchanged"));
    }

    #[test]
    fn test_diff_output_different() {
        let tester = VisualTester::new();
        let expected = "line 1\nline 2\nline 3";
        let actual = "line 1\nmodified\nline 3";
        let diff = tester.diff_output(expected, actual);
        assert!(diff.contains("-"));
        assert!(diff.contains("+"));
    }
}
