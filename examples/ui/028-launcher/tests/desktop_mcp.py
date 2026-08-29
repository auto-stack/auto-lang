#!/usr/bin/env python3
"""
Plan 464 T3/T4: MCP interaction tests for the 028-launcher app in VM mode.

Starts `auto run -r vm` in the 028-launcher project directory, waits for the
UI MCP server, then exercises the launcher palette/grid keyboard flow via
autoui_* HTTP tools:

  T1  landing snapshot (hidden state + open button)
  T2  open palette (click) + __focus_input consumed by the renderer tail
  T3  type "to" → filter (nres=4, sel=0)
  T4  ArrowDown ×2 → sel=2 → Enter → launches 012-stopwatch (plan
      acceptance flow), palette hides, recent recorded
  T5  reopen → recent top group first
  T6  Tab → grid mode, ArrowDown → gsel=4, Enter → 015-notes
  T7  Esc layering: clears query first, closes second
  T8  Ctrl+Space opens the palette (standalone self-managed summon)

Usage:
    cd examples/ui/028-launcher/tests
    python desktop_mcp.py

Prerequisites:
    - auto built with ui-iced (worktree target/debug/auto.exe or AUTO_BIN)
    - Python requests
"""

import os
import re
import subprocess
import sys
import time

try:
    import requests
except ImportError:
    print("Please install requests: pip install requests")
    sys.exit(1)

MCP_PORT_DEFAULT = 9428

AUTO_BIN = os.environ.get(
    "AUTO_BIN",
    os.path.join(os.path.dirname(__file__), "..", "..", "..", "..",
                 "target", "debug", "auto.exe"),
)
PROJECT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def pick_free_port(start=MCP_PORT_DEFAULT):
    """First port in [start, start+100) nothing binds (013 hardening)."""
    import socket
    for port in range(start, start + 100):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            if s.connect_ex(("127.0.0.1", port)) != 0:
                return port
    raise RuntimeError(f"No free port in [{start}, {start + 100})")


class McpClient:
    """JSON-RPC client for the UI MCP server (013 shape)."""

    def __init__(self, url):
        self.url = url

    def call(self, tool_name, **arguments):
        payload = {
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": {"name": tool_name, "arguments": arguments},
        }
        r = requests.post(self.url, json=payload, timeout=10)
        r.raise_for_status()
        data = r.json()
        if "error" in data:
            raise RuntimeError(f"MCP error: {data['error']}")
        return data["result"]["content"][0]["text"]

    def snapshot(self):
        return self.call("autoui_snapshot")

    def state(self, *fields):
        return self.call("autoui_state", fields=list(fields))

    def click(self, element_id):
        return self.call("autoui_action", element_id=element_id, action="press")

    def type_text(self, element_id, text):
        return self.call("autoui_type", element_id=element_id, text=text)

    def key(self, key, modifiers=None):
        args = {"key": key}
        if modifiers:
            args["modifiers"] = modifiers
        return self.call("autoui_keyboard", **args)

    def screenshot(self, name):
        return self.call("autoui_screenshot", name=name)

    def find(self, kind=None, label=None):
        """vnode id of the first node matching kind/label via autoui_find.

        The find result is an ancestor-chain subtree (root -> ... -> match),
        so the first vnode id in the text is an ANCESTOR, not the match —
        grab the kind-prefixed occurrence, falling back to the last id."""
        args = {"limit": 1}
        if kind:
            args["kind"] = kind
        if label:
            args["label"] = label
        out = self.call("autoui_find", **args)
        if kind:
            m = re.search(rf"{re.escape(kind)}\s+vnode_(\d+)", out)
            if m:
                return f"vnode_{m.group(1)}"
        ids = re.findall(r"vnode_(\d+)", out)
        return f"vnode_{ids[-1]}" if ids else None


def wait_for_server(url, timeout=30):
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            requests.post(url, json={
                "jsonrpc": "2.0", "id": 0, "method": "tools/list", "params": {},
            }, timeout=2)
            return True
        except requests.RequestException:
            time.sleep(0.5)
    return False


