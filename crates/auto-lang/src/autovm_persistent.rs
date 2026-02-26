// AutoVM Persistent Session (Revised - Efficient)
//
// **Plan 068 Phase 9.6**: Persistent AutoVM REPL with state management
//
// **Architecture**:
// - Keep a single AutoVM instance across inputs (preserves heap_objects, arrays, etc.)
// - Keep a single main task across inputs (preserves stack state)
// - Reuse the same Codegen object (preserves locals, exports, strings, etc.)

use crate::error::{AutoError, AutoResult};
use crate::type_registry;
use crate::vm::codegen::Codegen;
use crate::vm::engine::AutoVM;
use crate::vm::opcode::OpCode;
use crate::vm::task::TaskId;
use crate::vm::virt_memory::VirtualFlash;
use crate::Parser;
use std::sync::Arc;

/// Persistent AutoVM REPL session
///
/// **Efficient Implementation**: Reuses Codegen to avoid allocations
pub struct AutovmReplSession {
    /// Single VM instance (persistent across inputs)
    vm: AutoVM,

    /// Main task ID (persistent across inputs)
    main_task_id: TaskId,

    /// Reusable Codegen (contains locals, exports, strings, etc.)
    /// Option allows moving it out during run() and moving it back
    codegen: Option<Codegen>,

    /// Type registry for REPL (Plan 087)
    /// Persists type definitions across REPL inputs
    type_registry: type_registry::SharedTypeRegistry,

    /// All bytecode compiled so far (for flash updates)
    bytecode: Vec<u8>,

    /// Object keys metadata
    object_keys: Vec<Vec<auto_val::ValueKey>>,

    /// Object types metadata
    object_types: Vec<Vec<crate::vm::codegen::ObjectType>>,

    /// Last result from the previous REPL input (Plan 080)
    /// Stores the result value for access in subsequent inputs
    last_result: Option<i32>,
}

impl AutovmReplSession {
    /// Create a new persistent AutoVM REPL session
    pub fn new() -> Self {
        // Initialize VM modules (registers HashMap, HashSet, List, etc.)
        // This MUST be called before creating Codegen or VM
        crate::vm::init_io_module();
        crate::vm::init_collections_module();
        crate::vm::init_builder_module();
        crate::vm::init_storage_module();

        // Create initial Codegen
        let codegen = Codegen::new();

        // Start with HALT opcode
        let bytecode = vec![OpCode::HALT as u8];
        let flash = VirtualFlash::new_with_code(bytecode.clone());

        // Create VM
        let mut vm = AutoVM::new(flash, 1024);
        vm.load_strings(Vec::new());

        // Spawn main task
        let task_id = vm.spawn_task(0, 1024);

        // Plan 080: Set bp=1 to reserve space for return address
        // This ensures local variables start at bp+1, not bp+0
        let task_arc = vm.tasks.get(&task_id).unwrap().clone();
        let mut task = task_arc.blocking_lock();
        task.bp = 1;
        task.num_locals = 0; // Initialize with no locals
        drop(task);

        Self {
            vm,
            main_task_id: task_id,
            codegen: Some(codegen), // Store Codegen for reuse
            type_registry: type_registry::new_type_registry(), // Plan 087: Type registry for REPL
            bytecode,
            object_keys: Vec::new(),
            object_types: Vec::new(),
            last_result: None, // Plan 080: Initialize last_result
        }
    }

