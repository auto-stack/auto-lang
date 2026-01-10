pub mod ast;
pub mod config;
pub mod error;
pub mod eval;
pub mod interp;
mod lexer;
pub mod libs;
pub mod maker;
pub mod parser;
pub mod repl;
pub mod scope;
pub mod token;
pub mod trans;
mod universe;
pub mod util;
pub mod vm;

#[cfg(test)]
mod vm_functions_test;

use crate::scope::Meta;
use crate::trans::c::CTrans;
pub use crate::universe::Universe;
use crate::{eval::EvalMode, trans::Sink};
use crate::{parser::Parser, trans::Trans};
use auto_val::{AutoPath, Obj, Value};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::error::{AutoError, AutoResult};

/// Global error limit for parser error recovery
static ERROR_LIMIT: AtomicUsize = AtomicUsize::new(20);

/// Set the global error limit for parser error recovery
///
/// This controls how many errors the parser will collect before aborting.
/// Default is 20.
pub fn set_error_limit(limit: usize) {
    ERROR_LIMIT.store(limit, Ordering::SeqCst);
}

/// Get the current global error limit
pub fn get_error_limit() -> usize {
    ERROR_LIMIT.load(Ordering::SeqCst)
}

pub fn run(code: &str) -> AutoResult<String> {
    let mut interpreter = interp::Interpreter::new();

    // Try to interpret, and attach source code if we get a syntax error
    let result = interpreter.interpret(code);

    match result {
        Ok(_) => {
            // Resolve any ValueRef in the result before converting to string
            let resolved = resolve_value_in_result(interpreter.result, &interpreter.scope);
            Ok(resolved.repr().to_string())
        }
        Err(err) => {
            // Attach source code to the error
            Err(crate::error::attach_source(
                err,
                "<input>".to_string(),
                code.to_string(),
            ))
        }
    }
}

/// Run code and collect all errors during parsing
///
/// This function enables error recovery to collect multiple syntax errors
/// instead of aborting on the first error.
pub fn run_with_errors(code: &str) -> AutoResult<String> {
    let mut interpreter = interp::Interpreter::new();

    // Enable error recovery
    interpreter.enable_error_recovery();

    let result = interpreter.interpret(code);

    match result {
        Ok(_) => {
            let resolved = resolve_value_in_result(interpreter.result, &interpreter.scope);
            Ok(resolved.repr().to_string())
        }
        Err(err) => {
            // Attach source code to the error
            Err(crate::error::attach_source(
                err,
                "<input>".to_string(),
                code.to_string(),
            ))
        }
    }
}

/// Helper: Resolve ValueRef to actual value (for test output)
fn resolve_value_in_result(
    value: Value,
    universe: &std::rc::Rc<std::cell::RefCell<Universe>>,
) -> Value {
    match value {
        Value::ValueRef(vid) => {
            if let Some(data) = universe.borrow().get_value(vid) {
                let borrowed_data = data.borrow();
                let data_clone = borrowed_data.clone();
                drop(borrowed_data);
                // Convert ValueData to Value, which will create nested ValueRefs
                let val = Value::from_data(data_clone);
                // Recursively resolve nested ValueRefs
                resolve_value_in_result(val, universe)
            } else {
                Value::Nil
            }
        }
        Value::Array(arr) => {
            let resolved_vals: Vec<Value> = arr
                .values
                .into_iter()
                .map(|v| resolve_value_in_result(v, universe))
                .collect();
            Value::Array(resolved_vals.into())
        }
        Value::Obj(obj) => {
            let mut new_obj = auto_val::Obj::new();
            for (k, v) in obj.iter() {
                let resolved_v = resolve_value_in_result(v.clone(), universe);
                new_obj.set(k.clone(), resolved_v);
            }
            Value::Obj(new_obj)
        }
        _ => value,
    }
}

pub fn run_with_scope(code: &str, scope: Universe) -> AutoResult<String> {
    let mut interpreter = interp::Interpreter::with_scope(scope);
    interpreter.interpret(code)?;
    // Resolve any ValueRef in the result before converting to string
    let resolved = resolve_value_in_result(interpreter.result, &interpreter.scope);
    Ok(resolved.repr().to_string())
}

