#!/usr/bin/env python3
"""PLAN-815 M815-A staged ledger projection (guarded offline channel).

Phase increment only (no whole-plan delivery entries), following the
plan810/811 precedent: byte-exact backup, four surgical items
(designs/tests/reviews/reports), version bump by one, six-section schema
validation, read-back of the written IDs.
"""

from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LEDGER = ROOT / ".autoos/specs.json"
BACKUP = ROOT / ".autoos/specs.json.bak-v57-815a"

DELIVERY = "e609caf1a"  # M815-A staged master landing (ancestor-verified)
REVIEWED = "7045e20fa"  # R4 stage pass commit

DESIGN = {
    "id": "P815-A-D1",
    "title": "I-TOOL 轻工具协议与 auto-ui-tool 窄宿主设计（PLAN-815 M815-A）",
    "content": (
        "crates/auto-tool-protocol：版本1/能力生命周期描述符/预算（帧载荷8MiB不含前缀、"
        "进度1MiB/op、队列128、wire深度120/完整帧122=120+真实信封2、解析器128最后防线）/"
        "七类具名错误/双向信封 + 共享 4字节大端分帧（write/read/parse 三入口同限）。"
        "auto-project-model wire feature：GenerationRequest 同源无损 JSON codec（严格嵌套"
        "键集、受控 tag 解析、封闭诊断字段集、数值/窗口/路径/深度显名拒绝）。"
        "crates/auto-ui-tools：真实 auto-ui-tool 进程，describe+validate_project；版本/能力"
        "拒绝先于副作用；路径按请求内绝对根锚定（盘符相对/无盘根=cwd-qualified 不可校验），"
        "角色检查；日志仅 stderr；退出码0/1/2。客户端经子进程协议消费，不静态链接实现。"
        "canonical: docs/specs/tool-protocol/project.md + docs/specs/auto-ui-tools/project.md"
        " + docs/specs/project-model/project.md(wire节)。B（生成/build/run 迁移）未发布。"
    ),
    "status": "active",
    "file": "docs/specs/tool-protocol/project.md",
    "tags": ["PLAN-815", "stage:M815-A", f"delivery:{DELIVERY}", f"reviewed:{REVIEWED}"],
}

TESTS = {
    "id": "P815-A-T",
    "title": "I-TOOL/auto-ui-tool 定向回归与外部消费门（PLAN-815 M815-A）",
    "content": (
        "protocol 18（信封/分帧/预算耦合/深度链真信封边界/版本u64）；model wire 52"
        "（往返无损/嵌套漂移/数值边界/诊断准入/深度包络/受控tag含同路径合法对照）；"
        "tools 21（17 真实进程：describe/validate 诚实边界/未知版本与操作零副作用/缺字段/"
        "cancel 语义/超限帧 exit2/控制流纯帧日志分离/双cwd对照/盘符相对 cwd-qualified/"
        "坏payload受控不致命 + 4 资产清单锚定）；外部消费门 test/fixtures/modularity/"
        "ui-tool-consumer（cargo tree 闭包断言 + AUTO_UI_TOOL_BIN 真子进程 4 例）。"
        "冻结门（T-14）：裸 cargo t 5303/5303、fixture 双门禁、777 strict pass、"
        "二进制 SHA256 1dc1ec700c…（与 R3/R4 审查钉定一致）。"
    ),
    "status": "active",
    "file": "docs/specs/auto-ui-tools/project.md",
    "tags": ["PLAN-815", "stage:M815-A", f"delivery:{DELIVERY}"],
}

REVIEWS = {
    "id": "P815-A-R",
    "title": "PLAN-815 M815-A 阶段审查链 R1..R4（pass）",
    "content": (
        "R1 needs_fix（6 findings：诊断payload/路径无损、嵌套schema、数值窄化、cwd 路径、"
        "深度包络、健康门）→ R2 needs_fix（F-R1-02/04/05 续开 + F-R2-01 诊断前缀回归）→ "
        "R3 needs_fix（唯一 F-R3-01：负控单测双重包装断言空洞，实现全闭）→ R4 pass 绑定 "
        f"{REVIEWED}：全部 finding CLOSED、无新增；实现/依赖未变证据按哈希复用；规范增量"
        "校对为 spec-delta-reviewed.md（SD-01-A/02-A/04-A/06-A）。merge-owned 冻结门由 "
        "T-14 执行后在发布收据回填。不授予整 Plan reviewed。"
    ),
    "status": "verified",
    "file": "docs/plans/attachments/815/review-r4/review.md",
    "tags": ["PLAN-815", "stage:M815-A", "review:R4-pass", f"reviewed:{REVIEWED}"],
}

REPORTS = {
    "id": "P815-A-REL",
    "title": "M815-A 阶段发布收据（PLAN-815，非整 Plan 交付）",
    "content": (
        "delivery e609caf1a（master 祖先已验证）；协议版本1、能力 describe/"
        "validate_project；reviewed 7045e20fa；冻结门：裸 t 5303/5303 + fixture 双门禁 + "
        "777 strict；Spec 子集 SD-01-A/02-A/04-A/06-A 已落 canonical；二进制与来源指纹见"
        "收据。816 可消费已发布窄能力；生成/build/run/资产物理迁移/旧门面退出属 M815-B。"
        "Plan 保持 executing，唯一树/分支保留同步 master 继续 B。"
    ),
    "status": "active",
    "file": "docs/plans/attachments/815/m815-a-release.md",
    "tags": ["PLAN-815", "stage:M815-A", f"delivery:{DELIVERY}", "receipt:release"],
}

SECTION_OF = {"designs": DESIGN, "tests": TESTS, "reviews": REVIEWS, "reports": REPORTS}
# actual ledger section order (validated on read-back; schema drift = abort)
EXPECTED_SECTIONS = ["reports", "goals", "architecture", "designs", "tests", "reviews"]


def main() -> None:
    original = LEDGER.read_bytes()
    shutil.copyfile(LEDGER, BACKUP)
    ledger = json.loads(original.decode("utf-8"))

    ids = {s["id"] for s in ledger["sections"] for i in s.get("items", [])}
    for section_id, item in SECTION_OF.items():
        if item["id"] in ids:
            raise SystemExit(f"refusing to duplicate ledger id {item['id']}")
        section = next(s for s in ledger["sections"] if s["id"] == section_id)
        section.setdefault("items", []).append(item)

    ledger["version"] = int(ledger["version"]) + 1
    LEDGER.write_text(
        json.dumps(ledger, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    # validate: six sections, exact order, written ids read back
    reread = json.loads(LEDGER.read_text(encoding="utf-8"))
    assert [s["id"] for s in reread["sections"]] == EXPECTED_SECTIONS, "section schema drift"
    written = {
        i["id"] for s in reread["sections"] for i in s.get("items", []) if i["id"].startswith("P815-A-")
    }
    expected = {item["id"] for item in SECTION_OF.values()}
    assert written == expected, f"read-back mismatch: {written} != {expected}"
    print(f"ledger v{reread['version']}: upserted {sorted(written)}; backup {BACKUP.name}")


if __name__ == "__main__":
    _ = subprocess.run(["git", "rev-parse", "--show-toplevel"], cwd=ROOT, capture_output=True)
    main()
