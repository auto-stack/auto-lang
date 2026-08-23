#!/usr/bin/env python3
"""plan-429 B1: AAVM shim 需求盘点（一次性报告工具,plan-430 元信息工具的前身）.

扫描核心自举范围的 Rust 文件,提取 std API 使用面(全限定路径 + 带 receiver 推断的方法调用),
对照 dispatch 3000 手写臂输出缺口报告。receiver 推断是启发式的——只信任两种来源:
  1. `let x: Vec<..>` / `let x = Vec::new()` 型声明的类型注解/构造器
  2. 函数参数 `s: &str` / `m: &HashMap<..>`
未匹配 receiver 的方法只进"待人工归类"频率表。
"""
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

CORE_FILES = [
    "crates/auto-lang/src/token.rs",
    "crates/auto-lang/src/lexer.rs",
    "crates/auto-lang/src/error.rs",
    "crates/auto-lang/src/parser.rs",
    "crates/auto-lang/src/types.rs",
    "crates/auto-lang/src/ast.rs",
    "crates/auto-lang/src/infer/context.rs",
    "crates/auto-lang/src/infer/expr.rs",
    "crates/auto-lang/src/infer/stmt.rs",
    "crates/auto-lang/src/infer/functions.rs",
    "crates/auto-lang/src/infer/unification.rs",
    "crates/auto-lang/src/vm/opcode.rs",
    "crates/auto-lang/src/vm/codegen.rs",
    "crates/auto-lang/src/vm/engine.rs",
    "crates/auto-lang/src/vm/native_catalog.rs",
]

TRACKED_TYPES = ["String", "Vec", "HashMap", "HashSet", "str", "char", "Option", "Result", "Box", "Rc", "Arc"]
# dispatch 3000 现有覆盖(2026-08 snapshot,手工提取)
COVERED = set()  # filled by extract_covered()

DECL_LET = re.compile(r"\blet\s+(?:mut\s+)?(\w+)\s*(?::\s*([^=]+?))?\s*=\s*(.+?);")
PARAM = re.compile(r"\(\s*(\w+)\s*:\s*([^,)]+)")
CTOR = re.compile(r"^(String|Vec|HashMap|HashSet|Box|Rc|Arc|Option|Result)::")
METHOD_CALL = re.compile(r"(\w+)\s*\.\s*([a-z_][a-z0-9_]*)\s*\(")
STD_PATH = re.compile(r"\bstd::[a-z_][a-z0-9_:]*")


def base_type(ann: str) -> str | None:
    ann = ann.strip()
    for t in TRACKED_TYPES:
        if re.search(rf"(?:^|[^a-zA-Z_]){t}\s*(?:<|$)", ann):
            return t
    return None


def extract_covered():
    src = (ROOT / "crates/auto-lang/src/vm/ffi/stdlib.rs").read_text(encoding="utf-8")
    body = src.split("fn shim_rust_stdlib_dispatch", 1)[1]
    for ty, m in re.findall(r'\("([A-Za-z_][A-Za-z0-9_]*)",\s*"([a-z_][a-z0-9_]*)"\)', body):
        COVERED.add((ty, m))


def scan_file(path: Path):
    text = path.read_text(encoding="utf-8", errors="replace")
    rel = str(path.relative_to(ROOT))
    std_paths = Counter(STD_PATH.findall(text))

    # receiver 推断:逐行维护一个保守的 var->type 映射(只在本行之前可见的声明)
    var_types = {}
    typed_calls = defaultdict(Counter)   # Type -> method -> count
    untyped = Counter()                  # method -> count
    for line in text.splitlines():
        for m in DECL_LET.finditer(line):
            name, ann, rhs = m.group(1), m.group(2), m.group(3)
            ty = base_type(ann) if ann else None
            if ty is None:
                c = CTOR.match(rhs.strip())
                if c:
                    ty = c.group(1)
            if ty:
                var_types[name] = ty
        for m in PARAM.finditer(line):
            ty = base_type(m.group(2))
            if ty:
                var_types[m.group(1)] = ty
        for m in METHOD_CALL.finditer(line):
            recv, meth = m.group(1), m.group(2)
            ty = var_types.get(recv)
            if ty:
                typed_calls[ty][meth] += 1
            else:
                untyped[meth] += 1
    return rel, std_paths, typed_calls, untyped


def main():
    extract_covered()
    all_std = Counter()
    all_typed = defaultdict(Counter)
    all_untyped = Counter()
    per_file = []
    for f in CORE_FILES:
        p = ROOT / f
        if not p.exists():
            print(f"WARN missing {f}", file=sys.stderr)
            continue
        rel, std_paths, typed, untyped = scan_file(p)
        all_std.update(std_paths)
        for ty, ctr in typed.items():
            all_typed[ty].update(ctr)
        all_untyped.update(untyped)
        per_file.append(rel)

    out = []
    out.append("# plan-429 B1: AAVM shim 需求盘点报告\n")
    out.append(f"- 扫描范围({len(per_file)} 文件,核心自举粗口径,未剔 UI 段——parser.rs 全量计入,偏保守):\n")
    out.append("\n".join(f"  - {r}" for r in per_file))
    out.append("- 方法: 启发式 receiver 推断(let 注解/构造器/函数参数),未匹配 receiver 的调用单列\n")

    out.append("\n## 1. std 全限定路径使用频率\n")
    for k, v in all_std.most_common():
        out.append(f"- {k}: {v}")

    out.append("\n## 2. 核心 receiver 类型的 方法需求 vs dispatch 3000 覆盖\n")
    for ty in ["String", "Vec", "HashMap", "HashSet", "char", "Option", "Box", "Rc", "Arc", "str"]:
        ctr = all_typed.get(ty)
        if not ctr:
            continue
        out.append(f"\n### {ty}\n")
        out.append("| 方法 | 使用次数 | dispatch 3000 |")
        out.append("|---|---|---|")
        for meth, n in ctr.most_common():
            cov = "✅" if (ty, meth) in COVERED else ("~" if any(m == meth for t, m in COVERED if t == ty) else "❌")
            out.append(f"| {meth} | {n} | {cov} |")

    out.append("\n## 3. 未匹配 receiver 的方法频率(前 80,待 431 边界定稿后二次归类)\n")
    for meth, n in all_untyped.most_common(80):
        out.append(f"- {meth}: {n}")

    report = "\n".join(out)
    dest = ROOT / "docs/plans/reports/429-b1-shim-inventory.md"
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(report, encoding="utf-8")
    print(f"written: {dest} ({len(report)} chars)")
    # 速览
    core_gap = {}
    for ty in ["String", "Vec", "HashMap", "HashSet"]:
        ctr = all_typed.get(ty, Counter())
        gap = [m for m in ctr if (ty, m) not in COVERED]
        core_gap[ty] = (len(ctr), len(gap), sorted(gap))
    print("=== 核心类型缺口速览 ===")
    for ty, (used, gap, methods) in core_gap.items():
        print(f"{ty}: 使用 {used} 个方法, 缺口 {gap} 个: {','.join(methods)}")


if __name__ == "__main__":
    main()
