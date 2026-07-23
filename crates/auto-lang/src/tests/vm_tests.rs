// VM and Evaluator Integration Tests
// Most tests have been migrated to file-based tests in test/vm/ (Plan 179).
// This file only contains tests that cannot be expressed as file-based tests:
// - Direct bytecode tests
// - AST inspection tests
// - Config mode / parser-level tests
// - #[ignore] tests
use crate::config::AutoConfig;
use crate::{ast, run};

// ============================================================================
// Direct Bytecode Tests
// ============================================================================

#[test]
fn test_vm_ret_constant() {
    // Direct bytecode test: FN_PROLOG(0,0), CONST_I32(42), RET(0)
    use crate::vm::opcode::OpCode;
    use crate::vm::engine::AutoVM;
    use crate::vm::virt_memory::VirtualFlash;

    let bytecode = vec![
        OpCode::FN_PROLOG as u8, 0, 0,      // FN_PROLOG with n_args=0
        OpCode::CONST_I32 as u8, 42, 0, 0, 0,  // CONST_I32(42)
        OpCode::RET as u8, 0,                   // RET with n_args=0
    ];

    let flash = VirtualFlash::new_with_code(bytecode);
    let vm = AutoVM::new(flash, 1024);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let task_id = vm.spawn_task(0, 1024);
    rt.block_on(async { vm.run_task_loop().await; });

    if let Some(task_arc) = vm.tasks.get(&task_id).map(|r| r.value().clone()) {
        let mut task = task_arc.blocking_lock();
        let result = task.ram.pop_i32();
        assert_eq!(result, 42, "Should return 42");
    }
}

#[test]
fn test_vm_const_i32_add() {
    // Direct bytecode test: FN_PROLOG(0,0), CONST_I32(10), CONST_I32(20), ADD, RET(0)
    use crate::vm::opcode::OpCode;
    use crate::vm::engine::AutoVM;
    use crate::vm::virt_memory::VirtualFlash;

    let bytecode = vec![
        OpCode::FN_PROLOG as u8, 0, 0,      // FN_PROLOG
        OpCode::CONST_I32 as u8, 10, 0, 0, 0,   // 10
        OpCode::CONST_I32 as u8, 20, 0, 0, 0,  // 20
        OpCode::ADD as u8,                        // 10 + 20
        OpCode::RET as u8, 0,                   // RET
    ];

    let flash = VirtualFlash::new_with_code(bytecode);
    let vm = AutoVM::new(flash, 1024);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let task_id = vm.spawn_task(0, 1024);
    rt.block_on(async { vm.run_task_loop().await; });

    if let Some(task_arc) = vm.tasks.get(&task_id).map(|r| r.value().clone()) {
        let mut task = task_arc.blocking_lock();
        let result = task.ram.pop_i32();
        assert_eq!(result, 30, "Should return 30");
    }
}

// ============================================================================
// AST Inspection Tests
// ============================================================================

#[test]
fn test_vm_annotation_in_ext() {
    let code = r#"
    type File {
        path str

        static fn open(path str) File;

        fn read_text() str;

        fn close();
    }

    ext File {
        _handle uint64

        #[vm]
        pub static fn open(path str) File

        #[vm]
        pub fn read_text() str

        #[vm]
        pub fn close()
    }
    "#;

    let mut parser = crate::Parser::from(code);
    let result = parser.parse();

    assert!(result.is_ok(), "Parsing should succeed: {:?}", result.err());

    let ast = result.unwrap();

    // Find the TypeDecl for File
    let file_type_decl = ast.stmts.iter().find_map(|stmt| {
        if let ast::Stmt::TypeDecl(decl) = stmt {
            if decl.name == "File" {
                Some(decl)
            } else {
                None
            }
        } else {
            None
        }
    });

    assert!(file_type_decl.is_some(), "File TypeDecl should exist");

    let file_decl = file_type_decl.unwrap();

    // Check that methods were merged from ext block
    assert_eq!(file_decl.methods.len(), 3, "Should have 3 methods");

    // Check that methods are VmFunction (from ext block)
    let open_method = &file_decl.methods[0];
    assert_eq!(open_method.name, "open");
    assert!(
        matches!(open_method.kind, crate::ast::FnKind::VmFunction),
        "open method should be VmFunction from ext block"
    );

    let read_text_method = &file_decl.methods[1];
    assert_eq!(read_text_method.name, "read_text");
    assert!(
        matches!(read_text_method.kind, crate::ast::FnKind::VmFunction),
        "read_text method should be VmFunction from ext block"
    );

    let close_method = &file_decl.methods[2];
    assert_eq!(close_method.name, "close");
    assert!(
        matches!(close_method.kind, crate::ast::FnKind::VmFunction),
        "close method should be VmFunction from ext block"
    );
}

