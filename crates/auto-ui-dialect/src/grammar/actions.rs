//! actions/timer/routes/menubar/toolbar 与顶层 style recipe 声明文法
//! （PLAN-810 T-03/T-04 迁移）。

use super::helpers::at_dot;
use super::view::capture_condition_call_args;
use crate::UiParseContext;
use auto_parser_api::ast;
use auto_parser_api::ast::*;
use auto_parser_api::error::{pos_to_span, AutoResult, SyntaxError};
use auto_parser_api::token::TokenKind;

/// Parse routes block, returning the RoutesBlock directly (Plan 105/106)
///
/// Syntax (Plan 106 - recommended):
/// ```auto
/// routes {
///     "/" -> use index
///     "/button" -> use button
///     "/user/:id" -> use user
/// }
/// ```
///
/// Syntax (Plan 105 - backward compatible):
/// ```auto
/// routes {
///     "/button" -> ButtonPage {}
///     "/user/:id" -> UserPage {}
/// }
/// ```
pub(crate) fn parse_routes_block_inner(cx: &mut dyn UiParseContext) -> AutoResult<RoutesBlock> {
    cx.expect_ident("routes")?;
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut routes = Vec::new();

    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }

        // Parse: "/path" -> use module_name (Plan 106)
        //     or: "/path" -> ComponentName {} (Plan 105, backward compat)
        let path = cx.cur_text().to_string();
        cx.expect(TokenKind::Str)?;
        cx.expect(TokenKind::Arrow)?;

        // Check if next token is `use` keyword (Plan 106 syntax)
        let module = if cx.is_kind(TokenKind::Use) {
            // Plan 106: "/path" -> use module_name
            cx.advance(); // consume 'use'
            let module_name = cx.cur_text().to_string();
            cx.expect(TokenKind::Ident)?;
            module_name
        } else {
            // Plan 105 (backward compat): "/path" => ComponentName {}
            let component = cx.cur_text().to_string();
            cx.expect(TokenKind::Ident)?;

            // Parse empty braces: ComponentName {}
            cx.expect(TokenKind::LBrace)?;
            cx.skip_empty_lines();
            cx.expect(TokenKind::RBrace)?;

            // Convert PascalCase component name to lowercase module name
            // e.g., "ButtonPage" -> "button", "IndexPage" -> "index"
            component.to_lowercase()
        };

        routes.push(RouteDef::new(path, module));

        // Optional comma
        if cx.is_kind(TokenKind::Comma) {
            cx.advance();
        }
        cx.skip_empty_lines();
    }

    cx.expect(TokenKind::RBrace)?;
    Ok(RoutesBlock { routes })
}

/// Plan 451 P3: 顶层 `actions { ... }` 声明（模块级）。仅在 `actions`
/// 后继 `{` 时接管（否则返回 None 回退表达式路径——`actions = ...` 等
/// 普通表达式不受影响）；handler 引用此时无宿主可校验，合并进宿主
/// widget 后由消费端（vm 安装/vue 生成）落地。
pub fn parse_actions_decl_stmt(cx: &mut dyn UiParseContext) -> AutoResult<Option<Stmt>> {
    // cur = "actions"；词法前瞻一格（取回再压回，parser 流式无回溯）。
    let next_is_brace = match cx.probe_next_token() {
        Ok(t) => t.kind == TokenKind::LBrace,
        Err(_) => return Ok(None),
    };
    if !next_is_brace {
        return Ok(None);
    }
    let block = parse_actions_block_inner(cx)?;
    Ok(Some(Stmt::ActionsDecl(block)))
}

