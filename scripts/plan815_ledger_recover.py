#!/usr/bin/env python3
"""PLAN-815 M815-L T-15: bounded ledger compatibility recovery tool.

Store schema mirror of auto-musk `backend/crates/musk/src/specs.rs`
(SpecsDocument / SpecsSection / SpecItem / SpecStatus — read-only source of
truth; SHA256 recorded by every report). Default is DRY-RUN: `audit` and
`convert` never touch the production ledger; `apply` is the T-17-only
single-writer path and refuses to run unless T-16-bound hashes match AND a
real typed-loader probe verifies document identity after each checkpoint.

Two-checkpoint pipeline (review L1 F-05): checkpoint A = structural
recovery (+ ruled ID rename), version source+1; checkpoint B = the four
M815-A stage increments, version source+2. Every conversion row names its
checkpoint and carries the TRUE original value (missing vs null
distinguished); the source document is never mutated.

Concurrency contract (review L1 F-02): every write happens in a CAS
window — if the target changed since our last read, an external writer
wins and we stop without writing; rollback restores the original bytes
ONLY from content this run wrote, never over an external update.

Exit codes (review L1 F-01): audit=0; convert 0=READY / 3=BLOCKED;
apply 0=applied / 1=failed (rolled back) / 2=refused (pre-checks) /
4=external-update conflict (external content preserved).

Conversion philosophy (plan r3 §5.6): restore a *loadable projection* —
never rewrite knowledge. Values that exist are preserved verbatim or
moved between legacy/store names with the original recorded. Nothing
invents completion times or acceptance verdicts: statuses without a
proposed mapping and duplicate IDs are BLOCKED until T-16 rules. Unknown
extra fields go to a sidecar — the real store (serde, no
deny_unknown_fields) silently drops them on a load/save round-trip, which
the report states explicitly.
"""

from __future__ import annotations

import argparse
import copy
import datetime as _dt
import hashlib
import json
import shutil
import sys
import tempfile
from pathlib import Path

SCHEMA_SHA256_FILE = "D:/autostack/auto-musk/backend/crates/musk/src/specs.rs"

SPEC_STATUSES = {
    "empty", "proposed", "draft", "under_review", "approved", "in_progress",
    "in_implementation", "implemented", "verified", "done", "archived",
    "rejected", "backlog", "ready", "in_review", "blocked", "superseded",
    "outdated", "stable", "deprecated", "published", "analysed", "obsolete",
}
# SectionType serde wire format is PASCAL CASE (no rename_all on the enum
# derive — verified against the real loader: snake_case values fall into
# #[serde(other)] Unknown and are silently filtered by SpecsStore::load,
# "successfully" returning an empty document). Section *ids* stay snake.
SECTION_TYPES = {
    "goals": "Goals", "architecture": "Architecture", "designs": "Designs",
    "tests": "Tests", "reviews": "Reviews", "reports": "Reports",
}
# frozen ledger section identity/order (matches the live document and the
# deposit script's EXPECTED_SECTIONS)
SECTION_ORDER = ["reports", "goals", "architecture", "designs", "tests", "reviews"]
ITEM_STR = ("id", "title", "content")
ITEM_OPT_STR = ("priority", "assignee", "test_file", "file", "milestone", "module")
ITEM_LIST = ("depends_on", "related", "tags")
ITEM_U64 = ("created_at", "modified_at")

# --- proposed status representations (T-15 candidates, T-16 confirms) ------
STATUS_MAP = {
    "active": ("stable", "旧账本语义=现行有效；生命周期集合中最近似且不构成验收主张"),
    "current": ("stable", "同 active（现行有效）"),
    "delivered": ("done", "旧值本身即交付主张，映射为等强的 done，不新增主张"),
    "accepted": ("approved", "旧值本身即接受主张，映射为等强的 approved，不新增主张"),
}
STATUS_MISSING = ("empty", "store 工厂默认（SpecItem::new → Empty）；缺状态=无记录，非已验收")
TIMESTAMP_ZERO_BASIS = "历史未记录，0=未知哨兵（不补造时间）"
TIMESTAMP_MOVED_BASIS = "旧字段名搬移至 store 字段，值经解析逐字保留（ISO 日期→UTC 纪元秒，日精度无损）"
MISSING = "<missing>"  # sentinel distinguishing an absent key from a null value


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def parse_legacy_time(value):
    """Legacy `created`/`last_modified` values are ISO date strings or null."""
    if value is None:
        return 0, TIMESTAMP_ZERO_BASIS
    if type(value) is int:
        return value, TIMESTAMP_MOVED_BASIS
    if isinstance(value, str):
        try:
            day = _dt.datetime.fromisoformat(value.strip()).replace(tzinfo=_dt.timezone.utc)
            return int(day.timestamp()), TIMESTAMP_MOVED_BASIS
        except ValueError:
            return 0, "旧时间值不可解析，0=未知哨兵（原值保留于转换表）"
    return 0, "旧时间值类型不支持，0=未知哨兵（原值保留于转换表）"


def coerce_list(value, field: str, basis: str):
    """Legacy containers sometimes appear as null or a stringified list."""
    if value is None:
        return [], basis
    if isinstance(value, list):
        return value, None  # already fine
    if isinstance(value, str):
        text = value.strip()
        if text.startswith("[") and text.endswith("]"):
            try:
                parsed = json.loads(text.replace("'", '"'))
                if isinstance(parsed, list):
                    return [str(v) for v in parsed], basis
            except json.JSONDecodeError:
                pass
        return [text], basis
    return [], basis


def _is_u64(value) -> bool:
    """True u64 per serde: non-bool int in 0..=2^64-1 (review L1 F-04:
    version=-1 and bools used to pass a bare int check)."""
    return type(value) is int and 0 <= value <= 2**64 - 1


def _str_list(value) -> bool:
    return isinstance(value, list) and all(isinstance(x, str) for x in value)


def schema_errors(doc) -> list[str]:
    """Mirror of the store's serde load requirements (V-L1 oracle) with
    full element/range checks, plus the recovery contract's six-section
    identity/order (stricter than the store — documented as ours)."""
    errors: list[str] = []
    if not isinstance(doc, dict) or not isinstance(doc.get("project"), str):
        errors.append("root: project must be a string")
    if not _is_u64(doc.get("version")):
        errors.append("root: version must be u64 (0..=2^64-1, non-bool)")
    sections = doc.get("sections")
    if not isinstance(sections, list):
        return errors + ["root: sections must be a list"]
    seen_ids = []
    for s_ in sections:
        sid = s_.get("id", "?") if isinstance(s_, dict) else "?"
        if not isinstance(s_, dict):
            errors.append(f"section {sid}: not an object")
            continue
        seen_ids.append(sid)
        for field in ("id", "section_type", "title", "content"):
            if not isinstance(s_.get(field), str):
                errors.append(f"section {sid}: '{field}' must be a string")
        if s_.get("section_type") != SECTION_TYPES.get(sid):
            errors.append(f"section {sid}: section_type must be the PascalCase wire form of its id")
        if s_.get("status") not in SPEC_STATUSES:
            errors.append(f"section {sid}: invalid status")
        if not _str_list(s_.get("depends_on")):
            errors.append(f"section {sid}: depends_on must be a string list")
        if not _is_u64(s_.get("last_modified")):
            errors.append(f"section {sid}: 'last_modified' must be u64")
        if s_.get("last_verified") is not None and not _is_u64(s_.get("last_verified")):
            errors.append(f"section {sid}: last_verified must be u64 or null")
        for it in s_.get("items", []):
            iid = it.get("id", "?") if isinstance(it, dict) else "?"
            if not isinstance(it, dict):
                errors.append(f"item {iid}: not an object")
                continue
            for field in ITEM_STR:
                if not isinstance(it.get(field), str):
                    errors.append(f"item {iid}: '{field}' must be a string")
            for field in ITEM_OPT_STR:
                if it.get(field) is not None and not isinstance(it.get(field), str):
                    errors.append(f"item {iid}: '{field}' must be a string or null")
            for field in ITEM_LIST:
                if not _str_list(it.get(field)):
                    errors.append(f"item {iid}: '{field}' must be a string list")
            for field in ITEM_U64:
                if not _is_u64(it.get(field)):
                    errors.append(f"item {iid}: '{field}' must be u64")
            ca = it.get("completed_at")
            if ca is not None and not _is_u64(ca):
                errors.append(f"item {iid}: 'completed_at' must be u64 or null")
            if it.get("status") not in SPEC_STATUSES:
                errors.append(f"item {iid}: invalid status {it.get('status')!r}")
    if seen_ids != SECTION_ORDER:
        errors.append(f"sections: must be exactly {SECTION_ORDER} in order (got {seen_ids})")
    return errors


