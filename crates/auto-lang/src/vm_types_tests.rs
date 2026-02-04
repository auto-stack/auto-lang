// Plan 073 Stage B: BigVM Type System Integration Tests
//
// Comprehensive tests for float, double, i64, u64 types in BigVM
// These tests verify that code with these types compiles correctly to bytecode

use crate::vm::codegen::Codegen;
use crate::Parser;

/// Helper function to compile AutoLang code to BigVM bytecode
fn compile_to_bytecode(source: &str) -> Vec<u8> {
    // Parse the source code
    let mut parser = Parser::from(source);
    let code = parser.parse().expect("Parse failed");

    // Compile to bytecode
    let mut codegen = Codegen::new();
    for stmt in code.stmts {
        codegen.compile_stmt(&stmt).expect("Codegen failed");
    }

    codegen.code
}

/// Plan 073: Helper function to compile with object_keys metadata
fn compile_with_object_keys(source: &str) -> (Vec<u8>, Vec<Vec<auto_val::ValueKey>>) {
    let mut parser = Parser::from(source);
    let code = parser.parse().expect("Parse failed");

    let mut codegen = Codegen::new();
    for stmt in code.stmts {
        codegen.compile_stmt(&stmt).expect("Codegen failed");
    }

    (codegen.code, codegen.object_keys)
}


