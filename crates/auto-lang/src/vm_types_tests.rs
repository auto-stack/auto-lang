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
