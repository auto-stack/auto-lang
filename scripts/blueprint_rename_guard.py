#!/usr/bin/env python3
"""blueprint_rename_guard.py —— PLAN-639 重命名完整性 grep 门禁。

守卫目标：Block 层更名（PLAN-639）的**复活模式**——旧标识/旧路径/旧 CLI 面
在任何活跃代码/文档中重新出现即 fail。逐词封禁 "block" 一词不可行（AST
Stmt::Block、CSS display:block、`store {} block` 等同类同形词属豁免语义，
见 manifest §2.4），故本门禁锚定**精确复活面**。

用法:
  python scripts/blueprint_rename_guard.py              # 全量扫描（exit 0=clean）
  python scripts/blueprint_rename_guard.py --changed    # 只扫 git 变更文件（CI 快档）

口径依据：docs/plans/attachments/639-rename-manifest.md §2.4/§2.5。
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# 活跃层路径集（archive/历史归档不在内）
SCAN_PATHS = [
    "crates",
    "blueprints",
    "examples",
    "docs/design",
    "docs/specs",
    "scripts",
    "website",
]

# 路径豁免（历史归档/生成物/另一概念/构建缓存）
PATH_EXEMPT_SUBSTR = [
    "blueprint_rename_guard.py",  # 门禁自身（模式字面量自匹配）
    "/target/",
    "/node_modules/",
    "/archive/",
    "_archive",
    "autodown_blocks.rs",       # C 类：autodown 文档块家族（PLAN-041），另一概念
    "/deps/",                   # 构建期依赖物化（gitignore 面）
    "public/ui/blocks",         # D8 豁免：ui-gallery 构建产物路由（PLAN-070）
    "021-block-static",         # fixture id（EDGE-16 载体）
]

# 复活模式：PLAN-639 更名前的旧标识/旧路径/旧 CLI 面
FORBIDDEN = [
    # UI 层旧类型名（BlockRegistry/BlockPackage/BlockSpec/BlockAction）
    (re.compile(r"\bBlock(Registry|Package|Spec|Action)\b"), None),
    # 旧 CLI 模块面
    (re.compile(r"\bmod cmd_block\b"), None),
    (re.compile(r"\bcmd_block::"), None),
    (re.compile(r"use crate::cmd_block\b"), None),
    # 旧 ui_gen 模块路径
    (re.compile(r"ui_gen::block\b"), None),
    (re.compile(r"ui_gen/block\b"), None),
    (re.compile(r"pub mod block;"), None),
    # 旧包库/示例/落地路径
    (re.compile(r"blocks-gallery"), None),
    (re.compile(r'join\("blocks"\)'), None),
    (re.compile(r"src/front/blocks\b"), None),
    (re.compile(r"\bblocks/(form|data-display|editor|navigation|README)"), None),
    (re.compile(r"examples/blocks\b"), None),
    (re.compile(r"docs/design/blocks\b"), None),
    (re.compile(r"docs/specs/blocks\b"), None),
    # CLI 旧主命令面（`auto block xxx` 用法演示；弃用提示行豁免）
    (re.compile(r"`auto block "), "deprecated"),
    (re.compile(r"^\s*auto block "), "deprecated"),
]

# 行级豁免：弃用提示（cmd_bp.rs/main.rs 各一处，属保留面）；
# 历史注记（Design 17 归位注记/更名注记——保留历史语境，计划 §1 非目标）
LINE_ALLOW = re.compile(r"deprecated|归位注记|更名注记|随之", re.IGNORECASE)

SCAN_SUFFIX = {".rs", ".md", ".ts", ".vue", ".json", ".toml", ".py", ".css", ".html", ".at", ".yml", ".yaml", ".mjs", ".js"}


def iter_files(changed_only: bool):
    if changed_only:
        out = subprocess.run(
            ["git", "diff", "--name-only", "HEAD"],
            capture_output=True, text=True, cwd=ROOT,
        ).stdout.splitlines()
        for line in out:
            yield ROOT / line
        return
    # 仅扫 git 跟踪文件——website/zh/docs 等生成镜像/构建缓存不入库，
    # 逐 rglob 会把主检出的未跟踪产物误报（8e281d825 实勘）。
    for base in SCAN_PATHS:
        p = ROOT / base
        if not p.exists():
            print(f"warn: scan path missing: {base}")
            continue
        out = subprocess.run(
            ["git", "ls-files", base],
            capture_output=True, text=True, cwd=ROOT,
        ).stdout.splitlines()
        for line in out:
            yield ROOT / line


def check_file(f: Path) -> list[str]:
    s = str(f).replace("\\", "/")
    if any(x in s for x in PATH_EXEMPT_SUBSTR):
        return []
    if f.suffix not in SCAN_SUFFIX:
        return []
    try:
        text = f.read_text(encoding="utf-8")
    except (UnicodeDecodeError, OSError):
        return []
    hits = []
    for i, line in enumerate(text.splitlines(), 1):
        for rx, allow_tag in FORBIDDEN:
            if rx.search(line):
                # 行级豁免（deprecated 提示/历史注记）适用于全部模式
                if LINE_ALLOW.search(line):
                    continue
                hits.append(f"{s}:{i}: [{rx.pattern}] {line.strip()[:120]}")
    return hits


def main() -> int:
    changed_only = "--changed" in sys.argv
    all_hits: list[str] = []
    for f in iter_files(changed_only):
        all_hits.extend(check_file(f))
    if all_hits:
        print(f"blueprint_rename_guard: {len(all_hits)} resurrection pattern hit(s)")
        for h in all_hits:
            print("  " + h)
        print("manifest: docs/plans/attachments/639-rename-manifest.md §2")
        return 1
    print("blueprint_rename_guard: clean")
    return 0


if __name__ == "__main__":
    sys.exit(main())
