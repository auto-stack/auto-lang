#!/usr/bin/env python3
"""
Plan 547 F3: VM-side interaction-matrix evidence (rotation + pan).

Starts the 031-image-viewer app in VM mode, drives the core matrix through
the real keyboard/action MCP channel, asserts state after each step, and
captures screenshots:

  OpenFile -> (baseline) -> ArrowRight (rotation +90) -> d (pan +40px)

Run from the repo root or the tests dir:
    python vm_matrix.py

Prerequisites: auto built with ui-iced (AUTO_BIN env override supported);
python requests. 553/563 harness lessons: stdout DEVNULL, fresh snapshot
per interaction group.
"""
import os
import re
import socket
import subprocess
import sys
import time

try:
    import requests
except ImportError:
    print("pip install requests")
    sys.exit(1)

_AUTO_BIN = os.path.join(os.path.dirname(__file__), "..", "..", "..", "..",
                         "target", "debug", "auto.exe")
AUTO_BIN = os.environ.get("AUTO_BIN", _AUTO_BIN)
PROJECT = os.path.normpath(os.path.join(os.path.dirname(__file__), ".."))
SHOTS = os.path.join(os.path.dirname(__file__), "screenshots")


def free_port(start=9570):
    for p in range(start, start + 50):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            if s.connect_ex(("127.0.0.1", p)) != 0:
                return p
    raise RuntimeError("no free port")


class Mcp:
    def __init__(self, url):
        self.url = url
        self.id = 0

    def call(self, tool, **args):
        self.id += 1
        r = requests.post(self.url, json={
            "jsonrpc": "2.0", "method": "tools/call",
            "params": {"name": tool, "arguments": args}, "id": self.id,
        }, timeout=30)
        d = r.json()
        if "error" in d:
            raise RuntimeError(d["error"])
        c = d.get("result", {}).get("content", [])
        return c[0]["text"] if c else ""


def press_button(mcp, label):
    snap = mcp.call("autoui_snapshot")
    vid = None
    for b, lbl in re.findall(r'button #(vnode_\d+) "([^"]+)"', snap):
        if label in lbl:
            vid = b
            break
    assert vid, f"button {label!r} not found"
    out = mcp.call("autoui_action", element_id=vid, action="press")
    assert "ok" in out, f"press {label}: {out}"


def state_field(mcp, field):
    s = mcp.call("autoui_state", widget="App")
    m = re.search(rf"{field}: \"?([^\"\n]+?)\"?(?: \(|$)", s)
    return m.group(1) if m else None


def main():
    port = free_port()
    env = dict(os.environ)
    env["AUTOUI_MCP_PORT"] = str(port)
    proc = subprocess.Popen([AUTO_BIN, "run", "-r", "vm"], cwd=PROJECT,
                            stdout=subprocess.DEVNULL,
                            stderr=subprocess.DEVNULL, env=env)
    mcp = Mcp(f"http://127.0.0.1:{port}/mcp")
    try:
        for _ in range(60):
            try:
                requests.post(mcp.url, json={"jsonrpc": "2.0",
                                             "method": "tools/list",
                                             "params": {}, "id": 1}, timeout=1)
                break
            except Exception:
                time.sleep(0.5)
        # wait for initial state
        for _ in range(40):
            if state_field(mcp, "view_state"):
                break
            time.sleep(0.5)

        os.makedirs(SHOTS, exist_ok=True)
        checks = []

        # 1. baseline: open the deterministic fixture
        press_button(mcp, "Open File")
        for _ in range(40):
            if state_field(mcp, "view_state") == "ready":
                break
            time.sleep(0.25)
        rot0 = state_field(mcp, "rotation")
        off0 = state_field(mcp, "offset_x")
        checks.append(("open ready", state_field(mcp, "view_state") == "ready"))
        mcp.call("autoui_screenshot", name="vm-matrix-open",
                 baseline=True)

        # 2. rotation: ArrowRight => +90
        out = mcp.call("autoui_keyboard", key="ArrowRight")
        assert "sent" in out.lower() or "ok" in out.lower(), out
        time.sleep(0.5)
        rot1 = state_field(mcp, "rotation")
        checks.append((f"rotation {rot0}->{rot1}", rot1 in ("90", "90.0")))
        mcp.call("autoui_screenshot", name="vm-matrix-rotation",
                 baseline=True)

        # 3. pan: 'd' => offset_x + 40
        out = mcp.call("autoui_keyboard", key="d")
        assert "sent" in out.lower() or "ok" in out.lower(), out
        time.sleep(0.5)
        off1 = state_field(mcp, "offset_x")
        delta_ok = None
        try:
            delta_ok = abs(float(off1) - float(off0 or 0) - 40.0) < 1e-6
        except (TypeError, ValueError):
            delta_ok = False
        checks.append((f"pan offset {off0}->{off1}", delta_ok))
        checks.append(("pan free fit", state_field(mcp, "fit_mode") == "free"))
        mcp.call("autoui_screenshot", name="vm-matrix-pan",
                 baseline=True)

        failed = [name for name, ok in checks if not ok]
        for name, ok in checks:
            print(("PASS " if ok else "FAIL ") + name)
        if failed:
            print(f"\n{len(failed)} FAILED")
            return 1
        print("\nALL PASS (open/rotation/pan state + 3 screenshots)")
        return 0
    finally:
        proc.kill()


if __name__ == "__main__":
    sys.exit(main())
