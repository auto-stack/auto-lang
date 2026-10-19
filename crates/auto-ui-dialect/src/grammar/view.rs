//! view/fragment/component/节点/for/if/key/memo/link/style 绑定与
//! UI 事件文法（PLAN-810 T-04 迁移）。

use super::helpers::{
    extract_fstr_template_and_bindings, fragment_param_hint_to_type, get_primary_prop,
};
use super::widget::{
    mint_inline_event_handlers, parse_computed_block_inner, parse_on_block,
    parse_setup_block_inner, parse_style_block_inner, parse_watch_block_inner,
    parse_widget_use_block_inner,
};
use crate::UiParseContext;
use auto_parser_api::ast;
use auto_parser_api::ast::*;
use auto_parser_api::error::{pos_to_span, AutoResult, SyntaxError};
use auto_parser_api::token::TokenKind;

/// Parse view block (UI scenario only)
pub fn parse_view_block(cx: &mut dyn UiParseContext) -> AutoResult<Stmt> {
    let view = parse_view_block_inner(cx)?;
    Ok(Stmt::ViewBlock(view))
}

/// Plan 367 P2-3: Parse either a view block or a view fragment declaration.
pub fn parse_view_block_or_fragment(cx: &mut dyn UiParseContext) -> AutoResult<Stmt> {
    cx.expect_ident("view")?;
    cx.skip_empty_lines();
    if cx.cur_text().as_str() == "fn" {
        parse_view_fragment_decl_body(cx)
    } else {
        cx.expect(TokenKind::LBrace)?;
        let root = parse_view_root_nodes(cx)?;
        Ok(Stmt::ViewBlock(ViewBlock { root }))
    }
}

/// Parse view fragment declaration body (after 'view' has been consumed).
/// Plan 425: `view fn` only — `component fn` parses to a WidgetDecl via
/// parse_component_fn_decl.
pub(crate) fn parse_view_fragment_decl_body(cx: &mut dyn UiParseContext) -> AutoResult<Stmt> {
    cx.expect_ident("fn")?;
    parse_fragment_decl_body_tail(cx)
}

/// Plan 408: Parse `component fn Name(...) { ... }` — an independent
/// component declaration synthesized to its own Vue SFC.
///
/// Plan 425: `component fn` is SYNTACTIC SUGAR over `widget` — this
/// parser directly produces a `WidgetDecl` identical to
/// `widget Name(...) { blocks... view { body } }` (body elements wrapped
/// into a view block, params → props via the fragment hint mapping).
/// Block order (params → use → computed → msg → model → watch → on →
/// style → body) is unchanged from the Plan 408 grammar; the fragment
/// double track (ViewFragmentDecl + extract_widget_from_fragment) is
/// retired — `view fn` (inline) keeps the fragment path.
pub fn parse_component_fn_decl(cx: &mut dyn UiParseContext) -> AutoResult<Stmt> {
    cx.expect_ident("component")?;
    cx.expect_ident("fn")?;
    let name = cx.cur_text();
    cx.advance();
    // Params: `Name[: type][, ...]` — colon optional (Plan 408 grammar),
    // type captured as a raw token string and mapped via the fragment
    // hint mapping so sugared output stays byte-identical.
    let mut params: Vec<(ast::Name, String)> = Vec::new();
    if cx.cur_text().as_str() == "(" {
        cx.advance();
        while cx.cur_text().as_str() != ")" {
            cx.skip_empty_lines();
            let pname = cx.cur_text();
            cx.advance();
            let mut type_hint = String::new();
            if cx.cur_text().as_str() == ":" {
                cx.advance();
                let mut ty_parts = Vec::new();
                while !matches!(cx.cur_text().as_str(), "," | ")") {
                    ty_parts.push(cx.cur_text().as_str().to_string());
                    cx.advance();
                }
                type_hint = ty_parts.join(" ");
            }
            params.push((pname.into(), type_hint));
            if cx.cur_text().as_str() == "," {
                cx.advance();
            }
        }
        cx.advance();
    }
    cx.skip_empty_lines();
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();
    // Plan 408 P5: optional `use { }` blocks (fn/composable/component
    // escape-hatch imports) — imports before use.
    let mut ext_imports: Vec<ExtImport> = Vec::new();
    while cx.cur_text().as_str() == "use" {
        ext_imports.extend(parse_widget_use_block_inner(cx)?);
        cx.skip_empty_lines();
    }
    // Plan 408 P3: optional `computed { }` (after use, before view body).
    let computed = if cx.cur_text().as_str() == "computed" {
        Some(parse_computed_block_inner(cx)?)
    } else {
        None
    };
    cx.skip_empty_lines();
    // Plan 408 P4: optional `msg { }` (multiple) / `model { }` blocks.
    let mut messages: Vec<MsgDecl> = Vec::new();
    while cx.cur_text().as_str() == "msg" {
        messages.push(crate::declarations::parse_msg_decl(cx)?);
        cx.skip_empty_lines();
    }
    let model = if cx.cur_text().as_str() == "model" {
        Some(crate::declarations::parse_model_block(cx)?)
    } else {
        None
    };
    cx.skip_empty_lines();
    // Plan 367 P2-3 fix: register params in a fresh scope so the body and
    // handlers can reference them (see parse_fragment_decl_body_tail).
    cx.enter_scope();
    for (pname, _) in &params {
        cx.bind_var(pname.as_str(), Type::Unknown);
    }
    // Plan 408 P4: model fields join the scope so on-handlers / view body
    // can reference them (`.collapsed = !.collapsed`).
    if let Some(ref model_block) = model {
        for field in &model_block.fields {
            cx.bind_var(field.name.as_str(), Type::Unknown);
        }
    }
    // Plan 408 P12 §10.2: optional `watch { }` (after model, before on).
    let watch: Vec<WatchDecl> = if cx.cur_text().as_str() == "watch" {
        parse_watch_block_inner(cx)?
    } else {
        Vec::new()
    };
    cx.skip_empty_lines();
    // Plan 408 P4: optional `on { }` (after model/watch, before body —
    // handler bodies can reference params + model fields).
    let on = if cx.cur_text().as_str() == "on" {
        Some(parse_on_block(cx)?)
    } else {
        None
    };
    cx.skip_empty_lines();
    // PLAN-026 缺陷②: optional `style { }` (mirrors widget).
    let style: Option<String> = if cx.cur_text().as_str() == "style" {
        Some(parse_style_block_inner(cx)?)
    } else {
        None
    };
    cx.skip_empty_lines();
    // Plan 426: optional `setup { }` preamble (per-instance, before first
    // render) — mirrors widget.
    let setup: Option<ast::ui::SetupBlock> = if cx.cur_text().as_str() == "setup" {
        Some(parse_setup_block_inner(cx)?)
    } else {
        None
    };
    cx.skip_empty_lines();
    // The single body node is the view root — sugar wraps it in a view
    // block (body 即视图,与 view 可选化语义一致).
    let body = parse_view_node(cx)?;
    cx.exit_scope();
    cx.skip_empty_lines();
    cx.expect(TokenKind::RBrace)?;
    // Plan 448 B1: component fn is widget sugar — mint its inline lambda
    // handlers exactly like parse_widget_decl does.
    let mut decl = WidgetDecl {
        name: name.into(),
        messages,
        model,
        computed,
        view: Some(ViewBlock { root: body }),
        named_views: Vec::new(),
        on,
        bind: None,
        props: params
            .into_iter()
            .map(|(pname, hint)| PropDecl {
                name: pname,
                ty: fragment_param_hint_to_type(&hint),
                default: None,
            })
            .collect(),
        routes: None,
        lifecycle: Vec::new(),
        style,
        ext_imports,
        watch,
        setup,
        expose: Vec::new(),
        actions: None,
        timer: None,
    };
    mint_inline_event_handlers(&mut decl)?;
    Ok(Stmt::WidgetDecl(decl))
}

