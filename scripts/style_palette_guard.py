#!/usr/bin/env python3
"""style_palette_guard.py —— examples/ui demo「禁新增裸调色板色」grep 门禁（PLAN-637 D4）。

对清单（manifest）中已完成 rollout 的 demo 扫描 .at 源码：
  1. 调色板色字面量  <bg|text|border|ring>-<palette>-<n>[/<alpha>]
  2. 无色号字面量    <bg|text>-white[/<alpha>]        （r2 门禁模式扩容）

bg-black 系（遮罩/实黑）与渐变 from-/to-/via- 按 r2 裁定不入动态扫描，
在计划豁免表静态登记。豁免三类（对齐待澄清#1 裁定口径）：显式品牌色、
装饰性插画/渐变色、语法高亮色，逐条在 manifest exemptions 登记。

用法：
  python scripts/style_palette_guard.py                     # 按清单全量扫描
  python scripts/style_palette_guard.py --demo 013-todo     # 扫单个 demo
  python scripts/style_palette_guard.py --selftest          # 正反例自测

退出码：0 = 无非豁免命中；1 = 有命中或清单 demo 缺失；2 = 用法/环境错误。
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MANIFEST = REPO_ROOT / "scripts" / "style_palette_manifest.json"
DEFAULT_EXAMPLES = REPO_ROOT / "examples" / "ui"

PALETTES = (
    "red|blue|green|yellow|purple|pink|indigo|gray|slate|zinc|neutral|stone|"
    "amber|orange|lime|emerald|teal|cyan|sky|violet|fuchsia|rose"
)

# <bg|text|border|ring>-<palette>-<n>[/alpha]；渐变 from-/to-/via- 与 bg-black
# 不入扫描（r2：豁免表静态登记）。
RE_PALETTE = re.compile(
    rf"\b(bg|text|border|ring)-({PALETTES})-(\d{{2,3}})(/\d+)?\b"
)
# 无色号字面量（宿主角色映射：主按钮 text-white → text-primary-foreground、
# 卡片 bg-white → bg-card；r2 入扫描）。
RE_WHITE = re.compile(r"\b(bg|text)-white(/\d+)?\b")


def load_manifest(path: Path) -> dict:
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def scan_dir(demo_dir: Path, exemptions: dict[str, str]) -> list:
    hits = []
    for at_file in sorted(demo_dir.rglob("*.at")):
        # 依赖物化目录（junction/拷贝）不属 demo 源码面。
        if "deps" in at_file.relative_to(demo_dir).parts:
            continue
        for lineno, line in enumerate(
            at_file.read_text(encoding="utf-8").splitlines(), start=1
        ):
            stripped = line.strip()
            if stripped.startswith("//") or stripped.startswith("#"):
                continue  # 注释行不扫（映射表/豁免备案会引用旧字面量）
            for m in RE_PALETTE.finditer(line):
                literal = m.group(0)
                if literal not in exemptions:
                    hits.append((at_file, lineno, literal))
            for m in RE_WHITE.finditer(line):
                literal = m.group(0)
                if literal not in exemptions:
                    hits.append((at_file, lineno, literal))
    return hits


def run_guard(examples_dir: Path, manifest: dict, only_demo: str | None) -> int:
    completed = manifest.get("completed", [])
    exemptions_all = manifest.get("exemptions", {})
    targets = [only_demo] if only_demo else completed
    if only_demo and only_demo not in completed:
        print(f"[guard] {only_demo} 不在已完成清单（completed）内 —— 先登记再扫描")
        return 1
    total = 0
    for demo in targets:
        demo_dir = examples_dir / demo
        if not demo_dir.is_dir():
            print(f"[guard] 缺失 demo 目录: {demo_dir}")
            total += 1
            continue
        hits = scan_dir(demo_dir, exemptions_all.get(demo, {}))
        for f, lineno, literal in hits:
            print(f"[guard] {f.relative_to(examples_dir)}:{lineno}: {literal}")
        total += len(hits)
        print(f"[guard] {demo}: {'OK' if not hits else f'{len(hits)} hit(s)'}")
    if total:
        print(f"[guard] FAIL —— {total} 处非豁免裸色残留")
        return 1
    print(f"[guard] PASS —— {len(targets)} 个已完成 demo 零非豁免裸色")
    return 0


def selftest() -> int:
    """正反例自测：正例（纯 token/豁免）通过，反例（注入裸色）命中。"""
    with tempfile.TemporaryDirectory() as td:
        tmp = Path(td)
        good = tmp / "good-demo"
        bad = tmp / "bad-demo"
        good.mkdir()
        bad.mkdir()
        (good / "app.at").write_text(
            'widget App { view { text "hi" { style: "text-xs text-muted-foreground" } } }\n',
            encoding="utf-8",
        )
        (good / "swatch.at").write_text(
            'button { style: "w-5 h-5 rounded-full bg-indigo-500" }\n',  # 豁免命中
            encoding="utf-8",
        )
        (bad / "app.at").write_text(
            'text "x" { style: "px-4 bg-blue-500 text-white hover:bg-blue-600" }\n'
            'text "y" { style: "border border-gray-200" }\n',
            encoding="utf-8",
        )
        ok = True

        hits = scan_dir(good, {"bg-indigo-500": "accent swatch"})
        if hits:
            print(f"[selftest] FAIL 正例误报: {hits}")
            ok = False
        else:
            print("[selftest] PASS 正例（token + 豁免）零命中")

        hits = scan_dir(bad, {})
        literals = {h[2] for h in hits}
        expected = {"bg-blue-500", "text-white", "bg-blue-600", "border-gray-200"}
        if literals != expected:
            print(f"[selftest] FAIL 反例命中不符: {literals} != {expected}")
            ok = False
        else:
            print(f"[selftest] PASS 反例 4 处裸色全部命中: {sorted(literals)}")

        # 不扫面边界：渐变 from-/to-/via-、bg-black 系、text-black 不入扫描；
        # hover:<palette> 变体仍命中（底串即裸色）。
        edge = tmp / "edge-demo"
        edge.mkdir()
        (edge / "app.at").write_text(
            'col { style: "bg-gradient-to-r from-blue-500 to-purple-600" }\n'
            'col { style: "bg-black/40 text-black" }\n'
            'text "a" { style: "ring-2 ring-offset-2 ring-primary" }\n',
            encoding="utf-8",
        )
        hits = scan_dir(edge, {})
        if hits:
            print(f"[selftest] FAIL 边界误报（from-/to-/bg-black/text-black/ring-token 不应命中）: {hits}")
            ok = False
        else:
            print("[selftest] PASS 边界：from-/to-/bg-black/text-black 不扫，ring-token 不误报")

        hover = tmp / "hover-demo"
        hover.mkdir()
        (hover / "app.at").write_text(
            'button { style: "hover:bg-gray-300" }\n', encoding="utf-8"
        )
        hits = scan_dir(hover, {})
        if {h[2] for h in hits} != {"bg-gray-300"}:
            print(f"[selftest] FAIL hover 变体漏报: {hits}")
            ok = False
        else:
            print("[selftest] PASS hover:<palette> 变体命中（底串即裸色）")

        return 0 if ok else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--manifest", type=Path, default=DEFAULT_MANIFEST, help="清单 JSON 路径"
    )
    parser.add_argument(
        "--examples", type=Path, default=DEFAULT_EXAMPLES, help="examples/ui 根目录"
    )
    parser.add_argument("--demo", default=None, help="只扫这一个已完成 demo")
    parser.add_argument("--selftest", action="store_true", help="正反例自测")
    args = parser.parse_args()

    if args.selftest:
        return selftest()
    if not args.manifest.exists():
        print(f"[guard] 清单不存在: {args.manifest}")
        return 2
    manifest = load_manifest(args.manifest)
    return run_guard(args.examples, manifest, args.demo)


if __name__ == "__main__":
    sys.exit(main())
