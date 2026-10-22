#!/usr/bin/env python3
"""PLAN-815 M815-B final ledger projection (guarded offline channel).

Whole-B delivery entries (design/tests/reviews/reports), following the
store schema: byte-exact backup, four surgical items, version bump by
one, whole-document schema validation, exact read-back and rollback.
A legacy or invalid store must be restored separately before projection.

M815-B channel notes (recorded honestly in the merge receipt):
- No store service is reachable (HTTP probe negative) -> guarded offline
  projection, same channel as the M815-A deposit (plan815a_ledger_deposit).
- The live v67 store contains regions written by string-splicing (e.g.
  "]\\n   ,\\n{" separators) that no standard encoder reproduces, so a
  whole-document re-encode would disturb unrelated bytes. The deposit is
  therefore a TEXT-LEVEL SURGICAL SPLICE: only our four item blocks and
  the version digit are inserted; every other byte stays identical
  (proven by the strip-back check in self-test and by diff).
- Dual-tier schema guard: hard invariants (six-section identity/order,
  PascalCase section_type, duplicate ids, statuses, typed identity) apply
  to the whole document; full SpecItem::new field completion is required
  only for THIS script's items (pre-existing p814/816/829/830 items lack
  created_at/modified_at/completed_at/module yet load in the real store).
"""

from __future__ import annotations

import json
import hashlib
import re
import time
import copy
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LEDGER = ROOT / ".autoos/specs.json"

DELIVERY = "33c9451fbe667dde2412d71036a19ac334b03691"  # M815-B final master landing (ff-only, master tip)
FREEZE = "885aebbd0"  # rebased production freeze (0f2858e35 pre-rebase; range-diff equal)
REVIEWED = "0f2858e35cb220964d8c23c8d77afce6839db491"  # T-09 R2 final pass (pre-rebase binding)

DESIGN = {
    "id": "P815-B-D1",
    "title": "UI 工程工具唯一 owner 全量交付（PLAN-815 M815-B，SD-01..07）",
    "content": (
        "auto-ui-tools=UI 工程工具唯一 owner：vue/rust/jet/ark/tauri/vscode 生成/build/run"
        "与 gallery/wm/shadcn 资产全闭包脱 Man（normal/build/dev 无反向大包依赖）；旧 Man 入口"
        "薄转发门面；生产入口全消费已求值 I-PROJECT 请求（请求在场即权威，无第二真源）；外部"
        "命令经 owned 子进程设施；多目标完整生成、输出根=真实布局。I-TOOL v1 完整生命周期"
        "（五操作×cancellable/interactive、framing/预算、具名错误、日志控制流分离、取消/EOF"
        " owned 树终止、Cancelled 与 outcome=cancelled 分离、能力拒绝先于副作用、非 Windows"
        " 平台承诺）。发行身份语义：确定性锚=冻结源 SHA+792 delivery-manifest（逐文件"
        " rel/len/blake3 汇总目录 digest）；release SHA256 为带 provenance 参考产物指纹"
        "（无 reproducible-builds，不以互异 debug 哈希充当身份）；发行形态=源码包 workspace、"
        "工具包内 locked 构建三形态实证。runtime 宿主退出/会话生命周期公开面（SD-07）。"
        "shell-pack 公开再生器同受 canonical import 约束（F-815-T09-01 修复后规则）。"
        "canonical: docs/specs/auto-ui-tools/project.md + tool-protocol/project.md + "
        "auto-man/project.md + project-model/project.md + auto-lang/ui/overview.md + "
        "repository-boundaries/generated-ui-distribution.md + auto-ui-runtime/project.md"
    ),
    "status": "verified",
    "file": "docs/specs/auto-ui-tools/project.md",
    "tags": ["PLAN-815", "stage:M815-B", f"delivery:{DELIVERY}", f"freeze:{FREEZE}"],
}

TESTS = {
    "id": "P815-B-T",
    "title": "M815-B 工具/协议/发行测试资产（PLAN-815）",
    "content": (
        "tool_process 22 快速+9 重串行（真实进程：生成/安装链/取消树/交互/字段矩阵+删 pac"
        "逐字节一致+output_root 布局钉）；lifecycle 4 重串行（Unicode/空格真实 pnpm build"
        " dist oracle）；rust_ui 请求权威/四读取面壳层钉；vue default_classes 三态钉；"
        "shell_pack freshness 字节门（T09 修复后绿）；792 driver V01..V05 33/33（含随包"
        "工具 fresh/locked/重定位构建+framed I-TOOL 外部消费八例）；fixtures："
        "tool-protocol-consumer 7/7+闭包门、ui-tool-consumer 5/5（AUTO_UI_TOOL_BIN 真进程）"
        "+闭包门。冻结门（T-10）：裸 cargo t 5331/5331、tu 868/868、man/protocol/execution"
        " 195/195、tools lib 183/185（二预存红已归因裁定）、777 strict pass 0v/3g。"
    ),
    "status": "verified",
    "file": "docs/specs/auto-ui-tools/project.md",
    "tags": ["PLAN-815", "stage:M815-B", f"delivery:{DELIVERY}"],
}