/// Parse the param list + body of a `view fn` declaration (inline
/// fragment, Plan 367 P2-3), after `fn` has been consumed.
/// Plan 425: component fn left this shared path — only the inline view-fn
/// form remains (params + single body node, no blocks).
pub(crate) fn parse_fragment_decl_body_tail(cx: &mut dyn UiParseContext) -> AutoResult<Stmt> {
    let name = cx.cur_text();
    cx.advance();
    let mut params: Vec<(ast::Name, String)> = Vec::new();
    if cx.cur_text().as_str() == "(" {
        cx.advance();
        while cx.cur_text().as_str() != ")" {
            cx.skip_empty_lines();
            let pname = cx.cur_text();
            cx.advance();
            let mut type_hint = String::new();
            if cx.cur_text().as_str() == ":" {
                cx.advance();
                let mut ty_parts = Vec::new();
                while !matches!(cx.cur_text().as_str(), "," | ")") {
                    ty_parts.push(cx.cur_text().as_str().to_string());
                    cx.advance();
                }
                type_hint = ty_parts.join(" ");
            }
            params.push((pname.into(), type_hint));
            if cx.cur_text().as_str() == "," {
                cx.advance();
            }
        }
        cx.advance();
    }
    cx.skip_empty_lines();
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();
    // Plan 367 P2-3 fix: register params in a fresh scope so the body can
    // reference them. Without this, `check_symbol` flags a parameter used
    // as a bare `if` condition (e.g. `style: if active {..}`) as an
    // undefined variable, which (after error recovery) surfaces as
    // "Expected term, got RBrace" and makes the whole module fail to parse.
    cx.enter_scope();
    for (pname, _) in &params {
        cx.bind_var(pname.as_str(), Type::Unknown);
    }
    let body = parse_view_node(cx)?;
    cx.exit_scope();
    cx.skip_empty_lines();
    cx.expect(TokenKind::RBrace)?;
    Ok(Stmt::ViewFragmentDecl(ViewFragmentDecl {
        name: name.into(),
        params,
        body,
    }))
}

/// Parse view block, returning the ViewBlock directly
pub(crate) fn parse_view_block_inner(cx: &mut dyn UiParseContext) -> AutoResult<ViewBlock> {
    cx.expect_ident("view")?;
    cx.expect(TokenKind::LBrace)?;
    let root = parse_view_root_nodes(cx)?;
    Ok(ViewBlock { root })
}

/// Parse the nodes inside `view { ... }` up to the closing brace.
///
/// Plan 015 (auto-down 批次, docs/plans/DEBTS.md 015 🟡): the view block used to
/// parse a SINGLE root node and then expect `}` — two bare top-level
/// siblings (`div { onclick: .A(1) } div { onclick: .B(2) }`) cascaded
/// into "Expected term, got RBrace" while the same shape inside a
/// `col { }` container worked. Now all top-level nodes are parsed; a
/// single node stays the root unchanged, and multiple siblings are
/// wrapped in an implicit `col` (the documented workaround container).
/// The opening `{` has been consumed; this consumes the closing `}`.
pub(crate) fn parse_view_root_nodes(cx: &mut dyn UiParseContext) -> AutoResult<ViewNode> {
    cx.skip_empty_lines();

    let mut roots = vec![parse_view_node(cx)?];
    loop {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }
        roots.push(parse_view_node(cx)?);
    }
    cx.expect(TokenKind::RBrace)?;

    if roots.len() == 1 {
        Ok(roots.pop().unwrap())
    } else {
        Ok(ViewNode::Element {
            tag: "col".to_string(),
            props: Vec::new(),
            events: Vec::new(),
            children: roots,
            span: None,
        })
    }
}

