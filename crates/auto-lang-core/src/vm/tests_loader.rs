#[cfg(test)]
mod tests {
    use crate::vm::loader::{Linker, Module, RelocEntry, RelocType};
    // use crate::vm::opcode::OpCode; // Make sure OpCode is accessible or redefine simple constants
    use std::collections::HashMap;

    // Define simple OpCodes locally if needed or import
    // OpCode::CALL is 0x70
    // OpCode::RET is 0x71
    // OpCode::CONST_I32 is 0x10
    // OpCode::HALT is 0xFF

    #[test]
    fn test_linker_basic() {
        // Module 1: Main
        // CALL FunctionA
        // HALT
        let mut main_code = Vec::new();
        // CALL placeholder (size 5: opcode + 4 bytes addr)
        main_code.push(0x70); // CALL
        main_code.extend_from_slice(&0u32.to_le_bytes()); // Placeholder
        main_code.push(0xFF); // HALT

        let main_module = Module {
            name: "Main".to_string(),
            code: main_code,
            exports: HashMap::new(),
            relocs: vec![RelocEntry {
                offset: 1, // Parameter of CALL is at index 1
                symbol_name: "FuncA".to_string(),
                reloc_type: RelocType::FuncCall,
            }],
            strings: Vec::new(),
        };

        // Module 2: Lib
        // FuncA:
        //   CONST_I32 42
        //   RET 0
        let mut lib_code = Vec::new();
        // Offset 0 of lib_code is FuncA entry
        lib_code.push(0x10); // CONST_I32
        lib_code.extend_from_slice(&42i32.to_le_bytes());
        lib_code.push(0x71); // RET
        lib_code.push(0); // n_args

        let mut exports = HashMap::new();
        exports.insert("FuncA".to_string(), 0);

        let lib_module = Module {
            name: "Lib".to_string(),
            code: lib_code,
            exports,
            relocs: Vec::new(),
            strings: Vec::new(),
        };

        let mut linker = Linker::new();
        linker.add_module(main_module);
        linker.add_module(lib_module);

        let (linked_code, symbols) = linker.link().expect("Linking failed");

        // Verify Symbols
        // Main is first (6 bytes). Lib starts at 6.
        // FuncA should be at 6.
        assert_eq!(*symbols.get("FuncA").unwrap(), 6);

        // Verify Main Code Patching
        // CALL at 0. Address at 1..5.
        // Should contain address 6 (0x06000000 little endian)
        let addr_bytes = &linked_code[1..5];
        let addr = u32::from_le_bytes(addr_bytes.try_into().unwrap());
        assert_eq!(addr, 6);

        // Verify total size = 6 + 7 = 13
        // Main: 1+4+1 = 6
        // Lib: 1+4+1+1 = 7
        assert_eq!(linked_code.len(), 13);
    }
}