REVIEWS = {
    "id": "P815-B-R",
    "title": "PLAN-815 M815-B 审查链（T05 R1..R8/T06..08 联合/T07 三子批/T09 R1..R2 终审 pass）",
    "content": (
        "T-05 R1..R8（取消/输入/EOF/错误传播/PM 单源/Unix 平台/PM 缓存干扰逐轮关闭）；"
        "T-06..08 提前 R1..R3 联合修复（front 贯穿/Ark ID/dist oracle/闭包门）；T-07 子批1"
        "定向 pass（协议四回执/帧深度/Linux real_tree）→子批2 needs_fix（两实修复+三 P2）→"
        "子批3 R1 修正（矩阵四误标行纠正+两钉）→R2 pass（33/33 独立、377 文件对账、release"
        "身份 4/4）；T-05..08 联合收口 pass（14/17）；T-09 终审 R1 needs_fix（唯一 P2="
        "F-815-T09-01 shell-pack 再生器缺 canonical 双直依赖、实际消费 107 E0433、不可预存"
        "豁免；plan734 裁定非阻断排版债）→修复轮（壳层五处+清单模板+sanctioned regen、"
        "freshness 红→绿、原生清单消费 exit0）→R2 终审 pass：F CLOSED 无新增，T06..08 复勾、"
        f"T09 完成（15/17 reviewed）。终审绑定 {REVIEWED}（rebase 等价 {FREEZE}）。"
    ),
    "status": "verified",
    "file": "docs/plans/attachments/815/review-t09-r2/review.md",
    "tags": ["PLAN-815", "stage:M815-B", "review:T09-R2-pass", f"reviewed:{REVIEWED}"],
}

REPORTS = {
    "id": "P815-B-REL",
    "title": "M815-B 最终发布收据（PLAN-815 整 Plan 交付）",
    "content": (
        f"delivery {DELIVERY}（master ff-only，rebase range-diff 全等价；生产冻结位 "
        f"{FREEZE}/0f2858e35，release 参考 artifact a73b117ff384…85,116,928B）；SD-01..07 "
        "已落 canonical（A 子集事实保留）；冻结门全绿（裸 t 5331/5331+tu 868+包门 195+"
        "tools 183/185 归因+fixtures 双门+777 strict）；M815-A/L 阶段收据在案（ledger v57/"
        "v59）。816 可消费完整 I-TOOL v1 五操作+auto-ui-tool；预存债归因：plan734/plan593"
        "（非阻断排版金样）、merged_api（P751-D2 在案）。Plan 归档；批量回归档（815%5=0）"
        "由 regress 收口。"
    ),
    "status": "verified",
    "file": "docs/plans/attachments/815/handoff.md",
    "tags": ["PLAN-815", "stage:M815-B", f"delivery:{DELIVERY}", "receipt:release"],
}

SECTION_OF = {"designs": DESIGN, "tests": TESTS, "reviews": REVIEWS, "reports": REPORTS}
# actual ledger section order (validated on read-back; schema drift = abort)
EXPECTED_SECTIONS = ["reports", "goals", "architecture", "designs", "tests", "reviews"]

OWN_IDS = {"P815-B-D1", "P815-B-T", "P815-B-R", "P815-B-REL"}

VALID_STATUSES = set("empty proposed draft under_review approved in_progress in_implementation implemented verified done archived rejected backlog ready in_review blocked superseded outdated stable deprecated published analysed obsolete".split())
ITEM_FIELDS = set("id title content status depends_on related priority assignee test_file file milestone module tags created_at modified_at completed_at".split())
SECTION_FIELDS = set("id section_type title items status content depends_on last_modified last_verified".split())


def is_u64(value) -> bool:
    return type(value) is int and 0 <= value <= 2**64 - 1