/// Parse a single view node
pub(crate) fn parse_view_node(cx: &mut dyn UiParseContext) -> AutoResult<ViewNode> {
    let start_pos = cx.cur_pos();
    cx.skip_empty_lines();

    // Check for "for" keyword: for item in .list { body }
    if cx.cur_text().as_str() == "for" {
        return parse_view_for_loop(cx);
    }

    // Check for "if" keyword: if condition { then_body } else { else_body }
    if cx.cur_text().as_str() == "if" {
        return parse_view_conditional(cx);
    }

    // PLAN-046 T-04: explicit memo block — `memo (deps: .a, .b) { … }`.
    // Intercepted only in the `memo (` paren form (single-token peek,
    // same pattern as the dyn/paren probes below); a bare `memo` token
    // without parens stays on the generic element path.
    if cx.cur_text().as_str() == "memo" {
        let is_paren = cx.probe_next_token()?.kind == TokenKind::LParen;
        if is_paren {
            return parse_view_memo_block(cx);
        }
    }

    // Check for "outlet" keyword: router outlet (Plan 105). PLAN-046
    // T-05: optional `outlet (memo: true)` corpus prop — the only
    // accepted header form (anything else in the parens is a pointed
    // parse error; bare `outlet` stays byte-compatible).
    if cx.is_kind(TokenKind::Outlet) {
        cx.advance();
        // PLAN-708 T-01（S-01）三态头参：裸 `outlet`（None）/
        // `outlet (memo: true)`（Some(true)）/ `outlet (memo: false)`
        //（Some(false)）。其余头参仍为 pointed parse error。
        let mut memo: Option<bool> = None;
        if cx.is_kind(TokenKind::LParen) {
            cx.advance();
            cx.skip_empty_lines();
            let key = cx.cur_text().to_string();
            cx.advance();
            cx.expect(TokenKind::Colon)?;
            let v = cx.cur_text().to_string();
            cx.advance();
            let parsed = match (key.as_str(), v.as_str()) {
                ("memo", "true") => Some(true),
                ("memo", "false") => Some(false),
                _ => None,
            };
            if parsed.is_none() {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "outlet 头参只接受 `memo: true|false`，得到 `{}: {}`",
                        key, v
                    ),
                    span: pos_to_span(cx.prev_pos()),
                }
                .into());
            }
            cx.expect(TokenKind::RParen)?;
            memo = parsed;
        }
        return Ok(ViewNode::Outlet { memo });
    }

    // Check for "link" keyword: navigation link (Plan 105)
    // link (to: "/path") { children }
    if cx.is_kind(TokenKind::Link) {
        return parse_view_link(cx);
    }

    // Check for string literal as child node (e.g., Button { "Subscribe" })
    // This should be treated as a text node, not a component call
    if cx.is_kind(TokenKind::Str) {
        let content = cx.cur_text();
        cx.advance();
        return Ok(ViewNode::text(content));
    }

    // Check for f-string as child node (e.g., Text { f"Count: ${.count}" })
    if cx.is_kind(TokenKind::FStrStart) {
        let fstr_expr = cx.parse_fstr()?;
        let (template, bindings) = extract_fstr_template_and_bindings(&fstr_expr);

        // If there are bindings, it's interpolated; otherwise literal
        if bindings.is_empty() {
            return Ok(ViewNode::text(template));
        } else {
            return Ok(ViewNode::Text(ViewText::Interpolated {
                template,
                bindings,
            }));
        }
    }

    // Plan 367 P2-3: view fragment calls are detected at extraction time,
    // not parse time. PascalCase tags parse as normal elements, and the
    // aura extractor checks if the tag name matches a registered fragment.
    // This avoids interfering with normal component parsing.

    // Parse element tag
    let tag = cx.cur_text().to_string();
    cx.advance();

    let mut props = Vec::new();
    let mut events = Vec::new();
    let mut children = Vec::new();

    // Check for special text suffix: button +, button -
    if tag == "+" || tag == "-" {
        return Ok(ViewNode::text(tag));
    }

    // Check for string literal or f-string as primary property shorthand:
    // tag "value" → tag (primary_prop: "value")
    // tag f"count: ${.n}" → tag (primary_prop: f"count: ${.n})
    // The primary prop depends on the element type (from get_primary_prop)
    // Also handle .field as primary prop: Text .title → Text (text: .title)
    // Also handle ident.field as primary prop: Text item.order → Text (text: item.order)
    let has_primary_prop_value = cx.is_kind(TokenKind::Str) || cx.is_kind(TokenKind::FStrStart);
    let has_dot_primary = cx.is_kind(TokenKind::Dot);
    // Check if identifier is followed by dot (like item.order) or paren (fn call).
    // Plan 012 P2: 'link'/'task' are contextual keywords (TokenKind::Link/
    // Task) but legitimate loop-var/local names — treat them as
    // identifiers here too, so `text link.title` parses as a field
    // access instead of a router-link node (jade gaps 18/29).
    // Plan 407: also accept ident followed by LParen so that `text t('nav.chat')`
    // parses the whole fn-call expr as the primary prop value (Expr::Call),
    // letting text nodes render i18n `{{ t('nav.chat') }}` without escape hatches.
    // Plan 492 M2 (族 A1): also accept ident followed by LBracket so that
    // `text t["label"]` parses the whole Index chain as the primary prop
    // value (Expr::Index) — previously the `["label"]` suffix split off
    // into a stray dump/child node.
    let has_ident_field_primary = (cx.is_kind(TokenKind::Ident)
        || cx.is_kind(TokenKind::Link)
        || cx.is_kind(TokenKind::Task))
        && {
            // Peek ahead: ident 后跟 Dot（字段访问）/ LParen（函数调用）/
            // LBracket（索引访问）
            if let Ok(next_token) = cx.probe_next_token() {
                let is_field_or_call = next_token.kind == TokenKind::Dot
                    || next_token.kind == TokenKind::LParen
                    || next_token.kind == TokenKind::LSquare;
                is_field_or_call
            } else {
                false
            }
        };

    if has_primary_prop_value {
        if let Some(primary_prop) = get_primary_prop(&tag) {
            let is_fstr = cx.is_kind(TokenKind::FStrStart);

            // Plan 444 (ash-shell-057 ①c): `text "#" + j.id` — a Str
            // followed by `+` opens a binary concat chain, not a
            // standalone literal. Taking only the Str here dropped the
            // `+ j.id` remainder onto sibling view nodes (a stray "+"
            // text node plus a receiver-less `{{ id }}` interpolation).
            let opens_concat_chain = !is_fstr && {
                if let Ok(next_token) = cx.probe_next_token() {
                    let is_concat = next_token.kind == TokenKind::Add;
                    is_concat
                } else {
                    false
                }
            };

            // For f-strings, we need to parse the actual expression
            let value = if is_fstr {
                // Parse the f-string to get proper Expr::FStr with bindings
                cx.parse_fstr()?
            } else if opens_concat_chain {
                // parse_expr folds the whole chain (`"#" + j.id` →
                // Bina(Str, +, Dot)) and stops at the `{` props block.
                cx.parse_expr()?
            } else {
                let content = cx.cur_text();
                cx.advance();
                Expr::Str(content.into())
            };

            props.push(ViewProp {
                name: primary_prop.to_string(),
                value: ViewPropValue::Expr(value),
            });
        } else {
            // No primary prop defined for this element, skip the string
            cx.advance();
        }
    } else if has_dot_primary {
        // Handle .field as primary prop (e.g., Text .title)
        if let Some(primary_prop) = get_primary_prop(&tag) {
            let dot_expr = cx.parse_dot_item()?;
            props.push(ViewProp {
                name: primary_prop.to_string(),
                value: ViewPropValue::Expr(dot_expr),
            });
        }
    } else if has_ident_field_primary {
        // Handle ident.field as primary prop (e.g., Text item.order)
        if let Some(primary_prop) = get_primary_prop(&tag) {
            let expr = cx.parse_expr()?;
            props.push(ViewProp {
                name: primary_prop.to_string(),
                value: ViewPropValue::Expr(expr),
            });
        }
    } else if cx.is_kind(TokenKind::Ident)
        || cx.is_kind(TokenKind::Tag)
        || cx.is_kind(TokenKind::Link)
        || cx.is_kind(TokenKind::Task)
        || cx.is_kind(TokenKind::Type)
    {
        // Plan 354: bare identifier as primary prop (e.g., `badge t` where
        // t is a loop variable). Only when tag has a known primary prop.
        // Also accept contextual soft keywords (tag/link/task/type) so that
        // `text tag` where `tag` is a loop variable lexes correctly —
        // `tag` is TokenKind::Tag (a soft keyword), not TokenKind::Ident.
        if let Some(primary_prop) = get_primary_prop(&tag) {
            let id = cx.cur_text();
            cx.advance();
            props.push(ViewProp {
                name: primary_prop.to_string(),
                value: ViewPropValue::Expr(Expr::Ident(id.into())),
            });
        }
    }

    // Dynamic component: dyn (expr) { props } — the parenthesized
    // expression is the component value (a loop-iterator field like
    // .item.icon, a model field, or a computed), emitted by the Vue
    // backend as <component :is="expr">. Additional props/class/events
    // go in the braces (or as an `is:` prop without parentheses).
    if tag == "dyn" && cx.is_kind(TokenKind::LParen) {
        cx.advance();
        cx.skip_empty_lines();
        let is_expr = cx.parse_expr()?;
        cx.skip_empty_lines();
        cx.expect(TokenKind::RParen)?;
        props.push(ViewProp {
            name: "is".to_string(),
            value: ViewPropValue::Expr(is_expr),
        });
    // Parse props/events in parentheses: tag (props) { children }
    } else if cx.is_kind(TokenKind::LParen) {
        // Plan 041a⑥-c(musk 041 Phase 5 T20):`(` 后首 token 为 Dot 时,
        // 这是括号表达式简写(`text (.row.idx + 1).to_string()`),不是
        // prop 列表——props key 永不以 `.` 开头,无歧义。按表达式解析并
        // 挂到 primary prop。
        let paren_is_expr = cx.probe_next_token()?.kind == TokenKind::Dot;
        if paren_is_expr {
            if let Some(primary_prop) = get_primary_prop(&tag) {
                cx.advance(); // consume (
                let value = cx.parse_expr()?;
                cx.expect(TokenKind::RParen)?;
                props.push(ViewProp {
                    name: primary_prop.to_string(),
                    value: ViewPropValue::Expr(value),
                });
                // 走 children/后续处理(跳过 props 循环)——fallthrough
                // 到下方共同路径;用与 props 分支相同的方式继续。
                cx.skip_empty_lines();
            } else {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "element `<{}>` has no primary prop to bind a parenthesized expression to",
                        tag
                    ),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
        } else {
            cx.advance();
            cx.skip_empty_lines();

            while !cx.is_kind(TokenKind::RParen) {
                cx.skip_empty_lines();
                if cx.is_kind(TokenKind::RParen) {
                    break;
                }

                let key = cx.cur_text().to_string();
                cx.advance();

                // Custom event with a quoted name (allows ':'/'-' in the event
                // name, which an identifier key cannot hold):
                //   on "autodown:slash-open".document: .OnOpen($event)
                // Internally the name is kept quoted: on"autodown:slash-open".
                let key = if key == "on" && cx.is_kind(TokenKind::Str) {
                    let custom = cx.cur_text().to_string();
                    cx.advance();
                    format!("on\"{}\"", custom)
                } else {
                    key
                };

                // Check if it's an event (onclick, etc.)
                if key.starts_with("on") {
                    let key = parse_event_modifiers(cx, key)?;
                    cx.expect(TokenKind::Colon)?;
                    // Parse handler with optional parameters: .Inc or .Delete(todo.id)
                    let (handler, params, inline) = parse_event_value(cx)?;
                    events.push(ViewEvent {
                        name: key,
                        handler,
                        params,
                        inline,
                    });
                } else {
                    cx.expect(TokenKind::Colon)?;

                    // Check for style binding: style: { completed: todo.done }
                    // style_obj: { top: expr } is the inline-style (:style) form.
                    if (key == "style" || key == "style_obj") && cx.is_kind(TokenKind::LBrace) {
                        let binding = parse_style_binding(cx)?;
                        props.push(ViewProp {
                            name: key,
                            value: ViewPropValue::StyleBinding(binding),
                        });
                    } else {
                        let value = cx.parse_expr()?;
                        props.push(ViewProp {
                            name: key,
                            value: ViewPropValue::Expr(value),
                        });
                    }
                }

                if cx.is_kind(TokenKind::Comma) {
                    cx.advance();
                }
            }
            cx.expect(TokenKind::RParen)?;
        }
    }

    // Check for string literal or f-string as primary property shorthand AFTER parentheses:
    // tag (props) "value" → tag has props AND primary_prop: "value"
    // e.g., Text (variant: "muted") "Hello" → Text with variant="muted" and text="Hello"
    // Also supports f-string: Text f"Count: ${.count}" → Text with text=f"Count: ${.count}"
    if cx.is_kind(TokenKind::Str) || cx.is_kind(TokenKind::FStrStart) {
        if let Some(primary_prop) = get_primary_prop(&tag) {
            // Only add if not already set
            if !props.iter().any(|p| p.name == primary_prop) {
                let is_fstr = cx.is_kind(TokenKind::FStrStart);

                let value = if is_fstr {
                    // Parse the f-string to get proper Expr::FStr with bindings
                    cx.parse_fstr()?
                } else {
                    let content = cx.cur_text();
                    cx.advance();
                    Expr::Str(content.into())
                };

                props.push(ViewProp {
                    name: primary_prop.to_string(),
                    value: ViewPropValue::Expr(value),
                });
            }
        }
    }

    // Plan 435 挂账②续:管道文本简写 `tag | bare text…`(行尾止)——
    // 此前 `text | Slide 1` 靠垃圾解析碰巧能过(|/Slide/1 成为垃圾子元素),
    // 多词带句点形态(`ComboboxEmpty | No framework found.`)直接报错。
    // 现按语义实现:读整行文本,挂 primary prop(有声明)或 text 子节点。
    if cx.is_kind(TokenKind::VBar) {
        cx.advance();
        let mut text = String::new();
        while !cx.is_kind(TokenKind::Newline)
            && !cx.is_kind(TokenKind::RBrace)
            && !cx.is_kind(TokenKind::EOF)
        {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(&cx.cur_text());
            cx.advance();
        }
        if let Some(primary) = get_primary_prop(&tag) {
            if !props.iter().any(|p| p.name == primary) {
                props.push(ViewProp {
                    name: primary.to_string(),
                    value: ViewPropValue::Expr(Expr::Str(text.into())),
                });
            }
        } else {
            children.push(ViewNode::text(text));
        }
    }

    // Parse children or inline props/events in braces
    // Support syntax: col { child1 child2 style: "..." }
    // Also supports primary prop shorthand: text "Hello" { style: "..." }
    // Plan 435 挂账②续:while 支持连写块 `div { style: ".." } { children }`
    // (gallery 官方组件在用;首个块收 props,后续块收 children/props)
    while cx.is_kind(TokenKind::LBrace) {
        cx.advance();
        cx.skip_empty_lines();

        // Parse children (view nodes) and trailing props/events
        while !cx.is_kind(TokenKind::RBrace) {
            cx.skip_empty_lines();
            if cx.is_kind(TokenKind::RBrace) {
                break;
            }

            // Check if this looks like a prop: identifier followed by colon
            // e.g., "class:" or "onclick:"
            // "is" is a keyword token but a legitimate prop key
            // (Vue's <component is="..."> / dyn { is: ... }).
            // Plan 012 P2: keyword tokens are legitimate prop keys too —
            // `type:`/`to:`/`as:` (and any other reserved word) followed
            // by a colon is unambiguous in a view block: child nodes never
            // take a colon suffix (jade gaps 27/53). Without this,
            // `button { type: "button" }` silently parsed `type` as a
            // child element, emitting garbage `<div>button</div>` nodes.
            let is_prop_like = {
                // Peek at next token to see if it's a colon
                if let Ok(next_token) = cx.probe_next_token() {
                    let is_colon = next_token.kind == TokenKind::Colon;
                    // Event modifiers: `onwheel.prevent:` — an `on*` key
                    // followed by a dot also introduces an event prop.
                    let is_event_modifier = !is_colon
                        && next_token.kind == TokenKind::Dot
                        && cx.cur_text().starts_with("on");
                    // Quoted custom event: `on "autodown:slash-open":` —
                    // `on` followed by a string literal.
                    let is_quoted_event =
                        cx.cur_text() == "on" && next_token.kind == TokenKind::Str;
                    is_colon || is_event_modifier || is_quoted_event
                } else {
                    false
                }
            };

            if is_prop_like {
                // Parse as prop/event
                let key = cx.cur_text().to_string();
                cx.advance();
                // Quoted custom event name: on "autodown:slash-open" —
                // keep the quotes in the internal key (on"name") so the
                // generators can tell it apart from identifier events.
                let key = if key == "on" && cx.is_kind(TokenKind::Str) {
                    let custom = cx.cur_text().to_string();
                    cx.advance();
                    format!("on\"{}\"", custom)
                } else {
                    key
                };
                // Event modifiers (onwheel.prevent, onmousemove.window) are
                // dot-suffixed and must be consumed before the colon.
                let key = if key.starts_with("on") {
                    parse_event_modifiers(cx, key)?
                } else {
                    key
                };
                cx.expect(TokenKind::Colon)?;

                // Check if it's an event (onclick, etc.)
                if key.starts_with("on") {
                    let (handler, params, inline) = parse_event_value(cx)?;
                    events.push(ViewEvent {
                        name: key,
                        handler,
                        params,
                        inline,
                    });
                } else if (key == "style" || key == "style_obj") && cx.is_kind(TokenKind::LBrace) {
                    let binding = parse_style_binding(cx)?;
                    props.push(ViewProp {
                        name: key,
                        value: ViewPropValue::StyleBinding(binding),
                    });
                } else {
                    let value = cx.parse_expr()?;
                    props.push(ViewProp {
                        name: key,
                        value: ViewPropValue::Expr(value),
                    });
                }

                // Support both comma and semicolon as separators
                if cx.is_kind(TokenKind::Comma) || cx.is_kind(TokenKind::Semi) {
                    cx.advance();
                }
            } else {
                // Parse as child view node
                let child = parse_view_node(cx)?;
                children.push(child);
            }
            cx.skip_empty_lines();
        }
        cx.expect(TokenKind::RBrace)?;
    }

    Ok(ViewNode::Element {
        tag,
        props,
        events,
        children,
        span: Some((start_pos.pos, cx.cur_pos().pos - start_pos.pos)),
    })
}

