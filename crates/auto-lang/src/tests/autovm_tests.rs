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
#[ignore]
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
#[ignore]
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
#[ignore]
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

// ===== Level 2: Conditional Tests =====

#[test]
fn test_if() {
    // Test simple if statement
    let code = r#"
fn main() int {
    if true { 1 } else { 2 }
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "If should work: {:?}", result);
    assert_eq!(result.unwrap(), "1", "Should return 1");
}

#[test]
fn test_if_else() {
    // Test if with else branch
    let code = r#"
fn main() int {
    if false { 1 } else { 2 }
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "If else should work: {:?}", result);
    assert_eq!(result.unwrap(), "2", "Should return 2");
}

#[test]
fn test_if_else_if() {
    // Test if with else-if chain
    let code = r#"
fn main() int {
    if false { 1 } else if false { 2 } else { 3 }
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "If else if should work: {:?}", result);
    assert_eq!(result.unwrap(), "3", "Should return 3");
}

#[test]
fn test_comp() {
    // Test comparison operator
    let code = r#"
fn main() int {
    1 < 2
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Comparison should work: {:?}", result);
    assert_eq!(result.unwrap(), "1", "Should return 1");
}

#[test]
fn test_comp_false() {
    // Test comparison that returns false
    let code = r#"
fn main() int {
    2 < 1
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Comparison false should work: {:?}", result);
    assert_eq!(result.unwrap(), "0", "Should return 0");
}

#[test]
fn test_eq() {
    // Test equality operator
    let code = r#"
fn main() int {
    1 == 1
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Equality should work: {:?}", result);
    assert_eq!(result.unwrap(), "1", "Should return 1");
}

#[test]
fn test_eq_false() {
    // Test inequality
    let code = r#"
fn main() int {
    1 == 2
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Inequality should work: {:?}", result);
    assert_eq!(result.unwrap(), "0", "Should return 0");
}

// ===== Level3: Variable and Assignment Tests =====

#[test]
fn test_var() {
    // Test variable declaration and use
    let code = r#"
fn main() int {
    let a = 1
    a + 2
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Var should work: {:?}", result);
    assert_eq!(result.unwrap(), "3", "Should return 3");
}

#[test]
fn test_var_assign() {
    // Test variable reassignment with var
    let code = r#"
fn main() int {
    var a = 1
    a = 2
    a
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Var assign should work: {:?}", result);
    assert_eq!(result.unwrap(), "2", "Should return 2");
}

#[test]
fn test_var_mut() {
    // Test var with mutation
    let code = r#"
fn main() int {
    var x = 1
    x = 10
    x + 1
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Var mut should work: {:?}", result);
    assert_eq!(result.unwrap(), "11", "Should return 11");
}

#[test]
fn test_let() {
    // Test let binding
    let code = r#"
fn main() int {
    let x = 41
    x
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Let should work: {:?}", result);
    assert_eq!(result.unwrap(), "41", "Should return 41");
}

#[test]
fn test_var_if() {
    // Test variable in if expression
    let code = r#"
fn main() int {
    var x = if true { 1 } else { 2 }
    x + 1
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Var if should work: {:?}", result);
    assert_eq!(result.unwrap(), "2", "Should return 2");
}

#[test]
fn test_if_var() {
    // Test if with variable condition
    let code = r#"
fn main() int {
    var a = 10
    if a > 10 { a + 1 } else { a - 1 }
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "If var should work: {:?}", result);
    assert_eq!(result.unwrap(), "9", "Should return 9");
}

#[test]
fn test_compound_assignment_add() {
    // Test compound assignment +=
    let code = r#"
fn main() int {
    var a = 1
    a += 1
    a
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "+= should work: {:?}", result);
    assert_eq!(result.unwrap(), "2", "Should return 2");
}

#[test]
fn test_compound_assignment_sub() {
    // Test compound assignment -=
    let code = r#"
fn main() int {
    var a = 10
    a -= 3
    a
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "-= should work: {:?}", result);
    assert_eq!(result.unwrap(), "7", "Should return 7");
}

#[test]
fn test_compound_assignment_mul() {
    // Test compound assignment *=
    let code = r#"
fn main() int {
    var a = 5
    a *= 3
    a
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "*= should work: {:?}", result);
    assert_eq!(result.unwrap(), "15", "Should return 15");
}

#[test]
fn test_compound_assignment_div() {
    // Test compound assignment /=
    let code = r#"
fn main() int {
    var a = 20
    a /= 4
    a
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "/= should work: {:?}", result);
    assert_eq!(result.unwrap(), "5", "Should return 5");
}

#[test]
fn test_compound_assignment_chained() {
    // Test chained compound assignments
    let code = r#"
fn main() int {
    var a = 1
    a += 1
    a += 2
    a += 3
    a
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Chained += should work: {:?}", result);
    assert_eq!(result.unwrap(), "7", "Should return 7");
}

// ===== Level 4: Array and Object Tests =====

#[test]
fn test_array() {
    // Test array literal
    let code = r#"
fn main() int {
    [1, 2, 3]
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Array should work: {:?}", result);
    assert_eq!(result.unwrap(), "[1, 2, 3]", "Should return [1, 2, 3]");
}

#[test]
fn test_array_element() {
    // Test array element access
    let code = r#"
fn main() int {
    var a = [1, 2, 3]
    a[0]
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Array element should work: {:?}", result);
    assert_eq!(result.unwrap(), "1", "Should return 1");
}

#[test]
fn test_array_element_1() {
    // Test array element access at index 1
    let code = r#"
fn main() int {
    var a = [1, 2, 3]
    a[1]
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Array element 1 should work: {:?}", result);
    assert_eq!(result.unwrap(), "2", "Should return 2");
}

#[test]
fn test_array_element_2() {
    // Test array element access at index 2
    let code = r#"
fn main() int {
    var a = [1, 2, 3]
    a[2]
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Array element 2 should work: {:?}", result);
    assert_eq!(result.unwrap(), "3", "Should return 3");
}

#[test]
fn test_object() {
    // Test object literal
    let code = r#"
fn main() int {
    var a = { name: "auto", age: 18 }
    a.age
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Object should work: {:?}", result);
    assert_eq!(result.unwrap(), "18", "Should return 18");
}

#[test]
fn test_object_name() {
    // Test object field access for string field
    let code = r#"
fn main() int {
    var a = { name: "auto", age: 18 }
    a.name
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Object name should work: {:?}", result);
    assert_eq!(result.unwrap(), "auto", "Should return auto");
}

#[test]
fn test_object_name_len() {
    // Test object string field method call
    let code = r#"
fn main() int {
    var a = { name: "auto", age: 18 }
    a.name.len()
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Object name should work: {:?}", result);
    assert_eq!(result.unwrap(), "4", "Should return 4");
}

#[test]
fn test_nested_object() {
    // Test nested object access
    let code = r#"
fn main() int {
    var obj = { inner: { x: 10, y: 20 } }
    obj.inner.x
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Nested object should work: {:?}", result);
    assert_eq!(result.unwrap(), "10", "Should return 10");
}

#[test]
fn test_nested_object_y() {
    // Test nested object y field
    let code = r#"
fn main() int {
    var obj = { inner: { x: 10, y: 20 } }
    obj.inner.y
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Nested object y should work: {:?}", result);
    assert_eq!(result.unwrap(), "20", "Should return 20");
}

// ===== Level 5: Function Tests =====

#[test]
fn test_fn() {
    // Test simple function call
    let code = r#"
fn add(a int, b int) int {
    a + b
}

fn main() int {
    add(12, 2)
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Function call should work: {:?}", result);
    assert_eq!(result.unwrap(), "14", "Should return 14");
}

#[test]
fn test_fn_with_args() {
    // Test function with named arguments
    let code = r#"
fn add(a int, b int) int {
    a + b
}

fn main() int {
    add(a: 12, b: 2)
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Function with args should work: {:?}", result);
    assert_eq!(result.unwrap(), "14", "Should return 14");
}

#[test]
fn test_fn_multiple() {
    // Test multiple function calls
    let code = r#"
fn add(a int, b int) int {
    a + b
}

fn main() int {
    add(add(1, 2), add(3, 4))
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Multiple function calls should work: {:?}", result);
    assert_eq!(result.unwrap(), "10", "Should return 10");
}

#[test]
fn test_fn_in_expression() {
    // Test function call in expression
    let code = r#"
fn add(a int, b int) int {
    a + b
}

fn main() int {
    10 + add(5, 3)
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Function in expression should work: {:?}", result);
    assert_eq!(result.unwrap(), "18", "Should return 18");
}

#[test]
fn test_fn_nested() {
    // Test nested function calls
    let code = r#"
fn add(a int, b int) int {
    a + b
}

fn mul(a int, b int) int {
    a * b
}

fn main() int {
    add(mul(2, 3), mul(4, 5))
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Nested function calls should work: {:?}", result);
    assert_eq!(result.unwrap(), "26", "Should return 26");
}

#[test]
fn test_fn_with_local_var() {
    // Test function with local variable
    let code = r#"
fn double(a int) int {
    let x = a + a
    x
}

fn main() int {
    double(5)
}
"#;
    let result = run_autovm(code);
    assert!(result.is_ok(), "Function with local var should work: {:?}", result);
    assert_eq!(result.unwrap(), "10", "Should return 10");
}
