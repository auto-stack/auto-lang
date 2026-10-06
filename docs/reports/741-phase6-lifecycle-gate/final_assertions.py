#!/usr/bin/env python3
"""PLAN-741 Phase 6 (T-36) lifecycle assertions - replaces the phase5
final_assertions.py per R5-QA-01/02.

Contract (plan Phase 6 detailed design):
1. Frontmatter required fields are parsed and cross-checked against the
   UNIQUE task set: current_step == number of completed unique task IDs,
   total_steps == total unique task IDs (derived from the current contract,
   never a hardcoded 35/38). Missing/non-numeric fields, wrong counters,
   duplicate-row checkbox conflicts, or uncompleted tasks under an archived
   status are hard failures. executing allows an ACCURATE incomplete count.
2. Status/location consistency: archived implies the file lives under
   docs/plans/archive/ with everything complete; active/executing/
   execution_done/reviewed imply docs/plans/.
3. Links are checked in BOTH states required by AC-26:
   - REAL active parent: the active-form links (../reports/...) must resolve
     from docs/plans/.
   - SIMULATED archive: an explicitly CONVERTED copy of the text
     (../reports/ -> ../../reports/) must resolve from docs/plans/archive/.
   - archive mode: the real file (already converted at archival) resolves
     from the real parent.
   A simulated check is never passed off as the actual active check: both
   results are printed and both must pass.
4. Ledger: EVERY P741-* review item (any ID whose file lives under
   docs/plans/) is checked - no hardcoded ID windows. While active,
   archive-pointing review items are transitional and are explicitly listed
   (they are refreshed by the merge); at archive all must resolve at the
   archive path.

Exit code 0 = all assertions pass; 1 = any failure (reasons printed).
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
FENCE = re.compile(r"```.*?```", re.S)
INLINE = re.compile(r"`[^`]*`")
LINK = re.compile(r"\]\(([^)#\s]+)(?:#[^)]*)?\)")
TASK = re.compile(r"^- \[([ x])\] \*\*(T-\d+)\*\*", re.M)
ACTIVE_REPORT = "../reports/"
ARCHIVE_REPORT = "../../reports/"

failures: list[str] = []


def fail(msg: str) -> None:
    failures.append(msg)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--plan", required=True)
    ap.add_argument(
        "--location",
        required=True,
        choices=["active", "archive"],
    )
    args = ap.parse_args()
    plan = Path(args.plan).resolve()
    if not plan.is_file():
        fail(f"plan file not found: {plan}")
        report()
        return
    parent = plan.parent
    text = plan.read_text(encoding="utf8")

    # --- 1. frontmatter ------------------------------------------------------
    fm: dict[str, str] = {}
    m = re.match(r"^---\r?\n(.*?)\r?\n---", text, re.S)
    if not m:
        fail("frontmatter block missing")
    else:
        for line in m.group(1).splitlines():
            kv = re.match(r"^([a-z_]+):\s*(\S+)", line)
            if kv:
                fm[kv.group(1)] = kv.group(2)
    for field in ("plan_id", "status", "plan_revision", "current_step", "total_steps"):
        if field not in fm:
            fail(f"frontmatter missing required field: {field}")
    report()
    if failures:
        return
    for field in ("plan_revision", "current_step", "total_steps"):
        if not re.fullmatch(r"\d+", fm[field]):
            fail(f"frontmatter {field} is not numeric: {fm[field]!r}")

    status = fm["status"]
    active_statuses = {"executing", "execution_done", "reviewed"}
    if args.location == "archive" and status != "archived":
        fail(f"archive mode requires status: archived, found {status}")
    if args.location == "active" and status not in active_statuses:
        fail(f"active mode requires status in {sorted(active_statuses)}, found {status}")
    # Location consistency is enforced for the REAL repo plan file; fixture
    # copies in temp dirs skip it (their parent is the fixture sandbox) but
    # still go through every other assertion.
    expected_parent = REPO / "docs" / "plans" / ("archive" if args.location == "archive" else "")
    real_repo_homes = (REPO / "docs" / "plans", REPO / "docs" / "plans" / "archive")
    is_repo_plan = parent == expected_parent
    if parent in real_repo_homes and not is_repo_plan:
        # P741P6-R2: a REAL repo location that disagrees with --location is a
        # lifecycle contradiction (e.g. an archived-status file left in
        # docs/plans/), never a sandbox copy - hard failure.
        fail(f"status/location mismatch: {status} plan lives in {parent} "
             f"(--location {args.location} expects {expected_parent})")
    if is_repo_plan:
        print(f"OK  frontmatter: plan_id={fm.get('plan_id')} status={status} "
              f"revision={fm.get('plan_revision')} step={fm.get('current_step')}/"
              f"{fm.get('total_steps')}")
        print(f"OK  location: {parent} (repo plan, status {status})")
    else:
        print(f"NOTE fixture/external copy at {parent}: location check skipped")
    if failures:
        report()
        return

    # --- 2. tasks: derive totals, cross-check counters -----------------------
    states: dict[str, set[str]] = {}
    for t in TASK.finditer(text):
        states.setdefault(t.group(2), set()).add(t.group(1))
    total_unique = len(states)
    mixed = sorted(t for t, ss in states.items() if len(ss) > 1)
    if mixed:
        fail(f"same-ID rows with conflicting checkbox state: {mixed}")
    checked_unique = sorted(t for t, ss in states.items() if "x" in ss)
    completed = len(checked_unique)
    unchecked = sorted(t for t, ss in states.items() if ss == {" "})

    if int(fm["total_steps"]) != total_unique:
        fail(f"total_steps={fm['total_steps']} != unique task count {total_unique}")
    if int(fm["current_step"]) != completed:
        fail(f"current_step={fm['current_step']} != completed unique tasks {completed}")
    if args.location == "archive":
        if status != "archived" or unchecked:
            fail(f"archived plan must be fully complete; unchecked={unchecked}")
    print(f"OK  tasks: {completed}/{total_unique} complete (matches frontmatter), "
          f"duplicates consistent, unchecked={len(unchecked)}")
    if failures:
        report()
        return

    # --- 3. links: real active parent AND simulated archive ------------------
    # Active mode judges active-form links against the CANONICAL active home
    # (docs/plans/) - identical to the real parent for the repo file, and
    # the correct base for sandbox copies - plus the CONVERTED copy against
    # the archive home. Archive mode judges the real file (already
    # converted at archival) against the real archive parent.
    body = FENCE.sub("", text)
    body = INLINE.sub("", body)
    links = [m.group(1) for m in LINK.finditer(body) if "://" not in m.group(1)]
    active_home = REPO / "docs" / "plans"
    archive_home = REPO / "docs" / "plans" / "archive"
    if args.location == "active":
        real_broken = [
            f"{t} -> {(active_home / t).resolve()}"
            for t in links
            if not (active_home / t).resolve().exists()
        ]
        if real_broken:
            fail(f"{len(real_broken)} link(s) broken from the REAL active parent {active_home}:\n  "
                 + "\n  ".join(real_broken))
        else:
            print(f"OK  links (real active parent {active_home}): {len(links)} resolve")
        sim_text = text.replace(ACTIVE_REPORT, ARCHIVE_REPORT)
        sim_body = INLINE.sub("", FENCE.sub("", sim_text))
        sim_broken = [
            f"{t} -> {(archive_home / t).resolve()}"
            for t in (mm.group(1) for mm in LINK.finditer(sim_body))
            if "://" not in t and not (archive_home / t).resolve().exists()
        ]
        if sim_broken:
            fail(f"{len(sim_broken)} link(s) broken in the SIMULATED archive conversion:\n  "
                 + "\n  ".join(sim_broken))
        else:
            print(f"OK  links (simulated archive parent): {len(links)} resolve after conversion")
    else:
        real_broken = [
            f"{t} -> {(archive_home / t).resolve()}"
            for t in links
            if t.startswith(ACTIVE_REPORT) or not (archive_home / t).resolve().exists()
        ]
        if real_broken:
            fail(f"{len(real_broken)} link(s) broken from the REAL archive parent {archive_home}:\n  "
                 + "\n  ".join(real_broken))
        else:
            print(f"OK  links: {len(links)} resolve from the real archive parent")
    if failures:
        report()
        return

    # --- 4. ledger: every P741-* review item ---------------------------------
    ledger = json.loads((REPO / ".autoos/specs.json").read_text(encoding="utf8"))
    review_items = [
        it
        for sec in ledger["sections"]
        for it in sec.get("items", [])
        if it.get("id", "").startswith("P741-")
        and "docs/plans" in it.get("file", "")
    ]
    if not review_items:
        fail("ledger has no P741-* review items")
    else:
        transitional = []
        broken = []
        for it in review_items:
            target = (REPO / it["file"]).resolve()
            at_archive = "archive" in target.parts
            if target.exists():
                if args.location == "archive" and not at_archive:
                    broken.append(f"{it['id']} not archive-pointing: {it['file']}")
                continue
            # Unresolvable: only tolerable as the ACTIVE-phase transitional
            # state, and only for archive-pointing review items awaiting the
            # merge; listed explicitly, never silently passed.
            if args.location == "active" and at_archive:
                transitional.append(f"{it['id']} -> {it['file']}")
            else:
                broken.append(f"{it['id']} unresolvable: {it['file']}")
        if broken:
            fail(f"{len(broken)} ledger review item(s) broken:\n  " + "\n  ".join(broken))
        else:
            msg = f"OK  ledger: {len(review_items)} P741-* review items checked"
            if transitional:
                msg += (f" ({len(transitional)} transitional archive-pointing items, "
                        f"refreshed by this merge): {'; '.join(transitional)}")
            elif args.location == "archive":
                msg += " (all at archive/)"
            print(msg)
    report()
    if not failures:
        print("ALL-ASSERTIONS-PASS")


def report() -> None:
    if failures:
        for f in failures:
            print(f"FAIL: {f}")
        sys.exit(1)


if __name__ == "__main__":
    main()