def audit(source: dict) -> dict:
    """Itemized defect census (everything the store would reject)."""
    findings = {"sections": [], "items": []}
    for s in source.get("sections", []):
        sid = s.get("id", "?")
        missing = [f for f in ("section_type", "title", "status", "content", "depends_on", "last_modified", "last_verified") if f not in s]
        if missing:
            findings["sections"].append({"section": sid, "missing": missing, "present": sorted(k for k in s if k not in ("items",))})
        for it in s.get("items", []):
            iid = it.get("id", "?")
            problems = []
            missing = [f for f in ITEM_STR + ITEM_LIST + ITEM_U64 if f not in it]
            missing += [f for f in ITEM_OPT_STR if f not in it]
            if "completed_at" not in it:
                missing.append("completed_at")
            if missing:
                problems.append({"missing": sorted(missing)})
            status = it.get("status", MISSING)
            if not (isinstance(status, str) and status in SPEC_STATUSES):
                problems.append({"invalid_status": status if isinstance(status, str) else repr(status)})
            for f in ITEM_LIST:
                if f in it and not isinstance(it[f], list):
                    problems.append({"wrong_type": {f: it[f]}})
            extra = sorted(k for k in it if k not in set(ITEM_STR) | set(ITEM_OPT_STR) | set(ITEM_LIST) | set(ITEM_U64) | {"status", "completed_at"})
            if extra:
                problems.append({"unknown_fields": extra})
            if problems:
                findings["items"].append({"section": sid, "id": iid, "problems": problems})
    ids: dict[str, list[str]] = {}
    for s in source.get("sections", []):
        for it in s.get("items", []):
            ids.setdefault(it.get("id", "?"), []).append(s.get("id", "?"))
    findings["duplicate_ids"] = {k: v for k, v in ids.items() if len(v) > 1}
    return findings


def convert(
    source: dict,
    stage_increments: dict | None = None,
    ruling_rename: tuple[str, str, str] | None = None,
) -> tuple[dict, list[dict], dict, list[str]]:
    """Build the candidate projection. `source` is NEVER mutated (review L1
    F-03): the ruled rename happens on the deep copy and is recorded as an
    id conversion so protections compare the untouched original. Original
    values are captured BEFORE any normalization; missing vs null are
    distinguished via the MISSING sentinel."""
    candidate = copy.deepcopy(source)
    conversions: list[dict] = []
    unknown: dict[str, dict] = {}
    blocked: list[str] = []

    def record(section, iid, field, original, new, basis, note=""):
        conversions.append({
            "section": section, "id": iid, "field": field,
            "original": original, "new": new, "basis": basis, "note": note,
        })

    if ruling_rename:
        section, old_id, new_id = ruling_rename
        for s_ in candidate.get("sections", []):
            if s_.get("id") == section:
                for it_ in s_.get("items", []):
                    if it_.get("id") == old_id:
                        it_["id"] = new_id
                        record(section, new_id, "id", old_id, new_id,
                               "T-16 核准的重复 ID 裁定（--ruling）；原/新 ID 一并记录",
                               note="id rename")

    # --- sections ---
    section_titles = {"goals": "🎯 Goals", "architecture": "🏗️ Architecture",
                      "designs": "🎨 Designs", "tests": "🧪 Tests",
                      "reviews": "🔍 Reviews", "reports": "📊 Reports"}
    for s in candidate.get("sections", []):
        sid = s.get("id", "?")
        if "section_type" not in s:
            s["section_type"] = SECTION_TYPES[sid]
            record(sid, None, "section_type", MISSING, s["section_type"],
                   "SectionType::from_id 派生;线格式=PascalCase(真 loader 实测,snake 被 serde(other) 静默滤空)")
        if "title" not in s:
            s["title"] = section_titles[sid]
            record(sid, None, "title", MISSING, s["title"], "SpecsSection::new 工厂标题")
        if "status" not in s:
            s["status"] = "empty"
            record(sid, None, "status", MISSING, "empty", STATUS_MISSING[1])
        if "content" not in s:
            s["content"] = ""
            record(sid, None, "content", MISSING, "", "工厂默认（legacy 整节文本，未记录）")
        if "depends_on" not in s:
            s["depends_on"] = []
            record(sid, None, "depends_on", MISSING, [], "工厂默认")
        if "last_modified" not in s:
            s["last_modified"] = 0
            record(sid, None, "last_modified", MISSING, 0, TIMESTAMP_ZERO_BASIS)
        if "last_verified" not in s:
            s["last_verified"] = None
            record(sid, None, "last_verified", MISSING, None, "工厂默认")

    # --- items ---
    for s in candidate.get("sections", []):
        sid = s.get("id", "?")
        for it in s.get("items", []):
            iid = it.get("id", "?")
            status = it.get("status")
            # existence frozen BEFORE the assignment (review L2 F-L1-03:
            # evaluating `"status" not in it` after writing it recorded the
            # 213 truly-missing originals as null)
            status_was_missing = "status" not in it
            if status_was_missing or status is None:
                it["status"] = STATUS_MISSING[0]
                record(sid, iid, "status", MISSING if status_was_missing else None,
                       it["status"], STATUS_MISSING[1],
                       note="原值缺失" if status_was_missing else "原值显式 null=无记录，非已验收")
            elif not (isinstance(status, str) and status in SPEC_STATUSES):
                if isinstance(status, str) and status in STATUS_MAP:
                    mapped, basis = STATUS_MAP[status]
                    it["status"] = mapped
                    record(sid, iid, "status", status, mapped, basis)
                else:
                    blocked.append(f"{sid}/{iid}: 未知状态 {status!r} 无映射依据，待 T-16 裁定")
            # containers — original captured BEFORE any mutation
            for field in ITEM_LIST:
                if field not in it:
                    it[field] = []
                    record(sid, iid, field, MISSING, [], "容器缺失=空（无声明）")
                elif not isinstance(it[field], list):
                    original = it[field]
                    fixed, _ = coerce_list(original, field, "旧值规范化为列表（原值保留）")
                    it[field] = fixed
                    record(sid, iid, field, original, fixed, "旧值类型规范化为列表，值保留", note="stringified/null 容器")
            for field in ITEM_OPT_STR:
                if field not in it:
                    it[field] = None
                    record(sid, iid, field, MISSING, None, "Option 工厂默认")
            # timestamps: legacy names move with exact day-preserving parse
            created_key_existed = "created" in it
            legacy_created = it.pop("created", None)
            if "created_at" not in it:
                created_existed = False
                value, basis = parse_legacy_time(legacy_created)
                it["created_at"] = value
                if created_key_existed:
                    record(sid, iid, "created", legacy_created, f"removed→created_at={value}", TIMESTAMP_MOVED_BASIS)
                record(sid, iid, "created_at", legacy_created if legacy_created is not None else MISSING, value, basis,
                       note="旧字段 created 搬移" if legacy_created is not None else "无旧值")
            elif created_key_existed:
                # store field already present: the legacy name is a DUPLICATE
                # — removed to the sidecar WITH a declared conversion row,
                # whether its value was a real value or an explicit null
                # (review L2 F-L1-03: legacy removals must all be declared)
                unknown.setdefault(f"{sid}/{iid}", {})["created"] = legacy_created
                record(sid, iid, "created", legacy_created, "→sidecar",
                       "store 字段已存在，旧名副本删除；值逐字保留于 sidecar", note="unknown-field sidecar")
            modified_key_existed = "last_modified" in it
            legacy_modified = it.pop("last_modified", None)
            if "modified_at" not in it:
                fallback = legacy_modified if legacy_modified is not None else legacy_created
                value, basis = parse_legacy_time(fallback)
                it["modified_at"] = value
                if modified_key_existed:
                    record(sid, iid, "last_modified", legacy_modified, f"removed→modified_at={value}", TIMESTAMP_MOVED_BASIS)
                record(sid, iid, "modified_at", fallback if fallback is not None else MISSING, value, basis,
                       note="旧字段 last_modified/created 搬移" if fallback is not None else "无旧值")
            elif modified_key_existed:
                unknown.setdefault(f"{sid}/{iid}", {})["last_modified"] = legacy_modified
                record(sid, iid, "last_modified", legacy_modified, "→sidecar",
                       "store 字段已存在，旧名副本删除；值逐字保留于 sidecar", note="unknown-field sidecar")
            if "completed_at" not in it:
                it["completed_at"] = None
                record(sid, iid, "completed_at", MISSING, None, "不发明完成时间")
            if "content" not in it:
                it["content"] = ""
                record(sid, iid, "content", MISSING, "", "原即缺失（空=无内容），未补造")
            # remaining unknown extras → sidecar, WITH a conversion row
            # (the removal must be declared — review L1 F-03)
            known = set(ITEM_STR) | set(ITEM_OPT_STR) | set(ITEM_LIST) | set(ITEM_U64) | {"status", "completed_at"}
            for key in [k for k in it if k not in known]:
                value = it.pop(key)
                unknown.setdefault(f"{sid}/{iid}", {})[key] = value
                record(sid, iid, key, value, "→sidecar",
                       "store 无槽位且真实 load/save 会丢弃；逐字保留于 sidecar", note="unknown-field sidecar")

    # --- stage source increments: replace legacy 815 entries with the
    # closeout-authored records (field-level diffs recorded) ---------------
    if stage_increments:
        for sec_id, proposed in stage_increments.items():
            section = next((x for x in candidate.get("sections", []) if x.get("id") == sec_id), None)
            if section is None:
                blocked.append(f"{sec_id}: 阶段增量目标 section 不存在")
                continue
            existing = next((i for i in section.get("items", []) if i.get("id") == proposed.get("id")), None)
            if existing is None:
                section["items"].append(copy.deepcopy(proposed))
                record(sec_id, proposed["id"], "<record>", MISSING, "added",
                       "M815-A 阶段来源增量(收尾拟稿,R4 reviewed 7045e20fa / delivery e609caf1a)")
                continue
            for field, new_value in proposed.items():
                if existing.get(field, MISSING) != new_value:
                    record(sec_id, proposed["id"], field, existing.get(field, MISSING), new_value,
                           "M815-A 阶段来源增量(收尾拟稿,R4 reviewed 7045e20fa / delivery e609caf1a)")
                    existing[field] = new_value

    # --- duplicate ids: blocked, with a section-aware rename proposal -----
    seen: dict[str, list[str]] = {}
    for s in candidate.get("sections", []):
        for it in s.get("items", []):
            seen.setdefault(it.get("id", "?"), []).append(s.get("id", "?"))
    for iid, sections in seen.items():
        if len(sections) > 1 and not (ruling_rename and iid in (ruling_rename[1], ruling_rename[2])):
            blocked.append(
                f"{'/'.join(sections)}/{iid}: 重复 ID（{sections}）；"
                f"T-15 建议：保留 designs 原名、reviews 改名 {iid}-R（镜像 Pxxx-R 惯例），待 T-16 裁定"
            )
    return candidate, conversions, unknown, blocked