/// PLAN-607: 顶层 `style <name> = ...` 或 `style <name>(...) = ...` 声明。
/// 仅在 `style` 后继 `Ident` 且接着 `=` 或 `(` 时接管；
/// 否则返回 None 回退表达式路径（`style = ...` 等普通表达式不受影响）。
pub fn parse_style_recipe_decl_stmt(cx: &mut dyn UiParseContext) -> AutoResult<Option<Stmt>> {
    let cp = cx.raw_cursor_save();

    cx.advance(); // consume "style"
    if !cx.is_kind(TokenKind::Ident) {
        cx.raw_cursor_restore(cp)?;
        return Ok(None);
    }
    let recipe_name = cx.cur_text();
    cx.advance(); // consume recipe name

    // Lookahead: after recipe name, must be '=' or '('
    if !cx.is_kind(TokenKind::Asn) && !cx.is_kind(TokenKind::LParen) {
        cx.raw_cursor_restore(cp)?;
        return Ok(None);
    }
    // F810-R1-01：投机承诺（确认为 recipe 声明）即释放快照——此后
    // 一切成功/Err 出口不再持有句柄表槽位（旧臂 saved_* 出作用域即
    // 释放的端口等价）。
    cx.raw_cursor_release(cp);

    // Confirmed style recipe declaration
    let doc = cx.take_docs();
    let mut params = Vec::new();
    if cx.is_kind(TokenKind::LParen) {
        cx.advance(); // consume '('
        cx.skip_empty_lines();
        while !cx.is_kind(TokenKind::RParen) {
            let param_name = cx.cur_text();
            cx.expect(TokenKind::Ident)?;

            let param_type = if cx.is_kind(TokenKind::Colon) {
                cx.advance();
                Some(cx.parse_type()?)
            } else {
                None
            };

            let default_value = if cx.is_kind(TokenKind::Asn) {
                cx.advance();
                Some(cx.parse_expr()?)
            } else {
                None
            };

            params.push(ast::ui::StyleRecipeParam {
                name: param_name.into(),
                param_type,
                default_value,
            });

            cx.skip_empty_lines();
            if cx.is_kind(TokenKind::Comma) {
                cx.advance();
                cx.skip_empty_lines();
            } else {
                break;
            }
        }
        cx.expect(TokenKind::RParen)?;
        cx.skip_empty_lines();
    }

    cx.expect(TokenKind::Asn)?;
    cx.skip_empty_lines();

    let prev_skip = cx.set_skip_check(true);
    let body = cx.parse_expr()?;
    cx.set_skip_check(prev_skip);

    let decl = ast::ui::StyleRecipeDecl {
        name: recipe_name.into(),
        params,
        body,
        is_pub: false,
        doc: doc.map(Into::into),
    };
    // PLAN-607: register style recipe immediately so subsequent AST nodes recognize it
    cx.register_style_recipe(&decl);

    Ok(Some(Stmt::StyleRecipeDecl(decl)))
}

