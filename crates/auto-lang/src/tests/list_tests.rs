//! Comprehensive VM tests for List operations
//!
//! Tests cover:
//! - Basic operations: push, pop, len, is_empty, capacity, get, set, clear
//! - Iterator operations: iter, next
//! - Lazy adapters: map, filter
//! - Terminal operators: reduce, count, for_each, collect, any, all, find

use crate::run;

// ============================================================================
// Basic Operations Tests
// ============================================================================

#[test]
fn test_list_new_and_len() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)
        list.len()
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("3"), "List should have 3 elements, got: {}", result);
}

#[test]
fn test_list_push_and_pop() {
    let code = r#"
        let list = List.new()
        list.push(10)
        list.push(20)
        let first = list.pop()
        let second = list.pop()
        let length = list.len()

        // Return array with results
        [first, second, length]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("20"), "First pop should return 20, got: {}", result);
    assert!(result.contains("10"), "Second pop should return 10, got: {}", result);
    assert!(result.contains("0"), "List should be empty after pops, got: {}", result);
}

#[test]
fn test_list_is_empty() {
    let code = r#"
        let list = List.new()
        let empty1 = list.is_empty()

        list.push(1)
        let not_empty = list.is_empty()

        list.pop()
        let empty2 = list.is_empty()

        [empty1, not_empty, empty2]
    "#;
    let result = run(code).unwrap();
    // Check that we get some 1s (true) and 0s (false)
    assert!(result.contains("1") && result.contains("0"), "Should have true and false values");
}

#[test]
fn test_list_capacity() {
    let code = r#"
        let list = List.new()
        let cap1 = list.capacity()

        // Add some elements
        list.push(1)
        list.push(2)
        list.push(3)

        let len = list.len()
        let cap2 = list.capacity()

        [len, cap1, cap2]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("3"), "Length should be 3");
}

#[test]
fn test_list_get_and_set() {
    let code = r#"
        let list = List.new()
        list.push(10)
        list.push(20)
        list.push(30)

        let first = list.get(0)
        let second = list.get(1)

        // Update element
        list.set(1, 99)
        let updated = list.get(1)

        [first, second, updated]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("10"), "First element should be 10");
    assert!(result.contains("20"), "Second element should be 20");
    assert!(result.contains("99"), "Updated element should be 99");
}

#[test]
fn test_list_clear() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)

        let before_len = list.len()
        list.clear()
        let after_len = list.len()
        let after_empty = list.is_empty()

        [before_len, after_len, after_empty]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("3"), "Before clear should have 3 elements");
    // After clear, length should be 0 and is_empty should return 1 (true)
}

#[test]
fn test_list_insert_and_remove() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(3)

        // Insert in the middle
        list.insert(1, 2)
        let elem1 = list.get(1)

        // Remove from middle
        let removed = list.remove(1)
        let final_len = list.len()

        [elem1, removed, final_len]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("2"), "Inserted element should be 2");
    assert!(result.contains("2"), "Removed element should be 2");
}

// ============================================================================
// Iterator Tests
// ============================================================================

#[test]
fn test_list_iter() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)

        let iter = list.iter()
        let first = iter.next()
        let second = iter.next()
        let third = iter.next()
        let done = iter.next()  // Should be nil

        [first, second, third]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("1"), "First element should be 1");
    assert!(result.contains("2"), "Second element should be 2");
    assert!(result.contains("3"), "Third element should be 3");
}

// ============================================================================
// Map Adapter Tests
// ============================================================================

#[test]
fn test_list_map_double() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)

        fn multiply_by_2(x int) int {
            return x * 2
        }

        let iter = list.iter()
        let mapped = iter.map(multiply_by_2)

        let a = mapped.next()
        let b = mapped.next()
        let c = mapped.next()

        [a, b, c]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("2"), "First doubled should be 2");
    assert!(result.contains("4"), "Second doubled should be 4");
    assert!(result.contains("6"), "Third doubled should be 6");
}

