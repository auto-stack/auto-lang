#!/usr/bin/env python3
"""PLAN-810 F810-R2-01: baseline identity evidence for the unterminated-fragment
unbounded-EOF quirk.

Proves the migrated fragment-path grammar text is the base-commit text under
the frozen port mapping: extract the relevant functions from base ab08955c2
parser.rs and from the migrated view.rs, normalize both with the exact
mechanical mapping used by the migration (self.X -> cx.X port renames,
speculative lexer blocks -> probe form, multiline bind_var collapse, signature
boilerplate / .into() / type-annotation stripping, whitespace unification), and
diff. Empty normalized diff = text identity for the quirk path; any behavioral
divergence could only come from the port layer (covered by plan810 tests).

Usage: python scripts/plan810_baseline_identity.py
"""

from __future__ import annotations

import argparse
import difflib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASE = "ab08955c241f6a824091c53d3b02894ca17e193b"

FN_NAMES = [
    "parse_view_block_or_fragment",
    "parse_view_fragment_decl_body",
    "parse_fragment_decl_body_tail",
    "parse_view_block_inner",
    "parse_view_root_nodes",
    "parse_view_node",
    "parse_view_conditional",
    "parse_view_for_loop",
    "parse_view_memo_block",
    "parse_view_key_expr",
]

RENAMES = [
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
    ("self.fstr()", "cx.parse_fstr()"),
    ("self.dot_item(", "cx.parse_dot_item("),
    ("self.enter_scope()", "cx.enter_scope()"),
    ("self.exit_scope()", "cx.exit_scope()"),
    ("self.take_docs()", "cx.take_docs()"),
    ("crate::ast::ui::", "ast::ui::"),
    ("crate::ast::", "ast::"),
    ("Self::get_primary_prop(", "get_primary_prop("),
    ("Self::fragment_param_hint_to_type(", "fragment_param_hint_to_type("),
    ("Self::extract_fstr_template_and_bindings(", "extract_fstr_template_and_bindings("),
    ("Self::mint_inline_event_handlers(", "mint_inline_event_handlers("),
    ("self.kind()", "cx.cur_kind()"),
    ("self.pos()", "cx.cur_pos()"),
]

CALLED_FNS = FN_NAMES + [
    "parse_condition_expr", "capture_condition_call_args", "parse_style_binding",
    "parse_view_link", "parse_event_modifiers", "parse_event_value",
    "parse_event_handler", "parse_event_arg_list", "parse_event_arg",
    "parse_event_arg_object", "parse_handler_params", "parse_computed_block_inner",
    "parse_on_block", "parse_msg_decl_inner", "parse_model_block_inner",
    "parse_string_lit", "parse_widget_props", "parse_widget_use_block_inner",
    "parse_setup_block_inner", "parse_watch_block_inner", "parse_expose_block_inner",
    "parse_style_block_inner", "parse_actions_block_inner", "parse_routes_block_inner",
    "parse_timer_block_inner", "parse_scene_decl", "looks_like_scene_decl",
    "parse_scene_node", "parse_scene_value",
]
for _n in CALLED_FNS:
    RENAMES.append((f"self.{_n}(", f"{_n}(cx"))
# `self.` method-call forms of pure helpers (not port calls)
RENAMES.append(("self.extract_fstr_template_and_bindings(", "extract_fstr_template_and_bindings("))
# glued first-arg forms produced by the rename above for multi-arg fns
RENAMES.append(("parse_event_modifiers(cxkey)", "parse_event_modifiers(cx,key)"))

COMMENT = re.compile(r"//[^\n]*")
ARG = r"((?:[^()]|\([^()]*\))*)"