/// Plan 051 C7: `timer { ... }` 块体。已消费 `timer` 标识符。
///
/// ```auto
/// timer {
///     Tick (every_ms: 1000)
///     PollStream (every_ms: 500, when: .streaming)
/// }
/// ```
///
/// 条目 = 裸名（msg 变体）+ 括号属性列表（沿 actions `action (…)` 形）；
/// `when` 复用 `parse_actions_cond_attr`（引号串或裸 DSL 表达式源文本）。
pub(crate) fn parse_timer_block_inner(
    cx: &mut dyn UiParseContext,
) -> AutoResult<ast::ui::TimerBlock> {
    cx.expect_ident("timer")?;
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut entries: Vec<ast::ui::TimerEntry> = Vec::new();
    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }
        if !cx.is_kind(TokenKind::Ident) {
            return Err(SyntaxError::Generic {
                message: format!(
                    "Expected timer entry name in timer block, got '{}'",
                    cx.cur_text()
                ),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }
        let event: ast::Name = cx.cur_text().into();
        cx.advance();
        cx.expect(TokenKind::LParen)?;

        let mut every_ms: Option<u64> = None;
        let mut when: Option<String> = None;
        loop {
            cx.skip_empty_lines();
            if cx.is_kind(TokenKind::RParen) {
                cx.advance();
                break;
            }
            let key = cx.cur_text().to_string();
            cx.advance();
            cx.expect(TokenKind::Colon)?;
            match key.as_str() {
                "every_ms" => {
                    if !cx.is_kind(TokenKind::Int) {
                        return Err(SyntaxError::Generic {
                            message: format!(
                                "timer: every_ms expects an integer literal, got '{}'",
                                cx.cur_text()
                            ),
                            span: pos_to_span(cx.cur_pos()),
                        }
                        .into());
                    }
                    every_ms = Some(cx.cur_text().as_str().parse::<u64>().map_err(|_| {
                        SyntaxError::Generic {
                            message: format!("timer: invalid every_ms literal '{}'", cx.cur_text()),
                            span: pos_to_span(cx.cur_pos()),
                        }
                    })?);
                    cx.advance();
                }
                "when" => {
                    when = Some(parse_actions_cond_attr(cx)?);
                }
                _ => {
                    return Err(SyntaxError::Generic {
                        message: format!(
                            "timer: unknown attribute '{}' (expected every_ms / when)",
                            key
                        ),
                        span: pos_to_span(cx.cur_pos()),
                    }
                    .into());
                }
            }
            cx.skip_empty_lines();
            if cx.is_kind(TokenKind::Comma) {
                cx.advance();
            }
        }

        let every_ms = every_ms.ok_or_else(|| SyntaxError::Generic {
            message: format!(
                "timer: entry '{}' missing required attribute every_ms",
                event
            ),
            span: pos_to_span(cx.cur_pos()),
        })?;
        entries.push(ast::ui::TimerEntry {
            event,
            // 运行期钳制下限 16ms 防忙轮询（解析期即归一）。
            every_ms: every_ms.max(16),
            when,
        });
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::Comma) {
            cx.advance();
        }
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(ast::ui::TimerBlock { entries })
}

/// Plan 451: `actions { ... }` 块——动作注册表 + menubar/toolbar 声明
/// （语法见 ast::ui::ActionsBlock 文档）。enabled_if/checked_if 支持
/// 引号字符串与裸表达式两种拼写（parse_actions_cond_attr）。
pub(crate) fn parse_actions_block_inner(
    cx: &mut dyn UiParseContext,
) -> AutoResult<ast::ui::ActionsBlock> {
    cx.expect_ident("actions")?;
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();

    let mut actions: Vec<ast::ui::ActionEntry> = Vec::new();
    let mut menubar: Option<ast::ui::MenubarBlock> = None;
    let mut toolbar: Option<ast::ui::ToolbarBlock> = None;

    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }
        let ident = cx.cur_text().as_str().to_string();
        match ident.as_str() {
            "action" => {
                cx.advance();
                actions.push(parse_action_entry(cx)?);
            }
            "menubar" => {
                if menubar.replace(parse_menubar_block(cx)?).is_some() {
                    return Err(SyntaxError::Generic {
                        message: "duplicate `menubar` block in actions".into(),
                        span: pos_to_span(cx.cur_pos()),
                    }
                    .into());
                }
            }
            "toolbar" => {
                if toolbar.replace(parse_toolbar_block(cx)?).is_some() {
                    return Err(SyntaxError::Generic {
                        message: "duplicate `toolbar` block in actions".into(),
                        span: pos_to_span(cx.cur_pos()),
                    }
                    .into());
                }
            }
            _ => {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "actions: unexpected `{}` (expected action / menubar / toolbar)",
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
    Ok(ast::ui::ActionsBlock {
        actions,
        menubar,
        toolbar,
    })
}

/// `action (id: "...", handler: .ActXxx, title: "...", icon: "...",
///          shortcut: "...", enabled_if: "...", checked_if: "...")`
/// id 与 handler 必填；其余可选；均为引号字符串（handler 例外，前导点
/// 事件引用）。
pub(crate) fn parse_action_entry(cx: &mut dyn UiParseContext) -> AutoResult<ast::ui::ActionEntry> {
    cx.expect(TokenKind::LParen)?;
    let mut id: Option<String> = None;
    let mut handler: Option<String> = None;
    let mut title: Option<String> = None;
    let mut icon: Option<String> = None;
    let mut shortcut: Option<String> = None;
    let mut enabled_if: Option<String> = None;
    let mut checked_if: Option<String> = None;

    while !cx.is_kind(TokenKind::RParen) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RParen) {
            break;
        }
        let key = cx.cur_text().as_str().to_string();
        cx.advance();
        cx.expect(TokenKind::Colon)?;
        match key.as_str() {
            "id" => id = Some(parse_actions_string_attr(cx)?),
            "handler" => handler = Some(parse_actions_handler_attr(cx)?),
            "title" => title = Some(parse_actions_string_attr(cx)?),
            "icon" => icon = Some(parse_actions_string_attr(cx)?),
            "shortcut" => shortcut = Some(parse_actions_string_attr(cx)?),
            "enabled_if" => enabled_if = Some(parse_actions_cond_attr(cx)?),
            "checked_if" => checked_if = Some(parse_actions_cond_attr(cx)?),
            _ => {
                return Err(SyntaxError::Generic {
                        message: format!(
                            "action: unexpected attribute `{}` (expected id/handler/title/icon/shortcut/enabled_if/checked_if)",
                            key
                        ),
                        span: pos_to_span(cx.cur_pos()),
                    }.into());
            }
        }
        if cx.is_kind(TokenKind::Comma) {
            cx.advance();
        }
    }
    cx.expect(TokenKind::RParen)?;
    let id = id.ok_or_else(|| SyntaxError::Generic {
        message: "action: `id` is required".into(),
        span: pos_to_span(cx.cur_pos()),
    })?;
    let handler = handler.ok_or_else(|| SyntaxError::Generic {
        message: "action: `handler` is required".into(),
        span: pos_to_span(cx.cur_pos()),
    })?;
    Ok(ast::ui::ActionEntry {
        id,
        handler,
        title,
        icon,
        shortcut,
        enabled_if,
        checked_if,
    })
}

