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
    /// Function exports (name -> address)
    exports: StdHashMap<String, u32>,
    /// Global variables (name -> value)
    globals: StdHashMap<String, Value>,
}

impl VmInterpreter {
    pub fn new() -> Self {
        Self {
            exports: StdHashMap::new(),
            globals: StdHashMap::new(),
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
        let n = ast.stmts.len();
        for (i, stmt) in ast.stmts.iter().enumerate() {
            let is_last = i == n - 1;
            let old_pop = codegen.should_pop_expr_result;
            // Pop all but the last expression statement to get a result from the script
            if !is_last {
                codegen.should_pop_expr_result = true;
            }
            codegen.compile_stmt(stmt)?;
            codegen.should_pop_expr_result = old_pop;
        }

        // Add HALT instruction
        use crate::vm::opcode::OpCode;
        codegen.code.push(OpCode::HALT as u8);

        // 2b. Insert RESERVE_STACK for main task locals
        // Without this, temporary stack pushes overwrite local variable slots (BP+1, BP+2, etc.)
        let n_locals = codegen.max_locals;
        if n_locals > 0 {
            // Insert RESERVE_STACK at position 0 (2 bytes: opcode + count)
            codegen.code.insert(0, OpCode::RESERVE_STACK as u8);
            codegen.code.insert(1, n_locals as u8);

            // Shift all exports by 2 bytes
            for (_, addr) in codegen.exports.iter_mut() {
                *addr += 2;
            }

            // Shift all reloc offsets by 2 bytes
            for reloc in &mut codegen.relocs {
                reloc.offset += 2;
            }

            // Shift all jump placeholders by 2 bytes
            for placeholder in &mut codegen.jump_placeholders {
                *placeholder += 2;
            }
        }

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

        // 6. Run in tokio using global runtime
        let strings = codegen.strings;
        let exports = codegen.exports;

        // Use global runtime to avoid creating/dropping runtimes in async context
        let rt = crate::get_global_runtime();
        let final_result = rt.block_on(async move {
            let mut vm = AutoVM::new(flash, 4096);
            vm.load_strings(strings);

            let entry_point = exports.get("main").copied().unwrap_or(0) as usize;
            let task_id = vm.spawn_task(entry_point, 4096);
            vm.run_task_loop().await;

            // Extract the result from the task's RAM
            let mut result = None;
            if let Some(task_mutex) = vm.tasks.get(&task_id).map(|v| v.value().clone()) {
                let task = task_mutex.lock().await;
                if task.ram.sp > 0 {
                    let top_val = task.ram.raw[(task.ram.sp - 1) as usize];

                    // Negative values are tagged string indices: actual index = -(val+1)
                    // Non-negative values are integers
                    if top_val < 0 {
                        let str_idx = (-top_val - 1) as usize;
                        let strings = vm.strings.read().unwrap();
                        if str_idx < strings.len() {
                            let bytes = &strings[str_idx];
                            if let Ok(s) = String::from_utf8(bytes.clone()) {
                                result = Some(Value::Str(s.into()));
                            }
                        }
                    }
                    // Check if it's an object ID (1000000-1999999)
                    if result.is_none() && top_val >= 1000000 && top_val < 2000000 {
                        let id = top_val as u64;
                        if let Some(obj_arc) = vm.objects.get(&id) {
                            let obj = obj_arc.read().unwrap();
                            let mut result_obj = auto_val::Obj::new();
                            for (key, val) in &obj.fields {
                                result_obj.set(key.clone(), val.clone());
                            }
                            result = Some(Value::Obj(result_obj));
                        }
                    }
                    // Check if it's an array ID (2000000-2999999)
                    if result.is_none() && top_val >= 2000000 && top_val < 3000000 {
                        let id = top_val as u64;
                        if let Some(arr_arc) = vm.arrays.get(&id) {
                            let arr = arr_arc.read().unwrap();
                            let items: Vec<Value> = arr.iter().cloned().collect();
                            result = Some(Value::Array(auto_val::Array::from_vec(items)));
                        }
                    }
                    if result.is_none() {
                        result = Some(Value::Int(top_val));
                    }
                }
            }
            result
        });

        Ok(final_result.unwrap_or(Value::Nil))
    }

    /// Call a function with arguments
    pub fn call(&mut self, _fn_name: &str, _args: Vec<Value>) -> AutoResult<Value> {
        // TODO: Implement function calling
        Ok(Value::Nil)
    }

    /// Set a global variable
    pub fn set_global(&mut self, name: &str, value: Value) {
        self.globals.insert(name.to_string(), value);
    }

    /// Get a global variable
    pub fn get_global(&self, name: &str) -> Option<Value> {
        self.globals.get(name).cloned()
    }

    /// Reset interpreter state
    pub fn reset(&mut self) {
        self.exports.clear();
        self.globals.clear();
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