pub fn parse(code: &str) -> AutoResult<ast::Code> {
    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code, scope.clone());
    parser.parse().map_err(|e| e.to_string().into())
}

pub fn parse_with_scope(code: &str, scope: Rc<RefCell<Universe>>) -> AutoResult<ast::Code> {
    let mut parser = Parser::new(code, scope.clone());
    parser.parse().map_err(|e| e.to_string().into())
}

pub fn interpret(code: &str) -> AutoResult<interp::Interpreter> {
    let mut interpreter = interp::Interpreter::new();
    interpreter.interpret(code)?;
    Ok(interpreter)
}

pub fn interpret_with_scope(code: &str, scope: Universe) -> AutoResult<interp::Interpreter> {
    let mut interpreter = interp::Interpreter::with_scope(scope);
    interpreter.interpret(code)?;
    Ok(interpreter)
}

pub fn run_file(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    let mut interpreter = interp::Interpreter::new();
    let result = interpreter.interpret(&code);

    match result {
        Ok(_) => {
            // Resolve any ValueRef in the result before converting to string
            let resolved = resolve_value_in_result(interpreter.result, &interpreter.scope);
            Ok(resolved.repr().to_string())
        }
        Err(err) => {
            // Attach source code to all error types for better error messages
            Err(crate::error::attach_source(err, path.to_string(), code))
        }
    }
}

pub fn interpret_file(path: &str) -> interp::Interpreter {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
        .unwrap();
    let mut interpreter = interp::Interpreter::new();
    interpreter.interpret(&code).unwrap();
    interpreter
}

// TODO: to be deprecated, use Interpreter::eval_template instead
pub fn eval_template(template: &str, scope: Universe) -> AutoResult<interp::Interpreter> {
    let mut interpreter = interp::Interpreter::with_scope(scope).with_eval_mode(EvalMode::TEMPLATE);
    let result = interpreter.eval_template(template)?;
    interpreter.result = result;
    Ok(interpreter)
}

pub fn eval_config_with_scope(
    code: &str,
    args: &Obj,
    mut scope: Universe,
) -> AutoResult<interp::Interpreter> {
    scope.define_global("root", Rc::new(Meta::Node(ast::Node::new("root"))));
    scope.set_args(args);
    let mut interpreter = interp::Interpreter::with_scope(scope).with_eval_mode(EvalMode::CONFIG);
    interpreter.interpret(code)?;
    Ok(interpreter)
}

pub fn eval_config(code: &str, args: &Obj) -> AutoResult<interp::Interpreter> {
    let mut scope = Universe::new();
    scope.define_global("root", Rc::new(Meta::Node(ast::Node::new("root"))));
    scope.set_args(args);
    let mut interpreter = interp::Interpreter::with_scope(scope).with_eval_mode(EvalMode::CONFIG);
    interpreter.interpret(code)?;
    Ok(interpreter)
}

pub fn trans_c(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
        .unwrap();

    let cname = path.replace(".at", ".c");

    let fname = AutoPath::new(path).filename();
    println!("trans_C fname: {}", fname);

    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code.as_str(), scope);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut sink = Sink::new(fname);
    let mut trans = CTrans::new(cname.clone().into());
    trans.set_scope(parser.scope.clone());
    trans.trans(ast, &mut sink)?;

    // convert sink to .c/.h files
    std::fs::write(&cname, sink.done()?)?;
    // write the header file
    let h_path = path.replace(".at", ".h");
    std::fs::write(Path::new(h_path.as_str()), sink.header)?;

    Ok(format!("[trans] {} -> {}", path, cname))
}

/// Transpile AutoLang file to Rust
pub fn trans_rust(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
        .unwrap();

    let rsname = path.replace(".at", ".rs");
    let fname = AutoPath::new(path).filename();

    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code.as_str(), scope);
    parser.set_dest(crate::parser::CompileDest::TransRust);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut sink = Sink::new(fname.clone());
    let mut trans = crate::trans::rust::RustTrans::new(fname);
    trans.set_scope(parser.scope.clone());
    trans.trans(ast, &mut sink)?;

    // Write Rust file
    std::fs::write(&rsname, sink.done()?)?;

    Ok(format!("[trans] {} -> {}", path, rsname))
}

