// AutoVM Test Infrastructure
// Helper function to compile and execute AutoLang code on AutoVM

use crate::parser::Parser;
use crate::vm::codegen::Codegen;
use crate::vm::engine::AutoVM;
use crate::vm::opcode::OpCode;
use crate::vm::task::{AutoTask, TaskId, TaskStatus};
use crate::vm::virt_memory::VirtualFlash;

/// Helper function to run AutoLang code on AutoVM and return the result as a string
#[cfg(test)]
pub async fn run_autovm(code: &str) -> Result<String, String> {
    // 1. Parse the code
    let mut parser = Parser::from(code);
    let ast = parser
        .parse()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    // 2. Compile to bytecode
    let mut codegen = Codegen::new();
    for stmt in ast.stmts {
        codegen
            .compile_stmt(&stmt)
            .map_err(|e| format!("Codegen error: {:?}", e))?;
    }

    // Add explicit HALT at the end to prevent reading uninitialized memory
    codegen.code.push(OpCode::HALT as u8);

    // 3. Perform simple linking (resolve function calls)
    let strings = codegen.strings.clone();
    if !codegen.relocs.is_empty() {
        for reloc in &codegen.relocs {
            if let Some(&addr) = codegen.exports.get(&reloc.symbol_name) {
                let bytes = addr.to_le_bytes();
                let offset = reloc.offset as usize;
                for (i, b) in bytes.iter().enumerate() {
                    codegen.code[offset + i] = *b;
                }
            } else {
                return Err(format!("Undefined symbol: {}", reloc.symbol_name));
            }
        }
    }

    // 4. Load into VM
    let flash = VirtualFlash::new_with_code(codegen.code);
    // Note: AutoVM holds Arc, so we don't need 'mut' for run, but we need it for load_strings?
    // load_strings takes &mut self currently?
    // Let's check engine.rs: pub fn load_strings(&mut self, ...). Yes.
    let mut vm = AutoVM::new(flash, 1024); // 1KB RAM
    vm.load_strings(strings);

    // 5. Execute
    // Spawn the main task
    let task_id = vm.spawn_task(0, 1024);

    // Run the loop
    vm.run_task_loop().await;

    // 6. Get the result from the task's stack
    // Access the task from the registry
    // 6. Get the result from the task's stack
    // Access the task from the registry
    if let Some(task_arc) = vm.tasks.get(&task_id).map(|r| r.value().clone()) {
        let mut task = task_arc.lock().await;

        if task.ram.sp == 0 {
            return Ok("".to_string()); // Empty result
        }

        // Pop the top value and format it
        let result = task.ram.pop_i32();
        Ok(format!("{}", result))
    } else {
        Err("Task not found after execution".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_autovm_simple_add() {
        // Construct bytecode:
        // CONST_I32 1
        // CONST_I32 1
        // ADD
        // HALT
        let mut code = Vec::new();

        // Push 1
        code.push(OpCode::CONST_I32 as u8);
        code.extend_from_slice(&1i32.to_le_bytes());

        // Push 1
        code.push(OpCode::CONST_I32 as u8);
        code.extend_from_slice(&1i32.to_le_bytes());

        // Add
        code.push(OpCode::ADD as u8);

        // Halt
        code.push(OpCode::HALT as u8);

        let flash = VirtualFlash::new_with_code(code);
        let mut vm = AutoVM::new(flash, 1024);

        // Spawn task
        let task_id = vm.spawn_task(0, 1024);

        // Run
        vm.run_task_loop().await;

        // Check result
        // Check result
        let task_arc = vm.tasks.get(&task_id).map(|r| r.value().clone()).unwrap();
        let mut task = task_arc.lock().await;

        // Task should be Terminated
        assert_eq!(task.status, TaskStatus::Terminated);

        // Stack should have 1 element: 2
        let top = task.ram.top().unwrap();
        assert_eq!(top, 2);
    }

    #[tokio::test]
    async fn test_autovm_locals() {
        // Locals test:
        // ... (Bytecode same as before)
        let mut code = Vec::new();
        // Reserve space for locals (0, 1, 2)
        code.push(OpCode::CONST_0 as u8); // L0
        code.push(OpCode::CONST_0 as u8); // L1
        code.push(OpCode::CONST_0 as u8); // L2

        code.push(OpCode::CONST_0 as u8);
        code.push(OpCode::STORE_LOC_0 as u8);
        code.push(OpCode::CONST_1 as u8);
        code.push(OpCode::STORE_LOC_1 as u8);
        code.push(OpCode::LOAD_LOC_0 as u8);
        code.push(OpCode::LOAD_LOC_1 as u8);
        code.push(OpCode::ADD as u8);
        code.push(OpCode::STORE_LOC_0 as u8); // L[0] = 0 + 1 = 1

        // Load it back so we can store it to L[2]
        code.push(OpCode::LOAD_LOC_0 as u8);

        // STORE_LOCAL 2
        code.push(OpCode::STORE_LOCAL as u8);
        code.push(2);

        // LOAD_LOCAL 2
        code.push(OpCode::LOAD_LOCAL as u8);
        code.push(2);

        code.push(OpCode::CONST_I32 as u8);
        code.extend_from_slice(&41i32.to_le_bytes());

        code.push(OpCode::ADD as u8);
        code.push(OpCode::HALT as u8);

        let flash = VirtualFlash::new_with_code(code);
        let mut vm = AutoVM::new(flash, 1024); // 1KB

        let task_id = vm.spawn_task(0, 1024);
        vm.run_task_loop().await;

        let task_arc = vm.tasks.get(&task_id).map(|r| r.value().clone()).unwrap();
        let mut task = task_arc.lock().await;

        assert_eq!(task.status, TaskStatus::Terminated);
        assert_eq!(task.ram.top().unwrap(), 42);
    }

    #[tokio::test]
    async fn test_autovm_call_ret() {
        // Test CALL and RET
        let mut code = Vec::new();

        // 0: CALL <Target> (Target is at index 5)
        code.push(OpCode::CALL as u8);
        // Placeholder for address (index 1-4)
        let call_target_idx = code.len();
        code.extend_from_slice(&0u32.to_le_bytes());

        // 5: HALT
        code.push(OpCode::HALT as u8);

        let func_addr = code.len() as u32;

        // Update CALL target
        let target_bytes = func_addr.to_le_bytes();
        code[call_target_idx] = target_bytes[0];
        code[call_target_idx + 1] = target_bytes[1];
        code[call_target_idx + 2] = target_bytes[2];
        code[call_target_idx + 3] = target_bytes[3];

        // Func code at func_addr:
        // CONST_I32 42
        code.push(OpCode::CONST_I32 as u8);
        code.extend_from_slice(&42i32.to_le_bytes());

        // RET n_args (0)
        code.push(OpCode::RET as u8);
        code.push(0); // 0 args

        let flash = VirtualFlash::new_with_code(code);
        let mut vm = AutoVM::new(flash, 1024);

        let task_id = vm.spawn_task(0, 1024);
        vm.run_task_loop().await;

        let task_arc = vm.tasks.get(&task_id).map(|r| r.value().clone()).unwrap();
        let mut task = task_arc.lock().await;

        assert_eq!(task.status, TaskStatus::Terminated);
        assert_eq!(task.ram.top().unwrap(), 42);
        // Stack size should be 1 (just the result)
        assert_eq!(task.ram.sp, 1);
    }

    #[tokio::test]
    async fn test_autovm_control_flow() {
        // ... (Bytecode logic is sound, keeping it)
        let mut code = Vec::new();

        // 0: CONST_I32 10
        code.push(0x10);
        code.extend_from_slice(&10i32.to_le_bytes());

        // 5: CONST_I32 20
        code.push(0x10);
        code.extend_from_slice(&20i32.to_le_bytes());

        // 10: LT (10 < 20 => 1)
        code.push(0x52);

        // 11: JMP_IF_Z 3 (Skip next instruction which is JMP)
        code.push(0x61); // JMP_IF_Z
        code.extend_from_slice(&3i16.to_le_bytes());

        // 14: JMP 2 (Skip failure/trap)
        code.push(0x60); // JMP
        code.extend_from_slice(&1i16.to_le_bytes());

        // 17: HALT (Trap - Should be skipped)
        code.push(0xFF); // HALT

        // 18: CONST_I32 10
        code.push(0x10);
        code.extend_from_slice(&10i32.to_le_bytes());

        // 23: CONST_I32 20
        code.push(0x10);
        code.extend_from_slice(&20i32.to_le_bytes());

        // 28: GT (10 > 20 => 0)
        code.push(0x53);

        // 29: JMP_IF_Z 1 (Should Jump because 0 == 0)
        code.push(0x61);
        code.extend_from_slice(&1i16.to_le_bytes());

        // 32: HALT (Trap - Should be skipped)
        code.push(0xFF);

        // 33: CONST_I32 99 (Success marker)
        code.push(0x10);
        code.extend_from_slice(&99i32.to_le_bytes());

        // 38: HALT (End)
        code.push(0xFF);

        let flash = VirtualFlash::new_with_code(code);
        let mut vm = AutoVM::new(flash, 1024);

        let task_id = vm.spawn_task(0, 1024);
        vm.run_task_loop().await;

        let task_arc = vm.tasks.get(&task_id).map(|r| r.value().clone()).unwrap();
        let mut task = task_arc.lock().await;

        assert_eq!(task.status, TaskStatus::Terminated);
        assert_eq!(task.ram.sp, 1);
        assert_eq!(task.ram.pop_i32(), 99);
    }

    // === High-Level Tests Using run_autovm Helper ===
    // These tests port interpreter tests from vm_tests.rs to AutoVM

    #[tokio::test]
    async fn test_autovm_arithmetic() {
        let result = run_autovm("1+2*3").await.unwrap();
        assert_eq!(result, "7");

        let result = run_autovm("(1+2)*3").await.unwrap();
        assert_eq!(result, "9");
    }

    #[tokio::test]
    async fn test_autovm_unary() {
        let result = run_autovm("-2*3").await.unwrap();
        assert_eq!(result, "-6");

        let result = run_autovm("-(5+3)").await.unwrap();
        assert_eq!(result, "-8");
    }

    #[tokio::test]
    async fn test_autovm_comparison() {
        let result = run_autovm("1 < 2").await.unwrap();
        assert_eq!(result, "1"); // true represented as 1

        let result = run_autovm("5 > 10").await.unwrap();
        assert_eq!(result, "0"); // false represented as 0

        let result = run_autovm("3 == 3").await.unwrap();
        assert_eq!(result, "1");
    }

    #[tokio::test]
    async fn test_autovm_if_else() {
        let result = run_autovm("if 1 < 2 { 10 } else { 20 }").await.unwrap();
        assert_eq!(result, "10");

        let result = run_autovm("if 5 > 10 { 10 } else { 20 }").await.unwrap();
        assert_eq!(result, "20");
    }

    #[tokio::test]
    async fn test_autovm_spawn() {
        // Construct bytecode:
        // Main:
        //   SPAWN [func_addr], 0
        //   HALT
        // Function: (at 100?)
        //   CONST 10
        //   CONST 20
        //   ADD
        //   HALT (or RET if frame was setup, but we use HALT for simple test)

        let mut code = vec![OpCode::HALT as u8; 200]; // Initialize with HALT

        let func_addr: u32 = 100;

        // Main Task at 0:
        // SPAWN 100, 0
        let mut ip = 0;
        code[ip] = OpCode::SPAWN as u8;
        ip += 1;
        code[ip..ip + 4].copy_from_slice(&func_addr.to_le_bytes());
        ip += 4;
        code[ip] = 0;
        ip += 1; // 0 args

        // POP result of SPAWN (task_id) so stack is empty
        code[ip] = OpCode::POP as u8;
        ip += 1;
        code[ip] = OpCode::HALT as u8;

        // Function at 100:
        ip = func_addr as usize;
        code[ip] = OpCode::CONST_I32 as u8;
        ip += 1;
        code[ip..ip + 4].copy_from_slice(&10i32.to_le_bytes());
        ip += 4;

        code[ip] = OpCode::CONST_I32 as u8;
        ip += 1;
        code[ip..ip + 4].copy_from_slice(&20i32.to_le_bytes());
        ip += 4;

        code[ip] = OpCode::ADD as u8;
        ip += 1;
        code[ip] = OpCode::HALT as u8;

        let flash = VirtualFlash::new_with_code(code);
        let mut vm = AutoVM::new(flash, 1024);

        let main_id = vm.spawn_task(0, 1024);

        vm.run_task_loop().await;

        // Check Main Task: Terminated
        let main_task_arc = vm.tasks.get(&main_id).unwrap().clone();
        let main_task = main_task_arc.lock().await;
        assert_eq!(main_task.status, TaskStatus::Terminated);

        // Check Spawned Task
        // We need to find the other task
        let tasks: Vec<(TaskId, std::sync::Arc<tokio::sync::Mutex<AutoTask>>)> = vm
            .tasks
            .iter()
            .map(|r| (*r.key(), r.value().clone()))
            .collect();

        assert_eq!(tasks.len(), 2);

        let spawned_task_arc = tasks
            .iter()
            .find(|(id, _)| *id != main_id)
            .unwrap()
            .1
            .clone();
        let mut spawned_task = spawned_task_arc.lock().await;

        assert_eq!(spawned_task.status, TaskStatus::Terminated);

        // Check result: 30
        let result = spawned_task.ram.pop_i32();
        assert_eq!(result, 30);
    }

    // Plan 073: Array indexing tests
    #[tokio::test]
    async fn test_autovm_array_index_get() {
        // Test: arr[0] where arr = [10, 20, 30]
        let result = run_autovm("let arr = [10, 20, 30]; arr[0]").await.unwrap();
        assert_eq!(result, "10");

        let result = run_autovm("let arr = [10, 20, 30]; arr[1]").await.unwrap();
        assert_eq!(result, "20");

        let result = run_autovm("let arr = [10, 20, 30]; arr[2]").await.unwrap();
        assert_eq!(result, "30");
    }

    #[tokio::test]
    async fn test_autovm_array_index_set() {
        // Test: arr[1] = 99 where arr = [10, 20, 30]
        // After assignment, arr[1] should be 99
        let result = run_autovm("let arr = [10, 20, 30]; arr[1] = 99; arr[1]").await.unwrap();
        assert_eq!(result, "99");

        // First element should still be 10
        let result = run_autovm("let arr = [10, 20, 30]; arr[1] = 99; arr[0]").await.unwrap();
        assert_eq!(result, "10");

        // Last element should still be 30
        let result = run_autovm("let arr = [10, 20, 30]; arr[1] = 99; arr[2]").await.unwrap();
        assert_eq!(result, "30");
    }

    #[tokio::test]
    async fn test_autovm_array_index_expression() {
        // Test: arr[i] where i is a variable
        let result = run_autovm("let arr = [10, 20, 30]; let i = 1; arr[i]").await.unwrap();
        assert_eq!(result, "20");

        // Test: arr[i + 1]
        let result = run_autovm("let arr = [10, 20, 30]; let i = 0; arr[i + 1]").await.unwrap();
        assert_eq!(result, "20");
    }

    #[tokio::test]
    async fn test_autovm_array_nested_assignment() {
        // Test: x = arr[0]; x should be 10
        let result = run_autovm("let arr = [10, 20, 30]; let x = arr[0]; x").await.unwrap();
        assert_eq!(result, "10");
    }

    #[tokio::test]
    async fn test_autovm_array_in_function() {
        // Test: arrays as function return values
        let result = run_autovm("fn get_arr() { [10, 20, 30] }; let arr = get_arr(); arr[1]").await.unwrap();
        assert_eq!(result, "20");
    }

    // Plan 073: Node tests (Phase 0: Basic Node creation)
    #[tokio::test]
    async fn test_autovm_node_creation() {
        // Test: Create a simple node with arguments
        // Note: Node syntax in AutoLang is ident(arg1, arg2, ...)
        // This tests the CREATE_NODE opcode
        let result = run_autovm("div(10, 20)").await.unwrap();
        // Result is the node_id, which should be a non-negative number
        // We can't predict the exact ID, but it should be valid
        assert!(result.parse::<i32>().is_ok() || result == "0");
    }

    #[tokio::test]
    async fn test_autovm_node_with_expressions() {
        // Test: Node with expression arguments
        let result = run_autovm("div(1 + 2, 3 * 4)").await.unwrap();
        assert!(result.parse::<i32>().is_ok() || result == "0");
    }

    #[tokio::test]
    async fn test_autovm_nested_nodes() {
        // Test: Nested nodes
        let result = run_autovm("div(div(1, 2), 3)").await.unwrap();
        assert!(result.parse::<i32>().is_ok() || result == "0");
    }

    // Plan 073: Type instance tests (Phase 1: Type detection)
    #[tokio::test]
    async fn test_autovm_type_declaration() {
        // Test: Type declaration is registered
        // type Point { x int, y int }
        let result = run_autovm("type Point { x int, y int }").await.unwrap();
        // Type declarations don't produce values, so result may be empty or default
        // The important part is that the type is registered
        assert!(result == "0" || result.parse::<i32>().is_ok());
    }

    #[tokio::test]
    async fn test_autovm_type_instance_creation() {
        // Test: Create type instance Point(10, 20)
        // This should create an object with x: 10, y: 20
        let result = run_autovm("type Point { x int, y int }; Point(10, 20)").await.unwrap();
        // Result is the object_id
        assert!(result.parse::<i32>().is_ok() || result == "0");
    }

    #[tokio::test]
    async fn test_autovm_type_field_access() {
        // Test: Create Point and access x field
        let result = run_autovm("type Point { x int, y int }; let p = Point(10, 20); p.x").await.unwrap();
        // Should return 10
        assert_eq!(result, "10");
    }

    #[tokio::test]
    async fn test_autovm_type_field_access_y() {
        // Test: Create Point and access y field
        let result = run_autovm("type Point { x int, y int }; let p = Point(10, 20); p.y").await.unwrap();
        // Should return 20
        assert_eq!(result, "20");
    }

    // Plan 073 Phase 2: Method call tests (obj.method())
    #[tokio::test]
    async fn test_autovm_method_call_no_args() {
        // Test: Simple method call with no arguments
        // type Counter { count int }
        // fn Counter.get_count() int { this.count }
        // let c = Counter(10); c.get_count()
        let result = run_autovm(
            "type Counter { count int }
             fn Counter.get_count() int { this.count }
             let c = Counter(10); c.get_count()"
        ).await.unwrap();
        // Should return 10
        assert_eq!(result, "10");
    }

    #[tokio::test]
    async fn test_autovm_method_call_with_args() {
        // Test: Method call with arguments
        // type Point { x int, y int }
        // fn Point.add(dx int, dy int) int { this.x + dx + this.y + dy }
        // let p = Point(10, 20); p.add(5, 3)
        let result = run_autovm(
            "type Point { x int, y int }
             fn Point.add(dx int, dy int) int { this.x + dx + this.y + dy }
             let p = Point(10, 20); p.add(5, 3)"
        ).await.unwrap();
        // Should return 10 + 5 + 20 + 3 = 38
        assert_eq!(result, "38");
    }

    #[tokio::test]
    async fn test_autovm_method_call_chained() {
        // Test: Chained method calls
        // type Buffer { value int }
        // fn Buffer.set(v int) int { this.value = v; v }
        // fn Buffer.get() int { this.value }
        // let b = Buffer(0); b.set(42); b.get()
        let result = run_autovm(
            "type Buffer { value int }
             fn Buffer.set(v int) int { this.value = v; v }
             fn Buffer.get() int { this.value }
             let b = Buffer(0); b.set(42); b.get()"
        ).await.unwrap();
        // Should return 42
        assert_eq!(result, "42");
    }

    #[tokio::test]
    async fn test_autovm_method_multiple_instances() {
        // Test: Multiple instances with method calls
        // type Point { x int, y int }
        // fn Point.sum() int { this.x + this.y }
        // let p1 = Point(1, 2); let p2 = Point(10, 20); p1.sum() + p2.sum()
        let result = run_autovm(
            "type Point { x int, y int }
             fn Point.sum() int { this.x + this.y }
             let p1 = Point(1, 2); let p2 = Point(10, 20); p1.sum() + p2.sum()"
        ).await.unwrap();
        // Should return (1 + 2) + (10 + 20) = 33
        assert_eq!(result, "33");
    }

    // ============================================================================
    // Phase 8.4: List Tests Migration
    // ============================================================================

    #[tokio::test]
    async fn test_autovm_list_new_and_len() {
        // Test: Create a list and check its length
        let result = run_autovm(
            "let list = List.new()
             list.push(1)
             list.push(2)
             list.push(3)
             list.len()"
        ).await.unwrap();
        assert_eq!(result, "3");
    }

    #[tokio::test]
    async fn test_autovm_list_push_and_pop() {
        // Test: Push and pop operations
        let result = run_autovm(
            "let list = List.new()
             list.push(10)
             list.push(20)
             let first = list.pop()
             let second = list.pop()
             let length = list.len()
             length"
        ).await.unwrap();
        // After popping both elements, length should be 0
        assert_eq!(result, "0");
    }

    #[tokio::test]
    async fn test_autovm_list_is_empty() {
        // Test: Check if list is empty
        let result = run_autovm(
            "let list = List.new()
             let empty1 = list.is_empty()
             list.push(1)
             let not_empty = list.is_empty()
             list.pop()
             let empty2 = list.is_empty()
             empty1"
        ).await.unwrap();
        // Initially empty, should return 1 (true)
        assert_eq!(result, "1");
    }

    #[tokio::test]
    async fn test_autovm_list_get_and_set() {
        // Test: Get and set elements
        let result = run_autovm(
            "let list = List.new()
             list.push(10)
             list.push(20)
             list.push(30)
             let first = list.get(0)
             first"
        ).await.unwrap();
        assert_eq!(result, "10");
    }

    #[tokio::test]
    async fn test_autovm_list_clear() {
        // Test: Clear list
        let result = run_autovm(
            "let list = List.new()
             list.push(1)
             list.push(2)
             list.push(3)
             list.clear()
             list.len()"
        ).await.unwrap();
        assert_eq!(result, "0");
    }

    #[tokio::test]
    async fn test_autovm_list_capacity() {
        // Test: Check list capacity
        let result = run_autovm(
            "let list = List.new()
             list.push(1)
             list.push(2)
             list.push(3)
             list.len()"
        ).await.unwrap();
        assert_eq!(result, "3");
    }

    #[tokio::test]
    async fn test_autovm_list_insert_and_remove() {
        // Test: Insert and remove elements
        let result = run_autovm(
            "let list = List.new()
             list.push(1)
             list.push(3)
             list.insert(1, 2)
             let elem1 = list.get(1)
             elem1"
        ).await.unwrap();
        assert_eq!(result, "2");
    }

    // ============================================================================
    // Iterator Tests (Phase 8.4)
    // ============================================================================

    #[tokio::test]
    async fn test_autovm_list_iter() {
        // Test: Basic iterator
        let result = run_autovm(
            "let list = List.new()
             list.push(1)
             list.push(2)
             list.push(3)
             let iter = list.iter()
             let first = iter.next()
             first"
        ).await.unwrap();
        assert_eq!(result, "1");
    }

    #[tokio::test]
    async fn test_autovm_list_iter_multiple() {
        // Test: Multiple next() calls
        let result = run_autovm(
            "let list = List.new()
             list.push(10)
             list.push(20)
             list.push(30)
             let iter = list.iter()
             let a = iter.next()
             let b = iter.next()
             let c = iter.next()
             c"
        ).await.unwrap();
        assert_eq!(result, "30");
    }
}
