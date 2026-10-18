//! UI 方言：在 UI 场景下接管 widget/msg/model/view/on 关键字。
//!
//! 详见 `docs/design/dialect-extension-diagnosis.md` §6.1。
//!
//! - `widget`/`msg`/`model` 是普通标识符（`TokenKind::Ident`），走 `try_parse_stmt`。
//! - `view`/`on` 是真实 TokenKind（`TokenKind::View`/`TokenKind::On`），走 `try_parse_token_stmt`。
//!   其中 `view` 在 Core 语言里是参数模式关键字（`fn foo(view x int)`），
//!   UI 场景下在语句位置作为 view 块解析。
//!
//! PLAN-810：完整 UI 文法单源在 `auto-ui-dialect`（msg/model=PLAN-806
//! declarations，其余=grammar 模块）。本方言是**薄注册 adapter**——经
//! `ParserBridge` 的 `UiParseContext`/`ParseContext` 端口派发，不取回完整
//! Parser（escape hatch 已删除），不存在第二份算法。

use crate::ast::Stmt;
use crate::dialect::Dialect;
use crate::error::AutoResult;
use crate::parser_bridge::ParserBridge;
use crate::session::{CompilerSession, Scenario};
use crate::token::TokenKind;
use auto_ui_dialect::UiParseContext;

/// UI 方言：UI 场景下生效的关键字与语句解析。
pub struct UiDialect;

impl Dialect for UiDialect {
    fn matches(&self, s: &CompilerSession) -> bool {
        s.scenario == Scenario::UI
    }

    /// Ident 路径的关键字。不含 view/on —— 它们是真实 TokenKind，
    /// 走 try_parse_token_stmt。
    /// Plan 408: 加 `component`（`component fn` 独立组件声明）。
    /// Plan 451 P3: 加 `actions`（顶层动作声明——widget 块内 actions 的
    /// 模块级形态，可拆独立文件经 use 引用到宿主）。
    fn keywords(&self) -> &'static [&'static str] {
        &["widget", "msg", "model", "component", "actions", "style"]
    }

    fn try_parse_stmt(
        &self,
        bridge: &mut ParserBridge<'_, '_>,
        kw: &str,
    ) -> AutoResult<Option<Stmt>> {
        // Plan 028 T14：on 体里的语句不应被方言声明关键字拦截——
        // `msg.content = "x"` 是局部对象字段赋值，不是 msg 声明。
        if bridge.in_on_body() || bridge.in_fn_body() {
            return Ok(None);
        }
        let stmt = match kw {
            "widget" => auto_ui_dialect::grammar::widget::parse_widget_decl(bridge)?,
            // PLAN-806 单源声明文法（经 ParseContext 端口）。
            "msg" => Stmt::MsgDecl(auto_ui_dialect::parse_msg_decl(bridge)?),
            "model" => Stmt::ModelBlock(auto_ui_dialect::parse_model_block(bridge)?),
            // Plan 408: `component fn Name(...) { ... }` → 独立 Vue 组件声明。
            "component" => auto_ui_dialect::grammar::view::parse_component_fn_decl(bridge)?,
            // Plan 451 P3: 顶层 `actions { ... }`。仅当后继是 `{` 才接管——
            // `actions = ...` / `actions(...)` 等表达式用法回退原路径。
            "actions" => match auto_ui_dialect::grammar::actions::parse_actions_decl_stmt(bridge)? {
                Some(stmt) => stmt,
                None => return Ok(None),
            },
            // PLAN-607: 顶层 `style <name> = ...` 或 `style <name>(...) = ...`。
            "style" => {
                match auto_ui_dialect::grammar::actions::parse_style_recipe_decl_stmt(bridge)? {
                    Some(stmt) => stmt,
                    None => return Ok(None),
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(stmt))
    }

    fn try_parse_token_stmt(
        &self,
        bridge: &mut ParserBridge<'_, '_>,
        kind: TokenKind,
    ) -> AutoResult<Option<Stmt>> {
        match kind {
            // view 是 TokenKind::View。Plan 367 P2-3: `view fn` → view fragment；
            // 否则 view 块。文法侧自行判别（旧臂同形，PLAN-810 经端口）。
            TokenKind::View => Ok(Some(
                auto_ui_dialect::grammar::view::parse_view_block_or_fragment(bridge)?,
            )),
            // on 行首是 TokenKind::On，UI 场景仍解析为 OnEvents（行为不变）。
            // on-events 文法是 Core 共享（非 UI 回退同源，PLAN-810 归 Core），
            // 经 ParseContext 端口调用同一实现。
            TokenKind::On => Ok(Some(Stmt::OnEvents(bridge.parse_on_events()?))),
            _ => Ok(None),
        }
    }
}
