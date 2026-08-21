#!/usr/bin/env python3
"""
Plan 403 需求 1a: MCP interaction tests for the 011-calculator app (VM mode).

Starts `auto run -r vm` in the 011-calculator project directory, waits for the
UI MCP server (localhost:9247), then drives the REAL calculator through its
buttons via the autoui_* HTTP tools — chiefly `autoui_press_sequence` (the
tool Plan 403 需求 1b built for exactly this): keys are matched to rendered
buttons by label, so the math happens through real presses, not direct calls.

Covers: snapshot structure, initial state, integer eval (2+3=5), chained ops,
decimal eval (3.5+1=4.5 — the Phase 403-F VM float fix, e2e via MCP),
scientific-mode parens/precedence (2*(3+4)=14, mode switch by pressing the
"Scientific" tab), and Clear.

Usage:
    cd examples/ui/011-calculator/tests
    python desktop_mcp.py

Prerequisites:
    - auto built with ui-iced: cargo build --features ui-iced --bin auto
      (or set AUTO_BIN env var to the binary path)
    - Python requests: pip install requests
"""

import subprocess
import sys
import time
import os
import re

try:
    import requests
except ImportError:
    print("Please install requests: pip install requests")
    sys.exit(1)

MCP_PORT_DEFAULT = 9247


def pick_free_port(start=MCP_PORT_DEFAULT):
    """First port in [start, start+100) that nothing is bound to.

    Stale `auto.exe` VM processes from earlier sessions sometimes keep port
    9247 open with a half-dead UI (window closed, MCP thread alive, empty
    snapshot). Binding our own fresh port via AUTOUI_MCP_PORT makes the test
    hermetic against such zombies instead of silently querying them.
    """
    import socket
    for port in range(start, start + 100):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            if s.connect_ex(("127.0.0.1", port)) != 0:
                return port
    raise RuntimeError(f"No free port in [{start}, {start + 100})")
# Default auto binary: <repo>/target/debug/auto(.exe)
_AUTO_BIN = os.path.join(os.path.dirname(__file__), "..", "..", "..", "..",
                         "target", "debug", "auto.exe")
AUTO_BIN = os.environ.get("AUTO_BIN", _AUTO_BIN)
# Real 011-calculator project root (pac.at lives here).
CALC_PROJECT = os.path.normpath(
    os.path.join(os.path.dirname(__file__), ".."))


class McpClient:
    """JSON-RPC client for the UI MCP server."""

    def __init__(self, url):
        self.url = url
        self.req_id = 0

    def call(self, tool_name, **arguments):
        self.req_id += 1
        resp = requests.post(self.url, json={
            "jsonrpc": "2.0", "method": "tools/call",
            "params": {"name": tool_name, "arguments": arguments},
            "id": self.req_id,
        }, timeout=15)
        data = resp.json()
        if "error" in data:
            raise RuntimeError(f"MCP error: {data['error']}")
        content = data.get("result", {}).get("content", [])
        return content[0]["text"] if content else ""

    def snapshot(self):
        return self.call("autoui_snapshot")

    def state(self, *fields):
        return self.call("autoui_state", fields=list(fields))

    def press(self, keys):
        """Press buttons by label via autoui_press_sequence (Plan 403 1b)."""
        return self.call("autoui_press_sequence", keys=list(keys),
                         state_fields=["display", "mode", "expr", "error"])


def wait_for_server(url, timeout=30):
    for _ in range(timeout):
        try:
            requests.post(url, json={
                "jsonrpc": "2.0", "method": "tools/list", "params": {}, "id": 1
            }, timeout=2)
            return True
        except (requests.ConnectionError, requests.Timeout):
            time.sleep(1)
    return False


def field_of(state_text, name):
    """Extract `name: value` from autoui_state / press_sequence output.

    Strips surrounding quotes (VM state renders str fields as `"5"`)."""
    m = re.search(rf"{name}:\s*(\S+)", state_text)
    return m.group(1).strip('"').strip("'") if m else None


class TestResult:
    def __init__(self):
        self.passed = 0
        self.failed = 0
        self.skipped = 0
        self.errors = []

    def check(self, name, condition, detail=""):
        if condition:
            self.passed += 1
            print(f"  PASS  {name}")
        else:
            self.failed += 1
            self.errors.append(f"{name}: {detail}")
            print(f"  FAIL  {name}: {detail}")

    def skip(self, name, reason):
        self.skipped += 1
        print(f"  SKIP  {name}: {reason}")


# ── Real 011-calculator test suite ─────────────────────────────────────────

