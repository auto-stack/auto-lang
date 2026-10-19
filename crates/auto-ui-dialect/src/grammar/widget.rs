//! widget 声明文法（PLAN-810 T-03 迁移）：widget 壳/props/use/setup/
//! expose/watch/computed/on/bind 块与内联 handler mint 族。

use super::actions::{
    parse_actions_block_inner, parse_routes_block_inner, parse_timer_block_inner,
};
use super::helpers::{
    damerau_levenshtein_1, direct_state_value_field, hover_card_role, modal_dialog_tag_role,
    on_handler_event_name, stmt_expr_contains_await, wire_modal_close_recursive,
};
use super::view::{parse_handler_params, parse_view_node, parse_view_root_nodes};
use crate::UiParseContext;
use auto_parser_api::ast;
use auto_parser_api::ast::*;
use auto_parser_api::error::{pos_to_span, AutoResult, SyntaxError};
use auto_parser_api::token::TokenKind;

pub fn parse_widget_decl(cx: &mut dyn UiParseContext) -> AutoResult<Stmt> {
    cx.expect_ident("widget")?;
    let name = cx.cur_text();
    cx.advance();

    // Parse props if present: widget Name(prop: type, ...)
    let props = if cx.is_kind(TokenKind::LParen) {
        parse_widget_props(cx)?
    } else {
        Vec::new()
    };

    // Parse body
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut messages = Vec::new();
    let mut model = None;
    let mut computed = None;
    let mut view = None;
    let mut named_views: Vec<(Name, ViewBlock)> = Vec::new();
    let mut on = None;
    let mut bind = None;
    let mut routes = None;
    let mut style = None;
    let mut ext_imports = Vec::new();
    let mut watch = Vec::new();
    let mut expose = Vec::new();
    let mut setup: Option<ast::ui::SetupBlock> = None;
    let mut actions: Option<ast::ui::ActionsBlock> = None;
    let mut timer: Option<ast::ui::TimerBlock> = None;

    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }

        // Plan 425 复审:owned——fallback 分支的近失告警需要 &mut self,
        // 与 self.cur 的不可变借用冲突。
        let ident = cx.cur_text().as_str().to_string();
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
            "view" => {
                // PLAN-024: `view` 后 peek 分派——`{` → 主视图（重复声明
                // 报错，原为静默覆盖）；Ident → 命名视图 `view mini { … }`
                // （同名报错，样式沿 setup/actions 臂）。`fn` 是独立关键
                // 字 token，`view fn` 片段仍只在顶层位分派（widget 体内
                // 维持原报错行为），`fn f(view x)` 参数位不进此臂。
                cx.expect_ident("view")?;
                if cx.is_kind(TokenKind::LBrace) {
                    cx.advance(); // consume `{` — parse_view_root_nodes expects it gone
                    let root = parse_view_root_nodes(cx)?;
                    if view.replace(ViewBlock { root }).is_some() {
                        return Err(SyntaxError::Generic {
                            message: "duplicate `view` block in widget".into(),
                            span: pos_to_span(cx.cur_pos()),
                        }
                        .into());
                    }
                } else if cx.is_kind(TokenKind::Ident) {
                    let vname = cx.cur_text();
                    cx.advance();
                    cx.expect(TokenKind::LBrace)?;
                    let root = parse_view_root_nodes(cx)?;
                    if named_views
                        .iter()
                        .any(|(n, _)| n.as_str() == vname.as_str())
                    {
                        return Err(SyntaxError::Generic {
                            message: format!("duplicate named view `view {}` in widget", vname),
                            span: pos_to_span(cx.cur_pos()),
                        }
                        .into());
                    }
                    named_views.push((vname.into(), ViewBlock { root }));
                } else {
                    return Err(SyntaxError::Generic {
                        message: format!(
                            "Expected '{{' or a view name after `view` in widget, got '{}'",
                            cx.cur_text()
                        ),
                        span: pos_to_span(cx.cur_pos()),
                    }
                    .into());
                }
            }
            "on" => {
                on = Some(parse_on_block(cx)?);
            }
            "bind" => {
                bind = Some(parse_bind_block(cx)?);
            }
            "routes" => {
                routes = Some(parse_routes_block_inner(cx)?);
            }
            "style" => {
                style = Some(parse_style_block_inner(cx)?);
            }
            "use" => {
                ext_imports.extend(parse_widget_use_block_inner(cx)?);
            }
            "watch" => {
                watch.extend(parse_watch_block_inner(cx)?);
            }
            // Plan 426: setup 前导槽——位置任意,语义固定(setup 相位)。
            "setup" => {
                if setup.replace(parse_setup_block_inner(cx)?).is_some() {
                    return Err(SyntaxError::Generic {
                        message: "duplicate `setup` block in widget".into(),
                        span: pos_to_span(cx.cur_pos()),
                    }
                    .into());
                }
            }
            "expose" => {
                expose.extend(parse_expose_block_inner(cx)?);
            }
            // Plan 451: actions 声明块（动作注册表 + menubar/toolbar）。
            "actions" => {
                if actions.replace(parse_actions_block_inner(cx)?).is_some() {
                    return Err(SyntaxError::Generic {
                        message: "duplicate `actions` block in widget".into(),
                        span: pos_to_span(cx.cur_pos()),
                    }
                    .into());
                }
            }
            // Plan 051 C7: timer 声明块（周期计时器）。
            "timer" => {
                if timer.replace(parse_timer_block_inner(cx)?).is_some() {
                    return Err(SyntaxError::Generic {
                        message: "duplicate `timer` block in widget".into(),
                        span: pos_to_span(cx.cur_pos()),
                    }
                    .into());
                }
            }
            _ => {
                // Plan 425: view 可选化——widget 体以元素开头(无 view 块)
                // 时体即视图:首个非块关键字标识符按视图元素解析并自动
                // 包裹为 view。已有 view 块则报错(单视图)。
                // Plan 425 复审债务修复:与块关键字拼写近失(距离 1,含
                // 相邻换位)且后随 `{` 的标识符告警——否则 `veiw {}` 会
                // 静默变成一个名为 veiw 的视图元素。
                // 单 token 前看(ident 后是否紧跟 `{`)——peek() 返回的是
                // 当前 token,真前看用 lexer save/restore(Plan 395 模式)。
                let cp = cx.raw_cursor_save();
                cx.advance();
                let next_is_brace = cx.is_kind(TokenKind::LBrace);
                cx.raw_cursor_restore(cp)?;
                if next_is_brace {
                    const BLOCK_KEYWORDS: [&str; 13] = [
                        "msg", "model", "computed", "view", "on", "style", "use", "watch",
                        "expose", "routes", "setup", "actions", "timer",
                    ];
                    if let Some(kw) = BLOCK_KEYWORDS
                        .iter()
                        .find(|kw| damerau_levenshtein_1(ident.as_str(), kw))
                    {
                        cx.warn_suspicious_block_keyword(ident.to_string(), kw.to_string());
                    }
                }
                if view.is_none() {
                    view = Some(ViewBlock {
                        root: parse_view_node(cx)?,
                    });
                } else {
                    return Err(SyntaxError::Generic {
                            message: format!(
                                "Expected 'msg', 'model', 'computed', 'view', 'on', 'style', 'use', 'watch', 'expose', 'routes', or a view element in widget, got '{}'",
                                ident
                            ),
                            span: pos_to_span(cx.cur_pos()),
                        }.into());
                }
            }
        }
        cx.skip_empty_lines();
    }

    cx.expect(TokenKind::RBrace)?;

    // Plan 448 B1: mint widget-own inline lambda handlers right after the
    // decl is complete — single source of truth for extraction AND the
    // decl-based VM synthesis path (see mint_inline_event_handlers).
    let mut decl = WidgetDecl {
        name: name.into(),
        messages,
        model,
        computed,
        view,
        named_views,
        on,
        bind,
        props,
        routes,
        lifecycle: vec![],
        style,
        ext_imports,
        watch,
        setup,
        expose,
        actions,
        timer,
    };
    mint_inline_event_handlers(&mut decl)?;
    // Plan 051 C7: timer 条目头必须命中本 widget 的 msg{} 变体——
    // 编译期校验（沿 Plan 451 actions handler 校验承诺）。
    if let Some(ref tb) = decl.timer {
        let variants: std::collections::HashSet<String> = decl
            .messages
            .iter()
            .flat_map(|m| m.variants.iter().map(|v| v.name.to_string()))
            .collect();
        for e in &tb.entries {
            if !variants.contains(e.event.as_str()) {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "timer: event '{}' has no matching msg variant in widget {}",
                        e.event, decl.name
                    ),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
        }
    }
    // Plan 451: actions 块的 handler 必须命中本 widget 的 on{} 事件——
    // 编译期校验（auto-atom 文件形态要运行时才对得上）。
    if let Some(ref acts) = decl.actions {
        let handler_names: std::collections::HashSet<String> = decl
            .on
            .iter()
            .flat_map(|o| o.handlers.iter())
            .map(|h| on_handler_event_name(&h.pattern))
            .collect();
        for a in &acts.actions {
            let target = on_handler_event_name(&a.handler);
            if !handler_names.contains(&target) {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "actions: handler '{}' has no matching `on` handler in widget {}",
                        a.handler, decl.name
                    ),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
        }
    }
    Ok(Stmt::WidgetDecl(decl))
}