#[cfg(test)]
mod tests {
    use crate::config::AutoConfig;

    use super::*;
    use auto_val::Value;

    #[test]
    fn test_uint() {
        let code = "1u+2u";
        let result = run(code).unwrap();
        assert_eq!(result, "3u");

        let code = "25u+123u";
        let result = run(code).unwrap();
        assert_eq!(result, "148u");
    }

    #[test]
    fn test_byte() {
        let code = "let a byte = 255; a";
        let result = run(code).unwrap();
        assert_eq!(result, "0xFF");

        // promote byte to int
        let code = "let a int = 0xFF; a";
        let result = run(code).unwrap();
        assert_eq!(result, "255");
    }

    #[test]
    fn test_arithmetic() {
        let code = "1+2*3";
        let result = run(code).unwrap();
        assert_eq!(result, "7");

        let code = "(2+3.5)*5";
        let result = run(code).unwrap();
        assert_eq!(result, "27.5");
    }

    #[test]
    fn test_unary() {
        let code = "-2*3";
        let result = run(code).unwrap();
        assert_eq!(result, "-6");
    }

    #[test]
    fn test_group() {
        let code = "(1+2)*3";
        let result = run(code).unwrap();
        assert_eq!(result, "9");
    }

    #[test]
    fn test_comp() {
        let code = "1 < 2";
        let result = run(code).unwrap();
        assert_eq!(result, "true");
    }

    #[test]
    fn test_var_assign() {
        let code = "var a = 1; a = 2; a";
        let result = run(code).unwrap();
        assert_eq!(result, "2");
    }

    #[test]
    fn test_var_arithmetic() {
        let code = "var a = 12312; a * 10";
        let result = run(code).unwrap();
        assert_eq!(result, "123120");
    }

    #[test]
    fn test_if() {
        let code = "if true { 1 } else { 2 }";
        let result = run(code).unwrap();
        assert_eq!(result, "1");
    }

    #[test]
    fn test_if_else() {
        let code = "if false { 1 } else { 2 }";
        let result = run(code).unwrap();
        assert_eq!(result, "2");
    }