def normalize(text):
    t = COMMENT.sub("", text)
    for a, b in RENAMES:
        t = t.replace(a, b)
    # speculative lexer block (optional brace wrapper; outer binding name may
    # differ from the inner `is_lparen`/`is_dot`, e.g. `let is_paren = { … }`)
    # -> probe form
    t = re.sub(
        r"let (\w+)\s*=\s*\{?\s*let tok\s*=\s*self\.lexer\.next\(\)\?;\s*"
        r"let is_(?:lparen|dot)\s*=\s*tok\.kind\s*==\s*(TokenKind::\w+);\s*"
        r"self\.lexer\.push_token\(tok\);\s*is_(?:lparen|dot)\s*\}?\s*;?",
        lambda m: "let " + m.group(1) + " = cx.probe_next_token()?.kind == "
        + m.group(2),
        t,
    )
    # generic if-let-Ok probe forms (frozen mapping): lexer.next() -> probe,
    # pushback line deleted (identical to the migrated shape)
    t = t.replace("self.lexer.next()", "cx.probe_next_token()")
    t = t.replace("self.lexer.push_token(next_token);", "")
    t = t.replace("self.lexer.push_token(tok);", "")
    # multiline bind_var(Name::from(x), T) -> cx.bind_var(x, T)
    t = re.sub(
        r"self\.infer_ctx\.bind_var\(\s*ast::Name::from\(" + ARG + r"\)\s*,\s*([^,()]+?)\s*,?\s*\)",
        lambda m: "cx.bind_var(" + m.group(1) + ", " + m.group(2) + ")",
        t,
    )
    # single-line `let n = Name::from(x); bind_var(n, T)` -> cx.bind_var(x, T)
    t = re.sub(
        r"let (?:var_name|idx_name) = ast::Name::from\(" + ARG + r"\);\s*"
        r"self\.infer_ctx\.bind_var\((?:var_name|idx_name), ([^,()]+)\)",
        lambda m: "cx.bind_var(" + m.group(1) + ", " + m.group(2) + ")",
        t,
    )
    t = t.replace("pub(crate) fn ", "fn ").replace("pub fn ", "fn ")
    t = t.replace("ast::Type::", "Type::").replace("ast::Name", "Name")
    # receiver signature boilerplate identical on both sides after rewrite
    t = re.sub(
        r"(fn [a-z_0-9]+)\s*\(\s*(?:&mut self|cx\s*:\s*&mut dyn UiParseContext)\s*\)",
        r"\1(SIG)",
        t,
    )
    # mechanical adaptations stripped on BOTH sides: .into() + type ascriptions
    t = t.replace(".into()", "")
    t = re.sub(
        r":\s*[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*(?:\s*<[^=;{}]*?>)?\s*(?==)",
        "",
        t,
    )
    # field shorthand un-expansion (`name: name` from the .into() adaptation)
    t = re.sub(r"\b([A-Za-z_]\w*)\s*:\s*\1\b", r"\1", t)
    # punctuation-adjacent whitespace (rustfmt placement noise) — symmetric;
    # semicolons dropped entirely (statement-separator placement only)
    t = t.replace(";", "")
    t = re.sub(r"\s*([=,:(){}])\s*", r"\1", t)
    t = re.sub(r"\s+", " ", t)
    return t.strip()


def extract_fn(text, name):
    # strip comments FIRST: brace counting must ignore `'{'`-in-comment noise
    # (e.g. "// Parse condition expression (until we hit '{')")
    text = COMMENT.sub("", text)
    m = re.search(rf"^ *(?:pub(?:\(crate\))? )?fn {name}\(", text, flags=re.M)
    if not m:
        return ""
    i = m.start()
    lines = text[:i].split("\n")
    k = len(lines) - 1
    while k > 0 and (lines[k - 1].lstrip().startswith(("///", "//", "#["))):
        k -= 1
    begin = len("\n".join(lines[:k]))
    depth, j, seen = 0, i, False
    while j < len(text):
        nl = text.find("\n", j)
        end = nl if nl != -1 else len(text)
        line = text[j:end]
        depth += line.count("{") - line.count("}")
        if line.count("{"):
            seen = True
        if seen and depth == 0:
            return text[begin:end + 1]
        j = end + 1
    return text[begin:]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="docs/plans/attachments/810/eof-quirk-baseline-identity.json")
    args = ap.parse_args()

    base_src = subprocess.run(
        ["git", "show", f"{BASE}:crates/auto-lang/src/parser.rs"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout
    head_src = (ROOT / "crates/auto-ui-dialect/src/grammar/view.rs").read_text(encoding="utf-8")

    report = {
        "probe": "plan810 fragment-path baseline identity (normalized)",
        "finding": "F810-R2-01",
        "base_commit": BASE,
        "head_commit": subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True,
        ).stdout.strip(),
        "functions": {},
    }
    all_ident = True
    for name in FN_NAMES:
        b = normalize(extract_fn(base_src, name))
        h = normalize(extract_fn(head_src, name))
        ident = b == h and b != ""
        entry = {"baseline_found": bool(b), "head_found": bool(h), "identical": ident}
        if not ident:
            all_ident = False
            sm = difflib.SequenceMatcher(None, b, h)
            entry["first_diffs"] = [
                {"op": tag, "baseline": b[i1:i2][:160], "head": h[j1:j2][:160]}
                for tag, i1, i2, j1, j2 in sm.get_opcodes() if tag != "equal"
            ][:4]
        report["functions"][name] = entry
    report["verdict"] = (
        "all fragment-path functions identical under the frozen port normalization"
        if all_ident else "DIFFERS - inspect first_diffs"
    )
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps({k: report[k] for k in ("verdict", "functions")}, ensure_ascii=False, indent=1))
    return 0 if all_ident else 1


if __name__ == "__main__":
    sys.exit(main())
