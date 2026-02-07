# Property Tests Quick Reference

Quick commands and examples for working with property-based tests in filebrowser-tui.

## Essential Commands

```bash
# Run all property tests
cargo test --test property_tests

# Run specific test
cargo test --test property_tests prop_sort_by_name_is_ordered

# Run with more test cases (default: 256)
PROPTEST=1000 cargo test --test property_tests

# Run single-threaded for reproducibility
cargo test --test property_tests -- --test-threads=1

# Verbose output
cargo test --test property_tests -- --nocapture

# Persist failing cases
PROPTEST=PERSIST cargo test --test property_tests
```

## Test Categories at a Glance

| Category | Tests | Module |
|----------|-------|--------|
| File Sorting | 5 | `state::files` |
| Selection Operations | 5 | `state::selection` |
| Bulk Rename | 6 | `file_ops::bulk_rename` |
| Undo/Redo | 6 | `file_ops::history` |
| File Operations | 5 | `file_ops::operations` |
| Path Operations | 5 | `std::path` |
| Combined Operations | 4 | Multiple |

## Common Strategies

```rust
// Generate valid file names
file_name_strategy()

// Generate PathBuf
path_strategy()

// Generate FileEntry
file_entry_strategy()

// Generate list of FileEntry (0-100 items)
file_list_strategy()

// Generate SortBy enum
sort_by_strategy()

// Generate SortOrder enum
sort_order_strategy()
```

## Test Template

```rust
proptest! {
    #[test]
    fn prop_my_test(input in my_strategy()) {
        // Setup
        let sut = create_system();

        // Execute
        let result = sut.operation(input);

        // Verify
        prop_assert!(result.is_valid(),
            "Property violated: {:?}", input);
    }
}
```

## Common Patterns

### Idempotence
```rust
prop_assert_eq!(once(x), twice(once(x)));
```

### Associativity
```rust
prop_assert_eq!(op(a, op(b, c)), op(op(a, b), c));
```

### Invertibility
```rust
prop_assert_eq!(x, inverse(operation(x)));
```

### Preservation
```rust
prop_assert_eq!(length(x), length(operation(x)));
```

## Failure Debugging

When a test fails:

1. **Read the error message carefully** - It shows the minimal failing case
2. **Copy the seed** - Use it to reproduce: `cargo test -- --test-threads=1`
3. **Check the invariant** - Is the property correctly specified?
4. **Examine the strategy** - Is it generating valid inputs?

## File Locations

- Tests: `/tests/property_tests.rs`
- Documentation: `/tests/PROPERTY_TESTS.md`
- Summary: `/tests/PROPERTY_TESTS_SUMMARY.md`
- Guide: `/tests/ADDING_PROPERTY_TESTS.md`

## CI Integration

```bash
# Run in CI pipeline
make test          # All tests
make ci            # Check + format + tests
```

## Performance Tips

- Keep input sizes reasonable (0-100, not 0-10000)
- Use `--test-threads=1` for reproducible failures
- Reduce test count for slow operations: `PROPTEST=100`

## Resources

- [Property Tests Summary](./PROPERTY_TESTS_SUMMARY.md)
- [Adding Property Tests](./ADDING_PROPERTY_TESTS.md)
- [Main Documentation](./PROPERTY_TESTS.md)
- [Proptest Docs](https://docs.rs/proptest/)

## Quick Examples

### Test sorting preserves elements
```bash
cargo test --test property_tests prop_sort_preserves_elements
```

### Test selection invertibility
```bash
cargo test --test property_tests prop_invert_twice_returns_original
```

### Test bulk rename numbering
```bash
cargo test --test property_tests prop_numbered_pattern_sequential
```

---

**Last Updated**: 2026-02-06
