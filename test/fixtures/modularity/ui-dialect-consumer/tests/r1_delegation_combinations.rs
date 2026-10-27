//! PLAN-778 R1 复审组合控制的永久回归（F778-R1-01/02）。
//!
//! 来源：独立复审反例（docs/plans/attachments/778/review-r1/independent.rs）
//! ——修复后收编为永久组合控制，覆盖"外部回调委托期间的注册可见性"与
//! "委托 Err 后的 context 恢复"，含 Ident/token 两路派发与正控。
//!
//! UI model 使用显式领域组合；ContextProbe 保持非 UI 旧 trait 对照，
//! TokenNone/TokenRetry 使用 DomainDialect 委托与错误恢复控制。

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use auto_ast::ast::{Expr, Stmt};
use auto_lang::dialect::{Dialect, DialectPriority};
use auto_diagnostics::error::AutoResult;
use auto_lang_core::parser::Parser;
use auto_lang::parser_bridge::ParserBridge;
use auto_lang_core::session::CompilerSession;
use auto_lang::token::TokenKind;
use auto_parser_api::{DomainDialect, DomainParseContext, DomainStmt};
use auto_ui_dialect::{UiCode, UiDialect, UiStmt};
use dialect_api_consumer::{ExternalUiDialect, Probe};

/// 外部 UI model 只拥有顶层领域语句，不潜入 Core 闭包体。
#[test]
fn r1_nested_expression_keeps_active_registration() {
    let probe = Arc::new(Probe::default());
    let input = "model { child = () => { model { count int = 1 } } }";
    let result = Parser::from(input)
        .with_session(CompilerSession::ui())
        .parse_domain(&ExternalUiDialect::new(probe.clone()));
    assert!(result.is_err());
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}

/// Core 裸块同样不承接 UI model；后续顶层领域语句由显式组合接管。
#[test]
fn r1_sibling_nested_block_control() {
    let probe = Arc::new(Probe::default());
    assert!(Parser::from("{ model { count int = 1 } }")
        .with_session(CompilerSession::ui())
        .parse_domain(&ExternalUiDialect::new(probe.clone()))
        .is_err());
    let result = Parser::from("model { count int = 2 }")
        .with_session(CompilerSession::ui())
        .parse_domain(&ExternalUiDialect::new(probe.clone()));
    assert!(result.is_ok());
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
}

/// F778-R1-02 观察方言：每次派发记录 in_fn_body，`probe` 委托核心表达式解析。
struct ContextProbeDialect {
    seen: Arc<Mutex<Vec<bool>>>,
}

impl Dialect for ContextProbeDialect {
    fn matches(&self, _s: &CompilerSession) -> bool {
        true
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["probe", "tail"]
    }
    fn try_parse_stmt(&self, b: &mut ParserBridge<'_, '_>, kw: &str) -> AutoResult<Option<Stmt>> {
        self.seen.lock().unwrap().push(b.in_fn_body());
        b.advance();
        if kw == "probe" {
            Ok(Some(Stmt::Expr(b.parse_expr()?)))
        } else {
            Ok(Some(Stmt::Expr(Expr::Int(77))))
        }
    }
}

/// F778-R1-02 反例（P2）：委托核心解析（闭包内 fn 体缺 RBrace）Err 后，在
/// harness 显式恢复点（公开 checkpoint 回滚 + advance 到 tail）继续解析，
/// 恢复点派发观察的 in_fn_body 必须已恢复为 false。旧实现 fn 体
/// `body()?` 提前退出泄漏 true。
///
/// 期望序列 `[false, true, false]` 是 F778-R1-01 与 F778-R1-02 修复的联合
/// 语义（R1 复审冻结期望 [false,false] 编码的是 P1 未修——回调期间方言从
/// 表中消失、嵌套派发不可见——的行为）：
/// - `[0] false`：顶层 probe 派发；
/// - `[1] true`：probe 委托解析闭包体期间，`fn broken()` 体内的嵌套 tail
///   语句派发——P1 修复（Arc 留表）使其可见，此刻确在 fn 体内，true 是
///   **正确上下文**而非残留；
/// - `[2] false`：Err 边界恢复后、恢复点 tail 派发——P2 oracle（末位）。
#[test]
fn r1_delegated_error_restores_context_at_recovery_point() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let mut p = Parser::from("probe () => { fn broken() {\ntail").with_dialect(
        Box::new(ContextProbeDialect { seen: seen.clone() }),
        DialectPriority::Override,
    );
    let recovery = ParserBridge::new(&mut p).checkpoint();
    let result = p.parse_stmt();
    assert!(result.is_err(), "缺 RBrace 的 fn 体必须报错");
    // R2 收编更强 oracle：在 harness checkpoint restore **之前**直接观察
    // in_fn_body——证明 context 恢复来自派发 Err 边界，而非后续 restore 遮蔽
    // 了错误出口问题。
    assert!(
        !ParserBridge::new(&mut p).in_fn_body(),
        "context 必须在 checkpoint restore 之前已由 Err 边界恢复"
    );
    {
        let mut b = ParserBridge::new(&mut p);
        b.restore(recovery);
        while b.cur_kind() != TokenKind::EOF && b.cur_text() != "tail" {
            b.advance();
        }
        assert_eq!(b.cur_text(), "tail", "显式恢复点已到达");
    }
    assert!(p.parse_stmt().is_ok(), "恢复点后的合法语句必须解析成功");
    let seen = seen.lock().unwrap();
    assert_eq!(
        seen.last(),
        Some(&false),
        "P2 oracle：恢复点派发前 context 必须已恢复（末位 false）"
    );
    assert_eq!(
        *seen,
        vec![false, true, false],
        "联合语义：嵌套派发可见（P1）+ 恢复点 context 已恢复（P2）"
    );
}