#[test]
fn test_float_literal_compiles() {
    let source = r#"
fn main() -> int {
    let x float = 3.14
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CONST_F32 opcode
    assert!(bytecode.contains(&0x14), "Expected CONST_F32 opcode (0x14)");
}

#[test]
fn test_double_literal_compiles() {
    let source = r#"
fn main() -> int {
    let x double = 2.718281828d
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CONST_F64 opcode
    assert!(bytecode.contains(&0x15), "Expected CONST_F64 opcode (0x15)");
}

#[test]
fn test_float_addition_compiles() {
    let source = r#"
fn main() -> int {
    let result float = 1.5 + 2.5
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain ADD_F opcode
    assert!(bytecode.contains(&0x36), "Expected ADD_F opcode (0x36)");
}

#[test]
fn test_double_addition_compiles() {
    let source = r#"
fn main() -> int {
    let result double = 1.5d + 2.5d
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain ADD_D opcode
    assert!(bytecode.contains(&0x3B), "Expected ADD_D opcode (0x3B)");
}

#[test]
fn test_float_subtraction_compiles() {
    let source = r#"
fn main() -> int {
    let result float = 5.0 - 3.0
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain SUB_F opcode
    assert!(bytecode.contains(&0x37), "Expected SUB_F opcode (0x37)");
}

#[test]
fn test_double_subtraction_compiles() {
    let source = r#"
fn main() -> int {
    let result double = 5.0d - 3.0d
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain SUB_D opcode
    assert!(bytecode.contains(&0x3C), "Expected SUB_D opcode (0x3C)");
}

#[test]
fn test_float_multiplication_compiles() {
    let source = r#"
fn main() -> int {
    let result float = 2.5 * 4.0
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain MUL_F opcode
    assert!(bytecode.contains(&0x38), "Expected MUL_F opcode (0x38)");
}

#[test]
fn test_double_multiplication_compiles() {
    let source = r#"
fn main() -> int {
    let result double = 2.5d * 4.0d
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain MUL_D opcode
    assert!(bytecode.contains(&0x3D), "Expected MUL_D opcode (0x3D)");
}

#[test]
fn test_float_division_compiles() {
    let source = r#"
fn main() -> int {
    let result float = 10.0 / 2.0
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain DIV_F opcode
    assert!(bytecode.contains(&0x39), "Expected DIV_F opcode (0x39)");
}

#[test]
fn test_double_division_compiles() {
    let source = r#"
fn main() -> int {
    let result double = 10.0d / 2.0d
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain DIV_D opcode
    assert!(bytecode.contains(&0x3E), "Expected DIV_D opcode (0x3E)");
}

#[test]
fn test_float_negation_compiles() {
    let source = r#"
fn main() -> int {
    let result float = -3.14
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain NEG_F opcode
    assert!(bytecode.contains(&0x3A), "Expected NEG_F opcode (0x3A)");
}

#[test]
fn test_double_negation_compiles() {
    let source = r#"
fn main() -> int {
    let result double = -2.718d
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain NEG_D opcode
    assert!(bytecode.contains(&0x3F), "Expected NEG_D opcode (0x3F)");
}

#[test]
fn test_i64_literal_compiles() {
    let source = r#"
fn main() -> int {
    let x i64 = 9223372036854775807
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CONST_I64 opcode
    assert!(bytecode.contains(&0x16), "Expected CONST_I64 opcode (0x16)");
}

#[test]
fn test_u64_literal_compiles() {
    let source = r#"
fn main() -> int {
    let x u64 = 18446744073709551615
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CONST_U64 opcode
    assert!(bytecode.contains(&0x17), "Expected CONST_U64 opcode (0x17)");
}

#[test]
fn test_mixed_float_double_uses_double_compiles() {
    let source = r#"
fn main() -> int {
    // Mixing float and double should promote to double
    let result double = 3.14 + 2.718d
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain ADD_D opcode (promotes to double)
    assert!(bytecode.contains(&0x3B), "Expected ADD_D opcode (0x3B)");
}

#[test]
fn test_float_function_return_compiles() {
    let source = r#"
fn get_pi() -> float {
    3.14
}

fn main() -> int {
    let pi float = get_pi()
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CONST_F32 opcode
    assert!(bytecode.contains(&0x14), "Expected CONST_F32 opcode (0x14)");
}

#[test]
fn test_double_function_return_compiles() {
    let source = r#"
fn get_e() -> double {
    2.718281828d
}

fn main() -> int {
    let e double = get_e()
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CONST_F64 opcode
    assert!(bytecode.contains(&0x15), "Expected CONST_F64 opcode (0x15)");
}

#[test]
fn test_complex_float_expression_compiles() {
    let source = r#"
fn main() -> int {
    let result float = (1.5 + 2.5) * 3.0 - 1.0
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain float opcodes
    assert!(bytecode.contains(&0x36), "Expected ADD_F opcode (0x36)");
    assert!(bytecode.contains(&0x38), "Expected MUL_F opcode (0x38)");
}

#[test]
fn test_complex_double_expression_compiles() {
    let source = r#"
fn main() -> int {
    let result double = (1.5d + 2.5d) * 3.0d - 1.0d
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain double opcodes
    assert!(bytecode.contains(&0x3B), "Expected ADD_D opcode (0x3B)");
    assert!(bytecode.contains(&0x3D), "Expected MUL_D opcode (0x3D)");
}

// ============================================================================
// Plan 073 Stage B: Additional Type System Tests (uint, i8, u8, byte, char, cstr)
// ============================================================================

#[test]
fn test_uint_literal_compiles() {
    let source = r#"
fn main() -> int {
    let x uint = 42u
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CONST_I32 opcode (uint uses CONST_I32)
    assert!(bytecode.contains(&0x10), "Expected CONST_I32 opcode (0x10)");
}

#[test]
fn test_i8_literal_compiles() {
    let source = r#"
fn main() -> int {
    let x i8 = -127
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CONST_I32 opcode (i8 uses CONST_I32)
    assert!(bytecode.contains(&0x10), "Expected CONST_I32 opcode (0x10)");
}

#[test]
fn test_u8_literal_compiles() {
    let source = r#"
fn main() -> int {
    let x u8 = 255u8
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CONST_I32 opcode (u8 uses CONST_I32)
    assert!(bytecode.contains(&0x10), "Expected CONST_I32 opcode (0x10)");
}

#[test]
fn test_byte_literal_compiles() {
    let source = r#"
fn main() -> int {
    let x byte = 0xAB
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CONST_I32 opcode (byte uses CONST_I32)
    assert!(bytecode.contains(&0x10), "Expected CONST_I32 opcode (0x10)");
}

#[test]
fn test_char_literal_compiles() {
    let source = r#"
fn main() -> int {
    let c char = 'A'
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CONST_I32 opcode (char uses CONST_I32 for UTF-32 codepoint)
    assert!(bytecode.contains(&0x10), "Expected CONST_I32 opcode (0x10)");
}

#[test]
fn test_cstr_literal_compiles() {
    let source = r#"
fn main() -> int {
    let s cstr = c"hello"
    0
}
"#;
    // This test just verifies that cstr literals can be parsed and compiled
    // The actual bytecode generation for CStr uses LOAD_STR like regular strings
    let bytecode = compile_to_bytecode(source);
    // Should contain at least some bytecode
    assert!(!bytecode.is_empty(), "Expected non-empty bytecode");
    // CStr is stored in the strings pool
    // Note: c"hello" is parsed as a CStr token by the lexer
}

#[test]
fn test_all_small_int_types_compiled() {
    let source = r#"
fn test_all_types() -> int {
    let a uint = 100u
    let b i8 = -50
    let c u8 = 200u8
    let d byte = 0xFF
    let e char = 'Z'
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain multiple CONST_I32 opcodes
    let const_i32_count = bytecode.iter().filter(|&&x| x == 0x10).count();
    assert!(const_i32_count >= 5, "Expected at least 5 CONST_I32 opcodes, got {}", const_i32_count);
}

#[test]
fn test_mixed_types_compiles() {
    let source = r#"
fn test_mixed() -> int {
    let i int = 42
    let u uint = 42u
    let f float = 3.14
    let d double = 2.718d
    let c char = 'A'
    let s cstr = c"test"
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CONST_I32 (for int, uint, char)
    assert!(bytecode.contains(&0x10), "Expected CONST_I32 opcode (0x10)");
    // Should contain CONST_F32 (for float)
    assert!(bytecode.contains(&0x14), "Expected CONST_F32 opcode (0x14)");
    // Should contain CONST_F64 (for double)
    assert!(bytecode.contains(&0x15), "Expected CONST_F64 opcode (0x15)");
    // CStr uses LOAD_STR like regular strings, but we don't check for the opcode
    // since c"..." literals might need special handling in the lexer
}

#[test]
fn test_char_unicode_compiles() {
    let source = r#"
fn main() -> int {
    let c char = 'π'  // Unicode character
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CONST_I32 opcode
    assert!(bytecode.contains(&0x10), "Expected CONST_I32 opcode (0x10)");
}

// ============================================================================
// Plan 073: Object Literal Tests
// ============================================================================

#[test]
fn test_empty_object_compiles() {
    let source = r#"
fn main() -> int {
    let obj = {}
    0
}
"#;
    let (bytecode, object_keys) = compile_with_object_keys(source);
    // Should contain CREATE_OBJ opcode
    assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");
    // Should have one object with 0 fields
    assert_eq!(object_keys.len(), 1, "Expected 1 object");
    assert_eq!(object_keys[0].len(), 0, "Expected 0 fields");
}

#[test]
fn test_simple_object_compiles() {
    let source = r#"
fn main() -> int {
    let obj = {x: 1, y: 2}
    0
}
"#;
    let (bytecode, object_keys) = compile_with_object_keys(source);
    // Should contain CREATE_OBJ opcode
    assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");
    // Should have one object with 2 fields
    assert_eq!(object_keys.len(), 1, "Expected 1 object");
    assert_eq!(object_keys[0].len(), 2, "Expected 2 fields");
    // Should have CONST_I32 for the two integer values
    let const_i32_count = bytecode.iter().filter(|&&x| x == 0x10).count();
    assert!(const_i32_count >= 2, "Expected at least 2 CONST_I32 opcodes");
}

#[test]
fn test_nested_object_compiles() {
    let source = r#"
fn main() -> int {
    let outer = {name: "test", inner: {x: 10}}
    0
}
"#;
    let (bytecode, object_keys) = compile_with_object_keys(source);
    // Should contain CREATE_OBJ opcode (at least 2 for nested objects)
    let create_obj_count = bytecode.iter().filter(|&&x| x == 0x2E).count();
    assert!(create_obj_count >= 2, "Expected at least 2 CREATE_OBJ opcodes");
    // Should have 2 objects
    assert_eq!(object_keys.len(), 2, "Expected 2 objects");
}

#[test]
fn test_object_field_access_compiles() {
    let source = r#"
fn main() -> int {
    let obj = {x: 1, y: 2}
    let val = obj.x
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CREATE_OBJ opcode
    assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");
    // Should contain GET_FIELD opcode
    assert!(bytecode.contains(&0x2F), "Expected GET_FIELD opcode (0x2F)");
}

#[test]
fn test_chained_field_access_compiles() {
    let source = r#"
fn main() -> int {
    let outer = {inner: {value: 42}}
    let val = outer.inner.value
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CREATE_OBJ opcodes (2 objects)
    let create_obj_count = bytecode.iter().filter(|&&x| x == 0x2E).count();
    assert!(create_obj_count >= 2, "Expected at least 2 CREATE_OBJ opcodes");
    // Should contain GET_FIELD opcodes (2 field accesses)
    let get_field_count = bytecode.iter().filter(|&&x| x == 0x2F).count();
    assert!(get_field_count >= 2, "Expected at least 2 GET_FIELD opcodes");
}


