//! store 声明文法（PLAN-810 T-03 迁移）。msg/model 声明复用 806
//! declarations 单源（经端口直调，不复制第二份算法）。

use super::actions::parse_timer_block_inner;
use super::widget::{parse_computed_block_inner, parse_on_block, parse_watch_block_inner};
use crate::UiParseContext;
use auto_parser_api::ast;
use auto_parser_api::ast::*;
use auto_parser_api::error::{pos_to_span, AutoResult, SyntaxError};
use auto_parser_api::token::TokenKind;

/// Parse a `store` declaration (Plan 351 / Design 18).
/// A store is a view-less widget: `store Name { model{} msg{} on{} }`.
pub fn parse_store_decl(cx: &mut dyn UiParseContext) -> AutoResult<Stmt> {
    cx.expect_ident("store")?;
    let name = cx.cur_text();
    cx.advance();

    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut messages = Vec::new();
    let mut model = None;
    let mut on = None;
    let mut computed = None;
    let mut watch = Vec::new();
    let mut timer: Option<ast::ui::TimerBlock> = None;

    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }
        let ident = cx.cur_text();
        match ident.as_str() {
            "msg" => {
                messages.push(crate::declarations::parse_msg_decl(cx)?);
            }
            "model" => {
                model = Some(crate::declarations::parse_model_block(cx)?);
            }
            "computed" => {
                computed = Some(parse_computed_block_inner(cx)?);
            }
            "on" => {
                on = Some(parse_on_block(cx)?);
            }
            // Plan 012 Batch G (gap 12): store-level `watch { ... }` —
            // same syntax/semantics as the widget-level block, emitted
            // as module-level watch() calls in the store composable.
            "watch" => {
                watch.extend(parse_watch_block_inner(cx)?);
            }
            // Plan 051 C7: store-level `timer { ... }`（应用生命周期）。
            "timer" => {
                if timer.replace(parse_timer_block_inner(cx)?).is_some() {
                    return Err(SyntaxError::Generic {
                        message: "duplicate `timer` block in store".into(),
                        span: pos_to_span(cx.cur_pos()),
                    }
                    .into());
                }
            }
            _ => {
                return Err(SyntaxError::Generic {
                        message: format!(
                            "Expected 'model', 'msg', 'computed', 'on', 'watch', or 'timer' in store, got '{}'",
                            ident
                        ),
                        span: pos_to_span(cx.cur_pos()),
                    }
                    .into());
            }
        }
        cx.skip_empty_lines();
    }
    cx.expect(TokenKind::RBrace)?;

    // Plan 051 C7: timer 条目头必须命中本 store 的 msg{} 变体（同
    // widget 校验承诺）。
    if let Some(ref tb) = timer {
        let variants: std::collections::HashSet<String> = messages
            .iter()
            .flat_map(|m| m.variants.iter().map(|v| v.name.to_string()))
            .collect();
        for e in &tb.entries {
            if !variants.contains(e.event.as_str()) {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "timer: event '{}' has no matching msg variant in store {}",
                        e.event, name
                    ),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
        }
    }

    Ok(Stmt::StoreDecl(StoreDecl {
        name: name.into(),
        messages,
        model,
        computed,
        on,
        watch,
        timer,
    }))
}
