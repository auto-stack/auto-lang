// Test parser support for -> type return type annotation (Plan 073 Stage A.5)

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    #[test]
    fn test_arrow_return_type_float() {
        let source = r#"
fn test_float_return() -> float {
    3.14
}

fn main() -> int {
    0
}
"#;

        let mut parser = Parser::from(source);
        let result = parser.parse();

        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let code = result.unwrap();
        // Should have at least 2 function declarations (may have additional statements)
        assert!(code.stmts.len() >= 2, "Expected at least 2 function declarations, got {}", code.stmts.len());

        // Find test_float_return with float return type
        let float_fn = code.stmts.iter().find(|stmt| {
            if let crate::ast::Stmt::Fn(fn_decl) = stmt {
                fn_decl.name.as_str() == "test_float_return"
            } else {
                false
            }
        });

        assert!(float_fn.is_some(), "Expected to find test_float_return function");
        if let Some(crate::ast::Stmt::Fn(fn_decl)) = float_fn {
            assert!(matches!(fn_decl.ret, crate::ast::Type::Float));
        }
    }

    #[test]
    fn test_arrow_return_type_double() {
        let source = r#"
fn test_double_return() -> double {
    2.718281828
}

fn main() -> int {
    0
}
"#;

        let mut parser = Parser::from(source);
        let result = parser.parse();

        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let code = result.unwrap();
        assert!(code.stmts.len() >= 2, "Expected at least 2 function declarations, got {}", code.stmts.len());

        // Find test_double_return with double return type
        let double_fn = code.stmts.iter().find(|stmt| {
            if let crate::ast::Stmt::Fn(fn_decl) = stmt {
                fn_decl.name.as_str() == "test_double_return"
            } else {
                false
            }
        });

        assert!(double_fn.is_some(), "Expected to find test_double_return function");
        if let Some(crate::ast::Stmt::Fn(fn_decl)) = double_fn {
            assert!(matches!(fn_decl.ret, crate::ast::Type::Double));
        }
    }

    #[test]
    fn test_arrow_return_type_int() {
        let source = r#"
fn test_int_return() -> int {
    42
}

fn main() -> int {
    0
}
"#;

        let mut parser = Parser::from(source);
        let result = parser.parse();

        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let code = result.unwrap();
        assert!(code.stmts.len() >= 2, "Expected at least 2 function declarations, got {}", code.stmts.len());

        // Find test_int_return with int return type
        let int_fn = code.stmts.iter().find(|stmt| {
            if let crate::ast::Stmt::Fn(fn_decl) = stmt {
                fn_decl.name.as_str() == "test_int_return"
            } else {
                false
            }
        });

        assert!(int_fn.is_some(), "Expected to find test_int_return function");
        if let Some(crate::ast::Stmt::Fn(fn_decl)) = int_fn {
            assert!(matches!(fn_decl.ret, crate::ast::Type::Int));
        }
    }

    #[test]
    fn test_multiple_return_types() {
        let source = r#"
fn float_fn() -> float { 1.0 }
fn double_fn() -> double { 2.0 }
fn int_fn() -> int { 3 }
fn uint_fn() -> uint { 4 }
fn main() -> int { 0 }
"#;

        let mut parser = Parser::from(source);
        let result = parser.parse();

        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let code = result.unwrap();
        assert!(code.stmts.len() >= 5, "Expected at least 5 function declarations, got {}", code.stmts.len());

        // Verify each function has the correct return type
        for stmt in &code.stmts {
            if let crate::ast::Stmt::Fn(fn_decl) = stmt {
                match fn_decl.name.as_str() {
                    "float_fn" => assert!(matches!(fn_decl.ret, crate::ast::Type::Float)),
                    "double_fn" => assert!(matches!(fn_decl.ret, crate::ast::Type::Double)),
                    "int_fn" => assert!(matches!(fn_decl.ret, crate::ast::Type::Int)),
                    "uint_fn" => assert!(matches!(fn_decl.ret, crate::ast::Type::Uint)),
                    "main" => assert!(matches!(fn_decl.ret, crate::ast::Type::Int)),
                    _ => {} // ignore other statements
                }
            }
        }
    }

    #[test]
    fn test_old_style_return_type_still_works() {
        // Test that type without -> still works (backwards compatibility)
        let source = r#"
fn test_old_style() int {
    42
}

fn main() int {
    0
}
"#;

        let mut parser = Parser::from(source);
        let result = parser.parse();

        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let code = result.unwrap();
        assert!(code.stmts.len() >= 2, "Expected at least 2 function declarations, got {}", code.stmts.len());

        // Find test_old_style with int return type
        let old_style_fn = code.stmts.iter().find(|stmt| {
            if let crate::ast::Stmt::Fn(fn_decl) = stmt {
                fn_decl.name.as_str() == "test_old_style"
            } else {
                false
            }
        });

        assert!(old_style_fn.is_some(), "Expected to find test_old_style function");
        if let Some(crate::ast::Stmt::Fn(fn_decl)) = old_style_fn {
            assert!(matches!(fn_decl.ret, crate::ast::Type::Int));
        }
    }
}
