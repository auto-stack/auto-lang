#!/usr/bin/env python3
"""AAVM lib 七文件交叉引用分析 → 依赖树 + pub 导出面清单。

docs/specs/aavm/design/lib-modularization-map.md 的生成器（Plan 517 W0
方法化后升级版）：除顶层定义（fn/type/enum）外，提取 type 体方法
（Plan 514 γ4 方法化产物）——方法引用经 `.name(` 调用形态归属到定义
类型的文件，推导依赖边。可重跑校验 DAG。

用法:python scripts/aavm_lib_xref.py [--json]   （--json 追加机器可读边表）
"""
import re
import pathlib
import collections
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent / "auto" / "lib"
FILES = ["token", "lexer", "parser", "typeinfo", "codegen", "engine", "a2r"]

TOP_DEF_RE = re.compile(r"^(?:pub )?(fn|type|enum)\s+([A-Za-z_]\w*)")
METHOD_RE = re.compile(r"^    (?:pub )?(?:static )?fn\s+([A-Za-z_]\w*)")


def strip_comment(line: str) -> str:
    i = line.find("//")
    return line if i < 0 else line[:i]


defs = {}        # 顶层符号 -> (file, kind)
methods = collections.defaultdict(list)  # 方法名 -> [(type, file)] (可跨类型重名)

for f in FILES:
    in_type = None
    type_depth = 0
    depth = 0
    for raw in (ROOT / (f + ".at")).read_text(encoding="utf-8").splitlines():
        line = strip_comment(raw)
        if not line.strip():
            continue
        m = TOP_DEF_RE.match(line) if depth == 0 else None
        if m:
            defs[m.group(2)] = (f, m.group(1))
            if m.group(1) == "type":
                in_type = m.group(2)
                type_depth = depth
            else:
                in_type = None
        elif in_type:
            mm = METHOD_RE.match(line)
            if mm:
                methods[mm.group(1)].append((in_type, f))
        depth += line.count("{") - line.count("}")
        if in_type and depth <= type_depth:
            in_type = None

dup_methods = {k: v for k, v in methods.items() if len(v) > 1}
print(f"# 顶层定义: {len(defs)};type 体方法: {sum(len(v) for v in methods.values())}"
      f"（跨类型重名: {len(dup_methods)}"
      f"{' → ' + ', '.join(f'{k}:{[t for t, _ in v]}' for k, v in dup_methods.items()) if dup_methods else ''}）")

# ── 跨文件引用扫描 ──────────────────────────────────────────────
xref = collections.defaultdict(lambda: collections.defaultdict(set))   # 引用文件 → 定义文件 → 顶层符号
mxref = collections.defaultdict(lambda: collections.defaultdict(set))  # 引用文件 → 定义文件 → 方法名

for f in FILES:
    lines = [strip_comment(l) for l in (ROOT / (f + ".at")).read_text(encoding="utf-8").splitlines()]
    for name, (deffile, kind) in defs.items():
        if deffile == f:
            continue
        rx = re.compile(r"\b" + re.escape(name) + r"\b")
        if any(rx.search(l) for l in lines):
            xref[f][deffile].add(name)
    for mname, owners in methods.items():
        foreign = [(t, of) for t, of in owners if of != f]
        if not foreign:
            continue
        if len(owners) > 1:
            # 跨类型重名方法:按类型限定静态形态 TYPE.name( 归属
            # (构造族即此形态,原生 List.new() 天然不误匹配)
            for t, of in foreign:
                rx = re.compile(r"\b" + re.escape(t) + r"\." + re.escape(mname) + r"\(")
                if any(rx.search(l) for l in lines):
                    mxref[f][of].add(mname)
        else:
            rx = re.compile(r"\." + re.escape(mname) + r"\(")
            if any(rx.search(l) for l in lines):
                t, of = owners[0]
                mxref[f][of].add(mname)

print("\n## 依赖边(引用者 → 被引用者: 顶层符号 N + 方法 M)")
edges = set()
for f in FILES:
    tgts = set(xref.get(f, {})) | set(mxref.get(f, {}))
    for tgt in sorted(tgts):
        edges.add((tgt, f))
        tops = sorted(xref.get(f, {}).get(tgt, set()))
        mets = sorted(mxref.get(f, {}).get(tgt, set()))
        parts = []
        if tops:
            parts.append(f"顶层{len(tops)}: {', '.join(tops)}")
        if mets:
            parts.append(f"方法{len(mets)}: {', '.join(mets)}")
        print(f"{f}.at → {tgt}.at  [{'; '.join(parts)}]")

order = {f: i for i, f in enumerate(FILES)}
bad = sorted((t, s) for t, s in edges if order[t] > order[s])
print("\n## DAG 校验（对照 AUTO_LIB_FILES_V2 序）")
print("反向边(需处置):", bad if bad else "无 —— 依赖为 DAG")

print("\n## pub 导出面（模块级:类型+自由函数;方法随类型归属不需独立 pub）")
for f in FILES:
    out = set()
    for g in FILES:
        if g != f:
            out |= xref.get(g, {}).get(f, set())
    print(f"{f}.at pub({len(out)}): {', '.join(sorted(out)) if out else '(无跨文件引用)'}")

if "--json" in sys.argv:
    import json
    print("\n## EDGES-JSON")
    print(json.dumps(sorted([t, s] for t, s in edges)))
