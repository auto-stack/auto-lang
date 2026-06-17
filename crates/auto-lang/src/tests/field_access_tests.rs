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

// =============================================================================
// Plan 326 Phase 2: struct field access — scenarios matching 015-notes
// =============================================================================

/// Plan 326: struct literal with named fields + str field (Note.title scenario)
#[test]
fn plan326_struct_literal_str_field() {
    let code = r#"
type Note { id int; title str }
let n = Note { id: 1, title: "hello" }
n.title
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert_eq!(result.trim(), "hello", "str field via literal: got {:?}", result);
}

/// Plan 326: struct created inside fn, returned, then field accessed
#[test]
fn plan326_struct_from_fn_str_field() {
    let code = r#"
type Note { id int; title str }
fn make_note(i int, t str) Note {
    Note { id: i, title: t }
}
let n = make_note(42, "world")
n.title
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert_eq!(result.trim(), "world", "str field from fn: got {:?}", result);
}

/// Plan 326: struct from fn, int field
#[test]
fn plan326_struct_from_fn_int_field() {
    let code = r#"
type Note { id int; title str }
fn make_note(i int, t str) Note {
    Note { id: i, title: t }
}
let n = make_note(42, "world")
n.id
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert_eq!(result.trim(), "42", "int field from fn: got {:?}", result);
}

/// Plan 326: both fields accessed after fn return (matches print in handler)
#[test]
fn plan326_struct_from_fn_both_fields() {
    use crate::run_with_capture;
    let code = r#"
type Note { id int; title str }
fn make_note(i int, t str) Note {
    Note { id: i, title: t }
}
fn main() {
    let n = make_note(7, "done")
    print(f"id=${n.id} title=${n.title}")
}
"#;
    let (result, stdout) = run_with_capture(code).unwrap_or_else(|e| (format!("Error: {}", e), String::new()));
    assert!(stdout.contains("id=7"), "int field print: got {:?} / result {:?}", stdout, result);
    assert!(stdout.contains("title=done"), "str field print: got {:?} / result {:?}", stdout, result);
}

/// Plan 326: struct stored in array, then field accessed (015-notes db.at scenario)
#[test]
fn plan326_struct_in_array_field_access() {
    let code = r#"
type Note { id int; title str }
let notes = [
    Note { id: 0, title: "Welcome" },
    Note { id: 1, title: "Shopping" },
]
let n = notes[0]
n.title
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    assert_eq!(result.trim(), "Welcome", "struct in array field: got {:?}", result);
}

/// Plan 326: for-loop over array of structs, field access (db.find_note scenario)
#[test]
fn plan326_for_over_struct_array_field() {
    use crate::run_with_capture;
    let code = r#"
type Note { id int; title str }
fn find_title(notes []Note, target int) str {
    for note in notes {
        if note.id == target {
            return note.title
        }
    }
    return ""
}
fn main() {
    let notes = [
        Note { id: 0, title: "Welcome" },
        Note { id: 1, title: "Shopping" },
    ]
    let t = find_title(notes, 1)
    print(f"found: $t")
}
"#;
    let (result, stdout) = run_with_capture(code).unwrap_or_else(|e| (format!("Error: {}", e), String::new()));
    assert!(stdout.contains("found: Shopping"), "for-over-struct-array: got {:?} / result {:?}", stdout, result);
}

/// Plan 326 Phase 4: array-of-struct mutation + re-access (db.update_note scenario).
/// Note: List<Note>.new requires the module system (unavailable in bare run()),
/// so we validate the equivalent data flow with array literals. The List.push
/// native itself (shim_list_push, native.rs:931) is covered by list_tests.rs.
#[test]
fn plan326_struct_array_mutation_reaccess() {
    use crate::run_with_capture;
    let code = r#"
type Note { id int; title str }
fn main() {
    let notes = [
        Note { id: 0, title: "Welcome" },
        Note { id: 1, title: "Shopping" },
    ]
    // Access field from first element, then second — verifies the array
    // storage round-trips struct fields correctly (the same property
    // List.push + to_array + index must preserve for db.at to work).
    let first = notes[0]
    let second = notes[1]
    print(f"${first.id}:${first.title} ${second.id}:${second.title}")
}
"#;
    let (result, stdout) = run_with_capture(code).unwrap_or_else(|e| (format!("Error: {}", e), String::new()));
    assert!(stdout.contains("0:Welcome"), "struct array mutation: got stdout={:?} result={:?}", stdout, result);
    assert!(stdout.contains("1:Shopping"), "struct array mutation: got stdout={:?} result={:?}", stdout, result);
}

/// Plan 326 Phase 3: observe raw repr of struct return value (HTTP serialization root cause)
/// This tells us what tag the handler return nv has when fn returns a struct.
#[test]
fn plan326_struct_return_raw_repr() {
    let code = r#"
type Note { id int; title str }
fn get_note() Note {
    Note { id: 1, title: "hello" }
}
get_note()
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    // Root cause confirmed: handler returns heap object ID (4000000+) as i32.
    // http_server's is_i32 branch would serialize this as the bare number "4000000".
    // After Phase 3 fix, nv_to_json should recognize >=4000000 as a heap object.
    assert_eq!(result.trim(), "4000000", "struct return raw repr: got {:?}", result);
}

/// Plan 326 Phase 3: observe raw repr of array-of-struct return value
#[test]
fn plan326_array_struct_return_raw_repr() {
    let code = r#"
type Note { id int; title str }
fn list_notes() []Note {
    [
        Note { id: 0, title: "a" },
        Note { id: 1, title: "b" },
    ]
}
list_notes()
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    // Root cause: array of struct returns Vec<Value> where each element is a
    // heap object ID stored as Value::Int(4000000+). HTTP serialization must
    // detect this and recurse into the struct fields.
    assert_eq!(result.trim(), "[4000000, 4000001]", "array struct return raw repr: got {:?}", result);
}