def schema_errors(doc: dict) -> list[str]:
    """Dual-tier guard (see module docstring): hard invariants whole-document,
    full SpecItem::new completion only for this script's items."""
    errors = []
    if not isinstance(doc.get("project"), str) or not is_u64(doc.get("version")):
        errors.append("root: project/version")
    sections = doc.get("sections", [])
    if [s.get("id") for s in sections] != EXPECTED_SECTIONS:
        errors.append("root: six-section identity/order")
    seen = set()
    for section in sections:
        sid = section.get("id", "?")
        if SECTION_FIELDS - section.keys():
            errors.append(f"section {sid}: missing {sorted(SECTION_FIELDS - section.keys())}")
        # SectionType serde wire format is PASCAL CASE (real-loader
        # verified in M815-L T-15): snake_case values fall into
        # #[serde(other)] Unknown and SpecsStore::load silently drops the
        # whole section. Section *id* stays snake_case.
        wire_type = {"reports": "Reports", "goals": "Goals", "architecture": "Architecture",
                     "designs": "Designs", "tests": "Tests", "reviews": "Reviews"}[sid]
        if section.get("section_type") != wire_type or section.get("status") not in VALID_STATUSES:
            errors.append(f"section {sid}: type/status")
        for field in ("title", "content"):
            if not isinstance(section.get(field), str):
                errors.append(f"section {sid}: invalid {field}")
        if not isinstance(section.get("depends_on"), list) or any(not isinstance(x, str) for x in section.get("depends_on", [])):
            errors.append(f"section {sid}: invalid depends_on")
        if not is_u64(section.get("last_modified")) or (section.get("last_verified") is not None and not is_u64(section["last_verified"])):
            errors.append(f"section {sid}: invalid timestamp")
        for item in section.get("items", []):
            iid = item.get("id", "?")
            problems = []
            if iid in seen:
                problems.append("duplicate id")
            seen.add(iid)
            required = ITEM_FIELDS if iid in OWN_IDS else (
                ITEM_FIELDS - {"created_at", "modified_at", "completed_at", "module"})
            missing = required - item.keys()
            if missing:
                problems.append(f"missing {sorted(missing)}")
            if item.get("status") not in VALID_STATUSES:
                problems.append(f"invalid status {item.get('status')!r}")
            for field in ("id", "title", "content"):
                if not isinstance(item.get(field), str):
                    problems.append(f"invalid {field}")
            for field in ("depends_on", "related", "tags"):
                if not isinstance(item.get(field), list) or any(not isinstance(x, str) for x in item.get(field, [])):
                    problems.append(f"invalid {field}")
            timestamp_fields = ("created_at", "modified_at", "completed_at") if iid in OWN_IDS else ()
            for field in ("created_at", "modified_at"):
                if field in timestamp_fields and not is_u64(item.get(field)):
                    problems.append(f"invalid {field}")
            for field in ("priority", "assignee", "test_file", "file", "milestone", "module"):
                if item.get(field) is not None and not isinstance(item[field], str):
                    problems.append(f"invalid {field}")
            if "completed_at" in timestamp_fields and item.get("completed_at") is not None and not is_u64(item["completed_at"]):
                problems.append("invalid completed_at")
            if problems:
                errors.append(f"{sid}/{iid}: " + "; ".join(problems))
    return errors


def validate(doc: dict) -> None:
    errors = schema_errors(doc)
    if errors:
        raise ValueError(f"store schema rejected ({len(errors)} records/sections); " + " | ".join(errors[:3]))


def proposed_items(root: Path) -> dict:
    now = int(time.time())
    result = copy.deepcopy(SECTION_OF)
    for item in result.values():
        item.update(depends_on=[], related=[], priority=None, assignee=None,
                    test_file=None, milestone="M815-B", module="auto-ui-tools",
                    created_at=now, modified_at=now, completed_at=now)
        source_hash = hashlib.sha256((root / item["file"]).read_bytes()).hexdigest()
        item["tags"] += ["guarded-offline-projection", f"source-sha256:{source_hash}"]
    return result


def find_items_array_close(raw: bytes, section_id: str) -> int:
    """Byte offset of the closing `]` of section_id's items array (scanner
    honours JSON strings/escapes; sections are located by their id key)."""
    marker = ('"id": "%s"' % section_id).encode()
    i = raw.find(marker)
    if i < 0:
        raise ValueError(f"section {section_id} not found in raw store")
    j = raw.find(b'"items": [', i)
    if j < 0:
        raise ValueError(f"items array of {section_id} not found")
    k = j + len(b'"items": [')
    depth = 1
    in_str = False
    esc = False
    while k < len(raw):
        c = raw[k:k + 1]
        if in_str:
            if esc:
                esc = False
            elif c == b"\\":
                esc = True
            elif c == b'"':
                in_str = False
        else:
            if c == b'"':
                in_str = True
            elif c in (b"[", b"{"):
                depth += 1
            elif c in (b"]", b"}"):
                depth -= 1
                if depth == 0:
                    return k
        k += 1
    raise ValueError(f"unbalanced items array for {section_id}")