    #[test]
    fn test_if_else_if() {
        let code = "if false { 1 } else if false { 2 } else { 3 }";
        let result = run(code).unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_var() {
        let code = "var a = 1; a+2";
        let result = run(code).unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_if_var() {
        let code = "var a = 10; if a > 10 { a+1 } else { a-1  }";
        let result = run(code).unwrap();
        assert_eq!(result, "9");
    }

    #[test]
    fn test_var_if() {
        let code = "var x = if true { 1 } else { 2 }; x+1";
        let result = run(code).unwrap();
        assert_eq!(result, "2");
    }

    #[test]
    fn test_var_mut() {
        let code = "var x = 1; x = 10; x+1";
        let result = run(code).unwrap();
        assert_eq!(result, "11");
    }

    #[test]
    fn teste_array() {
        let code = "[1, 2, 3]";
        let result = run(code).unwrap();
        assert_eq!(result, "[1, 2, 3]");

        let code = "var a = [1, 2, 3]; [a[0], a[1], a[2], a[-1], a[-2], a[-3]]";
        let result = run(code).unwrap();
        assert_eq!(result, "[1, 2, 3, 3, 2, 1]");
    }

    #[test]
    fn test_range() {
        let code = "1..5";
        let result = run(code).unwrap();
        assert_eq!(result, "1..5");
    }

    #[test]
    fn test_range_print() {
        let code = r#"for i in 0..10 { print(i) }"#;
        let result = run(code).unwrap();
        // TODO: capture stdout and assert
        assert_eq!(result, "");
    }

    #[test]
    fn test_range_eq() {
        let code = "var sum = 0; for i in 0..=10 { sum = sum + i }; sum";
        let result = run(code).unwrap();
        assert_eq!(result, "55");
    }

    #[test]
    fn test_for() {
        let code = "var sum = 0; for i in 0..10 { sum = sum + i }; sum";
        let result = run(code).unwrap();
        assert_eq!(result, "45");

        let code = "var arr = [1, 2, 3]; var sum = 0; for i, x in arr { sum = sum + x + i }; sum";
        let result = run(code).unwrap();
        assert_eq!(result, "9");
    }

    #[test]
    fn test_for_with_mid() {
        let code = r#"$ for i in 0..10 { `${i}`; mid{","} }"#;
        let scope = Universe::new();
        let result = eval_template(code, scope).unwrap();
        assert_eq!(result.result.repr(), "0,1,2,3,4,5,6,7,8,9");
    }

    #[test]
    fn test_is_stmt() {
        let code = r#"var x = 10; is x { 10 => {print("Here is 10!"); x} }"#;
        let result = run(code).unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_for_with_mid_and_newline() {
        let code = r#"
$ for i in 0..10 {
    ${i}${mid{","}}
$ }"#;
        let scope = Universe::new();
        let result = eval_template(code, scope).unwrap();
        let expected =
            "\n    0,\n    1,\n    2,\n    3,\n    4,\n    5,\n    6,\n    7,\n    8,\n    9";
        if let Value::Str(s) = result.result {
            assert_eq!(s, expected);
        } else {
            assert_eq!(result.result.to_string(), expected);
        }
    }

    #[test]
    fn test_fn() {
        let code = "fn add(a, b) { a + b }; add(12, 2)";
        let result = run(code).unwrap();
        assert_eq!(result, "14");

        let code = "fn add(a, b) { a + b }; add(a:1, b:2)";
        let result = run(code).unwrap();
        assert_eq!(result, "3");

        let code = "fn hi(s str) { print(s) }; hi(\"hello\")";
        let result = run(code).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_object() {
        let code = "var a = { name: \"auto\", age: 18 }; a.name";
        let result = run(code).unwrap();
        assert_eq!(result, "auto");

        let code = "var a = { name: \"auto\", age: 18 }; a.age";
        let result = run(code).unwrap();
        assert_eq!(result, "18");

        let code = "var a = { 1: 2, 3: 4 }; a.3";
        let result = run(code).unwrap();
        assert_eq!(result, "4");

        let code = "var a = { true: 2, false: 4 }; a.false";
        let result = run(code).unwrap();
        assert_eq!(result, "4");
    }

    #[test]
    fn test_array_of_objects() {
        let code = "[1, 2]";
        let result = run(code).unwrap();
        println!("{}", result);
        assert_eq!(result, "[1, 2]");
    }

    #[test]
    fn test_lambda() {
        let code = "var add = |a int, b int| a + b; add(1, 2)";
        let result = run(code).unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_lambda_with_named_params() {
        let code = "let sub = |a, b| a - b; sub(b:5, a:12)";
        let result = run(code).unwrap();
        assert_eq!(result, "7");
    }

    #[test]
    fn test_json() {
        let code = r#"
            var ServiceInfo = [
                { id: 0x10, name: "DiagnosticSessionControl",  desc: "诊断会话控制" },
                { id: 0x11, name: "EcuReset",  desc: "电控单元复位" },
                { id: 0x14, name: "ClearDiagnosticInformation",  desc: "清除诊断信息" },
            ]
            ServiceInfo[2].name
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "ClearDiagnosticInformation");
    }

    #[test]
    fn test_fstr() {
        let code = r#"var name = "auto"; f"hello $name, now!""#;
        let result = run(code).unwrap();
        assert_eq!(result, "hello auto, now!");
    }

    #[test]
    fn test_fstr_with_expr() {
        let code = r#"var a = 1; var b = 2; f"a + b = ${a+b}""#;
        let result = run(code).unwrap();
        assert_eq!(result, "a + b = 3");
    }

    #[test]
    fn test_fstr_with_addition() {
        let code = r#"`[${no_tx_msgs + 1}u]`"#;
        let mut scope = Universe::new();
        scope.set_global("no_tx_msgs", Value::Int(9));
        let result = run_with_scope(code, scope).unwrap();
        assert_eq!(result, "[10u]");
    }

    #[test]
    fn test_asn_upper() {
        let code = "var a = 1; if true { a = 2 }; a";
        let result = run(code).unwrap();
        assert_eq!(result, "2");
    }

    #[test]
    fn test_node_props() {
        let code = r#"
            parent("parent") {
                size: 10
                kid("kid1") {}
            }
        "#;
        let interp = eval_config(code, &Obj::EMPTY).unwrap();
        let config = interp.result.as_node();
        let parent = &config.nodes[0];
        let size = parent.get_prop_of("size").to_uint();
        assert_eq!(size, 10);
    }

    #[test]
    fn test_nodes() {
        let code = r#"center {
            text("Hello") {}
            button("OK") {
                onclick: || print("clicked")
            }
        }"#;
        let result = run(code).unwrap();
        println!("{}", result);
    }

    #[test]
    fn test_simple_template() {
        let code = r#"
            var title = "Students"
            var rows = [
                { name: "Alice", age: 20 }
                { name: "Bob", age: 21 }
                { name: "Charlie", age: 22 }
            ]
        "#;
        let data = interpret(code).unwrap();
        let scope = data.scope.take();
        let template = r#"
<h1>$title</h1>
<table>
$ for row in rows {
    <tr>
        <td>${row.name}</td>
        <td>${row.age}</td>
    </tr>
$ }
</table>"#;
        let interpreter = eval_template(template, scope).unwrap();
        assert_eq!(
            interpreter.result.repr(),
            r#"
<h1>Students</h1>
<table>
    <tr>
        <td>Alice</td>
        <td>20</td>
    </tr>
    <tr>
        <td>Bob</td>
        <td>21</td>
    </tr>
    <tr>
        <td>Charlie</td>
        <td>22</td>
    </tr>
</table>"#
        );
    }

    #[test]
    fn test_eval_template() {
        let code = r#"
        var rows = [
            { name: "Alice", age: 20 }
            { name: "Bob", age: 21 }
            { name: "Charlie", age: 22 }
        ]
        "#;
        let interpreter = interpret(code).unwrap();
        let scope = interpreter.scope.take();
        let template = r#"
$ for row in rows {
{
    name: ${row.name},
    age: ${row.age},
},
$ }
"#;
        let result = eval_template(template, scope).unwrap();
        let expected = r#"
{
    name: Alice,
    age: 20,
},
{
    name: Bob,
    age: 21,
},
{
    name: Charlie,
    age: 22,
},
"#;
        assert_eq!(result.result.repr(), expected);
    }

    #[test]
    fn test_for_loop_with_object() {
        let code = r#"
        var items = [
            { name: "Alice", age: 20 }
            { name: "Bob", age: 21 }
            { name: "Charlie", age: 22 }
        ]
        for item in items {
            print(f"Hi ${item.name}")
        }
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_eval_template_with_note() {
        let code = "#{x+1}";
        use crate::interp::Interpreter;
        let mut scope = Universe::new();
        scope.set_global("x", Value::Int(41));
        let mut interp = Interpreter::with_scope(scope);
        let result = interp.eval_template_with_note(code, '#').unwrap();
        assert_eq!(result.repr(), "42");
    }

    #[test]
    fn test_to_string() {
        let code = r#"1.str()"#;
        let result = run(code).unwrap();
        assert_eq!(result, "1");

        let code = r#""hello".upper()"#;
        let result = run(code).unwrap();
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_insert_global_fn() {
        fn myjoin(arg: &auto_val::Args) -> Value {
            Value::Str(
                arg.args
                    .iter()
                    .map(|v| v.to_astr())
                    .collect::<Vec<auto_val::AutoStr>>()
                    .join("::")
                    .into(),
            )
        }

        let mut scope = Universe::new();
        scope.add_global_fn("myjoin", myjoin);
        let code = "myjoin(1, 2, 3)";
        let result = run_with_scope(code, scope).unwrap();
        assert_eq!(result, "1::2::3");
    }

    // #[test]
    // fn test_ref() {
    //     let code = "var a = 1; var b = ref a; b";
    //     let result = run(code).unwrap();
    //     assert_eq!(result, "1");
    // }

    // #[test]
    // fn test_ref_modify() {
    //     let code = "var a = 1; var b = ref a; b = 2; [a, b]";
    //     let result = run(code).unwrap();
    //     assert_eq!(result, "[2, 2]");
    // }

    // #[test]
    // fn test_ref_array() {
    //     let code = "var a = [1, 2, 3]; var b = ref a; b = [4, 5, 6]; var c = {a: a, b: b}; c";
    //     let result = run(code).unwrap();
    //     assert_eq!(result, "{a: [4, 5, 6], b: [4, 5, 6]}");
    // }

    #[test]
    fn test_obj_set() {
        let code = "var a = { name: \"Alice\" }; a.name = \"Bob\"; a.name";
        let result = run(code).unwrap();
        assert_eq!(result, "Bob");
    }

    #[test]
    fn test_array_update() {
        let code = "var a = [1, 2, 3]; a[0] = 4; a";
        let result = run(code).unwrap();
        assert_eq!(result, "[4, 2, 3]");
    }

    #[test]
    fn test_let() {
        let code = "let x = 41; x";
        let result = run(code).unwrap();
        assert_eq!(result, "41");
    }

    #[test]
    fn test_let_asn() {
        let code = "let x = 41; x = 10; x";
        let result = run(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_mut() {
        let code = "let x = 41; mut x = 10; x";
        let result = run(code).unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_type_decl() {
        let code = "type Point { x int = 5; y int }; let p = Point(y: 2); p";
        let mut interpreter = interpret(code).unwrap();
        assert_eq!(interpreter.result.repr(), "Point{x: 5, y: 2}");

        let code = "p.x";
        let result = interpreter.eval(code);
        assert_eq!(result.repr(), "5");

        let code = "p.y";
        let result = interpreter.eval(code);
        assert_eq!(result.repr(), "2");
    }

    #[test]
    fn test_deep_type() {
        let code = "type A { x int; y int }; type B { a A; b int }";
        let mut interpreter = interpret(code).unwrap();
        let code = "var v = B(a: A(x:1, y:2), b:3); v.a.y";
        let result = interpreter.eval(code);
        assert_eq!(result.repr(), "2");
    }

    #[test]
    fn test_type_with_method() {
        let code = r#"type Point {
            x int
            y int

            fn absquare() int {
                x * x + y * y
            }
        }"#;
        let mut interpreter = interpret(code).unwrap();
        let code = "var p = Point(3, 4); p.absquare()";
        let result = interpreter.eval(code);
        assert_eq!(result.repr(), "25");
    }

    #[test]
    fn test_simple_block() {
        let code = r#"
        let a = 10;
        {
            let a = 20;
        }
        a
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_type_compose() {
        let code = r#"
        type Wing {
            fn fly() {print("flap!flap!")}
        }
        type Duck has Wing {
        }
        var wing = Wing()
        var duck = Duck()
        duck.fly()
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_last_block_or_object() {
        let code = r#"
        {
            a: 1
            b: 2
        }"#;

        let result = run(code).unwrap();
        assert_eq!(result, "{a: 1, b: 2}");

        let code = r#"
        if true {
            {a:1, b:2}
        }
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "{a: 1, b: 2}");
    }

    #[test]
    fn test_grid() {
        let code = r#"var g = grid(a:"first", b:"second", c:"third") {
            [1, 2, 3]
            [4, 5, 6]
            [7, 8, 9]
        }
        g
        "#;
        let result = run(code).unwrap();
        assert_eq!(
            result,
            r#"grid(a:"first",b:"second",c:"third",) {[1, 2, 3];[4, 5, 6];[7, 8, 9]}"#
        );
    }

    #[test]
    fn test_config() {
        let code = r#"
name: "hello"
version: "0.1.0"

exe hello {
    dir: "src"
    main: "main.c"
}"#;
        let interp = eval_config(code, &Obj::EMPTY).unwrap();
        let result = interp.result;
        assert_eq!(
            result.repr(),
            r#"root {name: "hello"; version: "0.1.0"; exe hello {dir: "src"; main: "main.c"; }; }"#
        );
    }

    #[test]
    fn test_config_with_node() {
        let code = r#"
        name: "hello"

        var dirs = ["a", "b", "c"]

        lib hello {
            for d in dirs {
                dir(d) {}
            }
        }
        "#;

        let conf = AutoConfig::new(code).unwrap();

        // TODO: should be `dir("a") {}` instead of `dir a {}`
        assert_eq!(
            conf.root.to_string(),
            r#"root {name: "hello"; lib hello {dir a {}; dir b {}; dir c {}; }; }"#
        );
    }

    #[test]
    fn test_std() {
        let code = r#"
use auto.math: square

square(15)
"#;
        let result = run(code).unwrap();
        assert_eq!(result, "225");
    }

    #[test]
    fn test_str_index() {
        let code = r#"let a = "hello"
        a[1]
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "'e'");
    }

    #[test]
    fn test_methods_in_template() {
        use super::ast::{Expr, Store, StoreKind, Type};
        let code = r#"<div>${name.upper()}</div>"#;
        let mut scope = Universe::new();
        scope.set_global("name", Value::str("hello"));
        scope.define(
            "name",
            Rc::new(Meta::Store(Store {
                kind: StoreKind::Var,
                name: "name".into(),
                ty: Type::Str("hello".len()),
                expr: Expr::Str("hello".into()),
            })),
        );
        let result = eval_template(code, scope).unwrap();
        assert_eq!(result.result.repr(), "<div>HELLO</div>");
    }

    #[test]
    fn test_view_types() {
        let code = r#"
            type Hello {
                text str = "hello"
                fn view() {
                    label(text) {}
                }
            }
            var hello = Hello(text:"hallo")
            hello.view()
        "#;
        // TODO: implement view types
        let res = run(code).unwrap();
        println!("{}", res);
        assert!(true);
    }

    #[test]
    fn test_access_fields_in_method() {
        let code = r#"
            type Login {
                username str
                status str = ""

                fn on(ev str) {
                    // status = `Login ${username} ...`
                    var a = status
                }
            }"#;
        let result = run(code).unwrap();
        println!("{}", result);
        assert!(true);
    }

    #[test]
    fn test_int_enums() {
        let code = r#"
            enum Color {
                Red = 1
                Green = 2
                Blue = 3
            }
            Color.Red
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "1");
    }

    #[test]
    fn test_if_with_bool() {
        let code = r#"
            var succ = true
            if succ {
                print("I won!")
                "succ"
            } else {
                print("You failed!")
                "failed"
            }
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "succ");
    }

    #[test]
    fn test_if_in_array() {
        let code = r#"
            var is_lse = false
            var is_rh = true
            ["osal", if is_lse {"EB"}, if is_rh {"al"}]
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, r#"["osal", "al"]"#)
    }

    #[test]
    fn test_node_newline() {
        // the token sequence [')', '\n', '{'] should be treated as compile error
        // because it would be ambiguous:
        // 1. it might be a node statement with '{' written at the next line
        // 2. it might be a node statement without body, then followed by an object or a block.
        let code = r#"
            dep("x")
            {
                x: 1
                y: 2
            }
        "#;

        let config = AutoConfig::new(code);
        assert!(config.is_err());

        // this should pass, it is a normal node
        let code = r#"
            dep("x") {
                x: 1
                y: 2
            }
        "#;

        let config = AutoConfig::new(code);
        assert!(config.is_ok());

        // this should also pass, this is a call followed by an object
        let code = r#"
            dep("x")

            {
                x: 1
                y: 2
            }
        "#;

        let config = AutoConfig::new(code);
        assert!(config.is_ok());
    }

    #[test]
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
    fn test_node_arg_ident() {
        let code = r#"
            var myname = "Xiaoming"
            lib (myname) {}
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "lib Xiaoming {}");
    }

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

    #[test]
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

    // ===== Integration Tests for Nested Mutation (Phase 2) =====

    #[test]
    fn test_object_field_mutation() {
        let code = r#"
            mut obj = {x: 10, y: 20}
            obj.x = 30
            obj.x
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "30");
    }

    #[test]
    fn test_array_element_mutation() {
        let code = r#"
            mut arr = [1, 2, 3]
            arr[0] = 10
            arr[0]
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_multiple_field_mutations() {
        let code = r#"
            mut obj = {x: 10, y: 20, z: 30}
            obj.x = 100
            obj.y = 200
            obj.z = 300
            obj.x + obj.y + obj.z
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "600");
    }

    #[test]
    fn test_multiple_array_mutations() {
        let code = r#"
            mut arr = [1, 2, 3]
            arr[0] = 10
            arr[1] = 20
            arr[2] = 30
            arr[0] + arr[1] + arr[2]
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "60");
    }

    #[test]
    fn test_type_field_mutation() {
        let code = r#"
            type Point {
                x int
                y int
            }
            mut p = Point {x: 10, y: 20}
            p.x = 30
            p.x
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "30");
    }

