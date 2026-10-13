#!/usr/bin/env python3
"""PLAN-741 Phase 6 (T-36) controlled positive/negative fixtures.

Builds mutated copies of the plan in a temp directory and asserts the
final_assertions exit codes - real subprocess evidence, not assertions
about assertions. Mirrors the R5-QA-01 probe matrix (current_step=0,
total_steps=999, missing fields) plus the R5-QA-02 active-parent defect.
"""
from __future__ import annotations

import re
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
SCRIPT = Path(__file__).resolve().parent / "final_assertions.py"
PLAN = REPO / "docs/plans/741-ac-hir-native-core.md"


def run(plan_path: Path, location: str) -> int:
    return subprocess.run(
        [sys.executable, str(SCRIPT), "--plan", str(plan_path), "--location", location],
        capture_output=True,
        text=True,
    ).returncode


def mutated(name: str, transform) -> Path:
    work = Path(tempfile.mkdtemp(prefix="p741p6-fixture-"))
    dst = work / "plans"
    dst.mkdir(parents=True)
    text = PLAN.read_text(encoding="utf8")
    text = transform(text)
    (dst / PLAN.name).write_text(text, encoding="utf8")
    return dst / PLAN.name


def set_fm(text: str, field: str, value: str) -> str:
    return re.sub(rf"^{field}: \S+", f"{field}: {value}", text, count=1, flags=re.M)


def drop_fm(text: str, field: str) -> str:
    return re.sub(rf"^{field}: \S+\n", "", text, count=1, flags=re.M)


CASES: list[tuple[str, str, int]] = [
    # (name, location, expected exit)
    ("valid-active", "active", 0),
    ("negative-current-step-zero", "active", 1),
    ("negative-total-steps-999", "active", 1),
    ("negative-missing-current-step", "active", 1),
    ("negative-nonnumeric-revision", "active", 1),
    ("negative-archived-status-in-active-location", "active", 1),
    ("negative-conflicting-duplicate-checkbox", "active", 1),
    ("negative-active-with-archive-form-links", "active", 1),
]


def main() -> None:
    transforms = {
        "valid-active": lambda t: t,
        "negative-current-step-zero": lambda t: set_fm(t, "current_step", "0"),
        "negative-total-steps-999": lambda t: set_fm(t, "total_steps", "999"),
        "negative-missing-current-step": lambda t: drop_fm(t, "current_step"),
        "negative-nonnumeric-revision": lambda t: set_fm(t, "plan_revision", "r6"),
        "negative-archived-status-in-active-location": lambda t: set_fm(t, "status", "archived"),
        # Same-ID rows must never disagree (R5-QA-01 duplicate conflict):
        # the reopened T-35 rows are both unchecked in the active plan, so
        # flipping exactly ONE creates the conflict.
        "negative-conflicting-duplicate-checkbox": lambda t: t.replace(
            "- [ ] **T-35**", "- [x] **T-35**", 1
        ),
        # R5-QA-02: an active plan carrying ARCHIVE-form report links is
        # broken from the real active parent and must not pass.
        "negative-active-with-archive-form-links": lambda t: t.replace(
            "](../reports/", "](../../reports/"
        ),
    }
    failed = False
    for name, location, expected in CASES:
        plan = mutated(name, transforms[name])
        code = run(plan, location)
        ok = code == expected
        print(f"{'PASS' if ok else 'FAIL'}  {name}: exit {code} (expected {expected})")
        if not ok:
            failed = True
    print("FIXTURES-ALL-PASS" if not failed else "FIXTURES-FAILED")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
