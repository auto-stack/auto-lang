//! 外部 model 方言经显式领域端口组合的 V-01..V-05 正反控。

use std::sync::atomic::Ordering;
use std::sync::Arc;

use auto_ast::ast::{Expr, Stmt, Type};
use auto_diagnostics::error::{AutoError, AutoResult, SyntaxError};
use auto_lang_core::parser::Parser;
use auto_lang_core::session::CompilerSession;
use auto_ui_dialect::{UiCode, UiDialect, UiStmt};
use dialect_api_consumer::{ExternalUiDialect, Probe};

fn parse_with_external_ui(code: &str, probe: Arc<Probe>) -> AutoResult<UiCode> {
    Parser::from(code)
        .with_session(CompilerSession::ui())
        .parse_domain(&ExternalUiDialect::new(probe))
        .map(UiCode)
}

fn parse_builtin(code: &str) -> UiCode {
    Parser::from(code)
        .with_session(CompilerSession::ui())
        .parse_domain(&UiDialect)
        .map(UiCode)
        .expect("builtin UI parse")
}

#[test]
fn v01_external_model_subset_produces_typed_block() {
    let probe = Arc::new(Probe::default());
    let ast = parse_with_external_ui("model { count int = 1 }", probe.clone()).unwrap();
    assert_eq!(probe.attempts.load(Ordering::SeqCst), 1);
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
    let Some(UiStmt::ModelBlock(block)) = ast.domain_stmts().next() else {
        panic!("expected ModelBlock")
    };
    assert_eq!(block.fields.len(), 1);
    let field = &block.fields[0];
    assert_eq!(&*field.name, "count");
    assert!(matches!(field.ty, Type::Int));
    assert!(matches!(field.init, Expr::Int(1)));
    assert!(!field.mutable);
    assert!(!field.is_primary);
    assert!(field.decorators.is_empty());
}

#[test]
fn v01_subset_var_untyped_and_core_expr_init() {
    let probe = Arc::new(Probe::default());
    let code = "model {\n  count int = 1\n  var name str = \"hi\"\n  items = [1, 2]\n}";
    let ast = parse_with_external_ui(code, probe.clone()).unwrap();
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
    let Some(UiStmt::ModelBlock(block)) = ast.domain_stmts().next() else {
        panic!("expected ModelBlock")
    };
    assert_eq!(block.fields.len(), 3);
    assert!(block.fields[1].mutable);
    assert!(matches!(block.fields[1].ty, Type::StrSlice));
    assert!(matches!(block.fields[2].ty, Type::Unknown));
    assert!(!matches!(block.fields[2].init, Expr::Nil));
}

#[test]
fn v02_core_assignment_not_swallowed() {
    let probe = Arc::new(Probe::default());
    let ast = Parser::from("model = 1")
        .parse_domain(&ExternalUiDialect::new(probe.clone()))
        .expect("Core assignment");
    assert_eq!(probe.attempts.load(Ordering::SeqCst), 0);
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn v02_ui_view_param_on_and_scene_unchanged() {
    let probe = Arc::new(Probe::default());
    let ast = parse_with_external_ui("fn f(view x int) { }", probe.clone()).unwrap();
    assert!(matches!(ast.core_stmts().next(), Some(Stmt::Fn(_))));

    let ast = parse_with_external_ui("scene Main : Node2D { }", probe.clone()).unwrap();
    assert!(matches!(ast.core_stmts().next(), Some(Stmt::SceneDecl(_))));

    let code = concat!(
        "widget W {\n",
        "  msg Msg { S }\n",
        "  model { var e str = \"\" }\n",
        "  view { col { text \"hi\" } }\n",
        "  on { .S -> { .e = \"after\" } }\n",
        "}"
    );
    let ast = parse_with_external_ui(code, probe).unwrap();
    let widget = ast
        .domain_stmts()
        .find_map(|stmt| match stmt {
            UiStmt::WidgetDecl(widget) => Some(widget),
            _ => None,
        })
        .expect("widget");
    assert_eq!(widget.on.as_ref().unwrap().handlers.len(), 1);
}

#[test]
fn v03_registration_survives_with_session() {
    let probe = Arc::new(Probe::default());
    let ast = Parser::from("model { count int = 1 }")
        .with_session(CompilerSession::ui())
        .parse_domain(&ExternalUiDialect::new(probe.clone()))
        .map(UiCode)
        .unwrap();
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
    assert!(matches!(
        ast.domain_stmts().next(),
        Some(UiStmt::ModelBlock(_))
    ));
}

#[test]
fn v03_registration_propagates_into_nested_bodies() {
    let probe = Arc::new(Probe::default());
    assert!(parse_with_external_ui("{ model { count int = 1 } }", probe.clone()).is_err());

    let code =
        "widget W {\n  msg Msg { S }\n  model { e int = 0 }\n  view { col { text \"hi\" } }\n}";
    let ast = parse_with_external_ui(code, probe.clone()).unwrap();
    assert!(ast
        .domain_stmts()
        .any(|stmt| matches!(stmt, UiStmt::WidgetDecl(_))));
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 0);
}