/// Parse style binding: { completed: todo.done, editing: todo.editing }
pub(crate) fn parse_style_binding(
    cx: &mut dyn UiParseContext,
) -> AutoResult<Vec<StyleBindingEntry>> {
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut entries = Vec::new();

    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }

        // Parse style name (can be identifier or string)
        let style_name = if cx.is_kind(TokenKind::Str) || cx.cur_text().starts_with('"') {
            // Quoted string
            let s = cx.cur_text().to_string();
            cx.advance();
            // Remove quotes
            s.trim_matches('"').to_string()
        } else {
            // Unquoted identifier
            let name = cx.cur_text().to_string();
            cx.advance();
            name
        };

        cx.expect(TokenKind::Colon)?;

        // Parse condition expression
        let condition = cx.parse_expr()?;

        entries.push(StyleBindingEntry {
            style_name,
            condition,
        });

        // Optional comma
        if cx.is_kind(TokenKind::Comma) {
            cx.advance();
        }
        cx.skip_empty_lines();
    }

    cx.expect(TokenKind::RBrace)?;
    Ok(entries)
}

/// Parse for loop in view: for item in .list { body }
pub(crate) fn parse_view_for_loop(cx: &mut dyn UiParseContext) -> AutoResult<ViewNode> {
    let start_pos = cx.cur_pos();
    cx.expect_ident("for")?;

    // Parse loop variable (and optional index)
    let first_var = cx.cur_text().to_string();
    cx.advance();

    // Check for index: for idx, item in ...
    // Auto convention: first variable is index, second is value
    // This differs from Vue's (item, index) order, so we swap in the generator
    let (index, var) = if cx.is_kind(TokenKind::Comma) {
        cx.advance(); // consume comma
        let second_var = cx.cur_text().to_string();
        cx.advance();
        (Some(first_var), second_var)
    } else {
        (None, first_var)
    };

    // Expect "in" keyword
    cx.expect_ident("in")?;

    // Parse iterable (e.g., .todos or .section.items or 0..10)
    let iterable = if cx.is_kind(TokenKind::Dot) {
        cx.advance();
        let first_name = cx.cur_text().to_string();
        cx.advance();
        let mut result = format!(".{}", first_name);

        // Handle chained property access like .section.items
        while cx.is_kind(TokenKind::Dot) {
            cx.advance(); // skip dot
            let name = cx.cur_text().to_string();
            cx.advance();
            result.push_str(&format!(".{}", name));
        }
        result
    } else if cx.is_kind(TokenKind::Int) {
        // Range expression: 0..10 or 0..=n
        let start = cx.cur_text().to_string();
        cx.advance();
        let mut result = start;
        if cx.is_kind(TokenKind::Range) {
            cx.advance();
            result.push_str("..");
            result.push_str(&cx.cur_text().to_string());
            cx.advance();
        } else if cx.is_kind(TokenKind::RangeEq) {
            cx.advance();
            result.push_str("..=");
            result.push_str(&cx.cur_text().to_string());
            cx.advance();
        }
        result
    } else {
        // Plan 356 follow-up: accept `ident.field[.sub...]` iterables
        // (e.g. `note.tags`, `store.items`). The `.field` branch above
        // already handles chains starting with a dot; this branch handles
        // chains starting with a bare identifier. Without it, `note.tags`
        // was read as just `note`, leaving `.tags` to break the rest of
        // the view ("Expected term, got RBrace").
        let first = cx.cur_text().to_string();
        cx.advance();
        let mut result = first;
        while cx.is_kind(TokenKind::Dot) {
            cx.advance(); // skip dot
            let name = cx.cur_text().to_string();
            cx.advance();
            result.push_str(&format!(".{}", name));
        }
        result
    };

    // PLAN-046 T-02: optional keyed-for clause — `for x in .items key: .id { }`.
    // `key` is a soft identifier (not a keyword); the clause is probed only
    // in the slot between the iterable and the body brace, a position no
    // other form fills with a bare Ident. The bounded expression parser
    // stops before the body `{` by construction.
    let key_expr = if cx.is_kind(TokenKind::Ident) && cx.cur_text().as_str() == "key" {
        cx.advance(); // consume 'key'
        cx.expect(TokenKind::Colon)?;
        Some(parse_view_key_expr(cx)?)
    } else {
        None
    };

    // Enter new scope for loop body
    cx.enter_scope();

    // Define loop variable in scope
    cx.bind_var(var.as_str(), Type::Unknown);

    // Define index variable if present
    if let Some(ref idx) = index {
        cx.bind_var(idx.as_str(), Type::Int);
    }

    // Parse body
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut body = Vec::new();
    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }
        let node = parse_view_node(cx)?;
        body.push(node);
        cx.skip_empty_lines();
    }
    cx.expect(TokenKind::RBrace)?;

    // Exit scope
    cx.exit_scope();

    Ok(ViewNode::ForLoop {
        var,
        index,
        iterable,
        key_expr,
        body,
        span: Some((
            start_pos.pos,
            cx.prev_pos().pos + cx.prev_pos().len - start_pos.pos,
        )),
    })
}

