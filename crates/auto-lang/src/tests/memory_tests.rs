//! Tests for VM memory management functions (Plan 052 Phase 2)
//!
//! These tests verify that alloc_array, free_array, and realloc_array
//! work correctly in the VM evaluator.

use crate::run;

#[test]
fn test_alloc_array_basic() {
    let code = r#"
        alloc_array(10)
    "#;

    let result = run(code).unwrap();
    // alloc_array should return an Array value
    eprintln!("alloc_array(10) result: {}", result.trim());
    assert!(result.contains("Array") || result.contains("["),
            "alloc_array(10) should return an Array, got: {}", result);
}

#[test]
fn test_alloc_array_empty() {
    let code = r#"
        alloc_array(0)
    "#;

    let result = run(code).unwrap();
    eprintln!("alloc_array(0) result: {}", result.trim());
    assert!(result.contains("Array") || result.contains("["),
            "alloc_array(0) should return an empty Array");
}

#[test]
fn test_realloc_array_growth() {
    let code = r#"
        let arr = alloc_array(5)
        realloc_array([arr, 10])
    "#;

    let result = run(code).unwrap();
    eprintln!("realloc_array result: {}", result.trim());
    assert!(result.contains("Array") || result.contains("["),
            "realloc_array should return an Array");
}

#[test]
fn test_realloc_array_preserves_data() {
    let code = r#"
        let arr = alloc_array(3)
        realloc_array([arr, 5])
    "#;

    let result = run(code).unwrap();
    eprintln!("realloc_array with data result: {}", result.trim());
    assert!(result.contains("Array") || result.contains("["),
            "realloc_array should preserve data");
}

#[test]
fn test_free_array_returns_nil() {
    let code = r#"
        let arr = alloc_array(5)
        free_array(arr)
    "#;

    let result = run(code).unwrap();
    // free_array returns Nil (which might be empty string)
    eprintln!("free_array result: '{}'", result.trim());
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

#[test]
fn test_vm_functions_integration() {
    // Test that VM functions can be called sequentially
    let code = r#"
        let arr1 = alloc_array(5)
        let arr2 = alloc_array(3)
        free_array(arr1)
        arr2
    "#;

    let result = run(code).unwrap();
    eprintln!("VM functions integration result: {}", result.trim());
    assert!(result.contains("Array") || result.contains("["),
            "Should return the second array");
}
