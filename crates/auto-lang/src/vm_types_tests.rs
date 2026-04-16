// Plan 073 Stage B: AutoVM Type System Integration Tests
//
// Comprehensive tests for float, double, i64, u64 types in AutoVM
// These tests verify that code with these types compiles correctly to bytecode

use crate::vm::codegen::Codegen;
use crate::Parser;

/// Helper function to compile AutoLang code to AutoVM bytecode
fn compile_to_bytecode(source: &str) -> Vec<u8> {
    // Parse the source code
    let mut parser = Parser::from(source);
    let code = parser.parse().expect("Parse failed");

    // Compile to bytecode - share TypeStore with Parser for enum/type lookup
    let mut codegen = Codegen::new_with_type_store(parser.type_store.clone());
    for stmt in code.stmts {
        codegen.compile_stmt(&stmt).expect("Codegen failed");
    }

    codegen.code
}

/// Plan 073: Helper function to compile with object_keys metadata
fn compile_with_object_keys(source: &str) -> (Vec<u8>, Vec<Vec<auto_val::ValueKey>>, Vec<Vec<crate::vm::codegen::ObjectType>>) {
    let mut parser = Parser::from(source);
    let code = parser.parse().expect("Parse failed");

    // Share TypeStore with Parser for enum/type lookup
    let mut codegen = Codegen::new_with_type_store(parser.type_store.clone());
    for stmt in code.stmts {
        codegen.compile_stmt(&stmt).expect("Codegen failed");
    }

    (codegen.code, codegen.object_keys, codegen.object_types)
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
    let (bytecode, object_keys, object_types) = compile_with_object_keys(source);
    // Should contain CREATE_OBJ opcode
    assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");
    // Should have one object with 0 fields
    assert_eq!(object_keys.len(), 1, "Expected 1 object");
    assert_eq!(object_keys[0].len(), 0, "Expected 0 fields");
    assert_eq!(object_types[0].len(), 0, "Expected 0 field types");
}

#[test]
fn test_simple_object_compiles() {
    let source = r#"
fn main() -> int {
    let obj = {x: 1, y: 2}
    0
}
"#;
    let (bytecode, object_keys, object_types) = compile_with_object_keys(source);
    // Should contain CREATE_OBJ opcode
    assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");
    // Should have one object with 2 fields
    assert_eq!(object_keys.len(), 1, "Expected 1 object");
    assert_eq!(object_keys[0].len(), 2, "Expected 2 fields");
    assert_eq!(object_types[0].len(), 2, "Expected 2 field types");
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
    let (bytecode, object_keys, object_types) = compile_with_object_keys(source);
    // Should contain CREATE_OBJ opcode (at least 2 for nested objects)
    let create_obj_count = bytecode.iter().filter(|&&x| x == 0x2E).count();
    assert!(create_obj_count >= 2, "Expected at least 2 CREATE_OBJ opcodes");
    // Should have 2 objects
    assert_eq!(object_keys.len(), 2, "Expected 2 objects");
    // Should have 2 type lists
    assert_eq!(object_types.len(), 2, "Expected 2 type lists");
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
    assert!(bytecode.contains(&0x2D), "Expected GET_FIELD opcode (0x2D)");
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
    let get_field_count = bytecode.iter().filter(|&&x| x == 0x2D).count();
    assert!(get_field_count >= 2, "Expected at least 2 GET_FIELD opcodes");
}

// ============================================================================
// Plan 073: Object Field Type Tests
// ============================================================================

#[test]
fn test_object_with_float_field_compiles() {
    let source = r#"
fn main() -> int {
    let obj = {x: 1.5, y: 2.5}
    0
}
"#;
    let (bytecode, _object_keys, object_types) = compile_with_object_keys(source);
    // Should contain CREATE_OBJ opcode
    assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");
    // Should have Float field types
    assert_eq!(object_types[0][0], crate::vm::codegen::ObjectType::Float);
    assert_eq!(object_types[0][1], crate::vm::codegen::ObjectType::Float);
}

#[test]
fn test_object_with_double_field_compiles() {
    let source = r#"
fn main() -> int {
    let obj = {pi: 3.14159d, e: 2.71828d}
    0
}
"#;
    let (bytecode, _object_keys, object_types) = compile_with_object_keys(source);
    // Should contain CREATE_OBJ opcode
    assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");
    // Should have Double field types
    assert_eq!(object_types[0][0], crate::vm::codegen::ObjectType::Double);
    assert_eq!(object_types[0][1], crate::vm::codegen::ObjectType::Double);
}

