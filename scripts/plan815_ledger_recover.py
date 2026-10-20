#!/usr/bin/env python3
"""PLAN-815 M815-L T-15: bounded ledger compatibility recovery tool.

Store schema mirror of auto-musk `backend/crates/musk/src/specs.rs`
(SpecsDocument / SpecsSection / SpecItem / SpecStatus — read-only source of
truth; SHA256 recorded by every report). Default is DRY-RUN: `audit` and
`convert` never touch the production ledger; `apply` is the T-17-only
single-writer path and refuses to run unless T-16-bound hashes match.

Conversion philosophy (plan r3 §5.6): restore a *loadable projection* —
never rewrite knowledge. Only missing store-required structure gets a
representation; values that exist are preserved verbatim or moved between
legacy/store names with the original recorded. Nothing here invents
completion times or acceptance verdicts: unknown statuses without a
proposed mapping and duplicate IDs are BLOCKED (T-16 must rule on them
before apply can run). Unknown extra fields are captured to a sidecar —
the real store (serde, no deny_unknown_fields) silently drops them on a
load/save round-trip, which the report states explicitly.
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
SECTION_TITLES = {
    "goals": "🎯 Goals", "architecture": "🏗️ Architecture", "designs": "🎨 Designs",
    "tests": "🧪 Tests", "reviews": "🔍 Reviews", "reports": "📊 Reports",
}
ITEM_STR = ("id", "title", "content")
ITEM_OPT_STR = ("priority", "assignee", "test_file", "file", "milestone", "module")
ITEM_LIST = ("depends_on", "related", "tags")
ITEM_U64 = ("created_at", "modified_at")

# --- proposed status representations (T-15 candidates, T-16 confirms) -------
# basis: the legacy value's own claim strength is preserved, nothing stronger;
# the original value stays in the conversion table either way.
STATUS_MAP = {
    "active": ("stable", "旧账本语义=现行有效；生命周期集合中最近似且不构成验收主张"),
    "current": ("stable", "同 active（现行有效）"),
    "delivered": ("done", "旧值本身即交付主张，映射为等强的 done，不新增主张"),
    "accepted": ("approved", "旧值本身即接受主张，映射为等强的 approved，不新增主张"),
}
STATUS_MISSING = ("empty", "store 工厂默认（SpecItem::new → Empty）；缺状态=无记录，非已验收")
TIMESTAMP_ZERO_BASIS = "历史未记录，0=未知哨兵（不补造时间）"
TIMESTAMP_MOVED_BASIS = "旧字段名搬移至 store 字段，值经解析逐字保留（ISO 日期→UTC 纪元秒，日精度无损）"


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def parse_legacy_time(value) -> tuple[int, str]:
    """Legacy `created`/`last_modified` values are ISO date strings or null."""
    if value is None:
        return 0, TIMESTAMP_ZERO_BASIS
    if isinstance(value, int) and not isinstance(value, bool):
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


def schema_errors(doc) -> list[str]:
    """Mirror of the store's serde load requirements (V-L1 oracle)."""
    errors: list[str] = []
    if not isinstance(doc, dict) or not isinstance(doc.get("project"), str):
        errors.append("root: project must be a string")
    if not isinstance(doc.get("version"), int) or isinstance(doc.get("version"), bool):
        errors.append("root: version must be u64")
    sections = doc.get("sections")
    if not isinstance(sections, list):
        return errors + ["root: sections must be a list"]
    for s in sections:
        sid = s.get("id", "?") if isinstance(s, dict) else "?"
        if not isinstance(s, dict):
            errors.append(f"section {sid}: not an object")
            continue
        for field in ("id", "section_type", "title", "content"):
            if not isinstance(s.get(field), str):
                errors.append(f"section {sid}: '{field}' must be a string")
        if s.get("section_type") not in SECTION_TYPES.values():
            errors.append(f"section {sid}: section_type must be PascalCase store wire ({sorted(SECTION_TYPES.values())})")
        if s.get("status") not in SPEC_STATUSES:
            errors.append(f"section {sid}: invalid status")
        if not isinstance(s.get("depends_on"), list):
            errors.append(f"section {sid}: depends_on must be a list")
        for field in ("last_modified",):
            if not isinstance(s.get(field), int) or isinstance(s.get(field), bool):
                errors.append(f"section {sid}: '{field}' must be u64")
        if s.get("last_verified") is not None and not isinstance(s.get("last_verified"), int):
            errors.append(f"section {sid}: last_verified must be u64 or null")
        for it in s.get("items", []):
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
                if not isinstance(it.get(field), list):
                    errors.append(f"item {iid}: '{field}' must be a list")
            for field in ITEM_U64:
                if not isinstance(it.get(field), int) or isinstance(it.get(field), bool):
                    errors.append(f"item {iid}: '{field}' must be u64")
            if it.get("completed_at") is not None and (not isinstance(it.get("completed_at"), int) or isinstance(it.get("completed_at"), bool)):
                errors.append(f"item {iid}: 'completed_at' must be u64 or null")
            if it.get("status") not in SPEC_STATUSES:
                errors.append(f"item {iid}: invalid status {it.get('status')!r}")
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
            status = it.get("status", "<missing>")
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
    stage_increments: dict[str, dict] | None = None,
    ruling_rename: tuple[str, str, str] | None = None,
) -> tuple[dict, list[dict], dict, list[str]]:
    """Build the candidate projection. Returns (candidate, conversions,
    unknown_sidecar, blocked). Blocked non-empty ⇒ apply must refuse.

    stage_increments: {section_id: full SpecItem dict} for the four M815-A
    records — the *stage source increments* are in recovery scope (plan
    §5.6); they replace the legacy 815 entries field-by-field with the
    closeout-authored records (source-sha256 tags carried on the record).
    ruling_rename: (section, old_id, new_id) — applied only when T-16 has
    confirmed the ruling (e.g. P014-1 duplicate); without it duplicates
    stay BLOCKED."""
    if ruling_rename:
        section, old_id, new_id = ruling_rename
        for s_ in source.get("sections", []):
            if s_.get("id") == section:
                for it_ in s_.get("items", []):
                    if it_.get("id") == old_id:
                        it_["id"] = new_id
    candidate = copy.deepcopy(source)
    conversions: list[dict] = []
    unknown: dict[str, dict] = {}
    blocked: list[str] = []

    def record(section, iid, field, original, new, basis, note=""):
        conversions.append({
            "section": section, "id": iid, "field": field,
            "original": original, "new": new, "basis": basis, "note": note,
        })

    # --- sections ---
    for s in candidate.get("sections", []):
        sid = s.get("id", "?")
        if "section_type" not in s:
            s["section_type"] = SECTION_TYPES[sid]
            record(sid, None, "section_type", None, s["section_type"], "SectionType::from_id 派生;线格式=PascalCase(真 loader 实测,snake 会被 serde(other) 静默滤空)")
        if "title" not in s:
            s["title"] = SECTION_TITLES.get(sid, sid)
            record(sid, None, "title", None, s["title"], "SpecsSection::new 工厂标题")
        if "status" not in s:
            s["status"] = "empty"
            record(sid, None, "status", None, "empty", STATUS_MISSING[1])
        if "content" not in s:
            s["content"] = ""
            record(sid, None, "content", None, "", "工厂默认（legacy 整节文本，未记录）")
        if "depends_on" not in s:
            s["depends_on"] = []
            record(sid, None, "depends_on", None, [], "工厂默认")
        if "last_modified" not in s:
            s["last_modified"] = 0
            record(sid, None, "last_modified", None, 0, TIMESTAMP_ZERO_BASIS)
        if "last_verified" not in s:
            s["last_verified"] = None
            record(sid, None, "last_verified", None, None, "工厂默认")

    # --- items ---
    for s in candidate.get("sections", []):
        sid = s.get("id", "?")
        for it in s.get("items", []):
            iid = it.get("id", "?")
            # status
            status = it.get("status", None)
            ok = isinstance(status, str) and status in SPEC_STATUSES
            if status is None or "status" not in it:
                it["status"] = STATUS_MISSING[0]
                record(sid, iid, "status", status, it["status"], STATUS_MISSING[1], note="原值缺失")
            elif not ok:
                if isinstance(status, str) and status in STATUS_MAP:
                    mapped, basis = STATUS_MAP[status]
                    it["status"] = mapped
                    record(sid, iid, "status", status, mapped, basis)
                else:
                    blocked.append(f"{sid}/{iid}: 未知状态 {status!r} 无映射依据，待 T-16 裁定")
            # containers
            for field in ITEM_LIST:
                if field not in it:
                    it[field] = []
                    record(sid, iid, field, None, [], "容器缺失=空（无声明）")
                elif not isinstance(it[field], list):
                    fixed, _ = coerce_list(it[field], field, "旧值规范化为列表（原值保留）")
                    it[field] = fixed
                    record(sid, iid, field, it.get(field), fixed, "旧值类型规范化为列表，值保留", note="stringified/null 容器")
            # optional strings
            for field in ITEM_OPT_STR:
                if field not in it:
                    it[field] = None
                    record(sid, iid, field, None, None, "Option 工厂默认")
            # timestamps: legacy `created`/`last_modified` move (value
            # preserved via exact ISO-date parse), else 0 sentinel
            legacy_created = it.pop("created", None)
            if "created_at" not in it:
                value, basis = parse_legacy_time(legacy_created)
                it["created_at"] = value
                if legacy_created is not None:
                    record(sid, iid, "created", legacy_created, f"removed→created_at={value}", TIMESTAMP_MOVED_BASIS)
                record(sid, iid, "created_at", legacy_created, value, basis,
                       note="旧字段 created 搬移" if legacy_created is not None else "无旧值")
            elif legacy_created is not None:
                unknown.setdefault(f"{sid}/{iid}", {})["created"] = legacy_created
            legacy_modified = it.pop("last_modified", None)
            if "modified_at" not in it:
                fallback = legacy_modified if legacy_modified is not None else legacy_created
                value, basis = parse_legacy_time(fallback)
                it["modified_at"] = value
                if legacy_modified is not None:
                    record(sid, iid, "last_modified", legacy_modified, f"removed→modified_at={value}", TIMESTAMP_MOVED_BASIS)
                record(sid, iid, "modified_at", fallback, value, basis,
                       note="旧字段 last_modified/created 搬移" if fallback is not None else "无旧值")
            elif legacy_modified is not None:
                unknown.setdefault(f"{sid}/{iid}", {})["last_modified"] = legacy_modified
            if "completed_at" not in it:
                it["completed_at"] = None
                record(sid, iid, "completed_at", None, None, "不发明完成时间")
            if "content" not in it:
                it["content"] = ""
                record(sid, iid, "content", None, "", "原即缺失（空=无内容），未补造")
            # any remaining unknown extras → sidecar (store drops them on round-trip)
            known = set(ITEM_STR) | set(ITEM_OPT_STR) | set(ITEM_LIST) | set(ITEM_U64) | {"status", "completed_at"}
            for key in [k for k in it if k not in known]:
                unknown.setdefault(f"{sid}/{iid}", {})[key] = it.pop(key)

    # --- stage source increments: replace legacy 815 entries with the
    # closeout-authored records (field-level diffs recorded) -------------
    if stage_increments:
        for sec_id, proposed in stage_increments.items():
            section = next((x for x in candidate.get("sections", []) if x.get("id") == sec_id), None)
            if section is None:
                blocked.append(f"{sec_id}: 阶段增量目标 section 不存在")
                continue
            existing = next((i for i in section.get("items", []) if i.get("id") == proposed.get("id")), None)
            if existing is None:
                section["items"].append(copy.deepcopy(proposed))
                record(sec_id, proposed["id"], "<record>", None, "added",
                       "M815-A 阶段来源增量(收尾拟稿,R4 reviewed 7045e20fa / delivery e609caf1a)")
                continue
            for field, new_value in proposed.items():
                if existing.get(field) != new_value:
                    record(sec_id, proposed["id"], field, existing.get(field), new_value,
                           "M815-A 阶段来源增量(收尾拟稿,R4 reviewed 7045e20fa / delivery e609caf1a)")
                    existing[field] = new_value

    # --- duplicate ids: blocked, with a section-aware rename proposal -----
    seen: dict[str, list[str]] = {}
    for s in candidate.get("sections", []):
        for it in s.get("items", []):
            seen.setdefault(it.get("id", "?"), []).append(s.get("id", "?"))
    for iid, sections in seen.items():
        if len(sections) > 1:
            blocked.append(
                f"{'/'.join(sections)}/{iid}: 重复 ID（{sections}）；"
                f"T-15 建议：保留 designs 原名、reviews 改名 {iid}-R（镜像 Pxxx-R 惯例），待 T-16 裁定"
            )
    return candidate, conversions, unknown, blocked


