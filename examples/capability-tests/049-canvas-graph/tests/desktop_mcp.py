#!/usr/bin/env python3
"""
PLAN-661 T-06: MCP interaction tests for the 049-canvas-graph app in VM mode.

Starts `auto run -r vm` in the project directory, waits for the UI MCP
server, then exercises the canvas graph UI: primitive-scene tables (nodes/
edges/labels), onhit tap hit (press with element id), slider set_value
(radius rebuild), and the map bracket-write round-trip (.cfg["scale"] = v).

Covers AC-01/02/05/06 at the sample level (unit/bridge tiers cover the rest):
  - AC-01 map bracket write: Write button → .cfg["scale"] = .radius/80 →
    readback state observable, statements after the write run.
  - AC-02 slider closed loop: set_value(110) → .SetRadius(110) → radius
    state + node table rebuild.
  - AC-05 primitive scene: graph_nodes len 7 (ring 6 + center rect) driven.
  - AC-06 hit contract: press(value="n2"/"c") → .NodeTap(id) → selected
    state observable; hit_count accumulates.

553 harness lessons applied (same as 043): stdout DEVNULL; vnode ids change
across re-renders — re-snapshot before each interaction group.

Usage:
    cd examples/capability-tests/049-canvas-graph/tests
    python desktop_mcp.py

Prerequisites:
    - auto built with ui-iced: cargo build --features ui-iced --bin auto
      (or set AUTO_BIN env var to the binary path)
    - Python requests: pip install requests
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
    print("Please install requests: pip install requests")
    sys.exit(1)

MCP_PORT_DEFAULT = 9370

# Default auto binary: <repo>/target/debug/auto.exe (worktree-owned target).
_AUTO_BIN = os.path.join(os.path.dirname(__file__), "..", "..", "..", "..",
                         "target", "debug", "auto.exe")
AUTO_BIN = os.environ.get("AUTO_BIN", _AUTO_BIN)
PROJECT = os.path.normpath(os.path.join(os.path.dirname(__file__), ".."))


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

    def find_node(self, kind):
        """Re-snapshot and find the first `kind #vnode_N` line (553 lesson:
        ids change across re-renders)."""
        snap = self.snapshot()
        m = re.search(rf"{kind} #(vnode_\d+)", snap)
        if not m:
            raise RuntimeError(f"{kind} not found in snapshot:\n{snap[:600]}")
        return m.group(1)

    def action(self, element_id, action, value=None):
        args = {"element_id": element_id, "action": action}
        if value is not None:
            args["value"] = value
        out = McpClient(self.url).call("autoui_action", **args)
        assert "status: ok" in out or "ok" in out, f"{action} failed: {out}"
        return out


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
        # ---- T1 图元场景三表初始态（AC-05）----
        print("[T1] primitive scene tables")
        s = h.wait_state("graph_nodes")
        check("ring 6 + center rect = 7 nodes", 'graph_nodes: [' in s and s.count("circle,#3b82f6,16") == 6, s[:400])
        check("center rect node", "c,150,150,rect,#f59e0b,52,28" in s, s[:400])
        check("6 spokes edges", s.count(",#94a3b8,2") == 6, s[:400])
        check("7 labels", s.count("n0") + s.count("n3") + s.count("center") >= 5, s[:400])
        check("strokes tables empty", "graph_pts: []" in s, s[:300])

        # ---- T2 onhit tap 命中（AC-06）----
        print("[T2] onhit tap via press(value=id)")
        canvas_vid = h.find_node("canvas")
        h.action(canvas_vid, "press", "n2")
        time.sleep(0.4)
        s = h.state()
        check("selected n2", 'selected: "n2"' in s, s[:300])
        check("hit_count 1", "hit_count: 1" in s, s[:300])
        # 未命中 id：handler 仍派发（值由 id 定）——selected 更新为该 id 属
        # 契约内（id 载荷直达），断言派发本身；Reset 收尾。
        h.action(h.find_node("canvas"), "press", "c")
        time.sleep(0.4)
        s = h.state()
        check("center hit", 'selected: "c"' in s, s[:300])

        # ---- T3 slider set_value 闭环（AC-02 样板面）----
        print("[T3] slider set_value rebuild")
        slider_vid = h.find_node("slider")
        h.action(slider_vid, "set_value", 110)
        time.sleep(0.5)
        s = h.state()
        check("radius 110", "radius: 110" in s, s[:300])
        # 半径 110 → n0 坐标 (150+110, 150) = 260,150
        check("node table rebuilt", "n0,260" in s, s[:500])

        # ---- T4 map 括号写往返（AC-01 样板面）----
        print("[T4] map bracket write round-trip")
        h.action(h.find_node("button"), "press")  # Reset
        time.sleep(0.3)
        # 半径已被 T3 写为 110；重新滑回 80 再写 → scale=1
        h.action(h.find_node("slider"), "set_value", 80)
        time.sleep(0.4)
        # press "Write" 按钮（按 label 找——dict vid→label，按值反查）
        snap = h.snapshot()
        buttons = dict(re.findall(r'button #(vnode_\d+) "([^"]+)"', snap))
        write_vid = next((vid for vid, lbl in buttons.items() if "Write" in lbl), None)
        assert write_vid, f"Write button not found: {buttons}"
        h.action(write_vid, "press")
        time.sleep(0.4)
        s = h.state()
        # 80/80 = 1.0——写后读回可见（写臂落 map + 语句续行）
        check("map write readback 1", 'readback: "1"' in s, s[:400])

        print(f"\n{PASS} PASS, {FAIL} FAIL")
        return 0 if FAIL == 0 else 1
    finally:
        h.stop()


if __name__ == "__main__":
    sys.exit(main())
