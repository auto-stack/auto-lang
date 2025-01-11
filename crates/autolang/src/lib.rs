mod token;
mod lexer;
pub mod ast;
mod parser;
pub mod eval;
pub mod scope;
pub mod transpiler;
pub mod repl;
pub mod libs;
pub mod util;
pub mod interp;
pub mod config;

use std::rc::Rc;
use std::cell::RefCell;
use crate::eval::EvalMode;
use crate::scope::Universe;

pub fn run(code: &str) -> Result<String, String> {
    let scope = Rc::new(RefCell::new(Universe::new()));
    let ast = parser::parse(code, &mut scope.borrow_mut())?;
    let mut evaler = eval::Evaler::new(scope);
    let result = evaler.eval(&ast);
    Ok(result.repr())
}

pub fn run_with_scope(code: &str, scope: Universe) -> Result<String, String> {
    let scope = Rc::new(RefCell::new(scope));
    let ast = parser::parse(code, &mut scope.borrow_mut())?;
    let mut evaler = eval::Evaler::new(scope);
    let result = evaler.eval(&ast);
    Ok(result.repr())
}

pub fn parse(code: &str) -> Result<ast::Code, String> {
    let mut scope = Universe::new();
    parser::parse(code, &mut scope)
}

pub fn parse_scope(code: &str, scope: &mut Universe) -> Result<ast::Code, String> {
    parser::parse(code, scope)
}

pub fn interpret(code: &str) -> Result<interp::Interpreter, String> {
    let mut interpreter = interp::Interpreter::new();
    interpreter.interpret(code)?;
    Ok(interpreter)
}

pub fn interpret_with_scope(code: &str, scope: scope::Universe) -> Result<interp::Interpreter, String> {
    let mut interpreter = interp::Interpreter::with_scope(scope);
    interpreter.interpret(code)?;
    Ok(interpreter)
}

pub fn run_file(path: &str) -> Result<String, String> {
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    run(&code)
}

pub fn interpret_file(path: &str) -> interp::Interpreter {
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e)).unwrap();
    let mut interpreter = interp::Interpreter::new();
    interpreter.interpret(&code).unwrap();
    interpreter
}

pub fn eval_template(template: &str, scope: Universe) -> Result<interp::Interpreter, String> {
    let mut interpreter = interp::Interpreter::with_scope(scope).wit_eval_mode(EvalMode::TEMPLATE);
    // flip template
    let flipped = flip_template(template);
    println!("flipped: {}", flipped);
    interpreter.interpret(&flipped)?;
    Ok(interpreter)
}

pub fn eval_config(code: &str) -> Result<interp::Interpreter, String> {
    let mut interpreter = interp::Interpreter::new().wit_eval_mode(EvalMode::CONFIG);
    interpreter.interpret(code)?;
    Ok(interpreter)
}