/// PLAN-046: bounded key-expression parser for the keyed-for clause.
/// Accepts the cheap pure forms a cache key may take — state chains
/// (`.a.b` → self-rooted Dot, same shape as expression position),
/// item field chains (`item.id`), and scalar literals (Int/Str) — and
/// stops before the body `{` by construction (no call/index/object
/// grammar is reachable). Keys are fingerprinted per item; VM code
/// shapes are deliberately rejected with a pointed error.
pub(crate) fn parse_view_key_expr(cx: &mut dyn UiParseContext) -> AutoResult<Expr> {
    let lhs: Expr = match cx.cur_kind() {
        TokenKind::Dot => {
            cx.advance(); // skip dot
            let name = cx.cur_text();
            cx.advance();
            Expr::Dot(Box::new(Expr::Ident("self".into())), name.into())
        }
        TokenKind::Ident => {
            let name = cx.cur_text();
            cx.advance();
            Expr::Ident(name.into())
        }
        TokenKind::Int => {
            let text = cx.cur_text();
            cx.advance();
            let value: i32 = text.parse().map_err(|_| SyntaxError::Generic {
                message: format!("key clause Int literal out of range: `{}`", text),
                span: pos_to_span(cx.prev_pos()),
            })?;
            Expr::Int(value)
        }
        TokenKind::Str => {
            let value = cx.cur_text();
            cx.advance();
            Expr::Str(value.into())
        }
        other => {
            return Err(SyntaxError::Generic {
                message: format!(
                    "key clause accepts state chains (.a.b), field chains (item.id), \
                         or Int/Str literals — got {:?}",
                    other
                ),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }
    };

    let mut lhs = lhs;
    while cx.is_kind(TokenKind::Dot) {
        cx.advance(); // skip dot
        let field = cx.cur_text();
        cx.advance();
        lhs = Expr::Dot(Box::new(lhs), field.into());
    }
    Ok(lhs)
}

/// PLAN-046 T-04: explicit memo block — `memo (deps: .a, .b) { … }`,
/// `exact: true` optional. deps = comma-separated expressions（语料作者
/// 声明的依赖面，值指纹进命中条件）; exact = bool 字面量，选择纯 deps
/// 信任模式（canonical 档明示：陈旧风险由语料承担——正确性下限的唯一
/// 显式豁口）。
pub(crate) fn parse_view_memo_block(cx: &mut dyn UiParseContext) -> AutoResult<ViewNode> {
    let start_pos = cx.cur_pos();
    cx.expect_ident("memo")?;
    cx.expect(TokenKind::LParen)?;
    cx.skip_empty_lines();

    let mut deps: Vec<Expr> = Vec::new();
    let mut exact = false;
    while !cx.is_kind(TokenKind::RParen) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RParen) {
            break;
        }
        let key = cx.cur_text().to_string();
        cx.advance();
        cx.expect(TokenKind::Colon)?;
        match key.as_str() {
            "deps" => loop {
                cx.skip_empty_lines();
                let e = cx.parse_expr()?;
                match e {
                    // `deps: .a, .b` 的逗号在表达式位置可能被解析为
                    // Tuple（Plan 200）——拍平成逐项 deps。
                    Expr::Tuple(items) => deps.extend(items),
                    other => deps.push(other),
                }
                if cx.is_kind(TokenKind::Comma) {
                    cx.advance();
                    cx.skip_empty_lines();
                    // 逗号后跟 `exact:` = deps 列表结束、下一头参开始。
                    if cx.is_kind(TokenKind::Ident) && cx.cur_text().as_str() == "exact" {
                        break;
                    }
                    continue;
                }
                break;
            },
            "exact" => {
                let b = cx.cur_text().to_string();
                cx.advance();
                exact = match b.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => {
                        return Err(SyntaxError::Generic {
                            message: format!("memo exact 只接受 true/false 字面量，得到 `{}`", b),
                            span: pos_to_span(cx.prev_pos()),
                        }
                        .into());
                    }
                };
            }
            other => {
                return Err(SyntaxError::Generic {
                    message: format!("memo 头参只接受 deps/exact，得到 `{}`", other),
                    span: pos_to_span(cx.prev_pos()),
                }
                .into());
            }
        }
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::Comma) {
            cx.advance();
            cx.skip_empty_lines();
        }
    }
    cx.expect(TokenKind::RParen)?;

    // Block body: { node … }
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();
    let mut body = Vec::new();
    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }
        let node = parse_view_node(cx)?;
        body.push(node);
        cx.skip_empty_lines();
    }
    cx.expect(TokenKind::RBrace)?;

    Ok(ViewNode::MemoBlock {
        deps,
        exact,
        body,
        span: Some((
            start_pos.pos,
            cx.prev_pos().pos + cx.prev_pos().len - start_pos.pos,
        )),
    })
}

