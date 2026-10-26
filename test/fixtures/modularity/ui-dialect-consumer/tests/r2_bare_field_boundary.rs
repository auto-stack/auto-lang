//! PLAN-778 R2 裸字段边界组合，经显式 UI 领域组合重放。

use std::sync::atomic::Ordering;
use std::sync::Arc;

use auto_lang::ast::Type;
use auto_lang::error::AutoResult;
use auto_lang::parser::Parser;
use auto_lang::session::CompilerSession;
use auto_ui_dialect::{UiCode, UiDialect, UiStmt};
use dialect_api_consumer::{ExternalUiDialect, Probe};

fn parse_ui(code: &str, probe: Arc<Probe>) -> AutoResult<UiCode> {
    Parser::from(code)
        .with_session(CompilerSession::ui())
        .parse_domain(&ExternalUiDialect::new(probe))
        .map(UiCode)
}

fn only_model(ast: &UiCode) -> &auto_ui_dialect::ast::ModelBlock {
    let Some(UiStmt::ModelBlock(block)) = ast.domain_stmts().next() else {
        panic!("expected ModelBlock")
    };
    block
}

#[test]
fn r2_both_type_and_init() {
    let probe = Arc::new(Probe::default());
    let ast = parse_ui("model { count int = 1 }", probe.clone()).unwrap();
    let block = only_model(&ast);
    assert!(matches!(block.fields[0].ty, Type::Int));
    assert!(matches!(block.fields[0].init, auto_lang::ast::Expr::Int(1)));
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}

#[test]
fn r2_type_only_is_supported() {
    let probe = Arc::new(Probe::default());
    let ast = parse_ui("model { count int }", probe.clone()).unwrap();
    let block = only_model(&ast);
    assert!(matches!(block.fields[0].ty, Type::Int));
    assert!(matches!(block.fields[0].init, auto_lang::ast::Expr::Nil));
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}

#[test]
fn r2_init_only_is_supported() {
    let probe = Arc::new(Probe::default());
    let ast = parse_ui("model { count = 1 }", probe.clone()).unwrap();
    let block = only_model(&ast);
    assert!(matches!(block.fields[0].ty, Type::Unknown));
    assert!(matches!(block.fields[0].init, auto_lang::ast::Expr::Int(1)));
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}

#[test]
fn r2_bare_field_is_supported() {
    let probe = Arc::new(Probe::default());
    let ast = parse_ui("model { count }", probe.clone()).unwrap();
    let block = only_model(&ast);
    assert_eq!(&*block.fields[0].name, "count");
    assert!(matches!(block.fields[0].ty, Type::Unknown));
    assert!(matches!(block.fields[0].init, auto_lang::ast::Expr::Nil));
    assert!(!block.fields[0].mutable);
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}

#[test]
fn r2_bare_field_then_typed_field_multiline() {
    let probe = Arc::new(Probe::default());
    let ast = parse_ui("model { count\nother int = 2 }", probe.clone()).unwrap();
    let block = only_model(&ast);
    assert_eq!(block.fields.len(), 2);
    assert!(matches!(block.fields[0].ty, Type::Unknown));
    assert!(matches!(block.fields[0].init, auto_lang::ast::Expr::Nil));
    assert!(matches!(block.fields[1].ty, Type::Int));
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}

#[test]
fn r2_var_bare_field() {
    let probe = Arc::new(Probe::default());
    let ast = parse_ui("model { var flag }", probe).unwrap();
    let block = only_model(&ast);
    assert!(block.fields[0].mutable);
    assert!(matches!(block.fields[0].ty, Type::Unknown));
    assert!(matches!(block.fields[0].init, auto_lang::ast::Expr::Nil));
}

#[test]
fn r2_malformed_after_name_still_errors() {
    let probe = Arc::new(Probe::default());
    assert!(parse_ui("model { count 123 }", probe.clone()).is_err());
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 0);
}

#[test]
fn r2_builtin_bare_field_still_rejected_control() {
    assert!(Parser::from("model { count }")
        .with_session(CompilerSession::ui())
        .parse_domain(&UiDialect)
        .is_err());
}
