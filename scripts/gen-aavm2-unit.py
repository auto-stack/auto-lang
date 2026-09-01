#!/usr/bin/env python3
"""Plan 511 L3:aavm2 99_unit 聚合生成器(D5 定案——聚合方案)。

将 auto/lib 六文件(AUTO_LIB_FILES_V2 依赖序)与 scripts/aavm2_unit_cases/
的 #[test] 片段聚合为单文件 test/vm/aavm2/99_unit/all_unit.at,
`auto test test/vm/aavm2/99_unit` 直跑(仓库根执行;模块用例用相对路径)。

用法:python scripts/gen-aavm2-unit.py [--check]
  --check:仅校验生成物与源同步(退出码非 0 即过期),不写盘。
"""
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
LIB_FILES = [
    "auto/lib/token.at",
    "auto/lib/lexer.at",
    "auto/lib/parser.at",
    "auto/lib/typeinfo.at",
    "auto/lib/codegen.at",
    "auto/lib/engine.at",
]
CASES_DIR = ROOT / "scripts" / "aavm2_unit_cases"
OUT = ROOT / "crates/auto-lang/test/vm/aavm2/99_unit/all_unit.at"


def gen() -> str:
    parts = [
        "// 由 scripts/gen-aavm2-unit.py 生成(Plan 511 L3/D5 聚合方案)——请勿手改;\n"
        "// 再生成:python scripts/gen-aavm2-unit.py\n"
    ]
    for f in LIB_FILES:
        src = (ROOT / f).read_text(encoding="utf-8")
        parts.append(src.rstrip("\n") + "\n")
    for case in sorted(CASES_DIR.glob("*.at")):
        parts.append(case.read_text(encoding="utf-8").rstrip("\n") + "\n")
    return "\n".join(parts)


def main() -> int:
    out = gen()
    if "--check" in sys.argv:
        if OUT.exists() and OUT.read_text(encoding="utf-8") == out:
            print("up to date:", OUT)
            return 0
        print("STALE:", OUT, "(run python scripts/gen-aavm2-unit.py)")
        return 1
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(out, encoding="utf-8", newline="\n")
    print("generated:", OUT)
    return 0


if __name__ == "__main__":
    sys.exit(main())
