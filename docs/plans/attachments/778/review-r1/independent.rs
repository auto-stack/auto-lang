use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use auto_lang::ast::{Expr, Stmt};
use auto_lang::dialect::{Dialect, DialectPriority};
use auto_lang::error::AutoResult;
use auto_lang::parser::Parser;
use auto_lang::parser_bridge::ParserBridge;
use auto_lang::session::CompilerSession;
use auto_lang::token::TokenKind;
use ui_dialect_consumer::{ExternalModelDialect, Probe};

#[test]
fn nested_expression_keeps_active_registration() {
    let probe = Arc::new(Probe::default());
    let input = "model { child = () => { model { count int = 1 } } }";
    let result = Parser::from(input).with_session(CompilerSession::ui())
        .with_dialect(Box::new(ExternalModelDialect::new(probe.clone())), DialectPriority::Override)
        .parse();
    println!("input={input:?} result={result:?} attempts={} accepts={}",probe.attempts.load(Ordering::SeqCst),probe.accepts.load(Ordering::SeqCst));
    assert!(result.is_ok(), "frozen core-expression input must parse");
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 2, "outer and nested statement must both use external dialect");
}

#[test]
fn sibling_nested_block_control() {
    let probe = Arc::new(Probe::default());
    let result = Parser::from("{ model { count int = 1 } }\nmodel { count int = 2 }")
        .with_session(CompilerSession::ui())
        .with_dialect(Box::new(ExternalModelDialect::new(probe.clone())), DialectPriority::Override).parse();
    assert!(result.is_ok());
    assert_eq!(probe.accepts.load(Ordering::SeqCst),2);
}

struct ContextDialect { seen: Arc<Mutex<Vec<bool>>> }
impl Dialect for ContextDialect {
    fn matches(&self, _: &CompilerSession) -> bool { true }
    fn keywords(&self) -> &'static [&'static str] { &["probe", "tail"] }
    fn try_parse_stmt(&self, b: &mut ParserBridge<'_, '_>, kw: &str) -> AutoResult<Option<Stmt>> {
        self.seen.lock().unwrap().push(b.in_fn_body());
        b.advance();
        if kw == "probe" { Ok(Some(Stmt::Expr(b.parse_expr()?))) }
        else { Ok(Some(Stmt::Expr(Expr::Int(77)))) }
    }
}
#[test]
fn delegated_error_restores_context_at_recovery_point() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let mut p = Parser::from("probe () => { fn broken() {\ntail")
        .with_dialect(Box::new(ContextDialect{ seen: seen.clone() }), DialectPriority::Override);
    let recovery = ParserBridge::new(&mut p).checkpoint();
    let result = p.parse_stmt();
    println!("first_error={result:?}");
    assert!(result.is_err());
    { let mut b = ParserBridge::new(&mut p);
      b.restore(recovery);
      while b.cur_kind() != TokenKind::EOF && b.cur_text() != "tail" { b.advance(); }
      assert_eq!(b.cur_text(), "tail", "explicit recovery point reached"); }
    assert!(p.parse_stmt().is_ok());
    println!("context_seen={:?}",seen.lock().unwrap());
    assert_eq!(*seen.lock().unwrap(),vec![false,false], "Err must restore context before next legal statement");
}

struct TokenNone { calls: Arc<Mutex<usize>> }
#[test]
fn successful_delegation_context_control() {
    let seen=Arc::new(Mutex::new(Vec::new()));
    let mut p=Parser::from("probe () => { fn valid() { } }\ntail")
      .with_dialect(Box::new(ContextDialect{seen:seen.clone()}),DialectPriority::Override);
    assert!(p.parse_stmt().is_ok());
    {let mut b=ParserBridge::new(&mut p); b.skip_empty_lines();}
    assert!(p.parse_stmt().is_ok());
    assert_eq!(*seen.lock().unwrap(),vec![false,false]);
}
impl Dialect for TokenNone {
    fn matches(&self, _: &CompilerSession) -> bool { true }
    fn keywords(&self) -> &'static [&'static str] { &[] }
    fn try_parse_stmt(&self,_: &mut ParserBridge<'_, '_>,_: &str)->AutoResult<Option<Stmt>> {Ok(None)}
    fn try_parse_token_stmt(&self,b:&mut ParserBridge<'_, '_>,_:TokenKind)->AutoResult<Option<Stmt>> {
        *self.calls.lock().unwrap()+=1; b.advance(); Ok(None)
    }
}
#[test]
fn token_none_restores_cursor_for_builtin() {
    let calls=Arc::new(Mutex::new(0));
    let result=Parser::from("view { col { text \"hi\" } }")
      .with_session(CompilerSession::ui())
      .with_dialect(Box::new(TokenNone{calls:calls.clone()}),DialectPriority::Override).parse();
    println!("token_none_result={result:?}");
    assert!(result.is_ok()); assert!(*calls.lock().unwrap()>0);
}

struct TokenRetry { calls: Arc<Mutex<usize>> }
impl Dialect for TokenRetry {
    fn matches(&self,_:&CompilerSession)->bool {true}
    fn keywords(&self)->&'static [&'static str] {&[]}
    fn try_parse_stmt(&self,_:&mut ParserBridge<'_, '_>,_:&str)->AutoResult<Option<Stmt>> {Ok(None)}
    fn try_parse_token_stmt(&self,b:&mut ParserBridge<'_, '_>,_:TokenKind)->AutoResult<Option<Stmt>> {
        let mut n=self.calls.lock().unwrap(); *n+=1; b.advance();
        if *n==1 {Err(b.syntax_error("first token failure"))}
        else {Ok(Some(Stmt::Expr(Expr::Int(43))))}
    }
}
#[test]
fn token_err_keeps_registration_for_next_statement() {
    let calls=Arc::new(Mutex::new(0));
    let mut p=Parser::from("view\nview").with_session(CompilerSession::ui())
      .with_dialect(Box::new(TokenRetry{calls:calls.clone()}),DialectPriority::Override);
    assert!(p.parse_stmt().is_err());
    {let mut b=ParserBridge::new(&mut p); b.skip_empty_lines();}
    assert!(matches!(p.parse_stmt(),Ok(Stmt::Expr(Expr::Int(43)))));
    assert_eq!(*calls.lock().unwrap(),2);
}