def protections(source: dict, candidate: dict, conversions: list[dict]) -> list[str]:
    """Structural protections beyond schema (plan §5.6)."""
    errors = []
    ids_src = [(s.get("id"), it.get("id")) for s in source.get("sections", []) for it in s.get("items", [])]
    ids_dst = [(s.get("id"), it.get("id")) for s in candidate.get("sections", []) for it in s.get("items", [])]
    if sorted(ids_src) != sorted(ids_dst):
        errors.append("protection: 原始 ID 集合改变（含重复）")
    flat_dst = [i for _, i in ids_dst]
    if len(flat_dst) != len(set(flat_dst)):
        errors.append("protection: 候选存在重复 ID")
    # every original present field is either unchanged or explicitly converted
    converted = {(c["section"], c["id"], c["field"]) for c in conversions if c["id"]}
    touched_fields = set()
    for s_src, s_dst in zip(source.get("sections", []), candidate.get("sections", [])):
        for it_src in s_src.get("items", []):
            it_dst = next((i for i in s_dst.get("items", []) if i.get("id") == it_src.get("id")), None)
            if it_dst is None:
                continue
            for key, value in it_src.items():
                touched_fields.add((s_src.get("id"), it_src.get("id"), key))
                if key in it_dst and it_dst[key] != value and (s_src.get("id"), it_src.get("id"), key) not in converted:
                    errors.append(
                        f"protection: {s_src.get('id')}/{it_src.get('id')}.{key} 未经转换声明被改变"
                    )
    return errors


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


