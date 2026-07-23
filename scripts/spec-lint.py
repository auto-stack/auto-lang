#!/usr/bin/env python3
"""spec-lint.py — specs 树健康检查（只报告，不自动修改）。

检查项：
1. 每个 project 目录必须有 project.md；有 module 子目录则必须有 overview.md。
2. plan 编号唯一性（docs/plans/ 活跃区）与 .next-id 合理性。
3. specs 文档内相对链接（./ 或 ../ 开头）指向的文件存在。
4. overview.md 超期未更新（默认 90 天，按 git 最后提交时间）标 stale。

用法: python scripts/spec-lint.py [--stale-days N]
退出码: 有 ERROR 为 1，仅 WARN 为 0。
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPECS = ROOT / "docs" / "specs"
PLANS = ROOT / "docs" / "plans"

errors, warnings, infos = [], [], []

LINK_RE = re.compile(r"\[[^\]]*\]\((\.{1,2}/[^)\s]+)\)")


def git_last_commit_days(path: Path):
    try:
        out = subprocess.run(
            ["git", "log", "-1", "--format=%ct", "--", str(path)],
            cwd=ROOT, capture_output=True, text=True, timeout=10,
        ).stdout.strip()
        if out:
            import time
            return (time.time() - int(out)) / 86400
    except Exception:
        pass
    return None


def check_structure():
    for proj in sorted(SPECS.iterdir()):
        if not proj.is_dir() or proj.name.startswith("_"):
            continue
        if not (proj / "project.md").exists():
            errors.append(f"{proj.name}: 缺 project.md")
        for mod in sorted(proj.iterdir()):
            if mod.is_dir() and not (mod / "overview.md").exists():
                warnings.append(f"{proj.name}/{mod.name}: module 目录缺 overview.md")


def check_plan_ids():
    seen = {}
    for f in sorted(PLANS.glob("*.md")):
        m = re.match(r"(\d+)-", f.name)
        if m:
            n = int(m.group(1))
            if n in seen:
                errors.append(f"plan 编号重复: {n}（{seen[n]} 与 {f.name}）")
            seen[n] = f.name
    id_file = PLANS / ".next-id"
    if id_file.exists():
        nxt = int(id_file.read_text().strip())
        if seen and nxt <= max(seen):
            errors.append(f".next-id={nxt} 不大于活跃区最大编号 {max(seen)}")
    else:
        errors.append("docs/plans/.next-id 不存在")


def check_links():
    for md in sorted(SPECS.rglob("*.md")):
        if "_archive" in md.parts:
            continue
        for m in LINK_RE.finditer(md.read_text(encoding="utf-8")):
            target = (md.parent / m.group(1)).resolve()
            if not target.exists():
                # module 目录尚未创建属 Phase 0/1 正常状态，降级为 INFO
                if target.is_dir() or not target.suffix:
                    infos.append(f"{md.relative_to(SPECS)}: 链接目标尚不存在 {m.group(1)}")
                else:
                    warnings.append(f"{md.relative_to(SPECS)}: 断链 {m.group(1)}")


def check_stale(days: int):
    for ov in sorted(SPECS.rglob("overview.md")):
        age = git_last_commit_days(ov)
        if age is not None and age > days:
            warnings.append(f"{ov.relative_to(SPECS)}: {int(age)} 天未更新，疑似 stale")


def main():
    stale_days = 90
    if "--stale-days" in sys.argv:
        stale_days = int(sys.argv[sys.argv.index("--stale-days") + 1])
    check_structure()
    check_plan_ids()
    check_links()
    check_stale(stale_days)
    for label, items in [("ERROR", errors), ("WARN", warnings), ("INFO", infos)]:
        for it in items:
            print(f"[{label}] {it}")
    print(f"\n{len(errors)} errors, {len(warnings)} warnings, {len(infos)} infos")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
