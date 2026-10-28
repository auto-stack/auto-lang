#!/usr/bin/env python3
"""structural-packages-check.py — PLAN-869 T-05 / V-06 oracle.

Validates docs/plans/attachments/869/structural-packages.md: every hotspot
package (## S<n> section) must carry all seven required elements
(规模/职责/拆分假设/触面档/前置/风险/审查面), each non-empty. Missing element
or empty bullet => nonzero exit. Exit 0 = all packages complete."""
import re
import sys
from pathlib import Path

DOC = Path(__file__).resolve().parents[1] / \
    "docs/plans/attachments/869/structural-packages.md"
ELEMENTS = ["规模", "职责与耦合", "拆分假设", "触面档", "前置", "主要风险", "审查面"]


def main():
    if not DOC.exists():
        print(f"CHECK FAIL: {DOC} missing")
        return 1
    text = DOC.read_text(encoding="utf-8")
    sections = re.split(r"^## ", text, flags=re.M)[1:]
    packages = [s for s in sections if s.startswith("S")]
    if len(packages) < 6:
        print(f"CHECK FAIL: expected >=6 hotspot packages, found {len(packages)}")
        return 1
    fail = 0
    for pkg in packages:
        title = pkg.splitlines()[0]
        for el in ELEMENTS:
            m = re.search(r"[-*]\s*\*\*" + el + r"\*\*[:：]?\s*(.+)", pkg)
            if not m or not m.group(1).strip():
                print(f"CHECK FAIL: [{title}] element '{el}' missing/empty")
                fail += 1
    if fail:
        print(f"CHECK FAIL: {fail} missing elements across {len(packages)} packages")
        return 1
    print(f"CHECK PASS: {len(packages)} packages x 7 elements complete")
    return 0


if __name__ == "__main__":
    sys.exit(main())
