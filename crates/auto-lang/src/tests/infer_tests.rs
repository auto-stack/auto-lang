use crate::run_autovm;

// ===== .type Property Tests =====

#[test]
fn test_infer_int() {
    let code = "42.type";
    let result = run_autovm(code);
    assert_eq!(result.unwrap(), "int", "`42.type` Should return int");
}

#[test]
fn test_type_literal_int() {
    // Test: 1.type should return "int"
    let code = r#"
fn main() str {
    1.type
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "int.type should work: {:?}", result);
    assert_eq!(result.unwrap(), "int", "Should return 'int'");
}

#[test]
fn test_type_literal_float() {
    // Test: 1.5.type should return "float"
    let code = r#"
fn main() str {
    1.5.type
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "float.type should work: {:?}", result);
    assert_eq!(result.unwrap(), "float", "Should return 'float'");
}

#[test]
fn test_type_literal_str() {
    // Test: "hello".type should return "str"
    let code = r#"
fn main() str {
    "hello".type
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "str.type should work: {:?}", result);
    assert_eq!(result.unwrap(), "str", "Should return 'str'");
}

#[test]
fn test_type_literal_bool() {
    // Test: true.type should return "bool"
    let code = r#"
fn main() str {
    true.type
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "bool.type should work: {:?}", result);
    assert_eq!(result.unwrap(), "bool", "Should return 'bool'");
}

// Note: Chained dot access like { x: 1 }.x.type is not yet supported by parser
// TODO: Add test_type_object_field_int and test_type_object_field_str when parser supports it

// ===== Variable .type Tests =====

#[test]
fn test_type_variable_int() {
    // Test: let x = 42; x.type should return "int"
    let code = r#"
fn main() str {
    let x = 42
    x.type
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "variable int.type should work: {:?}", result);
    assert_eq!(result.unwrap(), "int", "Should return 'int'");
}

#[test]
fn test_type_variable_float() {
    // Test: let f = 1.5; f.type should return "float"
    let code = r#"
fn main() str {
    let f = 1.5
    f.type
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "variable float.type should work: {:?}", result);
    assert_eq!(result.unwrap(), "float", "Should return 'float'");
}

#[test]
fn test_type_variable_str() {
    // Test: let name = "hello"; name.type should return "str"
    let code = r#"
fn main() str {
    let name = "hello"
    name.type
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "variable str.type should work: {:?}", result);
    assert_eq!(result.unwrap(), "str", "Should return 'str'");
}

#[test]
fn test_type_variable_bool() {
    // Test: let flag = true; flag.type should return "bool"
    let code = r#"
fn main() str {
    let flag = true
    flag.type
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "variable bool.type should work: {:?}", result);
    assert_eq!(result.unwrap(), "bool", "Should return 'bool'");
}

// ===== Function .type Tests =====

#[test]
fn test_type_function_return_int() {
    // Test: function return value type
    let code = r#"
fn get_value() int {
    42
}

fn main() str {
    let x = get_value()
    x.type
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "function return .type should work: {:?}", result);
    assert_eq!(result.unwrap(), "int", "Should return 'int'");
}

#[test]
fn test_type_function_return_str() {
    // Test: function returning string
    let code = r#"
fn get_name() str {
    "hello"
}

fn main() str {
    let name = get_name()
    name.type
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "function return str .type should work: {:?}", result);
    assert_eq!(result.unwrap(), "str", "Should return 'str'");
}

#[test]
fn test_type_function_parameter() {
    // Test: function parameter type
    let code = r#"
fn identity(x int) int {
    x
}

fn main() str {
    let result = identity(42)
    result.type
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "function param .type should work: {:?}", result);
    assert_eq!(result.unwrap(), "int", "Should return 'int'");
}

// ===== Array .type Tests =====

#[test]
fn test_type_array_element() {
    // Test: array element type
    let code = r#"
fn main() str {
    let arr = [1, 2, 3]
    arr[0].type
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "array element .type should work: {:?}", result);
    assert_eq!(result.unwrap(), "int", "Should return 'int'");
}

#[test]
fn test_type_array_element_str() {
    // Test: string array element type
    let code = r#"
fn main() str {
    let arr = ["a", "b", "c"]
    arr[1].type
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "string array element .type should work: {:?}", result);
    assert_eq!(result.unwrap(), "str", "Should return 'str'");
}

// ===== Object/Node .type Tests =====

// Note: These tests require parser support for chained dot access
// TODO: Add when parser supports { x: 1 }.x.type

// ===== Binary Expression .type Tests =====

#[test]
fn test_type_binary_add() {
    // Test: binary operation result type
    let code = r#"
fn main() str {
    let sum = 1 + 2
    sum.type
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "binary add .type should work: {:?}", result);
    assert_eq!(result.unwrap(), "int", "Should return 'int'");
}

#[test]
fn test_type_binary_multiply() {
    // Test: binary multiplication result type
    let code = r#"
fn main() str {
    let product = 3 * 4
    product.type
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "binary multiply .type should work: {:?}", result);
    assert_eq!(result.unwrap(), "int", "Should return 'int'");
}
