#!/usr/bin/env python3
"""PLAN-887 merge ledger projection (guarded offline, 884 precedent).

Backup byte-exact -> append/upsert 4 items (designs/tests/reviews/reports)
with file+tags -> version+1 -> validate -> read back -> diff summary.
Store writer unreachable (127.0.0.1:8080), so this is the sanctioned
Regime B channel; named honestly in the merge receipt.
"""
from __future__ import annotations

import json
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path

LEDGER = Path(".autoos/specs.json")
BACKUP = Path(".autoos/specs.json.bak-p887-merge")
NOW = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def upsert(doc, section, item):
    sec = next(s for s in doc["sections"] if s["id"] == section)
    for i, existing in enumerate(sec["items"]):
        if existing["id"] == item["id"]:
            item = {**existing, **item}
            sec["items"][i] = item
            break
    else:
        sec["items"].append(item)


def main() -> int:
    if not LEDGER.is_file():
        print("ledger missing", file=sys.stderr)
        return 1
    shutil.copyfile(LEDGER, BACKUP)
    before = LEDGER.read_bytes()
    doc = json.loads(before.decode("utf-8"))
    version_before = doc.get("version")
    ids = {s["id"] for s in doc["sections"]}
    assert ids == {"reports", "goals", "architecture", "designs", "tests", "reviews"}, ids

    upsert(doc, "designs", {
        "id": "P887-GOVERNANCE",
        "title": "测试治理：盘点闭包/守恒裁判/floor 派生（governance-and-inventory）",
        "content": (
            "PLAN-887:r2 delivers the test-governance contract: managed-baseline "
            "inventory with subcase identity (crate\\tpath), 844+869 retirement + "
            "841 migration channels consumed by the runner (retirements() loads "
            "both manifests), per-ID conservation guards (check_inventory: ghost/"
            "missing/zero/suppressed/stale all red; actual 3703 listed/3203 "
            "selected with missing=0), floors as derived guards (language 3189 = "
            "3205−16 869-retired; tb 3076 = 3095−16−3 window), first-wave suite "
            "(30 reuse B01..B20+U01..U10 via original suites + 3 new D-group ACE "
            "consumption via wrapper entries with recorded byte mapping), 8-class "
            "long-term controls (V-04..V-08 in plan887_test_governance 12/12), "
            "coverage matrix with weak-judge deltas, and the verified-invalidated "
            "performance axes (worker session-less store is fresh-per-request; "
            "worker session reuse out-of-domain) closing AC-05 by mechanism-level "
            "negative evidence (user-ruled r2 revision)."
        ),
        "status": "stable",
        "file": "docs/specs/auto-test-case/governance-and-inventory.md",
        "tags": ["PLAN-887", "r2", "delivery:auto-lang:dd916c184", "guarded-offline-projection"],
        "module": "auto-test-case",
        "created_at": NOW,
        "modified_at": NOW,
    })
    upsert(doc, "tests", {
        "id": "P887-TESTS",
        "title": "PLAN-887 governance integration + governance scripts + controls",
        "content": (
            "PLAN-887:r2 test assets: experimental/ace/tests/conformance/tests/governance.rs "
            "(12 tests: first-wave D suite real-worker 6 checks exit 3/-6/9, 30-reuse "
            "original-suite runs, V-04..V-08 controls, byte-identity), "
            "scripts/test-governance/ (inventory/check_inventory/measure-first-wave/"
            "coverage-matrix), scripts/tests/test_test_governance.py (9 conservation "
            "controls), and the runner floor reconciliation. Run explicitly via "
            "cargo test --manifest-path experimental/ace/Cargo.toml -p ace-conformance --test conformance governance::."
        ),
        "status": "verified",
        "file": "experimental/ace/tests/conformance/tests/governance.rs",
        "tags": ["PLAN-887", "r2", "delivery:auto-lang:dd916c184", "guarded-offline-projection"],
        "module": "auto-test-case",
        "created_at": NOW,
        "modified_at": NOW,
    })
    upsert(doc, "reviews", {
        "id": "P887-REVIEW-FINAL",
        "title": "PLAN-887 独立终审（final, r2）",
        "content": (
            "Independent final review on the frozen tree e47ffb2e3 (rebased "
            "delivery dd916c184): AC-01..07 all pass with empirical evidence "
            "(conservation conserved:true, retirements 844+869, floors per-ID, "
            "wrapper bytes/sha/oracle verified, integration 12/12 re-run, protocol "
            "47/47, fmt/warning zero-new). Findings R887-F1/F2 (LOW, bookkeeping "
            "corrected), F3/F4 (INFO). Merge-owned gates completed: bare cargo t "
            "3204/3204, tv 163/163, tt 3575/3575, integration+protocol green on "
            "the rebased union; frozen three-repo batch attributed for documented "
            "load flakes (plan705/plan707 偶发时序族, worker-family walls, ffi_dual "
            "nested-build) with isolated replays all green and original exit codes "
            "preserved; tb floor reconciled per-ID."
        ),
        "status": "verified",
        "file": "docs/plans/archive/887-test-governance-and-ace-first-corpus.md",
        "tags": ["PLAN-887", "r2", "reviewed:e47ffb2e3", "delivery:auto-lang:dd916c184", "guarded-offline-projection"],
        "module": "auto-test-case",
        "created_at": NOW,
        "modified_at": NOW,
    })
    upsert(doc, "reports", {
        "id": "P887-DEPOSIT",
        "title": "PLAN-887 交付沉淀（merge）",
        "content": (
            "PLAN-887:r2 delivered: test governance closure (inventory/conservation/"
            "floors), first-wave ACE corpus (30 reuse + 3 new D consumption), "
            "8-class long-term controls, coverage matrix, performance negative "
            "evidence (user-ruled AC-05 revision), canonical SD-01..04 landed, "
            "ledger v135→v136 (guarded offline projection; store writer "
            "unreachable), archived 887-test-governance-and-ace-first-corpus.md."
        ),
        "status": "done",
        "file": "docs/plans/archive/887-test-governance-and-ace-first-corpus.md",
        "tags": ["PLAN-887", "r2", "delivery:auto-lang:dd916c184", "guarded-offline-projection"],
        "module": "auto-test-case",
        "created_at": NOW,
        "modified_at": NOW,
    })

    doc["version"] = version_before + 1 if isinstance(version_before, int) else version_before
    payload = json.dumps(doc, ensure_ascii=False, indent=2).encode("utf-8") + b"\n"
    LEDGER.write_bytes(payload)

    # guards: parse-back + surgical-scope diff vs backup (only 4 items + version)
    after = json.loads(payload.decode("utf-8"))
    assert after["version"] == version_before + 1
    ids = {s["id"] for s in after["sections"]}
    assert ids == {"reports", "goals", "architecture", "designs", "tests", "reviews"}
    for sid in ("designs", "tests", "reviews", "reports"):
        sec = next(s for s in after["sections"] if s["id"] == sid)
        item = next(i for i in sec["items"] if i["id"].startswith("P887-"))
        assert item["file"] and item["tags"], (sid, item["id"])
    import difflib
    diff = list(difflib.unified_diff(
        before.decode("utf-8").splitlines(), payload.decode("utf-8").splitlines(), lineterm=""))
    added_lines = [l for l in diff if l.startswith("+") and not l.startswith("+++")]
    removed_lines = [l for l in diff if l.startswith("-") and not l.startswith("---")]
    print(f"version {version_before} -> {after['version']}; +{len(added_lines)}/-{len(removed_lines)} lines")
    print(f"backup: {BACKUP} ({len(before)} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
