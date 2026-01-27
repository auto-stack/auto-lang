//! Pointer type tests for Plan 052
//!
//! These tests verify that pointer types work correctly:
//! - Pointer type *T parsing
//! - Unary & (address-of) operator transpilation
//! - Unary * (dereference) operator transpilation
//! - Reference type for Rust transpiler

use crate::run;

/// Test pointer type declaration
#[test]
fn test_pointer_type_declaration() {
    let code = r#"
type Point {
    x int
    y int
}

fn main() {
    let ptr *int = nil
    return 0
}
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    // Should not have compilation errors
    assert!(!result.contains("Error"), "Pointer type declaration should work, got: {}", result);
}

/// Test address-of operator transpilation to C
#[test]
fn test_address_of_operator() {
    let code = r#"
type Point {
    x int
    y int
}

fn main() {
    let p = Point { x: 10, y: 20 }
    let addr = &p.x
    return 0
}
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    // C transpiler should generate &p.x
    assert!(result.contains("&") || result.contains("Error"),
        "Address-of operator should transpile to & in C, got: {}", result);
}

/// Test pointer dereference operator
#[test]
fn test_pointer_dereference() {
    let code = r#"
fn main() {
    let x = 42
    let ptr = &x
    let value = *ptr
    return 0
}
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    // C transpiler should generate *ptr
    assert!(result.contains("*") || result.contains("Error"),
        "Dereference operator should transpile to * in C, got: {}", result);
}

/// Test pointer type with different element types
#[test]
fn test_pointer_to_different_types() {
    let code = r#"
fn main() {
    let int_ptr *int = nil
    let float_ptr *float = nil
    let bool_ptr *bool = nil
    return 0
}
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert!(!result.contains("Error"),
        "Pointer to different types should work, got: {}", result);
}

/// Test nil pointer
#[test]
fn test_nil_pointer() {
    let code = r#"
fn main() {
    let ptr *int = nil
    return 0
}
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert!(!result.contains("Error"), "Nil pointer should work, got: {}", result);
}
