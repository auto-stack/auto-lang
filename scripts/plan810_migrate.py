#!/usr/bin/env python3
"""PLAN-810 T-03/T-04: script-assisted verbatim migration of the UI grammar
region from crates/auto-lang/src/parser.rs into crates/auto-ui-dialect/src/grammar/*.

Applies the frozen mapping from docs/plans/attachments/810/contract-baseline.md.
Compile errors after this pass are fixed by hand (type mismatches String/AutoStr,
residual borrow shapes).
"""
import re, sys, pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
PARSER = ROOT / "crates/auto-lang/src/parser.rs"
GRAMMAR = ROOT / "crates/auto-ui-dialect/src/grammar"


def strip_code(line):
    out = []
    i, n = 0, len(line)
    in_str = in_char = False
    while i < n:
        c = line[i]
        if in_str:
            if c == "\\": i += 2; continue
            if c == '"': in_str = False
            i += 1; continue
        if in_char:
            if c == "\\": i += 2; continue
            if c == "'": in_char = False
            i += 1; continue
        if c == '"': in_str = True; i += 1; continue
        if c == "'":
            j = i + 1
            if j < n and line[j] == "\\":
                j += 2
                if j < n and line[j] == "'":
                    i = j + 1; continue
            elif j < n and line[j] not in "'" and j + 1 < n and line[j+1] == "'":
                i = j + 2; continue
            i += 1; continue
        if c == "/" and i + 1 < n and line[i+1] == "/":
            break
        out.append(c); i += 1
    return "".join(out)

src = PARSER.read_text(encoding="utf-8")
lines = src.split("\n")

# ---- 1. split the impl region [parse_widget_decl .. extract_fstr end) into fn blocks ----
def find_line(pred, start=0):
    for i in range(start, len(lines)):
        if pred(lines[i]):
            return i
    raise SystemExit("not found")

region_start = find_line(lambda l: "pub fn parse_widget_decl(&mut self)" in l)
region_end = find_line(lambda l: l.startswith("#[cfg(test)]"))  # first test module after region

FN_RE = re.compile(r"^    (pub(?:\(crate\))? )?fn ([a-z_0-9]+)\(")

blocks = []  # (name, start, end) with doc/attrs attached above
i = region_start
while i < region_end:
    m = FN_RE.match(lines[i])
    if m:
        # attach preceding doc comments / attributes
        s = i
        while s - 1 >= region_start and (
            lines[s - 1].lstrip().startswith("///")
            or lines[s - 1].lstrip().startswith("//")
            or lines[s - 1].lstrip().startswith("#[")
        ) and not FN_RE.match(lines[s - 1]):
            s -= 1
        # brace-count to block end (start line included; braces on string contents
        # are not present because these fns build AST, no raw brace string literals
        # except capture sites which use lexer calls, not literals — verified by
        # counting matching final "    }" line)
        depth = 0
        seen = False
        j = i
        while j < len(lines):
            code = strip_code(lines[j])
            depth += code.count("{") - code.count("}")
            if code.count("{") > 0:
                seen = True
            if seen and depth == 0:
                break
            j += 1
        blocks.append((m.group(2), s, j))
        i = j + 1
    else:
        i += 1

names = [b[0] for b in blocks]
print("blocks:", len(blocks))