/// Plan 448 B1: mint `onclick: () => { … }` inline-lambda event handlers
/// into explicit msg/on form, in place, right after a widget-shaped decl
/// is parsed.
///
/// Each ViewEvent carrying an `inline` body is rewritten to reference a
/// synthetic `__evt_<event>_<n>` (n counts depth-first in view order —
/// deterministic across parses), an `OnHandler` with the lambda body is
/// appended to the decl's `on` block (created if absent), and a unit
/// `MsgVariant` is appended to the msg list (created if absent).
///
/// Minting HERE is load-bearing: extraction builds view/handlers/variants
/// from the decl (Vue/Rust/a2ui paths), while the iced VM runtime
/// synthesizes handlers STRAIGHT from the decl's on-block
/// (`VmBridge::new_from_decls`) — minting only at extraction left the VM
/// without the functions and clicks silently no-oped. View-fn fragments
/// cannot mint here (their expansion context only exists at extraction);
/// they mint there under the `__evtf_` prefix instead.
pub(crate) fn mint_inline_event_handlers(decl: &mut WidgetDecl) -> AutoResult<()> {
    let Some(view) = decl.view.as_mut() else {
        return Ok(());
    };
    let mut counter = 0u32;
    let mut minted: Vec<(String, Vec<Stmt>)> = Vec::new();
    mint_view_inline(&mut view.root, &mut counter, &mut minted)?;
    mint_bare_input_sync(&decl.name, &mut view.root, &mut counter, &mut minted)?;
    let dlg_states = mint_modal_dialog_toggle(&mut view.root, &mut counter, &mut minted)?;
    // PLAN-533 T5: 铸造的 `__dlg_open_<n>` state 落 model 块（缺失则建）。
    if !dlg_states.is_empty() {
        let model = decl
            .model
            .get_or_insert_with(|| ModelBlock { fields: Vec::new() });
        for name in dlg_states {
            if !model.fields.iter().any(|f| f.name.as_str() == name) {
                model.fields.push(ModelField {
                    name: Name::from(name.as_str()),
                    ty: Type::Bool,
                    init: Expr::Bool(false),
                    mutable: true,
                    is_primary: false,
                    decorators: Vec::new(),
                });
            }
        }
    }
    if minted.is_empty() {
        return Ok(());
    }
    for (pattern, stmts) in minted {
        let variant_name = pattern.trim_start_matches('.').to_string();
        let declared = decl
            .messages
            .iter()
            .any(|m| m.variants.iter().any(|v| v.name.as_str() == variant_name));
        if !declared {
            if decl.messages.is_empty() {
                decl.messages.push(MsgDecl {
                    variants: Vec::new(),
                });
            }
            decl.messages[0].variants.push(MsgVariant {
                name: Name::from(variant_name.as_str()),
                quoted: false,
                payload: Vec::new(),
                payload_names: Vec::new(),
            });
        }
        let mut body = Body::new();
        body.stmts = stmts;
        let on = decl.on.get_or_insert_with(|| OnBlock {
            handlers: Vec::new(),
        });
        on.handlers.push(OnHandler {
            pattern,
            params: Vec::new(),
            body,
            stream: None,
        });
    }
    Ok(())
}

