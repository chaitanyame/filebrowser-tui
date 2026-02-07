# Testing Framework Index

Complete index of all testing resources for the file browser TUI project.

## Quick Links

- **[E2E Testing Quick Reference](E2E_TESTING_QUICK_REF.md)** - Fast API reference
- **[E2E Framework Summary](E2E_FRAMEWORK_SUMMARY.md)** - Complete implementation guide
- **[Testing Guide](README.md)** - Comprehensive testing documentation

## E2E Testing Framework

### Core Files

| File | Size | Description |
|------|------|-------------|
| `tui_tester.rs` | 33KB | PTY-based testing framework |
| `e2e_tests.rs` | 26KB | Comprehensive E2E test suite (29 tests) |
| `fixtures/mod.rs` | 20KB | Test fixture management |
| `common/mod.rs` | 16KB | Shared test utilities |

### Documentation

| File | Size | Description |
|------|------|-------------|
| `E2E_TESTING_QUICK_REF.md` | 6.4KB | Quick reference guide |
| `E2E_FRAMEWORK_SUMMARY.md` | 16KB | Implementation summary |

### Tooling

| File | Description |
|------|-------------|
| `scripts/run-e2e-tests.sh` | E2E test runner script |

## Other Testing Frameworks

### Snapshot Testing

| File | Description |
|------|-------------|
| `snapshot_tests.rs` | Visual regression tests |
| `visual_tester.rs` | Screen capture utility |
| `snapshots/` | Stored snapshot files |

