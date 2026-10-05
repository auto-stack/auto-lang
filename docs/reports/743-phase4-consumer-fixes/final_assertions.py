#!/usr/bin/env python3
"""PLAN-743 Phase 4 最终归档断言（r4 合同 QA-04/T-25 交付，merge 收口时运行）。

断言最终态：唯一归档计划、27/27 计数、全部任务勾选、全部 P743-* ledger 指针
可解析且计划引用指向 archive/ 路径、module plans.md 行归档态、canonical 无旧
R3 摘要残留、严格门绿。任何一项失败即非零并定位。

用法（merge 的 canonical/ledger 落地与归档完成后，主检出根目录）：
  PYTHONUTF8=1 python -B docs/reports/743-phase4-consumer-fixes/final_assertions.py .
"""
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

PLAN_ARCHIVE = Path("docs/plans/archive/743-acc-bootstrap-hir-contract.md")
PLAN_ACTIVE = Path("docs/plans/743-acc-bootstrap-hir-contract.md")


def main(root: str) -> int:
    root = Path(root).resolve()
    fails = []

    def check(ok: bool, msg: str):
        if not ok:
            fails.append(msg)

    # 1) active/archive 状态
    check(not PLAN_ACTIVE.exists(), "active 计划文件仍存在: %s" % PLAN_ACTIVE)
    check(PLAN_ARCHIVE.is_file(), "归档计划文件缺失: %s" % PLAN_ARCHIVE)
    if PLAN_ARCHIVE.is_file():
        head = PLAN_ARCHIVE.read_text(encoding="utf-8")[:600]
        check("status: archived" in head, "归档 frontmatter 非 archived")
        m = re.search(r"current_step:\s*(\d+)", head)
        n = re.search(r"total_steps:\s*(\d+)", head)
        check(m and n and m.group(1) == n.group(1) == "27",
              "计数非 27/27: current=%s total=%s" % (m and m.group(1), n and n.group(1)))
        body = PLAN_ARCHIVE.read_text(encoding="utf-8")
        # 双格式完成判定：复选框行（r1/r2/r4）与表格行+紧随完成证据（r3）
        done_ids = set(re.findall(r"^- \[x\] (T-\d+) ", body, re.M))
        lines_ = body.split("\n")
        for i, line in enumerate(lines_):
            m = re.match(r"^\| (T-\d+) \|", line)
            if m and any("[✅" in lines_[j]
                         for j in range(i + 1, min(i + 4, len(lines_)))):
                done_ids.add(m.group(1))
        expected = {"T-%02d" % n for n in range(1, 28)}
        check(done_ids == expected,
              "任务完成集不符: 缺 %s 多 %s" % (sorted(expected - done_ids),
                                              sorted(done_ids - expected)))
        check("PLAN-743:r4" in body, "缺 r3→r4 合并收据（key PLAN-743:r4）")

    # 2) ledger P743-* 指针全部可解析
    ledger = root / ".autoos" / "specs.json"
    check(ledger.is_file(), "ledger 缺失")
    if ledger.is_file():
        data = json.loads(ledger.read_text(encoding="utf-8"))
        p743 = [it for sec in data["sections"] for it in sec["items"]
                if str(it.get("id", "")).startswith("P743")]
        check(len(p743) >= 5, "P743-* 条目数异常: %d" % len(p743))
        for it in p743:
            fp = root / str(it.get("file", ""))
            check(fp.is_file(), "%s file 指针不可解析: %s" % (it["id"], it.get("file")))
            if str(it["id"]) == "P743-3":
                check("archive" in str(it.get("file", "")),
                      "P743-3 指针未对账为 archive 路径: %s" % it.get("file"))

    # 3) module plans.md 743 行归档态
    for mod in ("auto-acc", "auto-hir"):
        pm = root / "docs" / "specs" / mod / "plans.md"
        check(pm.is_file(), "%s plans.md 缺失" % mod)
        if pm.is_file():
            row = [l for l in pm.read_text(encoding="utf-8").split("\n")
                   if l.startswith("| 743 |")]
            check(len(row) == 1 and "archived" in row[0],
                  "%s plans.md 743 行非归档态: %s" % (mod, row))

    # 4) canonical：旧 R3 摘要已消失、r3/r4 关键内容在案
    sc = root / "docs" / "specs" / "auto-hir" / "stage-contract.md"
    check(sc.is_file() and "R3 effect 只能收窄" not in
          sc.read_text(encoding="utf-8"), "stage-contract 旧 R3 摘要残留")
    aa = root / "docs" / "specs" / "auto-acc" / "project.md"
    check(aa.is_file() and "enclosing owner" in
          aa.read_text(encoding="utf-8"), "auto-acc 缺 r3/r4 分类政策正文")

    # 5) 严格门（真实报告）
    r = subprocess.run(
        [sys.executable, "-B", "scripts/acc_inventory.py", "--root", str(root),
         "--output", str(root / "docs/reports/743-acc-hir-contract"),
         "--check", "--require-decisions"],
        capture_output=True, text=True, encoding="utf-8", errors="replace",
        env={"PYTHONUTF8": "1", "PATH": __import__("os").environ["PATH"],
             "SYSTEMROOT": __import__("os").environ.get("SYSTEMROOT", "")})
    check(r.returncode == 0, "严格门非零: %s" % (r.stderr.strip()[:200] or r.stdout.strip()[:200]))

    if fails:
        for f in fails:
            print("ASSERT-FAIL:", f, file=sys.stderr)
        print("FINAL-ASSERTIONS: %d 项失败" % len(fails), file=sys.stderr)
        return 1
    print("FINAL-ASSERTIONS: 全部通过（归档 27/27、P743-* 指针、索引、canonical、严格门）")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1] if len(sys.argv) > 1 else "."))