/// F778-R1-02 正控：成功委托（fn 体完整）不泄漏 context——排除
/// "任何委托都污染 context"的解释。
#[test]
fn r1_successful_delegation_context_control() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let mut p = Parser::from("probe () => { fn valid() { } }\ntail").with_dialect(
        Box::new(ContextProbeDialect { seen: seen.clone() }),
        DialectPriority::Override,
    );
    assert!(p.parse_stmt().is_ok());
    {
        let mut b = ParserBridge::new(&mut p);
        b.skip_empty_lines();
    }
    assert!(p.parse_stmt().is_ok());
    assert_eq!(*seen.lock().unwrap(), vec![false, false]);
}

/// token 路径显式回滚后委托内建 UI view 文法。
struct TokenNoneDialect {
    calls: Arc<Mutex<usize>>,
}

impl DomainDialect for TokenNoneDialect {
    type Output = UiStmt;

    fn matches(&self, _s: &CompilerSession) -> bool {
        true
    }
    fn keywords(&self) -> &'static [&'static str] {
        UiDialect.keywords()
    }
    fn try_parse_stmt(
        &self,
        cx: &mut dyn DomainParseContext,
        keyword: &str,
    ) -> AutoResult<Option<DomainStmt<UiStmt>>> {
        UiDialect.try_parse_stmt(cx, keyword)
    }
    fn try_parse_token_stmt(
        &self,
        cx: &mut dyn DomainParseContext,
        kind: TokenKind,
    ) -> AutoResult<Option<DomainStmt<UiStmt>>> {
        *self.calls.lock().unwrap() += 1;
        let cp = cx.checkpoint();
        cx.advance();
        cx.restore(cp)?;
        UiDialect.try_parse_token_stmt(cx, kind)
    }
}

#[test]
fn r1_token_none_restores_cursor_for_builtin() {
    let calls = Arc::new(Mutex::new(0));
    let result = Parser::from("view { col { text \"hi\" } }")
        .with_session(CompilerSession::ui())
        .parse_domain(&TokenNoneDialect {
            calls: calls.clone(),
        });
    assert!(result.is_ok(), "回滚后内建 view 块必须照常解析");
    assert!(*calls.lock().unwrap() > 0, "token 方言必须真实被咨询");
}

/// token 路径 Err 后同一显式组合实例可供下一次解析使用。
struct TokenRetryDialect {
    calls: Arc<Mutex<usize>>,
}

impl DomainDialect for TokenRetryDialect {
    type Output = UiStmt;

    fn matches(&self, _s: &CompilerSession) -> bool {
        true
    }
    fn keywords(&self) -> &'static [&'static str] {
        &[]
    }
    fn try_parse_stmt(
        &self,
        _cx: &mut dyn DomainParseContext,
        _keyword: &str,
    ) -> AutoResult<Option<DomainStmt<UiStmt>>> {
        Ok(None)
    }
    fn try_parse_token_stmt(
        &self,
        cx: &mut dyn DomainParseContext,
        _kind: TokenKind,
    ) -> AutoResult<Option<DomainStmt<UiStmt>>> {
        let mut n = self.calls.lock().unwrap();
        *n += 1;
        cx.advance();
        if *n == 1 {
            Err(cx.syntax_error("first token failure".into()))
        } else {
            Ok(Some(DomainStmt::Core(Stmt::Expr(Expr::Int(43)))))
        }
    }
}

#[test]
fn r1_token_err_keeps_registration_for_next_statement() {
    let calls = Arc::new(Mutex::new(0));
    let dialect = TokenRetryDialect {
        calls: calls.clone(),
    };
    assert!(Parser::from("view")
        .with_session(CompilerSession::ui())
        .parse_domain(&dialect)
        .is_err());
    let second = Parser::from("view")
        .with_session(CompilerSession::ui())
        .parse_domain(&dialect)
        .map(UiCode)
        .unwrap();
    assert!(matches!(
        second.core_stmts().next(),
        Some(Stmt::Expr(Expr::Int(43)))
    ));
    assert_eq!(*calls.lock().unwrap(), 2);
}
