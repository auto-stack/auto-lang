"""Active-entry author-format scan for fixed case groups (PLAN-1066 T-09).

Every fixed group's real `suite` selects the case files that a daily run
actually executes. Since the AutoDown migration, those files must be
document-format author files (`## Case:` headings); a legacy `test-case`
fence may stay on disk as frozen history but must never remain an INCLUDED
entry, and a file that is neither form is a poisoned include.

This module only reads: it reports group / suite / case for every offender so
a migration gap cannot hide behind a "frozen history" file that is still
wired into a suite. The scan is the active-entry half of the migration
closure; per-entry payload migration is verified by the consumers themselves.
"""
from __future__ import annotations

import re
import tomllib
from pathlib import Path

from governance_io import ROOT

FENCE_OPEN = re.compile(r"^(`{3,}|~{3,})")
CASE_HEADING = re.compile(r"^## Case: ", re.M)
FENCE_CASE = re.compile(r"^(`{3,}|~{3,})test-case", re.M)
SUITE_FENCE = re.compile(r"^(`{3,}|~{3,})test-suite\s*\n(.*?)^\1\s*$", re.M | re.S)
INCLUDE = re.compile(r"include:\s*\[(.*?)\]", re.S)


def top_level_text(text: str) -> str:
    """Drop fenced blocks so format markers are only read at the top level."""
    out: list[str] = []
    fence: str | None = None
    for line in text.splitlines():
        stripped = line.strip()
        match = FENCE_OPEN.match(stripped)
        if fence is None:
            if match:
                fence = match.group(1)[0] * len(match.group(1))
                continue
        else:
            if stripped.startswith(fence) and stripped.strip(fence[0]) == "":
                fence = None
                continue
            continue
        out.append(line)
    return "\n".join(out)


def classify(path: Path) -> str:
    text = path.read_text(encoding="utf-8", errors="replace")
    if CASE_HEADING.search(top_level_text(text)):
        return "document"
    if FENCE_CASE.search(text):
        return "fence"
    return "unknown"


def suite_includes(suite: Path) -> list[str]:
    text = suite.read_text(encoding="utf-8", errors="replace")
    match = SUITE_FENCE.search(text)
    body = match.group(2) if match else text
    include = INCLUDE.search(body)
    if not include:
        return []
    return re.findall(r'"([^"]+)"', include.group(1))


def scan(root: Path | None = None) -> dict:
    """Classify every INCLUDED case file of every fixed group's suite."""
    root = (root or ROOT).resolve()
    document = tomllib.loads((root / "scripts/test-groups.toml").read_text(encoding="utf-8"))
    groups = document.get("group", [])
    formats = {"document": 0, "fence": 0, "unknown": 0}
    offenders: list[dict] = []
    missing: list[dict] = []
    scanned_groups = 0
    for group in groups:
        suite_rel = group.get("suite")
        if not suite_rel:
            continue
        suite = root / suite_rel
        if not suite.is_file():
            missing.append({"group": group["name"], "suite": suite_rel, "reason": "suite missing"})
            continue
        scanned_groups += 1
        for include in suite_includes(suite):
            case = suite.parent / include
            rel = case.relative_to(root).as_posix() if case.is_relative_to(root) else str(case)
            if not case.is_file():
                missing.append({"group": group["name"], "suite": suite_rel,
                                "case": rel, "reason": "include missing"})
                continue
            kind = classify(case)
            formats[kind] += 1
            if kind != "document":
                offenders.append({"group": group["name"], "suite": suite_rel,
                                  "case": rel, "format": kind})
    return {
        "schema": "test-document-active-entries/1",
        "groups": scanned_groups,
        "files": sum(formats.values()),
        "formats": formats,
        "offenders": offenders,
        "missing": missing,
    }


def report(result: dict) -> str:
    lines = [
        "groups with suite: %d; included case files: %d; formats=%s"
        % (result["groups"], result["files"], result["formats"])
    ]
    for row in result["offenders"]:
        lines.append("OFFENDER[%s] group=%s suite=%s case=%s"
                     % (row["format"], row["group"], row["suite"], row["case"]))
    for row in result["missing"]:
        lines.append("MISSING group=%s suite=%s case=%s (%s)"
                     % (row["group"], row["suite"], row.get("case", "-"), row["reason"]))
    lines.append("offenders=%d missing=%d" % (len(result["offenders"]), len(result["missing"])))
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    import sys

    argv = argv if argv is not None else sys.argv[1:]
    root = Path(argv[0]).resolve() if argv else None
    result = scan(root)
    print(report(result))
    return 1 if result["offenders"] or result["missing"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