/// Parse conditional in view: if condition { then_body } else { else_body }
pub(crate) fn parse_view_conditional(cx: &mut dyn UiParseContext) -> AutoResult<ViewNode> {
    let start_pos = cx.cur_pos();
    cx.expect_ident("if")?;

    // Parse condition expression (until we hit '{')
    let condition = parse_condition_expr(cx)?;

    // Parse then body
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut then_body = Vec::new();
    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }
        let node = parse_view_node(cx)?;
        then_body.push(node);
        cx.skip_empty_lines();
    }
    cx.expect(TokenKind::RBrace)?;

    // Check for else clause
    let else_body = if cx.cur_text().as_str() == "else" {
        cx.expect_ident("else")?;
        // Plan 367 P2-1: support "else if" chains by peeking for "if"
        // after "else". If found, recursively parse the conditional
        // and wrap it as a single-element else_body. This naturally
        // handles multi-level "else if ... else if ... else" chains.
        if cx.cur_text().as_str() == "if" {
            let nested = parse_view_conditional(cx)?;
            Some(vec![nested])
        } else {
            cx.expect(TokenKind::LBrace)?;
            cx.skip_empty_lines();

            let mut else_nodes = Vec::new();
            while !cx.is_kind(TokenKind::RBrace) {
                cx.skip_empty_lines();
                if cx.is_kind(TokenKind::RBrace) {
                    break;
                }
                let node = parse_view_node(cx)?;
                else_nodes.push(node);
                cx.skip_empty_lines();
            }
            cx.expect(TokenKind::RBrace)?;
            Some(else_nodes)
        }
    } else {
        None
    };

    Ok(ViewNode::Conditional {
        condition,
        then_body,
        else_body,
        span: Some((
            start_pos.pos,
            cx.prev_pos().pos + cx.prev_pos().len - start_pos.pos,
        )),
    })
}

/// Parse navigation link in view: link (to: "/path") { children } (Plan 105)
/// Also supports: link (text: "label", href: "#") {} for external links
pub(crate) fn parse_view_link(cx: &mut dyn UiParseContext) -> AutoResult<ViewNode> {
    let start_pos = cx.cur_pos();
    cx.expect(TokenKind::Link)?;

    // Plan 502(预存债修复):`link "label" {}` 位置参数文本形态——与
    // text/badge 等文本主属性元素对齐。此前仅支持 (props) 形态,
    // kitchen-sink(docs_gen 产物)的 link "sample" {} 自 435 起整体
    // 解析失败,vue 轨路由 import 断链整站白屏。
    let mut to = String::new();
    let mut text = String::new();
    let mut href = String::new();
    if cx.is_kind(TokenKind::Str) {
        text = cx.cur_text().to_string();
        cx.advance();
    }

    if cx.is_kind(TokenKind::LParen) {
        cx.advance();
        cx.skip_empty_lines();

        while !cx.is_kind(TokenKind::RParen) {
            cx.skip_empty_lines();
            if cx.is_kind(TokenKind::RParen) {
                break;
            }

            let key = cx.cur_text().to_string();
            cx.advance();
            cx.expect(TokenKind::Colon)?;

            if key == "to" {
                to = cx.cur_text().to_string();
                cx.expect(TokenKind::Str)?;
            } else if key == "text" {
                text = cx.cur_text().to_string();
                cx.expect(TokenKind::Str)?;
            } else if key == "href" {
                href = cx.cur_text().to_string();
                cx.expect(TokenKind::Str)?;
            } else {
                // Skip unknown props
                cx.advance();
            }

            if cx.is_kind(TokenKind::Comma) {
                cx.advance();
            }
        }
        cx.expect(TokenKind::RParen)?;
    }

    // Parse children in braces
    let mut children = Vec::new();
    if cx.is_kind(TokenKind::LBrace) {
        cx.advance();
        cx.skip_empty_lines();

        while !cx.is_kind(TokenKind::RBrace) {
            cx.skip_empty_lines();
            if cx.is_kind(TokenKind::RBrace) {
                break;
            }
            let child = parse_view_node(cx)?;
            children.push(child);
            cx.skip_empty_lines();
        }
        cx.expect(TokenKind::RBrace)?;
    }

    // If text is provided but no children, create a text child
    if !text.is_empty() && children.is_empty() {
        children.push(ViewNode::Text(ViewText::Literal(text.clone())));
    }

    Ok(ViewNode::Link {
        to,
        text,
        href,
        children,
        span: Some((
            start_pos.pos,
            cx.prev_pos().pos + cx.prev_pos().len - start_pos.pos,
        )),
    })
}

/// Parse condition expression (until '{')
pub(crate) fn parse_condition_expr(cx: &mut dyn UiParseContext) -> AutoResult<String> {
    let mut parts = Vec::new();

    while !cx.is_kind(TokenKind::LBrace) {
        if cx.is_kind(TokenKind::Dot) {
            cx.advance();
            let name = cx.cur_text().to_string();
            cx.advance();
            let mut chain = format!(".{}", name);
            // Handle chained field access: .book.id -> .book.id
            while cx.is_kind(TokenKind::Dot) {
                cx.advance();
                let next_name = cx.cur_text().to_string();
                cx.advance();
                chain.push_str(&format!(".{}", next_name));
            }
            parts.push(chain);
        } else if cx.is_kind(TokenKind::Ident) {
            let text = cx.cur_text().to_string();
            cx.advance();
            // Build chained access: ident.field.method(args).another()
            let mut chain = text;
            // Plan 028 F2: direct fn call in condition position —
            // `if isLastMessage(.msgs, m.id) { }`. Previously the bare
            // name was pushed and the `(`-group fell apart at the first
            // comma, so multi-arg calls forced pre-computed bypasses.
            if cx.is_kind(TokenKind::LParen) {
                capture_condition_call_args(cx, &mut chain);
            }
            while cx.is_kind(TokenKind::Dot) {
                cx.advance();
                let member = cx.cur_text().to_string();
                cx.advance();
                chain.push_str(&format!(".{}", member));
                // Handle method call parentheses
                if cx.is_kind(TokenKind::LParen) {
                    cx.advance();
                    chain.push('(');
                    while !cx.is_kind(TokenKind::RParen) && !cx.is_kind(TokenKind::LBrace) {
                        chain.push_str(&cx.cur_text().to_string());
                        cx.advance();
                    }
                    if cx.is_kind(TokenKind::RParen) {
                        cx.advance();
                        chain.push(')');
                    }
                }
            }
            parts.push(chain);
        } else if cx.is_kind(TokenKind::Lt)
            || cx.is_kind(TokenKind::Gt)
            || cx.is_kind(TokenKind::Le)
            || cx.is_kind(TokenKind::Ge)
            || cx.is_kind(TokenKind::Eq)
            || cx.is_kind(TokenKind::Neq)
            || cx.is_kind(TokenKind::And)
            || cx.is_kind(TokenKind::Or)
            || cx.is_kind(TokenKind::Not)
        {
            let op = cx.cur_text().to_string();
            cx.advance();
            parts.push(op);
        } else if cx.is_kind(TokenKind::Int) {
            let num = cx.cur_text().to_string();
            cx.advance();
            parts.push(num);
        } else if cx.is_kind(TokenKind::Float) {
            // Plan 015 P1#5 (gap 37a follow-up): float literals in event
            // args — previously only Int parsed, floats fell to the loud
            // unsupported-argument error.
            let num = cx.cur_text().to_string();
            cx.advance();
            parts.push(num);
        } else if cx.is_kind(TokenKind::True) || cx.is_kind(TokenKind::False) {
            // Plan 015 P1#5: bool literals in view event args
            // (".HoverChange(true)") — previously a loud parse error,
            // forcing int 1/0 stand-ins.
            let b = cx.cur_text().to_string();
            cx.advance();
            parts.push(b);
        } else if cx.is_kind(TokenKind::Str) {
            let s = format!("\"{}\"", cx.cur_text());
            cx.advance();
            parts.push(s);
        } else if cx.is_kind(TokenKind::LParen) {
            cx.advance();
            parts.push("(".to_string());
        } else if cx.is_kind(TokenKind::RParen) {
            cx.advance();
            parts.push(")".to_string());
        } else if cx.is_kind(TokenKind::Range) || cx.is_kind(TokenKind::RangeEq) {
            let op = cx.cur_text().to_string();
            cx.advance();
            parts.push(op);
        } else if cx.is_kind(TokenKind::Add)
            || cx.is_kind(TokenKind::Sub)
            || cx.is_kind(TokenKind::Star)
            || cx.is_kind(TokenKind::Div)
            || cx.is_kind(TokenKind::Mod)
        {
            let op = cx.cur_text().to_string();
            cx.advance();
            parts.push(op);
        } else if cx.is_kind(TokenKind::True) || cx.is_kind(TokenKind::False) {
            let text = cx.cur_text().to_string();
            cx.advance();
            parts.push(text);
        } else if cx.is_kind(TokenKind::NoneKW) || cx.is_kind(TokenKind::Nil) {
            // Plan 043 M5: `None` / `nil` literal in a view condition
            // (e.g. `if .x != None`). Previously fell through to `break`
            // and desynced the parser, surfacing as "Expected term, got
            // RBrace" at the arm's closing brace.
            let text = cx.cur_text().to_string();
            cx.advance();
            parts.push(text);
        } else {
            break;
        }
    }

    Ok(parts.join(" "))
}