def find_element_by_event(snapshot_text, event_name, attr="onclick"):
    """First aura/vnode id bound to `.{event_name}` via attr (013 shape)."""
    pattern_id = re.compile(r"#(aura_\d+|vnode_\d+)")
    current_id = None
    target = f"{attr}: .{event_name}"
    for line in snapshot_text.splitlines():
        m = pattern_id.search(line)
        if m:
            current_id = m.group(1)
        if target in line and current_id is not None:
            return current_id
    return None


def state_field(state_text, field):
    """Extract `field: value` from autoui_state output (type suffix stripped)."""
    m = re.search(rf"{re.escape(field)}:\s*(.+)", state_text)
    if not m:
        return None
    val = m.group(1).strip()
    # strip renderer type annotations like ` (str)` / ` (int)` / ` (list)`
    val = re.sub(r"\s+\((str|int|float|bool|list)\)\s*$", "", val)
    return val


class Result:
    def __init__(self):
        self.passed = 0
        self.failed = 0

    def check(self, name, ok, detail=""):
        if ok:
            self.passed += 1
            print(f"  PASS  {name}")
        else:
            self.failed += 1
            print(f"  FAIL  {name}: {detail[:300]}")


def main():
    print("=" * 60)
    print("Plan 464 T3/T4: desktop MCP tests (028-launcher, vm mode)")
    print("=" * 60)
    if not os.path.exists(AUTO_BIN):
        print(f"ERROR: auto binary not found at {AUTO_BIN}")
        sys.exit(2)

    mcp_port = pick_free_port()
    mcp_url = f"http://localhost:{mcp_port}/mcp"
    print(f"\nStarting 028-launcher (vm) in {PROJECT}, MCP port {mcp_port}")
    proc = subprocess.Popen(
        [AUTO_BIN, "run", "-r", "vm"],
        cwd=PROJECT,
        env={**os.environ, "AUTOUI_MCP_PORT": str(mcp_port)},
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    try:
        if not wait_for_server(mcp_url):
            print("ERROR: MCP server did not start within 30s")
            sys.exit(1)
        time.sleep(1.0)  # 首帧渲染
        mcp = McpClient(mcp_url)
        result = Result()

        # ---- T1: landing (palette hidden, open button present) ----
        print("\nT1: landing snapshot")
        snap = mcp.snapshot()
        result.check("App widget present", 'widget: "App"' in snap, snap[:200])
        result.check("open button present", "Open launcher" in snap, "button missing")
        result.check("visible starts 0", state_field(mcp.state("visible"), "visible") == '"0"',
                     mcp.state("visible"))

        # ---- T2: open palette + focus request consumed ----
        print("\nT2: open palette (click)")
        open_btn = mcp.find(kind="button", label="Open launcher")
        result.check("Open button found", open_btn is not None, "no Open launcher button")
        if open_btn:
            mcp.click(open_btn)
            time.sleep(0.5)
            st = mcp.state("visible", "mode", "nres", "sel", "__focus_input")
            result.check("visible flipped 1", state_field(st, "visible") == '"1"', st)
            result.check("mode palette", state_field(st, "mode") == '"palette"', st)
            result.check("12 mock apps listed", state_field(st, "nres") == "12", st)
            result.check("__focus_input consumed by renderer tail",
                         state_field(st, "__focus_input") == '""', st)
            mcp.screenshot("t3_open_palette")

        # ---- T3: type "to" → filter ----
        print("\nT3: type query")
        inp = mcp.find(kind="input")
        result.check("palette input found", inp is not None, "no input in palette")
        if inp:
            mcp.type_text(inp, "to")
            time.sleep(0.5)
            st = mcp.state("q", "nres", "sel")
            result.check("q is to", state_field(st, "q") == '"to"', st)
            result.check("filtered to 4", state_field(st, "nres") == "4", st)
            result.check("sel reset 0", state_field(st, "sel") == "0", st)

        # ---- T4: ↓×2 → Enter launches 012-stopwatch (plan acceptance) ----
        print("\nT4: keyboard flow → Enter")
        mcp.key("ArrowDown")
        mcp.key("ArrowDown")
        time.sleep(0.3)
        st = mcp.state("sel")
        result.check("sel moved to 2", state_field(st, "sel") == "2", st)
        mcp.key("Enter")
        time.sleep(0.5)
        st = mcp.state("last", "visible", "recent")
        result.check("Enter launched 012-stopwatch",
                     state_field(st, "last") == '"012-stopwatch"', st)
        result.check("palette hidden after launch", state_field(st, "visible") == '"0"', st)
        result.check("recent recorded", "012-stopwatch" in (state_field(st, "recent") or ""), st)
        mcp.screenshot("t3_after_launch")

        # ---- T5: reopen → recent top group ----
        print("\nT5: recent grouping")
        snap = mcp.snapshot()
        open_btn = mcp.find(kind="button", label="Open launcher")
        if open_btn:
            mcp.click(open_btn)
            time.sleep(0.5)
            # ranked 元素经 VM 读回是 <vmref> 句柄 —— recent 分组断言走快照
            # （第一行 sub 列渲染 "recent" 文本）。
            snap = mcp.snapshot()
            recent_idx = snap.find('"recent"')
            todo_idx = snap.find('"012-stopwatch"')
            row_idx = snap.find("Stopwatch")
            result.check("recent row rendered", row_idx != -1 and recent_idx != -1,
                         f"recent={recent_idx} stopwatch={row_idx}")
            if recent_idx != -1 and todo_idx != -1:
                result.check("recent group precedes registry rows",
                             recent_idx < snap.find('"tool"'), snap[:120])

        # ---- T6: Tab → grid, arrows, Enter ----
        print("\nT6: grid form")
        mcp.key("Tab")
        time.sleep(0.3)
        st = mcp.state("mode", "gsel")
        result.check("Tab switches to grid", state_field(st, "mode") == '"grid"', st)
        mcp.key("ArrowDown")
        time.sleep(0.3)
        st = mcp.state("gsel")
        result.check("grid ArrowDown gsel 0→4", state_field(st, "gsel") == "4", st)
        mcp.key("Enter")
        time.sleep(0.5)
        st = mcp.state("last")
        result.check("grid Enter launched 015-notes",
                     state_field(st, "last") == '"015-notes"', st)
        mcp.screenshot("t3_grid")

        # ---- T7: Esc layering ----
        print("\nT7: esc layering")
        snap = mcp.snapshot()
        open_btn = mcp.find(kind="button", label="Open launcher")
        if open_btn:
            mcp.click(open_btn)
            time.sleep(0.3)
            inp = mcp.find(kind="input")
            mcp.type_text(inp, "calc")
            time.sleep(0.3)
            mcp.key("Escape")
            time.sleep(0.3)
            st = mcp.state("q", "nres", "visible")
            result.check("first Esc clears query",
                         state_field(st, "q") == '""' and state_field(st, "nres") == "12", st)
            result.check("palette still open", state_field(st, "visible") == '"1"', st)
            mcp.key("Escape")
            time.sleep(0.3)
            st = mcp.state("visible")
            result.check("second Esc closes", state_field(st, "visible") == '"0"', st)

        # ---- T8: Ctrl+Space summon (standalone) ----
        print("\nT8: ctrl+space summon")
        mcp.key(" ", modifiers=["ctrl"])
        time.sleep(0.3)
        st = mcp.state("visible")
        result.check("ctrl+space opens palette", state_field(st, "visible") == '"1"', st)

        print("\n" + "=" * 60)
        print(f"RESULTS: {result.passed} passed, {result.failed} failed")
        print("=" * 60)
        sys.exit(1 if result.failed else 0)
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    main()
