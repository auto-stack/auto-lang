// AutoVM Integration Tests
// These tests verify that AutoVM bytecode execution works correctly
// Starting with basic arithmetic, then progressing to generic types

use crate::{ast, compile, run_autovm};
use crate::error::AutoResult;
use crate::vm::opcode::OpCode;
use crate::vm::engine::AutoVM;
use crate::vm::virt_memory::VirtualFlash;
use std::io::Write;

// ===== Helper Functions for Direct Bytecode Testing =====

/// Create bytecode from an array of bytes
fn create_bytecode(bytes: &[u8]) -> Vec<u8> {
    bytes.to_vec()
}

/// Run raw bytecode directly on the VM (bypassing parser/codegen)
/// This is useful for testing specific opcode sequences
fn run_bytecode(bytecode: &[u8]) -> AutoResult<String> {
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    // Create virtual flash with the bytecode
    let flash = VirtualFlash::new_with_code(bytecode.to_vec());

    // Create VM
    let mut vm = AutoVM::new(flash, 1024);

    // Create tokio runtime for async execution
    let rt = Runtime::new()?;

    // Execute from address 0
    let task_id = vm.spawn_task(0, 1024);

    // Run the task loop
    rt.block_on(async {
        vm.run_task_loop().await;
    });

    // Get result from task stack
    if let Some(task_arc) = vm.tasks.get(&task_id).map(|r| r.value().clone()) {
        let mut task = task_arc.blocking_lock();

        if task.ram.sp == 0 {
            return Ok("0".to_string());
        }

        let result = task.ram.pop_i32();
        Ok(format!("{}", result))
    } else {
        Err(crate::error::AutoError::Msg(
            "Task not found after execution".to_string()
        ))
    }
}

// ===== Basic Arithmetic Tests =====

#[test]
fn test_basic_add() {
    // Test simplest case: 1 + 1 = 2
    let code = r#"
fn main() int {
    1 + 1
}
"#;

    let result = run_autovm(code);
    assert!(result.is_ok(), "Basic addition should work: {:?}", result);
    assert_eq!(result.unwrap(), "2", "Should return 2");
}

#[test]
fn test_simple_int() {
    // Test simple integer return
    let code = r#"
fn main() int {
    42
}
"#;

    let result = run_autovm(code);
    assert!(result.is_ok(), "Simple int return should work");
    assert_eq!(result.unwrap(), "42", "Should return 42");
}

#[test]
fn test_two_ints() {
    // Test two integer variables
    let code = r#"
fn main() int {
    let a = 10000
    let b = 20000
    a + b
}
"#;

    let result = run_autovm(code);
    assert!(result.is_ok(), "Two ints should work: {:?}", result);
    assert_eq!(result.unwrap(), "30000", "Should return 30000");
}

#[test]
fn test_arithmetic() {
    // Test operator precedence: 1+2*3 = 7
    let code = r#"
fn main() int {
    1 + 2 * 3
}
"#;

    let result = run_autovm(code);
    assert!(result.is_ok(), "Arithmetic should work");
    assert_eq!(result.unwrap(), "7", "Should return 7 (1+2*3=7)");
}

#[test]
fn test_unary() {
    // Test unary operator: -2*3 = -6
    let code = r#"
fn main() int {
    -2 * 3
}
"#;

    let result = run_autovm(code);
    assert!(result.is_ok(), "Unary should work");
    assert_eq!(result.unwrap(), "-6", "Should return -6");
}

#[test]
fn test_group() {
    // Test parentheses: (1+2)*3 = 9
    let code = r#"
fn main() int {
    (1 + 2) * 3
}
"#;

    let result = run_autovm(code);
    assert!(result.is_ok(), "Grouping should work");
    assert_eq!(result.unwrap(), "9", "Should return 9");
}

#[test]
fn test_var_arithmetic() {
    // Test variable with arithmetic: var a = 12312; a * 10 = 123120
    let code = r#"
fn main() int {
    let a = 12312
    a * 10
}
"#;

    let result = run_autovm(code);
    assert!(result.is_ok(), "Var arithmetic should work");
    assert_eq!(result.unwrap(), "123120", "Should return 123120");
}

// ===== Direct Bytecode Tests (Bypassing codegen) =====

#[test]
fn test_vm_ret_constant() {
    // Test: FN_PROLOG(0,0), CONST_I32(42), RET(0)
    // Expected: Returns 42
    let code = create_bytecode(&[
        OpCode::FN_PROLOG as u8, 0, 0,      // FN_PROLOG with n_args=0
        OpCode::CONST_I32 as u8, 42, 0, 0, 0,  // CONST_I32(42)
        OpCode::RET as u8, 0,                   // RET with n_args=0
    ]);

    let result = run_bytecode(&code);
    assert!(result.is_ok(), "VM constant return should work: {:?}", result);
    assert_eq!(result.unwrap(), "42", "Should return 42");
}

#[test]
fn test_vm_const_i32_add() {
    // Test: FN_PROLOG(0,0), CONST_I32(10), CONST_I32(20), ADD, RET(0)
    // Expected: Returns 30 (10 + 20)
    let code = create_bytecode(&[
        OpCode::FN_PROLOG as u8, 0, 0,      // FN_PROLOG
        OpCode::CONST_I32 as u8, 10, 0, 0, 0,   // 10
        OpCode::CONST_I32 as u8, 20, 0, 0, 0,  // 20
        OpCode::ADD as u8,                        // 10 + 20
        OpCode::RET as u8, 0,                   // RET
    ]);

    let result = run_bytecode(&code);
    assert!(result.is_ok(), "Const I32 add should work: {:?}", result);
    assert_eq!(result.unwrap(), "30", "Should return 30");
}

// ===== Generic Type Field Access Tests =====

#[test]
fn test_generic_type_instantiation() {
    // Test basic generic type instantiation
    let code = r#"
type Point<T> {
    x T
    y T
}

fn main() int {
    let p = Point { x: 100, y: 200 }
    p.x + p.y
}
"#;

    // Compile and run
    let result = run_autovm(code);

    // Check result
    assert!(result.is_ok(), "Generic type instantiation should work");
    let output = result.unwrap();
    assert!(output.contains("Point"), "Type name should be in exports");
    assert!(output.contains("300"), "Should return sum of fields (100 + 200 = 300)");
}

#[test]
fn test_generic_field_access_x() {
    // Test accessing x field of generic type
    let code = r#"
type Point<T> {
    x T
    y T
}

fn main() int {
    let p = Point { x: 100, y: 200 }
    p.x
}
"#;

    let result = run_autovm(code);

    // Should return 100
    assert!(result.is_ok(), "Field access x should work: {:?}", result);
    assert_eq!(result.unwrap(), "100", "Should return field value");
}

#[test]
fn test_generic_field_access_y() {
    // Test accessing y field of generic type
    let code = r#"
type Point<T> {
    x T
    y T
}

fn main() int {
    let p = Point { x: 100, y: 200 }
    p.y
}
"#;

    let result = run_autovm(code);

    // Should return 200
    assert!(result.is_ok(), "Field access y should work: {:?}", result);
    assert_eq!(result.unwrap(), "200", "Should return field value");
}

#[test]
fn test_generic_field_addition() {
    // Test adding two field values
    let code = r#"
type Point<T> {
    x T
    y T
}

fn main() int {
    let p = Point { x: 100, y: 200 }
    p.x + p.y
}
"#;

    let result = run_autovm(code);

    // Should return 300
    assert!(result.is_ok(), "Field addition should work: {:?}", result);
    assert_eq!(result.unwrap(), "300", "Should return sum");
}
