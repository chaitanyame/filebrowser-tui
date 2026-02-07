# E2E Testing Quick Reference

Quick reference for the PTY-based E2E testing framework.

## TuiTester API

### Creation

```rust
let tester = TuiTester::new("/path/to/test/dir")?
    .with_terminal_size(40, 120)
    .with_verbose(true);
```

### Lifecycle

```rust
tester.launch()?;           // Start the app
tester.quit(true)?;         // Quit and cleanup
```

### Input Simulation

```rust
// Send text
tester.send_keys("hello")?;

// Send special keys
tester.send_special_key(SpecialKey::Enter)?;
tester.send_special_key(SpecialKey::CtrlT)?;
tester.send_special_key(SpecialKey::F(5))?;
```

### Output Capture

```rust
// Wait for text
tester.wait_for("Search")?;

// Wait with custom timeout
tester.wait_for_timeout("Loading", Duration::from_secs(10))?;

// Wait for text to disappear
tester.wait_for_remove("Loading...")?;

// Get screen content
let screen = tester.get_screen()?;
```

### Assertions

```rust
// Text presence
tester.assert_contains("file.txt")?;
tester.assert_not_contains("Error")?;

// Regex match
tester.assert_matches(r"\d+ files")?;

// Exact screen match
tester.assert_screen_equals("Expected output")?;

// Parse TUI state
let state = tester.parse_state()?;
println!("Current dir: {}", state.current_directory);
```

## Special Keys

| Key | Code | Usage |
|-----|------|-------|
| Enter | `SpecialKey::Enter` | Confirm selections |
| Tab | `SpecialKey::Tab` | Switch tabs/panes |
| Escape | `SpecialKey::Escape` | Exit modes |
| Ctrl+T | `SpecialKey::CtrlT` | New tab |
| Ctrl+W | `SpecialKey::CtrlW` | Close tab |
| Ctrl+P | `SpecialKey::CtrlP` | Toggle split view |
| Ctrl+F | `SpecialKey::CtrlF` | Search |
| Ctrl+G | `SpecialKey::CtrlG` | Content search |
| Ctrl+R | `SpecialKey::CtrlR` | Redo |
| Ctrl+U | `SpecialKey::CtrlU` | Undo |
| F5 | `SpecialKey::F(5)` | Copy |
| F6 | `SpecialKey::F(6)` | Move |
| Down | `SpecialKey::Down` | Navigate down |
| Up | `SpecialKey::Up` | Navigate up |

## Fixtures API

```rust
let fixture = TestFixture::new()?;

// Pre-defined structures
fixture.create_standard_structure()?;
fixture.create_nested_structure()?;
fixture.create_large_structure()?;
fixture.create_search_structure()?;
fixture.create_operations_structure()?;
fixture.create_tab_structure()?;
fixture.create_split_structure()?;
fixture.create_undo_structure()?;

// Custom structures
fixture.create_dir("my_dir")?;
fixture.create_file("my_dir/file.txt", "content")?;

// File operations
fixture.exists("path")?;
fixture.read_file("path")?;
fixture.delete("path")?;
fixture.rename("old", "new")?;
fixture.copy("src", "dest")?;

// Cleanup
fixture.cleanup()?;
```

## Common Utilities

```rust
// Retry with backoff
retry(|| operation(), RetryConfig::default())?;

// Wait for condition
wait_for(|| condition(), timeout, interval)?;

// Assert eventually
assert_eventually(|| condition(), timeout, "message");

// File assertions
assert_file_exists("path")?;
assert_file_contains("path", "text")?;
assert_file_content("path", "expected")?;

// CI-aware timeouts
let timeout = adjust_timeout(Duration::from_secs(5));
```

## Test Template

```rust
#[test]
#[ignore] // Ignore by default
fn test_feature_name() {
    // Setup
    let (mut tester, fixture) = create_tester().unwrap();
    fixture.create_structure().unwrap();

    // Launch
    tester.launch().unwrap();

    // Act
    tester.wait_for("expected").unwrap();
    tester.send_keys("input").unwrap();

    // Assert
    tester.assert_contains("result").unwrap();
    crate::common::assert_eventually(
        || fixture.exists("created.txt"),
        Duration::from_secs(2),
        "Should create file"
    );

    // Cleanup
    tester.quit(true).unwrap();
}
```

## Running Tests

```bash
# Run all E2E tests
cargo test --test e2e_tests

# Run specific test
cargo test --test e2e_tests test_navigate_directories

# Verbose mode
TUI_TEST_VERBOSE=1 cargo test --test e2e_tests

# Run ignored tests
cargo test --test e2e_tests -- --ignored

# Use the script
./scripts/run-e2e-tests.sh -v -i
```

## Debugging Tips

1. **Enable verbose mode**: `TUI_TEST_VERBOSE=1`
2. **Don't cleanup**: `tester.quit(false)?`
3. **Retain fixtures**: `fixture.retain_on_drop()`
4. **Check screen content**: `println!("{:?}", tester.get_screen()?)`
5. **Parse state**: `let state = tester.parse_state()?; println!("{:?}", state)`
6. **Single thread**: `--test-threads=1`
7. **Run specific test**: `--test <name>`

## Common Patterns

### Navigation

```rust
// Enter directory
tester.send_keys("dir_name")?;
tester.send_special_key(SpecialKey::Enter)?;
tester.wait_for("file_in_dir")?;

// Go back up
tester.send_special_key(SpecialKey::Escape)?;
```

### File Operations

```rust
// Select and delete
tester.wait_for("file.txt")?;
tester.send_keys("d")?;
tester.send_special_key(SpecialKey::Enter)?;

// Select and rename
tester.wait_for("old.txt")?;
tester.send_keys("r")?;
tester.send_keys("new.txt")?;
tester.send_special_key(SpecialKey::Enter)?;
```

### Mode Changes

```rust
// Enter command mode
tester.send_special_key(SpecialKey::Escape)?;
tester.send_keys(":command")?;
tester.send_special_key(SpecialKey::Enter)?;

// Enter search mode
tester.send_special_key(SpecialKey::Escape)?;
tester.send_keys("/")?;
tester.send_keys("search")?;
tester.send_special_key(SpecialKey::Enter)?;
```

### Async Verification

```rust
// Wait for file system change
crate::common::assert_eventually(
    || fixture.exists("new_file.txt"),
    Duration::from_secs(2),
    "File should be created"
);

// Retry operation
retry(
    || fixture.read_file("file.txt"),
    RetryConfig::quick()
)?;
```

## Error Handling

```rust
// Expect errors
tester.send_keys("invalid_command")?;
tester.send_special_key(SpecialKey::Enter)?;
tester.wait_for("Error")?;
tester.assert_contains("not found")?;

// Handle timeout
match tester.wait_for_timeout("text", Duration::from_secs(10)) {
    Ok(_) => println!("Found!"),
    Err(e) => println!("Timeout or error: {}", e),
}
```

## Performance Testing

```rust
let (result, duration) = measure_time(|| {
    // Perform operation
    tester.send_keys("command")?;
    tester.wait_for("result")
});

assert!(duration < Duration::from_secs(1), "Too slow!");
```

## Best Practices

1. **Always use `#[ignore]`** for E2E tests by default
2. **Use fixtures** for test data
3. **Clean up** with `quit(true)`
4. **Use `wait_for()`** instead of `sleep()`
5. **Use `assert_eventually()`** for async operations
6. **Test workflows**, not implementation
7. **Keep tests independent**
8. **Use descriptive names**