pub(crate) fn mint_view_inline(
    node: &mut ViewNode,
    counter: &mut u32,
    minted: &mut Vec<(String, Vec<Stmt>)>,
) -> AutoResult<()> {
    match node {
        ViewNode::Element {
            events, children, ..
        } => {
            mint_events_inline(events, counter, minted)?;
            for c in children.iter_mut() {
                mint_view_inline(c, counter, minted)?;
            }
        }
        ViewNode::ForLoop { body, .. } => {
            for c in body.iter_mut() {
                mint_view_inline(c, counter, minted)?;
            }
        }
        ViewNode::Conditional {
            then_body,
            else_body,
            ..
        } => {
            for c in then_body.iter_mut() {
                mint_view_inline(c, counter, minted)?;
            }
            if let Some(else_nodes) = else_body {
                for c in else_nodes.iter_mut() {
                    mint_view_inline(c, counter, minted)?;
                }
            }
        }
        ViewNode::Component { events, .. } => {
            mint_events_inline(events, counter, minted)?;
        }
        ViewNode::Link { children, .. } => {
            for c in children.iter_mut() {
                mint_view_inline(c, counter, minted)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn mint_events_inline(
    events: &mut [ViewEvent],
    counter: &mut u32,
    minted: &mut Vec<(String, Vec<Stmt>)>,
) -> AutoResult<()> {
    for ev in events.iter_mut() {
        if let Some(stmts) = ev.inline.take() {
            if !ev.params.is_empty() {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "inline event lambdas with parameters are not supported yet \
                             (`{}: ({}) => …`); declare a msg variant and use `on`",
                        ev.name,
                        ev.params.join(", ")
                    ),
                    span: pos_to_span(Pos {
                        line: 0,
                        at: 0,
                        pos: 0,
                        len: 0,
                    }),
                }
                .into());
            }
            *counter += 1;
            // Sanitize the event name (modifiers like `onclick.stop` and
            // quoted customs like `on"x"` are not identifier-safe).
            let base: String = ev
                .name
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect();
            let pattern = format!(".__evt_{}_{}", base, counter);
            ev.handler = pattern.clone();
            ev.params.clear();
            minted.push((pattern, stmts));
        }
    }
    Ok(())
}

/// Plan 448 C: bare `value:` two-way binding. An `input`/`textarea`
/// Element whose `value:` targets a direct state field and that wires no
/// oninput/onchange event gets an empty-body minted `oninput` handler
/// (`.__evt_oninput_<n>`), so typed text reaches state on every backend:
/// Vue folds value+@input into v-model, the iced/VM runtime writes the
/// bound field via input_state_map before dispatching, and generated
/// Rust injects via input_fields. Without it, a bare `value:` input only
/// round-trips on Vue — iced wires on_change solely when a handler
/// exists, so edits never persist. Multi-level dots (`.store.me.image`)
/// stay one-way: every writeback path targets direct state fields only.
/// Registry Component nodes are out (they carry their own value/change
/// protocols); view-fn fragments keep the B v1 boundary (no minting).
pub(crate) fn mint_bare_input_sync(
    widget_name: &str,
    node: &mut ViewNode,
    counter: &mut u32,
    minted: &mut Vec<(String, Vec<Stmt>)>,
) -> AutoResult<()> {
    match node {
        ViewNode::Element {
            tag,
            props,
            events,
            children,
            ..
        } => {
            if matches!(tag.as_str(), "input" | "textarea")
                && direct_state_value_field(props).is_some()
                && !events.iter().any(|ev| {
                    let base = ev.name.split('.').next().unwrap_or("");
                    matches!(base, "oninput" | "onchange" | "input" | "change")
                })
            {
                *counter += 1;
                // `__bind_` (not `__evt_`) marks an auto-sync mint: the
                // body is empty by construction and Vue realizes the
                // sync via its v-model fold, so downstream consumers can
                // suppress the redundant empty handler wholesale. The
                // widget name is woven in because the registry-wide
                // input_state_map is keyed by handler name alone — two
                // sibling children each minting `__bind_oninput_1` would
                // collide and the second binding would silently no-op.
                let owner: String = widget_name
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                    .collect();
                let pattern = format!(".__bind_{}_oninput_{}", owner, counter);
                events.push(ViewEvent {
                    name: "oninput".to_string(),
                    handler: pattern.clone(),
                    params: Vec::new(),
                    inline: None,
                });
                minted.push((pattern, Vec::new()));
            }
            for c in children.iter_mut() {
                mint_bare_input_sync(widget_name, c, counter, minted)?;
            }
        }
        ViewNode::ForLoop { body, .. } => {
            for c in body.iter_mut() {
                mint_bare_input_sync(widget_name, c, counter, minted)?;
            }
        }
        ViewNode::Conditional {
            then_body,
            else_body,
            ..
        } => {
            for c in then_body.iter_mut() {
                mint_bare_input_sync(widget_name, c, counter, minted)?;
            }
            if let Some(else_nodes) = else_body {
                for c in else_nodes.iter_mut() {
                    mint_bare_input_sync(widget_name, c, counter, minted)?;
                }
            }
        }
        ViewNode::Link { children, .. } => {
            for c in children.iter_mut() {
                mint_bare_input_sync(widget_name, c, counter, minted)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// PLAN-533 T5: 模态对话框自管开合铸造。
///
/// alert-dialog/dialog 根无 `open` prop 时（vue 风格内建开合——
/// gallery dialog 页形态），三轨没有共享的内建 open 态。铸造一组
/// 显式装置让 vue/解释器/rust codegen 同源：
///   state   `__dlg_open_<n> bool = false`（调用方落 model 块）
///   root    补 `open: .__dlg_open_<n>`
///   trigger 无 onclick → 补 `onclick: .__dlg_toggle_<n>`（体：翻转）
///   cancel/action/close 无 onclick → 补 `onclick: .__dlg_close_<n>`（体：置 false）
/// handler 体经 `minted` 折叠进 decl.on + decl.messages（与 __evt_/
/// __bind_ 铸造同槽）。显式 `open` 绑定不铸造。
pub(crate) fn mint_modal_dialog_toggle(
    node: &mut ViewNode,
    counter: &mut u32,
    minted: &mut Vec<(String, Vec<Stmt>)>,
) -> AutoResult<Vec<String>> {
    let mut states = Vec::new();
    mint_modal_dialog_toggle_inner(node, counter, minted, &mut states)?;
    Ok(states)
}

pub(crate) fn mint_modal_dialog_toggle_inner(
    node: &mut ViewNode,
    counter: &mut u32,
    minted: &mut Vec<(String, Vec<Stmt>)>,
    states: &mut Vec<String>,
) -> AutoResult<()> {
    match node {
        ViewNode::Element {
            tag,
            props,
            events,
            children,
            ..
        } => {
            // PLAN-534 D4: hovercard 根——同一 __dlg_open_N 铸造机器,
            // 但 trigger 无显式事件时补 hover 进/出（onmouseenter→置
            // true / onmouseleave→置 false）而非 onclick toggle/close
            // （非模态即时开合;open/close-delay v1 不消费,KNOWN-DEBT）。
            // 显式绑定不覆盖（逐事件判定,同 toggle 规则）。事件落
            // trigger 自身节点——转换臂（convert_hovercard）从 trigger
            // events 读进 MouseArea,落内层会丢（与 toggle 的包裹规则
            // 相反,因渲染面是 MouseArea 包转换后的 anchor）。
            if hover_card_role(tag) == Some("root") && !props.iter().any(|p| p.name == "open") {
                *counter += 1;
                let n = *counter;
                let state_name = format!("__dlg_open_{n}");
                let enter = format!("__dlg_enter_{n}");
                let leave = format!("__dlg_leave_{n}");
                let state_ref = |name: &str| {
                    Expr::Dot(Box::new(Expr::Ident(Name::from("self"))), Name::from(name))
                };
                props.push(ViewProp {
                    name: "open".to_string(),
                    value: ViewPropValue::Expr(state_ref(&state_name)),
                });
                // enter 体：`.__dlg_open_<n> = true`
                let enter_body = Stmt::Expr(Expr::Bina(
                    Box::new(state_ref(&state_name)),
                    Op::Asn,
                    Box::new(Expr::Bool(true)),
                ));
                // leave 体：`.__dlg_open_<n> = false`
                let leave_body = Stmt::Expr(Expr::Bina(
                    Box::new(state_ref(&state_name)),
                    Op::Asn,
                    Box::new(Expr::Bool(false)),
                ));
                minted.push((format!(".{enter}"), vec![enter_body]));
                minted.push((format!(".{leave}"), vec![leave_body]));
                states.push(state_name);
                for c in children.iter_mut() {
                    if let ViewNode::Element {
                        tag: ctag,
                        events: cev,
                        ..
                    } = c
                    {
                        if hover_card_role(ctag) == Some("trigger") {
                            if !cev
                                .iter()
                                .any(|e| matches!(e.name.as_str(), "onmouseenter" | "onhover"))
                            {
                                cev.push(ViewEvent {
                                    name: "onmouseenter".to_string(),
                                    handler: format!(".{enter}"),
                                    params: Vec::new(),
                                    inline: None,
                                });
                            }
                            if !cev
                                .iter()
                                .any(|e| matches!(e.name.as_str(), "onmouseleave" | "onhoverout"))
                            {
                                cev.push(ViewEvent {
                                    name: "onmouseleave".to_string(),
                                    handler: format!(".{leave}"),
                                    params: Vec::new(),
                                    inline: None,
                                });
                            }
                        }
                    }
                }
            }
            if modal_dialog_tag_role(tag) == Some("root") && !props.iter().any(|p| p.name == "open")
            {
                *counter += 1;
                let n = *counter;
                let state_name = format!("__dlg_open_{n}");
                let toggle = format!("__dlg_toggle_{n}");
                let close = format!("__dlg_close_{n}");
                // root 补 open 绑定：与真实 `.state` 解析同形态
                //（Dot(Ident("self"), field)——见 direct_state_value_field 注）。
                let state_ref = |name: &str| {
                    Expr::Dot(Box::new(Expr::Ident(Name::from("self"))), Name::from(name))
                };
                props.push(ViewProp {
                    name: "open".to_string(),
                    value: ViewPropValue::Expr(state_ref(&state_name)),
                });
                // toggle 体：`.__dlg_open_<n> = !.__dlg_open_<n>`
                let toggle_body = Stmt::Expr(Expr::Bina(
                    Box::new(state_ref(&state_name)),
                    Op::Asn,
                    Box::new(Expr::Unary(Op::Not, Box::new(state_ref(&state_name)))),
                ));
                // close 体：`.__dlg_open_<n> = false`
                let close_body = Stmt::Expr(Expr::Bina(
                    Box::new(state_ref(&state_name)),
                    Op::Asn,
                    Box::new(Expr::Bool(false)),
                ));
                minted.push((format!(".{toggle}"), vec![toggle_body]));
                minted.push((format!(".{close}"), vec![close_body]));
                states.push(state_name);
                // 子件接线：trigger（直接子）→ toggle;cancel/action/close
                // （content 内任意深度）→ close。
                let is_click = |name: &str| matches!(name, "onclick" | "onClick" | "on_click");
                for c in children.iter_mut() {
                    if let ViewNode::Element {
                        tag: ctag,
                        events: cev,
                        children: cch,
                        ..
                    } = c
                    {
                        if modal_dialog_tag_role(ctag) == Some("trigger") {
                            // 包裹形态（trigger 内单 Element 子,如包住的
                            // button）且内层无 onclick → toggle 落内层按钮
                            // （渲染面只画子件,落 wrapper 会丢）;否则落
                            // trigger 自身（裸文本形态由转换层读）。
                            let target: &mut Vec<ViewEvent> = if cch.len() == 1 {
                                match &mut cch[0] {
                                    ViewNode::Element {
                                        events: inner_ev, ..
                                    } if !inner_ev.iter().any(|e| is_click(e.name.as_str())) => {
                                        inner_ev
                                    }
                                    _ => cev,
                                }
                            } else {
                                cev
                            };
                            if !target.iter().any(|e| is_click(e.name.as_str())) {
                                target.push(ViewEvent {
                                    name: "onclick".to_string(),
                                    handler: format!(".{toggle}"),
                                    params: Vec::new(),
                                    inline: None,
                                });
                            }
                        }
                    }
                    wire_modal_close_recursive(c, &close);
                }
            }
            for c in children.iter_mut() {
                mint_modal_dialog_toggle_inner(c, counter, minted, states)?;
            }
        }
        ViewNode::ForLoop { body, .. } => {
            for c in body.iter_mut() {
                mint_modal_dialog_toggle_inner(c, counter, minted, states)?;
            }
        }
        ViewNode::Conditional {
            then_body,
            else_body,
            ..
        } => {
            for c in then_body.iter_mut() {
                mint_modal_dialog_toggle_inner(c, counter, minted, states)?;
            }
            if let Some(else_nodes) = else_body {
                for c in else_nodes.iter_mut() {
                    mint_modal_dialog_toggle_inner(c, counter, minted, states)?;
                }
            }
        }
        ViewNode::Link { children, .. } => {
            for c in children.iter_mut() {
                mint_modal_dialog_toggle_inner(c, counter, minted, states)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Parse a widget-level external import block (Vue backend escape hatch):
///
/// ```auto
/// widget Icon(language: str) {
///     use {
///         fn: getLanguageIconUrl from "src/front/utils/codeBlockLanguage.ts"
///         fn: a, b from "marked"
///         component: FancyBadge from "src/front/components/FancyBadge.vue"
///         composable: useClock from "src/front/composables/useClock.ts"
///     }
/// }
/// ```
///
/// Each entry declares hand-written TS/Vue symbols the generated SFC may
/// reference. Paths are npm package specifiers (paired with pac.at
/// `npm_deps:`) or project-root-relative files (copied into the generated
/// project under `src/ext/` by auto-man). See [`ExtImport`].
pub(crate) fn parse_widget_use_block_inner(
    cx: &mut dyn UiParseContext,
) -> AutoResult<Vec<ExtImport>> {
    cx.expect_ident("use")?;
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut imports = Vec::new();
    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }

        let kind = match cx.cur_text().as_str() {
            "fn" => ExtImportKind::Fn,
            "component" => ExtImportKind::Component,
            "composable" => ExtImportKind::Composable,
            // Plan 435 P4:包引用 —— `package: official from "../components"`
            "package" => ExtImportKind::Package,
            other => {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Expected 'fn', 'component', or 'composable' in widget use block, got '{}'",
                        other
                    ),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
        };
        cx.advance();
        cx.expect(TokenKind::Colon)?;

        let mut symbols = Vec::new();
        loop {
            if !cx.is_kind(TokenKind::Ident) {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Expected symbol name in widget use block, got '{}'",
                        cx.cur_text()
                    ),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
            symbols.push(cx.cur_text().into());
            cx.advance();
            if cx.is_kind(TokenKind::Comma) {
                cx.advance();
                continue;
            }
            break;
        }

        // Plan 022: 可选调用参数（仅 composable 实际消费）：
        // `composable: useX(.arg) from "..."`。符号列表之后、from 之前，
        // 若跟 `(` 则解析参数列表。fn/component 声明也允许此语法但不消费。
        let mut call_args: Vec<ast::Expr> = Vec::new();
        if cx.is_kind(TokenKind::LParen) {
            cx.advance();
            cx.skip_empty_lines();
            while !cx.is_kind(TokenKind::RParen) {
                cx.skip_empty_lines();
                if cx.is_kind(TokenKind::RParen) {
                    break;
                }
                let arg = cx.parse_expr()?;
                call_args.push(arg);
                if cx.is_kind(TokenKind::Comma) {
                    cx.advance();
                }
            }
            cx.expect(TokenKind::RParen)?;
        }

        // Plan 408 P12 §10.4: 可选 ref 字段标注（仅 composable 消费）：
        // `composable: useX(refs: [currentSecretary, badgeCount]) from "..."`。
        // 列出的字段在 script 表达式里加 `.value`。fn/component 忽略。
        let mut ref_fields: Vec<Name> = Vec::new();
        if cx.cur_text().as_str() == "refs" {
            cx.advance(); // consume "refs"
            cx.expect(TokenKind::Colon)?;
            cx.expect(TokenKind::LSquare)?;
            cx.skip_empty_lines();
            while !cx.is_kind(TokenKind::RSquare) {
                let fname = cx.cur_text();
                cx.advance();
                ref_fields.push(fname.into());
                if cx.is_kind(TokenKind::Comma) {
                    cx.advance();
                    cx.skip_empty_lines();
                }
            }
            cx.expect(TokenKind::RSquare)?;
        }

        // Plan 408: `from "..."` is optional for `component` imports.
        // Without it, `component: X` references a same-project component
        // synthesized from a `component fn` declaration (import resolves to
        // `@/components/{X}.vue`, no file copy). fn/composable still require
        // an explicit `from` since they only make sense for external sources.
        let path = if cx.cur_text().as_str() == "from" {
            cx.advance();
            if !cx.is_kind(TokenKind::Str) {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Expected string import path after 'from' in widget use block, got '{}'",
                        cx.cur_text()
                    ),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
            let p = cx.cur_text();
            cx.advance();
            p
        } else if kind == ExtImportKind::Component {
            // No `from` → project-internal component fn reference.
            "".to_string()
        } else {
            return Err(SyntaxError::Generic {
                    message: format!(
                        "Expected 'from' in widget use block ({} imports require a source path), got '{}'",
                        match kind {
                            ExtImportKind::Fn | ExtImportKind::ExplicitFn => "fn",
                            ExtImportKind::Composable => "composable",
                            ExtImportKind::Component => "component",
                            ExtImportKind::Package => "package",
                        },
                        cx.cur_text()
                    ),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
        };

        imports.push(ExtImport {
            kind,
            symbols,
            path: path.into(),
            call_args,
            ref_fields,
        });
        cx.skip_empty_lines();
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(imports)
}

/// Parse a widget-level watch block (Vue backend reactive watchers):
///
/// ```auto
/// widget SlashMenu {
///     watch {
///         .filtered_items -> { .selected_index = 0 }
///         .ratio, .viewport_h.immediate -> { ... }
///         .items.deep -> { ... }
///     }
/// }
/// ```
///
/// Each entry lists one or more comma-separated sources (`.field` — a
/// model field, prop, or computed), optional `.immediate` / `.deep`
/// modifiers after the source list, `->`, then a handler body parsed
/// with the same conventions as `on` handlers.
pub(crate) fn parse_watch_block_inner(cx: &mut dyn UiParseContext) -> AutoResult<Vec<WatchDecl>> {
    cx.expect_ident("watch")?;
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut watchers = Vec::new();
    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }

        // Sources: .field (, .field)*
        let mut sources = Vec::new();
        loop {
            if !cx.is_kind(TokenKind::Dot) {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Expected '.field' watch source in watch block, got '{}'",
                        cx.cur_text()
                    ),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
            cx.advance(); // consume the dot
            let name = cx.cur_text();
            cx.advance();
            sources.push(name.into());
            if cx.is_kind(TokenKind::Comma) {
                cx.advance();
            } else {
                break;
            }
        }

        // Optional modifiers: .immediate / .deep (after the source list)
        let mut immediate = false;
        let mut deep = false;
        while cx.is_kind(TokenKind::Dot) {
            // Peek at the identifier after the dot.
            let modifier = match cx.probe_next_token().ok() {
                Some(tok) => tok.text.to_string(),
                None => String::new(),
            };
            match modifier.as_str() {
                "immediate" => {
                    cx.advance(); // dot
                    cx.advance(); // immediate
                    immediate = true;
                }
                "deep" => {
                    cx.advance(); // dot
                    cx.advance(); // deep
                    deep = true;
                }
                other => {
                    return Err(SyntaxError::Generic {
                        message: format!(
                            "Unknown watch modifier '.{}' (expected .immediate or .deep)",
                            other
                        ),
                        span: pos_to_span(cx.cur_pos()),
                    }
                    .into());
                }
            }
        }

        // Expect -> (might be Arrow, or Asn followed by Gt)
        if cx.is_kind(TokenKind::Arrow) {
            cx.advance();
        } else if cx.is_kind(TokenKind::Asn) {
            cx.advance();
            cx.expect(TokenKind::Gt)?;
        } else if cx.is_kind(TokenKind::Gt) {
            cx.advance();
        } else {
            return Err(SyntaxError::Generic {
                message: format!("Expected '->' after watch sources, got '{}'", cx.cur_text()),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }

        // Parse body with on-handler conventions (`.field` = state access)
        // PLAN-778 R1（F778-R1-02）：Err 平衡恢复 + 保存前值（嵌套 fn/on 体）。
        let prev_on_body = cx.set_in_on_body(true);
        let body_res = cx.parse_body();
        cx.set_in_on_body(prev_on_body);
        let body = body_res?;

        watchers.push(WatchDecl {
            sources,
            immediate,
            deep,
            body,
        });
        cx.skip_empty_lines();
    }

    cx.expect(TokenKind::RBrace)?;
    Ok(watchers)
}

/// Parse a widget-level expose block (Vue backend `defineExpose`):
///
/// ```auto
/// widget GraphView {
///     expose {
///         .fit, .graphEl
///         .relayout
///     }
/// }
/// ```
///
/// Each entry is a dot-prefixed member name (comma-separated and/or one
/// per line): a model field, computed property, template ref
/// (`ref: "el"` in the view), a `use { fn: ... }` import, or an `on`
/// handler (`.Fit` exposes the generated `Fit` function). The Vue
/// backend emits `defineExpose({ ... })` in `<script setup>` so a parent
/// holding a template ref on this component can call the exposed
/// members; other backends ignore the block.
/// Plan 426: parse a `setup { ... }` preamble block (consumes `setup`).
/// Statement set = handler-statement subset (let / expression statements);
/// `await` is REJECTED in the MVP (async setup needs a Suspense boundary —
/// separate task). A `refs: [f1, f2]` suffix after a `let` statement
/// annotates ref fields (script-side access injects `.value`).
pub(crate) fn parse_setup_block_inner(
    cx: &mut dyn UiParseContext,
) -> AutoResult<ast::ui::SetupBlock> {
    cx.expect_ident("setup")?;
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();
    let mut body = Body::new();
    let mut ref_annotations: Vec<(String, Vec<String>)> = Vec::new();
    while !cx.is_kind(TokenKind::RBrace) && !cx.is_kind(TokenKind::EOF) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }
        // Plan 426 决策(块级 refs 声明):`refs <binding>: [f1, f2]` —
        // 标注该 let 绑定的 ref 字段(script 侧访问注入 .value)。独立
        // 语句形态,不与 let 表达式语法纠缠。
        if cx.cur_text().as_str() == "refs" {
            cx.advance(); // consume "refs"
            let binding = cx.cur_text().to_string();
            cx.advance();
            cx.expect(TokenKind::Colon)?;
            cx.expect(TokenKind::LSquare)?;
            let mut fields = Vec::new();
            while !cx.is_kind(TokenKind::RSquare) {
                fields.push(cx.cur_text().to_string());
                cx.advance();
                if cx.is_kind(TokenKind::Comma) {
                    cx.advance();
                    cx.skip_empty_lines();
                }
            }
            cx.expect(TokenKind::RSquare)?;
            ref_annotations.push((binding, fields));
            cx.skip_empty_lines();
            continue;
        }
        let stmt = cx.parse_stmt()?;
        // MVP: reject await — async setup requires a Suspense boundary,
        // registered as future work. Covers expression statements and
        // `let x = <awaiting expr>` initializers alike.
        let contains_await = match &stmt {
            Stmt::Expr(expr) => stmt_expr_contains_await(expr),
            Stmt::Store(store) => stmt_expr_contains_await(&store.expr),
            _ => false,
        };
        if contains_await {
            return Err(SyntaxError::Generic {
                    message: "`await` is not allowed in a setup block (async setup requires a Suspense boundary - not yet supported)".into(),
                    span: pos_to_span(cx.cur_pos()),
                }.into());
        }
        body.stmts.push(stmt);
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(ast::ui::SetupBlock {
        body,
        ref_annotations,
    })
}

pub(crate) fn parse_expose_block_inner(cx: &mut dyn UiParseContext) -> AutoResult<Vec<Name>> {
    cx.expect_ident("expose")?;
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut names = Vec::new();
    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }

        if !cx.is_kind(TokenKind::Dot) {
            return Err(SyntaxError::Generic {
                message: format!(
                    "Expected '.member' name in expose block, got '{}'",
                    cx.cur_text()
                ),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }
        cx.advance(); // consume the dot
        let name = cx.cur_text();
        cx.advance();
        names.push(name.into());

        if cx.is_kind(TokenKind::Comma) {
            cx.advance();
        }
        cx.skip_empty_lines();
    }

    cx.expect(TokenKind::RBrace)?;
    Ok(names)
}

/// Parse a widget-level native CSS block: `style { ...raw CSS... }`.
///
/// The CSS content is captured verbatim by the lexer
/// ([`Lexer::capture_raw_block`]) — never tokenized — so nested `{}`,
/// `/* */` comments, media queries, and pseudo-classes pass through
/// unchanged. Returns the raw text between the braces.
pub(crate) fn parse_style_block_inner(cx: &mut dyn UiParseContext) -> AutoResult<String> {
    cx.expect_ident("style")?;
    cx.skip_empty_lines();

    if !cx.is_kind(TokenKind::LBrace) {
        return Err(SyntaxError::Generic {
            message: format!("Expected '{{' after 'style', got '{}'", cx.cur_text()),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }

    // `cur` is the LBrace token and the lexer sits immediately after it,
    // with no buffered tokens in between — safe to capture raw source.
    let css = cx.capture_raw_block()?;

    // Advance `cur` past the LBrace to the first token after the
    // captured block (the closing '}' was consumed by the capture).
    cx.advance();
    Ok(css)
}

/// Parse widget props: (name: type, name: type = default, ...)
pub(crate) fn parse_widget_props(cx: &mut dyn UiParseContext) -> AutoResult<Vec<PropDecl>> {
    cx.expect(TokenKind::LParen)?;
    let mut props = Vec::new();

    while !cx.is_kind(TokenKind::RParen) {
        let name = cx.cur_text();
        cx.advance();
        cx.expect(TokenKind::Colon)?;
        let ty = cx.parse_type()?;
        let default = if cx.is_kind(TokenKind::Asn) {
            cx.advance();
            Some(cx.parse_expr()?)
        } else {
            None
        };
        props.push(PropDecl {
            name: name.into(),
            ty,
            default,
        });

        if cx.is_kind(TokenKind::Comma) {
            cx.advance();
        }
    }
    cx.expect(TokenKind::RParen)?;
    Ok(props)
}

/// Parse computed block, returning the ComputedBlock directly
pub(crate) fn parse_computed_block_inner(cx: &mut dyn UiParseContext) -> AutoResult<ComputedBlock> {
    cx.expect_ident("computed")?;
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut properties = Vec::new();

    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }

        // Parse: name => expression
        let name = cx.cur_text();
        cx.advance();

        // Expect => (arrow)
        if cx.cur_text().as_str() == "=>" {
            cx.advance();
        } else {
            return Err(SyntaxError::Generic {
                message: format!(
                    "Expected '=>' after computed property name, got '{}'",
                    cx.cur_text()
                ),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }

        // Parse expression, or a multi-statement block body.
        // Plan 043 M5 #2: `x => { ...; return y }` now parses the brace as
        // a statement block (Expr::Block), reusing the same cx.parse_body()
        // path as on-handlers. A single-expression computed (`x => .foo`)
        // still goes through parse_expr(). Disable symbol checking: widget
        // /store computed expressions reference props and model fields
        // (e.g. `url => language`) that are not bound in the parser's
        // variable scope — name resolution happens in the code generator.
        // Without this, bare identifiers (and call args like `btoa(lang)`)
        // fail with "undefined variable" which surfaces as a confusing
        // "Expected term, got RBrace".
        //
        // Plan 028 F1: `x => { "k": v }` (dict/object literal) must not
        // fall into the block path — every pair is `key: value`, which no
        // statement sequence can start with. Try cx.parse_object_pairs()
        // speculatively first and roll back fully on failure so
        // multi-statement bodies keep the Plan 043 behavior.
        let old_skip_check = cx.set_skip_check(true);
        let expr = if cx.is_kind(TokenKind::LBrace) {
            let cp = cx.raw_cursor_save();
            let diag_mark = cx.diagnostics_mark();
            let obj = cx.parse_object_pairs();
            match obj {
                Ok(pairs) => {
                    // F810-R1-01：投机成功即释放快照（旧臂局部
                    // save_state 出作用域即释放的端口等价）。
                    cx.raw_cursor_release(cp);
                    Expr::Object(pairs)
                }
                Err(_) => {
                    cx.raw_cursor_restore(cp)?;
                    cx.diagnostics_truncate(diag_mark);
                    Expr::Block(cx.parse_body()?)
                }
            }
        } else {
            cx.parse_expr()?
        };
        cx.set_skip_check(old_skip_check);

        properties.push(ComputedProperty {
            name: name.into(),
            expr,
        });
        cx.skip_empty_lines();
    }
    cx.expect(TokenKind::RBrace)?;

    Ok(ComputedBlock { properties })
}

pub(crate) fn parse_on_block(cx: &mut dyn UiParseContext) -> AutoResult<OnBlock> {
    cx.expect_ident("on")?;
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut handlers = Vec::new();

    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }

        // Parse pattern (e.g., .Inc or Msg::Inc)
        // Plan 130: Support dot-prefixed patterns like .Inc
        // Plan 188 B4: Support parameterized patterns like .AddItem(text)
        //
        // Plan 028 F9: `stream sse(url[, "event"]) -> { … }` — a platform
        // stream subscription. The handler body receives the PRE-PARSED
        // event object as `ev` (platform layer parses JSON, 已决③).
        // One-token lookahead: bare `stream -> …` stays a msg pattern.
        let mut stream_sub: Option<ast::ui::StreamSubscription> = None;
        let is_stream_sub =
            if cx.cur_kind() == TokenKind::Ident && cx.cur_text().as_str() == "stream" {
                // PLAN-810: advance_peek_kind 具名化（旧臂回推+手动恢复形态）。
                cx.advance_peek_kind() == TokenKind::Ident
            } else {
                false
            };
        let (pattern, params) = if is_stream_sub {
            cx.advance(); // consume 'stream'
            let kind = cx.cur_text().to_string();
            cx.advance();
            if kind != "sse" {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Unsupported stream kind '{}' in on block (expected 'sse')",
                        kind
                    ),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
            cx.expect(TokenKind::LParen)?;
            let url = cx.cur_text().to_string();
            cx.expect(TokenKind::Str)?;
            let mut event: Option<String> = None;
            if cx.is_kind(TokenKind::Comma) {
                cx.advance();
                event = Some(cx.cur_text().to_string());
                cx.expect(TokenKind::Str)?;
            }
            cx.expect(TokenKind::RParen)?;
            // Synthetic pattern — unique per url/event, never collides
            // with a msg variant (starts with "__").
            let pattern = format!(
                "__stream_sse_{}",
                url.replace(|c: char| !c.is_ascii_alphanumeric(), "_")
            );
            stream_sub = Some(ast::ui::StreamSubscription { kind, url, event });
            (pattern, vec!["ev".to_string()])
        } else if cx.is_kind(TokenKind::Dot) {
            cx.advance(); // consume the dot
            let name = cx.cur_text().to_string();
            cx.advance();
            // Check for params: .Name(param1, param2) or .Name(name1 type1, name2 type2)
            let params = parse_handler_params(cx)?;
            (format!(".{}", name), params)
        } else {
            // Plan 435 挂账②续:无点前缀带参形态 `Change(pressed) ->`
            // (gallery 官方组件在用)——参数收集与点前缀分支一致;
            // pattern 保持无点(兼容既有无点语法)。
            let name = cx.cur_text().to_string();
            cx.advance();
            let params = parse_handler_params(cx)?;
            (name, params)
        };

        // Expect -> (might be Arrow, or Asn followed by Gt)
        if cx.is_kind(TokenKind::Arrow) {
            cx.advance();
        } else if cx.is_kind(TokenKind::Asn) {
            cx.advance();
            cx.expect(TokenKind::Gt)?;
        } else if cx.is_kind(TokenKind::Gt) {
            // Allow just > for simplicity
            cx.advance();
        }

        // Define params in scope before parsing body
        cx.enter_scope();
        for param in &params {
            cx.define_handler_param(param.as_str());
        }
        // Parse body
        // PLAN-778 R1（F778-R1-02）：Err 平衡恢复 + 保存前值（嵌套 fn/on 体）。
        let prev_on_body = cx.set_in_on_body(true);
        let body_res = cx.parse_body();
        cx.set_in_on_body(prev_on_body);
        let body = body_res?;
        cx.exit_scope();

        handlers.push(OnHandler {
            pattern,
            params,
            body,
            stream: stream_sub,
        });
        cx.skip_empty_lines();
    }

    cx.expect(TokenKind::RBrace)?;
    Ok(OnBlock { handlers })
}

/// Parse a bind block: bind { "key" -> .Handler, ... }
/// Plan 275: Key bindings for AURA widgets
pub(crate) fn parse_bind_block(cx: &mut dyn UiParseContext) -> AutoResult<BindBlock> {
    cx.expect_ident("bind")?;
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut bindings = Vec::new();

    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }

        // Parse key string: "1", "+", "Enter", etc.
        let key = if cx.is_kind(TokenKind::Str) {
            let s = cx.cur_text().to_string();
            cx.advance();
            s
        } else {
            return Err(SyntaxError::Generic {
                message: format!("Expected string key in bind block, got '{}'", cx.cur_text()),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        };

        // Expect ->
        if cx.is_kind(TokenKind::Arrow) {
            cx.advance();
        } else if cx.is_kind(TokenKind::Asn) {
            cx.advance();
            cx.expect(TokenKind::Gt)?;
        } else {
            return Err(SyntaxError::Generic {
                message: format!("Expected '->' in bind block, got '{}'", cx.cur_text()),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }

        // Parse handler: .Name
        let handler = if cx.is_kind(TokenKind::Dot) {
            cx.advance(); // consume the dot
            let name = cx.cur_text().to_string();
            cx.advance();
            format!(".{}", name)
        } else {
            return Err(SyntaxError::Generic {
                message: format!(
                    "Expected '.HandlerName' in bind block, got '{}'",
                    cx.cur_text()
                ),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        };

        bindings.push(KeyBinding { key, handler });
        cx.skip_empty_lines();
    }

    cx.expect(TokenKind::RBrace)?;
    Ok(BindBlock { bindings })
}
