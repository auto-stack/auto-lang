//! Integration test for const generic parameters in type/tag definitions (Plan 052)
//!
//! These tests verify that const generic parameters work in real-world scenarios:
//! - tag Inline<T, const N u32> { ... }
//! - type Buffer<T, const SIZE u32> { ... }

use crate::run;

/// Test const generic parameter in tag definition
#[test]
fn test_const_generic_in_tag() {
    let code = r#"
tag Inline<T, N u32> {
    buffer: [N]T
}

fn main() {
    return 0
}
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    // Should not have compilation errors
    assert!(!result.contains("syntax error"),
        "Const generic in tag should parse correctly, got: {}", result);
}

/// Test type parameter still works in tag
#[test]
fn test_type_param_in_tag() {
    let code = r#"
tag May<T> {
    value: T
}

fn main() {
    return 0
}
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert!(!result.contains("syntax error"),
        "Type parameter in tag should still work, got: {}", result);
}

/// Test mixed type and const parameters
#[test]
fn test_mixed_params_in_tag() {
    let code = r#"
tag FixedBuffer<T, SIZE u32> {
    data: [SIZE]T
    count u32
}

fn main() {
    return 0
}
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert!(!result.contains("syntax error"),
        "Mixed type and const parameters should work, got: {}", result);
}

/// Test const generic in type definition
#[test]
fn test_const_generic_in_type() {
    let code = r#"
type Array<T, N u32> {
    data: [N]T
    len u32
}

fn main() {
    return 0
}
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert!(!result.contains("syntax error"),
        "Const generic in type should parse correctly, got: {}", result);
}

/// Test multiple const parameters
#[test]
fn test_multiple_const_params() {
    let code = r#"
type Matrix<ROWS u32, COLS u32> {
    data: [ROWS * COLS]int
}

fn main() {
    return 0
}
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert!(!result.contains("syntax error"),
        "Multiple const parameters should work, got: {}", result);
}

/// Test const parameter with usize type
#[test]
fn test_const_param_usize() {
    let code = r#"
type Buffer<SIZE usize> {
    data: [SIZE]byte
}

fn main() {
    return 0
}
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert!(!result.contains("syntax error"),
        "Const parameter with usize should work, got: {}", result);
}
