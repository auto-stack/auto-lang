#!/usr/bin/env python3
"""PLAN-870 ledger projection (guarded offline projection, Regime B).

No store writer reachable (HTTP probes empty); specs.json is TRACKED, so
this prepares the derived view in the plan worktree and lands it with the
ff merge. Guards: byte-exact backup, surgical JSON read-modify-write,
schema/ID read-back, version+1.
"""
import json
import shutil
import sys
from pathlib import Path

LEDGER = Path(".autoos/specs.json")
BACKUP = Path(".autoos/specs.json.bak-p870-merge")

SECTIONS = ["goals", "architecture", "designs", "tests", "reviews", "reports"]


def main() -> int:
    if not LEDGER.is_file():
        print("ledger missing/unparseable: stop")
        return 2
    raw = LEDGER.read_bytes()
    shutil.copyfile(LEDGER, BACKUP)
    doc = json.loads(raw)
    ids = [s["id"] for s in doc["sections"]]
    if sorted(ids) != sorted(SECTIONS):
        print(f"schema mismatch: {ids}")
        return 2

    def upsert(section: str, item: dict) -> None:
        sec = next(s for s in doc["sections"] if s["id"] == section)
        for i, existing in enumerate(sec["items"]):
            if existing["id"] == item["id"]:
                item = {**existing, **item}
                sec["items"][i] = item
                break
        else:
            sec["items"].append(item)

    gate_t, gate_tv = sys.argv[1], sys.argv[2]
    now = "2026-10-28"

    upsert("designs", {
        "id": "P870-DESIGN-NATIVE-DATA-OWNED-LOWERING",
        "title": "mn native data/owned lowering contract (av.data/1 + av.owned/1)",
        "content": (
            "PLAN-870:r2 delivers the mn (ac-mir-native) native lowering for the "
            "shared MIR's av.data/1 pure-value records and av.owned/1 text/"
            "text-record subset: frozen layouts (Str field 12B/align8, declaration "
            "order, iterative DFS), the mn private call ABI (text pairs, record "
            "by-address, hidden record out-param, MutRet write-back via field-move), "
            "the flat-memory ownership discipline (dead slot = zeroed pointer; "
            "all-exit sweeps by last-owning-write DESC with record fields in "
            "declaration order), the ac_text_runtime surface (ABI-C, negative "
            "returns trap 70), the three-tier ledger observation ruling (alloc "
            "byte-sequence exact / free multiset+exactly-once / per-instruction "
            "frees exact; within-exit order compared only on non-clone rows), "
            "the named rejection surface (wide scalars, collections/closures/"
            "dynamic, Unit, entry shape), and the conditional textlib cache axis "
            "keeping scalar manifests byte-identical. Frontend escalation picks "
            "the narrowest compiling profile (scalar/record/text-record) so B* "
            "sources keep av.core-i32/1 identity."
        ),
        "status": "done",
        "file": "docs/specs/auto-mir/native-data-owned-lowering.md",
        "tags": ["PLAN-870", "r2", "impl:edb6e794a", "spec:auto-mir/native-data-owned-lowering.md"],
        "module": "auto-mir",
        "created_at": now,
        "modified_at": now,
    })
    upsert("tests", {
        "id": "P870-TESTS",
        "title": "PLAN-870 differential tier + ace-m1 tr-native binding",
        "content": (
            "PLAN-870:r2 test assets: data_owned_tier 6 tests (10 executable "
            "ACE-M1 T*/R* rows across three real paths — AV arena baseline, old "
            "HIR-native reference, mn PE — with independent frozen oracles and "
            "the three-tier ledger comparison; mut write-back wrapper three-path "
            "with hand-computed 15; StrCmp eq/ne and invalid-codepoint machine "
            "faces through real emit+link+run; rejection-surface and "
            "static/layout/fingerprint counter-controls) plus the ace-m1 "
            "tr-native suite (17 cases / 31 checks, ac+av legs on frozen "
            "oracles, T05 named upstream exclusion) with the negative-oracle "
            "red control, and the mechanical repair of the pre-existing ac-mn "
            "fixture red (823 env field)."
        ),
        "status": "done",
        "file": "experimental/ace/crates/native/tests/data_owned_tier.rs",
        "tags": ["PLAN-870", "r2", "impl:edb6e794a",
                 "suite:crates/auto-lang/test/cases/ace-m1/ace-m1-tr-native.suite.ad"],
        "module": "auto-test-case",
        "created_at": now,
        "modified_at": now,
    })
    upsert("reviews", {
        "id": "P870-REVIEW-R1R2",
        "title": "PLAN-870 T-07 review chain: R1 needs_fix -> R2 pass",
        "content": (
            "PLAN-870 T-07 final review chain (reviewer was the implementation "
            "session; recorded as an independence limitation — verdicts "
            "reconstructed from artifacts): R1 needs_fix with two findings, both "
            "'implemented machine face without a runtime oracle' — F-870-R1-01 "
            "MutRet wrapper write-back and F-870-R1-02 StrCmp/invalid-codepoint "
            "(the only source-level eq row T05 is upstream-blocked by the shared "
            "lowering's generic-Bin view-eq path) — repaired in 042022df3 with "
            "targeted tests only (no src change; R1 evidence stands), R2 pass on "
            "6/6 differential tier, fmt clean, zero warnings. Merge-owned gates "
            "ran on the frozen delivery baseline and are recorded in the merge "
            "receipt."
        ),
        "status": "done",
        "file": "docs/plans/870-ac-mn-record-text-native-expansion.md",
        "tags": ["PLAN-870", "r2", "reviewed:042022df3", "repair:042022df3"],
        "module": "auto-test-case",
        "created_at": now,
        "modified_at": now,
    })
    upsert("reports", {
        "id": "P870-MERGE",
        "title": "PLAN-870 delivery consolidation",
        "content": (
            "PLAN-870:r2 delivery consolidation. The mn native backend grew from "
            "scalar-only to the av.data/1 + av.owned/1 record/text surface: the "
            "three ACE-M1 lower.rs rejections (394/462/484) are closed for the "
            "in-scope subset while collections/closures/dynamic stay named "
            "rejections per the 2026-10-28 ruling. ACE-M1 T*/R* rows are bound "
            "for real on ac-native (binding ac column graded; 10 value/trap rows "
            "ok, mirrored gap grading for the shared-stack-rejected rows). "
            "Upstream registrations handed to the S owner (862 line): view eq "
            "generic-Bin lowering gap (T05 named exclusion), split_units "
            "instruction-embedded type indices unremapped (mn resolves through "
            "the verify-guaranteed dst slot type), AV clone-row within-exit "
            "release order is a host Rc-drop artifact (three-tier ledger "
            "ruling). Canonical specs: new "
            "docs/specs/auto-mir/native-data-owned-lowering.md; native-consumer "
            "sections in data-place/owned-access profiles; ace-m1-conformance "
            "T*/R* binding section. Baseline refresh merged D862 (Unit compat "
            "arms). Known environmental reds attributed per KNOWN-DEBT "
            "(Windows timer-resolution sampler; rounds liveness under parallel "
            "load — isolated reruns green)."
        ),
        "status": "done",
        "file": "docs/plans/870-ac-mn-record-text-native-expansion.md",
        "tags": ["PLAN-870", "r2", "impl:edb6e794a", "reviewed:042022df3",
                 f"gate:cargo-t-{gate_t}", f"gate:cargo-tv-{gate_tv}"],
        "module": "auto-mir",
        "created_at": now,
        "modified_at": now,
    })

    if "version" in doc:
        doc["version"] = int(doc["version"]) + 1
    LEDGER.write_text(json.dumps(doc, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    # Read-back verification.
    back = json.loads(LEDGER.read_bytes())
    assert [s["id"] for s in back["sections"]] == ids or sorted(s["id"] for s in back["sections"]) == sorted(SECTIONS)
    for sec, want in [("designs", "P870-DESIGN-NATIVE-DATA-OWNED-LOWERING"),
                      ("tests", "P870-TESTS"),
                      ("reviews", "P870-REVIEW-R1R2"),
                      ("reports", "P870-MERGE")]:
        s = next(x for x in back["sections"] if x["id"] == sec)
        assert any(i["id"] == want for i in s["items"]), f"read-back missing {want}"
    # Scope guard: only the four items may differ beyond version.
    import difflib
    old = json.loads(raw)
    def snapshot(d):
        return {s["id"]: [i["id"] for i in s["items"]] for s in d["sections"]}
    added = {k: set(snapshot(back)[k]) - set(snapshot(old)[k]) for k in snapshot(old)}
    flat = {i for v in added.values() for i in v}
    assert flat == {"P870-DESIGN-NATIVE-DATA-OWNED-LOWERING", "P870-TESTS", "P870-REVIEW-R1R2", "P870-MERGE"}, flat
    print("projection ok: 4 items upserted, version ->", back.get("version"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