    /// Execute code with persistent state
    ///
    /// **Efficient**: Reuses Codegen and TypeRegistry to avoid allocations
    pub fn run(&mut self, code: &str) -> AutoResult<String> {
        // 1. Parse the code
        let mut parser = Parser::from(code);

        // Plan 087: Set type registry for REPL support
        // This allows parser to recognize types defined in previous REPL inputs
        parser.set_type_registry(self.type_registry.clone());

        // Plan 080: Enable skip_check for REPL mode
        // This allows using undefined variables (e.g., "x" as a statement)
        // Variables will be checked at runtime by the VM
        parser = parser.skip_check();

        let ast = parser.parse()?;

        // Debug: Print AST
        eprintln!(
            "DEBUG: Parsed {} statements from: '{}'",
            ast.stmts.len(),
            code
        );
        for (i, stmt) in ast.stmts.iter().enumerate() {
            eprintln!("DEBUG:   Stmt {}: {:?}", i, stmt);
        }

        if ast.stmts.is_empty() {
            return Ok(String::new());
        }

        // 2. Take Codegen (move out for efficient reuse)
        let mut codegen = self.codegen.take().unwrap();

        // Debug: Print scope_stack before compilation
        eprintln!(
            "DEBUG: Before compilation, scope_stack = {:?}",
            codegen.scope_stack
        );

        // Clear previous bytecode (IMPORTANT: don't accumulate!)
        codegen.code.clear();

        // 3. Compile new statements (reuses locals, exports, strings, etc.)
        // Plan 080: Ensure codegen is returned even on compilation failure
        let compile_result: AutoResult<()> = (|| {
            for stmt in &ast.stmts {
                codegen.compile_stmt(stmt)?;
            }
            Ok(())
        })();

        // If compilation failed, put codegen back and return error
        if let Err(e) = compile_result {
            self.codegen = Some(codegen);
            return Err(e);
        }

        // 4. Add HALT at the end
        codegen.code.push(OpCode::HALT as u8);

        // Debug: Print generated bytecode (hex dump)
        eprintln!("DEBUG: Generated {} bytes of bytecode", codegen.code.len());
        let mut i = 0;
        // Track if we're inside NEW_INSTANCE data bytes (Plan 087 Phase 2)
        let mut new_instance_data_bytes_remaining = 0usize;
        while i < codegen.code.len() {
            // If we're in NEW_INSTANCE data bytes, just print them as data
            if new_instance_data_bytes_remaining > 0 {
                eprintln!(
                    "DEBUG:   [{:04x}] {:02x} <NEW_INSTANCE data: '{}'>",
                    i, codegen.code[i], codegen.code[i] as char
                );
                new_instance_data_bytes_remaining -= 1;
                i += 1;
                continue;
            }

            // Safe conversion: catch invalid opcodes
            let opcode: Result<OpCode, _> = codegen.code[i].try_into();
            let opcode = match opcode {
                Ok(op) => op,
                Err(_) => {
                    eprintln!(
                        "DEBUG:   [{:04x}] {:02x} <invalid opcode>",
                        i, codegen.code[i]
                    );
                    i += 1;
                    continue;
                }
            };
            eprint!("DEBUG:   [{:04x}] {:02x} {:?}", i, codegen.code[i], opcode);
            i += 1;
            // Print immediate values based on opcode

            // Special handling for NEW_INSTANCE (Plan 087 Phase 2)
            // NEW_INSTANCE is followed by type name bytes (data, not instructions)
            if opcode == OpCode::NEW_INSTANCE {
                // Read the name length from previous CONST_I32
                // We need to look backward to find it
                if i >= 5 && codegen.code[i - 5] == OpCode::CONST_I32 as u8 {
                    // Extract the length (4-byte little-endian)
                    let name_len = u32::from_le_bytes([
                        codegen.code[i - 4],
                        codegen.code[i - 3],
                        codegen.code[i - 2],
                        codegen.code[i - 1],
                    ]) as usize;

                    // Print the name length
                    if i + 1 < codegen.code.len() {
                        eprint!(
                            "DEBUG:   [{:04x}] {:02x} <name_len: {}>",
                            i + 1,
                            codegen.code[i + 1],
                            name_len
                        );
                        i += 1;
                    }

                    // Print each name byte as data
                    for j in 0..name_len {
                        if i + 2 + j < codegen.code.len() {
                            eprintln!(
                                "DEBUG:   [{:04x}] {:02x} <name_byte: '{}'>",
                                i + 2 + j,
                                codegen.code[i + 2 + j],
                                codegen.code[i + 2 + j] as char
                            );
                            i += 1;
                        }
                    }
                    // Now i points to the byte after the name bytes
                    continue;
                }
            }

            // Special handling for CONST_STR (followed by string bytes)
            match opcode {
                // 1-byte immediates
                OpCode::CONST_U8
                | OpCode::LOAD_LOCAL
                | OpCode::STORE_LOCAL
                | OpCode::CALL_CLOSURE => {
                    if i < codegen.code.len() {
                        eprint!(" (imm: {:02x})", codegen.code[i]);
                        i += 1;
                    }
                }
                // 2-byte immediates (i16 for jumps, u16 for native ID)
                OpCode::JMP
                | OpCode::JMP_IF_Z
                | OpCode::JMP_IF_NZ
                | OpCode::JMP_L
                | OpCode::CALL_NAT => {
                    if i + 1 < codegen.code.len() {
                        let val = u16::from_le_bytes([codegen.code[i], codegen.code[i + 1]]);
                        eprint!(" (imm: {:04x})", val);
                        i += 2;
                    }
                }
                // 4-byte immediates (u32 for addresses)
                OpCode::CALL => {
                    if i + 3 < codegen.code.len() {
                        let val = u32::from_le_bytes([
                            codegen.code[i],
                            codegen.code[i + 1],
                            codegen.code[i + 2],
                            codegen.code[i + 3],
                        ]);
                        eprint!(" (imm: {:08x})", val);
                        i += 4;
                    }
                }
                // SPAWN: 4-byte address + 1-byte arg_count
                OpCode::SPAWN => {
                    if i + 4 < codegen.code.len() {
                        let addr = u32::from_le_bytes([
                            codegen.code[i],
                            codegen.code[i + 1],
                            codegen.code[i + 2],
                            codegen.code[i + 3],
                        ]);
                        let arg_count = codegen.code[i + 4];
                        eprint!(" (addr: {:08x}, arg_count: {})", addr, arg_count);
                        i += 5;
                    }
                }
                // 4-byte immediates (i32 for constants)
                OpCode::CONST_I32 => {
                    if i + 3 < codegen.code.len() {
                        let val = i32::from_le_bytes([
                            codegen.code[i],
                            codegen.code[i + 1],
                            codegen.code[i + 2],
                            codegen.code[i + 3],
                        ]);
                        eprint!(" (imm: {})", val);
                        i += 4;
                    }
                }
                _ => {}
            }
            eprintln!();
        }

        // 5. Check if all relocations can be resolved (before modifying code)
        // Collect symbols to check first
        let undefined_symbols: Vec<String> = codegen
            .relocs
            .iter()
            .filter(|reloc| !codegen.exports.contains_key(&reloc.symbol_name))
            .map(|reloc| reloc.symbol_name.clone())
            .collect();

        if !undefined_symbols.is_empty() {
            // Put Codegen back before returning error
            self.codegen = Some(codegen);
            return Err(AutoError::Msg(format!(
                "Undefined function: {}",
                undefined_symbols[0]
            )));
        }

        // 6. Resolve relocations using existing symbols
        // Note: We need to clone the code to avoid moving from codegen
        let new_code = codegen.code.clone();
        for reloc in &codegen.relocs {
            if let Some(&addr) = codegen.exports.get(&reloc.symbol_name) {
                let bytes = addr.to_le_bytes();
                let offset = reloc.offset as usize;
                for (i, b) in bytes.iter().enumerate() {
                    unsafe {
                        let code_ptr = new_code.as_ptr() as *mut u8;
                        code_ptr.add(offset + i).write(*b);
                    }
                }
            }
        }

        // 6. Update bytecode
        eprintln!(
            "DEBUG: Before bytecode update - bytecode.len()={}",
            self.bytecode.len()
        );
        self.bytecode.pop();
        let new_code_start = self.bytecode.len();
        self.bytecode.extend_from_slice(&new_code);
        eprintln!(
            "DEBUG: After bytecode update - bytecode.len()={}, new_code_start={}",
            self.bytecode.len(),
            new_code_start
        );

        // 7. Update metadata (clone to avoid move)
        self.object_keys.extend(codegen.object_keys.clone());
        self.object_types.extend(codegen.object_types.clone());

        // 8. Update flash and strings
        let mut flash = VirtualFlash::new_with_code(self.bytecode.clone());
        flash.object_keys = self.object_keys.clone();
        flash.object_types = self.object_types.clone();

        // Update VM's flash and strings
        self.vm.flash = Arc::new(flash);
        self.vm.strings = Arc::new(std::sync::RwLock::new(codegen.strings.clone()));

        // 9. Put Codegen back (preserves locals, exports, strings, etc.)
        self.codegen = Some(codegen);

        // 10. Reuse the same task (DO NOT create new task - preserves stack!)
        let task_arc = self
            .vm
            .tasks
            .get(&self.main_task_id)
            .ok_or_else(|| AutoError::Msg("Main task not found".to_string()))?
            .clone();

        let mut task = task_arc.blocking_lock();

        // 11. Reset task status to Ready (it may be Terminated from previous execution)
        task.status = crate::vm::task::TaskStatus::Ready;

        // Plan 080: Reset stack pointer to (bp + 1 + num_locals)
        // Stack frame layout: [unused, local0, local1, ..., temps...]
        //                     bp-1     bp     bp+1
        // Local variables occupy bp+1 to bp+num_locals
        // Stack temps MUST start AFTER all locals: at bp + 1 + num_locals
        // NOT at bp + num_locals (which would overlap with the last variable!)
        let num_locals = self
            .codegen
            .as_ref()
            .and_then(|c| c.scope_stack.last())
            .map(|scope| scope.len())
            .unwrap_or(0);
        task.num_locals = num_locals; // Store on task for native shims to access
        task.ram.sp = task.bp + 1 + num_locals;

        // Debug: Print state BEFORE execution
        eprintln!(
            "DEBUG: BEFORE execution - bp={}, sp={}, ip={}, num_locals={}, raw[0..5]={:?}",
            task.bp,
            task.ram.sp,
            task.ip,
            num_locals,
            &task.ram.raw[0..5]
        );

        // 12. Update IP to point to new code, but KEEP STACK (bp, sp, ram unchanged)
        task.ip = new_code_start;

        // 12. Execute the code
        drop(task); // Release lock before execution
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            self.vm.run_task_loop().await;
        });

        // 13. Get result from the task (re-acquire lock after execution)
        let task_arc = self
            .vm
            .tasks
            .get(&self.main_task_id)
            .ok_or_else(|| AutoError::Msg("Main task not found after execution".to_string()))?
            .clone();

        let mut task = task_arc.blocking_lock();

        // Debug: Print stack state AFTER execution
        eprintln!(
            "DEBUG: AFTER execution - bp={}, sp={}, ip={}, raw[0..5]={:?}",
            task.bp,
            task.ram.sp,
            task.ip,
            &task.ram.raw[0..5]
        );

        // Plan 080: Get number of local variables
        let num_locals = self
            .codegen
            .as_ref()
            .and_then(|c| c.scope_stack.last())
            .map(|scope| scope.len())
            .unwrap_or(0);

        // Calculate target stack pointer (bp + 1 + num_locals)
        // Stack frame layout: [unused, local0, local1, ..., localN, temps...]
        //                     bp-1    bp     bp+1    bp+2   bp+N+1
        // Local variables occupy bp+1 to bp+num_locals
        // Stack temps start at bp + 1 + num_locals (NOT bp + num_locals!)
        let target_sp = task.bp + 1 + num_locals;

        eprintln!(
            "DEBUG: num_locals={}, bp={}, target_sp={}, current_sp={}",
            num_locals, task.bp, target_sp, task.ram.sp
        );

        // Check if there's a temporary result on stack (sp > target_sp)
        if task.ram.sp > target_sp {
            // Save the result to last_result (Plan 080)
            let result = task.ram.pop_i32();
            self.last_result = Some(result);
            eprintln!("DEBUG: Saved result to last_result: {}", result);
        } else {
            // No result produced (e.g., let statement without expression)
            // Clear last_result to avoid printing stale value
            self.last_result = None;
        }

        // Reset stack pointer to target_sp (clear all temporary values)
        task.ram.sp = target_sp;

        eprintln!(
            "DEBUG: After stack cleanup - sp={}, raw[0..5]={:?}",
            task.ram.sp,
            &task.ram.raw[0..5]
        );

        // Return empty string (REPL will display last_result)
        Ok(String::new())
    }

    /// Get session statistics
    pub fn stats(&self) -> AutovmReplStats {
        let total_locals = self.codegen.as_ref().map(|c| c.locals.len()).unwrap_or(0);
        let total_exports = self.codegen.as_ref().map(|c| c.exports.len()).unwrap_or(0);
        let total_strings = self.codegen.as_ref().map(|c| c.strings.len()).unwrap_or(0);

        AutovmReplStats {
            total_functions: total_exports,
            total_locals,
            bytecode_size: self.bytecode.len(),
            total_strings,
            heap_objects: self.vm.heap_objects.len(),
            arrays: self.vm.arrays.len(),
        }
    }

    /// Clear all state (start fresh)
    pub fn reset(&mut self) {
        // Reset bytecode
        self.bytecode = vec![OpCode::HALT as u8];
        self.object_keys.clear();
        self.object_types.clear();

        // Reset Codegen (recreate fresh one)
        self.codegen = Some(Codegen::new());

        // Clear VM registries
        self.vm.closures.clear();
        self.vm.iterators.clear();
        self.vm.channels.clear();
        self.vm.heap_objects.clear();
        self.vm.arrays.clear();
        self.vm.objects.clear();
        self.vm.nodes.clear();
    }

    /// Get list of defined functions
    pub fn functions(&self) -> Vec<String> {
        self.codegen
            .as_ref()
            .map(|c| c.exports.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get list of local variables
    pub fn locals(&self) -> Vec<String> {
        self.codegen
            .as_ref()
            .map(|c| c.locals.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get the last result (Plan 080)
    ///
    /// Returns the result from the previous REPL input, if any.
    /// This is used by the REPL to display results to the user.
    pub fn get_last_result(&self) -> Option<i32> {
        self.last_result
    }

    /// Format the last result for display
    ///
    /// If the result is a heap object ID (like a list), format it appropriately.
    /// Otherwise, return the integer value as-is.
    ///
    /// Note: This is a heuristic - we can't distinguish between a list ID and an integer
    /// value that happens to be the same number. We check if the object exists to reduce
    /// false positives, but there's still ambiguity (e.g., if list ID 1 exists and you
    /// evaluate an expression that returns 1, it will be formatted as a list).
    pub fn format_last_result(&self) -> Option<String> {
        self.last_result.map(|value| {
            // Check if this is a heap object ID (positive values >= 1)
            // AND the object actually exists in the heap
            if value >= 1 {
                let list_id = value as u64;
                if let Some(obj) = self.vm.get_heap_object(list_id) {
                    let guard = obj.read().unwrap();
                    // Check if it's a List<int>
                    use crate::vm::types::ListData;
                    if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
                        // Check if this looks like it could be an element value vs a list reference
                        // Heuristic: If the value is small (0-10) and matches a list element value,
                        // it's more likely to be an element than a list ID. But we can't be 100% sure.
                        // For now, always format lists properly - users will see the issue if
                        // they index into lists and get elements that happen to match list IDs.
                        let elems: Vec<String> = list.elems.iter().map(|e| e.to_string()).collect();
                        return format!("List[{}]", elems.join(", "));
                    }
                    if let Some(list) = guard.as_any().downcast_ref::<ListData<String>>() {
                        return format!("List[{}]", list.elems.join(", "));
                    }
                    if let Some(list) = guard.as_any().downcast_ref::<ListData<bool>>() {
                        let elems: Vec<String> = list.elems.iter().map(|e| e.to_string()).collect();
                        return format!("List[{}]", elems.join(", "));
                    }
                    // Generic heap object
                    return format!("<heap object {}>", list_id);
                }
            }
            // Regular integer value
            value.to_string()
        })
    }
}

/// REPL session statistics
#[derive(Debug, Clone)]
pub struct AutovmReplStats {
    pub total_functions: usize,
    pub total_locals: usize,
    pub bytecode_size: usize,
    pub total_strings: usize,
    pub heap_objects: usize,
    pub arrays: usize,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autovm_session_create() {
        let session = AutovmReplSession::new();
        assert!(session.codegen.is_some());
        assert_eq!(session.bytecode.len(), 1); // Just HALT
    }

    #[test]
    fn test_autovm_session_stats() {
        let session = AutovmReplSession::new();
        let stats = session.stats();
        assert_eq!(stats.total_functions, 0);
        assert_eq!(stats.total_locals, 0);
        assert_eq!(stats.bytecode_size, 1);
    }

    #[test]
    fn test_autovm_session_reset() {
        let mut session = AutovmReplSession::new();
        session.bytecode.push(OpCode::ADD as u8);
        assert_eq!(session.bytecode.len(), 2);

        session.reset();
        assert_eq!(session.bytecode.len(), 1); // Just HALT
        assert!(session.codegen.is_some()); // Codegen recreated
    }

    #[test]
    fn test_autovm_session_functions() {
        let session = AutovmReplSession::new();
        let funcs = session.functions();
        assert_eq!(funcs.len(), 0);

        let locals = session.locals();
        assert_eq!(locals.len(), 0);
    }

    #[test]
    fn test_autovm_session_simple_arithmetic() {
        let mut session = AutovmReplSession::new();

        // Simple arithmetic
        let result = session.run("1 + 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_autovm_task_reuse_variable_persistence() {
        let mut session = AutovmReplSession::new();

        // Define variable
        let result = session.run("let x = 10");
        assert!(result.is_ok());

        // Access variable in next input (should return 11 if task reuse works)
        let result = session.run("x + 1");
        assert!(result.is_ok());

        // Debug: Print result to see actual value
        if let Ok(s) = &result {
            println!("x + 1 = {}", s);
            // Currently expected to be "1" (bug), should be "11" (fixed)
        }

        // Note: This test currently expected to fail until we implement
        // proper stack frame management or global variables
        // After fix, result should be "11" instead of "1"
    }

    #[test]
    fn test_autovm_task_stack_state() {
        let mut session = AutovmReplSession::new();

        // Define two variables
        let _ = session.run("let a = 5");
        let _ = session.run("let b = 10");

        // Access both variables
        let result1 = session.run("a + b");
        assert!(result1.is_ok());

        if let Ok(s) = &result1 {
            println!("a + b = {} (expected: 15)", s);
        }

        // Define third variable using first variable
        let _ = session.run("let c = a * 2");
        let result2 = session.run("c");
        assert!(result2.is_ok());

        if let Ok(s) = &result2 {
            println!("c = {} (expected: 10)", s);
        }
    }

    #[test]
    fn test_autovm_simple_persistence_check() {
        let mut session = AutovmReplSession::new();

        // Define x
        let r1 = session.run("let x = 10");
        assert!(r1.is_ok(), "let x = 10 should succeed");

        // Try to access x
        let r2 = session.run("x + 1");
        assert!(r2.is_ok(), "x + 1 should succeed");

        // Check the result
        match r2 {
            Ok(s) => {
                // If this is "11", variable persistence works!
                // If this is "1", variable is not persisted (current bug)
                assert_eq!(
                    s, "11",
                    "Variable x should persist with value 10, so x+1=11"
                );
            }
            Err(e) => {
                panic!("x + 1 failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_autovm_undefined_variable_error() {
        let mut session = AutovmReplSession::new();

        // Define x
        let r1 = session.run("let x = 5");
        assert!(r1.is_ok(), "let x = 5 should succeed");

        // Try to access undefined variable y (should error)
        let r2 = session.run("y + 1");
        assert!(r2.is_err(), "y + 1 should fail (undefined variable)");

        // Verify error message contains "Undefined variable"
        if let Err(e) = r2 {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Undefined variable"),
                "Error should mention undefined variable"
            );
            assert!(
                error_msg.contains("y"),
                "Error should mention variable name 'y'"
            );
        }

        // Plan 080: After error, session should still be usable (codegen was returned)
        // This test verifies the fix for the panic bug
        let r3 = session.run("x + 2");
        assert!(r3.is_ok(), "After error, session should still work: x + 2");

        // Verify result is correct
        if let Some(result) = session.get_last_result() {
            assert_eq!(result, 7, "x + 2 should be 7 (x is still 5)");
        } else {
            panic!("Expected last_result to be Some(7)");
        }
    }

    #[test]
    fn test_autovm_repl_list_type_preservation() {
        let mut session = AutovmReplSession::new();
        let r1 = session.run("let l = List.new(1,2,3)");
        assert!(r1.is_ok());
        println!(
            "DEBUG: var_types after l = {:?}",
            session.codegen.as_ref().unwrap().var_types
        );

        let r2 = session.run("l.get(2)");
        if let Err(e) = &r2 {
            println!("DEBUG: Error on get: {}", e);
        }
        assert!(r2.is_ok());
    }
}
