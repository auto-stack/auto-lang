#!/usr/bin/env python3
"""PLAN-741 Phase 5 (T-33) final link / counter / pointer assertions.

Resolves the plan file's markdown links (outside code fences) against the
FINAL ARCHIVED location, so a link that only works while the file lives in
docs/plans/ fails here once the file sits in docs/plans/archive/ (fence- and
lifecycle-aware - no "file exists somewhere" checks).

Modes:
  --location active   file under docs/plans/; ledger review items may still
                      point at the active path (transitional). Markdown links
                      are validated against the SIMULATED ARCHIVE parent
                      (docs/plans/archive/) - the r5 fix targets the final
                      archived location, so report links resolve from there
                      by design.
  --location archive  file under docs/plans/archive/; links resolve from the
                      real parent and all pointers must use archived forms.

Also asserts: 35 unique task IDs, identical checkbox state for duplicate
task rows, and (archive mode) zero unchecked unique tasks.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
FENCE = re.compile(r"```.*?```", re.S)
LINK = re.compile(r"\]\(([^)#\s]+)(?:#[^)]*)?\)")
TASK = re.compile(r"^- \[([ x])\] \*\*(T-\d+)\*\*", re.M)
TASK_TOTAL = 35


def fail(msg: str) -> None:
    print(f"FAIL: {msg}")
    sys.exit(1)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--plan", required=True, help="path to 741-ac-hir-native-core.md")
    ap.add_argument(
        "--location",
        required=True,
        choices=["active", "archive"],
        help="expected lifecycle location of the plan file",
    )
    args = ap.parse_args()
    plan = Path(args.plan).resolve()
    if not plan.is_file():
        fail(f"plan file not found: {plan}")
    parent = plan.parent
    text = plan.read_text(encoding="utf8")

    # --- lifecycle location -------------------------------------------------
    expected_parent = REPO / "docs" / "plans" / ("archive" if args.location == "archive" else "")
    if parent != expected_parent:
        fail(f"plan lives in {parent}, expected {expected_parent} for --location {args.location}")
    status = re.search(r"^status: (\S+)", text, re.M)
    if not status:
        fail("frontmatter status missing")
    if args.location == "archive" and status.group(1) != "archived":
        fail(f"archive mode requires status: archived, found {status.group(1)}")
    if args.location == "active" and status.group(1) != "executing":
        fail(f"active mode requires status: executing, found {status.group(1)}")
    print(f"OK  location: {parent} (status {status.group(1)})")

    # --- markdown links ------------------------------------------------------
    # Links are judged against the FINAL archived location: from docs/plans/
    # (active) that is the simulated archive parent; from docs/plans/archive/
    # it is the real parent. This is exactly the r5 contract - ../reports
    # links are correct only while active and were the archived-state defect.
    body = FENCE.sub("", text)
    link_base = parent if args.location == "archive" else parent / "archive"
    broken = []
    checked = 0
    for m in LINK.finditer(body):
        target = m.group(1)
        if "://" in target or target.startswith("/"):
            continue
        resolved = (link_base / target).resolve()
        checked += 1
        if not resolved.exists():
            broken.append(f"{target} -> {resolved}")
    if broken:
        fail(
            f"{len(broken)} unresolved markdown link(s) from {link_base}:"
            + "".join(f"\n  {b}" for b in broken)
        )
    print(f"OK  links: {checked} relative links resolve from {link_base}")

    # --- task counter: 35 unique IDs, duplicate rows agree ------------------
    states: dict[str, set[str]] = {}
    for m in TASK.finditer(text):
        states.setdefault(m.group(2), set()).add(m.group(1))
    if len(states) != TASK_TOTAL:
        fail(f"unique task IDs = {len(states)}, expected {TASK_TOTAL}")
    mixed = sorted(t for t, ss in states.items() if len(ss) > 1)
    if mixed:
        fail(f"same-ID rows with conflicting checkbox state: {mixed}")
    unchecked = sorted(t for t, ss in states.items() if ss == {" "})
    if args.location == "archive" and unchecked:
        fail(f"archive mode requires all tasks complete; unchecked: {unchecked}")
    print(
        f"OK  tasks: {len(states)}/{TASK_TOTAL} unique IDs, duplicates consistent, "
        f"unchecked={len(unchecked)}"
    )

    # --- navigation / metadata pointers -------------------------------------
    readme = (REPO / "experimental/ac-core/README.md").read_text(encoding="utf8")
    m = re.search(r"\]\((\.\./\.\./docs/plans[^)]*741-ac-hir-native-core\.md)\)", readme)
    if not m:
        fail("README plan link missing")
    readme_target = (REPO / "experimental/ac-core" / m.group(1)).resolve()
    if args.location == "archive" and "archive" not in readme_target.parts:
        fail(f"README link must point at archive/ in archive mode: {readme_target}")
    if not readme_target.exists():
        fail(f"README plan link unresolvable: {readme_target}")
    print(f"OK  README plan link -> {readme_target}")

    for mod in ("auto-hir", "auto-ac"):
        nav = (REPO / f"docs/specs/{mod}/plans.md").read_text(encoding="utf8")
        row = [ln for ln in nav.splitlines() if ln.startswith("| 741 |")]
        if len(row) != 1:
            fail(f"{mod}/plans.md: expected exactly one 741 row")
        m = re.search(r"\]\((\.\./\.\./plans(?:/archive)?/741-ac-hir-native-core\.md)\)", row[0])
        if not m:
            fail(f"{mod}/plans.md: 741 row link missing/malformed")
        nav_target = (REPO / f"docs/specs/{mod}" / m.group(1)).resolve()
        if args.location == "archive" and "archive" not in nav_target.parts:
            fail(f"{mod}/plans.md must point at archive/ in archive mode: {nav_target}")
        if not nav_target.exists():
            fail(f"{mod}/plans.md link unresolvable: {nav_target}")
        print(f"OK  {mod}/plans.md 741 row -> {nav_target}")

    # --- ledger P741-* pointers ----------------------------------------------
    ledger = json.loads((REPO / ".autoos/specs.json").read_text(encoding="utf8"))
    p741 = [
        it
        for sec in ledger["sections"]
        for it in sec.get("items", [])
        if it.get("id", "").startswith("P741-")
    ]
    if not p741:
        fail("ledger has no P741-* items")
    transitional = 0
    for it in p741:
        target = (REPO / it["file"]).resolve()
        if not target.exists():
            # Active-phase transitional state: review items left pointing at
            # the archive path by the previous merge are refreshed when the
            # file returns there at THIS merge (AC-26: never claim synced).
            if (
                args.location == "active"
                and it["id"] in ("P741-3", "P741-4", "P741-5", "P741-6")
                and "archive" in target.parts
            ):
                transitional += 1
                continue
            fail(f"ledger {it['id']} file unresolvable: {it['file']}")
        if it["id"] in ("P741-3", "P741-4", "P741-5", "P741-6") and args.location == "archive":
            if "archive" not in target.parts:
                fail(f"ledger {it['id']} must point at archive/ in archive mode: {it['file']}")
    note = (
        f" ({transitional} transitional archive-pointing review items, refreshed at merge)"
        if transitional
        else ""
    )
    print(
        f"OK  ledger: {len(p741)} P741-* items resolve"
        + (" (review items at archive/)" if args.location == "archive" else "")
        + note
    )

    print("ALL-ASSERTIONS-PASS")


if __name__ == "__main__":
    main()