#[test]
fn test_object_with_string_field_compiles() {
    let source = r#"
fn main() -> int {
    let obj = {name: "Alice", city: "Boston"}
    0
}
"#;
    let (bytecode, _object_keys, object_types) = compile_with_object_keys(source);
    // Should contain CREATE_OBJ opcode
    assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");
    // Should have String field types
    assert_eq!(object_types[0][0], crate::vm::codegen::ObjectType::String);
    assert_eq!(object_types[0][1], crate::vm::codegen::ObjectType::String);
}

#[test]
fn test_object_with_bool_field_compiles() {
    let source = r#"
fn main() -> int {
    let obj = {active: true, verified: false}
    0
}
"#;
    let (bytecode, _object_keys, object_types) = compile_with_object_keys(source);
    // Should contain CREATE_OBJ opcode
    assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");
    // Should have Bool field types
    assert_eq!(object_types[0][0], crate::vm::codegen::ObjectType::Bool);
    assert_eq!(object_types[0][1], crate::vm::codegen::ObjectType::Bool);
}

#[test]
fn test_object_with_char_field_compiles() {
    let source = r#"
fn main() -> int {
    let obj = {initial: 'A', grade: 'B'}
    0
}
"#;
    let (bytecode, _object_keys, object_types) = compile_with_object_keys(source);
    // Should contain CREATE_OBJ opcode
    assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");
    // Should have Char field types
    assert_eq!(object_types[0][0], crate::vm::codegen::ObjectType::Char);
    assert_eq!(object_types[0][1], crate::vm::codegen::ObjectType::Char);
}

#[test]
fn test_object_with_mixed_types_compiles() {
    let source = r#"
fn main() -> int {
    let obj = {
        name: "test",
        count: 42,
        price: 9.99,
        active: true
    }
    0
}
"#;
    let (bytecode, _object_keys, object_types) = compile_with_object_keys(source);
    // Should contain CREATE_OBJ opcode
    assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");
    // Should have mixed field types
    assert_eq!(object_types[0][0], crate::vm::codegen::ObjectType::String);
    assert_eq!(object_types[0][1], crate::vm::codegen::ObjectType::Int);
    assert_eq!(object_types[0][2], crate::vm::codegen::ObjectType::Float);
    assert_eq!(object_types[0][3], crate::vm::codegen::ObjectType::Bool);
}

// ============================================================================
// Plan 073: Nested Object and Array Field Tests
// ============================================================================

#[test]
fn test_object_with_nested_object_compiles() {
    let source = r#"
fn main() -> int {
    let obj = {
        name: "test",
        nested: {x: 1, y: 2}
    }
    0
}
"#;
    let (bytecode, object_keys, object_types) = compile_with_object_keys(source);
    // Should contain CREATE_OBJ opcode
    assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");
    // Should have 2 objects total (nested + outer)
    assert_eq!(object_keys.len(), 2, "Expected 2 objects");
    // Nested object is created first (index 0), outer object second (index 1)
    // Nested object should have 2 Int fields
    assert_eq!(object_types[0].len(), 2, "Expected 2 fields in nested object");
    assert_eq!(object_types[0][0], crate::vm::codegen::ObjectType::Int);
    assert_eq!(object_types[0][1], crate::vm::codegen::ObjectType::Int);
    // Outer object should have String and NestedObject fields
    assert_eq!(object_types[1].len(), 2, "Expected 2 fields in outer object");
    assert_eq!(object_types[1][0], crate::vm::codegen::ObjectType::String);
    assert_eq!(object_types[1][1], crate::vm::codegen::ObjectType::NestedObject);
}