// convert template (ex, a C file with interpolated auto expressions) into an auto source code with C code converted to lines of interpolated strings
// Example:
// template:
// <code>
// #include <stdio.h>
// int main() {
//     printf("Hello, $name!\n");
//     return 0;
// }
// </code>
// flipped:
// <code>
// f`#include <stdio.h>`
// f`int main() {`
// f`    printf(\"Hello, $name!\\n\");`
// f`    return 0;`
// f`}`
// </code>
fn flip_template(template: &str) -> String {
    let lines = template.lines();
    let mut result = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            // NOTE: keep empty lines
            result.push("``".to_string());
            continue;
        }
        if trimmed.starts_with("$") && !trimmed.starts_with("${") {
            let code = &trimmed[1..].trim();
            result.push(format!("{}", code));
        } else {
            result.push(format!("`{}`", line));
        }
    }
    // str.lines() does not include the last empty line
    if template.ends_with("\n") {
        result.push("``".to_string());
    }
    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use autoval::Value;

    #[test]
    fn test_unit() {
        let code = "1u+2u";
        let result = run(code).unwrap();
        assert_eq!(result, "3u");

        let code = "25u+123";
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
        let code = "var a = 10; if a > 10 { a+1 } else { a-1 }";
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
        let code = r#"$ for i in 0..10 { `$i`; mid(",") }"#;
        let scope = Universe::new();
        let result = eval_template(code, scope).unwrap();
        assert_eq!(result.result.repr(), "0,1,2,3,4,5,6,7,8,9");
    }

    #[test]
    fn test_for_with_mid_and_newline() {
        let code = r#"
$ for i in 0..10 {
    ${i}${mid(",")}
$ }"#;
        let scope = Universe::new();
        let result = eval_template(code, scope).unwrap();
        let expected = "\n    0,\n    1,\n    2,\n    3,\n    4,\n    5,\n    6,\n    7,\n    8,\n    9";
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
        // TODO: capture stdout and assert
        assert_eq!(result, "void");
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
        let code = r#"[ 1,
        2
        ]
        "#;
        let result = run(code).unwrap();
        println!("{}", result);
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
    fn test_widget() {
        let code = r#"
        widget MyWidget {
            model {
                var a = 1
            }
            view {
                text(f"Count: $a")
                button("+") {
                    onclick: || a = a + 1
                }
            }
        }"#;
        let result = run(code).unwrap();
        println!("{}", result);
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
    fn test_nodes() {
        let code = r#"center {
            text("Hello")
            button("OK") {
                onclick: || print("clicked")
            }
        }"#;
        let result = run(code).unwrap();
        println!("{}", result);
    }

    #[test]
    fn test_app() {
        let code = r#"
        widget hello {
            model {
                var name = ""
            }

            view {
                text(f"Hello $name")
            }
        }

        app {
            center {
                hello(name="You")
            }
            bottom {
                text("Bottom")
            }
        }"#;

        let result = interpret(code);
        match result {
            Ok(result) => {
                match result.result {
                    autoval::Value::Node(app) => {
                        println!("node: {}", app.to_string());
                        app.nodes.iter().for_each(|node| {
                            println!("node: {}", node.to_string());
                        });
                    }
                    _ => {}
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
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
        assert_eq!(interpreter.result.repr(), r#"
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
</table>"#);
    }

    #[test]
    fn test_flip_template() {
        let code = r#"#include <stdio.h>

int main() {
    printf("Hello, $name!\n");

    $ for i in 0..10 {
        printf("i = $i\n");
    $ }

    return 0;
}
"#;
        let result = flip_template(code);

        assert_eq!(result, r#"`#include <stdio.h>`
``
`int main() {`
`    printf("Hello, $name!\n");`
``
for i in 0..10 {
`        printf("i = $i\n");`
}
``
`    return 0;`
`}`
``"#);
    }

    #[test]
    fn test_flip_template_with_multiple_lines() {
        let template = r#"
$ for row in rows {
{
    name: ${row.name},
    age: ${row.age},
}${mid(",")}
$ }
"#;
        let result = flip_template(template);
        assert_eq!(result, r#"``
for row in rows {
`{`
`    name: ${row.name},`
`    age: ${row.age},`
`}${mid(",")}`
}
``"#);
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
}${mid(",")}
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
}
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
        assert_eq!(result, "void");
    }


    #[test]
    fn test_to_string() {
        let code = r#"1.str()"#;
        let result = run(code).unwrap();
        assert_eq!(result, "1");

        let code = r#""hello".up()"#;
        let result = run(code).unwrap();
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_insert_global_fn() {
        fn myjoin(arg: &autoval::Args) -> Value {
            Value::Str(arg.args.iter().map(|v| v.to_string()).collect::<Vec<String>>().join("::"))
        }

        let mut scope = Universe::new();
        scope.add_global_fn("myjoin", myjoin);
        let code = "myjoin(1, 2, 3)";
        let result = run_with_scope(code, scope).unwrap();
        assert_eq!(result, "1::2::3");
    }

    #[test]
    fn test_ref() {
        let code = "var a = 1; var b = ref a; b";
        let result = run(code).unwrap();
        assert_eq!(result, "1");
    }

    #[test]
    fn test_ref_modify() {
        let code = "var a = 1; var b = ref a; b = 2; [a, b]";
        let result = run(code).unwrap();
        assert_eq!(result, "[2, 2]");
    }

    #[test]
    fn test_ref_array() {
        let code = "var a = [1, 2, 3]; var b = ref a; b = [4, 5, 6]; var c = {a: a, b: b}; c";
        let result = run(code).unwrap();
        assert_eq!(result, "{a: [4, 5, 6], b: [4, 5, 6]}");
    }

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
        let code = "type Point { x int; y int }; let p = Point(x:1, y:2); p";
        let mut interpreter = interpret(code).unwrap();
        assert_eq!(interpreter.result.repr(), "Point{x: 1, y: 2}");

        let code = "p.x";
        let result = interpreter.eval(code);
        assert_eq!(result.repr(), "1");
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
        wing.fly()
        var duck = Duck()
        duck.fly()
        "#;
        let result = run(code).unwrap();
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
        ref g
        "#;
        let result = run(code).unwrap();
        assert_eq!(result, r#"grid(a:"first",b:"second",c:"third",) {[1, 2, 3];[4, 5, 6];[7, 8, 9]}"#);
    }

    #[test]
    fn test_config() {
        let code = r#"
name: "hello"
version: "0.1.0"

exe(hello) {
    dir: "src"
    main: "main.c"
}"#;
        let interp = eval_config(code).unwrap();
        let result = interp.result;
        assert_eq!(result.repr(), r#"root {name: "hello", version: "0.1.0"} [exe (hello) {dir: "src", main: "main.c"}]"#);
    }

    #[test]
    fn test_std() {
        let code = r#"
use std.math: square

square(5)
"#;
        // let result = run(code).unwrap();
        //assert_eq!(result, "25");
    }


}
