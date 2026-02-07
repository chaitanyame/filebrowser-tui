# Property-Based Tests for File Browser TUI

This directory contains property-based tests using the `proptest` crate to verify invariants hold true across random inputs, catching edge cases and logic errors that might be missed by unit tests.

## Overview

Property-based testing is a powerful technique where you specify properties (invariants) that your code should satisfy, and the testing framework generates hundreds or thousands of random inputs to verify these properties hold true.

## Running the Tests

```bash
# Run all property tests
cargo test --test property_tests

# Run specific property test
cargo test --test property_tests prop_sort_by_name_is_ordered

# Run with more test cases (default is 256)
cargo test --test property_tests -- --test-threads=1 PROPTEST=1000

# Run on failure to reproduce
cargo test --test property_tests prop_sort_by_name_is_ordered -- --test-threads=1

# Persist failing test cases
cargo test --test property_tests -- --test-threads=1 PROPTEST=PERSIST
```

## Test Categories

### 1. File Sorting Properties

Tests for the `sort_files` function in `state/files.rs`:

- **prop_sort_by_name_is_ordered**: Verifies alphabetical ordering after sorting by name
- **prop_sort_directories_before_files**: Ensures directories always precede files
- **prop_sort_twice_is_idempotent**: Sorting twice yields the same result
- **prop_sort_preserves_elements**: No files are lost or duplicated during sorting
- **prop_sort_descending_is_reverse_of_ascending**: Descending sort is the reverse of ascending

### 2. Selection Operations Properties

Tests for the `SelectionManager` in `state/selection.rs`:

- **prop_select_all_deselect_all_is_empty**: Select all + deselect all = empty selection
- **prop_invert_twice_returns_original**: Inverting selection twice returns to original
- **prop_toggle_twice_returns_original**: Toggling twice returns to original state
- **prop_select_range_respects_bounds**: Select range never selects outside valid indices
- **prop_select_invert_covers_all_paths**: Invert selection covers all non-selected paths

### 3. Bulk Rename Properties

Tests for the `BulkRenamer` in `file_ops/bulk_rename.rs`:

- **prop_numbered_pattern_sequential**: Numbered patterns produce sequential numbers
- **prop_regex_substitution_valid_paths**: Regex substitution never produces invalid paths
- **prop_case_transform_preserves_length**: Case transforms preserve string length
- **prop_empty_find_does_nothing**: Empty find pattern doesn't change names
- **prop_extension_replace_changes_only_extension**: Extension replace only changes the extension
- **prop_uppercase_then_lowercase_preserves**: Uppercase then lowercase preserves original length

### 4. Undo/Redo Properties

Tests for the `HistoryManager` in `file_ops/history.rs`:

- **prop_cannot_undo_empty_history**: Cannot undo when history is empty
- **prop_cannot_redo_empty_stack**: Cannot redo when redo stack is empty
- **prop_record_clears_redo_stack**: Recording new operations clears the redo stack
- **prop_undo_count_increases**: Undo count increments with each recorded operation
- **prop_clear_empties_stacks**: Clear operation empties both undo and redo stacks
- **prop_undo_redo_restores_counts**: Undo followed by redo restores original counts

### 5. File Operations Properties

Tests for file operations in `file_ops/operations.rs`:

- **prop_copy_preserves_content**: Copy operation preserves file content exactly
- **prop_move_preserves_content**: Move changes location but not content
- **prop_delete_removes_file**: Delete operation removes the file
- **prop_copy_creates_separate_file**: Copy creates independent files
- **prop_copy_directory_preserves_structure**: Directory copy preserves structure

### 6. Path Operations Properties

Tests for path manipulation:

- **prop_parent_of_parent_is_grandparent**: Parent chain is consistent
- **prop_join_absolute_ignores_base**: Joining with absolute path ignores base
- **prop_extension_extraction_consistent**: Extension extraction is consistent
- **prop_filename_extraction_lossless**: Filename extraction is lossless
- **prop_ancestor_chain_terminates**: Ancestor chain terminates at root

### 7. Combined Operations Properties

Tests for interactions between multiple operations:

- **prop_sort_then_filter_preserves_order**: Sort then filter maintains ordering
- **prop_selection_operations_idempotent**: Selection operations are idempotent
- **prop_empty_selection_is_safe**: Empty selection operations are safe
- **prop_history_respects_max_size**: History respects maximum size limit

## Custom Strategies

The tests use custom proptest strategies for generating valid test data:

### `file_name_strategy()`
Generates valid file names without invalid characters (no `/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`)

### `path_strategy()`
Generates valid `PathBuf` objects from file names

### `file_entry_strategy()`
Generates `FileEntry` objects with random but valid properties

### `file_list_strategy()`
Generates vectors of `FileEntry` objects (0-100 entries)

## Understanding Failures

When a property test fails, proptest will:

1. Show the minimal failing case (shrunk input)
2. Display the seed for reproduction
3. Persist the failing case if `PROPTEST=PERSIST` is set

Example failure output:
```
thread 'prop_sort_by_name_is_ordered' panicked at 'Files not in order: zebra should come before apple', ...
```

To reproduce:
```bash
cargo test prop_sort_by_name_is_ordered -- --exact --test-threads=1 --nocapture
```

## Best Practices

### Writing New Property Tests

1. **Identify Invariants**: What should always be true regardless of input?
   - Example: "After sorting, elements are in order"

2. **Use Custom Strategies**: Generate valid, realistic test data
   - Avoid generating invalid paths, empty strings where not allowed

3. **Keep Tests Focused**: Each test should verify one property
   - Makes failures easier to understand

4. **Add Helpful Messages**: Use `prop_assert!` with descriptive messages
   - Helps debug when tests fail

5. **Consider Edge Cases**:
   - Empty collections
   - Single element
   - All identical elements
   - Maximum values

### Example Property Test

```rust
proptest! {
    #[test]
    fn prop_my_invariant(input in my_strategy()) {
        // Setup
        let mut sut = create_system_under_test();

        // Execute
        sut.process(input.clone());

        // Verify invariant
        prop_assert!(sut.check_invariant(),
            "Invariant violated for input: {:?}", input);
    }
}
```

## Integration with CI

Property tests are integrated into the CI pipeline via `make test` or `make ci`:

```bash
# Run all tests including property tests
make test

# CI workflow
make ci  # Runs: check, fmt-check, test
```

## Performance Considerations

Property tests can be slower than unit tests due to:
- Generating many random inputs
- Running the same code hundreds of times

Tips:
- Keep test data size reasonable (e.g., 0-100 items, not 0-10000)
- Use `--test-threads=1` for reproducible failures
- Consider reducing test count for slow operations:

```rust
proptest! {
    #[test]
    fn prop_slow_operation(input in prop::collection::vec(strategy(), 0..10)) {
        // Only generate 0-10 items instead of default 0-100
    }
}
```

## Further Reading

- [Proptest Book](https://altsysrq.github.io/proptest-book/intro.html)
- [Property-Based Testing in Rust](https://blog.yoshuawuyts.com/property-based-testing-in-rust/)
- [Rust Testing Patterns](https://matklad.github.io/2021/05/31/how-to-test.html)

## License

These tests are part of the File Browser TUI project and share the same license.
