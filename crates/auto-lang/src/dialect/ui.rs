//! UI 方言：在 UI 场景下接管 widget/msg/model/view/on 关键字。
//!
//! 详见 `docs/design/dialect-extension-diagnosis.md` §6.1。
//!
//! - `widget`/`msg`/`model` 是普通标识符（`TokenKind::Ident`），走 `try_parse_stmt`。
//! - `view`/`on` 是真实 TokenKind（`TokenKind::View`/`TokenKind::On`），走 `try_parse_token_stmt`。
//!   其中 `view` 在 Core 语言里是参数模式关键字（`fn foo(view x int)`），
//!   UI 场景下在语句位置作为 view 块解析。

use crate::ast::Stmt;
use crate::dialect::Dialect;
use crate::error::AutoResult;
use crate::parser::Parser;
use crate::session::{CompilerSession, Scenario};
use crate::token::TokenKind;

/// UI 方言：UI 场景下生效的关键字与语句解析。
pub struct UiDialect;

impl Dialect for UiDialect {
    fn matches(&self, s: &CompilerSession) -> bool {
        s.scenario == Scenario::UI
    }

    /// Ident 路径的关键字。不含 view/on —— 它们是真实 TokenKind，
    /// 走 try_parse_token_stmt。
    fn keywords(&self) -> &'static [&'static str] {
        &["widget", "msg", "model"]
    }

    fn try_parse_stmt(&self, p: &mut Parser, kw: &str) -> AutoResult<Option<Stmt>> {
        let stmt = match kw {
            "widget" => p.parse_widget_decl()?,
            "msg" => p.parse_msg_decl()?,
            "model" => p.parse_model_block()?,
            _ => return Ok(None),
        };
        Ok(Some(stmt))
    }

    fn try_parse_token_stmt(
        &self,
        p: &mut Parser,
        kind: TokenKind,
    ) -> AutoResult<Option<Stmt>> {
        match kind {
            // view 是 TokenKind::View（Core 参数模式关键字），UI 场景下作为 view 块。
            TokenKind::View => Ok(Some(p.parse_view_block()?)),
            // on 行首是 TokenKind::On，UI 场景仍解析为 OnEvents（行为不变）。
            TokenKind::On => Ok(Some(Stmt::OnEvents(p.parse_on_events()?))),
            _ => Ok(None),
        }
    }
}
