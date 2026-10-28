#!/usr/bin/env python3
"""measure-source-size.py — PLAN-869 T-01 受管三仓源码规模度量（A/B/C 口径）。

来源：PLAN-844 冻结口径探针 docs/plans/attachments/844/probe/measure.py 的
经审查等价移植（V-01 同输入逐桶对照归因），长期受管于 scripts/。
分桶/行分类/白名单与 844 完全一致；新增（不影响桶归属）：

- A/B/C 三口径汇总（PLAN-869 §5.1，2026-10-28 用户裁定）：
  A 约定范围全部 .rs = P+T2+XT+G+VEND 桶 total（XO 另列不并入）
  B 生产 NET-PROD  = Σ (P total − P inline test)
  C 旧"单 crate"口径 = 历史对照常量（2026-10-02 拆分前形态，不再成立）
- per-crate 切片（PLAN-869 §5.2 清单数据源）
- --json 机读输出（含 commit 绑定）；两次运行同输入逐字节一致（V-01）

用法：
  python scripts/measure-source-size.py                 # 三仓文本报告（env 可覆盖路径）
  python scripts/measure-source-size.py --json out.json # 附机读输出
  MEASURE_LANG=... MEASURE_UI=... MEASURE_MAN=... python scripts/measure-source-size.py
"""
import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

REPOS = {
    "auto-lang": os.environ.get("MEASURE_LANG", "D:/autostack/auto-lang"),
    "auto-ui": os.environ.get("MEASURE_UI", "D:/autostack/auto-ui"),
    "auto-man": os.environ.get("MEASURE_MAN", "D:/autostack/auto-man"),
}

# 与 844 measure.py 逐字一致（evidence: attachments/844/probe/measure.py）：
# verified cfg(test)-gated file-based test modules
TEST_FILE_WHITELIST = {
    "auto-lang": [
        "crates/auto-lang/src/tests/",           # lib.rs #[cfg(test)] pub(crate) mod tests
        "crates/auto-lang-core/src/parser/tests.rs",  # parser.rs:2885 #[cfg(test)] mod tests
    ],
    "auto-ui": [],
    "auto-man": [],
}

GEN_MARKER = re.compile(r"do not edit", re.I)
CFG_TEST_ATTR = re.compile(r"#\[\s*cfg\(\s*(all\()?\s*test")

# C 口径历史对照（THR-D1 / PLAN-873 记载：2026-10-02 拆分前单 crate 76.6 万行/1,650 文件）
C_HISTORICAL_NOTE = ("C(旧单crate口径)=2026-10-02 拆分前形态 766k 行/1650 文件"
                     "（THR-D1 记载）；拆分后该口径已不成立，仅历史对照，不作当前分母")

BUCKET_ORDER = ["P", "T2", "G", "XT", "XO", "VEND", "AT-P", "AT-L", "AT-T", "AT-O", "OTHER"]


def ls_files(root: Path) -> list:
    out = subprocess.run(["git", "-c", "core.quotepath=off", "ls-files"],
                         cwd=root, capture_output=True, text=True, check=True)
    return out.stdout.splitlines()


def classify_rs_lines(text: str):
    """Return (total, prod, comment, blank, test). 与 844 状态机逐字等价。"""
    lines = text.splitlines()
    total = len(lines)
    prod = comment = blank = test = 0
    state = "NORMAL"
    depth = 0
    for raw in lines:
        line = raw.strip()
        is_blank = not line
        is_comment = line.startswith("//") or line.startswith("/*") or line.startswith("*")

        if state == "IN_TEST":
            test += 1
            depth += line.count("{") - line.count("}")
            if depth <= 0:
                state = "NORMAL"
            continue

        if line.startswith("#["):
            if state == "PENDING" or CFG_TEST_ATTR.search(line):
                state = "PENDING"
                test += 1
            elif is_blank:
                blank += 1
            elif is_comment:
                comment += 1
            else:
                prod += 1
            continue

        if state == "PENDING":
            opens = line.count("{") - line.count("}")
            test += 1
            if "{" in line:
                if opens > 0:
                    depth = opens
                    state = "IN_TEST"
                elif opens == 0:
                    state = "NORMAL"  # single-line body fn f() {}
            elif line == ";":
                state = "NORMAL"  # file-based mod decl
            # else: item header line (`mod tests`, `fn foo()`), stay PENDING
            continue

        if is_blank:
            blank += 1
        elif is_comment:
            comment += 1
        else:
            prod += 1
    return total, prod, comment, blank, test


