//! Plan 125: Phase 3 - Polymorphic routing tests
//!
//! Tests for:
//! - `on(ctx)` context parameter parsing
//! - Literal patterns: `"ping"`, `404`, `true`
//! - Type binding patterns: `msg string`, `u User`
//! - Guard expressions: `amount int if amount > 10000`

use crate::ast::{Code, LiteralValue, Stmt, TaskDef, TaskMsgPattern, TaskOnBlock, Type};
use crate::parser::Parser;

/// Helper to extract task definitions from parsed code
fn extract_tasks(code: &Code) -> Vec<&TaskDef> {
    code.stmts
        .iter()
        .filter_map(|stmt| {
            if let Stmt::TaskDef(task) = stmt {
                Some(task)
            } else {
                None
            }
        })
        .collect()
}

// ========== Phase 3.1: AST Tests ==========

#[test]
fn test_literal_value_string() {
    let lit = LiteralValue::String("ping".into());
    assert_eq!(lit.to_string(), "\"ping\"");
}

#[test]
fn test_literal_value_int() {
    let lit = LiteralValue::Int(404);
    assert_eq!(lit.to_string(), "404");
}

#[test]
fn test_literal_value_uint() {
    let lit = LiteralValue::Uint(200);
    assert_eq!(lit.to_string(), "200u");
}

#[test]
fn test_literal_value_bool() {
    assert_eq!(LiteralValue::Bool(true).to_string(), "true");
    assert_eq!(LiteralValue::Bool(false).to_string(), "false");
}

#[test]
fn test_task_msg_pattern_literal_string() {
    let pattern = TaskMsgPattern::Literal(LiteralValue::String("ping".into()));
    assert_eq!(pattern.to_string(), "\"ping\"");
    assert!(pattern.is_literal());
    assert!(!pattern.has_bindings());
    assert!(pattern.variant_name().is_none());
}

#[test]
fn test_task_msg_pattern_literal_int() {
    let pattern = TaskMsgPattern::Literal(LiteralValue::Int(404));
    assert_eq!(pattern.to_string(), "404");
    assert!(pattern.is_literal());
}

#[test]
fn test_task_msg_pattern_type_binding() {
    let pattern = TaskMsgPattern::TypeBinding {
        name: "msg".into(),
        type_expr: Box::new(Type::Str(0)),
    };
    assert_eq!(pattern.to_string(), "msg str");
    assert!(pattern.is_type_binding());
    assert!(!pattern.has_bindings());
    assert!(pattern.variant_name().is_none());
    assert!(pattern.type_expr().is_some());
    assert!(pattern.binding_name().is_some());
    assert_eq!(pattern.binding_name().unwrap().as_str(), "msg");
}

// ========== Phase 3.2: Parser Tests ==========

