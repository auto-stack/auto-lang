#!/usr/bin/env python3
"""
Plan 563 T8: MCP interaction tests for the 043-canvas-paint app in VM mode.

Starts `auto run -r vm` in the project directory, waits for the UI MCP
server, then exercises the canvas paint UI: initial empty state, pen stroke
via autoui_action 'pen' (synthesized onpenstart/onpenmove/onpenend with
logical coords), color/width selection, eraser meta bit, undo, clear, and
storage save/load round-trip.

553 harness lessons applied: subprocess stdout DEVNULL (pipe backpressure
freezes the UI); vnode ids change on re-render — re-snapshot before each
interaction group.

Usage:
    cd examples/capability-tests/043-canvas-paint/tests
    python desktop_mcp.py

Prerequisites:
    - auto built with ui-iced: cargo build --features ui-iced --bin auto
      (or set AUTO_BIN env var to the binary path)
    - Python requests: pip install requests
"""

import os
import socket
import subprocess
import sys
import time

try:
    import requests
except ImportError:
    print("Please install requests: pip install requests")
    sys.exit(1)

MCP_PORT_DEFAULT = 9360

# Default auto binary: <repo>/target/debug/auto.exe (worktree-owned target).
_AUTO_BIN = os.path.join(os.path.dirname(__file__), "..", "..", "..", "..",
                         "target", "debug", "auto.exe")
AUTO_BIN = os.environ.get("AUTO_BIN", _AUTO_BIN)
PROJECT = os.path.normpath(os.path.join(os.path.dirname(__file__), ".."))

SEP = "\x1f"  # payload separator (renderer PAYLOAD_SEP U+001F)


def pick_free_port(start=MCP_PORT_DEFAULT):
    for port in range(start, start + 100):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            if s.connect_ex(("127.0.0.1", port)) != 0:
                return port
    raise RuntimeError(f"No free port in [{start}, {start + 100})")


class McpClient:
    def __init__(self, url):
        self.url = url
        self.req_id = 0

    def call(self, tool_name, **arguments):
        self.req_id += 1
        resp = requests.post(self.url, json={
            "jsonrpc": "2.0", "method": "tools/call",
            "params": {"name": tool_name, "arguments": arguments},
            "id": self.req_id,
        }, timeout=20)
        data = resp.json()
        if "error" in data:
            raise RuntimeError(f"MCP error: {data['error']}")
        content = data.get("result", {}).get("content", [])
        return content[0]["text"] if content else ""


class Harness:
    def __init__(self):
        self.port = pick_free_port()
        env = dict(os.environ)
        env["AUTOUI_MCP_PORT"] = str(self.port)
        # 553 lesson: stdout DEVNULL — PIPE backpressure freezes rendering.
        self.proc = subprocess.Popen(
            [AUTO_BIN, "run", "-r", "vm"], cwd=PROJECT,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=env)
        self.url = f"http://127.0.0.1:{self.port}/mcp"
        for _ in range(60):
            try:
                requests.post(self.url, json={"jsonrpc": "2.0",
                                               "method": "tools/list",
                                               "params": {}, "id": 1},
                              timeout=1)
                return
            except Exception:
                time.sleep(0.5)
        raise RuntimeError("MCP server never came up")

    def stop(self):
        self.proc.kill()

    def state(self, widget="App"):
        return McpClient(self.url).call("autoui_state", widget=widget)

    def snapshot(self):
        return McpClient(self.url).call("autoui_snapshot")

    def wait_state(self, needle, timeout_s=30):
        for _ in range(timeout_s * 2):
            s = self.state()
            if needle in s:
                return s
            time.sleep(0.5)
        raise RuntimeError(f"state never contained {needle!r}")

    def pen(self, points, start="PenStart", move="PenMove", end="PenEnd"):
        spec = SEP.join(["App", start, move, end,
                         ";".join(f"{x},{y}" for x, y in points)])
        out = McpClient(self.url).call(
            "autoui_action", element_id="aura_0", action="pen", value=spec)
        assert "status: ok" in out, f"pen action failed: {out}"

    def press_button(self, label):
        # 553 lesson: vnode ids change across re-renders — re-snapshot and
        # re-find the button in every interaction group (031 regex form).
        import re
        snap = self.snapshot()
        buttons = {}
        for vid, btn_label in re.findall(r'button #(vnode_\d+) "([^"]+)"', snap):
            buttons.setdefault(btn_label, vid)
        vid = None
        for btn_label, v in buttons.items():
            if label in btn_label:
                vid = v
                break
        if vid is None:
            raise RuntimeError(f"button {label!r} not found in snapshot")
        out = McpClient(self.url).call(
            "autoui_action", element_id=vid, action="press")
        assert "ok" in out, f"press {label} failed: {out}"