def is_generated(path: Path) -> bool:
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            for i, line in enumerate(f):
                if i >= 12:
                    break
                if GEN_MARKER.search(line):
                    return True
    except OSError:
        return False
    return False


def bucket_for(repo: str, rel: str, suffix: str, gen: bool) -> str:
    wl = TEST_FILE_WHITELIST.get(repo, [])
    if any(rel == w or rel.startswith(w) for w in wl):
        return "T2"
    if rel.startswith("docs/"):
        return "OTHER"  # tracked docs/historical evidence attachments
    if gen:
        return "G"
    if suffix == ".rs":
        parts = rel.split("/")
        if rel.startswith(("test/", "tests/")):
            return "XT"
        if len(parts) >= 3 and parts[0] == "crates" and parts[2] in ("test", "tests"):
            return "XT"  # per-crate corpora / integration tests
        if rel.startswith("vendor/") or rel.startswith("crates/patches/"):
            return "VEND"
        if rel.startswith(("crates/", "src/")) and "/src/" in ("/" + rel):
            return "P"
        return "XO"
    if suffix == ".at":
        if rel.startswith("packages/aavm/lib-legacy/"):
            return "AT-L"
        if rel.startswith(("packages/aavm/", "stdlib/")):
            return "AT-P"
        if rel.startswith(("test/", "tests/")):
            return "AT-T"
        return "AT-O"
    return "OTHER"


def crate_of(rel: str):
    """P/T2/XT 桶文件 → crate 切片键（crates/<a>/<b>/... 支持嵌套成员；根 src → (root)）。"""
    parts = rel.split("/")
    if parts[0] == "crates" and "/src/" in ("/" + rel):
        # crates/auto-cosmic/ports/src/... → auto-cosmic/ports；crates/auto-lang/src/... → auto-lang
        idx = parts.index("src")
        if idx == 2:
            return parts[1]
        if idx > 2:
            return "/".join(parts[1:idx])
    if parts[0] == "src":
        return "(root)"
    return None


def git_head(root: Path) -> str:
    out = subprocess.run(["git", "rev-parse", "HEAD"], cwd=root,
                         capture_output=True, text=True, check=True)
    return out.stdout.strip()


def git_dirty_rs(root: Path) -> int:
    """未提交的 .rs 改动数（绑定时点提醒用，不改度量本身）。"""
    out = subprocess.run(["git", "status", "--porcelain", "--", "*.rs", "*.at"],
                         cwd=root, capture_output=True, text=True, check=True)
    return len([l for l in out.stdout.splitlines() if l.strip()])


def measure(repo: str, root: Path):
    files = ls_files(root)
    agg = {}
    per_file = []  # (bucket, rel, total, prod, comment, blank, test)
    per_crate = {}
    for rel in files:
        p = root / rel
        if p.is_dir():
            continue
        suffix = p.suffix.lower()
        gen = suffix in (".rs", ".at") and is_generated(p)
        b = bucket_for(repo, rel.replace("\\", "/"), suffix, gen)
        if suffix == ".rs":
            try:
                text = p.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            t, pr, c, bl, te = classify_rs_lines(text)
            per_file.append((b, rel.replace("\\", "/"), t, pr, c, bl, te))
            e = agg.setdefault(b, [0, 0, 0, 0, 0, 0])
            e[0] += t; e[1] += pr; e[2] += c; e[3] += bl; e[4] += te; e[5] += 1
            ck = crate_of(rel.replace("\\", "/"))
            if ck is not None:
                ce = per_crate.setdefault(ck, {"files": 0, "total": 0, "prod": 0,
                                               "comment": 0, "blank": 0,
                                               "inline_test": 0, "buckets": {}})
                ce["files"] += 1; ce["total"] += t; ce["prod"] += pr
                ce["comment"] += c; ce["blank"] += bl; ce["inline_test"] += te
                be = ce["buckets"].setdefault(b, [0, 0])  # [files, total]
                be[0] += 1; be[1] += t
        else:
            try:
                n = sum(1 for _ in p.open("rb"))
            except OSError:
                continue
            e = agg.setdefault(b if suffix == ".at" else "OTHER",
                               [0, 0, 0, 0, 0, 0])
            e[0] += n; e[1] += n; e[5] += 1
    return agg, per_file, per_crate