MIGRATE = {
    "helpers": [
        "parse_string_lit", "get_primary_prop", "extract_fstr_template_and_bindings",
        "fragment_param_hint_to_type", "modal_dialog_tag_role", "hover_card_role",
        "direct_state_value_field", "wire_modal_close_recursive",
    ],
    "widget": [
        "parse_widget_decl", "mint_inline_event_handlers", "mint_view_inline",
        "mint_events_inline", "mint_bare_input_sync", "mint_modal_dialog_toggle",
        "mint_modal_dialog_toggle_inner", "parse_widget_use_block_inner",
        "parse_watch_block_inner", "parse_setup_block_inner", "parse_expose_block_inner",
        "parse_style_block_inner", "parse_widget_props", "parse_computed_block_inner",
        "parse_on_block", "parse_bind_block",
    ],
    "store": ["parse_store_decl"],
    "scene": [
        "looks_like_scene_decl", "parse_scene_decl", "parse_scene_node",
        "parse_scene_instance", "parse_scene_connection", "parse_scene_signal",
        "parse_scene_prop", "parse_scene_value", "looks_like_subresource",
        "parse_scene_subresource",
    ],
    "view": [
        "parse_view_block", "parse_view_block_or_fragment", "parse_view_fragment_decl_body",
        "parse_component_fn_decl", "parse_fragment_decl_body_tail", "parse_view_block_inner",
        "parse_view_root_nodes", "parse_view_node", "parse_style_binding",
        "parse_view_for_loop", "parse_view_key_expr", "parse_view_memo_block",
        "parse_view_conditional", "parse_view_link", "parse_condition_expr",
        "capture_condition_call_args", "parse_event_modifiers", "parse_event_value",
        "parse_event_handler", "parse_event_arg_list", "parse_event_arg",
        "parse_event_arg_object", "parse_handler_params",
    ],
    "actions": [
        "parse_routes_block_inner", "parse_actions_decl_stmt", "parse_style_recipe_decl_stmt",
        "parse_timer_block_inner", "parse_actions_block_inner", "parse_action_entry",
        "parse_actions_string_attr", "parse_actions_handler_attr", "parse_actions_cond_attr",
        "parse_menubar_block", "parse_menu_entry", "parse_toolbar_block",
        "parse_menu_item_entries",
    ],
}
KEEP_IN_PARSER = {"parse_msg_decl", "parse_model_block", "expect_ident"}
DELETE_DEAD = {"parse_msg_decl_inner", "parse_model_block_inner"}  # 806 inner：调用方随迁后死代码
PUB_ENTRIES = {
    "parse_widget_decl", "parse_store_decl", "parse_scene_decl", "parse_actions_decl_stmt",
    "parse_style_recipe_decl_stmt", "parse_view_block", "parse_view_block_or_fragment",
    "parse_component_fn_decl",
}
migrated = {n for v in MIGRATE.values() for n in v}
unknown = [n for n in names if n not in migrated and n not in KEEP_IN_PARSER and n not in DELETE_DEAD]
if unknown:
    print("UNCLASSIFIED blocks:", unknown); sys.exit(1)
missing = migrated - set(names)
if missing:
    print("MISSING blocks:", missing); sys.exit(1)

def block_text(name):
    for (n, s, j) in blocks:
        if n == name:
            return "\n".join(lines[s:j + 1])
    raise KeyError(name)