#[test]
fn test_function_body_parsing() {
    let code = r#"
    fn test() int {
        42
    }
    "#;

    let mut parser = crate::Parser::from(code);
    let result = parser.parse();

    assert!(result.is_ok(), "Parsing should succeed: {:?}", result.err());

    let ast = result.unwrap();

    // Find the function
    let fn_decl = ast.stmts.iter().find_map(|stmt| {
        if let ast::Stmt::Fn(fn_decl) = stmt {
            if fn_decl.name == "test" {
                Some(fn_decl)
            } else {
                None
            }
        } else {
            None
        }
    });

    assert!(fn_decl.is_some(), "test function should exist");

    let test_fn = fn_decl.unwrap();

    // Check that the function is marked as Function (not VmFunction)
    assert!(
        matches!(test_fn.kind, crate::ast::FnKind::Function),
        "Function should be FnKind::Function, got {:?}",
        test_fn.kind
    );

    // Check that the function body has statements
    assert_eq!(
        test_fn.body.stmts.len(),
        1,
        "Function body should have 1 statement"
    );

    // The statement should be an expression statement containing Int(42)
    if let ast::Stmt::Expr(expr) = &test_fn.body.stmts[0] {
        if let ast::Expr::Int(val) = expr {
            assert_eq!(*val, 42);
        } else {
            panic!("Expected Expr::Int(42), got {:?}", expr);
        }
    } else {
        panic!("Expected Stmt::Expr, got {:?}", &test_fn.body.stmts[0]);
    }
}

// ============================================================================
// Config Mode / Parser-Level Tests
// ============================================================================

#[test]
fn test_nodes() {
    let code = r#"
col {
    text (text: "Hello, World!") {
        class: "text-2xl font-bold"
    }
    class: "w-full h-full justify-center items-center bg-white"
}
"#;
    let result = run(code);
    assert!(result.is_ok(), "Nodes should work: {:?}", result);
}

#[test]
fn test_node_arg_ident() {
    // Node with named arg using variable substitution: lib (id: myname) {}
    let code = r#"
            var myname = "Xiaoming"
            lib (id: myname) {}
        "#;
    let result = run(code);
    assert!(result.is_ok(), "Node with ident arg should parse, got: {:?}", result);
}

#[test]
fn test_node_newline() {
    // the token sequence [')', '\n', '{'] should be treated as compile error
    // because it would be ambiguous:
    // 1. it might be a node statement with '{' written at the next line
    // 2. it might be a node statement without body, then followed by an object or a block.
    // Note: use 'app' instead of 'dep' because 'dep' is now a reserved keyword
    let code = r#"
            app("x")
            {
                x: 1
                y: 2
            }
        "#;

    let config = AutoConfig::new(code);
    assert!(config.is_err());

    // this should pass, it is a normal node
    let code = r#"
            app("x") {
                x: 1
                y: 2
            }
        "#;

    let config = AutoConfig::new(code);
    assert!(config.is_ok());
}

// ============================================================================
// u8 Arithmetic (not yet in file-based tests)
// ============================================================================

#[test]
fn test_add_u8() {
    let code = r#"
            var a = 1u8
            var b = 2u8
            a + b
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "3");
}

// ============================================================================
// Plan 348 D1: Recursive enum + is-pattern payload binding
// ============================================================================
// Regression guard for the two claims in Plan 348 Bug D1:
//   1. A recursive enum variant (`node(Tree, Tree)`) must parse and run.
//      The enum name is only registered AFTER parse_enum_body returns, but
//      lookup_type() falls back to Type::User for unregistered names, so
//      parsing never actually fails. This test pins that behaviour.
//   2. `is`-pattern arms must bind payload variables for user-defined enums
//      (not just the built-in Ok/Err/Option destructuring). The variant must
//      be registered in generic_registry during Stmt::EnumDecl codegen.
// If either behaviour regresses, this test fails.

#[test]
fn test_plan348_d1_recursive_enum_is_pattern_binding() {
    let code = r#"
tag Tree {
    leaf(int)
    node(Tree, Tree)
}

fn sum_tree(t Tree) int {
    is t {
        Tree.leaf(v) -> v
        Tree.node(l, r) -> sum_tree(l) + sum_tree(r)
    }
}

fn main() {
    var t = Tree.node(Tree.leaf(4), Tree.node(Tree.leaf(1), Tree.leaf(2)))
    print(sum_tree(t).to(str))
    sum_tree(t)
}
"#;
    let (result, stdout) = crate::run_with_capture(code).unwrap();
    assert_eq!(stdout, "7\n");
    assert_eq!(result, "7");
}