def build_checkpoints(
    source: dict,
    stage_increments: dict | None = None,
    ruling_rename: tuple[str, str, str] | None = None,
) -> dict:
    """Two-checkpoint pipeline (review L1 F-05, plan §5.6): A = structural
    recovery (incl. the ruled rename), version source+1; B = the four
    stage increments, version source+2. Each row names its checkpoint;
    protections run per checkpoint against the untouched original."""
    source_version = source.get("version", 0)
    cand_a, conv_a, unknown_a, blocked = convert(source, None, ruling_rename)
    cand_a["version"] = source_version + 1
    cand_b, conv_b, unknown_b, blocked_b = convert(cand_a, stage_increments, None)
    cand_b["version"] = source_version + 2
    for row in conv_a:
        row["checkpoint"] = "A-recovery"
    for row in conv_b:
        row["checkpoint"] = "B-stage-increments"
    return {
        "A": {"candidate": cand_a, "conversions": conv_a, "unknown": unknown_a, "blocked": blocked},
        "B": {"candidate": cand_b, "conversions": conv_a + conv_b,
              "unknown": {**unknown_a, **unknown_b}, "blocked": blocked + blocked_b},
    }


# ---------------------------------------------------------------------------
# protections (review L1 F-03: deletions/additions/rename-aware, original
# baseline, undeclared changes rejected)
# ---------------------------------------------------------------------------

def _iter_items(doc):
    for s in doc.get("sections", []):
        for it in s.get("items", []):
            yield s.get("id"), it.get("id")


def _iter_fields(doc):
    for s in doc.get("sections", []):
        for it in s.get("items", []):
            for key, value in it.items():
                yield s.get("id"), it.get("id"), key, value


def _lookup(doc, sid, iid):
    for s in doc.get("sections", []):
        if s.get("id") != sid:
            continue
        for it in s.get("items", []):
            if it.get("id") == iid:
                return it
    return None