# ---- 2. text transformation ----
def transform(t, module):
    # -- special sites first (exact shapes from the baseline) --
    # widget near-miss lookahead
    t = t.replace(
"""                    let lex_state = self.lexer.save_state();
                    let saved_cur = self.cur.clone();
                    let saved_prev = self.prev.clone();
                    self.next();
                    let next_is_brace = self.is_kind(TokenKind::LBrace);
                    self.lexer.restore_state(lex_state);
                    self.cur = saved_cur;
                    self.prev = saved_prev;""",
"""                    let cp = cx.raw_cursor_save();
                    cx.advance();
                    let next_is_brace = cx.is_kind(TokenKind::LBrace);
                    cx.raw_cursor_restore(cp)?;""")
    # computed speculative object
    t = t.replace(
"""            let old_skip_check = self.skip_check;
            self.skip_check = true;
            let expr = if self.is_kind(TokenKind::LBrace) {
                let lex_state = self.lexer.save_state();
                let saved_cur = self.cur.clone();
                let saved_prev = self.prev.clone();
                let saved_errors_len = self.errors.len();
                let saved_warnings_len = self.warnings.len();
                match self.object() {
                    Ok(pairs) => Expr::Object(pairs),
                    Err(_) => {
                        self.lexer.restore_state(lex_state);
                        self.cur = saved_cur.clone();
                        self.prev = saved_prev.clone();
                        self.errors.truncate(saved_errors_len);
                        self.warnings.truncate(saved_warnings_len);
                        Expr::Block(self.body()?)
                    }
                }
            } else {
                self.parse_expr()?
            };
            self.skip_check = old_skip_check;""",
"""            let old_skip_check = cx.set_skip_check(true);
            let expr = if cx.is_kind(TokenKind::LBrace) {
                let cp = cx.raw_cursor_save();
                let diag_mark = cx.diagnostics_mark();
                match cx.parse_object_pairs() {
                    Ok(pairs) => Expr::Object(pairs),
                    Err(_) => {
                        cx.raw_cursor_restore(cp)?;
                        cx.diagnostics_truncate(diag_mark);
                        Expr::Block(cx.parse_body()?)
                    }
                }
            } else {
                cx.parse_expr()?
            };
            cx.set_skip_check(old_skip_check);""")
    # style recipe speculative head (two occurrences)
    t = t.replace(
"""        let saved_lexer = self.lexer.save_state();
        let saved_cur = self.cur.clone();
        let saved_prev = self.prev.clone();

        self.next(); // consume "style"
        if !self.is_kind(TokenKind::Ident) {
            self.lexer.restore_state(saved_lexer);
            self.cur = saved_cur;
            self.prev = saved_prev;
            return Ok(None);
        }""",
"""        let cp = cx.raw_cursor_save();

        cx.advance(); // consume "style"
        if !cx.is_kind(TokenKind::Ident) {
            cx.raw_cursor_restore(cp)?;
            return Ok(None);
        }""")
    t = t.replace(
"""        if !self.is_kind(TokenKind::Asn) && !self.is_kind(TokenKind::LParen) {
            self.lexer.restore_state(saved_lexer);
            self.cur = saved_cur;
            self.prev = saved_prev;
            return Ok(None);
        }""",
"""        if !cx.is_kind(TokenKind::Asn) && !cx.is_kind(TokenKind::LParen) {
            cx.raw_cursor_restore(cp)?;
            return Ok(None);
        }""")
    # style recipe skip_check wrap
    t = t.replace(
"""        let prev_skip = self.skip_check;
        self.skip_check = true;
        let body = self.parse_expr()?;
        self.skip_check = prev_skip;""",
"""        let prev_skip = cx.set_skip_check(true);
        let body = cx.parse_expr()?;
        cx.set_skip_check(prev_skip);""")
    # watch modifier peek
    t = t.replace(
"""                let modifier = if let Ok(tok) = self.lexer.next() {
                    let text = tok.text.to_string();
                    self.lexer.push_token(tok);
                    text
                } else {
                    String::new()
                };""",
"""                let modifier = match cx.probe_next_token().ok() {
                    Some(tok) => tok.text.to_string(),
                    None => String::new(),
                };""")
    # scene gate lookahead
    t = t.replace(
"""        let scene_tok = self.cur.clone();
        self.next(); // advance past 'scene'
        let is_name_next = self.cur.kind == TokenKind::Ident;
        // Restore both tokens (push in reverse consumption order: name first, scene last)
        self.lexer.push_token(self.cur.clone());
        self.lexer.push_token(scene_tok);
        self.next(); // restore cur to 'scene'
        is_name_next""",
"""        // PLAN-810: advance_peek_kind 具名化（旧臂双 token 回推+重同步形态）。
        cx.advance_peek_kind() == TokenKind::Ident""")
    # subresource lookahead
    t = t.replace(
"""        let ident_tok = self.cur.clone();
        self.next();
        let is_brace = self.is_kind(TokenKind::LBrace);
        // Restore: push the current token back, then the ident, then resync.
        self.lexer.push_token(self.cur.clone());
        self.lexer.push_token(ident_tok);
        self.next();
        is_brace""",
"""        // PLAN-810: advance_peek_kind 具名化（旧臂双 token 回推+重同步形态）。
        cx.advance_peek_kind() == TokenKind::LBrace""")
    # stream lookahead
    t = t.replace(
"""                let saved_cur = self.cur.clone();
                self.next();
                let next_is_kind_ident = self.cur.kind == TokenKind::Ident;
                self.lexer.push_token(self.cur.clone());
                self.cur = saved_cur;
                next_is_kind_ident""",
"""                // PLAN-810: advance_peek_kind 具名化（旧臂回推+手动恢复形态）。
                cx.advance_peek_kind() == TokenKind::Ident""")
    # bind_var multiline (params / model fields)
    t = t.replace(
"""            self.infer_ctx.bind_var(
                crate::ast::Name::from(pname.as_str()),
                crate::ast::Type::Unknown,
            );""",
"""            cx.bind_var(pname.as_str(), Type::Unknown);""")
    t = t.replace(
"""                self.infer_ctx.bind_var(
                    crate::ast::Name::from(field.name.as_str()),
                    crate::ast::Type::Unknown,
                );""",
"""                cx.bind_var(field.name.as_str(), Type::Unknown);""")
    t = t.replace(
"""        let var_name = crate::ast::Name::from(var.as_str());
        self.infer_ctx.bind_var(var_name, Type::Unknown);""",
"""        cx.bind_var(var.as_str(), Type::Unknown);""")
    t = t.replace(
"""            let idx_name = crate::ast::Name::from(idx.as_str());
            self.infer_ctx.bind_var(idx_name, Type::Int);""",
"""            cx.bind_var(idx.as_str(), Type::Int);""")
    # on-handler param define
    t = t.replace(
"""            for param in &params {
                let meta = Meta::Store(Store {
                    kind: StoreKind::Var,
                    attrs: vec![],
                    is_pub: false,
                    name: param.clone().into(),
                    expr: Expr::Nil,
                    ty: Type::Unknown,
                });
                self.define(param.as_str(), meta);
            }""",
"""            for param in &params {
                cx.define_handler_param(param.as_str());
            }""")
    # in_on_body wraps
    t = t.replace(
"""            let prev_on_body = self.in_on_body;
            self.in_on_body = true;""",
"""            let prev_on_body = cx.set_in_on_body(true);""")
    t = t.replace("            self.in_on_body = prev_on_body;",
                  "            cx.set_in_on_body(prev_on_body);")
    t = t.replace(
"""                let prev_on_body = self.in_on_body;
                self.in_on_body = true;""",
"""                let prev_on_body = cx.set_in_on_body(true);""")
    t = t.replace("                self.in_on_body = prev_on_body;",
                  "                cx.set_in_on_body(prev_on_body);")
    # warn near-miss
    t = t.replace(
"""                            self.warn(Warning::SuspiciousBlockKeyword {
                                found: ident.to_string(),
                                suggestion: kw.to_string(),
                                span: pos_to_span(self.cur.pos),
                            });""",
"""                            cx.warn_suspicious_block_keyword(
                                ident.to_string(),
                                kw.to_string(),
                            );""")
    # memo / paren_is_expr `?` peeks
    t = t.replace(
"""            let is_paren = {
                let tok = self.lexer.next()?;
                let is_lparen = tok.kind == TokenKind::LParen;
                self.lexer.push_token(tok);
                is_lparen
            };""",
"""            let is_paren = cx.probe_next_token()?.kind == TokenKind::LParen;""")
    t = t.replace(
"""            let paren_is_expr = {
                let tok = self.lexer.next()?;
                let is_dot = tok.kind == TokenKind::Dot;
                self.lexer.push_token(tok);
                is_dot
            };""",
"""            let paren_is_expr = cx.probe_next_token()?.kind == TokenKind::Dot;""")
    # has_primary_prop_value probe
    t = t.replace(
"""                if let Ok(next_token) = self.lexer.next() {
                    let is_field_or_call = next_token.kind == TokenKind::Dot
                        || next_token.kind == TokenKind::LParen
                        || next_token.kind == TokenKind::LSquare;
                    self.lexer.push_token(next_token);
                    is_field_or_call
                } else {
                    false
                }""",
"""                if let Ok(next_token) = cx.probe_next_token() {
                    let is_field_or_call = next_token.kind == TokenKind::Dot
                        || next_token.kind == TokenKind::LParen
                        || next_token.kind == TokenKind::LSquare;
                    is_field_or_call
                } else {
                    false
                }""")
    # concat chain probe
    t = t.replace(
"""                    if let Ok(next_token) = self.lexer.next() {
                        let is_concat = next_token.kind == TokenKind::Add;
                        self.lexer.push_token(next_token);
                        is_concat
                    } else {
                        false
                    }""",
"""                    if let Ok(next_token) = cx.probe_next_token() {
                        let is_concat = next_token.kind == TokenKind::Add;
                        is_concat
                    } else {
                        false
                    }""")
    # prop-like probe (checks cur text AND next kind)
    t = t.replace(
"""                    if let Ok(next_token) = self.lexer.next() {
                        let is_colon = next_token.kind == TokenKind::Colon;""",
"""                    if let Ok(next_token) = cx.probe_next_token() {
                        let is_colon = next_token.kind == TokenKind::Colon;""")
    # register_style_recipe side effect
    t = t.replace("        crate::design_tokens::recipe::register_style_recipe(&decl);",
                  "        cx.register_style_recipe(&decl);")
    # 806 single-source declarations
    t = t.replace("self.parse_msg_decl_inner()?", "crate::declarations::parse_msg_decl(cx)?")
    t = t.replace("self.parse_model_block_inner()?", "crate::declarations::parse_model_block(cx)?")
    t = t.replace("Some(self.parse_model_block_inner()?)", "Some(crate::declarations::parse_model_block(cx)?)")

    # -- mechanical cursor/port mapping --
    for old, new in [
        ("self.expect_ident(", "cx.expect_ident("),
        ("self.cur_is_soft_ident()", "cx.cur_is_soft_ident()"),
        ("self.is_kind(", "cx.is_kind("),
        ("self.skip_empty_lines()", "cx.skip_empty_lines()"),
        ("self.expect(", "cx.expect("),
        ("self.next()", "cx.advance()"),
        ("self.cur.text.as_str()", "cx.cur_text().as_str()"),
        ("self.cur.text.clone()", "cx.cur_text()"),
        ("self.cur.text", "cx.cur_text()"),
        ("self.cur.pos", "cx.cur_pos()"),
        ("self.cur.kind", "cx.cur_kind()"),
        ("self.prev.pos", "cx.prev_pos()"),
        ("self.parse_expr()", "cx.parse_expr()"),
        ("self.parse_type()", "cx.parse_type()"),
        ("self.body()", "cx.parse_body()"),
        ("self.parse_stmt()", "cx.parse_stmt()"),
        ("self.parse_closure()", "cx.parse_closure()"),
        ("self.object()", "cx.parse_object_pairs()"),
        ("self.fstr()", "cx.parse_fstr()"),
        ("self.dot_item(", "cx.parse_dot_item("),
        ("self.enter_scope()", "cx.enter_scope()"),
        ("self.exit_scope()", "cx.exit_scope()"),
        ("self.take_docs()", "cx.take_docs()"),
        ("self.lexer.capture_raw_block()?", "cx.capture_raw_block()?"),
        ("crate::ast::ui::", "ast::ui::"),
        ("crate::ast::", "ast::"),
    ]:
        t = t.replace(old, new)
    # Self:: assoc-fn calls -> free fn calls
    for fname in ["mint_view_inline", "mint_events_inline", "mint_bare_input_sync",
                  "mint_modal_dialog_toggle_inner", "mint_modal_dialog_toggle",
                  "get_primary_prop", "fragment_param_hint_to_type",
                  "modal_dialog_tag_role", "hover_card_role",
                  "direct_state_value_field", "wire_modal_close_recursive"]:
        t = t.replace(f"Self::{fname}(", f"{fname}(")
    # method signature: self -> cx
    t = re.sub(r"^    (pub(?:\(crate\))? )?fn ([a-z_0-9]+)\(&mut self\)",
               lambda m: f"    {'pub ' if m.group(2) in PUB_ENTRIES else 'pub(crate) '}fn {m.group(2)}(cx: &mut dyn UiParseContext)",
               t, flags=re.M)
    return t

