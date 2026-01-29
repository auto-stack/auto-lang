//! Field access tests for Plan 056
//!
//! These tests verify that field access syntax works correctly:
//! - Reading fields from instances
//! - Field access doesn't move the base object
//! - Nested field access
//! - Field access with different types

use crate::run;

/// Test basic field access
#[test]
fn test_field_access_basic() {
    let code = r#"
type Point {
    x int
    y int
}

let p = Point(10, 20)
p.x
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert!(result.contains("10"), "Expected result to contain '10', got: {}", result);
}

/// Test field access doesn't move the object
#[test]
fn test_field_access_no_move() {
    let code = r#"
type Point {
    x int
    y int
}

let p = Point(1, 2)
p.x
p.y
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    // Should not get "Use after move" error
    assert!(!result.contains("Use after move"), "Field access should not move object");
    assert!(result.contains("1") || result.contains("2"), "Should access both fields");
}

/// Test multiple field accesses from same object
#[test]
fn test_multiple_field_accesses() {
    let code = r#"
type Data {
    a int
    b int
    c int
}

let d = Data(1, 2, 3)
d.a
d.b
d.c
d.a  // Access a again - last expr determines result
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert!(!result.contains("Use after move"), "Multiple field accesses should not fail");
    assert!(result.contains("1") || result.contains("2") || result.contains("3"), "Should access fields");
}

/// Test field assignment and access
#[test]
fn test_field_assignment_and_access() {
    let code = r#"
type Point {
    x int
    y int
}

let p = Point(1, 2)
p.x = 10
p.y = 20
p.x  // Return x
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert!(result.contains("10"), "Should access updated x field, got: {}", result);
}

/// Test nested field access (when we have nested types)
#[test]
fn test_nested_field_access() {
    let code = r#"
type Inner {
    value int
}

type Outer {
    inner Inner
}

let outer = Outer(Inner(42))
outer.inner.value
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert!(result.contains("42"), "Should access nested field value, got: {}", result);
}

/// Test field access on type instances created with positional args
#[test]
fn test_field_access_positional_args() {
    let code = r#"
type Point {
    x int
    y int
}

let p = Point(1, 2)
p.x
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert!(result.contains("1"), "Should access x from positional arg, got: {}", result);
}

/// Test field access returns correct type
#[test]
fn test_field_access_type() {
    let code = r#"
type Data {
    name str
    count int
    active bool
}

let d = Data("test", 42, true)
d.name
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert!(result.contains("test"), "Should access str field, got: {}", result);
}

/// Test field access with int type
#[test]
fn test_field_access_int() {
    let code = r#"
type Data {
    value int
}

let d = Data(42)
d.value
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert!(result.contains("42"), "Should access int field, got: {}", result);
}

/// Test field access with bool type
#[test]
fn test_field_access_bool() {
    let code = r#"
type Data {
    active bool
}

let d = Data(true)
d.active
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert!(result.contains("true"), "Should access bool field, got: {}", result);
}