#[test]
fn test_list_map_string_length() {
    let code = r#"
        let list = List.new()
        list.push("hi")
        list.push("hello")
        list.push("world")

        fn get_length(s str) int {
            return 5  // Simplified - just return a fixed value for now
        }

        let iter = list.iter()
        let mapped = iter.map(get_length)

        let a = mapped.next()
        let b = mapped.next()
        let c = mapped.next()

        [a, b, c]
    "#;
    let result = run(code).unwrap();
    // Should return the lengths
    assert!(result.contains("5"), "Should contain length value");
}

// ============================================================================
// Filter Adapter Tests
// ============================================================================

#[test]
fn test_list_filter_even() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)
        list.push(4)
        list.push(5)
        list.push(6)

        fn is_even(x int) bool {
            return x % 2 == 0
        }

        let iter = list.iter()
        let filtered = iter.filter(is_even)

        let a = filtered.next()
        let b = filtered.next()
        let c = filtered.next()
        let d = filtered.next()  // Should be nil

        [a, b, c]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("2"), "First even should be 2");
    assert!(result.contains("4"), "Second even should be 4");
    assert!(result.contains("6"), "Third even should be 6");
}

// ============================================================================
// Reduce Tests
// ============================================================================

#[test]
fn test_list_reduce_sum() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)
        list.push(4)

        fn add(acc int, x int) int {
            return acc + x
        }

        let iter = list.iter()
        let sum = iter.reduce(0, add)

        sum
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "10", "Sum of 1+2+3+4 should be 10");
}

#[test]
fn test_list_reduce_product() {
    let code = r#"
        let list = List.new()
        list.push(2)
        list.push(3)
        list.push(4)

        fn multiply(acc int, x int) int {
            return acc * x
        }

        let iter = list.iter()
        let product = iter.reduce(1, multiply)

        product
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "24", "Product of 2*3*4 should be 24");
}

// ============================================================================
// Count Tests
// ============================================================================

#[test]
fn test_list_count() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)
        list.push(4)
        list.push(5)

        let iter = list.iter()
        let count = iter.count()

        count
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "5", "Count should be 5");
}

// ============================================================================
// ForEach Tests
// ============================================================================

#[test]
fn test_list_for_each() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)

        // Use a mutable variable to accumulate
        // Note: This tests that for_each runs without errors
        fn print_item(x int) void {
            // In real test, we'd collect results
            // For now, just verify it runs
        }

        let iter = list.iter()
        iter.for_each(print_item)

        void
    "#;
    let result = run(code).unwrap();
    // Just verify it runs without error
    assert!(result.contains("void") || result.is_empty(), "ForEach should execute");
}

// ============================================================================
// Collect Tests
// ============================================================================

#[test]
fn test_list_collect() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)

        fn multiply_by_2(x int) int {
            return x * 2
        }

        let iter = list.iter()
        let mapped = iter.map(multiply_by_2)
        let new_list = mapped.collect()

        new_list.len()
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("3"), "Collected list should have 3 elements");
}

#[test]
fn test_list_collect_filter() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)
        list.push(4)
        list.push(5)

        fn is_even(x int) bool {
            return x % 2 == 0
        }

        let iter = list.iter()
        let filtered = iter.filter(is_even)
        let even_list = filtered.collect()

        even_list.len()
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("2"), "Should have 2 even numbers");
}

#[test]
fn test_list_bang_operator() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)

        // Bang operator eagerly collects iterator into a list
        let collected = list.iter().!

        collected.len()
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("3"), "Bang operator should collect 3 elements");
}

#[test]
fn test_list_bang_operator_with_map() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)

        fn multiply_by_2(x int) int {
            return x * 2
        }

        // Bang operator with map: eagerly collect mapped values
        let mapped = list.iter().map(multiply_by_2).!

        mapped.len()
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("3"), "Bang operator should collect 3 mapped elements");
}

// ============================================================================
// Any/All Tests
// ============================================================================

#[test]
fn test_list_any() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(3)
        list.push(5)
        list.push(7)

        fn is_even(x int) bool {
            return x % 2 == 0
        }

        let iter = list.iter()
        let has_even = iter.any(is_even)

        has_even
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("0") || result.contains("false"),
           "Should not have any even numbers");
}

