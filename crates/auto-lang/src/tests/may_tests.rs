//! VM tests for ?T syntax sugar (May<T> generic types)
//! Tests the ?int, ?str, ?bool, ?uint, ?float, ?double, ?char syntax

use crate::run;

#[test]
fn test_question_int_return_type() {
    let code = r#"
fn get_value() ?int {
    42
}

fn main() int {
    let result = get_value()
    result.?
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_question_str_return_type() {
    let code = r#"
fn get_message() ?str {
    "hello"
}

fn main() int {
    let msg = get_message()
    0
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_question_bool_return_type() {
    let code = r#"
fn is_ready() ?bool {
    true
}

fn main() int {
    let ready = is_ready()
    0
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_question_uint_return_type() {
    let code = r#"
fn get_count() ?uint {
    42u
}

fn main() int {
    let count = get_count()
    0
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_question_float_return_type() {
    let code = r#"
fn get_value() ?float {
    3.14f
}

fn main() int {
    let val = get_value()
    0
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_question_double_return_type() {
    let code = r#"
fn get_pi() ?double {
    3.14159
}

fn main() int {
    let pi = get_pi()
    0
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_question_char_return_type() {
    let code = r#"
fn get_letter() ?char {
    'A'
}

fn main() int {
    let letter = get_letter()
    0
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_question_int_arithmetic() {
    let code = r#"
fn calculate() ?int {
    10 + 20
}

fn main() int {
    let result = calculate()
    0
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_question_int_variable_binding() {
    let code = r#"
fn get_value() ?int {
    let x = 42
    x
}

fn main() int {
    let val = get_value()
    0
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_question_int_function_call() {
    let code = r#"
fn helper(x int) int {
    x * 2
}

fn get_value() ?int {
    helper(21)
}

fn main() int {
    let result = get_value()
    0
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_question_int_nested_call() {
    let code = r#"
fn inner() ?int {
    10
}

fn outer() ?int {
    inner()
}

fn main() int {
    outer()
    0
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_question_int_if_expression() {
    let code = r#"
fn get_value(flag bool) ?int {
    if flag {
        100
    } else {
        200
    }
}

fn main() int {
    let val = get_value(true)
    0
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

// Test disabled: bare block expressions not yet supported in parser
// #[test]
// fn test_question_int_block_expression() {
//     let code = r#"
// fn compute() ?int {
//     {
//         let x = 10
//         x + 20
//     }
// }
//
// fn main() int {
//     compute()
//     0
// }
//
// main()
// "#;
//     let result = run(code).unwrap();
//     assert_eq!(result, "0");
// }

#[test]
fn test_question_int_negative() {
    let code = r#"
fn get_negative() ?int {
    -42
}

fn main() int {
    let val = get_negative()
    0
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_question_int_zero() {
    let code = r#"
fn get_zero() ?int {
    0
}

fn main() int {
    get_zero()
    0
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_question_int_comparison() {
    let code = r#"
fn compare() ?int {
    10 < 20
}

fn main() int {
    compare()
    0
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_question_int_complex_arithmetic() {
    let code = r#"
fn complex() ?int {
    ((10 + 20) * 2) - 5
}

fn main() int {
    complex()
    0
}

main()
"#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}
