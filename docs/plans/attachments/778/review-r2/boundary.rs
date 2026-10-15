use auto_lang::ast::{Expr, Stmt, Type};
use auto_lang::dialect::DialectPriority;
use auto_lang::parser::Parser;
use auto_lang::session::CompilerSession;
use std::sync::{Arc, atomic::Ordering};
use ui_dialect_consumer::{ExternalModelDialect, Probe};

#[test]
fn type_only_is_supported_control() {
    let probe = Arc::new(Probe::default());
    let ast = Parser::from("model { count int }")
        .with_session(CompilerSession::ui())
        .with_dialect(Box::new(ExternalModelDialect::new(probe.clone())), DialectPriority::Override)
        .parse().unwrap();
    let Stmt::ModelBlock(block) = &ast.stmts[0] else { panic!("typed block") };
    assert!(matches!(block.fields[0].ty, Type::Int));
    assert!(matches!(block.fields[0].init, Expr::Nil));
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}

#[test]
fn builtin_bare_field_is_also_rejected_control() {
    assert!(Parser::from("model { count }").with_session(CompilerSession::ui()).parse_stmt().is_err());
}

#[test]
fn multiline_optional_type_and_initializer() {
    let probe = Arc::new(Probe::default());
    let mut parser = Parser::from("model { count\nother int = 2 }")
        .with_session(CompilerSession::ui())
        .with_dialect(Box::new(ExternalModelDialect::new(probe)), DialectPriority::Override);
    let result = parser.parse_stmt();
    println!("multiline={result:?}");
    assert!(result.is_ok(), "frozen name [type] [= expr] includes bare field before newline");
}
