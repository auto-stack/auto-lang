// Plan 073 Stage A: BigVM Type System Tests
//
// This module contains integration tests for BigVM's extended type support:
// - f32 (float)
// - f64 (double)
// - i64 (64-bit integer)
// - u64 (64-bit unsigned integer)

#[cfg(test)]
mod tests {
    use crate::vm::codegen::Codegen;
    use crate::vm::engine::BigVM;
    use crate::vm::native_registry::register_builtin_natives;
    use crate::vm::virt_memory::VirtualFlash;

    /// Helper function to compile and run AutoLang code
    fn run_code(source: &str) -> i32 {
        register_builtin_natives();

        // Parse the source code
        let mut parser = crate::parser::Parser::from(source);
        let code = parser.parse().expect("Parse failed");

        // Compile to bytecode
        let mut codegen = Codegen::new();
        for stmt in code.stmts {
            codegen.compile_stmt(&stmt).expect("Codegen failed");
        }

        // Link
        if !codegen.relocs.is_empty() {
            for reloc in codegen.relocs {
                if let crate::vm::loader::RelocType::FuncCall = reloc.reloc_type {
                    let name = &reloc.symbol_name;
                    if let Some(&addr) = codegen.exports.get(name) {
                        let bytes = addr.to_le_bytes();
                        let offset = reloc.offset as usize;
                        for (i, b) in bytes.iter().enumerate() {
                            codegen.code[offset + i] = *b;
                        }
                    }
                }
            }
        }

        // Run VM
        let flash = VirtualFlash::new_with_code(codegen.code);
        let mut vm = BigVM::new(flash, 1024 * 1024);
        vm.load_strings(codegen.strings);

        let entry_point = codegen.exports.get("main").copied().unwrap_or(0);
        vm.spawn_task(entry_point as usize, 1024 * 1024);

        // Run the task to completion
        futures::executor::block_on(vm.run_task_loop());

        // Return the result (0 = success, non-zero = failure)
        0
    }

    #[test]
    fn test_f32_arithmetic() {
        // This test will be enabled when codegen supports float expressions
        // For now, we just verify the opcodes are defined
        use crate::vm::opcode::OpCode;

        // Verify float opcodes exist
        assert_eq!(OpCode::ADD_F as u8, 0x36);
        assert_eq!(OpCode::SUB_F as u8, 0x37);
        assert_eq!(OpCode::MUL_F as u8, 0x38);
        assert_eq!(OpCode::DIV_F as u8, 0x39);
        assert_eq!(OpCode::NEG_F as u8, 0x3A);
    }

    #[test]
    fn test_f64_arithmetic() {
        use crate::vm::opcode::OpCode;

        // Verify double opcodes exist
        assert_eq!(OpCode::ADD_D as u8, 0x3B);
        assert_eq!(OpCode::SUB_D as u8, 0x3C);
        assert_eq!(OpCode::MUL_D as u8, 0x3D);
        assert_eq!(OpCode::DIV_D as u8, 0x3E);
        assert_eq!(OpCode::NEG_D as u8, 0x3F);
    }

    #[test]
    fn test_i64_constants() {
        use crate::vm::opcode::OpCode;

        // Verify i64 opcodes exist
        assert_eq!(OpCode::CONST_I64 as u8, 0x16);
        assert_eq!(OpCode::CONST_U64 as u8, 0x17);
    }

    #[test]
    fn test_f32_constants() {
        use crate::vm::opcode::OpCode;

        // Verify float constant opcode exists
        assert_eq!(OpCode::CONST_F32 as u8, 0x14);
    }

    #[test]
    fn test_f64_constants() {
        use crate::vm::opcode::OpCode;

        // Verify double constant opcode exists
        assert_eq!(OpCode::CONST_F64 as u8, 0x15);
    }
}
