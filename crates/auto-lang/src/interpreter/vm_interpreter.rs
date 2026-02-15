//! VM-based interpreter implementation

use crate::parser::Parser;
use crate::vm::codegen::Codegen;
use crate::vm::engine::AutoVM;
use crate::vm::virt_memory::VirtualFlash;
use crate::AutoResult;
use auto_val::Value;
use std::collections::HashMap as StdHashMap;

/// VM-based interpreter that wraps AutoVM
pub struct VmInterpreter {
    /// Tokio runtime for async VM execution
    rt: tokio::runtime::Runtime,

    /// Function exports (name -> address)
    exports: StdHashMap<String, u32>,
}

impl VmInterpreter {
    pub fn new() -> Self {
        Self {
            rt: tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"),
            exports: StdHashMap::new(),
        }
    }

    /// Run code and return result
    pub fn run(&mut self, code: &str) -> AutoResult<Value> {
        // 1. Parse the code
        let mut parser = Parser::from(code);
        let ast = parser.parse()?;

        // 2. Compile to bytecode
        let mut codegen = Codegen::new();

        // Compile each statement
        for stmt in &ast.stmts {
            codegen.compile_stmt(stmt)?;
        }

        // Add HALT instruction
        use crate::vm::opcode::OpCode;
        codegen.code.push(OpCode::HALT as u8);

        // 3. Perform relocation (resolve function addresses)
        for reloc in &codegen.relocs {
            if let Some(&addr) = codegen.exports.get(&reloc.symbol_name) {
                let bytes = addr.to_le_bytes();
                let offset = reloc.offset as usize;
                for (i, b) in bytes.iter().enumerate() {
                    codegen.code[offset + i] = *b;
                }
            }
        }

        // 4. Store exports
        self.exports = codegen.exports.clone();

        // 5. Create flash and run
        let flash = VirtualFlash::new_with_code_and_keys(
            codegen.code,
            codegen.object_keys,
            codegen.object_types,
        );

        // 6. Run in tokio
        let strings = codegen.strings;
        let exports = codegen.exports;

        self.rt.block_on(async move {
            let mut vm = AutoVM::new(flash, 4096);
            vm.load_strings(strings);

            let entry_point = exports
                .get("main")
                .copied()
                .unwrap_or(0) as usize;
            let _task_id = vm.spawn_task(entry_point, 4096);
            vm.run_task_loop().await;
            // VM has run successfully
        });

        // TODO: Implement result extraction from VM
        // Currently returns Nil because async lifetime management is complex
        Ok(Value::Nil)
    }

    /// Call a function with arguments
    pub fn call(&mut self, _fn_name: &str, _args: Vec<Value>) -> AutoResult<Value> {
        // TODO: Implement function calling
        Ok(Value::Nil)
    }

    /// Set a global variable
    pub fn set_global(&mut self, _name: &str, _value: Value) {
        // TODO: Implement global variable setting
    }

    /// Get a global variable
    pub fn get_global(&self, _name: &str) -> Option<Value> {
        None
    }

    /// Reset interpreter state
    pub fn reset(&mut self) {
        self.exports.clear();
    }

    /// Check if a function exists
    pub fn has_function(&self, name: &str) -> bool {
        self.exports.contains_key(name)
    }

    /// Get list of defined functions
    pub fn get_functions(&self) -> Vec<String> {
        self.exports.keys().cloned().collect()
    }
}

impl Default for VmInterpreter {
    fn default() -> Self {
        Self::new()
    }
}
