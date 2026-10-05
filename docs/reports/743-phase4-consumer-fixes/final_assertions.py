#!/usr/bin/env python3
"""PLAN-743 最终归档断言（r5 版；r4 helper 检查项全部保留并扩展，见 git 历史）。

r5 扩展（T-32）：33 任务双格式完成、frontmatter 33/33、收据 key PLAN-743:r5、
Markdown 相对链接从【实际父目录】解析（忽略围栏代码）、全部 P743-* 指针、
module plans.md 行、canonical（r3 摘要消失 + r4 分类政策 + r5 严格门附加保证）、
严格门。任何一项失败即非零并定位。

用法（merge 的 canonical/ledger 落地与归档完成后，主检出根目录）：
  PYTHONUTF8=1 python -B docs/reports/743-phase4-consumer-fixes/final_assertions.py .
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

PLAN_ARCHIVE = Path("docs/plans/archive/743-acc-bootstrap-hir-contract.md")
PLAN_ACTIVE = Path("docs/plans/743-acc-bootstrap-hir-contract.md")
TOTAL_TASKS = 33
RECEIPT_KEY = "PLAN-743:r5"


def _link_check(plan_path: Path, body: str, fails: list):
    """从计划文件实际父目录解析相对链接（忽略围栏代码）。"""
    outside, fence = [], False
    for line in body.split("\n"):
        if line.strip().startswith("~~~"):
            fence = not fence
            continue
        if not fence:
            outside.append(line)
    for m in re.finditer(r"\]\(([^)#]+?)\)", "\n".join(outside)):
        link = m.group(1)
        if link.startswith(("http://", "https://")):
            continue
        if not (plan_path.parent / link).resolve().exists():
            fails.append("断链（自 %s）: %s" % (plan_path.parent.name, link))


def main(root: str) -> int:
    root = Path(root).resolve()
    fails = []

    def check(ok: bool, msg: str):
        if not ok:
            fails.append(msg)

    # 1) active/archive 状态与计数
    check(not PLAN_ACTIVE.exists(), "active 计划文件仍存在: %s" % PLAN_ACTIVE)
    check(PLAN_ARCHIVE.is_file(), "归档计划文件缺失: %s" % PLAN_ARCHIVE)
    if PLAN_ARCHIVE.is_file():
        head = PLAN_ARCHIVE.read_text(encoding="utf-8")[:600]
        check("status: archived" in head, "归档 frontmatter 非 archived")
        m = re.search(r"current_step:\s*(\d+)", head)
        n = re.search(r"total_steps:\s*(\d+)", head)
        check(m and n and m.group(1) == n.group(1) == str(TOTAL_TASKS),
              "计数非 %d/%d: current=%s total=%s"
              % (TOTAL_TASKS, TOTAL_TASKS, m and m.group(1), n and n.group(1)))
        body = PLAN_ARCHIVE.read_text(encoding="utf-8")
        # 双格式完成判定：复选框行（r1/r2/r4/r5）与表格行+紧随完成证据（r3）
        done_ids = set(re.findall(r"^- \[x\] (T-\d+) ", body, re.M))
        lines_ = body.split("\n")
        for i, line in enumerate(lines_):
            m2 = re.match(r"^\| (T-\d+) \|", line)
            if m2 and any("[✅" in lines_[j]
                          for j in range(i + 1, min(i + 4, len(lines_)))):
                done_ids.add(m2.group(1))
        expected = {"T-%02d" % k for k in range(1, TOTAL_TASKS + 1)}
        check(done_ids == expected,
              "任务完成集不符: 缺 %s 多 %s" % (sorted(expected - done_ids),
                                              sorted(done_ids - expected)))
        check(RECEIPT_KEY in body, "缺 %s 合并收据" % RECEIPT_KEY)
        _link_check(PLAN_ARCHIVE, body, fails)

    # 2) ledger P743-* 指针全部可解析
    ledger = root / ".autoos" / "specs.json"
    check(ledger.is_file(), "ledger 缺失")
    if ledger.is_file():
        data = json.loads(ledger.read_text(encoding="utf-8"))
        p743 = [it for sec in data["sections"] for it in sec["items"]
                if str(it.get("id", "")).startswith("P743")]
        check(len(p743) >= 6, "P743-* 条目数异常: %d" % len(p743))
        for it in p743:
            fp = root / str(it.get("file", ""))
            check(fp.is_file(), "%s file 指针不可解析: %s" % (it["id"], it.get("file")))

    # 3) module plans.md 743 行归档态
    for mod in ("auto-acc", "auto-hir"):
        pm = root / "docs" / "specs" / mod / "plans.md"
        check(pm.is_file(), "%s plans.md 缺失" % mod)
        if pm.is_file():
            row = [l for l in pm.read_text(encoding="utf-8").split("\n")
                   if l.startswith("| 743 |")]
            check(len(row) == 1 and "archived" in row[0],
                  "%s plans.md 743 行非归档态: %s" % (mod, row))

    # 4) canonical：无旧 R3 摘要残留、r3/r4/r5 关键内容在案
    sc = root / "docs" / "specs" / "auto-hir" / "stage-contract.md"
    check(sc.is_file() and "R3 effect 只能收窄" not in
          sc.read_text(encoding="utf-8"), "stage-contract 旧 R3 摘要残留")
    aa = root / "docs" / "specs" / "auto-acc" / "project.md"
    aa_text = aa.read_text(encoding="utf-8") if aa.is_file() else ""
    check("enclosing owner" in aa_text, "auto-acc 缺 r3/r4 分类政策正文")
    check("不能替代审定的证据引用" in aa_text, "auto-acc 缺 r4 证据覆盖约束")
    check("bare 调用与 qualified 调用在宿主/类型升级前均检查可观察绑定" in aa_text
          or "bare调用与qualified调用" in aa_text.replace(" ", "")
          or "裸调用与qualified调用在宿主/类型升级前均检查可观察绑定" in aa_text,
          "auto-acc 缺 r5 SD-09 裸/qualified 绑定优先规则")

    # 5) 严格门（真实报告）
    r = subprocess.run(
        [sys.executable, "-B", "scripts/acc_inventory.py", "--root", str(root),
         "--output", str(root / "docs/reports/743-acc-hir-contract"),
         "--check", "--require-decisions"],
        capture_output=True, text=True, encoding="utf-8", errors="replace",
        env={"PYTHONUTF8": "1", "PATH": os.environ["PATH"],
             "SYSTEMROOT": os.environ.get("SYSTEMROOT", "")})
    check(r.returncode == 0, "严格门非零: %s" % (r.stderr.strip()[:200] or r.stdout.strip()[:200]))

    if fails:
        for f in fails:
            print("ASSERT-FAIL:", f, file=sys.stderr)
        print("FINAL-ASSERTIONS: %d 项失败" % len(fails), file=sys.stderr)
        return 1
    print("FINAL-ASSERTIONS: 全部通过（归档 %d/%d、P743-* 指针、索引、canonical、"
          "链接、严格门）" % (TOTAL_TASKS, TOTAL_TASKS))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1] if len(sys.argv) > 1 else "."))