# ---- 3. write grammar module files ----
HEADER = {
    "helpers": """//! 纯 helper（无游标）：自 auto-lang Parser 迁出的 UI 文法静态工具
//! （PLAN-810 T-03/T-04 逐字迁移；端口映射见 contract-baseline §2）。

use auto_parser_api::ast::{Expr, Name, Type, ViewNode, ViewProp};

""",
    "widget": """//! widget 声明文法（PLAN-810 T-03 迁移）：widget 壳/props/use/setup/
//! expose/watch/computed/on/bind 块与内联 handler mint 族。

use super::helpers::{
    damerau_levenshtein_1, direct_state_value_field, get_primary_prop,
    hover_card_role, modal_dialog_tag_role, on_handler_event_name,
    stmt_expr_contains_await, wire_modal_close_recursive,
};
use super::actions::{
    parse_actions_block_inner, parse_routes_block_inner, parse_timer_block_inner,
};
use super::view::{parse_view_node, parse_view_root_nodes};
use crate::UiParseContext;
use auto_parser_api::ast::*;
use auto_parser_api::error::{pos_to_span, AutoResult, SyntaxError, Warning};
use auto_parser_api::token::TokenKind;

""",
    "store": """//! store 声明文法（PLAN-810 T-03 迁移）。msg/model 声明复用 806
//! declarations 单源（经端口直调，不复制第二份算法）。

use super::widget::{parse_computed_block_inner, parse_on_block, parse_watch_block_inner};
use super::actions::parse_timer_block_inner;
use crate::UiParseContext;
use auto_parser_api::ast::*;
use auto_parser_api::error::{pos_to_span, AutoResult, SyntaxError};
use auto_parser_api::token::TokenKind;

""",
    "scene": """//! Godot scene 声明文法（Plan 306；PLAN-810 T-03 迁移。门在任何
//! scenario 生效，Core 经 Parser 薄委托调用）。

use super::helpers::parse_string_lit;
use crate::UiParseContext;
use auto_parser_api::ast::*;
use auto_parser_api::error::{pos_to_span, AutoResult, SyntaxError};
use auto_parser_api::token::TokenKind;

""",
    "view": """//! view/fragment/component/节点/for/if/key/memo/link/style 绑定与
//! UI 事件文法（PLAN-810 T-04 迁移）。

use super::helpers::{
    extract_fstr_template_and_bindings, fragment_param_hint_to_type, get_primary_prop,
};
use super::actions::parse_actions_string_attr;
use super::widget::{
    parse_computed_block_inner, parse_on_block, parse_setup_block_inner,
    parse_style_block_inner, parse_watch_block_inner, parse_widget_use_block_inner,
};
use crate::UiParseContext;
use auto_parser_api::ast::*;
use auto_parser_api::error::{pos_to_span, AutoResult, SyntaxError};
use auto_parser_api::token::TokenKind;

""",
    "actions": """//! actions/timer/routes/menubar/toolbar 与顶层 style recipe 声明文法
//! （PLAN-810 T-03/T-04 迁移）。

use super::helpers::on_handler_event_name;
use crate::UiParseContext;
use auto_parser_api::ast::*;
use auto_parser_api::error::{pos_to_span, AutoResult, SyntaxError};
use auto_parser_api::token::TokenKind;

""",
}