def protections(source: dict, candidate: dict, conversions: list[dict]) -> list[str]:
    errors = []
    if source.get("project") != candidate.get("project"):
        errors.append("protection: root project 改变")
    if [x.get("id") for x in source.get("sections", [])] != [x.get("id") for x in candidate.get("sections", [])]:
        errors.append("protection: section 身份/顺序改变")
    # section-level declared-VALUE protection (review L2 F-L1-03: a mutated
    # reports.title passed because only ids were compared). Section
    # conversion rows carry id=None and live outside `declared` (which is
    # item-keyed), so compare them separately.
    section_rows = {(c["section"], c["field"]) for c in conversions if c.get("id") is None}
    # declared VALUE check: a section field produced by a conversion row
    # must equal the declared new value (mutating it to anything else is
    # an undeclared change — review L2 F-L1-03)
    section_declared = {}
    for c in conversions:
        if c.get("id") is None:
            section_declared[(c["section"], c["field"])] = c["new"]
    for s_src in source.get("sections", []):
        sid = s_src.get("id")
        s_dst = next((x for x in candidate.get("sections", []) if x.get("id") == sid), None)
        if s_dst is None:
            continue
        for key, value in s_src.items():
            if key == "items":
                continue
            if key not in s_dst:
                if (sid, key) not in section_rows:
                    errors.append(f"protection: section {sid}.{key} 被删除且无转换声明")
            elif s_dst[key] != value and (sid, key) not in section_rows:
                errors.append(f"protection: section {sid}.{key} 未经转换声明被改变")
        for key in s_dst:
            if key != "items" and key not in s_src and (sid, key) not in section_rows:
                errors.append(f"protection: section {sid}.{key} 新增且无转换声明")
        for (row_sid, row_field), declared_value in section_declared.items():
            if row_sid == sid and row_field in s_dst and s_dst[row_field] != declared_value:
                errors.append(
                    f"protection: section {sid}.{row_field} 与声明转换值不符"
                    f"（候选 {s_dst[row_field]!r} != 声明 {declared_value!r}）"
                )
    renames: dict[tuple[str, str], str] = {}  # section-scoped (review L1 F-03)
    declared: set[tuple[str, str, str]] = set()
    removed_keys: set[tuple[str, str, str]] = set()
    increment_items: set[tuple[str, str]] = set()
    for c in conversions:
        if c.get("id") is None:
            continue
        if c["field"] == "id" and c.get("note") == "id rename":
            renames[(c["section"], c["original"])] = c["new"]
            # the id-field change itself is declared under the NEW id
            declared.add((c["section"], c["new"], "id"))
            continue
        declared.add((c["section"], c["id"], c["field"]))
        if isinstance(c["new"], str) and (c["new"].startswith("removed→") or c["new"].startswith("→sidecar")):
            removed_keys.add((c["section"], c["id"], c["field"]))
        if c["field"] == "<record>" or "阶段来源增量" in c.get("basis", ""):
            increment_items.add((c["section"], c["id"]))

    def map_id(sid: str, iid: str) -> str:
        return renames.get((sid, iid), iid)

    ids_src = sorted(map_id(sid, iid) for sid, iid in _iter_items(source))
    ids_dst = sorted(i for _, i in _iter_items(candidate))
    if ids_src != ids_dst:
        errors.append("protection: 原始 ID 集合（含改名映射）改变")
    flat_dst = [i for _, i in _iter_items(candidate)]
    if len(flat_dst) != len(set(flat_dst)):
        errors.append("protection: 候选存在重复 ID")

    for sid, iid, key, value in _iter_fields(source):
        row = (sid, map_id(sid, iid), key)
        dst = _lookup(candidate, sid, map_id(sid, iid))
        if dst is None:
            errors.append(f"protection: {sid}/{iid} 在候选中消失")
            continue
        if key not in dst:
            if row not in removed_keys:
                errors.append(f"protection: {sid}/{iid}.{key} 被删除且无转换声明")
        elif dst[key] != value and row not in declared:
            errors.append(f"protection: {sid}/{iid}.{key} 未经转换声明被改变")
    for s in candidate.get("sections", []):
        for it in s.get("items", []):
            for key in it:
                row = (s.get("id"), it.get("id"))
                src = _lookup(source, s.get("id"), it.get("id"))
                # new keys are allowed only on increment records or declared rows
                if src is not None and key not in src and row not in increment_items \
                        and (row[0], row[1], key) not in declared:
                    errors.append(f"protection: {row[0]}/{row[1]}.{key} 新增且无转换声明")
    return errors


# ---------------------------------------------------------------------------
# commands
# ---------------------------------------------------------------------------

def cmd_audit(args) -> int:
    ledger = Path(args.ledger)
    raw = ledger.read_bytes()
    doc = json.loads(raw.decode("utf-8"))
    report = {
        "ledger": str(ledger),
        "source_sha256": sha256_bytes(raw),
        "schema_sha256": sha256_bytes(Path(SCHEMA_SHA256_FILE).read_bytes()),
        "version": doc.get("version"),
        "census": audit(doc),
    }
    payload = json.dumps(report, ensure_ascii=False, indent=2)
    _emit(args.out, "audit.json", payload + "\n")
    print(payload if not args.out else f"wrote {Path(args.out) / 'audit.json'}")
    return 0


def _emit(out_dir, name, text) -> None:
    if out_dir:
        Path(out_dir).mkdir(parents=True, exist_ok=True)
        (Path(out_dir) / name).write_text(text, encoding="utf-8", newline="\n")


