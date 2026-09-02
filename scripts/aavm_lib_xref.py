#!/usr/bin/env python3
"""AAVM lib 七文件交叉引用分析 → 依赖树 + pub 导出面清单（docs/specs/aavm/design/lib-modularization-map.md 的生成器）。

方法:提取各文件顶层定义(fn/type/enum,行首无缩进),建立 符号→定义文件 映射;
再在其余文件中做词边界引用扫描(剔除行注释),得 出引用矩阵。
输出:每文件的跨文件引用(按定义文件分组)+ 反向依赖边 + 环检测提示。
"""
import re
import pathlib
import collections
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent / "auto" / "lib"
FILES = ["token", "lexer", "parser", "typeinfo", "codegen", "engine", "a2r"]

DEF_RE = re.compile(r"^(?:pub )?(fn|type|enum)\s+([A-Za-z_]\w*)")
IDENT_RE = {}  # lazily compiled

defs = {}          # symbol -> (file, kind)
for f in FILES:
    for line in (ROOT / (f + ".at")).read_text(encoding="utf-8").splitlines():
        m = DEF_RE.match(line)
        if m:
            defs[m.group(2)] = (f, m.group(1))

dup = [s for s, c in collections.Counter(defs).items() if True]  # placeholder
print(f"# 顶层定义总数: {len(defs)}")
by_file = collections.Counter(v[0] for v in defs.values())
for f in FILES:
    print(f"#   {f}.at: {by_file[f]} 个定义")

def strip_comment(line: str) -> str:
    i = line.find("//")
    return line if i < 0 else line[:i]

xref = collections.defaultdict(lambda: collections.defaultdict(set))
for f in FILES:
    text_lines = [strip_comment(l) for l in (ROOT / (f + ".at")).read_text(encoding="utf-8").splitlines()]
    for name, (deffile, kind) in defs.items():
        if deffile == f:
            continue
        rx = re.compile(r"\b" + re.escape(name) + r"\b")
        if any(rx.search(l) for l in text_lines):
            xref[f][deffile].add(name)

print("\n## 依赖边(引用者 → 被引用者: 符号)")
edges = set()
for f in FILES:
    if f not in xref:
        continue
    for tgt, names in sorted(xref[f].items()):
        edges.add((tgt, f))  # tgt 被 f 依赖
        pretty = ", ".join(sorted(names))
        print(f"{f}.at → {tgt}.at ({len(names)}): {pretty}")

print("\n## 依赖树(拓扑序引用)")
order = {f: i for i, f in enumerate(FILES)}
bad = [(t, s) for (t, s) in edges if order[t] > order[s]]
print("反向边(疑似环,需人工核):", bad if bad else "无 —— 依赖与 AUTO_LIB_FILES_V2 序一致")

print("\n## 各文件需 pub 的导出面(被 ≥1 个其他文件引用的符号)")
for f in FILES:
    out = set()
    for g in FILES:
        if g != f:
            out |= xref[g].get(f, set())
    if out:
        print(f"{f}.at pub({len(out)}): {', '.join(sorted(out))}")
    else:
        print(f"{f}.at pub: (无跨文件引用)")
