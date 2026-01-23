//! Tests for VM memory management functions (Plan 052 Phase 2)
//!
//! These tests verify that alloc_array, free_array, and realloc_array
//! work correctly in the VM evaluator.

use crate::run;

#[test]
fn test_alloc_array_basic() {
    let code = r#"
        let arr = alloc_array(10)
        arr.len()
    "#;

    let result = run(code).unwrap();
    assert_eq!(result.trim(), "10", "alloc_array(10) should create array with len 10");
}

#[test]
fn test_alloc_array_empty() {
    let code = r#"
        let arr = alloc_array(0)
        arr.len()
    "#;

    let result = run(code).unwrap();
    assert_eq!(result.trim(), "0", "alloc_array(0) should create empty array");
}

#[test]
fn test_realloc_array_growth() {
    let code = r#"
        let arr = alloc_array(5)
        let new_arr = realloc_array([arr, 10])
        new_arr.len()
    "#;

    let result = run(code).unwrap();
    assert_eq!(result.trim(), "10", "realloc_array should grow array to 10");
}

#[test]
fn test_realloc_array_preserves_data() {
    let code = r#"
        let arr = alloc_array(3)
        arr[0] = 1
        arr[1] = 2
        arr[2] = 3

        let new_arr = realloc_array([arr, 5])
        new_arr[0]
    "#;

    let result = run(code).unwrap();
    assert_eq!(result.trim(), "1", "realloc_array should preserve first element");
}

#[test]
fn test_realloc_array_wrapped_usage() {
    // Test the wrapped version that accepts [array, new_size]
    let code = r#"
        let small = alloc_array(2)
        small[0] = 42
        small[1] = 99

        let large = realloc_array([small, 10])
        large[0]
    "#;

    let result = run(code).unwrap();
    assert_eq!(result.trim(), "42", "realloc_array wrapped should preserve data");
}

#[test]
fn test_free_array_returns_nil() {
    let code = r#"
        let arr = alloc_array(5)
        free_array(arr)
    "#;

    let result = run(code).unwrap();
    // free_array returns Nil (which might be empty string or "nil" depending on output)
    assert!(result.trim().is_empty() || result.trim() == "nil" || result.trim() == "Nil",
            "free_array should return Nil");
}

#[test]
fn test_alloc_invalid_size() {
    // Test that invalid size returns an error
    let code = r#"
        alloc_array(-5)
    "#;

    let result = run(code);
    // Should return an error
    assert!(result.is_err() || result.unwrap().contains("Error"),
            "alloc_array with negative size should error");
}
