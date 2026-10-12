#!/usr/bin/env python3
"""PLAN-741 Phase 7 (T-39/T-40) lifecycle assertions - v3.

Supersedes the phase5/phase6 versions per R6-QA-01/02:
- frontmatter required fields parsed and cross-checked against the UNIQUE
  task set: current_step == completed unique IDs, total_steps == unique
  total (derived from the current contract, never hardcoded);
- status/location consistency: archived implies docs/plans/archive/ with
  everything complete; active statuses imply docs/plans/;
- links verified in BOTH lifecycle states: the REAL active parent gets the
  active form (../reports/...), the SIMULATED ARCHIVE gets an explicitly
  converted copy (../../reports/...); both must pass - the simulation never
  stands in for the actual check (R5-QA-02);
- archive mode validates the REAL archived file (already converted at
  archival) from the real parent;
- EVERY P741-* review item in the ledger is checked dynamically (no ID
  windows); while active, archive-pointing items are transitional and are
  listed explicitly (refreshed by the merge);
- NEW (R6-QA-02): the final gate also covers the prototype README 741 link
  and the auto-hir/auto-ac module 741 navigation rows - status text and
  link form must match the lifecycle mode, links must resolve, with
  file/line in failures;
- --repo-root allows fixtures to run against a sandbox repo skeleton
  (negative controls) without touching the real repository.

Exit 0 = all assertions pass; 1 = any failure.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

SCRIPT_PARENT = Path(__file__).resolve().parent
DEFAULT_REPO = SCRIPT_PARENT.parents[2]
FENCE = re.compile(r"```.*?```", re.S)
INLINE = re.compile(r"`[^`]*`")
LINK = re.compile(r"\]\(([^)#\s]+)(?:#[^)]*)?\)")
TASK = re.compile(r"^- \[([ x])\] \*\*(T-\d+)\*\*", re.M)
ACTIVE_REPORT = "../reports/"
ARCHIVE_REPORT = "../../reports/"

PLAN_NAME = "741-ac-hir-native-core.md"
failures: list[str] = []


def fail(msg: str) -> None:
    failures.append(msg)


def report_and_exit() -> None:
    if failures:
        for f in failures:
            print(f"FAIL: {f}")
        sys.exit(1)
    print("ALL-ASSERTIONS-PASS")
    sys.exit(0)


def plan_links(text: str) -> list[str]:
    body = INLINE.sub("", FENCE.sub("", text))
    return [m.group(1) for m in LINK.finditer(body) if "://" not in m.group(1)]


def broken_links(links: list[str], base: Path) -> list[str]:
    return [
        f"{t} -> {(base / t).resolve()}"
        for t in links
        if not (base / t).resolve().exists()
    ]


def convert_to_archive(text: str) -> str:
    return text.replace(ACTIVE_REPORT, ARCHIVE_REPORT)


def convert_to_active(text: str) -> str:
    return text.replace(ARCHIVE_REPORT, ACTIVE_REPORT)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--plan", required=True)
    ap.add_argument("--location", required=True, choices=["active", "archive"])
    ap.add_argument("--repo-root", default=str(DEFAULT_REPO))
    args = ap.parse_args()
    repo = Path(args.repo_root).resolve()
    plan = Path(args.plan).resolve()
    if not plan.is_file():
        fail(f"plan file not found: {plan}")
        report_and_exit()
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
    if failures:
        report_and_exit()
    for field in ("plan_revision", "current_step", "total_steps"):
        if not re.fullmatch(r"\d+", fm[field]):
            fail(f"frontmatter {field} is not numeric: {fm[field]!r}")

    status = fm["status"]
    active_statuses = {"executing", "execution_done", "reviewed"}
    if args.location == "archive" and status != "archived":
        fail(f"archive mode requires status: archived, found {status}")
    if args.location == "active" and status not in active_statuses:
        fail(f"active mode requires status in {sorted(active_statuses)}, found {status}")

    # Location consistency for the REAL repo plan file; sandbox copies skip.
    expected_parent = repo / "docs" / "plans" / ("archive" if args.location == "archive" else "")
    real_repo_homes = (repo / "docs" / "plans", repo / "docs" / "plans" / "archive")
    is_repo_plan = parent == expected_parent
    if parent in real_repo_homes and not is_repo_plan:
        fail(f"status/location mismatch: {status} plan lives in {parent} "
             f"(--location {args.location} expects {expected_parent})")
    if is_repo_plan:
        print(f"OK  frontmatter: plan_id={fm.get('plan_id')} status={status} "
              f"revision={fm.get('plan_revision')} step={fm.get('current_step')}/"
              f"{fm.get('total_steps')}")
        print(f"OK  location: {parent} (repo plan, status {status})")
    else:
        print(f"NOTE fixture/external copy at {parent}: location check skipped")

    # --- 2. tasks -------------------------------------------------------------
    states: dict[str, set[str]] = {}
    for t in TASK.finditer(text):
        states.setdefault(t.group(2), set()).add(t.group(1))
    total_unique = len(states)
    mixed = sorted(t for t, ss in states.items() if len(ss) > 1)
    if mixed:
        fail(f"same-ID rows with conflicting checkbox state: {mixed}")
    completed = sorted(t for t, ss in states.items() if "x" in ss)
    unchecked = sorted(t for t, ss in states.items() if ss == {" "})
    if int(fm.get("total_steps", "-1")) != total_unique:
        fail(f"total_steps={fm.get('total_steps')} != unique task count {total_unique}")
    if int(fm.get("current_step", "-1")) != len(completed):
        fail(f"current_step={fm.get('current_step')} != completed unique tasks {len(completed)}")
    if args.location == "archive" and unchecked:
        fail(f"archived plan must be fully complete; unchecked={unchecked}")
    if failures:
        report_and_exit()
    print(f"OK  tasks: {len(completed)}/{total_unique} complete (matches frontmatter), "
          f"duplicates consistent, unchecked={len(unchecked)}")

    # --- 3. plan links in both lifecycle states --------------------------------
    links = plan_links(text)
    active_home = repo / "docs" / "plans"
    archive_home = repo / "docs" / "plans" / "archive"
    if args.location == "active":
        # REAL active parent: active-form links must resolve.
        real = broken_links(links, active_home)
        if real:
            fail(f"{len(real)} link(s) broken from the REAL active parent {active_home}:\n  "
                 + "\n  ".join(real))
        else:
            print(f"OK  links (real active parent {active_home}): {len(links)} resolve")
        # SIMULATED archive: converted copy against the archive home.
        converted = plan_links(convert_to_archive(text))
        sim = broken_links(converted, archive_home)
        if sim:
            fail(f"{len(sim)} link(s) broken in the SIMULATED archive conversion:\n  "
                 + "\n  ".join(sim))
        else:
            print(f"OK  links (simulated archive parent): {len(converted)} resolve after conversion")
    else:
        # Archive mode: the real file carries converted links already.
        stale = [t for t in links if t.startswith(ACTIVE_REPORT)]
        broken = broken_links(links, archive_home)
        if stale or broken:
            allb = [f"stale active-form link: {t}" for t in stale] + broken
            fail(f"{len(allb)} link problem(s) from the REAL archive parent {archive_home}:\n  "
                 + "\n  ".join(allb))
        else:
            print(f"OK  links: {len(links)} resolve from the real archive parent")

    # --- 4. navigation: prototype README + module 741 rows (R6-QA-02) --------
    # R7-QA-02: every 741 reference must resolve to EXACTLY the canonical
    # PLAN-741 lifecycle file; a missing reference or a reference to any
    # other target is a failure with file/line named.
    canonical = repo / "docs" / "plans" / ("archive" if args.location == "archive" else "") / PLAN_NAME
    canonical_archive = repo / "docs" / "plans" / "archive" / PLAN_NAME
    if args.location == "archive" and not canonical.exists():
        fail(f"canonical archived plan missing: {canonical}")
    readme = repo / "experimental/ac-core/README.md"
    if readme.is_file():
        rtext = readme.read_text(encoding="utf8")
        refs = []
        for lineno, line in enumerate(rtext.splitlines(), 1):
            if "741-ac-hir-native-core.md" in line and "](" in line:
                m = LINK.search(line)
                if not m:
                    continue
                refs.append((lineno, m, (readme.parent / m.group(1)).resolve()))
        if not refs:
            fail("prototype README missing required 741 plan reference (R7-QA-02)")
        for lineno, m, target in refs:
            at_archive = "archive" in target.parts
            if args.location == "active" and at_archive:
                fail(f"README.md:{lineno}: plan link points at archive/ while active "
                     f"(lifecycle mismatch): {m.group(1)}")
            if args.location == "archive" and not at_archive:
                fail(f"README.md:{lineno}: plan link not archive-form after archival: "
                     f"{m.group(1)}")
            if target != canonical:
                fail(f"README.md:{lineno}: 741 reference does not match the canonical plan "
                     f"target (expected {canonical}, got {target})")
            if not target.exists():
                fail(f"README.md:{lineno}: link unresolvable: {m.group(1)}")
            else:
                print(f"OK  README.md:{lineno} plan link -> {target}")
    else:
        fail(f"prototype README missing: {readme}")

    for mod in ("auto-hir", "auto-ac"):
        nav = repo / f"docs/specs/{mod}/plans.md"
        if not nav.is_file():
            fail(f"module nav missing: {nav}")
            continue
        ntext = nav.read_text(encoding="utf8")
        hit = None
        for lineno, line in enumerate(ntext.splitlines(), 1):
            if line.startswith("| 741 |"):
                hit = (lineno, line)
                break
        if not hit:
            fail(f"{mod}/plans.md: no 741 navigation row found")
            continue
        lineno, line = hit
        m = LINK.search(line)
        if not m:
            fail(f"{mod}/plans.md:{lineno}: 741 row has no markdown link")
            continue
        target = (nav.parent / m.group(1)).resolve()
        at_archive = "archive" in target.parts
        says_delivered = "delivered" in line
        # R7-QA-02: the nav row must resolve to EXACTLY the canonical plan.
        if target != canonical:
            fail(f"{mod}/plans.md:{lineno}: nav target does not match the canonical plan "
                 f"(expected {canonical}, got {target})")
        if args.location == "archive":
            if not at_archive:
                fail(f"{mod}/plans.md:{lineno}: nav link not archive-form after archival: "
                     f"{m.group(1)}")
            if not says_delivered:
                fail(f"{mod}/plans.md:{lineno}: nav status must say delivered after archival: "
                     f"{line.strip()[:120]}")
            if "executing" in line:
                fail(f"{mod}/plans.md:{lineno}: nav still says executing after archival")
        else:
            if at_archive:
                fail(f"{mod}/plans.md:{lineno}: nav link points at archive/ while active: "
                     f"{m.group(1)}")
            if says_delivered:
                fail(f"{mod}/plans.md:{lineno}: nav says delivered while active "
                     f"(lifecycle mismatch)")
        if not target.exists():
            fail(f"{mod}/plans.md:{lineno}: nav link unresolvable: {m.group(1)}")
        elif not failures or all("unresolvable" not in f for f in failures):
            print(f"OK  {mod}/plans.md:{lineno} 741 row -> {target}")

    # --- 5. ledger: every P741-* review item ----------------------------------
    ledger = repo / ".autoos/specs.json"
    if not ledger.is_file():
        fail(f"ledger missing: {ledger}")
    else:
        data = json.loads(ledger.read_text(encoding="utf8"))
        review_items = [
            it
            for sec in data["sections"]
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
                target = (repo / it["file"]).resolve()
                at_archive = "archive" in target.parts
                # R7-QA-02: every review pointer must resolve to EXACTLY the
                # canonical plan target (entry ID named on mismatch).
                if target != canonical:
                    if args.location == "active" and target == canonical_archive:
                        transitional.append(f"{it['id']} -> {it['file']}")
                        continue
                    broken.append(f"{it['id']} does not match the canonical plan target "
                                  f"(expected {canonical}, got {it['file']})")
                    continue
                if target.exists():
                    if args.location == "archive" and not at_archive:
                        broken.append(f"{it['id']} not archive-pointing: {it['file']}")
                    continue
                if args.location == "active" and at_archive:
                    transitional.append(f"{it['id']} -> {it['file']}")
                else:
                    broken.append(f"{it['id']} unresolvable: {it['file']}")
            if broken:
                fail(f"{len(broken)} ledger review item(s) broken:\n  "
                     + "\n  ".join(broken))
            else:
                msg = f"OK  ledger: {len(review_items)} P741-* review items checked"
                if transitional:
                    msg += (f" ({len(transitional)} transitional archive-pointing items, "
                            f"refreshed by this merge): {'; '.join(transitional)}")
                elif args.location == "archive":
                    msg += " (all at archive/)"
                print(msg)

    report_and_exit()


if __name__ == "__main__":
    main()
