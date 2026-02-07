# Testing Guide for File Browser TUI

This directory contains comprehensive testing frameworks for the file browser TUI application, including snapshot tests, E2E tests, property tests, and more.

## Testing Frameworks

### 1. E2E Testing Framework (PTY-based)

**Location**: `tests/e2e_tests.rs`, `tests/tui_tester.rs`

The E2E testing framework uses pseudo-terminals (PTY) to simulate real user interactions with the TUI application through keyboard input.

#### Quick Start

```bash
# Run all E2E tests (requires compiled binary)
cargo test --test e2e_tests

# Run with verbose output for debugging
TUI_TEST_VERBOSE=1 cargo test --test e2e_tests

# Run ignored tests manually
cargo test --test e2e_tests -- --ignored
```

#### Key Components

- **TuiTester** (`tests/tui_tester.rs`): Core PTY testing framework
  - Spawns TUI app in pseudo-terminal
  - Simulates keyboard input
  - Captures and parses screen output
  - Provides assertion methods

- **Fixtures** (`tests/fixtures/mod.rs`): Test data management
  - Pre-defined directory structures
  - File/directory creation helpers
  - Automatic cleanup

- **Common Utils** (`tests/common/mod.rs`): Shared utilities
  - Retry logic with backoff
  - Async condition waiting
  - File/directory assertions
  - CI-aware timeout adjustment

#### Writing E2E Tests

```rust
#[test]
#[ignore] // Ignore by default, requires compiled binary
fn test_my_feature() {
    let (mut tester, fixture) = create_tester().unwrap();

    // Setup test data
    fixture.create_file("test.txt", "Content").unwrap();

    // Launch application
    tester.launch().unwrap();

    // Interact with app
    tester.wait_for("test.txt").unwrap();
    tester.send_keys("dd").unwrap(); // Delete

    // Verify result
    crate::common::assert_eventually(
        || !fixture.exists("test.txt"),
        Duration::from_secs(2),
        "File should be deleted"
    );

    // Cleanup
    tester.quit(true).unwrap();
}
```

#### Test Categories

1. **Navigation**: Directory navigation, scrolling, deep hierarchies
2. **File Operations**: Create, delete, rename, copy, move
3. **Search/Filter**: Name search, content search, filtering
4. **Tab Management**: Create, switch, close tabs
5. **Split View**: Toggle, pane switching, cross-pane operations
6. **Undo/Redo**: Operation history
7. **Bulk Rename**: Pattern-based renaming
8. **Error Handling**: Permission errors, invalid paths
9. **Performance**: Large directories, responsiveness
10. **Edge Cases**: Empty dirs, special chars, Unicode

### 2. Snapshot Testing

**Location**: `tests/snapshot_tests.rs`, `tests/visual_tester.rs`

Snapshot tests capture the rendered terminal output and compare it against stored snapshots to detect visual regressions.

#### Quick Start

```bash
# Run all snapshot tests
cargo test --test snapshot_tests

# Update snapshots interactively
cargo insta test --review

# Accept all snapshot changes
cargo insta test --accept
```

#### Coverage Areas

- Empty directory view
- Directory with files
- Selected file states
- Search/command modes
- Split view
- Tab bar with multiple tabs
- Preview pane
- Bulk rename preview
- Confirmation dialogs
- Message types (info, warning, error, success)
- Hidden files visibility
- Different terminal sizes
- Long filenames
- Scrolling behavior