def abc_summary(all_agg):
    """A=Σ(P+T2+XT+G+VEND total)；B=Σ NET-PROD；C=历史常量说明。"""
    a = sum(all_agg[r].get(b, [0] * 6)[0]
            for r in all_agg for b in ("P", "T2", "XT", "G", "VEND"))
    xo = sum(all_agg[r].get("XO", [0] * 6)[0] for r in all_agg)
    net_prod = {r: (all_agg[r].get("P", [0] * 6)[0] - all_agg[r].get("P", [0] * 6)[4])
                for r in all_agg}
    b = sum(net_prod.values())
    return a, xo, net_prod, b


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--json", dest="json_out", help="机读 JSON 输出路径")
    ap.add_argument("--top", type=int, default=20, help="top-N 生产文件（文本报告）")
    args = ap.parse_args()

    out = []
    machine = {"repos": {}, "abc": {}, "generated_by": "scripts/measure-source-size.py"}
    all_agg = {}
    for repo in REPOS:
        root = Path(REPOS[repo])
        agg, per_file, per_crate = measure(repo, root)
        all_agg[repo] = agg
        commit = git_head(root)
        dirty = git_dirty_rs(root)
        machine["repos"][repo] = {
            "path": str(root), "commit": commit, "dirty_rs_atmpts": dirty,
            "buckets": {b: {"files": v[5], "total": v[0], "prod": v[1],
                            "comment": v[2], "blank": v[3], "inline_test": v[4]}
                        for b, v in sorted(agg.items())},
            "net_prod": agg.get("P", [0] * 6)[0] - agg.get("P", [0] * 6)[4],
            "per_crate": {k: v for k, v in sorted(per_crate.items())},
        }
        out.append(f"================ {repo} @ {commit[:9]} (dirty_rs={dirty}) ================")
        out.append(f"{'bucket':6} {'files':>6} {'total':>9} {'prod':>9} {'comment':>8} {'blank':>8} {'test_inline':>12}")
        for b in BUCKET_ORDER:
            if b in agg:
                e = agg[b]
                out.append(f"{b:6} {e[5]:>6} {e[0]:>9} {e[1]:>9} {e[2]:>8} {e[3]:>8} {e[4]:>12}")
        if "P" in agg:
            p = agg["P"]
            out.append(f"NET-PROD (P total minus inline test) = {p[0] - p[4]}")
        top = sorted([f for f in per_file if f[0] == "P"], key=lambda x: -x[2])[:args.top]
        out.append("--- top production .rs files ---")
        for b, rel, t, pr, c, bl, te in top:
            out.append(f"{t:>7} {rel}")
        out.append("")

    a, xo, net_prod, b_total = abc_summary(all_agg)
    out.append("================ A/B/C 口径（PLAN-869 §5.1 裁定） ================")
    for r in net_prod:
        out.append(f"B[{r}] NET-PROD = {net_prod[r]}")
    out.append(f"A 约定范围全部 .rs (P+T2+XT+G+VEND total) = {a}")
    out.append(f"  其中 XO（examples/experimental/website 等，另列不并入 A）= {xo}")
    out.append(f"B 生产 NET-PROD 三仓合计 = {b_total}")
    out.append(f"A − 800,000 = {a - 800000}")
    out.append(C_HISTORICAL_NOTE)

    machine["abc"] = {
        "A_all_rs": a, "A_xo_separately": xo, "A_minus_800k": a - 800000,
        "B_net_prod_total": b_total, "B_net_prod_per_repo": net_prod,
        "C_historical": C_HISTORICAL_NOTE,
        "rule": "A=对外披露/达标口径；B=不反弹约束；测试/生成/第三方分列（net-slimming §统一口径）",
    }

    report = "\n".join(out)
    print(report)
    if args.json_out:
        Path(args.json_out).write_text(
            json.dumps(machine, indent=1, sort_keys=True), encoding="utf-8")
        print(f"[written] {args.json_out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