for module, fnlist in MIGRATE.items():
    body = "\n\n".join(transform(block_text(n), module) for n in fnlist)
    (GRAMMAR / f"{module}.rs").write_text(HEADER[module] + body + "\n", encoding="utf-8")
    print("wrote", module, len(fnlist), "fns")

# free functions from parser.rs head -> helpers.rs
def extract_free_fn(text, name):
    start = text.index(f'fn {name}(')
    ls = text[:start].split(chr(10))
    k = len(ls) - 1
    while k > 0 and (ls[k - 1].lstrip().startswith('///') or ls[k - 1].lstrip().startswith('//')):
        k -= 1
    begin = len(chr(10).join(ls[:k]))
    depth = 0
    i = start
    while True:
        nl = text.index(chr(10), i)
        code = strip_code(text[i:nl])
        depth += code.count('{') - code.count('}')
        i = nl + 1
        if depth == 0:
            break
    return text[begin:i]

free_fns = chr(10) + chr(10).join([extract_free_fn(src, n) for n in
                       ["on_handler_event_name", "damerau_levenshtein_1", "stmt_expr_contains_await"]])
with (GRAMMAR / "helpers.rs").open("a", encoding="utf-8") as f:
    f.write("\n" + free_fns + "\n")
print("free fns appended")

