// Plan 071 Phase 6.1: Borrow Checking Integration Tests
//
// These tests verify that the compiler correctly detects and blocks
// unsafe closure captures (.view/.mut) that could cause dangling references.

use crate::vm::codegen::Codegen;
use crate::ast::{Expr, Stmt, Closure, Body, Arg, Args, Branch, Type};
use auto_val::Op;

#[cfg(test)]
mod borrow_check_tests {
    use super::*;

    /// Test that closure with .view capture is rejected
    #[test]
    fn test_borrow_check_view_capture() {
        let mut codegen = Codegen::new();

        // Create a closure that captures x.view (should error)
        // fn make_closure(x int) {
        //     return y => y + x.view  // ERROR: Cannot capture borrowed value
        // }
        let closure = Closure {
            params: vec![],
            ret: None,
            body: Box::new(Expr::Bina(
                Box::new(Expr::Ident("y".into())),
                Op::Add,
                Box::new(Expr::View(Box::new(Expr::Ident("x".into())))),
            )),
        };

        let result = codegen.compile_closure(&closure);

        // Should fail with error about unsafe capture
        if result.is_ok() {
            eprintln!("ERROR: Expected compile to fail, but it succeeded!");
        }
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("Cannot capture borrowed value"));
        assert!(err_msg.contains("x"));
    }

    /// Test that closure with .mut capture is rejected
    #[test]
    fn test_borrow_check_mut_capture() {
        let mut codegen = Codegen::new();

        // Create a closure that captures x.mut (should error)
        let closure = Closure {
            params: vec![],
            ret: None,
            body: Box::new(Expr::Bina(
                Box::new(Expr::Ident("y".into())),
                Op::Add,
                Box::new(Expr::Mut(Box::new(Expr::Ident("x".into())))),
            )),
        };

        let result = codegen.compile_closure(&closure);

        // Should fail with error about unsafe capture
        if result.is_ok() {
            eprintln!("ERROR: Expected compile to fail, but it succeeded!");
        }
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("Cannot capture borrowed value"));
    }

    /// Test that closure with default capture (copy) is accepted
    #[test]
    fn test_borrow_check_default_copy_allowed() {
        let mut codegen = Codegen::new();

        // Create a closure that captures x (default copy semantics - should work)
        let closure = Closure {
            params: vec![],
            ret: None,
            body: Box::new(Expr::Bina(
                Box::new(Expr::Ident("y".into())),
                Op::Add,
                Box::new(Expr::Ident("x".into())),  // Default: copy semantics
            )),
        };

        let result = codegen.compile_closure(&closure);

        // Should succeed (default copy semantics are safe)
        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    /// Test that closure with .take is accepted
    #[test]
    fn test_borrow_check_take_allowed() {
        let mut codegen = Codegen::new();

        // Create a closure that captures x.take (move semantics - should work)
        let closure = Closure {
            params: vec![],
            ret: None,
            body: Box::new(Expr::Bina(
                Box::new(Expr::Ident("y".into())),
                Op::Add,
                Box::new(Expr::Take(Box::new(Expr::Ident("x".into())))),
            )),
        };

        let result = codegen.compile_closure(&closure);

        // Should succeed (.take is explicit move, safe)
        // Note: The checker only checks View/Mut, not Take
        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    /// Test that closure with multiple captures checks all variables
    #[test]
    fn test_borrow_check_multiple_captures() {
        let mut codegen = Codegen::new();

        // Create a closure that captures both x and y, one with .view
        // return z => z + x.view + y  // ERROR on x.view
        let closure = Closure {
            params: vec![],
            ret: None,
            body: Box::new(Expr::Bina(
                Box::new(Expr::Bina(
                    Box::new(Expr::Ident("z".into())),
                    Op::Add,
                    Box::new(Expr::View(Box::new(Expr::Ident("x".into())))),
                )),
                Op::Add,
                Box::new(Expr::Ident("y".into())),
            )),
        };

        let result = codegen.compile_closure(&closure);

        // Should fail because x.view is unsafe
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("Cannot capture borrowed value"));
        assert!(err_msg.contains("x"));
    }

    /// Test that nested expressions are checked for unsafe captures
    #[test]
    fn test_borrow_check_nested_expressions() {
        let mut codegen = Codegen::new();

        // Create a closure with nested view: (x.view + y.view)
        let closure = Closure {
            params: vec![],
            ret: None,
            body: Box::new(Expr::Bina(
                Box::new(Expr::View(Box::new(Expr::Ident("x".into())))),
                Op::Add,
                Box::new(Expr::View(Box::new(Expr::Ident("y".into())))),
            )),
        };

        let result = codegen.compile_closure(&closure);

        // Should fail - both x.view and y.view are unsafe
        assert!(result.is_err());
    }

    /// Test that unsafe captures in function calls are detected
    #[test]
    fn test_borrow_check_unsafe_in_function_call() {
        let mut codegen = Codegen::new();

        // Create a closure that passes x.view to a function
        // return z => some_func(x.view, z)
        let closure = Closure {
            params: vec![],
            ret: None,
            body: Box::new(Expr::Call(crate::ast::Call {
                name: Box::new(Expr::Ident("some_func".into())),
                args: Args {
                    args: vec![
                        Arg::Pos(Expr::View(Box::new(Expr::Ident("x".into())))),
                        Arg::Pos(Expr::Ident("z".into())),
                    ],
                },
                ret: Type::Unknown,
                type_args: vec![],
            })),
        };

        let result = codegen.compile_closure(&closure);

        // Should fail - x.view in function argument is unsafe
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("Cannot capture borrowed value"));
    }

    /// Test that unsafe captures in array elements are detected
    #[test]
    fn test_borrow_check_unsafe_in_array() {
        let mut codegen = Codegen::new();

        // Create a closure that captures array with view
        // return z => [x.view, z]
        let closure = Closure {
            params: vec![],
            ret: None,
            body: Box::new(Expr::Array(vec![
                Expr::View(Box::new(Expr::Ident("x".into()))),
                Expr::Ident("z".into()),
            ])),
        };

        let result = codegen.compile_closure(&closure);

        // Should fail - x.view in array element is unsafe
        assert!(result.is_err());
    }

    /// Test that unsafe captures in if expressions are detected
    #[test]
    fn test_borrow_check_unsafe_in_if_expression() {
        let mut codegen = Codegen::new();

        // Create a closure with if statement using view
        // return z => if x.view > 0 { z } else { 0 }
        let closure = Closure {
            params: vec![],
            ret: None,
            body: Box::new(Expr::If(crate::ast::If {
                branches: vec![
                    Branch {
                        cond: Expr::Bina(
                            Box::new(Expr::View(Box::new(Expr::Ident("x".into())))),
                            Op::Gt,
                            Box::new(Expr::Int(0)),
                        ),
                        body: Body::single_expr(Expr::Ident("z".into())),
                    },
                ],
                else_: Some(Body::single_expr(Expr::Int(0))),
            })),
        };

        let result = codegen.compile_closure(&closure);

        // Should fail - x.view in condition is unsafe
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("Cannot capture borrowed value"));
    }

    /// Test that unsafe captures in block expressions are detected
    #[test]
    fn test_borrow_check_unsafe_in_block() {
        let mut codegen = Codegen::new();

        // Create a closure with block using view
        // return z => { let temp = x.view; temp + z }
        let closure = Closure {
            params: vec![],
            ret: None,
            body: Box::new(Expr::Block(crate::ast::Body {
                stmts: vec![
                    Stmt::Expr(Expr::Bina(
                        Box::new(Expr::Ident("temp".into())),
                        Op::Asn,
                        Box::new(Expr::View(Box::new(Expr::Ident("x".into())))),
                    )),
                    Stmt::Expr(Expr::Bina(
                        Box::new(Expr::Ident("temp".into())),
                        Op::Add,
                        Box::new(Expr::Ident("z".into())),
                    )),
                ],
                has_new_line: false,
            })),
        };

        let result = codegen.compile_closure(&closure);

        // Should fail - x.view in block is unsafe
        assert!(result.is_err());
    }

    /// Test that direct variable reference (no borrow) is safe
    #[test]
    fn test_borrow_check_direct_reference_safe() {
        let mut codegen = Codegen::new();

        // Create a closure that directly references variable (safe)
        let closure = Closure {
            params: vec![],
            ret: None,
            body: Box::new(Expr::Ident("x".into())),
        };

        let result = codegen.compile_closure(&closure);

        // Should succeed - direct reference is safe (will be copied)
        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }
}