PASS = 0
FAIL = 0


def check(name, cond, detail=""):
    global PASS, FAIL
    if cond:
        PASS += 1
        print(f"  PASS  {name}")
    else:
        FAIL += 1
        print(f"  FAIL  {name}  {detail}")


def main():
    h = Harness()
    try:
        # ---- T1 结构/初始态 ----
        print("[T1] initial state")
        s = h.wait_state("strokes_pts")
        check("strokes_pts empty", "strokes_pts: []" in s, s[:200])
        check("count 0", 'stroke_count: "0"' in s, s[:200])
        check("pencil default", 'tool: "pencil"' in s, s[:300])

        # ---- T2 pen 笔画(start/move/end 坐标链)----
        print("[T2] pen stroke wiring")
        h.pen([(40, 50), (120, 80), (200, 140)])
        time.sleep(0.4)
        s = h.state()
        check("stroke count 1", 'stroke_count: "1"' in s, s[:300])
        check("3-point pts", "40.001,50.001|120.001,80.001|200.001,140.001" in s,
              s[:400])
        check("default meta", '"#111827,4,0"' in s, s[:400])

        # ---- T3 线宽选择 → meta ----
        print("[T3] width selection")
        # 颜色钮是空 label 色块(press-by-label 不可达);线宽/工具/操作
        # 纽均带文本。颜色链路由生成单测+对拍覆盖,此处断 width/eraser。
        h.press_button("8")
        time.sleep(0.3)
        h.pen([(60, 60), (100, 100)])
        time.sleep(0.4)
        s = h.state()
        check("width 8 meta", '"#111827,8,0"' in s, s[:400])
        check("stroke count 2", 'stroke_count: "2"' in s, s[:300])

        # ---- T4 eraser 位 ----
        print("[T4] eraser meta bit")
        h.press_button("Eraser")
        time.sleep(0.3)
        h.pen([(10, 10), (30, 30)])
        time.sleep(0.4)
        s = h.state()
        check("eraser meta bit", '"#111827,8,1"' in s, s[:400])
        check("stroke count 3", 'stroke_count: "3"' in s, s[:300])

        # ---- T5 undo ----
        print("[T5] undo pops last stroke")
        h.press_button("Undo")
        time.sleep(0.3)
        s = h.state()
        check("undo -> 2 strokes", 'stroke_count: "2"' in s, s[:300])

        # ---- T6 save/load 往返 ----
        print("[T6] storage save/load round-trip")
        h.press_button("Save")
        time.sleep(0.3)
        h.press_button("Clear")
        time.sleep(0.3)
        s = h.state()
        check("cleared", 'stroke_count: "0"' in s, s[:300])
        h.press_button("Load")
        time.sleep(0.4)
        s = h.state()
        check("load restores 2", 'stroke_count: "2"' in s, s[:300])
        check("load restores pts", "40.001,50.001" in s, s[:400])

        print(f"\n{PASS} PASS, {FAIL} FAIL")
        return 0 if FAIL == 0 else 1
    finally:
        h.stop()


if __name__ == "__main__":
    sys.exit(main())