#[test]
fn test_parse_on_with_context_param() {
    let code = r#"
        task TestTask {
            on(ctx) {
                "ping" -> { }
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should succeed: {:?}", result.err());

    let module = result.unwrap();
    let tasks = extract_tasks(&module);
    assert_eq!(tasks.len(), 1);

    let task = tasks[0];
    assert!(task.on_block.has_context());
    assert_eq!(task.on_block.context_param, Some("ctx".into()));
}

#[test]
fn test_parse_on_without_context_param() {
    let code = r#"
        task TestTask {
            on {
                Reset -> { }
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should succeed: {:?}", result.err());

    let module = result.unwrap();
    let tasks = extract_tasks(&module);
    let task = tasks[0];
    assert!(!task.on_block.has_context());
    assert!(task.on_block.context_param.is_none());
}

#[test]
fn test_parse_literal_pattern_string() {
    let code = r#"
        task TestTask {
            on {
                "ping" -> { }
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse().expect("Parser should succeed");

    let tasks = extract_tasks(&result);
    let task = tasks[0];
    assert_eq!(task.on_block.handlers.len(), 1);

    let (pattern, guard, _body) = &task.on_block.handlers[0];
    assert!(matches!(pattern, TaskMsgPattern::Literal(LiteralValue::String(_))));
    assert!(matches!(guard, None));
}

#[test]
fn test_parse_literal_pattern_int() {
    let code = r#"
        task TestTask {
            on {
                404 -> { }
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse().expect("Parser should succeed");

    let tasks = extract_tasks(&result);
    let task = tasks[0];
    let (pattern, _, _) = &task.on_block.handlers[0];
    assert!(matches!(pattern, TaskMsgPattern::Literal(LiteralValue::Int(404))));
}

#[test]
fn test_parse_literal_pattern_bool() {
    let code = r#"
        task TestTask {
            on {
                true -> { }
                false -> { }
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse().expect("Parser should succeed");

    let tasks = extract_tasks(&result);
    let task = tasks[0];
    assert_eq!(task.on_block.handlers.len(), 2);

    let (p1, _, _) = &task.on_block.handlers[0];
    assert!(matches!(p1, TaskMsgPattern::Literal(LiteralValue::Bool(true))));

    let (p2, _, _) = &task.on_block.handlers[1];
    assert!(matches!(p2, TaskMsgPattern::Literal(LiteralValue::Bool(false))));
}

#[test]
fn test_parse_type_binding_pattern() {
    // Use empty body to avoid undefined variable issues during parsing
    let code = r#"
        task TestTask {
            on {
                msg str -> { }
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse().expect("Parser should succeed");

    let tasks = extract_tasks(&result);
    let task = tasks[0];
    let (pattern, _, _) = &task.on_block.handlers[0];

    match pattern {
        TaskMsgPattern::TypeBinding { name, type_expr } => {
            assert_eq!(name.as_str(), "msg");
            // Check that it's a string type
            assert!(matches!(type_expr.as_ref(), Type::StrSlice), "Expected StrSlice type, got: {:?}", type_expr);
        }
        _ => panic!("Expected TypeBinding pattern, got: {:?}", pattern),
    }
}

#[test]
fn test_parse_guard_expression() {
    // Use empty body to avoid undefined variable issues
    // The guard expression references the binding variable, but since we're only
    // testing parsing, we use a simple comparison that should parse correctly
    let code = r#"
        task TestTask {
            on {
                amount int if amount > 10000 -> { }
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse().expect("Parser should succeed");

    let tasks = extract_tasks(&result);
    let task = tasks[0];
    let (pattern, guard, _body) = &task.on_block.handlers[0];

    // Check pattern is TypeBinding
    assert!(pattern.is_type_binding());

    // Check guard is present
    assert!(matches!(guard, Some(_)), "Guard expression should be present");
}

#[test]
fn test_parse_mixed_patterns() {
    // Simplified test with empty bodies to focus on pattern parsing
    let code = r#"
        task TestTask {
            on(ctx) {
                "ping" -> { }
                msg string -> { }
                amount int if amount > 10000 -> { }
                Reset -> { }
                Add(val) -> { }
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse().expect("Parser should succeed");

    let tasks = extract_tasks(&result);
    let task = tasks[0];

    // Check context param
    assert!(task.on_block.has_context());

    // Check all handlers
    assert_eq!(task.on_block.handlers.len(), 5);

    // "ping" - literal string
    let (p1, _, _) = &task.on_block.handlers[0];
    assert!(p1.is_literal());

    // msg string - type binding
    let (p2, _, _) = &task.on_block.handlers[1];
    assert!(p2.is_type_binding());

    // amount int if ... - type binding with guard
    let (p3, g3, _) = &task.on_block.handlers[2];
    assert!(p3.is_type_binding());
    assert!(matches!(g3, Some(_)));

    // Reset - simple variant
    let (p4, _, _) = &task.on_block.handlers[3];
    assert!(matches!(p4, TaskMsgPattern::Simple(_)));

    // Add(val) - variant with bindings
    let (p5, _, _) = &task.on_block.handlers[4];
    assert!(p5.has_bindings());
}

#[test]
fn test_parse_else_handler_with_context() {
    let code = r#"
        task TestTask {
            on(ctx) {
                "ping" -> { }
                else -> { }
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse().expect("Parser should succeed");

    let tasks = extract_tasks(&result);
    let task = tasks[0];
    assert!(task.on_block.else_handler.is_some());
}

// ========== Phase 3.3: Implicit Union Tests (Future) ==========
// These tests will be implemented when Phase 3.3 is complete

#[test]
fn test_task_on_block_with_context() {
    use crate::token::Pos;

    let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
    let on_block = TaskOnBlock::with_context("ctx".into(), pos);
    assert!(on_block.has_context());
    assert_eq!(on_block.context_param, Some("ctx".into()));
}

#[test]
fn test_task_on_block_add_handler_with_guard() {
    use crate::ast::{Body, Expr};
    use crate::token::Pos;
    use auto_val::Op;

    let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
    let mut on_block = TaskOnBlock::new(pos);
    let pattern = TaskMsgPattern::TypeBinding {
        name: "amount".into(),
        type_expr: Box::new(Type::Int),
    };
    let guard = Some(Expr::Bina(
        Box::new(Expr::Ident("amount".into())),
        Op::Gt,
        Box::new(Expr::Int(10000)),
    ));
    let body = Body::new();

    on_block.add_handler_with_guard(pattern, guard.clone(), body);
    assert_eq!(on_block.handlers.len(), 1);
    let (p, g, _) = &on_block.handlers[0];
    assert!(g.is_some());
    assert!(p.is_type_binding());
}