def cmd_convert(args) -> int:
    """Dry-run (never writes the ledger). Exit 0 = READY; exit 3 = BLOCKED
    (report still produced — explicit semantics, review L1 F-01)."""
    ledger = Path(args.ledger)
    raw = ledger.read_bytes()
    source = json.loads(raw.decode("utf-8"))
    increments = _load_increments(args.stage_increments)
    ruling = _parse_ruling(args.ruling)
    cps = build_checkpoints(source, increments, ruling)
    problems: dict[str, list[str]] = {}
    guard: dict[str, list[str]] = {}
    for name, cp in cps.items():
        problems[name] = schema_errors(cp["candidate"])
        guard[name] = protections(source, cp["candidate"], cp["conversions"])
        encoded = json.dumps(cp["candidate"], ensure_ascii=False, indent=2) + "\n"
        reloaded = json.loads(encoded)
        # direct re-conversion of the already-recovered document must be
        # a no-op (version excluded: build_checkpoints would bump again)
        recp, _c2, _u2, _b2 = convert(reloaded, increments if name == "B" else None, None)
        recp["version"] = reloaded["version"]
        if schema_errors(recp) or json.dumps(recp, ensure_ascii=False, sort_keys=True) != json.dumps(reloaded, ensure_ascii=False, sort_keys=True):
            problems[name].append("round-trip: 候选再转换不稳定")
    blocked = cps["B"]["blocked"]
    status = "BLOCKED" if (blocked or any(problems.values()) or any(guard.values())) else "READY"
    a_bytes = (json.dumps(cps["A"]["candidate"], ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    b_bytes = (json.dumps(cps["B"]["candidate"], ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    conv_bytes = json.dumps(cps["B"]["conversions"], ensure_ascii=False, sort_keys=True).encode("utf-8")
    result = {
        "ledger": str(ledger),
        "source_sha256": sha256_bytes(raw),
        "schema_sha256": sha256_bytes(Path(SCHEMA_SHA256_FILE).read_bytes()),
        "source_version": source.get("version"),
        "checkpoint_a_version": cps["A"]["candidate"].get("version"),
        "checkpoint_b_version": cps["B"]["candidate"].get("version"),
        "candidate_a_sha256": sha256_bytes(a_bytes),
        "candidate_b_sha256": sha256_bytes(b_bytes),
        "conversions_sha256": sha256_bytes(conv_bytes),
        "conversion_count": len(cps["B"]["conversions"]),
        "checkpoint_a_conversions": len(cps["A"]["conversions"]),
        "unknown_field_items": len(cps["B"]["unknown"]),
        "blocked": blocked,
        "schema_errors": problems,
        "protection_errors": guard,
        "status": status,
    }
    if args.out:
        out = Path(args.out)
        out.mkdir(parents=True, exist_ok=True)
        (out / "candidate-a-recovery.json").write_bytes(a_bytes)
        (out / "candidate-b-final.json").write_bytes(b_bytes)
        (out / "conversions.json").write_text(
            json.dumps(cps["B"]["conversions"], ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8", newline="\n")
        (out / "unknown-fields-sidecar.json").write_text(
            json.dumps(cps["B"]["unknown"], ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8", newline="\n")
        (out / "convert-summary.json").write_text(
            json.dumps(result, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8", newline="\n")
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0 if status == "READY" else 3


class ExternalUpdate(RuntimeError):
    """Another writer touched the ledger inside our write window."""


class WriteFailed(RuntimeError):
    """OUR write failed mid-stream; `restored` says whether the exact
    prior content was recovered under the lock (review L2 F-L1-02)."""

    def __init__(self, message: str, restored: bool):
        super().__init__(message)
        self.restored = restored


def _probe_identity(probe: str, ledger: Path, expect: dict) -> None:
    """Run the REAL typed loader and verify document identity — exit code
    alone is not enough (review L1 F-04)."""
    import subprocess

    # .py probes are invoked through the interpreter (not directly
    # executable on Windows); real loaders are .exe and run as-is
    argv = [sys.executable, probe, str(ledger)] if probe.endswith(".py") else [probe, str(ledger)]
    result = subprocess.run(argv, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"真实 typed loader 拒绝: {result.stderr.strip()[:400]}")
    try:
        summary = json.loads(result.stdout.splitlines()[0])
    except (json.JSONDecodeError, IndexError) as exc:
        raise RuntimeError(f"loader 摘要不可解析: {exc}") from exc
    problems = []
    if summary.get("project") != expect["project"]:
        problems.append(f"project {summary.get('project')!r} != {expect['project']!r}")
    if summary.get("version") != expect["version"]:
        problems.append(f"version {summary.get('version')} != {expect['version']}")
    if summary.get("total_items") != expect["total_items"]:
        problems.append(f"total_items {summary.get('total_items')} != {expect['total_items']}")
    got_sections = [
        (x[0], x[1]) if isinstance(x, list) else (x.get("id"), x.get("section_type"))
        for x in summary.get("sections", [])
    ]
    if got_sections != expect["sections"]:
        problems.append(f"sections {got_sections} != {expect['sections']}")
    if problems:
        raise RuntimeError("loader 身份核验失败: " + "; ".join(problems))


def _ledger_lock(ledger: Path):
    """Cooperative single-writer lock for the write channel (review L1
    F-02): an exclusive byte-range lock on the ledger itself. All writes
    and guarded rollbacks in this tool happen while holding it, so two
    instances of this tool (or any lock-cooperating writer) serialize.
    Non-cooperating external edits are still caught by the in-lock CAS
    read that precedes every write."""
    import contextlib

    @contextlib.contextmanager
    def cm():
        handle = open(ledger, "r+b")
        try:
            try:
                import msvcrt
                handle.seek(0)
                msvcrt.locking(handle.fileno(), msvcrt.LK_NBLCK, 1)
            except ImportError:
                import fcntl
                fcntl.flock(handle.fileno(), fcntl.LOCK_EX)
            yield handle
        finally:
            try:
                try:
                    import msvcrt
                    handle.seek(0)
                    msvcrt.locking(handle.fileno(), msvcrt.LK_UNLCK, 1)
                except ImportError:
                    import fcntl
                    fcntl.flock(handle.fileno(), fcntl.LOCK_UN)
            finally:
                handle.close()

    return cm()


def _write_step(ledger: Path, payload: bytes, ours_before: bytes, label: str) -> None:
    """Single-writer CAS window under the ledger lock: refuse if the
    target changed since our last read (external writer wins); never
    clobber (review L1 F-02)."""
    with _ledger_lock(ledger) as handle:
        # read/write through the SAME handle that holds the lock (a second
        # open would collide with our own byte-range lock on Windows)
        handle.seek(0)
        current = handle.read()
        if current != ours_before:
            raise ExternalUpdate(f"{label}: 写入前发现外部更新（拒绝写入，外部内容保留）")
        handle.seek(0)
        try:
            handle.write(payload)
            handle.truncate()
            handle.flush()
        except OSError as exc:
            # OUR partial write failed mid-stream (review L2 F-L1-02): the
            # residue is ours, never "external" — restore the exact prior
            # content under the same lock, then report ownership honestly
            try:
                handle.seek(0)
                handle.write(ours_before)
                handle.truncate()
                handle.flush()
                raise WriteFailed(
                    f"{label}: 写入失败({exc})，已在本锁内恢复写入前内容", restored=True
                ) from exc
            except WriteFailed:
                raise
            except OSError as restore_exc:
                raise WriteFailed(
                    f"{label}: 写入失败({exc})且恢复失败({restore_exc})；"
                    f"文件含本运行部分写残留，需人工按备份恢复", restored=False
                ) from restore_exc


def cmd_apply(args) -> int:
    """T-17 single-writer recovery: two checkpoints, each written in a CAS
    window and verified by the REAL typed loader with identity checks; any
    failure rolls back to the exact original bytes — but ONLY from content
    this run wrote (external updates are never overwritten)."""
    if not args.loader_probe:
        print("REFUSED: --loader-probe 为必填（真实 typed 验证不可跳过）", file=sys.stderr)
        return 2
    ledger = Path(args.ledger)
    raw = ledger.read_bytes()
    source_sha = sha256_bytes(raw)
    if source_sha != args.expected_source_sha256:
        print(
            f"REFUSED: 源文件 hash 漂移（现 {source_sha[:12]}，绑定 {args.expected_source_sha256[:12]}）",
            file=sys.stderr,
        )
        return 2
    source = json.loads(raw.decode("utf-8"))
    increments = _load_increments(args.stage_increments)
    cps = build_checkpoints(source, increments, _parse_ruling(args.ruling))
    if cps["B"]["blocked"]:
        print("REFUSED: 存在待 T-16 裁定的阻断项", file=sys.stderr)
        return 2
    for name, cp in cps.items():
        problems = schema_errors(cp["candidate"]) or protections(source, cp["candidate"], cp["conversions"])
        if problems:
            print(f"REFUSED: checkpoint {name}: " + "; ".join(problems), file=sys.stderr)
            return 2
    a_bytes = (json.dumps(cps["A"]["candidate"], ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    b_bytes = (json.dumps(cps["B"]["candidate"], ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    if sha256_bytes(a_bytes) != args.expected_candidate_a_sha256:
        print("REFUSED: checkpoint-A 候选 hash 与绑定不符", file=sys.stderr)
        return 2
    if sha256_bytes(b_bytes) != args.expected_candidate_b_sha256:
        print("REFUSED: checkpoint-B 候选 hash 与绑定不符", file=sys.stderr)
        return 2
    if args.expected_conversions_sha256 and sha256_bytes(
        json.dumps(cps["B"]["conversions"], ensure_ascii=False, sort_keys=True).encode("utf-8")
    ) != args.expected_conversions_sha256:
        print("REFUSED: 转换表 hash 与绑定不符", file=sys.stderr)
        return 2

    def expect_identity(candidate: dict) -> dict:
        return {
            "project": candidate["project"],
            "version": candidate["version"],
            "total_items": sum(len(s["items"]) for s in candidate["sections"]),
            "sections": [(s["id"], SECTION_TYPES[s["id"]]) for s in candidate["sections"]],
        }

    backup = ledger.with_name(ledger.name + ".bak-recover-" + source_sha[:16])
    if backup.exists():
        if sha256_bytes(backup.read_bytes()) != source_sha:
            print(f"REFUSED: 备份 {backup.name} 已存在且字节不同（CAS，不覆盖）", file=sys.stderr)
            return 2
    else:
        backup.write_bytes(raw)

    def rollback_to_original(ours_residue: bool = False) -> str:
        """Restore original bytes ONLY from content this run wrote; an
        external update is preserved and reported (never clobbered). With
        `ours_residue` the un-restorable content is OUR partial write —
        reported as such, never mislabeled external."""
        current = ledger.read_bytes()
        if current == raw:
            return "rollback: 目标已是原件（无写入残留）"
        if current in (a_bytes, b_bytes):
            with _ledger_lock(ledger) as handle:
                # re-check under the lock: only restore what is still ours
                handle.seek(0)
                if handle.read() == current:
                    handle.seek(0)
                    handle.write(raw)
                    handle.truncate()
                    handle.flush()
                    return "rollback: 已恢复精确原件"
        if ours_residue:
            return ("rollback-failed: 目标含本运行部分写残留（恢复已失败），"
                    "外部内容未受影响；请按 CAS 备份人工恢复")
        return "rollback-skipped: 目标含外部更新（非本运行产物），外部内容已保留"
        return "rollback-skipped: 目标含外部更新（非本运行产物），外部内容已保留"

    wrote: list[str] = []
    try:
        _write_step(ledger, a_bytes, raw, "checkpoint-A")
        wrote.append("A")
        _probe_identity(args.loader_probe, ledger, expect_identity(cps["A"]["candidate"]))
        _write_step(ledger, b_bytes, a_bytes, "checkpoint-B")
        wrote.append("B")
        _probe_identity(args.loader_probe, ledger, expect_identity(cps["B"]["candidate"]))
    except WriteFailed as exc:
        # ownership is OURS (partial write): the in-lock restore already
        # ran; verify/re-attribute honestly — a failed restore reports OUR
        # residue, never "external"
        note = rollback_to_original(ours_residue=not exc.restored)
        print(f"FAILED(1)（本运行部分写）: {exc}; {note}", file=sys.stderr)
        return 1
    except ExternalUpdate as exc:
        note = rollback_to_original()
        print(f"CONFLICT(4)（外部更新优先，不覆盖）: {exc}; {note}", file=sys.stderr)
        return 4
    except Exception as exc:  # noqa: BLE001 — rollback covers every failure
        note = rollback_to_original()
        print(f"FAILED(1)（checkpoints {wrote}）: {exc}; {note}", file=sys.stderr)
        return 1
    after = sha256_bytes(ledger.read_bytes())
    if after != sha256_bytes(b_bytes):
        # we OBSERVED a post-verification fingerprint mismatch (review L2
        # F-L2-01): an external update happened after the final probe —
        # preserve it, report accurately, never emit a success receipt
        print(
            f"CONFLICT(4): 最终读回指纹与 checkpoint-B 不符（外部更新，内容已保留，未发布成功收据）",
            file=sys.stderr,
        )
        return 4
    print(json.dumps({
        "applied": True,
        "ledger": str(ledger),
        "source_sha256": source_sha,
        "checkpoint_a_sha256": sha256_bytes(a_bytes),
        "checkpoint_b_sha256": sha256_bytes(b_bytes),
        "final_version": cps["B"]["candidate"]["version"],
        "backup": backup.name,
        "after_sha256": after,
        "after_matches_checkpoint_b": True,
    }, ensure_ascii=False, indent=2))
    return 0


# ---------------------------------------------------------------------------
# self-test (V-L3): real CLI subprocesses for exit codes, external-writer
# injections for the CAS/rollback windows, schema negatives, deletion
# protection, rename recording, checkpoint versioning
# ---------------------------------------------------------------------------

def self_test() -> None:
    import subprocess
    import unittest.mock as mock

    tool = str(Path(__file__).resolve())
    py = sys.executable
    with tempfile.TemporaryDirectory() as td:
        td = Path(td)
        src = {"project": "fixture", "version": 3, "sections": [
            {"id": "reports", "items": [
                {"id": "X1", "title": "t", "content": "c", "status": "active",
                 "created": "2026-10-01", "note": "extra"},
                {"id": "X2", "title": "no-status", "content": "c"},
                {"id": "X3", "title": "null-status", "content": "c", "status": None,
                 "created_at": 1700000000, "created": "2026-01-05"},
            ]},
            {"id": "goals", "items": []}, {"id": "architecture", "items": []},
            {"id": "designs", "items": []}, {"id": "tests", "items": []},
            {"id": "reviews", "items": [
                {"id": "X1", "title": "dup", "content": "c", "status": "active"},
            ]},
        ]}
        ledger = td / "specs.json"
        ledger.write_text(json.dumps(src, ensure_ascii=False), encoding="utf-8")
        raw = ledger.read_bytes()

        def run(*argv):
            return subprocess.run([py, tool, *argv], capture_output=True, text=True)

        # F-01: BLOCKED convert → exit 3 (real CLI); dry-run never writes
        r = run("convert", "--ledger", str(ledger), "--out", str(td / "out"))
        assert r.returncode == 3, (r.returncode, r.stdout, r.stderr)
        assert ledger.read_bytes() == raw, "dry-run 不得改原件"

        # rename the duplicate via the ruling → READY (exit 0)
        ledger.write_text(json.dumps(src, ensure_ascii=False), encoding="utf-8")
        raw = ledger.read_bytes()
        r = run("convert", "--ledger", str(ledger), "--out", str(td / "out"),
                "--ruling", "reviews/X1=X1-R")
        assert r.returncode == 0, (r.returncode, r.stdout, r.stderr)
        summary = json.loads((td / "out" / "convert-summary.json").read_text(encoding="utf-8"))
        assert summary["status"] == "READY", summary
        # F-05: two checkpoints with version bumps 3→4→5
        assert summary["checkpoint_a_version"] == 4 and summary["checkpoint_b_version"] == 5
        # F-03: rename recorded; unknown field sidecarred; true originals
        convs = json.loads((td / "out" / "conversions.json").read_text(encoding="utf-8"))
        rename_rows = [c for c in convs if c["field"] == "id" and c.get("note") == "id rename"]
        assert rename_rows and rename_rows[0]["original"] == "X1" and rename_rows[0]["new"] == "X1-R"
        sidecar = json.loads((td / "out" / "unknown-fields-sidecar.json").read_text(encoding="utf-8"))
        assert sidecar.get("reports/X1", {}).get("note") == "extra"
        status_rows = [c for c in convs if c["field"] == "status" and c["id"] == "X1"]
        assert status_rows[0]["original"] == "active", status_rows[0]
        cand_a = json.loads((td / "out" / "candidate-a-recovery.json").read_text(encoding="utf-8"))
        assert cand_a["sections"][0]["items"][0]["created_at"] == 1790812800  # 2026-10-01 UTC
        assert cand_a["sections"][0]["items"][0]["status"] == "stable"
        assert cand_a["version"] == 4

        # F-03: undeclared deletion must fail protections
        victim = copy.deepcopy(cand_a)
        del victim["sections"][0]["items"][0]["title"]
        assert any("被删除" in e for e in protections(src, victim, build_checkpoints(src, None, ("reviews", "X1", "X1-R"))["A"]["conversions"]))

        # F-04: schema negatives the old mirror passed
        for mutate, want in [
            (lambda d: d.update(version=-1), "version"),
            (lambda d: d.update(version=True), "version"),
            (lambda d: d["sections"][0]["items"][0].update(tags=[7]), "tags"),
            (lambda d: d["sections"][0]["items"][0].update(created_at=-2), "created_at"),
            (lambda d: d["sections"].pop(), "sections"),
        ]:
            bad = copy.deepcopy(cand_a)
            mutate(bad)
            errs = schema_errors(bad)
            assert any(want in e for e in errs), (want, errs)

        # F-01: apply refusal propagates nonzero (real CLI)
        r = run("apply", "--ledger", str(ledger),
                "--expected-source-sha256", "0" * 64,
                "--expected-candidate-a-sha256", summary["candidate_a_sha256"],
                "--expected-candidate-b-sha256", summary["candidate_b_sha256"],
                "--loader-probe", py)
        assert r.returncode == 2 and ledger.read_bytes() == raw, (r.returncode, r.stderr)

        # F-02 between-checkpoints window: a "loader probe" that acts as an
        # external writer (writes v777, exits 0) after checkpoint A — the
        # checkpoint-B CAS must refuse (exit 4) and preserve the external
        # content byte-exactly (real CLI subprocess)
        evil_ok = td / "evil_ok_probe.py"
        evil_ok.write_text(
            "import sys,json,pathlib\n"
            "p=pathlib.Path(sys.argv[1]);d=json.loads(p.read_text(encoding='utf-8'))\n"
            "n=sum(len(s['items']) for s in d['sections'])\n"
            "T={'reports':'Reports','goals':'Goals','architecture':'Architecture','designs':'Designs','tests':'Tests','reviews':'Reviews'}\n"
            "sec=[(s['id'],T[s['id']]) for s in d['sections']]\n"
            "d['version']=777;p.write_text(json.dumps(d),encoding='utf-8')\n"
            "print(json.dumps({'project':'fixture','version':4,'total_items':n,'sections':sec}))\n", encoding="utf-8")
        r = run("apply", "--ledger", str(ledger),
                "--expected-source-sha256", summary["source_sha256"],
                "--expected-conversions-sha256", summary["conversions_sha256"],
                "--expected-candidate-a-sha256", summary["candidate_a_sha256"],
                "--expected-candidate-b-sha256", summary["candidate_b_sha256"],
                "--ruling", "reviews/X1=X1-R",
                "--loader-probe", str(evil_ok))
        assert r.returncode == 4, (r.returncode, r.stderr)
        after = json.loads(ledger.read_text(encoding="utf-8"))
        assert after["version"] == 777, "checkpoint 间外部写入必须保留"

        # F-02 post-write window: probe modifies the file (external wins)
        # then fails — rollback must NOT clobber the external content
        ledger.write_bytes(raw)
        evil_probe = td / "evil_probe.py"
        evil_probe.write_text(
            "import sys,json,pathlib\n"
            "p=pathlib.Path(sys.argv[1]);d=json.loads(p.read_text(encoding='utf-8'))\n"
            "d['version']=777;p.write_text(json.dumps(d),encoding='utf-8')\n"
            "sys.exit(1)\n", encoding="utf-8")
        r = run("apply", "--ledger", str(ledger),
                "--expected-source-sha256", summary["source_sha256"],
                "--expected-conversions-sha256", summary["conversions_sha256"],
                "--expected-candidate-a-sha256", summary["candidate_a_sha256"],
                "--expected-candidate-b-sha256", summary["candidate_b_sha256"],
                "--ruling", "reviews/X1=X1-R",
                "--loader-probe", str(evil_probe))
        assert r.returncode == 1, (r.returncode, r.stderr)
        after = json.loads(ledger.read_text(encoding="utf-8"))
        assert after["version"] == 777, "外部更新不得被回滚覆盖"

        # F-02: failing probe that does NOT touch the file → exact rollback
        ledger.write_bytes(raw)
        fail_probe = td / "fail_probe.py"
        fail_probe.write_text("import sys; sys.exit(1)\n", encoding="utf-8")
        r = run("apply", "--ledger", str(ledger),
                "--expected-source-sha256", summary["source_sha256"],
                "--expected-conversions-sha256", summary["conversions_sha256"],
                "--expected-candidate-a-sha256", summary["candidate_a_sha256"],
                "--expected-candidate-b-sha256", summary["candidate_b_sha256"],
                "--ruling", "reviews/X1=X1-R",
                "--loader-probe", str(fail_probe))
        assert r.returncode == 1 and ledger.read_bytes() == raw, (r.returncode, r.stderr)

        # happy path with a PASSING identity probe (checks version/items/sections)
        good_probe = td / "good_probe.py"
        good_probe.write_text(
            "import sys, json, pathlib\n"
            "doc = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding='utf-8'))\n"
            "n = sum(len(s['items']) for s in doc['sections'])\n"
            "assert doc['project'] == 'fixture' and n == 4 and len(doc['sections']) == 6, doc\n"
            "print(json.dumps({'project': doc['project'], 'version': doc['version'],\n"
            "  'total_items': n, 'sections': [(s['id'], {'reports':'Reports','goals':'Goals',\n"
            "  'architecture':'Architecture','designs':'Designs','tests':'Tests',\n"
            "  'reviews':'Reviews'}[s['id']]) for s in doc['sections']]}))\n", encoding="utf-8")
        r = run("apply", "--ledger", str(ledger),
                "--expected-source-sha256", summary["source_sha256"],
                "--expected-conversions-sha256", summary["conversions_sha256"],
                "--expected-candidate-a-sha256", summary["candidate_a_sha256"],
                "--expected-candidate-b-sha256", summary["candidate_b_sha256"],
                "--ruling", "reviews/X1=X1-R",
                "--loader-probe", str(good_probe))
        assert r.returncode == 0, (r.returncode, r.stderr)
        final = json.loads(ledger.read_text(encoding="utf-8"))
        assert final["version"] == 5, final["version"]  # 3 → 4 (A) → 5 (B)
        backup = next(td.glob("specs.json.bak-recover-*"))
        assert sha256_bytes(backup.read_bytes()) == summary["source_sha256"]

        # re-apply: source drifted → REFUSED 2; CAS backup untouched
        r = run("apply", "--ledger", str(ledger),
                "--expected-source-sha256", summary["source_sha256"],
                "--expected-candidate-a-sha256", summary["candidate_a_sha256"],
                "--expected-candidate-b-sha256", summary["candidate_b_sha256"],
                "--loader-probe", str(good_probe))
        assert r.returncode == 2
        assert sha256_bytes(backup.read_bytes()) == summary["source_sha256"]

        # ---- review L2 regressions ------------------------------------
        convs = json.loads((td / "out" / "conversions.json").read_text(encoding="utf-8"))
        # status three-state originals (F-L1-03): missing vs null vs value
        x2 = [c for c in convs if c["id"] == "X2" and c["field"] == "status"]
        x3 = [c for c in convs if c["id"] == "X3" and c["field"] == "status"]
        x1 = [c for c in convs if c["id"] == "X1" and c["field"] == "status"]
        assert x2 and x2[0]["original"] == "<missing>", x2
        assert x3 and x3[0]["original"] is None, x3
        assert x1 and x1[0]["original"] == "active", x1
        # legacy duplicate WITH store field present: declared removal row
        dup = [c for c in convs if c["id"] == "X3" and c["field"] == "created"]
        assert dup and dup[0]["new"] == chr(8594) + "sidecar" and dup[0]["original"] == "2026-01-05", dup
        sidecar2 = json.loads((td / "out" / "unknown-fields-sidecar.json").read_text(encoding="utf-8"))
        assert sidecar2.get("reports/X3", {}).get("created") == "2026-01-05"
        # section-value mutation must fail protections (F-L1-03)
        bad_sec = copy.deepcopy(cand_a)
        bad_sec["sections"][0]["title"] = "mutated"
        assert any("reports.title" in e for e in protections(src, bad_sec, build_checkpoints(src, None, ("reviews", "X1", "X1-R"))["A"]["conversions"]))
        # legacy-name deletion is no longer unconditionally exempt: strip a
        # legacy key from the candidate without a row -> protection error
        noex = copy.deepcopy(cand_a)
        for s_ in noex["sections"]:
            for i_ in s_.get("items", []):
                i_.pop("note", None)
        base = build_checkpoints(src, None, ("reviews", "X1", "X1-R"))["A"]["conversions"]
        # WITHOUT the note removal row, the same deletion must be flagged
        base_no_note = [c for c in base if not (c["id"] == "X1" and c["field"] == "note")]
        assert any("被删除" in e for e in protections(src, noex, base_no_note)), "legacy 删除无行必须被拒"
        # WITH the row, the identical deletion is declared and passes
        assert not any("note" in e and "被删除" in e for e in protections(src, noex, base))

        # F-L1-02 partial write: REAL fd via a faulting handle wrapper —
        # the first large write lands 257 bytes, flushes, then raises; the
        # in-lock restore must bring back the exact original bytes
        ledger.write_bytes(raw)

        class FaultingHandle:
            def __init__(self, inner, cut):
                self._inner = inner
                self._cut = cut
                self._armed = True

            def write(self, data):
                if self._armed and len(data) > self._cut:
                    self._armed = False
                    self._inner.write(data[: self._cut])
                    self._inner.flush()
                    raise OSError("injected partial write failure")
                return self._inner.write(data)

            def __getattr__(self, name):
                return getattr(self._inner, name)

        real_open = open

        def faulting_open(file, *a, **k):
            handle = real_open(file, *a, **k)
            mode = (a[0] if a else k.get("mode", "r"))
            if "b" in mode and "+" in mode and str(file).endswith("specs.json"):
                return FaultingHandle(handle, 257)
            return handle

        args = _parse_args_apply(str(ledger), summary["source_sha256"],
                                 summary["conversions_sha256"],
                                 summary["candidate_a_sha256"], summary["candidate_b_sha256"],
                                 probe=str(good_probe))
        args.ruling = "reviews/X1=X1-R"
        with mock.patch("builtins.open", faulting_open):
            code = cmd_apply(args)
        assert code == 1, code
        assert ledger.read_bytes() == raw, "部分写后必须恢复精确原件（不是残文件）"

        # F-L2-01 final-probe external update: wrapper probe verifies B via
        # the real document, THEN mutates a title — apply must observe the
        # fingerprint mismatch, exit nonzero, preserve the external content,
        # and never emit a success receipt
        ledger.write_bytes(raw)
        sneaky = td / "sneaky_probe.py"
        # a wrapper probe: prints a VALID summary for whatever is on disk,
        # THEN mutates a title — the final fingerprint gate must catch it
        sneaky_lines = [
            "import sys, json, pathlib",
            "p = pathlib.Path(sys.argv[1])",
            "d = json.loads(p.read_text(encoding='utf-8'))",
            "n = sum(len(s['items']) for s in d['sections'])",
            "T = {'reports':'Reports','goals':'Goals','architecture':'Architecture',",
            "     'designs':'Designs','tests':'Tests','reviews':'Reviews'}",
            "sec = [(s['id'], T[s['id']]) for s in d['sections']]",
            "print(json.dumps({'project':'fixture','version':d['version'],",
            "                  'total_items':n,'sections':sec}))",
            "d['sections'][0]['items'][0]['title'] = 'externally-mutated'",
            "p.write_text(json.dumps(d, ensure_ascii=False, indent=2), encoding='utf-8')",
        ]
        sneaky.write_text(chr(10).join(sneaky_lines) + chr(10), encoding="utf-8")
        r = run("apply", "--ledger", str(ledger),
                "--expected-source-sha256", summary["source_sha256"],
                "--expected-conversions-sha256", summary["conversions_sha256"],
                "--expected-candidate-a-sha256", summary["candidate_a_sha256"],
                "--expected-candidate-b-sha256", summary["candidate_b_sha256"],
                "--ruling", "reviews/X1=X1-R",
                "--loader-probe", str(sneaky))
        assert r.returncode == 4, (r.returncode, r.stdout, r.stderr)
        assert "applied" not in r.stdout, "指纹不符不得发布成功收据"
        after_sneaky = json.loads(ledger.read_text(encoding="utf-8"))
        assert after_sneaky["sections"][0]["items"][0]["title"] == "externally-mutated", "外部更新必须保留"

        # happy path once more on a clean copy (final gate green)
        ledger.write_bytes(raw)
        r = run("apply", "--ledger", str(ledger),
                "--expected-source-sha256", summary["source_sha256"],
                "--expected-conversions-sha256", summary["conversions_sha256"],
                "--expected-candidate-a-sha256", summary["candidate_a_sha256"],
                "--expected-candidate-b-sha256", summary["candidate_b_sha256"],
                "--ruling", "reviews/X1=X1-R",
                "--loader-probe", str(good_probe))
        assert r.returncode == 0 and "applied" in r.stdout, (r.returncode, r.stderr)

    print("PASS: exit-codes/blocked-3/ready-0/two-checkpoint-versions/rename-recorded/"
          "original-values/undeclared-delete/schema-negatives/cas-prewrite-external-wins/"
          "postwrite-external-not-clobbered/exact-rollback/happy-path/cas-backup/"
          "l2:status-three-state/legacy-dup-row/section-value-guard/no-legacy-exemption/"
          "partial-write-restore/final-fingerprint-gate")


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def _load_increments(path) -> dict | None:
    if not path:
        return None
    data = json.loads(Path(path).read_text(encoding="utf-8"))
    if isinstance(data, dict) and "increments" in data:
        data = data["increments"]
    if isinstance(data, dict) and all(isinstance(v, dict) for v in data.values()):
        return data
    raise SystemExit(f"--stage-increments 形态不符: {path}")


def _parse_ruling(ruling):
    if not ruling:
        return None
    section, rest = ruling.split("/", 1)
    old_id, new_id = rest.split("=", 1)
    return section, old_id, new_id


def _parse_args_from(argv):
    parser = argparse.ArgumentParser(prog="plan815_ledger_recover")
    sub = parser.add_subparsers(dest="cmd", required=True)
    p_audit = sub.add_parser("audit")
    p_audit.add_argument("--ledger", default=".autoos/specs.json")
    p_audit.add_argument("--out")
    p_conv = sub.add_parser("convert")
    p_conv.add_argument("--ledger", default=".autoos/specs.json")
    p_conv.add_argument("--out")
    p_conv.add_argument("--stage-increments", help="JSON {section: SpecItem} 四条 M815-A 增量拟稿")
    p_conv.add_argument("--ruling", help="T-16 核准的重复 ID 改名,形如 reviews/P014-1=P014-1-R")
    p_apply = sub.add_parser("apply")
    p_apply.add_argument("--ledger", default=".autoos/specs.json")
    p_apply.add_argument("--expected-source-sha256", required=True)
    p_apply.add_argument("--expected-conversions-sha256")
    p_apply.add_argument("--expected-candidate-a-sha256", required=True)
    p_apply.add_argument("--expected-candidate-b-sha256", required=True)
    p_apply.add_argument("--loader-probe", required=True,
                         help="真实 typed loader 可执行文件（写后读回+身份核验，必填）")
    p_apply.add_argument("--stage-increments")
    p_apply.add_argument("--ruling")
    return parser.parse_args(argv)


def _parse_args():
    return _parse_args_from(sys.argv[1:])


def _parse_args_apply(ledger, src_sha, conv_sha, cand_a_sha, cand_b_sha, probe=None):
    return argparse.Namespace(
        cmd="apply", ledger=ledger, expected_source_sha256=src_sha,
        expected_conversions_sha256=conv_sha,
        expected_candidate_a_sha256=cand_a_sha,
        expected_candidate_b_sha256=cand_b_sha,
        loader_probe=probe, stage_increments=None, ruling=None,
    )


def main() -> None:
    if len(sys.argv) > 1 and sys.argv[1] == "self-test":
        self_test()
        return
    args = _parse_args()
    code = {"audit": cmd_audit, "convert": cmd_convert, "apply": cmd_apply}[args.cmd](args)
    sys.exit(code)


if __name__ == "__main__":
    main()
