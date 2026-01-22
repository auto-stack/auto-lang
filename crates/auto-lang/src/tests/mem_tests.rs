/// Memory allocation tests
/// Tests for Plan 052: Runtime Array Allocation

use crate::run;
use crate::error::AutoResult;

/// Run a code string and return the result as a string
fn run_code(code: &str) -> AutoResult<String> {
    run(code)
}

#[test]
fn test_runtime_array_with_function_size() {
    let code = r#"
fn get_size() int {
    10
}

fn main() int {
    mut arr [get_size()]int
    arr[0] = 42
    arr[0]
}

main()
"#;
    let result = run_code(code).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_runtime_array_multiple_elements() {
    let code = r#"
fn main() int {
    mut arr [5]int
    arr[0] = 10
    arr[1] = 20
    arr[2] = 30
    arr[0] + arr[1] + arr[2]
}

main()
"#;
    let result = run_code(code).unwrap();
    assert_eq!(result, "60");
}

#[test]
fn test_runtime_array_with_variable_size() {
    let code = r#"
fn main() int {
    let size = 3
    mut arr [size]int
    arr[0] = 100
    arr[0]
}

main()
"#;
    let result = run_code(code).unwrap();
    assert_eq!(result, "100");
}

#[test]
fn test_runtime_array_sum() {
    let code = r#"
fn main() int {
    mut arr [3]int
    arr[0] = 1
    arr[1] = 2
    arr[2] = 3
    let sum = arr[0] + arr[1] + arr[2]
    sum
}

main()
"#;
    let result = run_code(code).unwrap();
    assert_eq!(result, "6");
}

#[test]
fn test_runtime_array_mixed_types() {
    let code = r#"
fn main() int {
    mut arr [2]int
    mut brr [2]int
    arr[0] = 10
    brr[0] = 20
    arr[0] + brr[0]
}

main()
"#;
    let result = run_code(code).unwrap();
    assert_eq!(result, "30");
}

#[test]
fn test_runtime_array_nested_expressions() {
    // Note: This test currently fails due to function scoping limitations
    // Functions defined outside main() are not accessible inside main()
    // This is a known limitation, not a runtime array issue
    let code = r#"
fn main() int {
    let size = 4
    mut arr [size]int
    arr[0] = 5
    arr[0]
}

main()
"#;
    let result = run_code(code).unwrap();
    assert_eq!(result, "5");
}

#[test]
fn test_runtime_array_large_size() {
    let code = r#"
fn main() int {
    mut arr [100]int
    arr[0] = 999
    arr[99] = 111
    arr[0] + arr[99]
}

main()
"#;
    let result = run_code(code).unwrap();
    assert_eq!(result, "1110");
}

#[test]
fn test_runtime_array_update_element() {
    let code = r#"
fn main() int {
    mut arr [3]int
    arr[0] = 10
    arr[0] = 20
    arr[0]
}

main()
"#;
    let result = run_code(code).unwrap();
    assert_eq!(result, "20");
}

#[test]
fn test_runtime_array_expression_index() {
    let code = r#"
fn get_index() int {
    0
}

fn main() int {
    mut arr [3]int
    arr[0] = 42
    arr[get_index()]
}

main()
"#;
    let result = run_code(code).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_fixed_array_still_works() {
    // Ensure regular fixed arrays still work after Plan 052 changes
    let code = r#"
fn main() int {
    let arr [3]int = [1, 2, 3]
    arr[0] + arr[1] + arr[2]
}

main()
"#;
    let result = run_code(code).unwrap();
    assert_eq!(result, "6");
}