/// actions 块内 `key: "..."` 的引号字符串取值（去引号；text 带引号是
/// lexer 惯例，见 style name 解析）。
pub(crate) fn parse_actions_string_attr(cx: &mut dyn UiParseContext) -> AutoResult<String> {
    let s = cx.cur_text().to_string();
    cx.expect(TokenKind::Str)?;
    Ok(s.trim_matches('"').to_string())
}

/// actions 块内 `handler: .ActXxx`——前导点事件引用。前导点的词法
/// 形态两可（Dot token 或 text 为 "." 的标识符，parse_name 的 "." 特例
/// 同源），两者都接受。
pub(crate) fn parse_actions_handler_attr(cx: &mut dyn UiParseContext) -> AutoResult<String> {
    if cx.is_kind(TokenKind::Dot) || cx.cur_text().as_str() == "." {
        cx.advance();
    } else {
        return Err(SyntaxError::Generic {
            message: "action: `handler` expects a `.Event` reference".into(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    let name = cx.cur_text().to_string();
    cx.expect(TokenKind::Ident)?;
    Ok(format!(".{}", name))
}

/// Plan 451 P2: `enabled_if`/`checked_if` 取值——两种拼写：
/// ① 引号字符串（P1 形态，与 auto-atom 外挂文件同源，原样保存）；
/// ② 裸 DSL 表达式（`enabled_if: .store.tab_count > 0`）——按 view if
/// 条件同一 token 文法捕获（空格连接的规范条件串），解析期即校验。
/// 两种形态在 AST/运行时等价：vm 侧 eval_condition_with 对条件串求值，
/// vue 侧 convert_condition 转译（与 view if 条件共用一张翻译表）。
pub(crate) fn parse_actions_cond_attr(cx: &mut dyn UiParseContext) -> AutoResult<String> {
    if cx.is_kind(TokenKind::Str) {
        return parse_actions_string_attr(cx);
    }
    let mut parts: Vec<String> = Vec::new();
    // 分组括号深度：`(.a || .b) && .c` 的组括号按文法收集；深度归零后
    // 的 RParen 是 action 属性行的收尾括号，终止捕获。
    let mut depth: i32 = 0;
    // 前导点词法两可（Dot token 或 text 为 "." 的标识符，同
    // parse_actions_handler_attr 的处理）。
    while !cx.is_kind(TokenKind::Comma) && !cx.is_kind(TokenKind::EOF) {
        if cx.is_kind(TokenKind::RParen) {
            if depth <= 0 {
                break;
            }
            depth -= 1;
            parts.push(")".to_string());
            cx.advance();
            continue;
        }
        if at_dot(cx) {
            cx.advance();
            let name = cx.cur_text().to_string();
            cx.advance();
            let mut chain = format!(".{}", name);
            while at_dot(cx) {
                cx.advance();
                let next_name = cx.cur_text().to_string();
                cx.advance();
                chain.push_str(&format!(".{}", next_name));
            }
            parts.push(chain);
        } else if cx.is_kind(TokenKind::Ident) {
            let mut chain = cx.cur_text().to_string();
            cx.advance();
            if cx.is_kind(TokenKind::LParen) {
                capture_condition_call_args(cx, &mut chain);
            }
            while at_dot(cx) {
                cx.advance();
                let member = cx.cur_text().to_string();
                cx.advance();
                chain.push_str(&format!(".{}", member));
                if cx.is_kind(TokenKind::LParen) {
                    capture_condition_call_args(cx, &mut chain);
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
            || cx.is_kind(TokenKind::Add)
            || cx.is_kind(TokenKind::Sub)
            || cx.is_kind(TokenKind::Star)
            || cx.is_kind(TokenKind::Div)
            || cx.is_kind(TokenKind::Mod)
        {
            parts.push(cx.cur_text().to_string());
            cx.advance();
        } else if cx.is_kind(TokenKind::Int)
            || cx.is_kind(TokenKind::Float)
            || cx.is_kind(TokenKind::True)
            || cx.is_kind(TokenKind::False)
            || cx.is_kind(TokenKind::NoneKW)
            || cx.is_kind(TokenKind::Nil)
        {
            parts.push(cx.cur_text().to_string());
            cx.advance();
        } else if cx.is_kind(TokenKind::Str) {
            // 条件内的字符串字面量（保留引号，eval_condition_with 与
            // convert_condition 均按带引号处理）
            parts.push(format!("\"{}\"", cx.cur_text()));
            cx.advance();
        } else if cx.is_kind(TokenKind::LParen) {
            depth += 1;
            parts.push("(".to_string());
            cx.advance();
        } else {
            return Err(SyntaxError::Generic {
                message: format!(
                    "action condition: unexpected `{}` in enabled_if/checked_if expression",
                    cx.cur_text()
                ),
                span: pos_to_span(cx.cur_pos()),
            }
            .into());
        }
    }
    if parts.is_empty() {
        return Err(SyntaxError::Generic {
            message: "action condition: enabled_if/checked_if expression is empty".into(),
            span: pos_to_span(cx.cur_pos()),
        }
        .into());
    }
    Ok(parts.join(" "))
}

/// `menubar { menu (id: "...", title: "...") { ... } ... }`
pub(crate) fn parse_menubar_block(
    cx: &mut dyn UiParseContext,
) -> AutoResult<ast::ui::MenubarBlock> {
    cx.expect_ident("menubar")?;
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();
    let mut menus = Vec::new();
    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }
        cx.expect_ident("menu")?;
        menus.push(parse_menu_entry(cx)?);
        cx.skip_empty_lines();
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(ast::ui::MenubarBlock { menus })
}

/// `menu (id: "...", title: "...") { item (action: "...") | sep ... }`
pub(crate) fn parse_menu_entry(cx: &mut dyn UiParseContext) -> AutoResult<ast::ui::MenuEntry> {
    cx.expect(TokenKind::LParen)?;
    let mut id: Option<String> = None;
    let mut title: Option<String> = None;
    while !cx.is_kind(TokenKind::RParen) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RParen) {
            break;
        }
        let key = cx.cur_text().as_str().to_string();
        cx.advance();
        cx.expect(TokenKind::Colon)?;
        match key.as_str() {
            "id" => id = Some(parse_actions_string_attr(cx)?),
            "title" => title = Some(parse_actions_string_attr(cx)?),
            _ => {
                return Err(SyntaxError::Generic {
                    message: format!("menu: unexpected attribute `{}` (expected id/title)", key),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
        }
        if cx.is_kind(TokenKind::Comma) {
            cx.advance();
        }
    }
    cx.expect(TokenKind::RParen)?;
    let id = id.ok_or_else(|| SyntaxError::Generic {
        message: "menu: `id` is required".into(),
        span: pos_to_span(cx.cur_pos()),
    })?;
    let title = title.ok_or_else(|| SyntaxError::Generic {
        message: "menu: `title` is required".into(),
        span: pos_to_span(cx.cur_pos()),
    })?;
    let items = parse_menu_item_entries(cx)?;
    Ok(ast::ui::MenuEntry { id, title, items })
}

/// `toolbar { item (action: "...") | sep ... }`
pub(crate) fn parse_toolbar_block(
    cx: &mut dyn UiParseContext,
) -> AutoResult<ast::ui::ToolbarBlock> {
    cx.expect_ident("toolbar")?;
    // 开花括号由 parse_menu_item_entries 消费（与 menu 条目列表共用）。
    let items = parse_menu_item_entries(cx)?;
    Ok(ast::ui::ToolbarBlock { items })
}

/// menu/toolbar 共用的条目列表：`item (action: "...")` 或 `sep`，
/// 换行分隔（允许尾随逗号）。
pub(crate) fn parse_menu_item_entries(
    cx: &mut dyn UiParseContext,
) -> AutoResult<Vec<ast::ui::MenuItemEntry>> {
    cx.expect(TokenKind::LBrace)?;
    cx.skip_empty_lines();
    let mut items = Vec::new();
    while !cx.is_kind(TokenKind::RBrace) {
        cx.skip_empty_lines();
        if cx.is_kind(TokenKind::RBrace) {
            break;
        }
        let ident = cx.cur_text().as_str().to_string();
        match ident.as_str() {
            "sep" => {
                cx.advance();
                items.push(ast::ui::MenuItemEntry::Sep);
            }
            "item" => {
                cx.advance();
                cx.expect(TokenKind::LParen)?;
                let mut action: Option<String> = None;
                while !cx.is_kind(TokenKind::RParen) {
                    cx.skip_empty_lines();
                    if cx.is_kind(TokenKind::RParen) {
                        break;
                    }
                    let key = cx.cur_text().as_str().to_string();
                    cx.advance();
                    cx.expect(TokenKind::Colon)?;
                    if key == "action" {
                        action = Some(parse_actions_string_attr(cx)?);
                    } else {
                        return Err(SyntaxError::Generic {
                            message: format!(
                                "item: unexpected attribute `{}` (expected action)",
                                key
                            ),
                            span: pos_to_span(cx.cur_pos()),
                        }
                        .into());
                    }
                    if cx.is_kind(TokenKind::Comma) {
                        cx.advance();
                    }
                }
                cx.expect(TokenKind::RParen)?;
                let action = action.ok_or_else(|| SyntaxError::Generic {
                    message: "item: `action` is required".into(),
                    span: pos_to_span(cx.cur_pos()),
                })?;
                items.push(ast::ui::MenuItemEntry::Action(action));
            }
            _ => {
                return Err(SyntaxError::Generic {
                    message: format!("unexpected `{}` (expected item / sep)", ident),
                    span: pos_to_span(cx.cur_pos()),
                }
                .into());
            }
        }
        if cx.is_kind(TokenKind::Comma) {
            cx.advance();
        }
        cx.skip_empty_lines();
    }
    cx.expect(TokenKind::RBrace)?;
    Ok(items)
}
