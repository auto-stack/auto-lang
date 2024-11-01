mod token;
mod lexer;
pub mod ast;
mod parser;
mod eval;
// pub mod value;
mod scope;
pub mod transpiler;
pub mod repl;
pub mod libs;
pub mod util;

pub fn run(code: &str) -> Result<String, String> {
    let mut scope = scope::Universe::new();
    let ast = parser::parse(code, &mut scope)?;
    let mut evaler = eval::Evaler::new(&mut scope);
    let result = evaler.eval(&ast);
    Ok(result.to_string())
}

pub fn parse(code: &str) -> Result<ast::Code, String> {
    println!("parsing code: {}", code);
    let mut scope = scope::Universe::new();
    parser::parse(code, &mut scope)
}

pub fn parse_scope(code: &str, scope: &mut scope::Universe) -> Result<ast::Code, String> {
    parser::parse(code, scope)
}

pub fn interpret(code: &str) -> Result<String, String> {
    run(code)
}

pub fn interpret_file(path: &str) -> Result<String, String> {
    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    run(&code)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn test_fn() {
        let code = "fn add(a, b) { a + b }; add(12, 2)";
        let result = run(code).unwrap();
        assert_eq!(result, "14");

        let code = "fn add(a, b) { a + b }; add(a=1, b=2)";
        let result = run(code).unwrap();
        assert_eq!(result, "3");
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
        let code = "fn sub(a, b) { a - b }; sub(b=5, a=12)";
        let result = run(code).unwrap();
        assert_eq!(result, "7");
    }

    // #[test]
    // fn test_type() {
    //     let code = "type Point { x int, y int }; var p = Point(x=1, y=2); p.x";
    //     let result = run(code).unwrap();
    //     assert_eq!(result, "1");
    // }


    #[test]
    fn test_json() {
        let code = r#"
            var ServiceInfo = [
                { id: 0x10, name: "DiagnosticSessionControl",  desc: "诊断会话控制" },
                { id: 0x11, name: "EcuReset",  desc: "电控单元复位" },
                { id: 0x14, name: "ClearDiagnosticInformation",  desc: "清除诊断信息" },
                { id: 0x19, name: "ReadDTCInformation",  desc: "读取DTC信息" },
                { id: 0x22, name: "ReadDataByIdentifier",  desc: "读取数据" },
                { id: 0x23, name: "ReadMemoryByAddress",  desc: "读取内存" },
                { id: 0x27, name: "SecurityAccess",  desc: "安全访问" },
                { id: 0x28, name: "CommunicationControl",  desc: "通信控制 " },
                { id: 0x2A, name: "ReadDataByPeriodicIdentifier",  desc: "读取数据（周期标识符）" },
                { id: 0x2C, name: "DynamicallyDefineDataIdentifier",  desc: "动态定义数据标识符" },
                { id: 0x2E, name: "WriteDataByIdentifier",  desc: "写入数据" },
                { id: 0x2F, name: "InputOutputControlByIdentifier",  desc: "输入输出控制" },
                { id: 0x31, name: "RoutineControl",  desc: "例程控制" },
                { id: 0x3D, name: "WriteMemoryByAddress",  desc: "写入内存" },
                { id: 0x3E, name: "TesterPresent",  desc: "诊断设备在线" },
                { id: 0x85, name: "ControlDTCSetting",  desc: "控制DTC设置" },
            ]
        "#;
        let result = run(code).unwrap();
        println!("{}", result);
    }


    #[test]
    fn test_widget() {
        let code = r#"
        widget MyWidget {
            model { 
                var a = 1 
            }
            view {
                button("Click me")
            }
        }"#;
        let result = run(code).unwrap();
        println!("{}", result);
    }
}