**Documentation**: [Snapshot Testing Guide](README.md#snapshot-testing-guide)

### Property-Based Testing

| File | Description |
|------|-------------|
| `property_tests.rs` | Property-based test suite |

**Documentation**:
- [PROPERTY_TESTS.md](PROPERTY_TESTS.md)
- [PROPERTY_TESTS_SUMMARY.md](PROPERTY_TESTS_SUMMARY.md)
- [ADDING_PROPERTY_TESTS.md](ADDING_PROPERTY_TESTS.md)

### Unit Tests

| Location | Description |
|----------|-------------|
| `src/**/*.rs` | Inline unit tests |

## Test Categories

### 1. E2E Tests (29 tests)
- Navigation (3)
- File Operations (6)
- Search/Filter (3)
- Tab Management (3)
- Split View (3)
- Undo/Redo (2)
- Bulk Rename (1)
- Error Handling (2)
- Performance (2)
- Edge Cases (3)
- Integration (1)

### 2. Snapshot Tests (16 tests)
- Basic views (3)
- Mode-specific (3)
- Layout variations (3)
- Dialogs (3)
- Edge cases (4)

### 3. Property Tests (12 tests)
- File sorting (2)
- Path manipulation (2)
- Filter correctness (2)
- Navigation history (2)
- Selection behavior (2)
- Tab management (2)

## Running Tests

### E2E Tests

```bash
# All E2E tests
cargo test --test e2e_tests

# Specific test
cargo test --test e2e_tests test_navigate_directories

# Verbose mode
TUI_TEST_VERBOSE=1 cargo test --test e2e_tests

# Using script
./scripts/run-e2e-tests.sh -v
```

### Snapshot Tests

```bash
# Run snapshot tests
cargo test --test snapshot_tests

# Update snapshots
cargo insta test --review

# Accept all changes
cargo insta test --accept
```

### Property Tests

```bash
# Run property tests
cargo test --test property_tests

# With more cases
cargo test --test property_tests -- --nocapture
```

### All Tests

```bash
# Run everything
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --tests
```

## Key APIs

### TuiTester

```rust
// Creation
let tester = TuiTester::new(path)?
    .with_terminal_size(rows, cols)
    .with_verbose(true);

// Lifecycle
tester.launch()?;
tester.quit(true)?;

// Input
tester.send_keys("text")?;
tester.send_special_key(SpecialKey::Enter)?;

// Output
tester.wait_for("text")?;
let screen = tester.get_screen()?;

// Assertions
tester.assert_contains("text")?;
tester.assert_not_contains("error")?;
```

### Fixtures

```rust
let fixture = TestFixture::new()?;

// Pre-defined structures
fixture.create_standard_structure()?;
fixture.create_operations_structure()?;

// Custom operations
fixture.create_dir("dir")?;
fixture.create_file("dir/file.txt", "content")?;
fixture.exists("dir/file.txt")?; // true
```

### Common Utils

```rust
// Retry
retry(|| operation(), RetryConfig::default())?;

// Wait
wait_for(|| condition(), timeout, interval)?;

// Assert
assert_eventually(|| condition(), timeout, "message")?;
assert_file_exists("path")?;
```

## Special Keys Reference

| Key | Enum | Usage |
|-----|------|-------|
| Enter | `SpecialKey::Enter` | Confirm |
| Escape | `SpecialKey::Escape` | Cancel/Exit |
| Tab | `SpecialKey::Tab` | Switch tab/pane |
| Ctrl+T | `SpecialKey::CtrlT` | New tab |
| Ctrl+W | `SpecialKey::CtrlW` | Close tab |
| Ctrl+P | `SpecialKey::CtrlP` | Split view |
| Ctrl+F | `SpecialKey::CtrlF` | Search |
| Ctrl+G | `SpecialKey::CtrlG` | Content search |
| Ctrl+R | `SpecialKey::CtrlR` | Redo |
| Ctrl+U | `SpecialKey::CtrlU` | Undo |
| F5 | `SpecialKey::F(5)` | Copy |
| F6 | `SpecialKey::F(6)` | Move |

## CI/CD Integration

### GitHub Actions Example

```yaml
- name: Run E2E tests
  run: |
    cargo build --release
    cargo test --test e2e_tests -- --ignored

- name: Run snapshot tests
  run: |
    cargo install cargo-insta
    cargo insta test --accept --unreferenced=auto

- name: Run property tests
  run: cargo test --test property_tests
```

## Debugging Tips

### E2E Tests

1. Enable verbose mode: `TUI_TEST_VERBOSE=1`
2. Don't cleanup: `tester.quit(false)?`
3. Retain fixtures: `fixture.retain_on_drop()`
4. Check screen: `println!("{:?}", tester.get_screen()?)`
5. Parse state: `let state = tester.parse_state()?;`

### Snapshot Tests

1. Review interactively: `cargo insta test --review`
2. Check diffs: `cargo insta review`
3. Update specific: `cargo insta test -- snapshot_name`

### Property Tests

1. Use smaller cases: `proptest::config::Config::with_cases(10)`
2. Seed for reproducibility: `prop_compose![(... in prop::strategy::Strategy::boxed(...))]`
3. Save failing cases: `PROCTEST_SAVE_FAILS=1`

## Best Practices

### E2E Tests

1. Always use `#[ignore]` by default
2. Use fixtures for test data
3. Clean up with `quit(true)`
4. Use `wait_for()` instead of `sleep()`
5. Test workflows, not implementation

### Snapshot Tests

1. Run before committing
2. Review changes carefully
3. Keep snapshots in version control
4. Use descriptive test names

### Property Tests

1. Test invariants, not specific values
2. Keep properties simple
3. Use good generators
4. Document properties

## Contributing

When adding tests:

1. **Choose the right type**: E2E for workflows, snapshot for UI, property for invariants
2. **Use fixtures**: Never use real directories
3. **Clean up**: Ensure tests remove artifacts
4. **Document**: Explain what and why
5. **Review**: Check for flakiness

## Resources

- [Insta Documentation](https://insta.rs/)
- [Proptest Documentation](https://altsysrq.github.io/proptest-book/intro.html)
- [Ratatui Testing Guide](https://ratatui.rs/how-to/testing/)

## Statistics

- **Total Test Files**: 10
- **E2E Tests**: 29
- **Snapshot Tests**: 16
- **Property Tests**: 12
- **Total Lines of Test Code**: ~2,500
- **Documentation Pages**: 7

## Support

For issues or questions:
1. Check the relevant documentation above
2. Review example tests in `e2e_tests.rs`
3. Enable verbose mode for debugging
4. Check CI logs for patterns

---

**Last Updated**: 2026-02-06
**Framework Version**: 1.0.0
**Status**: Production Ready
