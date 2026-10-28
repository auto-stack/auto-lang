#!/usr/bin/env python3
"""PLAN-867 T-06: golden/corpus input ledger across the split.

Compares the frozen golden-bearing inputs between the pre-split commit
(464e2316b) and the frozen current combination, then attributes every
file-level change to the plan that made it. A change nobody cites is
unexplained and blocks. Regenerating a golden to make a diff disappear is not
an option this script offers.

Sides:
  lang: git diff 464e2316b..HEAD restricted to the golden roots; each changed
        file is attributed via the first commit in the window that touched it
        (commit messages cite their plan).
  ui:   same against the auto-ui import commit (9359ee7, proven byte-identical
        to the pre-split sources by PLAN-831 T-04) — its whole history is the
        route window.

Manual rows live in scripts/split-golden-dispositions.json (path → class +
evidence) for changes the commit walk cannot attribute cleanly (e.g. moved
files whose deletion side cites the move plan).
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
PLAN_CITE = re.compile(r"\((?:Plan|PLAN)[ |-](\d+)\)|\bPLAN-(\d+)\b|\bPlan (\d+)\b")
ROUTE_PLANS = {831, 832, 833, 834, 835, 836, 837, 838, 839, 840, 841, 842, 843, 844, 845, 846,
               847, 848, 849, 850, 851, 852, 853, 854, 855, 856, 857, 858, 859, 860, 861, 862,
               863, 864, 865, 866}

LANG_ROOTS = ("crates/auto-lang/test/vm", "crates/auto-lang/test/a2c", "crates/auto-lang/test/a2r",
              "packages/blueprints", "packages/widgets", "packages/aavm", "book/rust/listings")
UI_ROOTS = ("crates/auto-ui-compiler/test", "crates/auto-ui-compiler/tests",
            "crates/auto-ui-runtime/tests", "crates/auto-ui-tools/tests",
            "crates/auto-ui-tools/drawlist-renderer/test", "crates/auto-ui-tools/wm-test/test")
UI_IMPORT = "9359ee7"


def git(repo: Path, *args: str) -> str:
    return subprocess.check_output(["git", "-C", str(repo), *args], text=True,
                                   encoding="utf-8", errors="replace")


def diff_side(repo: Path, base: str, roots: tuple[str, ...]) -> list[dict]:
    rows = []
    status = git(repo, "diff", "--name-status", base, "HEAD", "--", *roots)
    for line in status.splitlines():
        parts = line.split("\t")
        if len(parts) < 2:
            continue
        change, path = parts[0], parts[-1]
        # first commit in (base, HEAD] touching this path carries the citation
        log = git(repo, "log", "--oneline", "--reverse", f"{base}..HEAD", "--", path).splitlines()
        cite = None
        for entry in log:
            match = PLAN_CITE.search(entry)
            if match:
                cite = int(next(group for group in match.groups() if group))
                break
        rows.append({"path": path, "status": change[:1], "plan": cite,
                     "commit": log[0].split(" ", 1)[0] if log else None})
    return rows


def load_dispositions(path: Path) -> dict:
    if not path.is_file():
        return {}
    doc = json.loads(path.read_text(encoding="utf-8"))
    if doc.get("schema") != "split-golden-dispositions/1":
        raise SystemExit(f"unsupported dispositions schema: {doc.get('schema')}")
    return {row["path"]: row for row in doc["dispositions"]}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--lang", type=Path, default=HERE, help="frozen auto-lang tree")
    parser.add_argument("--ui", type=Path, default=HERE.parent / "auto-ui", help="frozen auto-ui tree")
    parser.add_argument("--lang-base", default="464e2316b")
    parser.add_argument("--ui-base", default=UI_IMPORT)
    parser.add_argument("--dispositions", type=Path,
                        default=HERE / "scripts" / "split-golden-dispositions.json")
    parser.add_argument("--write-md", type=Path)
    parser.add_argument("--diff-out", type=Path)
    args = parser.parse_args(argv)

    manual = load_dispositions(args.dispositions)
    lang_rows = diff_side(args.lang, args.lang_base, LANG_ROOTS)
    ui_rows = diff_side(args.ui, args.ui_base, UI_ROOTS)

    explained, unexplained = [], []
    for side, rows in (("auto-lang", lang_rows), ("auto-ui", ui_rows)):
        for row in rows:
            entry = dict(row, side=side)
            if row["path"] in manual:
                entry.update(manual[row["path"]])
                explained.append(entry)
            elif row["plan"] in ROUTE_PLANS:
                entry["class"] = "ROUTE_ADJUDICATED"
                explained.append(entry)
            else:
                entry["class"] = "UNEXPLAINED"
                unexplained.append(entry)

    lines = ["# PLAN-867 金样/语料输入台账（T-06）", "",
             f"- lang 侧：`git diff --name-status {args.lang_base}..HEAD -- {' '.join(LANG_ROOTS)}`"
             f" @ {args.lang.resolve()}",
             f"- ui 侧：`git diff --name-status {args.ui_base}..HEAD -- {' '.join(UI_ROOTS)}`"
             f" @ {args.ui.resolve()}（import 基线 {UI_IMPORT}，831 T-04 已证逐字节恒等）",
             "- 无文件输入的重族：1M churn（测试体内生成）；画廊围栏输入=packages/widgets（在列）。",
             "", "## 变更分类", ""]
    counts: dict[str, int] = {}
    for entry in explained:
        counts[entry["class"]] = counts.get(entry["class"], 0) + 1
    counts["UNEXPLAINED"] = len(unexplained)
    for key in sorted(counts):
        lines.append(f"- {key}: {counts[key]}")
    lines += ["", "## 明细", "", "| side | status | path | plan | commit | class/evidence |", "|---|---|---|---|---|---|"]
    for entry in explained + unexplained:
        lines.append(f"| {entry['side']} | {entry['status']} | `{entry['path']}` | "
                     f"{entry.get('plan') or '-'} | {entry.get('commit') or '-'} | {entry['class']}"
                     + (f"：{entry['evidence']}" if entry.get("evidence") else "") + " |")
    lines += ["", f"结论：{'无未解释漂移，台账成立' if not unexplained else f'{len(unexplained)} 处未解释漂移，阻塞'}"]
    text = "\n".join(lines) + "\n"
    if args.write_md:
        args.write_md.parent.mkdir(parents=True, exist_ok=True)
        args.write_md.write_text(text, encoding="utf-8")
    else:
        sys.stdout.write(text)
    if args.diff_out:
        args.diff_out.mkdir(parents=True, exist_ok=True)
        (args.diff_out / "golden-unexplained.txt").write_text(
            "".join(f"{e['side']}\t{e['status']}\t{e['path']}\n" for e in unexplained), encoding="utf-8")
    print(f"golden ledger: explained={len(explained)} unexplained={len(unexplained)}")
    return 1 if unexplained else 0


if __name__ == "__main__":
    sys.exit(main())
