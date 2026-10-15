use auto_lang::ast::{Expr, Stmt, Type};
use auto_lang::dialect::DialectPriority;
use auto_lang::error::{AutoError, SyntaxError};
use auto_lang::parser::Parser;
use auto_lang::session::CompilerSession;
use std::sync::{Arc, atomic::Ordering};
use ui_dialect_consumer::{ExternalModelDialect, Probe};

#[test]
fn mixed_bare_var_typed_and_comment_boundaries() {
    for input in ["model { var flag\ncount int = 2\nempty }", "model { var flag // comment\r\ncount int = 2\r\nempty\r\n}"] {
        let probe=Arc::new(Probe::default());
        let ast=Parser::from(input).with_session(CompilerSession::ui())
            .with_dialect(Box::new(ExternalModelDialect::new(probe.clone())),DialectPriority::Override)
            .parse().unwrap();
        let Stmt::ModelBlock(block)=&ast.stmts[0] else {panic!("ModelBlock")};
        assert_eq!(block.fields.len(),3);
        assert!(block.fields[0].mutable);
        assert!(matches!(block.fields[0].ty,Type::Unknown));
        assert!(matches!(block.fields[0].init,Expr::Nil));
        assert!(matches!(block.fields[1].ty,Type::Int));
        assert!(matches!(block.fields[1].init,Expr::Int(2)));
        assert!(matches!(block.fields[2].ty,Type::Unknown));
        assert!(matches!(block.fields[2].init,Expr::Nil));
        assert_eq!(probe.accepts.load(Ordering::SeqCst),1);
    }
}

#[test]
fn eof_ends_field_but_does_not_accept_unclosed_model() {
    let probe=Arc::new(Probe::default());
    let mut parser=Parser::from("model { flag").with_session(CompilerSession::ui())
        .with_dialect(Box::new(ExternalModelDialect::new(probe.clone())),DialectPriority::Override);
    let result=parser.parse_stmt();
    assert!(matches!(result,Err(AutoError::Syntax(SyntaxError::UnexpectedToken{..}))),"missing closing brace must still reject: {result:?}");
    assert_eq!(probe.accepts.load(Ordering::SeqCst),0);
}

#[test]
fn malformed_field_keeps_original_diagnostic_location() {
    let probe=Arc::new(Probe::default());
    let mut parser=Parser::from("model { count 123 }").with_session(CompilerSession::ui())
        .with_dialect(Box::new(ExternalModelDialect::new(probe.clone())),DialectPriority::Override);
    match parser.parse_stmt().unwrap_err() {
        AutoError::Syntax(SyntaxError::Generic{message,span}) => {
            assert!(message.contains("Expected type"));
            assert_eq!(span.offset(),14);
        }
        other=>panic!("type diagnostic: {other:?}"),
    }
    assert_eq!(probe.accepts.load(Ordering::SeqCst),0);
}