/// Plan 028 F2: capture a balanced `(...)` argument group after a fn name
/// in a view condition, verbatim — nested calls, commas, and string
/// literals included (`isLastMessage(.msgs, m.id)`, `f(g(.x), 2)`).
/// `self.cur` must be the opening LParen; on return the closing RParen is
/// consumed. Unbalanced input stops at EOF without consuming a body brace.
pub(crate) fn capture_condition_call_args(cx: &mut dyn UiParseContext, chain: &mut String) {
    chain.push('(');
    cx.advance();
    let mut depth = 1usize;
    while !cx.is_kind(TokenKind::EOF) {
        if cx.is_kind(TokenKind::LParen) {
            depth += 1;
            chain.push('(');
            cx.advance();
        } else if cx.is_kind(TokenKind::RParen) {
            depth -= 1;
            chain.push(')');
            cx.advance();
            if depth == 0 {
                break;
            }
        } else if cx.is_kind(TokenKind::Str) {
            chain.push_str(&format!("\"{}\"", cx.cur_text()));
            cx.advance();
        } else if cx.is_kind(TokenKind::Comma) {
            chain.push_str(", ");
            cx.advance();
        } else {
            chain.push_str(&cx.cur_text().to_string());
            cx.advance();
        }
    }
}

/// Parse event handler with optional parameters: .Inc or .Delete(todo.id) or nav("route", data)
/// Consume an optional `.modifier` chain after an `on*` event key.
///
/// Supported modifier kinds (interpreted by the code generators):
/// - event modifiers: `prevent`, `stop`, `capture`, `once`, `self`, `passive`
/// - key modifiers: `enter`, `esc`/`escape`, `up`, `down`, `left`, `right`,
///   `space`, `tab`, `del`/`delete`
/// - global targets: `window`, `document` (generator emits
///   addEventListener/removeEventListener pairs instead of a template attr)
///
/// Example: `onkeydown.up.prevent: .MoveUp`, `onwheel.document.capture: .Lock`
pub(crate) fn parse_event_modifiers(
    cx: &mut dyn UiParseContext,
    mut key: String,
) -> AutoResult<String> {
    while cx.is_kind(TokenKind::Dot) {
        cx.advance();
        if !cx.is_kind(TokenKind::Ident) && !cx.cur_is_soft_ident() {
            return Err(SyntaxError::Generic {
                message: format!(
                    "expected event modifier after `{}.`, got `{}`",
                    key,
                    cx.cur_text()
                ),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }
        let modifier = cx.cur_text().to_string();
        cx.advance();
        key.push('.');
        key.push_str(&modifier);
    }
    Ok(key)
}

/// Plan 448 B1: parse the value of an `on*` event binding.
///
/// Legacy forms (`.Inc`, `.Delete(todo.id)`, `nav("route", data)`) go
/// through `parse_event_handler`. A value starting with `(` can only be
/// an inline lambda (`onclick: () => { .count += 1 }`) — no legacy form
/// starts with a paren — so it parses via `parse_closure` and returns the
/// lambda's statements with an empty handler string; extraction mints the
/// anonymous event name and injects the matching on-handler.
pub(crate) fn parse_event_value(
    cx: &mut dyn UiParseContext,
) -> AutoResult<(String, Vec<String>, Option<Vec<Stmt>>)> {
    if cx.is_kind(TokenKind::LParen) {
        let closure = cx.parse_closure()?;
        if let Expr::Closure(c) = &closure {
            let params = c
                .params
                .iter()
                .map(|p| p.name.to_string())
                .collect::<Vec<_>>();
            let body_stmts = match c.body.as_ref() {
                Expr::Block(b) => b.stmts.clone(),
                // Expression body: keep it as an expression statement.
                other => vec![Stmt::Expr(other.clone())],
            };
            return Ok((String::new(), params, Some(body_stmts)));
        }
        return Err(SyntaxError::Generic {
            message: "expected a closure after `(` in event handler".to_string(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    let (handler, params) = parse_event_handler(cx)?;
    Ok((handler, params, None))
}

pub(crate) fn parse_event_handler(
    cx: &mut dyn UiParseContext,
) -> AutoResult<(String, Vec<String>)> {
    // Handle dot-prefixed handlers like .Inc (meaning Msg::Inc in widget scope)
    if cx.is_kind(TokenKind::Dot) {
        cx.advance(); // consume the dot
        let name = cx.cur_text().to_string();
        cx.advance();

        // Check for parameters: .Delete(todo.id)
        let params = if cx.is_kind(TokenKind::LParen) {
            cx.advance();
            let args = parse_event_arg_list(cx)?;
            args
        } else {
            Vec::new()
        };

        Ok((format!(".{}", name), params))
    } else {
        // Handle function calls like nav("route", data)
        let handler = cx.cur_text().to_string();
        cx.advance();

        // Check for parameters: nav("route", data)
        let params = if cx.is_kind(TokenKind::LParen) {
            cx.advance();
            let args = parse_event_arg_list(cx)?;
            args
        } else {
            Vec::new()
        };

        Ok((handler, params))
    }
}

/// Parse a comma/semicolon-separated argument list terminated by `)`.
/// The opening `(` has already been consumed; this consumes the closing `)`.
///
/// Plan 356: carries a runaway guard so an unhandled token (which would
/// leave the cursor stuck) produces a clean error instead of an infinite
/// loop / OOM. This is the structural fix for the `tag`-as-argument bug,
/// complementing the soft-identifier handling in `parse_event_arg`.
pub(crate) fn parse_event_arg_list(cx: &mut dyn UiParseContext) -> AutoResult<Vec<String>> {
    let mut args = Vec::new();
    let mut iters = 0u32;
    while !cx.is_kind(TokenKind::RParen) {
        iters += 1;
        if iters > 64 {
            return Err(SyntaxError::Generic {
                message: format!(
                    "too many event-handler arguments (near `{}`)",
                    cx.cur_text()
                ),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }
        let arg = parse_event_arg(cx)?;
        args.push(arg);
        // Support both comma and semicolon as separators
        if cx.is_kind(TokenKind::Comma) || cx.is_kind(TokenKind::Semi) {
            cx.advance();
        }
    }
    cx.expect(TokenKind::RParen)?;
    Ok(args)
}

/// Parse a single event argument
pub(crate) fn parse_event_arg(cx: &mut dyn UiParseContext) -> AutoResult<String> {
    // Handle object literals: { key: value, ... }
    if cx.is_kind(TokenKind::LBrace) {
        return parse_event_arg_object(cx);
    }

    let mut parts: Vec<String> = Vec::new();

    // Handle expressions like: todo.id, .count, 123, "string"
    // Track if previous token was an identifier (for proper dot handling)
    let mut prev_was_ident = false;
    // Plan 356: defensive runaway guard. A single event argument is a short
    // expression; if this loop ever fails to make progress (a future token
    // no branch consumes), bail with a clear error instead of spinning into
    // OOM. 512 is far above any legitimate argument length.
    let mut iters = 0u32;
    loop {
        iters += 1;
        if iters > 512 {
            return Err(SyntaxError::Generic {
                message: format!(
                    "event argument too complex or unterminated (near `{}`)",
                    cx.cur_text()
                ),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }
        if cx.is_kind(TokenKind::Dot) {
            cx.advance();
            let name = cx.cur_text().to_string();
            cx.advance();
            // If previous was an identifier (like 'item'), use property access syntax
            // Otherwise, this is .field shorthand -> convert to this.field
            if prev_was_ident {
                parts.push(format!(".{}", name));
            } else {
                // Standalone .field -> this.field
                parts.push(format!("this.{}", name));
            }
            // Plan 043 M5 B-5: after emitting `this.field`, the next `.name`
            // in `this.field.name` is a property access, NOT another
            // `this.` prefix. Without this, `.c.name` parsed as
            // `this.c` + `this.name` = `this.cthis.name`, and the Vue
            // backend's `this.`-strip turned it into the bogus identifier
            // `cthis` (undefined — the v-for binds `c`).
            prev_was_ident = true;
            // Plan 015 P1#5: ".Handler(args)" — a dot-prefixed handler name
            // followed by an argument list (the args themselves may be
            // literals like true / 0.5).
            if cx.is_kind(TokenKind::LParen) {
                cx.advance();
                parts.push("(".to_string());
                let mut first = true;
                while !cx.is_kind(TokenKind::RParen) {
                    if !first {
                        if cx.is_kind(TokenKind::Comma) {
                            cx.advance();
                            parts.push(", ".to_string());
                        }
                    }
                    first = false;
                    let arg = parse_event_arg(cx)?;
                    parts.push(arg);
                }
                cx.expect(TokenKind::RParen)?;
                parts.push(")".to_string());
            }
        } else if cx.is_kind(TokenKind::FStrNote) {
            // `$event` (and `$event.field`) — the DOM event object, mapped
            // verbatim to Vue's `$event` template variable by the Vue
            // generator. `$` lexes as FStrNote outside f-strings.
            cx.advance();
            let name = cx.cur_text().to_string();
            cx.advance();
            parts.push(format!("${}", name));
            prev_was_ident = true;
        } else if cx.is_kind(TokenKind::Ident) || cx.cur_is_soft_ident() {
            // Plan 356: also accept soft keywords used as identifiers in
            // argument position (e.g. a loop variable named `tag`, which
            // lexes as TokenKind::Tag). Without this, such a token matched
            // no branch here, so parse_event_arg returned "" *without
            // consuming it*, and the caller's arg loop spun forever → OOM.
            let text = cx.cur_text().to_string();
            cx.advance();
            parts.push(text);
            prev_was_ident = true;

            // Check for function call: ident(args)
            if cx.is_kind(TokenKind::LParen) {
                cx.advance();
                parts.push("(".to_string());
                let mut first = true;
                while !cx.is_kind(TokenKind::RParen) {
                    if !first {
                        if cx.is_kind(TokenKind::Comma) {
                            cx.advance();
                            parts.push(", ".to_string());
                        }
                    }
                    first = false;
                    let arg = parse_event_arg(cx)?;
                    parts.push(arg);
                }
                cx.expect(TokenKind::RParen)?;
                parts.push(")".to_string());
            }
        } else if cx.is_kind(TokenKind::Int) {
            let num = cx.cur_text().to_string();
            cx.advance();
            parts.push(num);
        } else if cx.is_kind(TokenKind::Float)
            || cx.is_kind(TokenKind::True)
            || cx.is_kind(TokenKind::False)
        {
            // Plan 015 P1#5: float and bool literals in view event args
            // (".HoverChange(true)") — previously loud parse errors.
            let lit = cx.cur_text().to_string();
            cx.advance();
            parts.push(lit);
        } else if cx.is_kind(TokenKind::Str) {
            let s = format!("\"{}\"", cx.cur_text().as_str());
            cx.advance();
            parts.push(s);
        } else {
            // Plan 345 (OOM fix): handle binary operators so an event arg
            // like .Bump(.n + 1) doesn't fall out here. Previously the
            // caller's arg loop (which only handles ,/;/) would spin on the
            // leftover operator token, pushing empty args forever -> 48 GiB
            // OOM. Consume the operator as part of this arg expression.
            let op_text = cx.cur_text().to_string();
            if matches!(
                op_text.as_str(),
                "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&" | "||"
            ) {
                parts.push(format!(" {} ", op_text));
                cx.advance();
                prev_was_ident = false;
            } else {
                break;
            }
        }
    }

    // Plan error-spans (jade gap 37a): if no branch consumed anything, the
    // current token is an unsupported event argument (e.g. the bool literal
    // `false` in `."update:open"(false)`). Previously this returned ""
    // without consuming, so the caller's arg loop spun until its runaway
    // guard fired; top-level error recovery then churned out derivative
    // errors and the final report pointed at the view's closing `}` —
    // far from the offending token. Fail loudly AT the offending token.
    if parts.is_empty() {
        return Err(SyntaxError::Generic {
                message: format!(
                    "unsupported event argument `{}` (expected a model ref like `.field`, a loop variable, a string, a number, a map literal, or `$event`)",
                    cx.cur_text()
                ),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
    }

    Ok(parts.join(""))
}

/// Parse an object literal as event argument: { key: value, ... }
pub(crate) fn parse_event_arg_object(cx: &mut dyn UiParseContext) -> AutoResult<String> {
    cx.expect(TokenKind::LBrace)?;
    let mut parts = vec!["{ ".to_string()];
    let mut first = true;

    while !cx.is_kind(TokenKind::RBrace) {
        if !first {
            if cx.is_kind(TokenKind::Comma) {
                cx.advance();
                parts.push(", ".to_string());
            }
        }
        first = false;

        // Parse key (identifier or string)
        if cx.is_kind(TokenKind::Ident) || cx.cur_is_soft_ident() {
            parts.push(cx.cur_text().to_string());
            cx.advance();
        } else if cx.is_kind(TokenKind::Str) {
            parts.push(format!("\"{}\"", cx.cur_text().as_str()));
            cx.advance();
        } else {
            // Skip unknown tokens
            break;
        }

        // Expect colon
        if cx.is_kind(TokenKind::Colon) {
            cx.advance();
            parts.push(": ".to_string());
        } else {
            break;
        }

        // Parse value (recursively)
        let value = parse_event_arg(cx)?;
        parts.push(value);
    }

    cx.expect(TokenKind::RBrace)?;
    parts.push(" }".to_string());

    Ok(parts.join(""))
}

/// Parse on block for widget (returns OnBlock, not OnEvents)
/// Plan 435 挂账②续:on 块 handler 形参收集 —— `Name(p1, p2)` 或
/// `Name(n1 t1, n2 t2)`(类型标注忽略,取每组首 token 为参数名)。
/// 点前缀/无点前缀两种 pattern 形态共用。
pub(crate) fn parse_handler_params(cx: &mut dyn UiParseContext) -> AutoResult<Vec<String>> {
    if !cx.is_kind(TokenKind::LParen) {
        return Ok(Vec::new());
    }
    cx.advance();
    let mut groups: Vec<Vec<String>> = Vec::new();
    while !cx.is_kind(TokenKind::RParen) {
        let mut group = Vec::new();
        while !cx.is_kind(TokenKind::Comma) && !cx.is_kind(TokenKind::RParen) {
            group.push(cx.cur_text().to_string());
            cx.advance();
        }
        if !group.is_empty() {
            groups.push(group);
        }
        if cx.is_kind(TokenKind::Comma) {
            cx.advance();
        }
    }
    cx.expect(TokenKind::RParen)?;
    Ok(groups
        .into_iter()
        .filter_map(|g| g.into_iter().next())
        .collect())
}
