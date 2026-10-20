#!/usr/bin/env python3
"""PLAN-815 M815-A staged ledger projection (guarded offline channel).

Phase increment only (no whole-plan delivery entries), following the
store schema: byte-exact backup, four surgical items, version bump by
one, whole-document schema validation, exact read-back and rollback.
A legacy or invalid store must be restored separately before projection.
"""

from __future__ import annotations

import json
import hashlib
import time
import copy
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LEDGER = ROOT / ".autoos/specs.json"

DELIVERY = "e609caf1af511b96832f511ab9d1fa6c0c11f563"  # M815-A staged master landing (ancestor-verified)
REVIEWED = "7045e20fa71ba437bc80075f17d9aace755f5abd"  # R4 stage pass commit

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
    "status": "verified",
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
    "status": "verified",
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
    "status": "verified",
    "file": "docs/plans/attachments/815/m815-a-release.md",
    "tags": ["PLAN-815", "stage:M815-A", f"delivery:{DELIVERY}", "receipt:release"],
}

SECTION_OF = {"designs": DESIGN, "tests": TESTS, "reviews": REVIEWS, "reports": REPORTS}
# actual ledger section order (validated on read-back; schema drift = abort)
EXPECTED_SECTIONS = ["reports", "goals", "architecture", "designs", "tests", "reviews"]


VALID_STATUSES = set("empty proposed draft under_review approved in_progress in_implementation implemented verified done archived rejected backlog ready in_review blocked superseded outdated stable deprecated published analysed obsolete".split())
ITEM_FIELDS = set("id title content status depends_on related priority assignee test_file file milestone module tags created_at modified_at completed_at".split())
SECTION_FIELDS = set("id section_type title items status content depends_on last_modified last_verified".split())


def is_u64(value) -> bool:
    return type(value) is int and 0 <= value <= 2**64 - 1


def schema_errors(doc: dict) -> list[str]:
    """Mirror required store fields; preserve compatible unknown fields."""
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
            missing = ITEM_FIELDS - item.keys()
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
            for field in ("created_at", "modified_at"):
                if not is_u64(item.get(field)):
                    problems.append(f"invalid {field}")
            for field in ("priority", "assignee", "test_file", "file", "milestone", "module"):
                if item.get(field) is not None and not isinstance(item[field], str):
                    problems.append(f"invalid {field}")
            if item.get("completed_at") is not None and not is_u64(item["completed_at"]):
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
                    test_file=None, milestone="M815-A", module="auto-ui-tools",
                    created_at=now, modified_at=now, completed_at=now)
        source_hash = hashlib.sha256((root / item["file"]).read_bytes()).hexdigest()
        item["tags"] += ["guarded-offline-projection", f"source-sha256:{source_hash}"]
    return result


def encode(doc: dict) -> bytes:
    return (json.dumps(doc, ensure_ascii=False, indent=2) + "\n").encode("utf-8")


def deposit(path: Path, items: dict, verify=validate) -> bool:
    original = path.read_bytes()
    ledger = json.loads(original)
    # A parseable JSON object is insufficient: reject a store load failure
    # before creating/overwriting any backup or modifying any ledger byte.
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
    for sid, item in items.items():
        next(s for s in ledger["sections"] if s["id"] == sid)["items"].append(item)
    ledger["version"] += 1
    validate(ledger)
    stripped = copy.deepcopy(ledger)
    for section in stripped["sections"]:
        section["items"] = [i for i in section["items"] if i["id"] not in expected]
    stripped["version"] -= 1
    if encode(stripped) != original:
        raise ValueError("unrelated bytes or original formatting would change")
    backup = path.with_name(path.name + f".bak-v{ledger['version']}-815a-" + hashlib.sha256(original).hexdigest()[:12])
    if backup.exists() and backup.read_bytes() != original:
        raise ValueError("backup collision")
    backup.write_bytes(original)
    if path.read_bytes() != original:
        raise ValueError("concurrent ledger edit before write")
    wrote = encode(ledger)
    try:
        path.write_bytes(wrote)
        reread = json.loads(path.read_bytes())
        verify(reread)
        if reread != ledger:
            raise ValueError("read-back mismatch")
    except Exception:
        # guarded rollback (review L1 F-02): restore the original bytes
        # ONLY if the current content is exactly what this run wrote — an
        # external update is preserved, never overwritten
        current = path.read_bytes()
        if current == wrote:
            path.write_bytes(original)
        else:
            raise RuntimeError(
                "read-back failed AND the file changed beyond this run's write; "
                "external update preserved (no rollback)"
            ) from None
        raise
    return True


def self_test() -> None:
    # Tests protect the failure paths which previously allowed corrupt writes.
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "specs.json"
        wire = {"reports": "Reports", "goals": "Goals", "architecture": "Architecture",
                "designs": "Designs", "tests": "Tests", "reviews": "Reviews"}
        doc = {"project": "fixture", "version": 0, "sections": [
            dict(id=sid, section_type=wire[sid], title=sid, items=[], status="empty",
                 content="", depends_on=[], last_modified=0, last_verified=None)
            for sid in EXPECTED_SECTIONS]}
        items = proposed_items(ROOT)
        path.write_bytes(encode(doc))
        original = path.read_bytes()
        def reject(_):
            raise ValueError("injected read-back failure")
        try:
            deposit(path, items, verify=reject)
            raise AssertionError("read-back failure accepted")
        except ValueError:
            assert path.read_bytes() == original
        assert deposit(path, items)
        written = path.read_bytes()
        assert not deposit(path, items) and path.read_bytes() == written
        bad = json.loads(written)
        bad["sections"][0]["items"][0]["status"] = "active"
        path.write_bytes(encode(bad))
        rejected = path.read_bytes()
        try:
            deposit(path, items)
            raise AssertionError("invalid status accepted")
        except ValueError:
            assert path.read_bytes() == rejected
        bad["sections"][0]["items"][0]["status"] = "verified"
        bad["sections"][1]["items"].append(copy.deepcopy(bad["sections"][0]["items"][0]))
        assert any("duplicate id" in e for e in schema_errors(bad))
        for field, value in (("last_modified", 2**64), ("title", 7), ("depends_on", [1])):
            bad = copy.deepcopy(doc)
            bad["sections"][0][field] = value
            assert schema_errors(bad), field
        bad = copy.deepcopy(doc)
        bad["version"] = True
        assert schema_errors(bad)
    print("PASS: rollback, valid projection, idempotence, invalid-schema zero-write, duplicate-ID rejection")


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
        changed = deposit(LEDGER, proposed_items(ROOT))
        print("projection written" if changed else "already projected; no write")
    except ValueError as exc:
        raise SystemExit(f"BLOCKED, ledger unchanged: {exc}") from exc


if __name__ == "__main__":
    main()
