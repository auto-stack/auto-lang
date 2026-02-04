// Test full float/double support from parsing to BigVM codegen (Plan 073 Stage A.5)

#[cfg(test)]
mod tests {
    use crate::parser::Parser;
    use crate::vm::codegen::Codegen;

    #[test]
    fn test_float_literals_codegen() {
        let source = r#"
fn test_float_lit() -> float {
    3.14
}

fn main() -> int {
    0
}
"#;

        // Parse the source
        let mut parser = Parser::from(source);
        let code = parser.parse().expect("Parse failed");

        // Compile to bytecode
        let mut codegen = Codegen::new();
        for stmt in code.stmts {
            codegen.compile_stmt(&stmt).expect("Codegen failed");
        }

        // Verify that float constant opcode was emitted
        assert!(codegen.code.contains(&0x14), "Expected CONST_F32 opcode (0x14)");
    }

    #[test]
    fn test_double_literals_codegen() {
        let source = r#"
fn test_double_lit() -> double {
    2.718281828d
}

fn main() -> int {
    0
}
"#;

        // Parse the source
        let mut parser = Parser::from(source);
        let code = parser.parse().expect("Parse failed");

        // Compile to bytecode
        let mut codegen = Codegen::new();
        for stmt in code.stmts {
            codegen.compile_stmt(&stmt).expect("Codegen failed");
        }

        // Verify that double constant opcode was emitted
        assert!(codegen.code.contains(&0x15), "Expected CONST_F64 opcode (0x15)");
    }

    #[test]
    fn test_float_arithmetic_codegen() {
        let source = r#"
fn test_float_arith() -> float {
    1.5 + 2.5
}

fn main() -> int {
    0
}
"#;

        // Parse the source
        let mut parser = Parser::from(source);
        let code = parser.parse().expect("Parse failed");

        // Compile to bytecode
        let mut codegen = Codegen::new();
        for stmt in code.stmts {
            codegen.compile_stmt(&stmt).expect("Codegen failed");
        }

        // Verify that float arithmetic opcode was emitted
        assert!(codegen.code.contains(&0x36), "Expected ADD_F opcode (0x36)");
    }

    #[test]
    fn test_double_arithmetic_codegen() {
        let source = r#"
fn test_double_arith() -> double {
    3.14d * 2.0d
}

fn main() -> int {
    0
}
"#;

        // Parse the source
        let mut parser = Parser::from(source);
        let code = parser.parse().expect("Parse failed");

        // Compile to bytecode
        let mut codegen = Codegen::new();
        for stmt in code.stmts {
            codegen.compile_stmt(&stmt).expect("Codegen failed");
        }

        // Verify that double arithmetic opcode was emitted
        assert!(codegen.code.contains(&0x3D), "Expected MUL_D opcode (0x3D)");
    }

    #[test]
    fn test_float_negation_codegen() {
        let source = r#"
fn test_float_neg() -> float {
    -3.14
}

fn main() -> int {
    0
}
"#;

        // Parse the source
        let mut parser = Parser::from(source);
        let code = parser.parse().expect("Parse failed");

        // Compile to bytecode
        let mut codegen = Codegen::new();
        for stmt in code.stmts {
            codegen.compile_stmt(&stmt).expect("Codegen failed");
        }

        // Verify that float negation opcode was emitted
        assert!(codegen.code.contains(&0x3A), "Expected NEG_F opcode (0x3A)");
    }

    #[test]
    fn test_end_to_end_float_support() {
        let source = r#"
fn test_float_return() -> float {
    3.14
}

fn test_double_return() -> double {
    2.718281828d
}

fn test_float_arith() -> float {
    1.5 + 2.5
}

fn test_double_arith() -> double {
    3.14d * 2.0d
}

fn test_float_neg() -> float {
    -3.14
}

fn test_mixed() -> double {
    3.14d + 2.718d
}

fn main() -> int {
    0
}
"#;

        // Parse the source
        let mut parser = Parser::from(source);
        let code = parser.parse().expect("Parse failed");

        // Compile to bytecode
        let mut codegen = Codegen::new();
        for stmt in code.stmts {
            codegen.compile_stmt(&stmt).expect("Codegen failed");
        }

        // Verify all expected opcodes were emitted
        assert!(codegen.code.contains(&0x14), "Expected CONST_F32 opcode");
        assert!(codegen.code.contains(&0x15), "Expected CONST_F64 opcode");
        assert!(codegen.code.contains(&0x36), "Expected ADD_F opcode");
        assert!(codegen.code.contains(&0x3D), "Expected MUL_D opcode");
        assert!(codegen.code.contains(&0x3A), "Expected NEG_F opcode");
    }
}