#[test]
fn v03_two_parser_instances_isolated() {
    let probe = Arc::new(Probe::default());
    let _ = parse_with_external_ui("model { count int = 1 }", probe.clone()).unwrap();
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
    let ast = parse_builtin("model { count int = 1 }");
    assert!(matches!(
        ast.domain_stmts().next(),
        Some(UiStmt::ModelBlock(_))
    ));
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}

#[test]
fn v04_err_keeps_registry_and_recovers_to_legal_stmts() {
    let probe = Arc::new(Probe::default());
    let err = parse_with_external_ui("model 123\nfn bridge() { }", probe.clone()).unwrap_err();
    assert!(matches!(err, AutoError::Syntax(_)));
    let ast = parse_with_external_ui("model { ok int = 2 }", probe.clone()).unwrap();
    assert!(matches!(
        ast.domain_stmts().next(),
        Some(UiStmt::ModelBlock(_))
    ));
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}

#[test]
fn v04_error_position_points_at_offending_token() {
    let probe = Arc::new(Probe::default());
    let err = parse_with_external_ui("model { = 5 }", probe).unwrap_err();
    match &err {
        AutoError::Syntax(SyntaxError::Generic { message, span }) => {
            assert!(message.contains("model field name expected identifier"));
            assert_eq!(span.offset(), 8);
        }
        other => panic!("expected Generic syntax error, got {other:?}"),
    }
}

#[test]
fn v04_none_rolls_back_cursor_annotated_model_falls_back_to_builtin() {
    let probe = Arc::new(Probe::default());
    let ast = parse_with_external_ui("model { #[primary] count int = 1 }", probe.clone()).unwrap();
    assert_eq!(probe.attempts.load(Ordering::SeqCst), 1);
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 0);
    let Some(UiStmt::ModelBlock(block)) = ast.domain_stmts().next() else {
        panic!("expected builtin ModelBlock")
    };
    assert!(block.fields[0].is_primary);
}

#[test]
fn v05_default_builtin_path_unchanged_without_registration() {
    let ast = parse_builtin("model { count int = 1 }");
    let Some(UiStmt::ModelBlock(block)) = ast.domain_stmts().next() else {
        panic!("expected builtin ModelBlock")
    };
    assert_eq!(&*block.fields[0].name, "count");
    assert!(matches!(block.fields[0].init, Expr::Int(1)));
}

#[test]
fn v05_conflict_resolution_distinguishable() {
    let code =
        "widget W {\n  msg Msg { S }\n  view { col { text \"hi\" } }\n}\nmodel { count int = 1 }";
    let probe = Arc::new(Probe::default());
    let ast = parse_with_external_ui(code, probe.clone()).unwrap();
    assert!(ast
        .domain_stmts()
        .any(|stmt| matches!(stmt, UiStmt::WidgetDecl(_))));
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);

    let probe2 = Arc::new(Probe::default());
    let ast2 = parse_builtin("model { count int = 1 }");
    assert!(matches!(
        ast2.domain_stmts().next(),
        Some(UiStmt::ModelBlock(_))
    ));
    assert_eq!(probe2.attempts.load(Ordering::SeqCst), 0);
}
