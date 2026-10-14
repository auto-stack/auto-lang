#!/usr/bin/env python3
"""PLAN-780 T-01: managed pac.at corpus survey.

Classifies every git-managed pac.at in this repository by the constructs it
uses, into:

  static   - only literal flat props / literal dep+target nodes / literal
             arrays / objects (comments allowed); nothing requiring the
             config VM (no `${}` interpolation, no bare variable references,
             no assignments, no calls)
  dynamic  - contains at least one construct that only the config VM can
             produce (interpolation, bare identifier value outside the
             builtin set, `x = ...` assignment)
  unknown  - constructs the survey scanner cannot decide (recorded, never
             silently dropped)

The scanner is a small state machine (string/comment aware). It is a survey
oracle for PLAN-780 T-01 evidence, NOT a parser: the frozen static subset and
the per-field classification contract live in crates/auto-project-model.

Usage:
  python pac_corpus_survey.py [--root REPO_ROOT] [--json OUT] [--md OUT]
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

# Bare identifier values that the config VM defines as builtins
# (eval_config_with_vm injects on/off; true/false/null/nil are literals).
BUILTIN_IDENTS = {"true", "false", "null", "nil", "on", "off"}

# Top-level prop keys known to be consumed by production readers
# (Pac::new / parse_pac_fields / parse_pac_back_project / vue parse_pac_* /
#  dep gate pac_declares_dep / pac_workspace_member_dir). Unknown keys are
# catalogued, not dropped.
KNOWN_PROP_KEYS = {
    "name", "version", "description", "lang", "backend", "render", "api",
    "scene", "members", "defines", "front_port", "back_port", "media_root",
    "photo_root", "window", "title", "title_zh", "icon", "category", "theme",
    "accent", "desktop_render", "ui_config", "back", "shadcn",
    "default_classes", "entry", "exe_name", "opens", "daemon", "desktop",
    "desktop_exe", "port", "ports", "workspace",
}

# Node kinds understood by TargetKind / Pac::new (targets and deps); unknown
# node names are catalogued as unknown_nodes.
KNOWN_NODE_NAMES = {
    "dep", "app", "port", "device", "lib", "bin", "test", "tool",
}


def strip_comments_and_scan(text: str) -> dict:
    """Single pass: strip comments, track strings, record indicators.

    Returns dict with:
      clean      - text with comments blanked (strings preserved)
      dynamic    - list of (line, kind, detail) dynamic indicators
      unknown    - list of (line, kind, detail) undecidable constructs
    """
    out = []
    dynamic = []
    unknown = []
    i = 0
    n = len(text)
    line = 1
    state = "code"  # code | line_comment | block_comment | dq | sq | bt | fdq
    while i < n:
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if state == "code":
            if c == "/" and nxt == "/":
                state = "line_comment"
                out.append("  ")
                i += 2
                continue
            if c == "/" and nxt == "*":
                state = "block_comment"
                out.append("  ")
                i += 2
                continue
            if c == '"':
                state = "dq"
                out.append(c)
                i += 1
                continue
            if c == "'":
                state = "sq"
                out.append(c)
                i += 1
                continue
            if c == "`":
                state = "bt"
                dynamic.append((line, "fstring-backtick", "backtick f-string"))
                out.append(c)
                i += 1
                continue
            if c == "#" and nxt == "[":
                # attribute - not valid pac.at; record unknown
                unknown.append((line, "attribute", "#[..."))
                out.append(c)
                i += 1
                continue
            out.append(c)
            if c == "\n":
                line += 1
            i += 1
            continue
        if state == "line_comment":
            if c == "\n":
                state = "code"
                out.append("\n")
                line += 1
            else:
                out.append(" ")
            i += 1
            continue
        if state == "block_comment":
            if c == "*" and nxt == "/":
                state = "code"
                out.append("  ")
                i += 2
                continue
            if c == "\n":
                out.append("\n")
                line += 1
            else:
                out.append(" ")
            i += 1
            continue
        # string states
        if c == "\\" and i + 1 < n and state in ("dq", "sq"):
            out.append(text[i : i + 2])
            i += 2
            continue
        if c == "$" and state in ("dq", "bt"):
            # ${expr} or $ident interpolation inside string/f-string
            if nxt == "{":
                dynamic.append((line, "interpolation", "${...}"))
            elif nxt.isalpha() or nxt == "_":
                dynamic.append((line, "interpolation", "$ident"))
            out.append(c)
            i += 1
            continue
        closing = {"dq": '"', "sq": "'", "bt": "`"}[state]
        if c == closing:
            state = "code"
        out.append(c)
        if c == "\n":
            line += 1
        i += 1
    return {
        "clean": "".join(out),
        "dynamic": dynamic,
        "unknown": unknown,
    }


def scan_file(path: Path, rel: str) -> dict:
    text = path.read_text(encoding="utf-8", errors="replace")
    s = strip_comments_and_scan(text)
    clean = s["clean"]

    props = []          # (key, value_text)
    nodes = []          # (name, form) form in bare/quoted/arg
    assignments = []
    bare_ident_values = []

    for ln in clean.splitlines():
        stripped = ln.strip()
        if not stripped:
            continue
        # assignment form: ident = value  (config VM supports `x = ...`)
        if "=" in stripped and not stripped.startswith(("==", "!=")):
            head = stripped.split("=", 1)[0].strip()
            if head and (head[0].isalpha() or head[0] == "_") and all(
                ch.isalnum() or ch in "_-." for ch in head
            ):
                # only a real assignment statement when head is a bare ident
                # and the line is not `key: value` or `key ==` compare
                if ":" not in head:
                    assignments.append((head, stripped[:60]))

        # node form: ident (ident|"ident"|'ident'|...)? (...) {  }
        # detect: ^word word...{  or ^word("..."){  or ^word(...)
        import re

        m = re.match(
            r"^([A-Za-z_][\w-]*)\s+([A-Za-z_][\w-]*|\"[^\"]*\"|'[^']*')\s*(\(|\{)",
            stripped,
        )
        if m and m.group(1) not in ("dep",):
            # generic node: name sub {...} / name(...)  -- e.g. `app main {`
            nodes.append((m.group(1), "bare" if not m.group(3) == "(" else "arg"))
        m = re.match(
            r"^dep\s+([A-Za-z_][\w-]*|\"[^\"]*\"|'[^']*')\s*(\(|\{)",
            stripped,
        )
        if m:
            name = m.group(1)
            form = "bare" if name[0].isalpha() or name[0] == "_" else "quoted"
            if m.group(2) == "(":
                form = "arg"
            nodes.append(("dep", form))
        m = re.match(r"^dep\s*\(", stripped)
        if m:
            nodes.append(("dep", "arg"))

        # prop form: key: value  (first ':' outside strings; survey-level:
        # simple split, values with ':' inside quotes are rare)
        m = re.match(r"^([A-Za-z_][\w-]*)\s*:\s*(.*)$", stripped)
        if m:
            key = m.group(1)
            value = m.group(2).strip().rstrip(",").strip()
            props.append((key, value))
            # bare identifier value = variable reference unless builtin
            if value and (value[0].isalpha() or value[0] == "_"):
                tok = re.match(r"^([A-Za-z_][\w-]*)", value)
                if tok and tok.group(1) not in BUILTIN_IDENTS:
                    bare_ident_values.append((key, tok.group(1)))
            # call-in-value: ident(...) at value start
            call = re.match(r"^[A-Za-z_][\w-]*\s*\(", value)
            if call:
                bare_ident_values.append((key, value[:30]))

    indicators = list(s["dynamic"])
    for h, txt in assignments:
        indicators.append((0, "assignment", f"{h} = ..."))
    for key, tok in bare_ident_values:
        indicators.append((0, "bare-ident-value", f"{key}: {tok}"))

    if indicators:
        classification = "dynamic"
    elif s["unknown"]:
        classification = "unknown"
    else:
        classification = "static"

    prop_keys = sorted({k for k, _ in props})
    unknown_props = [k for k in prop_keys if k not in KNOWN_PROP_KEYS]
    unknown_nodes = sorted({nm for nm, _ in nodes if nm not in KNOWN_NODE_NAMES})
    dep_forms = sorted({form for nm, form in nodes if nm == "dep"})

    return {
        "path": rel,
        "classification": classification,
        "indicators": [
            {"kind": kind, "detail": detail, "line": ln}
            for ln, kind, detail in indicators
        ],
        "scan_unknown": [f"{kind}:{detail}" for kind, detail in s["unknown"]],
        "prop_keys": prop_keys,
        "unknown_props": unknown_props,
        "node_forms": sorted(set(nodes)),
        "dep_forms": dep_forms,
        "unknown_nodes": unknown_nodes,
        "has_members_array": any(k == "members" for k, _ in props),
        "multiline_members": "members" in text and re.search(
            r"members\s*:\s*\[[^\]]*\]", text, re.S
        )
        is None,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=None)
    ap.add_argument("--json", default=None)
    ap.add_argument("--md", default=None)
    args = ap.parse_args()

    root = Path(args.root).resolve() if args.root else Path(__file__).resolve().parents[5]
    files = subprocess.run(
        ["git", "ls-files", "*.at"], cwd=root, capture_output=True, text=True, check=True
    ).stdout.splitlines()
    pac_files = [f for f in files if f.replace("\\", "/").endswith("pac.at")]
    pac_files.sort()

    results = [scan_file(root / p, p.replace("\\", "/")) for p in pac_files]

    by_class: dict[str, list] = {"static": [], "dynamic": [], "unknown": []}
    for r in results:
        by_class[r["classification"]].append(r["path"])

    all_prop_keys = sorted({k for r in results for k in r["prop_keys"]})
    all_unknown_props = sorted({k for r in results for k in r["unknown_props"]})
    all_dep_forms = sorted({f for r in results for f in r["dep_forms"]})
    all_unknown_nodes = sorted({n for r in results for n in r["unknown_nodes"]})
    indicator_kinds = sorted(
        {ind["kind"] for r in results for ind in r["indicators"]}
    )

    summary = {
        "total": len(results),
        "counts": {k: len(v) for k, v in by_class.items()},
        "by_class": by_class,
        "all_prop_keys": all_prop_keys,
        "unknown_prop_keys": all_unknown_props,
        "dep_forms": all_dep_forms,
        "unknown_node_names": all_unknown_nodes,
        "dynamic_indicator_kinds": indicator_kinds,
        "members_multiline_files": [
            r["path"] for r in results if r["has_members_array"] and r["multiline_members"]
        ],
    }

    out = {"summary": summary, "files": results}
    if args.json:
        Path(args.json).write_text(
            json.dumps(out, ensure_ascii=False, indent=1), encoding="utf-8"
        )
    md = render_md(out)
    if args.md:
        Path(args.md).write_text(md, encoding="utf-8")
    print(md)
    return 0


def render_md(out: dict) -> str:
    s = out["summary"]
    lines = [
        "# PLAN-780 T-01 pac.at corpus survey",
        "",
        f"- total managed pac.at: **{s['total']}**",
        f"- static: **{s['counts']['static']}**",
        f"- dynamic: **{s['counts']['dynamic']}**",
        f"- unknown: **{s['counts']['unknown']}**",
        f"- dep forms: {s['dep_forms']}",
        f"- unknown node names: {s['unknown_node_names'] or 'none'}",
        f"- dynamic indicator kinds: {s['dynamic_indicator_kinds']}",
        "",
        "## unknown prop keys (not in KNOWN_PROP_KEYS readers map)",
    ]
    lines += [f"- {k}" for k in s["unknown_prop_keys"]] or ["- none"]
    lines += ["", "## files by classification"]
    for cls in ("dynamic", "unknown"):
        lines.append(f"### {cls} ({s['counts'][cls]})")
        for p in s["by_class"][cls]:
            inds = [i for f in out["files"] if f["path"] == p for i in f["indicators"]]
            det = "; ".join(sorted({f'{i["kind"]}:{i["detail"]}' for i in inds}))[:120]
            lines.append(f"- {p} — {det}")
    lines.append(f"### static ({s['counts']['static']})")
    lines += [f"- {p}" for p in s["by_class"]["static"]]
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    sys.exit(main())