# ---- 4. rewrite parser.rs: drop migrated blocks, keep rest + thin delegates ----
drop_ranges = []
for (n, s, j) in blocks:
    if n in migrated or n in DELETE_DEAD:
        drop_ranges.append((s, j))
keep = []
for idx, line in enumerate(lines):
    if any(s <= idx <= j for (s, j) in drop_ranges):
        continue
    keep.append((idx, line))

# thin delegates inserted where parse_widget_decl used to start
DELEGATES = """    /// PLAN-810：以下公开 parse_* 为薄委托门面——完整 UI 文法已迁
    /// auto-ui-dialect::grammar（单一份实现），经 ParserBridge 驱动。
    pub fn parse_widget_decl(&mut self) -> AutoResult<Stmt> {
        let mut bridge = crate::parser_bridge::ParserBridge::new(self);
        auto_ui_dialect::grammar::widget::parse_widget_decl(&mut bridge)
    }

    pub fn parse_store_decl(&mut self) -> AutoResult<Stmt> {
        let mut bridge = crate::parser_bridge::ParserBridge::new(self);
        auto_ui_dialect::grammar::store::parse_store_decl(&mut bridge)
    }

    pub fn parse_scene_decl(&mut self) -> AutoResult<Stmt> {
        let mut bridge = crate::parser_bridge::ParserBridge::new(self);
        auto_ui_dialect::grammar::scene::parse_scene_decl(&mut bridge)
    }

    pub fn parse_actions_decl_stmt(&mut self) -> AutoResult<Option<Stmt>> {
        let mut bridge = crate::parser_bridge::ParserBridge::new(self);
        auto_ui_dialect::grammar::actions::parse_actions_decl_stmt(&mut bridge)
    }

    pub fn parse_style_recipe_decl_stmt(&mut self) -> AutoResult<Option<Stmt>> {
        let mut bridge = crate::parser_bridge::ParserBridge::new(self);
        auto_ui_dialect::grammar::actions::parse_style_recipe_decl_stmt(&mut bridge)
    }

    pub fn parse_view_block(&mut self) -> AutoResult<Stmt> {
        let mut bridge = crate::parser_bridge::ParserBridge::new(self);
        auto_ui_dialect::grammar::view::parse_view_block(&mut bridge)
    }

    pub fn parse_view_block_or_fragment(&mut self) -> AutoResult<Stmt> {
        let mut bridge = crate::parser_bridge::ParserBridge::new(self);
        auto_ui_dialect::grammar::view::parse_view_block_or_fragment(&mut bridge)
    }

    pub fn parse_component_fn_decl(&mut self) -> AutoResult<Stmt> {
        let mut bridge = crate::parser_bridge::ParserBridge::new(self);
        auto_ui_dialect::grammar::view::parse_component_fn_decl(&mut bridge)
    }

    /// scene 门（Plan 306；任意 scenario）——文法侧 lookahead 经端口。
    fn looks_like_scene_decl(&mut self) -> bool {
        let mut bridge = crate::parser_bridge::ParserBridge::new(self);
        auto_ui_dialect::grammar::scene::looks_like_scene_decl(&mut bridge)
    }
"""

out = []
inserted = False
region_start_line = drop_ranges[0][0]
for (idx, line) in keep:
    if idx == region_start_line and not inserted:
        out.append(DELEGATES)
        inserted = True
    out.append(line)
new_src = "\n".join(out)

# drop the three free fns from parser.rs (migrated to helpers)
for name in ["on_handler_event_name", "damerau_levenshtein_1", "stmt_expr_contains_await"]:
    frag = extract_free_fn(new_src, name)
    new_src = new_src.replace(frag + "\n", "", 1)

PARSER.write_text(new_src, encoding="utf-8", newline="")
print("parser.rs rewritten")