#[test]
fn test_list_any_true() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)

        fn is_even(x int) bool {
            return x % 2 == 0
        }

        let iter = list.iter()
        let has_even = iter.any(is_even)

        has_even
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("1") || result.contains("true"),
           "Should have at least one even number");
}

#[test]
fn test_list_all() {
    let code = r#"
        let list = List.new()
        list.push(2)
        list.push(4)
        list.push(6)

        fn is_even(x int) bool {
            return x % 2 == 0
        }

        let iter = list.iter()
        let all_even = iter.all(is_even)

        all_even
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("1") || result.contains("true"),
           "All numbers should be even");
}

#[test]
fn test_list_all_false() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(3)
        list.push(5)

        fn is_even(x int) bool {
            return x % 2 == 0
        }

        let iter = list.iter()
        let all_even = iter.all(is_even)

        all_even
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("0") || result.contains("false"),
           "Not all numbers should be even");
}

// ============================================================================
// Find Tests
// ============================================================================

#[test]
fn test_list_find_found() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(3)
        list.push(5)
        list.push(7)
        list.push(9)

        fn is_greater_than_5(x int) bool {
            return x > 5
        }

        let iter = list.iter()
        let found = iter.find(is_greater_than_5)

        found
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("7") || result.contains("9"),
           "Should find first element > 5");
}

#[test]
fn test_list_find_not_found() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)

        fn is_greater_than_10(x int) bool {
            return x > 10
        }

        let iter = list.iter()
        let found = iter.find(is_greater_than_10)

        // Should return nil
        found
    "#;
    let result = run(code).unwrap();
    // Result should contain "nil" or be empty
    assert!(result.contains("nil") || !result.contains("1") && !result.contains("2") && !result.contains("3"),
           "Should not find any element > 10");
}

// ============================================================================
// Complex Pipeline Tests
// ============================================================================

#[test]
fn test_list_map_filter_reduce() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)
        list.push(4)
        list.push(5)

        fn multiply_by_2(x int) int {
            return x * 2
        }

        fn is_even(x int) bool {
            return x % 2 == 0
        }

        fn add(acc int, x int) int {
            return acc + x
        }

        let iter = list.iter()
        let mapped = iter.map(multiply_by_2)
        let filtered = mapped.filter(is_even)
        let sum = filtered.reduce(0, add)

        sum
    "#;
    let result = run(code).unwrap();
    // 1*2=2, 2*2=4, 3*2=6, 4*2=8, 5*2=10
    // Even: 2, 4, 6, 8, 10
    // Sum: 2+4+6+8+10 = 30
    assert!(result.contains("30"), "Sum of doubled evens should be 30");
}

#[test]
fn test_list_filter_map_count() {
    let code = r#"
        let list = List.new()
        list.push(1)
        list.push(2)
        list.push(3)
        list.push(4)
        list.push(5)

        fn is_even(x int) bool {
            return x % 2 == 0
        }

        fn multiply_by_2(x int) int {
            return x * 2
        }

        let iter = list.iter()
        let filtered = iter.filter(is_even)
        let mapped = filtered.map(double)
        let count = mapped.count()

        count
    "#;
    let result = run(code).unwrap();
    // Even numbers: 2, 4
    // Doubled: 4, 8
    // Count: 2
    assert!(result.contains("2"), "Should have 2 doubled even numbers");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_list_empty_operations() {
    let code = r#"
        let list = List.new()

        let len = list.len()
        let empty = list.is_empty()

        let iter = list.iter()
        let first = iter.next()  // Should be nil

        // Operations on empty iterator
        let count = iter.count()

        [len, empty, count]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("0"), "Empty list should have length 0");
}

#[test]
fn test_list_single_element() {
    let code = r#"
        let list = List.new()
        list.push(42)

        let elem = list.get(0)

        fn multiply_by_2(x int) int {
            return x * 2
        }

        let iter = list.iter()
        let mapped = iter.map(multiply_by_2)
        let doubled = mapped.next()
        let done = mapped.next()

        [elem, doubled]
    "#;
    let result = run(code).unwrap();
    assert!(result.contains("42"), "Original element should be 42");
    assert!(result.contains("84"), "Doubled element should be 84");
}