#[test]
fn test_simple_array_compiles() {
    let source = r#"
fn main() -> int {
    let arr = [1, 2, 3]
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CREATE_ARRAY opcode
    assert!(bytecode.contains(&0x2F), "Expected CREATE_ARRAY opcode (0x2F)");
}

#[test]
fn test_array_indexing_compiles() {
    let source = r#"
fn main() -> int {
    let arr = [1, 2, 3]
    let val = arr[0]
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CREATE_ARRAY opcode
    assert!(bytecode.contains(&0x2F), "Expected CREATE_ARRAY opcode (0x2F)");
    // Should contain GET_ELEM opcode
    assert!(bytecode.contains(&0x2C), "Expected GET_ELEM opcode (0x2C)");
}

#[test]
fn test_object_with_array_field_compiles() {
    let source = r#"
fn main() -> int {
    let obj = {
        name: "test",
        items: [1, 2, 3]
    }
    0
}
"#;
    let (bytecode, _object_keys, object_types) = compile_with_object_keys(source);
    // Should contain CREATE_OBJ opcode
    assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");
    // Should have String and Array field types
    assert_eq!(object_types[0].len(), 2, "Expected 2 fields");
    assert_eq!(object_types[0][0], crate::vm::codegen::ObjectType::String);
    assert_eq!(object_types[0][1], crate::vm::codegen::ObjectType::Array);
}

#[test]
fn test_deeply_nested_objects_compiles() {
    let source = r#"
fn main() -> int {
    let obj = {
        level1: {
            level2: {
                value: 42
            }
        }
    }
    0
}
"#;
    let (bytecode, object_keys, object_types) = compile_with_object_keys(source);
    // Should contain 3 CREATE_OBJ opcodes
    let create_obj_count = bytecode.iter().filter(|&&x| x == 0x2E).count();
    assert!(create_obj_count >= 3, "Expected at least 3 CREATE_OBJ opcodes");
    // Should have 3 objects total
    assert_eq!(object_keys.len(), 3, "Expected 3 objects");
    // Innermost object created first (index 0), then middle (index 1), then outer (index 2)
    assert_eq!(object_types[0].len(), 1, "Expected 1 field in innermost object");
    assert_eq!(object_types[0][0], crate::vm::codegen::ObjectType::Int);
    assert_eq!(object_types[1].len(), 1, "Expected 1 field in middle object");
    assert_eq!(object_types[1][0], crate::vm::codegen::ObjectType::NestedObject);
    assert_eq!(object_types[2].len(), 1, "Expected 1 field in outer object");
    assert_eq!(object_types[2][0], crate::vm::codegen::ObjectType::NestedObject);
}

#[test]
fn test_array_index_assignment_compiles() {
    let source = r#"
fn main() -> int {
    let arr = [1, 2, 3]
    arr[0] = 10
    arr[1] = 20
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CREATE_ARRAY opcode
    assert!(bytecode.contains(&0x2F), "Expected CREATE_ARRAY opcode (0x2F)");
    // Should contain SET_ELEM opcode (Plan 073)
    assert!(bytecode.contains(&0x2B), "Expected SET_ELEM opcode (0x2B)");
}

#[test]
fn test_array_index_read_and_write_compiles() {
    let source = r#"
fn main() -> int {
    let arr = [1, 2, 3]
    let val = arr[0]
    arr[1] = val + 5
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain both GET_ELEM and SET_ELEM opcodes
    assert!(bytecode.contains(&0x2C), "Expected GET_ELEM opcode (0x2C)");
    assert!(bytecode.contains(&0x2B), "Expected SET_ELEM opcode (0x2B)");
}

// Plan 073: For loop tests

#[test]
fn test_for_loop_range_compiles() {
    let source = r#"
fn main() -> int {
    let sum = 0
    for x in 0..10 {
        // Loop body
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain LT (for range comparison)
    assert!(bytecode.contains(&0x52), "Expected LT opcode (0x52) for range check");
    // Should contain JMP (for loop control)
    assert!(bytecode.contains(&0x60), "Expected JMP opcode (0x60) for loop");
}

#[test]
fn test_for_loop_inclusive_range_compiles() {
    let source = r#"
fn main() -> int {
    for x in 0..=10 {
        // Loop body
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain LE (for inclusive range comparison)
    assert!(bytecode.contains(&0x54), "Expected LE opcode (0x54) for inclusive range");
}

#[test]
fn test_for_loop_conditional_compiles() {
    let source = r#"
fn main() -> int {
    var i = 0
    for i < 10 {
        i = i + 1
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain JMP_IF_Z (for condition check)
    assert!(bytecode.contains(&0x61), "Expected JMP_IF_Z opcode (0x61) for condition");
}

#[test]
#[ignore = "for ever syntax not fully implemented in codegen"]
fn test_for_loop_infinite_compiles() {
    let source = r#"
fn main() -> int {
    for ever {
        // Infinite loop body (will need break statement support in future)
        let x = 1
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain JMP (for infinite loop)
    assert!(bytecode.contains(&0x60), "Expected JMP opcode (0x60) for infinite loop");
}

#[test]
fn test_for_loop_with_array_compiles() {
    let source = r#"
fn main() -> int {
    let arr = [1, 2, 3]
    for i in 0..3 {
        let val = arr[i]
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain both for loop opcodes and array indexing
    assert!(bytecode.contains(&0x52), "Expected LT opcode (0x52)");
    assert!(bytecode.contains(&0x2C), "Expected GET_ELEM opcode (0x2C)");
}

// Plan 073: Break statement tests

#[test]
fn test_break_in_range_loop_compiles() {
    let source = r#"
fn main() -> int {
    for x in 0..10 {
        break
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain JMP for break statement
    let jmp_count = bytecode.iter().filter(|&&b| b == 0x60).count();
    assert!(jmp_count >= 2, "Expected at least 2 JMP opcodes (0x60) - one for loop, one for break");
}

#[test]
fn test_break_in_conditional_loop_compiles() {
    let source = r#"
fn main() -> int {
    var i = 0
    for i < 10 {
        i = i + 1
        if i > 5 {
            break
        }
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain JMP_IF_Z for condition and JMP for break
    assert!(bytecode.contains(&0x61), "Expected JMP_IF_Z opcode (0x61) for condition");
    assert!(bytecode.contains(&0x60), "Expected JMP opcode (0x60) for break");
}

#[test]
fn test_break_in_infinite_loop_compiles() {
    let source = r#"
fn main() -> int {
    for {
        break
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain JMP for both loop and break
    let jmp_count = bytecode.iter().filter(|&&b| b == 0x60).count();
    assert!(jmp_count >= 2, "Expected at least 2 JMP opcodes (0x60)");
}

#[test]
fn test_nested_loops_with_break_compiles() {
    let source = r#"
fn main() -> int {
    for x in 0..10 {
        for y in 0..10 {
            break  // Breaks inner loop
        }
        break  // Breaks outer loop
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should have multiple JMPs for loops and breaks
    let jmp_count = bytecode.iter().filter(|&&b| b == 0x60).count();
    assert!(jmp_count >= 4, "Expected at least 4 JMP opcodes (0x60) for nested loops and breaks");
}

#[test]
fn test_break_with_array_compiles() {
    let source = r#"
fn main() -> int {
    let arr = [1, 2, 3, 4, 5]
    for i in 0..5 {
        let val = arr[i]
        if val == 3 {
            break
        }
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain loop opcodes, array indexing, and break
    assert!(bytecode.contains(&0x52), "Expected LT opcode (0x52)");
    assert!(bytecode.contains(&0x2C), "Expected GET_ELEM opcode (0x2C)");
    assert!(bytecode.contains(&0x60), "Expected JMP opcode (0x60) for break");
}

// Plan 073: Indexed iteration tests

#[test]
fn test_indexed_iteration_range_compiles() {
    let source = r#"
fn main() -> int {
    for i, x in 0..10 {
        // i and x both go from 0 to 9
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain LT (for range comparison) and multiple STORE_LOC
    assert!(bytecode.contains(&0x52), "Expected LT opcode (0x52)");

    // Count the number of STORE_LOCAL opcodes (should be at least 2 for i and x initialization)
    let store_count = bytecode.iter().filter(|&&b| b == 0x21 || b == 0x25 || b == 0x26).count();
    assert!(store_count >= 2, "Expected at least 2 STORE opcodes for i and x variables");
}

#[test]
fn test_indexed_iteration_inclusive_range_compiles() {
    let source = r#"
fn main() -> int {
    for i, val in 0..=5 {
        // i and val both go from 0 to 5
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain LE (for inclusive range comparison)
    assert!(bytecode.contains(&0x54), "Expected LE opcode (0x54) for inclusive range");
}

#[test]
fn test_indexed_iteration_with_operations_compiles() {
    let source = r#"
fn main() -> int {
    let sum = 0
    for i, x in 0..10 {
        // Use both index and value
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain loop control opcodes
    assert!(bytecode.contains(&0x52), "Expected LT opcode (0x52)");
    assert!(bytecode.contains(&0x60), "Expected JMP opcode (0x60)");
}

#[test]
fn test_indexed_iteration_with_break_compiles() {
    let source = r#"
fn main() -> int {
    for i, x in 0..10 {
        if i == 5 {
            break
        }
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain loop control and break opcodes
    assert!(bytecode.contains(&0x52), "Expected LT opcode (0x52)");
    assert!(bytecode.contains(&0x61), "Expected JMP_IF_Z opcode (0x61)");
    // Multiple JMPs: one for loop, one for break
    let jmp_count = bytecode.iter().filter(|&&b| b == 0x60).count();
    assert!(jmp_count >= 2, "Expected at least 2 JMP opcodes");
}

#[test]
fn test_indexed_iteration_nested_compiles() {
    let source = r#"
fn main() -> int {
    for i, x in 0..5 {
        for j, y in 0..3 {
            // Nested indexed iteration
        }
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain multiple loop structures
    let lt_count = bytecode.iter().filter(|&&b| b == 0x52).count();
    assert!(lt_count >= 2, "Expected at least 2 LT opcodes for nested loops");
}

// ============================================================================
// Plan 073: Iterator-based For Loop Tests
// ============================================================================

#[test]
fn test_iterator_for_loop_basic_compiles() {
    let source = r#"
fn main() -> int {
    let list = List.new()
    list.push(1)
    list.push(2)
    list.push(3)
    for x in list.iter() {
        // Iterate over list elements
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CALL_NAT opcode for Iterator.next
    assert!(bytecode.contains(&0x72), "Expected CALL_NAT opcode (0x72)");
    // Should contain loop control opcodes
    assert!(bytecode.contains(&0x60), "Expected JMP opcode (0x60)");
    assert!(bytecode.contains(&0x61), "Expected JMP_IF_Z opcode (0x61)");
}

#[test]
fn test_iterator_for_loop_with_break_compiles() {
    let source = r#"
fn main() -> int {
    let list = List.new()
    list.push(1)
    list.push(2)
    list.push(3)
    for x in list.iter() {
        if x == 2 {
            break
        }
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CALL_NAT for Iterator.next
    assert!(bytecode.contains(&0x72), "Expected CALL_NAT opcode (0x72)");
    // Should contain break opcodes
    assert!(bytecode.contains(&0x61), "Expected JMP_IF_Z opcode (0x61)");
    // Multiple JMPs: one for loop, one for break
    let jmp_count = bytecode.iter().filter(|&&b| b == 0x60).count();
    assert!(jmp_count >= 2, "Expected at least 2 JMP opcodes");
}

#[test]
fn test_iterator_for_loop_nested_compiles() {
    let source = r#"
fn main() -> int {
    let list1 = List.new()
    let list2 = List.new()
    for x in list1.iter() {
        for y in list2.iter() {
            // Nested iterator-based loops
        }
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain multiple CALL_NAT opcodes (one for each iterator)
    let call_nat_count = bytecode.iter().filter(|&&b| b == 0x72).count();
    assert!(call_nat_count >= 2, "Expected at least 2 CALL_NAT opcodes for nested iterators");
}

#[test]
fn test_iterator_for_loop_with_body_compiles() {
    let source = r#"
fn main() -> int {
    let list = List.new()
    list.push(10)
    list.push(20)
    list.push(30)
    var sum = 0
    for x in list.iter() {
        sum = sum + x
    }
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CALL_NAT for Iterator.next
    assert!(bytecode.contains(&0x72), "Expected CALL_NAT opcode (0x72)");
    // Should contain arithmetic opcodes
    assert!(bytecode.contains(&0x30), "Expected ADD opcode (0x30)");
}

#[test]
fn test_iterator_for_loop_collect_compiles() {
    let source = r#"
fn main() -> int {
    let list = List.new()
    list.push(1)
    list.push(2)
    list.push(3)
    let iter = list.iter()
    let new_list = iter.collect()
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CALL_NAT for List.iter and Iterator.collect
    assert!(bytecode.contains(&0x72), "Expected CALL_NAT opcode (0x72)");
}

// ============================================================================
// Plan 073: Range Expression Tests
// ============================================================================

#[test]
fn test_range_exclusive_compiles() {
    let source = r#"
fn main() -> int {
    let r = 0..10
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CREATE_RANGE opcode (0x75)
    assert!(bytecode.contains(&0x75), "Expected CREATE_RANGE opcode (0x75)");
    // Should contain CONST_I32 for start and end values
    assert!(bytecode.contains(&0x10), "Expected CONST_I32 opcode (0x10)");
}

#[test]
fn test_range_inclusive_compiles() {
    let source = r#"
fn main() -> int {
    let r = 0..=10
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CREATE_RANGE_EQ opcode (0x76)
    assert!(bytecode.contains(&0x76), "Expected CREATE_RANGE_EQ opcode (0x76)");
    // Should contain CONST_I32 for start and end values
    assert!(bytecode.contains(&0x10), "Expected CONST_I32 opcode (0x10)");
}

#[test]
fn test_range_with_variables_compiles() {
    let source = r#"
fn main() -> int {
    let start int = 0
    let end int = 100
    let r = start..end
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain CREATE_RANGE opcode
    assert!(bytecode.contains(&0x75), "Expected CREATE_RANGE opcode (0x75)");
    // Variables should be stored and loaded (check for STORE_LOCAL)
    assert!(bytecode.contains(&0x21), "Expected STORE_LOCAL opcode (0x21)");
}

#[test]
fn test_range_in_for_loop_compiles() {
    let source = r#"
fn main() -> int {
    var sum = 0
    for x in 0..10 {
        sum = sum + x
    }
    sum
}
"#;
    let bytecode = compile_to_bytecode(source);
    // For loops use range expressions internally
    // Should contain loop control opcodes
    assert!(bytecode.contains(&0x52), "Expected LT opcode (0x52)");
    assert!(bytecode.contains(&0x60), "Expected JMP opcode (0x60)");
}

#[test]
fn test_range_nested_compiles() {
    let source = r#"
fn main() -> int {
    let r1 = 0..10
    let r2 = 5..=15
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain both CREATE_RANGE and CREATE_RANGE_EQ
    assert!(bytecode.contains(&0x75), "Expected CREATE_RANGE opcode (0x75)");
    assert!(bytecode.contains(&0x76), "Expected CREATE_RANGE_EQ opcode (0x76)");
}

// ============================================================================
// Plan 073: F-String Tests
// ============================================================================

#[test]
fn test_fstr_simple_compiles() {
    let source = r#"
fn main() -> int {
    let s = f"hello world"
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain BUILD_FSTR opcode (0x77)
    assert!(bytecode.contains(&0x77), "Expected BUILD_FSTR opcode (0x77)");
    // Should contain LOAD_STR for string literal
    assert!(bytecode.contains(&0x1F), "Expected LOAD_STR opcode (0x1F)");
}

#[test]
fn test_fstr_with_variable_compiles() {
    let source = r#"
fn main() -> int {
    let name = "World"
    let s = f"hello $name"
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain BUILD_FSTR opcode
    assert!(bytecode.contains(&0x77), "Expected BUILD_FSTR opcode (0x77)");
    // Should contain LOAD_STR for string literal part
    assert!(bytecode.contains(&0x1F), "Expected LOAD_STR opcode (0x1F)");
}

#[test]
fn test_fstr_with_expression_compiles() {
    let source = r#"
fn main() -> int {
    let x = 10
    let y = 20
    let s = f"sum: ${x + y}"
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain BUILD_FSTR opcode
    assert!(bytecode.contains(&0x77), "Expected BUILD_FSTR opcode (0x77)");
    // Should contain ADD opcode for expression
    assert!(bytecode.contains(&0x30), "Expected ADD opcode (0x30)");
}

#[test]
fn test_fstr_multiple_parts_compiles() {
    let source = r#"
fn main() -> int {
    let name = "World"
    let count = 42
    let s = f"hello $name, you have ${count} messages"
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain BUILD_FSTR opcode
    assert!(bytecode.contains(&0x77), "Expected BUILD_FSTR opcode (0x77)");
}

#[test]
fn test_fstr_nested_compiles() {
    let source = r#"
fn main() -> int {
    let x = 10
    let s1 = f"value: $x"
    let s2 = f"result: ${s1}"
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain BUILD_FSTR opcode
    // Count occurrences to verify we have 2 f-strings
    let fstr_count = bytecode.iter().filter(|&&b| b == 0x77).count();
    assert!(fstr_count >= 2, "Expected at least 2 BUILD_FSTR opcodes");
}

// ============================================================================
// Plan 073: Is Pattern Matching Tests
// ============================================================================

#[test]
fn test_is_simple_compiles() {
    let source = r#"
fn main() -> int {
    let x = 10
    is x {
        10 -> 0
        20 -> 1
        else -> 2
    }
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain EQ for pattern matching
    assert!(bytecode.contains(&0x50), "Expected EQ opcode (0x50)");
    // Should contain JMP_IF_Z for conditional branching
    assert!(bytecode.contains(&0x61), "Expected JMP_IF_Z opcode (0x61)");
    // Should contain JMP for branching
    assert!(bytecode.contains(&0x60), "Expected JMP opcode (0x60)");
}

#[test]
fn test_is_single_match_compiles() {
    let source = r#"
fn main() -> int {
    let x = 10
    is x {
        10 -> 0
        else -> 1
    }
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain control flow opcodes
    assert!(bytecode.contains(&0x50), "Expected EQ opcode (0x50)");
    assert!(bytecode.contains(&0x61), "Expected JMP_IF_Z opcode (0x61)");
}

#[test]
fn test_is_nested_compiles() {
    let source = r#"
fn main() -> int {
    let x = 1
    is x {
        1 -> {
            let y = 2
            y
        }
        else -> 0
    }
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain EQ opcode
    assert!(bytecode.contains(&0x50), "Expected EQ opcode (0x50)");
}

#[test]
fn test_is_multiple_branches_compiles() {
    let source = r#"
fn main() -> int {
    let x = 10
    is x {
        1 -> 0
        2 -> 1
        3 -> 2
        4 -> 3
        5 -> 4
        else -> 99
    }
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain multiple EQ opcodes (one per branch)
    let eq_count = bytecode.iter().filter(|&&b| b == 0x50).count();
    assert!(eq_count >= 5, "Expected at least 5 EQ opcodes for 5 branches");
}

// Plan 073 Phase 8.3.7: Is IfBranch (Conditional Pattern Matching)
#[test]
fn test_is_if_branch_compiles() {
    let source = r#"
fn main() -> int {
    let x = 10
    is x {
        if x > 5 -> 100
        else -> 0
    }
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain JMP_IF_Z for condition check
    assert!(bytecode.contains(&0x61), "Expected JMP_IF_Z opcode (0x61)");
    // Should contain JMP for branching
    assert!(bytecode.contains(&0x60), "Expected JMP opcode (0x60)");
}

#[test]
fn test_is_if_branch_with_comparison() {
    let source = r#"
fn main() -> int {
    let x = 15
    is x {
        if x > 10 -> 1
        if x < 10 -> 2
        else -> 0
    }
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain JMP_IF_Z for each condition
    let jz_count = bytecode.iter().filter(|&&b| b == 0x61).count();
    assert!(jz_count >= 2, "Expected at least 2 JMP_IF_Z opcodes for 2 if branches");
    // Should contain JMP opcodes
    assert!(bytecode.contains(&0x60), "Expected JMP opcode (0x60)");
}

#[test]
fn test_is_if_branch_complex_condition() {
    let source = r#"
fn main() -> int {
    let x = 25
    is x {
        if x > 20 -> 100
        else -> 0
    }
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain conditional jump opcodes
    assert!(bytecode.contains(&0x61), "Expected JMP_IF_Z opcode (0x61)");
}

#[test]
fn test_is_if_branch_with_block_body() {
    let source = r#"
fn main() -> int {
    let x = 10
    is x {
        if x > 5 -> {
            let y = x * 2
            y
        }
        else -> 0
    }
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain control flow opcodes
    assert!(bytecode.contains(&0x61), "Expected JMP_IF_Z opcode (0x61)");
    assert!(bytecode.contains(&0x60), "Expected JMP opcode (0x60)");
}

#[test]
fn test_is_mixed_branches() {
    let source = r#"
fn main() -> int {
    let x = 10
    is x {
        5 -> 1
        if x > 10 -> 2
        else -> 0
    }
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain EQ for EqBranch
    assert!(bytecode.contains(&0x50), "Expected EQ opcode (0x50)");
    // Should contain JMP_IF_Z for IfBranch
    assert!(bytecode.contains(&0x61), "Expected JMP_IF_Z opcode (0x61)");
    // Should contain JMP for branching
    assert!(bytecode.contains(&0x60), "Expected JMP opcode (0x60)");
}

// Plan 073 Phase 8.4: May<T> Question Operators (?? and .?)
#[test]
fn test_null_coalesce_compiles() {
    let source = r#"
fn main() -> int {
    let x = 10
    let y = x ?? 0
    y
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain NULL_COALESCE opcode (0x78)
    assert!(bytecode.contains(&0x78), "Expected NULL_COALESCE opcode (0x78)");
}

#[test]
fn test_null_coalesce_with_variable() {
    let source = r#"
fn main() -> int {
    let x = 42
    let result = x ?? 100
    result
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain NULL_COALESCE opcode
    assert!(bytecode.contains(&0x78), "Expected NULL_COALESCE opcode (0x78)");
}

#[test]
fn test_null_coalesce_nested() {
    let source = r#"
fn main() -> int {
    let x = 10
    let y = 20
    let z = x ?? y
    z
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain NULL_COALESCE opcode
    assert!(bytecode.contains(&0x78), "Expected NULL_COALESCE opcode (0x78)");
}

#[test]
fn test_error_propagate_compiles() {
    let source = r#"
fn main() -> int {
    let x = 10
    let y = x.?
    y
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain ERROR_PROPAGATE opcode (0x79)
    assert!(bytecode.contains(&0x79), "Expected ERROR_PROPAGATE opcode (0x79)");
}

#[test]
fn test_error_propagate_with_function_call() {
    let source = r#"
fn get_value() -> int {
    42
}

fn main() -> int {
    let x = get_value()
    let y = x.?
    y
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain ERROR_PROPAGATE opcode
    assert!(bytecode.contains(&0x79), "Expected ERROR_PROPAGATE opcode (0x79)");
}

// Plan 073 Phase 8.6: TypeDecl/EnumDecl/SpecDecl Support
#[test]
fn test_type_decl_compiles() {
    let source = r#"
type Point {
    x int
    y int
}

fn main() -> int {
    let p = Point(10, 20)
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Type declarations don't generate bytecode, but type instances should work
    // The code should compile without errors
    assert!(bytecode.len() > 0, "Bytecode should be generated");
}

#[test]
#[ignore = "type methods not fully implemented in codegen"]
fn test_type_decl_with_methods_compiles() {
    let source = r#"
type Counter {
    count int

    fn increment() {
        count = count + 1
    }
}

fn main() -> int {
    let c = Counter(0)
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Type with methods should still compile
    assert!(bytecode.len() > 0, "Bytecode should be generated");
}

#[test]
fn test_enum_decl_compiles() {
    let source = r#"
enum Color {
    Red = 0
    Green = 1
    Blue = 2
}

fn main() -> int {
    let c = Color.Red
    0
}
"#;
    // Parse the source code
    let mut parser = Parser::from(source);
    let code = parser.parse().expect("Parse failed");

    // Create Codegen with shared TypeStore from Parser
    let mut codegen = Codegen::new_with_type_store(parser.type_store.clone());
    for stmt in code.stmts {
        codegen.compile_stmt(&stmt).expect("Codegen failed");
    }

    // Enum declarations don't generate bytecode
    // But the code should compile without errors
    assert!(codegen.code.len() > 0, "Bytecode should be generated");
}

#[test]
fn test_spec_decl_compiles() {
    let source = r#"
spec Printable {
    fn to_str()
}

fn main() -> int {
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Spec declarations don't generate bytecode
    // But the code should compile without errors
    assert!(bytecode.len() > 0, "Bytecode should be generated");
}

#[test]
fn test_spec_decl_with_default_methods_compiles() {
    let source = r#"
spec Show {
    fn show()
}

fn main() -> int {
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Spec with default methods should compile
    assert!(bytecode.len() > 0, "Bytecode should be generated");
}

#[test]
fn test_combined_type_enum_spec_compiles() {
    let source = r#"
type Point {
    x int
    y int
}

enum Color {
    Red = 0
    Green = 1
}

spec Printable {
    fn to_str()
}

fn main() -> int {
    let p = Point(10, 20)
    let c = Color.Red
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // All three declarations should coexist
    assert!(bytecode.len() > 0, "Bytecode should be generated");
}

// Plan 075: Tests for Dot expression assignment support
#[test]
fn test_dot_expression_simple_assignment() {
    let source = r#"
fn main() -> int {
    let obj = {x: 0}
    obj.x = 42
    obj.x
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain SET_FIELD opcode (0x2A)
    assert!(bytecode.contains(&0x2A), "Expected SET_FIELD opcode (0x2A)");
}

#[test]
fn test_dot_expression_nested_assignment() {
    let source = r#"
fn main() -> int {
    let server = {host: "localhost", port: 0}
    server.host = "example.com"
    server.port = 8080
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain SET_FIELD opcode (0x2A) for both assignments
    let set_field_count = bytecode.iter().filter(|&&x| x == 0x2A).count();
    assert_eq!(set_field_count, 2, "Expected 2 SET_FIELD opcodes");
}

#[test]
fn test_dot_expression_with_expressions() {
    let source = r#"
fn main() -> int {
    let obj = {value: 0}
    obj.value = 10 * 5
    obj.value
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain SET_FIELD opcode
    assert!(bytecode.contains(&0x2A), "Expected SET_FIELD opcode (0x2A)");
}

#[test]
fn test_dot_expression_multiple_fields() {
    let source = r#"
fn main() -> int {
    let config = {
        host: "localhost"
        port: 8080
        debug: true
    }
    config.host = "example.com"
    config.port = 9090
    config.debug = false
    0
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain SET_FIELD opcode (0x2A) for three assignments
    let set_field_count = bytecode.iter().filter(|&&x| x == 0x2A).count();
    assert_eq!(set_field_count, 3, "Expected 3 SET_FIELD opcodes");
}

#[test]
fn test_dot_expression_assignment_and_access() {
    let source = r#"
fn main() -> int {
    let obj = {x: 0, y: 0}
    obj.x = 10
    obj.y = obj.x + 5
    obj.y
}
"#;
    let bytecode = compile_to_bytecode(source);
    // Should contain both SET_FIELD (0x2A) and GET_FIELD (0x2D)
    assert!(bytecode.contains(&0x2A), "Expected SET_FIELD opcode (0x2A)");
    assert!(bytecode.contains(&0x2D), "Expected GET_FIELD opcode (0x2D)");
}