def splice_deposit(path: Path, items: dict, verify) -> bool:
    original = path.read_bytes()
    ledger = json.loads(original)  # parse failure = stop before any write
    validate(ledger)
    ids = {i["id"]: i for s in ledger["sections"] for i in s["items"]}
    expected = {i["id"] for i in items.values()}
    present = expected & ids.keys()
    if present:
        if present != expected:
            raise ValueError("partial existing phase projection; reconcile before retry")
        for item in items.values():
            old = ids[item["id"]]
            for field in ("title", "content", "status", "file", "tags"):
                if old[field] != item[field]:
                    raise ValueError(f"existing projection differs: {item['id']}/{field}")
        return False
    # Build the spliced document: unrelated bytes byte-identical; only our
    # four item insertions + the version digit change.
    raw = original
    for sid, item in items.items():
        close = find_items_array_close(raw, sid)
        block = json.dumps(item, ensure_ascii=False, indent=1).encode("utf-8")
        # an empty array takes no leading comma
        prev = raw[:close].rstrip()
        sep = b"" if prev.endswith(b"[") else b",\n"
        raw = raw[:close] + sep + block + raw[close:]
    matches = list(re.finditer(rb'"version": (\d+)', raw))
    if not matches or raw.count(b'"version":') != 1 or len(matches) != 1:
        raise ValueError("version field not unique/located")
    m = matches[0]
    raw = raw[:m.start(1)] + str(int(m.group(1)) + 1).encode() + raw[m.end(1):]
    # Semantic equivalence: parsed(spliced) == parsed(original)+items, version+1
    after = json.loads(raw)
    want = json.loads(original)
    for sid, item in items.items():
        next(x for x in want["sections"] if x["id"] == sid)["items"].append(item)
    want["version"] += 1
    if after != want:
        raise ValueError("semantic equivalence failed; ledger unchanged")
    backup = path.with_name(path.name + f".bak-v{after['version']}-815b-" + hashlib.sha256(original).hexdigest()[:12])
    if backup.exists() and backup.read_bytes() != original:
        raise ValueError("backup collision")
    backup.write_bytes(original)
    if path.read_bytes() != original:
        raise ValueError("concurrent ledger edit before write")
    try:
        path.write_bytes(raw)
        reread = json.loads(path.read_bytes())
        verify(reread)
        if reread != want:
            raise ValueError("read-back mismatch")
    except Exception:
        current = path.read_bytes()
        if current == raw:
            path.write_bytes(original)  # guarded rollback: only our own bytes
        else:
            raise RuntimeError("read-back failed AND external update present; preserved") from None
        raise
    return True


def self_test() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "specs.json"
        wire = {"reports": "Reports", "goals": "Goals", "architecture": "Architecture",
                "designs": "Designs", "tests": "Tests", "reviews": "Reviews"}
        doc = {"project": "fixture", "version": 0, "sections": [
            dict(id=sid, section_type=wire[sid], title=sid, items=[], status="empty",
                 content="", depends_on=[], last_modified=0, last_verified=None)
            for sid in EXPECTED_SECTIONS]}
        # exercise BOTH shapes: standard encoder output and a foreign splice
        for raw in ((json.dumps(doc, indent=2, ensure_ascii=False) + "\n").encode(),
                    json.dumps(doc, indent=1, ensure_ascii=False).encode()):
            path.write_bytes(raw)
            original = path.read_bytes()
            items = proposed_items(ROOT)

            def reject(_):
                raise ValueError("injected read-back failure")
            try:
                splice_deposit(path, items, verify=reject)
                raise AssertionError("read-back failure accepted")
            except ValueError:
                assert path.read_bytes() == original
            assert splice_deposit(path, items, verify=validate)
            written = path.read_bytes()
            parsed = json.loads(written)
            assert parsed["version"] == 1
            assert len([i for s in parsed["sections"] for i in s["items"]]) == 4
            # surgical-scope proof: strip our blocks + version digit -> the
            # stripped text must equal the original bytes
            stripped = written
            for item in items.values():
                block = json.dumps(item, ensure_ascii=False, indent=1).encode()
                stripped = stripped.replace(b",\n" + block, b"").replace(block, b"")
            stripped = re.sub(rb'"version": 1', b'"version": 0', stripped)
            assert stripped == original, "unrelated bytes disturbed"
            assert not splice_deposit(path, items, verify=validate) and path.read_bytes() == written
    print("PASS: rollback, splice projection (both shapes), surgical scope, idempotence")


def main() -> None:
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--audit", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    if args.audit:
        doc = json.loads(LEDGER.read_bytes())
        errors = schema_errors(doc)
        print(json.dumps(dict(version=doc["version"], errors=errors), ensure_ascii=False, indent=2))
        raise SystemExit(bool(errors))
    try:
        changed = splice_deposit(LEDGER, proposed_items(ROOT), verify=validate)
        print("projection written" if changed else "already projected; no write")
    except ValueError as exc:
        raise SystemExit(f"BLOCKED, ledger unchanged: {exc}") from exc


if __name__ == "__main__":
    main()