See [Snapshot Testing Guide](#snapshot-testing-guide) below for detailed information.

### 3. Property-Based Testing

**Location**: `tests/property_tests.rs`

Property tests use randomized inputs to verify invariants hold true across many cases.

#### Quick Start

```bash
# Run property tests
cargo test --test property_tests

# Run with more test cases
CARGO_TEST_VALGRIND=1 cargo test --test property_tests
```

#### Coverage Areas

- File sorting invariants
- Path manipulation
- Filter correctness
- Navigation history
- Selection behavior
- Tab management invariants

See [Property Testing Guide](#property-testing-guide) below for detailed information.

### 4. Unit Tests

**Location**: `src/` (inline with source code)

Unit tests for individual modules and functions.

#### Quick Start

```bash
# Run all unit tests
cargo test

# Run unit tests in specific module
cargo test state::tests

# Run with output
cargo test -- --nocapture
```

## Architecture

```
tests/
├── e2e_tests.rs           # E2E test cases
├── tui_tester.rs          # PTY testing framework
├── snapshot_tests.rs      # Snapshot test cases
├── visual_tester.rs       # Visual testing utility
├── property_tests.rs      # Property-based tests
├── inline_snapshots.rs    # Inline snapshot tests
├── fixtures/
│   └── mod.rs             # Test fixture management
├── common/
│   └── mod.rs             # Shared test utilities
└── snapshots/             # Stored snapshot files
```

## Common Testing Utilities

### Fixtures

```rust
use tests::fixtures::TestFixture;

let fixture = TestFixture::new()?;
fixture.create_standard_structure()?;
fixture.create_file("test.txt", "content")?;
fixture.exists("test.txt"); // true
```

### Assertions

```rust
use tests::common::*;

assert_file_exists("path/to/file")?;
assert_file_contains("path/to/file", "content")?;
assert_eventually(
    || condition_is_true(),
    Duration::from_secs(5),
    "Condition should become true"
)?;
```

### Retry Logic

```rust
use tests::common::{retry, RetryConfig};

let result = retry(
    || operation_that_might_fail(),
    RetryConfig::default()
        .with_max_attempts(5)
        .with_delay(Duration::from_millis(100))
)?;
```

## Running Tests

### All Tests

```bash
# Run everything
cargo test

# Run with documentation
cargo test -- --doc

# Run tests in parallel
cargo test -- --test-threads=4
```

### Specific Test Types

```bash
# Only E2E tests
cargo test --test e2e_tests

# Only snapshot tests
cargo test --test snapshot_tests

# Only property tests
cargo test --test property_tests

# Only unit tests (no integration tests)
cargo test --lib
```

### Debugging Tests

```bash
# Show test output
cargo test -- --nocapture

# Show test output with timestamps
cargo test -- --nocapture -- --exact

# Run one test with full output
cargo test test_name -- --nocapture -- --exact

# Enable logging
RUST_LOG=debug cargo test
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install dependencies
        run: cargo install cargo-insta

      - name: Run unit tests
        run: cargo test --lib

      - name: Run snapshot tests
        run: cargo insta test --accept --unreferenced=auto

      - name: Check for snapshot changes
        run: git diff --exit-code tests/snapshots/

      - name: Run property tests
        run: cargo test --test property_tests
```

## Best Practices

### 1. Test Organization

- **Unit tests**: Test individual functions in isolation
- **Integration tests**: Test module interactions
- **E2E tests**: Test complete user workflows
- **Snapshot tests**: Test UI rendering
- **Property tests**: Test invariants with random inputs

### 2. Test Naming

```rust
// Good: Descriptive
fn test_delete_file_removes_from_filesystem()

// Less clear
fn test_delete()
```

### 3. Test Independence

Each test should:
- Clean up after itself
- Not depend on other tests
- Work in isolation
- Use fresh fixtures

### 4. Assert Eventually

For async operations, use `assert_eventually`:

```rust
assert_eventually(
    || fixture.exists("created.txt"),
    Duration::from_secs(2),
    "File should be created"
);
```

### 5. Use Fixtures

Never use real directories:

```rust
// Good
let fixture = TestFixture::new()?;
fixture.create_file("test.txt", "content")?;

// Bad
let path = PathBuf::from("/tmp/test");
std::fs::write(path, "content")?;
```

---

## Snapshot Testing Guide

## Overview

The snapshot testing framework provides:

- **Automated visual regression detection** - Catches UI changes that break the expected appearance
- **Easy snapshot review workflow** - Interactive review of changes before accepting
- **Comprehensive test coverage** - Tests all major UI states and edge cases
- **Clear diff output** - Side-by-side comparison when snapshots don't match

## Architecture

```
tests/
├── common/
│   └── mod.rs              # Shared test utilities and fixtures
├── fixtures/               # Test data and fixtures
├── snapshots/              # Stored snapshot files (auto-generated)
├── visual_tester.rs        # VisualTester utility for capturing output
└── snapshot_tests.rs       # Comprehensive snapshot test suite
```

## Key Components

### VisualTester (`tests/visual_tester.rs`)

The `VisualTester` struct handles rendering the TUI to an in-memory terminal using ratatui's `TestBackend`:

```rust
let tester = VisualTester::new();
let output = tester.capture(&app)?;
```

Features:
- Custom terminal dimensions
- Buffer-to-text conversion
- Colored diff output
- Detailed inspection capabilities

### Snapshot Tests (`tests/snapshot_tests.rs`)

Comprehensive test scenarios covering:
1. Empty directory view
2. Directory with files
3. Selected file state
4. Search mode
5. Split view
6. Tab bar with multiple tabs
7. Preview pane
8. Bulk rename preview
9. Confirmation dialog
10. Various message types (info, warning, error, success)
11. Hidden files visibility
12. Command mode
13. Content search mode
14. Different terminal sizes
15. Long filenames
16. Scrolling with many files

## Usage

### Running Snapshot Tests

```bash
# Run all snapshot tests (will fail if snapshots don't exist)
cargo test --test snapshot_tests

# Run snapshot tests with review
cargo insta test --review

# Run snapshot tests and accept all changes
cargo insta test --accept

# Run only snapshot tests
cargo test --test snapshot_tests -- --test-threads=1
```

### Creating/Updating Snapshots

#### Interactive Review (Recommended)

```bash
# Run tests and review changes interactively
./scripts/update-snapshots.sh

# Or using cargo-insta directly
cargo insta test --review
```

This will:
1. Run all snapshot tests
2. Show a summary of changes
3. Allow you to review each change
4. Accept or reject individual snapshots

#### Accept All Changes

```bash
# Accept all snapshot changes without review
./scripts/update-snapshots.sh --accept

# Or using cargo-insta directly
cargo insta test --accept
```

#### Review Existing Pending Snapshots

```bash
# Only review existing snapshot changes without running tests
./scripts/update-snapshots.sh --review

# Or using cargo-insta directly
cargo insta review
```

## Test Scenarios

### Basic Scenarios

```rust
#[test]
fn snapshot_empty_directory() {
    // Tests rendering of an empty directory
}

#[test]
fn snapshot_directory_with_files() {
    // Tests rendering of a directory with multiple files
}

#[test]
fn snapshot_selected_file() {
    // Tests rendering with a file selected
}
```

### Mode-Specific Scenarios

```rust
#[test]
fn snapshot_search_mode() {
    // Tests rendering in search mode
}

#[test]
fn snapshot_command_mode() {
    // Tests rendering in command mode
}

#[test]
fn snapshot_content_search_mode() {
    // Tests rendering in content search mode
}
```

### Layout Variations

```rust
#[test]
fn snapshot_split_view() {
    // Tests dual-pane split view
}

#[test]
fn snapshot_preview_pane() {
    // Tests rendering with preview panel
}

#[test]
fn snapshot_multiple_tabs() {
    // Tests tab bar with multiple tabs
}
```

### Dialogs and Overlays

```rust
#[test]
fn snapshot_bulk_rename_preview() {
    // Tests bulk rename mode
}

#[test]
fn snapshot_confirmation_dialog() {
    // Tests delete confirmation dialog
}

#[test]
fn snapshot_overwrite_dialog() {
    // Tests file overwrite confirmation
}
```

## Snapshot File Format

Snapshots are stored in `tests/snapshots/` with the naming convention:
```
snapshot_tests_<test_name>.snap
```

Example snapshot file:
```snap
---
source: tests/snapshot_tests.rs
expression: output
---
┌────────────────────────────────────────────────────────────────────────────────┐
│Tab 1 [test] │ │test                                                           │
├────────────────────────────────────────────────────────────────────────────────┤
│                                                                          │     │
│  📁 Documents                                                            │     │
│  📁 Downloads                                                            │     │
│  📁 Pictures                                                             │     │
│▶ 📁 Music                                                                │     │
│  📁 Videos                                                               │     │
│  📄 file1.txt                                                            │     │
│  📄 file2.txt                                                            │     │
...
```

## Best Practices

### 1. Run Tests Before Committing

Always run snapshot tests before committing UI changes:

```bash
cargo test --test snapshot_tests
```

### 2. Review Changes Carefully

When updating snapshots after intentional UI changes:

1. Review each change visually
2. Ensure the change matches your intent
3. Check for unintended side effects
4. Document the reason for the change in the commit message

### 3. Keep Snapshots in Version Control

Commit snapshots to version control to:
- Track visual changes over time
- Enable regression detection in CI/CD
- Provide historical reference for UI evolution

### 4. Write Tests for New Features

When adding new UI features:
1. Add a snapshot test covering the new feature
2. Test various states of the feature
3. Include edge cases and error conditions
4. Update this documentation

### 5. Use Descriptive Test Names

Clear test names help identify what's being tested:

```rust
// Good
fn snapshot_split_view_with_different_paths()

// Less clear
fn snapshot_test_3()
```

## Troubleshooting

### Snapshots Don't Match After Refactoring

If you've refactored code without changing the UI:

1. Review the diff carefully
2. If the output is truly identical, accept the changes
3. If there are unexpected changes, investigate the refactoring

### Tests Fail on Different Terminals

Snapshot tests use `TestBackend` with fixed dimensions, so they should work consistently across platforms. However:

- Ensure you're using the latest version of `ratatui`
- Check that terminal width/height settings match expectations
- Be aware of platform-specific differences in path rendering

### Large Number of Pending Snapshots

If you have many pending snapshots:

```bash
# Review them in batches
cargo insta review

# Or accept all if you've made intentional UI changes
./scripts/update-snapshots.sh --accept
```

## CI/CD Integration

Add snapshot testing to your CI pipeline:

```yaml
# Example GitHub Actions
- name: Run snapshot tests
  run: |
    cargo install cargo-insta
    cargo insta test --accept --unreferenced=auto

- name: Check for snapshot changes
  run: |
    git diff --exit-code tests/snapshots/
```

## Contributing

When contributing to the UI:

1. Write snapshot tests for new features
2. Update existing snapshots if UI changes are intentional
3. Document the reason for snapshot changes in PRs
4. Ensure all snapshot tests pass before submitting

## Additional Resources

- [Insta Documentation](https://insta.rs/)
- [Ratatui Testing Guide](https://ratatui.rs/how-to/testing/)
- [Snapshot Testing Best Practices](https://jsoverson.medium.com/test-strategies-snapshot-testing-6f5df0755c1e)