    // ===== Level 1: Basic Nested Mutations (2-Level Depth) =====

    #[test]
    fn test_nested_object_field_mutation() {
        let code = r#"
            mut obj = { inner: { x: 10, y: 20 } }
            obj.inner.x = 30
            obj.inner.x
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "30");
    }

    #[test]
    fn test_array_element_field_mutation() {
        let code = r#"
            mut arr = [{x: 1}, {x: 2}, {x: 3}]
            arr[0].x = 10
            arr[0].x
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_object_array_element_mutation() {
        let code = r#"
            mut obj = { items: [1, 2, 3] }
            obj.items[0] = 10
            obj.items[0]
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_nested_array_element_mutation() {
        let code = r#"
            mut matrix = [[1, 2], [3, 4]]
            matrix[0][1] = 20
            matrix[0][1]
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "20");
    }

    // ===== Level 2: Type Instance Nested Fields =====

    #[test]
    fn test_type_instance_nested_field_mutation() {
        let code = r#"
            type Inner { x int }
            type Outer { inner Inner }
            mut obj = Outer(inner: Inner(x: 10))
            obj.inner.x = 20
            obj.inner.x
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "20");
    }

    // ===== Level 3: Complex Nested Mutations (3+ Level Depth) =====

    #[test]
    fn test_three_level_object_nesting() {
        let code = r#"
            mut obj = { level1: { level2: { value: 100 } } }
            obj.level1.level2.value = 200
            obj.level1.level2.value
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "200");
    }

    #[test]
    fn test_deep_array_of_objects_mutation() {
        let code = r#"
            mut data = [
                { info: { name: "Alice", age: 20 } },
                { info: { name: "Bob", age: 21 } }
            ]
            data[0].info.age = 25
            data[0].info.age
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "25");
    }

    // ===== Level 4: Structure Preservation =====

    #[test]
    fn test_nested_structure_preservation() {
        let code = r#"
            mut obj = { a: { x: 1, y: 2 }, b: { x: 3, y: 4 } }
            obj.a.x = 10
            [obj.a.y, obj.b.x, obj.b.y]
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "[2, 3, 4]");
    }

    // ===== Level 5: Error Cases =====

    #[test]
    fn test_nested_out_of_bounds_index() {
        let code = r#"
            mut obj = { items: [1, 2, 3] }
            obj.items[10] = 100
        "#;
        let result = run(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_invalid_field_access() {
        let code = r#"
            mut obj = { inner: { x: 10 } }
            obj.inner.nonexistent = 20
        "#;
        let result = run(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_type_mismatch() {
        let code = r#"
            mut obj = { items: [1, 2, 3] }
            obj.items.invalid_field = 10
        "#;
        let result = run(code);
        assert!(result.is_err());
    }

    // ===== Type Instance Creation Diagnostic Tests =====

    #[test]
    fn test_simple_nested_type_instance_creation() {
        let code = r#"
            type Inner { x int }
            type Outer { inner Inner }
            var obj = Outer(inner: Inner(x: 10))
            obj.inner.x
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_nested_type_instance_field_access() {
        let code = r#"
            type Inner { x int }
            type Outer { inner Inner }
            var obj = Outer(inner: Inner(x: 10))
            obj.inner.x
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_type_instance_field_value() {
        let code = r#"
            type Inner { x int }
            type Outer { inner Inner }
            var inner = Inner(x: 10)
            var obj = Outer(inner: inner)
            obj.inner.x
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_nested_type_instance_positional_args() {
        let code = r#"
            type Inner { x int }
            type Outer { inner Inner }
            var v = Outer(inner: Inner(x: 10))
            v.inner.x
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, "10");
    }

    #[test]
    fn test_config_with_deep_data() {
        let code = r#"let dirs = ["a" , "b", "c"]
for d in dirs {
    dir(id: d) {
        at: d
    }
}
"#;
        let interp = eval_config(code, &auto_val::Obj::new()).unwrap();
        assert_eq!(
            interp.result.repr(),
            r#"root {dir a {at: "a"; }; dir b {at: "b"; }; dir c {at: "c"; }; }"#
        );
    }
}