def run_tests_011(mcp_url):
    mcp = McpClient(mcp_url)
    result = TestResult()

    # T1: Snapshot shows the real App structure (grid keypad + mode tabs)
    print("\nT1: UI Snapshot of real 011-calculator")
    snap = mcp.snapshot()
    result.check("Snapshot contains App widget name", 'widget: "App"' in snap, snap[:200])
    result.check("Snapshot has element IDs", "aura_" in snap or "vnode_" in snap,
                 "No aura_/vnode_ IDs found (v2 snapshot)")
    result.check("Snapshot shows digits", '"1"' in snap and '"9"' in snap, "digit buttons missing")
    result.check("Snapshot shows operators", '"+"' in snap and '"="' in snap, "operator buttons missing")
    result.check("Snapshot shows mode tabs", '"Scientific"' in snap, "mode tabs missing")

    # T2: Initial state — display "0", mode "basic"
    print("\nT2: Initial State")
    state = mcp.state("display", "mode", "expr", "error")
    result.check("display starts at 0", field_of(state, "display") in ('"0"', "'0'", "0"), state[:200])
    result.check("mode is basic", "basic" in (field_of(state, "mode") or ""), state[:200])
    result.check("no error", (field_of(state, "error") or "") in ("", '""'), state[:200])

    # T3: Integer eval via real presses: 2 + 3 = 5   (需求 1b 的标志性用例)
    print("\nT3: Integer eval 2+3=5")
    out = mcp.press(["C", "2", "+", "3", "="])
    result.check("display is 5", field_of(out, "display") == "5", out[:300])

    # T4: Chained ops: 1 + 2 + 3 = 6
    print("\nT4: Chained eval 1+2+3=6")
    out = mcp.press(["C", "1", "+", "2", "+", "3", "="])
    result.check("display is 6", field_of(out, "display") == "6", out[:300])

    # T5: Decimal eval: 3.5 + 1 = 4.5   (Phase 403-F VM float fix, e2e via MCP)
    print("\nT5: Decimal eval 3.5+1=4.5 (403-F)")
    out = mcp.press(["C", "3", ".", "5", "+", "1", "="])
    result.check("display is 4.5", field_of(out, "display") == "4.5", out[:300])

    # T6: Scientific mode — parens + precedence: 2 * (3 + 4) = 14
    # Mode switch itself is a button press ("Scientific" tab label).
    print("\nT6: Scientific mode 2*(3+4)=14")
    out = mcp.press(["Scientific"])
    result.check("mode switched to scientific", "scientific" in (field_of(out, "mode") or ""), out[:300])
    out = mcp.press(["C", "2", "*", "(", "3", "+", "4", ")", "="])
    result.check("display is 14", field_of(out, "display") == "14", out[:300])
    mcp.press(["Basic"])  # restore for later suites

    # T7: Clear: C resets display to 0
    print("\nT7: Clear resets display")
    mcp.press(["7"])
    out = mcp.press(["C"])
    result.check("display back to 0", field_of(out, "display") in ("0", '"0"'), out[:300])

    return result


def main():
    print("=" * 60)
    print("Plan 403 需求 1a: Desktop MCP Tests (real 011-calculator, VM mode)")
    print("=" * 60)

    if not os.path.exists(AUTO_BIN):
        print(f"ERROR: auto binary not found at {AUTO_BIN}")
        print("Build it first: cargo build --features ui-iced --bin auto")
        print("or set AUTO_BIN env var to the binary path.")
        sys.exit(2)
    if not os.path.exists(os.path.join(CALC_PROJECT, "pac.at")):
        print(f"ERROR: 011-calculator project not found at {CALC_PROJECT}")
        sys.exit(2)

    mcp_port = pick_free_port()
    mcp_url = f"http://localhost:{mcp_port}/mcp"
    if mcp_port != MCP_PORT_DEFAULT:
        print(f"NOTE: port {MCP_PORT_DEFAULT} busy (stale auto.exe?); "
              f"using AUTOUI_MCP_PORT={mcp_port}")

    print(f"\nStarting real 011-calculator in {CALC_PROJECT}...")
    proc = subprocess.Popen(
        [AUTO_BIN, "run", "-r", "vm"],
        cwd=CALC_PROJECT,
        env={**os.environ, "AUTOUI_MCP_PORT": str(mcp_port)},
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

    try:
        print(f"Waiting for MCP server on port {mcp_port}...")
        if not wait_for_server(mcp_url):
            print(f"ERROR: MCP server did not start within 30s. "
                  f"Is the auto binary built with --features ui-iced? "
                  f"Binary: {AUTO_BIN}")
            proc.kill()
            sys.exit(1)
        print("MCP server ready")

        # Wait for the iced window to render its first frame.
        print("Waiting for UI to render...")
        client = McpClient(mcp_url)
        rendered = False
        for i in range(20):
            time.sleep(2)
            try:
                snap = client.snapshot()
                if "aura_" in snap or "Scientific" in snap:
                    print(f"UI rendered after {(i + 1) * 2}s")
                    rendered = True
                    break
            except Exception:
                pass
        if not rendered:
            print("WARNING: UI may not have rendered; running tests anyway...")

        result = run_tests_011(mcp_url)

        print(f"\n{'=' * 60}")
        print(f"Results: {result.passed} passed, {result.failed} failed, "
              f"{result.skipped} skipped")
        if result.errors:
            for err in result.errors:
                print(f"  FAIL  {err}")
        print(f"{'=' * 60}")

        sys.exit(0 if result.failed == 0 else 1)
    finally:
        proc.kill()
        proc.wait()
        print("VM process terminated.")


if __name__ == "__main__":
    main()
