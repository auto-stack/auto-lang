#!/usr/bin/env python3
"""PLAN-832 final ledger projection (guarded offline channel).

Four delivery entries (design/tests/reviews/reports), following the store
schema: byte-exact backup, surgical insertion, version bump by one,
whole-document validation, exact read-back and rollback.

Channel notes:
- No store service is reachable (HTTP probe negative) -> guarded offline
  projection, same channel as earlier repository deliveries.
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

DELIVERY = "1b9f70dab0109dcc405ea66e30d01cddcacbef88"
FREEZE = "1b9f70dab"
REVIEWED = "adb5ce0f3ba8935f3f546b4bc40217c4d20235e9"
MAN_DELIVERY = "ceb8dfdd469b2dec1f2490aae1126b0b3303e35e"

DESIGN = {
    "id": "P832-D1",
    "title": "Man 独立仓与三仓 Core/full 交付（PLAN-832，SD-01..06）",
    "content": (
        "auto CLI、管理 runtime/core、包锁/resolver/toolchain/assets 与 I-TOOL client 归 auto-man；"
        "语言 SDK/AST/Value、I-PROJECT/I-TOOL 协议和 std-only owned 执行设施留 auto-lang，UI 实现归 auto-ui。"
        "Core 以独立 core workspace/lock/target 复用同一 CLI 源，仅需 Man/lang，无 UI 源码或工具；"
        "full 组合固定四仓源与真实 auto-ui-tool。开发发现按显式 env、Cargo 锚和 current_exe 布局解析 owner，"
        "安装态邻接；不搜索任意主检出。安装经 staging/hash/source/profile 校验与受管交换，"
        "仅确认死亡 owner 可清理 staging；真实 backup 恢复保存 owned marker 后严格 verify，陌生文件仍拒绝。"
        "canonical: docs/specs/repository-boundaries/man-three-repository-delivery.md 及 SD-01..06 目标。"
    ),
    "status": "verified",
    "file": "docs/specs/repository-boundaries/man-three-repository-delivery.md",
    "tags": ["PLAN-832", f"delivery:{DELIVERY}", f"man-delivery:{MAN_DELIVERY}", f"freeze:{FREEZE}"],
}

TESTS = {
    "id": "P832-T",
    "title": "三仓 consumer、独立 Core 与发行恢复测试资产（PLAN-832）",
    "content": (
        "three-repository-consumer 与 fixture 驱动管理+SDK+协议单源；独立 Core workspace 和 man-tcore "
        "覆盖 18 个 bin+5 个非 UI driver；跨仓 man-ti 分别预建 UI tool/lang te-child。"
        "man-install/man-distribution 覆盖无 UI/down fresh/locked/重定位、full 安装、真实升级备份恢复、"
        "未知 owner staging、严格额外文件和缺工具负控。永久 gen 测试不再由父进程注入工具路径。"
        "R3 终审 F01..F04 全关闭；merge 统一门结果与原始收据归档于 docs/plans/attachments/832/merge。"
    ),
    "status": "verified",
    "file": "docs/plans/attachments/832/ci-entrypoints.md",
    "tags": ["PLAN-832", f"delivery:{DELIVERY}", f"man-delivery:{MAN_DELIVERY}", "tier:merge"],
}

REVIEWS = {
    "id": "P832-R4",
    "title": "PLAN-832 独立终审 R3+R4 — pass（F01..F04 与 merge blueprint 全关闭）",
    "content": (
        "R3 绑定 lang e320abf4f、Man f52cd93b，F01..F04 全关闭。merge 门禁另发现 Man 占位 blueprint"
        "遮蔽与安装来源回退缺口；最终修复按显式 env/真实项目库→安装 SDK→开发 owner，"
        "安装 SDK 唯一 marker 被真实 bp show 读取，项目库/env 优先和缺件负控保持。"
        f"R4 独立受影响复审绑定 lang {REVIEWED}、Man {MAN_DELIVERY}，F832-MERGE-BP-01 CLOSED；"
        "jet/ark 真实构建按 r3 裁定转 v0.7，统一 merge 门另验。"
    ),
    "status": "verified",
    "file": "docs/plans/attachments/832/review-r4/review.md",
    "tags": ["PLAN-832", "review:R4-pass", f"reviewed:{REVIEWED}", f"man-reviewed:{MAN_DELIVERY}"],
}

REPORTS = {
    "id": "P832-M",
    "title": "PLAN-832 三仓交付与 merge 收据",
    "content": (
        f"lang delivery {DELIVERY}（28 个提交 rebase range-diff 全等价，ff-only）；"
        f"Man delivery {MAN_DELIVERY}（12 个提交 ff-only）；UI/down 固定版本不变。"
        "T-11 统一门、三仓来源读回、canonical SD-01..06、受管 ledger/索引、归档与 guard 清理"
        "按归档 Plan 的 merge 收据逐项核验。jet/ark 构建延期至 v0.7，不冒称已测；"
        "批量 tf 仍按主检收到期规则与既有 blocker 分诊。"
    ),
    "status": "verified",
    "file": "docs/plans/attachments/832/handoff.md",
    "tags": ["PLAN-832", f"delivery:{DELIVERY}", f"man-delivery:{MAN_DELIVERY}", "receipt:merge"],
}

SECTION_OF = {"designs": DESIGN, "tests": TESTS, "reviews": REVIEWS, "reports": REPORTS}
# actual ledger section order (validated on read-back; schema drift = abort)
EXPECTED_SECTIONS = ["reports", "goals", "architecture", "designs", "tests", "reviews"]

OWN_IDS = {"P832-D1", "P832-T", "P832-R3", "P832-M"}

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
            required = ITEM_FIELDS if iid in OWN_IDS else {"id", "title", "content", "status"}
            missing = required - item.keys()
            if missing:
                problems.append(f"missing {sorted(missing)}")
            if item.get("status") not in VALID_STATUSES:
                problems.append(f"invalid status {item.get('status')!r}")
            for field in ("id", "title", "content"):
                if not isinstance(item.get(field), str):
                    problems.append(f"invalid {field}")
            for field in ("depends_on", "related", "tags"):
                if field in item and (not isinstance(item[field], list) or any(not isinstance(x, str) for x in item[field])):
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
                    test_file=None, milestone="M1", module="repository-boundaries",
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
    backup = path.with_name(path.name + f".bak-v{after['version']}-832-" + hashlib.sha256(original).hexdigest()[:12])
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
