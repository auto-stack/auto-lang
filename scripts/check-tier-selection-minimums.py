#!/usr/bin/env python3
"""PLAN-867 T-05: per-tier minimum-selection guard across the three repositories.

Zero or collapsed tier selections must be loud: a green run that silently
selected nothing (the retired language `tu` alias pattern) is worse than red.
This script enumerates each tier's real selection (nextest list / libtest
-- --list — it never runs tests) and asserts the frozen minimum from
scripts/tier-selection-minimums.json.txt, the same data file the tf --batch
receipt asserts. Exit 0 = every tier at or above its floor; nonzero = the
report lists each violation with its leg, count and floor.
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
MINIMUMS = HERE / "scripts" / "tier-selection-minimums.json.txt"


def load():
    doc = json.loads(MINIMUMS.read_text(encoding="utf-8"))
    if doc.get("schema") != "tier-selection-minimums/1":
        raise SystemExit(f"unsupported schema: {doc.get('schema')}")
    return doc


def resolve_root(name):
    spec = importlib.util.spec_from_file_location("full", HERE / "scripts" / "run-full-regression.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.root(name)


def nextest_count(cmd, cwd):
    proc = subprocess.run(cmd + ["--message-format", "json"], cwd=str(cwd), capture_output=True)
    if proc.returncode:
        return None, f"exit {proc.returncode}: {proc.stderr.decode(errors='replace')[-400:]}"
    data = json.loads(proc.stdout)
    ids, selected, ignored = 0, 0, 0
    for value in data["rust-suites"].values():
        for case in value["testcases"].values():
            ids += 1
            if case["ignored"]:
                ignored += 1
            elif case["filter-match"]["status"] == "matches":
                selected += 1
    return {"ids": ids, "selected": selected, "ignored": ignored}, None


def libtest_count(cmd, cwd):
    proc = subprocess.run(cmd, cwd=str(cwd), capture_output=True)
    if proc.returncode:
        return None, f"exit {proc.returncode}: {proc.stderr.decode(errors='replace')[-400:]}"
    ids, ignored = 0, 0
    for line in proc.stdout.decode("utf-8", "replace").splitlines():
        if line.rstrip().endswith(": test"):
            ids += 1
        elif line.rstrip().endswith(": ignore"):
            ids += 1
            ignored += 1
    return {"ids": ids, "selected": ids - ignored, "ignored": ignored}, None


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--roots", nargs="*", default=[],
                        help="repo roots, e.g. --roots auto-lang=D:/autostack/auto-lang auto-ui=... auto-man=...; "
                             "default: sibling/main-checkout resolution like run-full-regression.py")
    parser.add_argument("--tiers", nargs="*", help="subset of tier labels (default: all)")
    parser.add_argument("--output", type=Path, help="write the JSON report to this path")
    args = parser.parse_args(argv)

    doc = load()
    roots = dict(item.split("=", 1) for item in args.roots) if args.roots else {}
    repos = {entry["repo"] for entry in doc["legs"].values()}
    repo_roots = {repo: (Path(roots[repo]) if repo in roots else resolve_root(repo)) for repo in repos}

    labels = args.tiers or list(doc["legs"])
    unknown = [t for t in labels if t not in doc["legs"]]
    if unknown:
        raise SystemExit(f"unknown tiers: {unknown}")
    report, failures = {}, []
    for label in labels:
        entry = doc["legs"][label]
        if "command" not in entry:
            # legs whose floor is asserted by the tf --batch receipt itself
            print(f"[skip] {label}: floor {entry['minimum']} asserted by {entry.get('via', 'tf --batch receipt')}")
            continue
        cwd = repo_roots[entry["repo"]]
        command = ["cargo", *entry["command"]]
        if entry["kind"] == "nextest":
            counts, error = nextest_count(command, cwd)
        else:
            counts, error = libtest_count(command, cwd)
        row = {"repo": entry["repo"], "root": str(cwd), "command": command,
               "minimum": entry["minimum"], "owner": entry.get("owner", ""), "source": entry.get("source", "")}
        if error:
            row["error"] = error
            failures.append(label)
        else:
            row.update(counts)
            if counts["selected"] < entry["minimum"]:
                row["violation"] = f"selected {counts['selected']} < minimum {entry['minimum']}"
                failures.append(label)
        report[label] = row
        status = "FAIL" if label in failures else "ok"
        print(f"[{status}] {label}: {row.get('selected', '?')} selected (minimum {entry['minimum']})", flush=True)
    result = {"schema": "tier-selection-minimums-report/1", "failures": failures, "tiers": report}
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
