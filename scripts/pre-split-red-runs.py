#!/usr/bin/env python3
"""PLAN-867 T-02: bounded pre-split red-set collection (D-1 decision: rerun).

No batch receipt exists anywhere near the pinned pre-split commit (the last
one predates the route window by two weeks), so per the plan's evidence
priority the red baseline is collected by bounded targeted reruns on the same
read-only trees the denominator came from. This script runs each face's real
selector (`cargo nextest run` / bare `cargo test`), never regenerates
baselines, and writes per-face exit codes and failed-ID lists (the same
nextest FAIL-line / libtest FAILED-line extraction the tf --batch receipt
uses). th is deliberately absent: it is compile-broken on both sides of the
split (reference face only, see pre-split-universe/manifest.json).
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("enum", HERE / "scripts" / "enumerate-pre-split-universe.py")
enum = importlib.util.module_from_spec(spec)
spec.loader.exec_module(enum)
spec2 = importlib.util.spec_from_file_location("full", HERE / "scripts" / "run-full-regression.py")
full = importlib.util.module_from_spec(spec2)
spec2.loader.exec_module(full)

PROTECTED_REPORTS = ("docs/plans/reports/738-stdlib-matrix.json",
                     "docs/plans/reports/738-stdlib-matrix.md",
                     "docs/reports/plan796/turnaround-ipc.json",
                     "docs/reports/plan818/turnaround-mn-*.json",
                     "docs/reports/plan827/inventory.json")


def sha(data: bytes) -> str:
    import hashlib
    return hashlib.sha256(data).hexdigest()


def run_face(label: str, cwd: Path, log_dir: Path) -> dict:
    if label in enum.NEXTEST_FACES:
        cmd = ["cargo", "nextest", "run", *enum.NEXTEST_FACES[label], "--no-fail-fast"]
        package = None
    else:
        package, argv = enum.LIBTEST_FACES[label]
        run_args = argv[:argv.index("--")] if "--" in argv else argv
        cmd = ["cargo", "test", *run_args]
    log = log_dir / f"{label}-run.log"
    started = time.time()
    with log.open("w", encoding="utf-8") as output:
        proc = subprocess.Popen(cmd, cwd=str(cwd), stdout=output, stderr=subprocess.STDOUT,
                                start_new_session=sys.platform != "nt")
        code = proc.wait()
    failed = full.failed_from_log(log, package) if code else []
    return {"leg": label, "command": cmd, "exit": code, "failed": failed,
            "failed_count": len(failed), "duration_s": round(time.time() - started, 1),
            "log": str(log.resolve())}


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--pin", required=True)
    parser.add_argument("--faces", nargs="*", required=True)
    parser.add_argument("--out", type=Path, required=True, help="directory for run logs + summary.json")
    args = parser.parse_args(argv)

    head = subprocess.check_output(["git", "-C", str(args.root), "rev-parse", "HEAD"], text=True).strip()
    if not head.startswith(args.pin[:9]):
        raise SystemExit(f"refusing: {args.root} is {head[:9]}, expected {args.pin}")
    args.out.mkdir(parents=True, exist_ok=True)

    protected = {}
    for pattern in PROTECTED_REPORTS:
        for path in sorted(args.root.glob(pattern)):
            protected[str(path.relative_to(args.root)).replace("\\", "/")] = sha(path.read_bytes())

    summary = {"schema": "pre-split-red-runs/1", "commit": head, "root": str(args.root),
               "protected_reports": protected, "faces": {}}
    for label in args.faces:
        print(f"[red-run] {label} @ {head[:9]}", flush=True)
        summary["faces"][label] = run_face(label, args.root, args.out)
        row = summary["faces"][label]
        print(f"[red-run] {label}: exit={row['exit']} failed={row['failed_count']} ({row['duration_s']}s)", flush=True)

    for rel, before in protected.items():
        path = args.root / rel
        if sha(path.read_bytes()) != before:
            subprocess.check_call(["git", "-C", str(args.root), "checkout", "--", rel])
            summary.setdefault("restored_reports", []).append(rel)
    (args.out / "summary.json").write_text(json.dumps(summary, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    reds = sum(row["failed_count"] for row in summary["faces"].values())
    print(f"[red-run] total failed ids: {reds} → {args.out / 'summary.json'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
