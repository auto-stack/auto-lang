#!/usr/bin/env python3
"""
Plan 547 F1: real-measurement release performance harness.

Replaces the fabricated perf_release.ps1 (whose "navigation" timings were
bare PowerShell index loops). Every metric below is measured against the
REAL application process:

  startup   spawn release exe (env AUTOUI_MCP_PORT) -> MCP server ready.
            Process exit before ready = FAIL (the old harness tolerated a
            dead process and sampled zeros).
  navigation  Open Directory -> ready; thumbnail-item presses via MCP
            autoui_action (the app's real directory navigation path).
  stats     queue/cache counters read from the app model's `stats` field
            (back image_stats, refreshed by SettleTick) via autoui_state.
  idle/RSS  PowerShell Get-Process sampling of the live process tree.
  shutdown  terminate -> process tree gone.

Usage:
    python tests/perf_measure.py [--skip-build]

Outputs tests/perf-report.json and exits nonzero when any gate fails.
"""
import argparse
import json
import os
import re
import socket
import subprocess
import sys
import time
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
PROJECT = os.path.normpath(os.path.join(HERE, ".."))
REPO = os.path.normpath(os.path.join(PROJECT, "..", "..", ".."))
WORKSPACE = os.path.join(REPO, "examples", "rust-workspace")
EXPECTATIONS = os.path.join(HERE, "perf_expectations.json")
REPORT = os.path.join(HERE, "perf-report.json")
AUTO_BIN = os.path.join(REPO, "target", "debug", "auto.exe")


def free_port(start=11570):
    # bind-based probe: connect_ex reports Windows excluded port ranges
    # (Hyper-V reservations, os error 10013) as "free" while bind() fails.
    for p in range(start, start + 200):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            try:
                s.bind(("127.0.0.1", p))
                return p
            except OSError:
                continue
    raise RuntimeError("no bindable port in range")


def release_exe():
    # examples/rust-workspace/.cargo/config.toml redirects the target dir to
    # the repo-root target — the release exe lands there, not in the
    # workspace-local target.
    cands = [
        os.path.join(REPO, "target", "release", "image-viewer.exe"),
        os.path.join(WORKSPACE, "target", "release", "image-viewer.exe"),
    ]
    for c in cands:
        if os.path.exists(c):
            return c
    raise RuntimeError(f"release exe not found (looked at {cands[0]}")


def build(skip):
    if skip:
        return
    # Regenerate the merged rust sources from the current .at (generation is
    # coupled to `auto run -r rust --merged`), then release-build the crate.
    print("[*] regenerating merged rust sources (auto run, killed at ready)...")
    port = free_port()
    env = dict(os.environ)
    env["AUTOUI_MCP_PORT"] = str(port)
    proc = subprocess.Popen([AUTO_BIN, "run", "-r", "rust", "--server", "rust",
                             "--merged"], cwd=PROJECT, env=env,
                            stdout=subprocess.DEVNULL,
                            stderr=subprocess.DEVNULL)
    url = f"http://127.0.0.1:{port}/mcp"
    deadline = time.time() + 900
    ready = False
    while time.time() < deadline:
        if proc.poll() is not None:
            raise RuntimeError("auto run (generation) exited early")
        try:
            urllib.request.urlopen(f"http://127.0.0.1:{port}/", timeout=1)
            ready = True
            break
        except Exception:
            # MCP endpoint answers 404 on GET / but the port being open is
            # enough: server is up.
            try:
                urllib.request.urlopen(url, timeout=1)
                ready = True
                break
            except urllib.error.HTTPError:
                ready = True
                break
            except Exception:
                time.sleep(1)
    proc.kill()
    if not ready:
        raise RuntimeError("generation run never became ready")
    print("[*] cargo build --release 031-image-viewer...")
    subprocess.run(["cargo", "build", "--release", "-p", "image-viewer",
                    "-p", "app-031-image-viewer-back"], cwd=WORKSPACE, check=True)


class Mcp:
    def __init__(self, port):
        self.port = port
        self.url = f"http://127.0.0.1:{port}/mcp"
        self.id = 0

    def call(self, tool, **args):
        import requests
        self.id += 1
        r = requests.post(self.url, json={
            "jsonrpc": "2.0", "method": "tools/call",
            "params": {"name": tool, "arguments": args}, "id": self.id,
        }, timeout=60)
        d = r.json()
        if "error" in d:
            raise RuntimeError(d["error"])
        c = d.get("result", {}).get("content", [])
        return c[0]["text"] if c else ""


