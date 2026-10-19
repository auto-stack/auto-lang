#!/usr/bin/env python3
"""PLAN-810 F810-R2-01: isolated-process probe for the unterminated-fragment
unbounded-EOF quirk (baseline behavior, frozen — NOT a regression).

Runs the #[ignore]d plan810 test `r02_fragment_unterminated_eof_quirk_isolated_only`
in a child `cargo test` process with an external deadline, records exit code /
elapsed / command to a JSON receipt. The in-suite test never runs normally
(#[ignore]); this probe is the only sanctioned way to observe the quirk.

Usage: python scripts/plan810_eof_quirk_probe.py [--deadline-secs 5]
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--deadline-secs", type=float, default=5.0)
    ap.add_argument("--out", default="docs/plans/attachments/810/eof-quirk-isolated.json")
    args = ap.parse_args()

    cmd = [
        "cargo", "test", "-p", "auto-lang", "--lib",
        "r02_fragment_unterminated_eof_quirk_isolated_only",
        "--features", "ui-iced,test-cases", "--", "--ignored", "--exact",
        "tests::plan810_full_ui_grammar::r02_fragment_unterminated_eof_quirk_isolated_only",
        "--nocapture",
    ]
    t0 = time.monotonic()
    try:
        proc = subprocess.run(
            cmd, cwd=ROOT, timeout=args.deadline_secs,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
        )
        elapsed = round(time.monotonic() - t0, 2)
        receipt = {
            "probe": "plan810 eof quirk isolated observation",
            "finding": "F810-R2-01",
            "input": "view fn F(x: int) { if x {  (unterminated fragment, EOF)",
            "command": " ".join(cmd[:2] + ["…", "-- --ignored --exact"]),
            "deadline_secs": args.deadline_secs,
            "outcome": "returned",
            "exit_code": proc.returncode,
            "elapsed_secs": elapsed,
            "head_commit": subprocess.run(
                ["git", "rev-parse", "HEAD"], cwd=ROOT,
                capture_output=True, text=True).stdout.strip(),
            "note": (
                "exit_code != 0 with fast return = baseline unbounded-EOF fail-fast "
                "(stack overflow / abort); elapsed >= deadline = hang killed externally. "
                "Either way the in-process suites must not run this input (#[ignore])."
            ),
            "tail": (proc.stdout or "")[-400:],
        }
    except subprocess.TimeoutExpired as e:
        elapsed = round(time.monotonic() - t0, 2)
        receipt = {
            "probe": "plan810 eof quirk isolated observation",
            "finding": "F810-R2-01",
            "input": "view fn F(x: int) { if x {  (unterminated fragment, EOF)",
            "command": "cargo test … -- --ignored --exact --nocapture",
            "deadline_secs": args.deadline_secs,
            "outcome": "timeout_killed",
            "exit_code": None,
            "elapsed_secs": elapsed,
            "head_commit": subprocess.run(
                ["git", "rev-parse", "HEAD"], cwd=ROOT,
                capture_output=True, text=True).stdout.strip(),
            "note": "no convergence within external deadline; child killed and reaped",
            "tail": ((e.stdout or b"").decode(errors="replace")[-400:]
                     if isinstance(e.stdout, bytes) else str(e.stdout or "")[-400:]),
        }

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(receipt, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(receipt, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
