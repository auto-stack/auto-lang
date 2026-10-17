//! PLAN-801 R1 复审组合控制的永久回归（F-801-R1-01 闭合后收编）。
//!
//! 来源：独立复审反例（docs/plans/attachments/801/review-r1-snapshot.rs）
//! ——12 例：Ident/token 两路派发 × Some/None/Err 三态结局。前 6 例为
//! 无洞正控（不预置 tombstone）；后 6 例为**复用槽位反例**：外层先取并
//! restore 一张快照留下 tombstone，递归回调的内层调用复用该槽位签发
//! 凭据——长度水位清理漏掉它，过期凭据曾可恢复游标（child_rejected=
//! false）。修复=按签发作用域失效（`port_release_scope`）。
//!
//! oracle 三元组（每例断言）：
//! - `outer_restored`：外层存活快照在内层退出后仍有效并回到语句原位；
//! - `child_rejected`：内层调用退出的凭据必须受控拒绝（Err）；
//! - `cursor_unchanged`：拒绝不动当前游标。
//! Err 结局例中 lambda 体恢复可收集子错误后仍返回表达式——held 非空即
//! 证明子回调真实执行，不以 parse_expr 顶层 Err 为必要假设。

use std::sync::{Arc, Mutex};

use auto_lang::parser::Parser;
use auto_parser_api::ast::{Expr, Stmt};
use auto_parser_api::error::AutoResult;
use auto_parser_api::session::CompilerSession;
use auto_parser_api::token::TokenKind;
use auto_parser_api::{DialectPriority, ExternalDialect, ParseCheckpoint, ParseContext};

#[derive(Default)]
struct State {
    held: Option<ParseCheckpoint>,
    observed: Option<(bool, bool, bool)>,
}

struct ScopeProbe {
    state: Arc<Mutex<State>>,
    /// true = 预置 tombstone（外层先取再 restore 一张快照），制造内层
    /// 复用槽位的洞形态；false = 无洞正控。
    hole: bool,
    /// true = 子回调走 token 派发（TokenKind::View）；false = Ident("inner")。
    token: bool,
    /// 子回调结局：0=Some、1=None、2=Err。
    outcome: u8,
}

impl ScopeProbe {
    fn child(&self, c: &mut dyn ParseContext) -> AutoResult<Option<Stmt>> {
        self.state.lock().unwrap().held = Some(c.checkpoint());
        c.advance();
        match self.outcome {
            1 => Ok(None),
            2 => Err(c.syntax_error("review child error".into())),
            _ => Ok(Some(Stmt::Expr(Expr::Int(7)))),
        }
    }
}

impl ExternalDialect for ScopeProbe {
    fn matches(&self, _: &CompilerSession) -> bool {
        true
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["outer", "inner"]
    }
    fn try_parse_stmt(&self, c: &mut dyn ParseContext, kw: &str) -> AutoResult<Option<Stmt>> {
        if kw == "inner" {
            return self.child(c);
        }
        let outer = c.checkpoint();
        if self.hole {
            let consumed = c.checkpoint();
            c.restore(consumed)?;
        }
        c.expect_keyword("outer")?;
        // 委托核心解析：lambda 体（`outer () => { inner }` / `{ view }`）
        // 触发递归语句派发——子回调在此发生。
        let _delegated = c.parse_expr();
        let outer_ok = c.restore(outer).is_ok() && c.cur_text() == "outer";
        let child = self
            .state
            .lock()
            .unwrap()
            .held
            .take()
            .expect("child callback ran");
        let rejected = c.restore(child).is_err();
        let cursor_unchanged = c.cur_text() == "outer";
        self.state.lock().unwrap().observed = Some((outer_ok, rejected, cursor_unchanged));
        Ok(Some(Stmt::Expr(Expr::Int(99))))
    }
    fn try_parse_token_stmt(
        &self,
        c: &mut dyn ParseContext,
        k: TokenKind,
    ) -> AutoResult<Option<Stmt>> {
        if self.token && k == TokenKind::View {
            self.child(c)
        } else {
            Ok(None)
        }
    }
}

fn run(hole: bool, token: bool, outcome: u8) {
    let state = Arc::new(Mutex::new(State::default()));
    let source = if token {
        "outer () => { view }"
    } else {
        "outer () => { inner }"
    };
    let mut p = Parser::from(source).with_external_dialect(
        Box::new(ScopeProbe {
            state: state.clone(),
            hole,
            token,
            outcome,
        }),
        DialectPriority::Override,
    );
    assert!(matches!(p.parse_stmt(), Ok(Stmt::Expr(Expr::Int(99)))));
    let actual = state.lock().unwrap().observed.unwrap();
    assert_eq!(
        actual,
        (true, true, true),
        "hole={hole} token={token} outcome={outcome}: 外层快照存活、内层凭据过期受拒、游标不动"
    );
}

// ---- 无洞正控（不预置 tombstone；内层凭据同样必须在调用退出失效） ----

#[test]
fn r1_scope_ident_some_no_hole_control() {
    run(false, false, 0);
}
#[test]
fn r1_scope_ident_none_no_hole_control() {
    run(false, false, 1);
}
#[test]
fn r1_scope_ident_err_no_hole_control() {
    run(false, false, 2);
}
#[test]
fn r1_scope_token_some_no_hole_control() {
    run(false, true, 0);
}
#[test]
fn r1_scope_token_none_no_hole_control() {
    run(false, true, 1);
}
#[test]
fn r1_scope_token_err_no_hole_control() {
    run(false, true, 2);
}

// ---- 复用槽位反例（F-801-R1-01：曾可过期恢复，修复后必须全拒） ----

#[test]
fn r1_scope_ident_some_reused_slot_expires() {
    run(true, false, 0);
}
#[test]
fn r1_scope_ident_none_reused_slot_expires() {
    run(true, false, 1);
}
#[test]
fn r1_scope_ident_err_reused_slot_expires() {
    run(true, false, 2);
}
#[test]
fn r1_scope_token_some_reused_slot_expires() {
    run(true, true, 0);
}
#[test]
fn r1_scope_token_none_reused_slot_expires() {
    run(true, true, 1);
}
#[test]
fn r1_scope_token_err_reused_slot_expires() {
    run(true, true, 2);
}