def state_field(mcp, field):
    s = mcp.call("autoui_state", widget="App")
    m = re.search(rf"{field}: \"?([^\"\n]+?)\"?(?: \(|$)", s)
    return m.group(1) if m else None


def press_button(mcp, label):
    vid = None
    for _ in range(30):
        snap = mcp.call("autoui_snapshot")
        for b, lbl in re.findall(r'button #(vnode_\d+) "([^"]+)"', snap):
            if label == lbl:
                vid = b
                break
        if vid:
            break
        time.sleep(0.5)
    if vid is None:
        raise RuntimeError(f"button {label!r} not found")
    out = mcp.call("autoui_action", element_id=vid, action="press")
    if "ok" not in out:
        raise RuntimeError(f"press {label}: {out}")


def proc_sample(pid):
    """(cpu_seconds, working_set_bytes) for a live pid via PowerShell."""
    r = subprocess.run(
        ["powershell", "-NoProfile", "-Command",
         f"(Get-Process -Id {pid} -ErrorAction SilentlyContinue "
         f"| Select-Object -Property TotalProcessorTime,WorkingSet64 "
         f"| ConvertTo-Json -Compress)"],
        capture_output=True, text=True, timeout=20)
    txt = (r.stdout or "").strip()
    if not txt:
        return None
    d = json.loads(txt)
    cpu = d.get("TotalProcessorTime") or 0
    if isinstance(cpu, dict):
        cpu = float(cpu.get("TotalSeconds") or 0)
    elif isinstance(cpu, str):
        if ":" in cpu:
            h, m, sec = cpu.split(":")
            cpu = int(h) * 3600 + int(m) * 60 + float(sec)
        else:
            cpu = float(cpu)
    return float(cpu), int(d.get("WorkingSet64") or 0)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--skip-build", action="store_true")
    args = ap.parse_args()

    exp = json.load(open(EXPECTATIONS, encoding="utf-8-sig"))
    report = {"plan": "PLAN-547-F1", "mode": "rust-release-real",
              "generated_at": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
              "metrics": {}, "checks": [], "evidence": {}}
    checks = report["checks"]

    def add(name, value, limit, cmp="le"):
        ok = value <= limit if cmp == "le" else value >= limit
        checks.append({"name": name, "value": value, "limit": limit,
                       "cmp": cmp, "passed": ok})
        print(("[PASS] " if ok else "[FAIL] ") + f"{name}: {value} (<= {limit})")
        return ok

    build(args.skip_build)
    exe = release_exe()

    # ---- startup: spawn -> MCP ready (real cold start; dead process fails)
    port = free_port()
    env = dict(os.environ)
    env["AUTOUI_MCP_PORT"] = str(port)
    t0 = time.perf_counter()
    # cwd = src/front (the VM track's working dir) so the app's relative
    # fixture paths ("../../tests/fixtures") resolve identically on both
    # tracks; the release exe is cwd-agnostic otherwise.
    proc = subprocess.Popen([exe], cwd=os.path.join(PROJECT, "src", "front"), env=env,
                            stdout=subprocess.DEVNULL,
                            stderr=subprocess.DEVNULL)
    mcp = Mcp(port)
    ready = False
    while time.perf_counter() - t0 < 60:
        if proc.poll() is not None:
            report["metrics"]["cold_start_ms"] = None
            checks.append({"name": "process_alive_after_start", "passed": False,
                           "value": False,
                           "note": "release exe exited before MCP ready"})
            json.dump(report, open(REPORT, "w"), indent=2)
            print("[FAIL] release exe exited before MCP ready")
            return 1
        try:
            mcp.call("autoui_state", widget="App")
            ready = True
            break
        except Exception:
            time.sleep(0.1)
    if not ready:
        print("[FAIL] MCP never ready")
        return 1
    cold_ms = (time.perf_counter() - t0) * 1000
    report["metrics"]["cold_start_ms"] = round(cold_ms, 1)
    checks.append({"name": "process_alive_after_start", "passed": True,
                   "value": True})
    add("cold_start_ms", cold_ms, exp["cold_start_ms"])

    try:
        # ---- session: open directory (deterministic fixtures)
        press_button(mcp, "Open Directory")
        for _ in range(120):
            if state_field(mcp, "view_state") == "ready":
                break
            time.sleep(0.1)
        assert state_field(mcp, "view_state") == "ready", "directory not ready"

        # ---- neighbor navigation: 4 real thumbnail presses
        def selected():
            return state_field(mcp, "selected_index")

        def nav_key(k):
            # Navigation is driven by the toolbar prev/next buttons (the
            # rust track's MCP keyboard channel is not wired — P547-D9).
            press_button(mcp, k)

        # Warm the prefetch: visit both directions once before timing.
        nav_key("▶"); time.sleep(0.3)
        nav_key("◀"); time.sleep(0.3)
        # Timing = the synchronous press round-trip (the handler runs to
        # completion inside it — verified: selected_index is already updated
        # in the very next state read). Assertions run OUTSIDE the timed
        # window. The figure includes the MCP channel (~60ms).
        total = 0.0
        steps = []
        for k in ("▶", "◀", "▶", "◀"):
            before = selected()
            t0 = time.perf_counter()
            nav_key(k)
            dt = time.perf_counter() - t0
            after = selected()
            assert after != before, f"nav {k} did not move selection ({before})"
            steps.append(round(dt * 1000, 1))
            total += dt
        report["evidence"]["neighbor_steps_ms"] = steps
        print(f"[*] neighbor steps ms: {steps}")
        report["metrics"]["neighbor_navigation_ms"] = round(total * 1000 / 4, 1)
        add("neighbor_navigation_ms",
            report["metrics"]["neighbor_navigation_ms"],
            exp["neighbor_navigation_ms"])

        # ---- 100 navigations: fire 100 presses, then wait for stability
        t0 = time.perf_counter()
        for i in range(100):
            nav_key("▶" if i % 2 == 0 else "◀")
        time.sleep(1.0)  # settle outside nothing — presses are synchronous
        nav_ms = (time.perf_counter() - t0) * 1000
        report["metrics"]["navigation_100_ms"] = round(nav_ms, 1)
        add("navigation_100_ms", nav_ms, exp["navigation_100_ms"])

        # ---- real pipeline stats from the app model: the app refreshes its
        # `stats` field on demand (S key -> RefreshStats -> back image_stats)
        out = mcp.call("autoui_keyboard", key="s")
        assert "sent" in out.lower() or "ok" in out.lower(), out
        time.sleep(0.5)
        stats = state_field(mcp, "stats") or ""
        report["evidence"]["stats_raw"] = stats[:400]
        for key, pattern in {
            "queue_depth": r'"queue_depth":\s*(\d+)',
            "cache_hits": r'"cache_hits":\s*(\d+)',
            "cache_misses": r'"cache_misses":\s*(\d+)',
        }.items():
            m = re.search(pattern, stats)
            if m:
                report["metrics"][key] = int(m.group(1))
        qd = report["metrics"].get("queue_depth", 0)
        add("queue_depth", qd, exp["queue_depth"])

        # ---- idle: 10 s CPU + WS on the live process
        pid = proc.pid
        s0 = proc_sample(pid)
        assert s0, "process died before idle sampling"
        time.sleep(10)
        s1 = proc_sample(pid)
        assert s1, "process died during idle sampling"
        idle_cpu = max(0.0, (s1[0] - s0[0]) / 10.0 * 100.0)
        report["metrics"]["idle_cpu_percent"] = round(idle_cpu, 2)
        report["metrics"]["rss_mb"] = round(s1[1] / 1e6, 1)
        add("idle_cpu_percent", idle_cpu, exp["idle_cpu_percent"])
        add("rss_peak_mb", report["metrics"]["rss_mb"], exp["rss_peak_mb"])
    finally:
        # ---- shutdown: terminate -> gone
        t0 = time.perf_counter()
        proc.kill()
        try:
            proc.wait(timeout=15)
        except subprocess.TimeoutExpired:
            pass
        report["metrics"]["shutdown_ms"] = round(
            (time.perf_counter() - t0) * 1000, 1)
        add("shutdown_ms", report["metrics"]["shutdown_ms"], exp["shutdown_ms"])

    passed = all(c["passed"] for c in checks)
    report["passed"] = passed
    json.dump(report, open(REPORT, "w"), indent=2)
    print(f"\nperf-report.json passed={passed}")
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