// Recursive enum whose variant carries a collection payload (the shape
// serde_json needs for Value::Array(List<Value>)). Verifies that generic
// payload types containing the enum itself parse and that the bound payload
// is usable in the branch body.
#[test]
fn test_plan348_d1_recursive_enum_collection_payload() {
    let code = r#"
tag Value {
    nul
    num(int)
    arr(List<Value>)
}

fn describe(v Value) str {
    is v {
        Value.nul -> "n"
        Value.num(x) -> "num:" + x.to(str)
        Value.arr(items) -> "arr:" + items.len().to(str)
    }
}

fn main() {
    var v = Value.arr([Value.num(42), Value.nul, Value.num(7)])
    print(describe(v))
    print(describe(Value.num(99)))
    describe(v)
}
"#;
    let (result, stdout) = crate::run_with_capture(code).unwrap();
    assert_eq!(stdout, "arr:3\nnum:99\n");
    assert_eq!(result, "arr:3");
}

// ============================================================================
// Ignored Tests (kept for future reference)
// ============================================================================

#[test]
#[ignore = "TODO: capture stdout"]
fn test_range_print() {
    let code = r#"
print(1..5)
"#;
    let result = run(code);
    assert!(result.is_ok(), "Range print should work: {:?}", result);
}

#[test]
#[ignore = "Indexed for loops with arrays not supported yet"]
fn test_for_array() {
    let code = r#"
var a = [10, 20, 30]
var sum = 0
for i in 0..3 {
    sum = sum + a[i]
}
sum
"#;
    let result = run(code);
    assert!(result.is_ok(), "For array should work: {:?}", result);
    assert_eq!(result.unwrap(), "60");
}

#[test]
#[ignore = "Grid expression removed"]
fn test_grid() {
    let code = r#"
grid {
    col {
        text "Hello"
        text "World"
    }
}
"#;
    let result = run(code);
    assert!(result.is_ok(), "Grid should work: {:?}", result);
}

#[test]
#[ignore]
fn test_view_types() {
    // Plan 083: view keyword for borrow semantics
    let code = r#"
let s = "hello"
let v = s.view
v
"#;
    let result = run(code);
    assert!(result.is_ok(), "View should work: {:?}", result);
    assert_eq!(result.unwrap(), "hello");
}

#[test]
#[ignore = "TODO: lib x {..} should define local var x (sugar for var x = lib(id: \"x\") {...})"]
fn test_node_store() {
    let code = r#"
            lib x {
                at: "src/x"
            }
            x.at
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "src/x");
}

#[test]
#[ignore = "TODO: dynamic node types (root, etc.) need parser flag support; also nested node query (atom.body.p[1].content) needs implementation"]
fn test_atom_query() {
    let code = r#"
            var atom = root {
                header "This is header"
                body {
                    p "This is a paragraph"
                    p "This is another paragraph"
                }
            }
            atom.body.p[1].content
        "#;
    let result = run(code).unwrap();
    assert_eq!(result, "This is another paragraph");
}

#[test]
fn test_for_loop_outer_var_assignment_no_overflow() {
    // Regression test: hash = hash + 1 inside for loop should not stack overflow
    let code = r#"
fn simple_hash() {
    var hash = 5381
    for i in 0..5 {
        hash = hash + 1
    }
}

fn main() {
    simple_hash()
    print("hello")
}
"#;
    let result = run(code);
    assert!(result.is_ok(), "for loop outer var assignment should work: {:?}", result);
}

#[test]
fn test_for_loop_outer_var_on_small_stack() {
    // Regression test: for loop with outer var assignment must not stack overflow
    // even on small stacks (1MB, the Windows default main thread stack size).
    use std::thread;
    let code = r#"
fn simple_hash() {
    var hash = 5381
    for i in 0..5 {
        hash = hash + 1
    }
}

fn main() {
    simple_hash()
    print("hello")
}
"#.to_string();

    let handle = thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(move || crate::run(&code))
        .expect("Failed to spawn thread");

    match handle.join() {
        Ok(Ok(_)) => {},
        Ok(Err(e)) => panic!("run() failed on 1MB stack: {:?}", e),
        Err(_) => panic!("thread panicked on 1MB stack (stack overflow?)"),
    }
}

#[test]
#[ignore = "str_slice type removed"]
fn test_str_slice_type_lookup() {
    let code = r#"
let s = "hello"
let v = s.view
v
"#;
    let result = run(code);
    assert!(result.is_ok(), "str_slice type lookup should work: {:?}", result);
}
#[test]
// Iterator is not recognized as a type name in codegen, so Iterator.next(it)
// returns a string instead of int. Needs proper Iterator type support.
fn test_str_bytes_iterator() {
    let code = r#"
fn main() {
    let s = "AB"
    let it = s.bytes()
    let v1 = Iterator.next(it).?(0)
    let v2 = Iterator.next(it).?(0)
    let v3 = Iterator.next(it).?(0)
    v1 + v2 + v3
}
"#;
    let result = crate::run(code).unwrap();
    // 65 + 66 + -1 = 130
    assert_eq!(result, "130");
}

#[test]
fn test_relet() {
    let code = r#"
        let a = 1
        let a = 2
        a
    "#;
    let result = crate::run(code).unwrap();
    assert_eq!(result, "2");
}