def _emit(out_dir: str | None, name: str, text: str) -> None:
    if out_dir:
        Path(out_dir).mkdir(parents=True, exist_ok=True)
        (Path(out_dir) / name).write_text(text, encoding="utf-8", newline="\n")


def cmd_convert(args) -> int:
    ledger = Path(args.ledger)
    raw = ledger.read_bytes()
    source = json.loads(raw.decode("utf-8"))
    increments = _load_increments(args.stage_increments)
    candidate, conversions, unknown, blocked = convert(source, increments, _parse_ruling(args.ruling))
    problems = schema_errors(candidate)
    guard = protections(source, candidate, conversions)
    # round-trip stability of the candidate through the same codec
    encoded = json.dumps(candidate, ensure_ascii=False, indent=2) + "\n"
    reloaded = json.loads(encoded)
    second, _conv2, _unk2, _block2 = convert(reloaded)
    if schema_errors(second) or json.dumps(second, ensure_ascii=False, sort_keys=True) != json.dumps(reloaded, ensure_ascii=False, sort_keys=True):
        problems.append("round-trip: 候选再转换不稳定")
    candidate_bytes = encoded.encode("utf-8")
    result = {
        "ledger": str(ledger),
        "source_sha256": sha256_bytes(raw),
        "schema_sha256": sha256_bytes(Path(SCHEMA_SHA256_FILE).read_bytes()),
        "source_version": source.get("version"),
        "candidate_version": candidate.get("version"),
        "candidate_sha256": sha256_bytes(candidate_bytes),
        "conversions_sha256": sha256_bytes(json.dumps(conversions, ensure_ascii=False, sort_keys=True).encode("utf-8")),
        "conversion_count": len(conversions),
        "unknown_field_items": len(unknown),
        "blocked": blocked,
        "schema_errors": problems,
        "protection_errors": guard,
        "status": "BLOCKED" if (blocked or problems or guard) else "READY",
    }
    if args.out:
        out = Path(args.out)
        out.mkdir(parents=True, exist_ok=True)
        (out / "candidate.json").write_bytes(candidate_bytes)
        (out / "conversions.json").write_text(
            json.dumps(conversions, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
        (out / "unknown-fields-sidecar.json").write_text(
            json.dumps(unknown, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
        (out / "convert-summary.json").write_text(
            json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


def cmd_apply(args) -> int:
    ledger = Path(args.ledger)
    raw = ledger.read_bytes()
    source_sha = sha256_bytes(raw)
    if source_sha != args.expected_source_sha256:
        print(f"REFUSED: 源文件 hash 漂移（现 {source_sha[:12]}，T-16 绑定 {args.expected_source_sha256[:12]}）", file=sys.stderr)
        return 2
    source = json.loads(raw.decode("utf-8"))
    increments = _load_increments(args.stage_increments)
    candidate, conversions, unknown, blocked = convert(source, increments, _parse_ruling(args.ruling))
    if blocked:
        print("REFUSED: 存在待 T-16 裁定的阻断项", file=sys.stderr)
        return 2
    problems = schema_errors(candidate) or protections(source, candidate, conversions)
    if problems:
        print("REFUSED: " + "; ".join(problems), file=sys.stderr)
        return 2
    if args.expected_conversions_sha256 and sha256_bytes(
            json.dumps(conversions, ensure_ascii=False, sort_keys=True).encode("utf-8")) != args.expected_conversions_sha256:
        print("REFUSED: 转换表 hash 与 T-16 绑定不符", file=sys.stderr)
        return 2
    candidate_bytes = (json.dumps(candidate, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    if sha256_bytes(candidate_bytes) != args.expected_candidate_sha256:
        print("REFUSED: 候选 hash 与 T-16 绑定不符", file=sys.stderr)
        return 2
    backup = ledger.with_name(ledger.name + ".bak-recover-" + source_sha[:16])
    if backup.exists():
        existing = sha256_bytes(backup.read_bytes())
        if existing != source_sha:
            print(f"REFUSED: 备份 {backup.name} 已存在且字节不同（CAS 保护，不覆盖）", file=sys.stderr)
            return 2
    else:
        backup.write_bytes(raw)
    tmp = ledger.with_suffix(".json.recover-tmp")
    tmp.write_bytes(candidate_bytes)
    # post-write validation: schema read-back on the real file bytes, then
    # (when given) the REAL typed loader; any failure rolls back the exact
    # original bytes
    try:
        import subprocess

        check = json.loads(tmp.read_text(encoding="utf-8"))
        if schema_errors(check) or sha256_bytes(tmp.read_bytes()) != sha256_bytes(candidate_bytes):
            raise RuntimeError("写后读回校验失败")
        shutil.copyfile(tmp, ledger)
        if args.loader_probe:
            probe = subprocess.run(
                [args.loader_probe, str(ledger)], capture_output=True, text=True
            )
            if probe.returncode != 0:
                raise RuntimeError(f"真实 typed loader 读回失败: {probe.stderr.strip()[:400]}")
    except Exception as exc:  # noqa: BLE001 — rollback must cover every failure
        if sha256_bytes(ledger.read_bytes()) != source_sha:
            ledger.write_bytes(raw)
        print(f"FAILED（已恢复原件）: {exc}", file=sys.stderr)
        return 1
    finally:
        tmp.unlink(missing_ok=True)
    after = sha256_bytes(ledger.read_bytes())
    print(json.dumps({
        "applied": True, "ledger": str(ledger), "source_sha256": source_sha,
        "candidate_sha256": sha256_bytes(ledger.read_bytes()),
        "backup": backup.name, "after_sha256_matches_candidate": after == sha256_bytes(candidate_bytes),
        "loader_probe": bool(args.loader_probe),
    }, ensure_ascii=False, indent=2))
    return 0


def self_test() -> None:
    """V-L3: failure & concurrency branches on temp fixtures — the
    production ledger is never touched."""
    import unittest.mock as mock

    with tempfile.TemporaryDirectory() as td:
        td = Path(td)
        src = {"project": "fixture", "version": 3, "sections": [
            {"id": "reports", "items": [
                {"id": "X1", "title": "t", "content": "c", "status": "active",
                 "created": "2026-10-01", "note": "extra"},
                {"id": "X1", "title": "dup", "content": "c", "status": "active"},
            ]},
        ]}
        ledger = td / "specs.json"
        ledger.write_text(json.dumps(src, ensure_ascii=False), encoding="utf-8")
        raw = ledger.read_bytes()

        # duplicate id + unknown extra field → convert BLOCKED, original intact
        out = td / "out"
        sys.argv = ["recover", "convert", "--ledger", str(ledger), "--out", str(out)]
        args = _parse_args()
        cmd_convert(args)
        summary = json.loads((out / "convert-summary.json").read_text(encoding="utf-8"))
        assert summary["status"] == "BLOCKED", summary
        assert any("重复 ID" in b for b in summary["blocked"])
        assert ledger.read_bytes() == raw, "dry-run 不得改原件"

        # rename the dup → READY; unknown field captured to sidecar
        src["sections"][0]["items"][1]["id"] = "X1-R"
        ledger.write_text(json.dumps(src, ensure_ascii=False), encoding="utf-8")
        raw = ledger.read_bytes()
        cmd_convert(_parse_args())
        summary = json.loads((out / "convert-summary.json").read_text(encoding="utf-8"))
        assert summary["status"] == "READY", summary
        sidecar = json.loads((out / "unknown-fields-sidecar.json").read_text(encoding="utf-8"))
        assert sidecar.get("reports/X1", {}).get("note") == "extra"
        candidate = json.loads((out / "candidate.json").read_text(encoding="utf-8"))
        assert candidate["sections"][0]["items"][0]["created_at"] == 1790812800  # 2026-10-01 UTC
        assert candidate["sections"][0]["items"][0]["status"] == "stable"

        # apply with wrong expected source hash → refused, CAS backup intact
        args = _parse_args_apply(str(ledger), "0" * 64, None, None)
        assert cmd_apply(args) == 2 and ledger.read_bytes() == raw

        # apply with drifted conversions hash → refused
        summary = json.loads((out / "convert-summary.json").read_text(encoding="utf-8"))
        args = _parse_args_apply(str(ledger), summary["source_sha256"], "1" * 64, summary["candidate_sha256"])
        assert cmd_apply(args) == 2 and ledger.read_bytes() == raw

        # apply happy path (no probe) → backup created once; bytes match candidate
        args = _parse_args_apply(str(ledger), summary["source_sha256"], summary["conversions_sha256"], summary["candidate_sha256"])
        assert cmd_apply(args) == 0
        backup = next(td.glob("specs.json.bak-recover-*"))
        assert sha256_bytes(backup.read_bytes()) == summary["source_sha256"]
        candidate_bytes = (out / "candidate.json").read_bytes()
        assert sha256_bytes(ledger.read_bytes()) == sha256_bytes(candidate_bytes)

        # re-apply: source drifted (ledger is now the candidate) → refused,
        # existing backup not overwritten (CAS)
        assert cmd_apply(args) == 2
        assert sha256_bytes(backup.read_bytes()) == summary["source_sha256"]

        # pre-validation failure injection → REFUSED(2) before any write
        ledger.write_bytes(raw)  # reset
        args = _parse_args_apply(str(ledger), summary["source_sha256"], summary["conversions_sha256"], summary["candidate_sha256"])
        with mock.patch(__name__ + ".schema_errors", return_value=["injected pre-check failure"]):
            code = cmd_apply(args)
        assert code == 2 and ledger.read_bytes() == raw, "前检失败必须拒绝且零写入"

        # real-loader read-back failure branch → rollback restores exact bytes
        # (sys.executable run on a JSON file exits nonzero = failing probe)
        args = _parse_args_apply(str(ledger), summary["source_sha256"], summary["conversions_sha256"], summary["candidate_sha256"], probe=sys.executable)
        code = cmd_apply(args)
        assert code == 1 and ledger.read_bytes() == raw, "真实 loader 读回失败必须恢复精确原件"
        # every backup holds the original bytes (CAS, never overwritten)
        assert all(sha256_bytes(p.read_bytes()) == summary["source_sha256"] for p in td.glob("specs.json.bak-recover-*"))

    print("PASS: dup-block/dry-run-intact/unknown-sidecar/iso-time-move/status-map/"
          "apply-hash-drift/conversion-drift/cas-backup/read-back-rollback")


def _parse_args():
    return _parse_args_from(sys.argv[1:])


def _parse_args_from(argv):
    parser = argparse.ArgumentParser(prog="plan815_ledger_recover")
    sub = parser.add_subparsers(dest="cmd", required=True)
    p_audit = sub.add_parser("audit")
    p_audit.add_argument("--ledger", default=".autoos/specs.json")
    p_audit.add_argument("--out")
    p_conv = sub.add_parser("convert")
    p_conv.add_argument("--ledger", default=".autoos/specs.json")
    p_conv.add_argument("--out")
    p_conv.add_argument("--stage-increments", help="JSON {{section: SpecItem}} 四条 M815-A 增量拟稿")
    p_conv.add_argument("--ruling", help="T-16 核准的重复 ID 改名,形如 reviews/P014-1=P014-1-R")
    p_apply = sub.add_parser("apply")
    p_apply.add_argument("--ledger", default=".autoos/specs.json")
    p_apply.add_argument("--expected-source-sha256", required=True)
    p_apply.add_argument("--expected-conversions-sha256")
    p_apply.add_argument("--expected-candidate-sha256", required=True)
    p_apply.add_argument("--loader-probe", help="真实 typed loader 可执行文件（写后读回）")
    p_apply.add_argument("--stage-increments")
    p_apply.add_argument("--ruling")
    for p in (p_audit, p_conv, p_apply):
        p.set_defaults(func=None)
    return parser.parse_args(argv)


def _load_increments(path: str | None) -> dict[str, dict] | None:
    if not path:
        return None
    data = json.loads(Path(path).read_text(encoding="utf-8"))
    if isinstance(data, dict) and "increments" in data:
        data = data["increments"]
    if isinstance(data, dict) and all(isinstance(v, dict) for v in data.values()):
        return data
    raise SystemExit(f"--stage-increments 形态不符: {path}")


def _parse_ruling(ruling: str | None) -> tuple[str, str, str] | None:
    if not ruling:
        return None
    section, rest = ruling.split("/", 1)
    old_id, new_id = rest.split("=", 1)
    return section, old_id, new_id


def _parse_args_apply(ledger, src_sha, conv_sha, cand_sha, probe=None):
    ns = argparse.Namespace(
        cmd="apply", ledger=ledger, expected_source_sha256=src_sha,
        expected_conversions_sha256=conv_sha, expected_candidate_sha256=cand_sha,
        loader_probe=probe, stage_increments=None, ruling=None,
    )
    return ns


def main() -> None:
    if len(sys.argv) > 1 and sys.argv[1] == "self-test":
        self_test()
        return
    args = _parse_args()
    {"audit": cmd_audit, "convert": cmd_convert, "apply": cmd_apply}[args.cmd](args)


if __name__ == "__main__":
    main()
