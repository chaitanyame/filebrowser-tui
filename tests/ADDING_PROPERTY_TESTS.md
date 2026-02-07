# Example: Adding New Property Tests

This guide shows you how to add new property-based tests to the filebrowser-tui project.

## Basic Structure

Every property test follows this pattern:

```rust
proptest! {
    /// Brief description of what property is being tested
    #[test]
    fn prop_descriptive_test_name(
        input1 in strategy1(),
        input2 in strategy2()
    ) {
        // 1. Setup: Create system under test
        let mut sut = MyStruct::new();

        // 2. Execute: Perform the operation
        sut.do_something(input1, input2);

        // 3. Verify: Check the invariant holds
        prop_assert!(sut.check_invariant(),
            "Invariant violated: {}",
            sut.debug_info());
    }
}
```

## Example 1: Testing a Simple Function

Let's test a hypothetical `normalize_path` function:

```rust
// Add this to tests/property_tests.rs

proptest! {
    /// Property: Normalizing a path twice gives the same result as normalizing once
    #[test]
    fn prop_normalize_is_idempotent(path in path_strategy()) {
        use filebrowser_tui::path_utils::normalize_path;

        let once = normalize_path(&path);
        let twice = normalize_path(&once);

        prop_assert_eq!(once, twice,
            "Normalizing twice should give same result as once");
    }

    /// Property: Normalizing preserves the actual path semantics
    #[test]
    fn prop_normalize_preserves_semantics(path in path_strategy()) {
        use filebrowser_tui::path_utils::normalize_path;
        use std::fs;

        let normalized = normalize_path(&path);

        // If original path exists, normalized should point to same location
        if path.exists() {
            prop_assert_eq!(
                fs::canonicalize(&path).ok(),
                fs::canonicalize(&normalized).ok(),
                "Normalized path should point to same location"
            );
        }
    }
}
```

## Example 2: Testing State Transitions

Testing state machine-like behavior:

```rust
proptest! {
    /// Property: Filter then sort maintains sort order
    #[test]
    fn prop_filter_then_sort_sorted(
        mut files in file_list_strategy(),
        query in file_name_strategy()
    ) {
        use filebrowser_tui::state::files::{sort_files, filter_files, SortBy, SortOrder};

        // Filter first
        let indices = filter_files(&files, false, Some(&query));

        // Sort the filtered indices
        let mut filtered_files: Vec<_> = indices.iter().map(|&i| files[i].clone()).collect();
        sort_files(&mut filtered_files, SortBy::Name, SortOrder::Ascending);

        // Verify the filtered result is sorted
        for i in 1..filtered_files.len() {
            let prev = filtered_files[i-1].name.to_lowercase();
            let curr = filtered_files[i].name.to_lowercase();
            prop_assert!(prev <= curr,
                "Filtered files should be sorted: {} <= {}",
                prev, curr);
        }
    }
}
```

## Example 3: Testing with Custom Strategies

Create a custom strategy for your specific needs:

```rust
// Add a new strategy to the "Arbitrary Implementations" section

/// Strategy for generating valid search queries
fn search_query_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Empty query
        Just(String::new()),
        // Single character
        "[a-zA-Z]",
        // Multiple characters
        "[a-zA-Z0-9]{1,20}",
        // Query with wildcard
        "[a-zA-Z]{1,10}\\*",
    ]
}

/// Strategy for generating file lists with guaranteed duplicates
fn file_list_with_duplicates_strategy() -> impl Strategy<Value = Vec<FileEntry>> {
    prop::collection::vec(file_entry_strategy(), 1..50)
        .prop_map(|mut files| {
            // Add some duplicates
            let original_len = files.len();
            for i in 0..(original_len / 3) {
                if let Some(original) = files.get(i) {
                    files.push(original.clone());
                }
            }
            files
        })
}

// Now use these strategies in your tests
proptest! {
    #[test]
    fn prop_search_works(
        files in file_list_strategy(),
        query in search_query_strategy()
    ) {
        // Your test logic here
    }
}
```

## Example 4: Testing Error Handling

Test that error conditions are handled correctly:

```rust
proptest! {
    /// Property: Invalid paths are handled gracefully
    #[test]
    fn prop_invalid_paths_handled(
        invalid_name in "[/\\\\:*?\"<>|]{1,10}"
    ) {
        use filebrowser_tui::state::files::FileEntry;

        let path = PathBuf::from(invalid_name);

        // Should not panic, may return error or handle gracefully
        let result = FileEntry::from_path(path);

        // Either we get an error or a valid entry
        // We should never crash/panic
        match result {
            Ok(_) => {}, // Valid entry
            Err(_) => {}, // Expected error for invalid paths
        }
    }
}
```

## Example 5: Testing Numeric Properties

```rust
proptest! {
    /// Property: File size is always non-negative
    #[test]
    fn prop_file_size_non_negative(entry in file_entry_strategy()) {
        prop_assert!(entry.size >= 0,
            "File size should never be negative");
    }

    /// Property: Total size of files is at least sum of individual sizes
    #[test]
    fn prop_directory_size_at_least_sum(entries in file_list_strategy()) {
        use filebrowser_tui::file_ops::operations::calculate_size;

        let mut total_individual: u64 = 0;
        for entry in &entries {
            if !entry.is_dir {
                total_individual += entry.size;
            }
        }

        // If we had a directory containing these files,
        // its calculated size should be at least the sum
        // (it might be more due to directory overhead)
        prop_assert!(calculate_size(/* directory path */) >= total_individual);
    }
}
```

## Common Patterns

### Associativity: A ◦ (B ◦ C) = (A ◦ B) ◦ C

```rust
proptest! {
    #[test]
    fn prop_operation_is_associative(
        a in some_strategy(),
        b in some_strategy(),
        c in some_strategy()
    ) {
        let result1 = op(a, op(b, c));
        let result2 = op(op(a, b), c);

        prop_assert_eq!(result1, result2);
    }
}
```

### Idempotence: f(x) = f(f(x))

```rust
proptest! {
    #[test]
    fn prop_operation_is_idempotent(input in some_strategy()) {
        let once = operation(input.clone());
        let twice = operation(once);

        prop_assert_eq!(once, twice);
    }
}
```

### Commutativity: A ◦ B = B ◦ A

```rust
proptest! {
    #[test]
    fn prop_operation_is_commutative(
        a in some_strategy(),
        b in some_strategy()
    ) {
        let result1 = op(a.clone(), b.clone());
        let result2 = op(b, a);

        prop_assert_eq!(result1, result2);
    }
}
```

### Identity: f(identity, x) = x

```rust
proptest! {
    #[test]
    fn prop_has_identity_element(input in some_strategy()) {
        let identity = IdentityValue::new();
        let result = op(identity, input.clone());

        prop_assert_eq!(result, input);
    }
}
```

## Testing Checklist

Before adding a property test, ask yourself:

- [ ] What is the invariant I'm testing?
- [ ] Can I express it clearly in the test name?
- [ ] Is my strategy generating valid inputs?
- [ ] Will the test catch real bugs?
- [ ] Is the test focused on one property?
- [ ] Did I add helpful failure messages?

## Running Your New Tests

```bash
# Run just your new test
cargo test --test property_tests prop_your_new_test

# Run with verbose output
cargo test --test property_tests prop_your_new_test -- --nocapture

# Run many cases to find edge cases
PROPTEST=10000 cargo test --test property_tests prop_your_new_test

# Reproduce a failure with a specific seed
cargo test --test property_tests prop_your_new_test -- --test-threads=1
```

## Tips for Good Property Tests

1. **Start Simple**: Begin with obvious properties, then add edge cases
2. **Be Specific**: Clear test names make failures easier to understand
3. **Use Strategies**: Leverage existing strategies or create custom ones
4. **Shrinkable Inputs**: Proptest will shrink failing cases to minimal examples
5. **Test Invariants, Not Implementations**: Focus on what should be true, not how it's achieved

## Resources

- [Proptest Documentation](https://docs.rs/proptest/latest/proptest/)
- [Proptest Book](https://altsysrq.github.io/proptest-book/)
- [Property-Based Testing: A Comprehensive Guide](https://blog.ploeh.dk/2021/05/10/property-based-testing-intro/)

## Need Help?

Check the existing tests in `property_tests.rs` for more examples, or refer to `PROPERTY_TESTS.md` for an overview of all property tests in the project.
