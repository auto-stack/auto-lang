#!/usr/bin/env python3
"""Plan 564 T2: per-test peak-memory measurement harness.

Runs the matching tests ONE AT A TIME under nextest (--jobs=1) — each nextest
test executes in its own process — while a PowerShell watcher samples
Win32_Process.PeakWorkingSetSize for auto_lang-* child processes. The child's
command line carries the exact test id, giving precise per-test attribution.

Usage:
  python scripts/measure_test_mem.py <filter> [-F test-vm-files] [--ignored]
  python scripts/measure_test_mem.py aavm2_ -F test-vm-files
  python scripts/measure_test_mem.py str_churn --ignored

Output: markdown table sorted by peak desc, with tier classification
(Plan 564 D2: XL>=800MB, LG>=300MB, MD>=100MB, LT<100MB). If
.config/test-mem-weights.md already contains a row for a test, drift vs the
registered value is annotated (drift > 50% highlighted for review).
"""
import argparse
import re
import subprocess
import sys
import threading
import time
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
WEIGHTS_MD = REPO / ".config" / "test-mem-weights.md"

WATCHER_PS = r"""
while ($true) {
  Get-CimInstance Win32_Process -Filter "Name LIKE 'auto_lang%'" -ErrorAction SilentlyContinue | ForEach-Object {
    $cl = ''
    try { $cl = $_.CommandLine } catch {}
    "$($_.ProcessId)`t$($_.WorkingSetSize)`t$cl"
  }
  Start-Sleep -Milliseconds 120
}
"""


def watcher_thread(samples: dict, stop: threading.Event, proc, repo: str):
    """Read the watcher PowerShell stdout, keeping max peak per PID.

    Only children whose command line references THIS worktree's target dir are
    counted — other sessions' concurrent test runs in sibling worktrees must
    not pollute the report. WorkingSetSize is polled (bytes) rather than
    Win32_Process.PeakWorkingSetSize whose unit is docs-ambiguous (KB).
    """
    for line in proc.stdout:
        line = line.strip()
        if not line:
            continue
        parts = line.split("\t", 2)
        if len(parts) < 2:
            continue
        pid, ws = parts[0], parts[1]
        cmd = parts[2] if len(parts) > 2 else ""
        if repo.lower() not in cmd.lower():
            continue  # belongs to another worktree/session
        try:
            ws = int(ws)
        except ValueError:
            continue
        rec = samples.get(pid)
        if rec is None:
            samples[pid] = {"peak": ws, "cmd": cmd}
        elif ws > rec["peak"]:
            rec["peak"] = ws
    stop.set()


def test_name_from_cmd(cmd: str) -> str:
    """nextest invokes children as `<exe> --exact <testid> ...`."""
    m = re.search(r"--exact\s+(\S+)", cmd)
    if m:
        return m.group(1)
    # fallback: longest token that looks like a test path
    cands = [t for t in cmd.split() if "::" in t]
    return max(cands, key=len) if cands else "<unknown>"


def tier(mb: float) -> str:
    if mb >= 800:
        return "XL"
    if mb >= 300:
        return "LG"
    if mb >= 100:
        return "MD"
    return "LT"


def load_registered() -> dict:
    """Parse existing weights md: rows like `| name | 811 | LG |`."""
    out = {}
    if not WEIGHTS_MD.exists():
        return out
    for line in WEIGHTS_MD.read_text(encoding="utf-8").splitlines():
        m = re.match(r"\|\s*`?([\w:]+)`?\s*\|\s*(\d+)\s*\|", line)
        if m:
            out[m.group(1)] = int(m.group(2))
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("filter", help="test name substring / nextest filter")
    ap.add_argument("-F", "--features", default="test-vm-files")
    ap.add_argument("--config", default=None,
                    help="nextest --config-file override (e.g. .config/nextest-full.toml "
                         "to measure tests excluded by the daily default-filter)")
    ap.add_argument("--ignored", action="store_true",
                    help="measure #[ignore] tests too (--run-ignored=ignored-only)")
    ap.add_argument("--json-out", default=None, help="also dump raw samples as JSON")
    args = ap.parse_args()

    watcher = subprocess.Popen(
        ["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", WATCHER_PS],
        stdout=subprocess.PIPE, text=True, encoding="utf-8", errors="replace",
    )
    samples: dict = {}
    stop = threading.Event()
    th = threading.Thread(target=watcher_thread, args=(samples, stop, watcher, str(REPO)), daemon=True)
    th.start()

    cmd = ["cargo", "nextest", "run", "-p", "auto-lang", "--lib",
           "--features", args.features, "--jobs", "1", "--no-fail-fast",
           "--final-status-level", "none"]
    if args.config:
        cmd += ["--config-file", args.config]
    if args.ignored:
        cmd += ["--run-ignored", "ignored-only"]
    cmd.append(args.filter)
    t0 = time.time()
    run = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True,
                         encoding="utf-8", errors="replace")
    elapsed = time.time() - t0
    time.sleep(0.5)  # let the watcher catch the last processes
    watcher.kill()
    th.join(timeout=2)

    if run.returncode != 0:
        sys.stderr.write(run.stdout[-3000:] + "\n" + run.stderr[-3000:] + "\n")
        print(f"NOTE: nextest exit={run.returncode} (failures do not invalidate "
              f"memory numbers below)", file=sys.stderr)

    rows = []
    for pid, rec in samples.items():
        name = test_name_from_cmd(rec["cmd"])
        if name == "<unknown>":
            continue
        rows.append((name, rec["peak"] / (1024 * 1024)))
    # same test may appear via several PIDs (retry/restart) — keep the max
    best: dict = {}
    for name, mb in rows:
        best[name] = max(best.get(name, 0.0), mb)

    registered = load_registered()
    print(f"\n# peak memory report  filter={args.filter!r} features={args.features} "
          f"ignored={args.ignored} wall={elapsed:.0f}s samples={len(samples)}\n")
    print("| test | peak_MB | tier | drift_vs_registered |")
    print("|---|---|---|---|")
    for name, mb in sorted(best.items(), key=lambda kv: -kv[1]):
        drift = ""
        if name in registered:
            d = (mb - registered[name]) / max(registered[name], 1) * 100
            flag = " **DRIFT**" if abs(d) > 50 else ""
            drift = f"{d:+.0f}%{flag}"
        print(f"| `{name}` | {mb:.0f} | {tier(mb)} | {drift} |")
    print(f"\ntotal tests sampled: {len(best)}")
    if args.json_out:
        import json
        Path(args.json_out).write_text(
            json.dumps({k: round(v, 1) for k, v in best.items()}, indent=1),
            encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
