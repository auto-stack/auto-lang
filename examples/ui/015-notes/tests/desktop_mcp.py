#!/usr/bin/env python3
"""
Plan 370 Phase 2: MCP interaction tests for VM mode.

Starts a simple counter widget in VM mode, waits for the UI MCP server
(localhost:9247), then exercises it via autoui_* HTTP tools.

Note: 015-notes App uses store composable + back.api which are Vue-only.
VM mode testing uses a self-contained counter widget instead.
Full 015-notes desktop testing requires fixing D-GAP-4 (store composable
in VM mode) first.

Usage:
    cd examples/ui/015-notes/tests
    python desktop_mcp.py

Prerequisites:
    - auto built with ui-iced: cargo build --features ui-iced --bin auto
    - Python requests: pip install requests
"""

import subprocess
import sys
import time
import tempfile
import os

try:
    import requests
except ImportError:
    print("Please install requests: pip install requests")
    sys.exit(1)

MCP_PORT = 9247
MCP_URL = f"http://localhost:{MCP_PORT}/mcp"
AUTO_BIN = os.environ.get("AUTO_BIN",
    os.path.join(os.path.dirname(__file__), "..", "..", "..", "..",
                 "target", "debug", "auto.exe"))

COUNTER_AT = """\
widget Counter {
    msg Msg { Inc, Dec }
    model { var count int = 0 }
    view {
        col {
            text .count
            button "Inc" { onclick: .Inc }
            button "Dec" { onclick: .Dec }
        }
    }
    on {
        .Inc -> { .count = .count + 1 }
        .Dec -> { .count = .count - 1 }
    }
}
"""


class McpClient:
    """JSON-RPC client for the UI MCP server."""

    def __init__(self, url=MCP_URL):
        self.url = url
        self.req_id = 0

    def call(self, tool_name, **arguments):
        self.req_id += 1
        resp = requests.post(self.url, json={
            "jsonrpc": "2.0", "method": "tools/call",
            "params": {"name": tool_name, "arguments": arguments},
            "id": self.req_id,
        }, timeout=10)
        data = resp.json()
        if "error" in data:
            raise RuntimeError(f"MCP error: {data['error']}")
        # Result content is in [{text: "...", type: "text"}]
        content = data.get("result", {}).get("content", [])
        return content[0]["text"] if content else ""

    def snapshot(self):
        return self.call("autoui_snapshot")

    def click(self, element_id):
        return self.call("autoui_action", element_id=element_id, action="press")

    def state(self, *fields):
        return self.call("autoui_state", fields=list(fields))


def wait_for_server(url=MCP_URL, timeout=30):
    for _ in range(timeout):
        try:
            requests.post(url, json={
                "jsonrpc": "2.0", "method": "tools/list", "params": {}, "id": 1
            }, timeout=2)
            return True
        except (requests.ConnectionError, requests.Timeout):
            time.sleep(1)
    return False


class TestResult:
    def __init__(self):
        self.passed = 0; self.failed = 0; self.errors = []

    def check(self, name, condition, detail=""):
        if condition:
            self.passed += 1; print(f"  ✅ {name}")
        else:
            self.failed += 1; self.errors.append(f"{name}: {detail}")
            print(f"  ❌ {name}: {detail}")


def run_tests():
    mcp = McpClient()
    result = TestResult()

    # T-MCP-1: Snapshot shows widget structure
    print("\nT-MCP-1: UI Snapshot")
    snap = mcp.snapshot()
    result.check("Snapshot contains widget name", "Counter" in snap, snap[:200])
    result.check("Snapshot has aura IDs", "aura_" in snap, "No aura IDs found")

    # T-MCP-2: Initial state = 0
    print("\nT-MCP-2: Initial State")
    state = mcp.state("count")
    result.check("Initial count is 0", "count: 0" in state, state)

    # T-MCP-3: Click Inc → count becomes 1
    print("\nT-MCP-3: Click Inc Button")
    action_result = mcp.click("aura_2")  # aura_2 = Inc button from snapshot
    result.check("Action status ok", "status: ok" in action_result, action_result)
    result.check("Count changed 0→1", "0 -> 1" in action_result, action_result)

    # T-MCP-4: State after Inc = 1
    print("\nT-MCP-4: State After Inc")
    state = mcp.state("count")
    result.check("Count is 1", "count: 1" in state, state)

    # T-MCP-5: Click Inc again → count = 2
    print("\nT-MCP-5: Second Inc Click")
    action_result = mcp.click("aura_2")
    result.check("Count changed 1→2", "1 -> 2" in action_result, action_result)

    # T-MCP-6: Click Dec → count back to 1
    print("\nT-MCP-6: Click Dec Button")
    action_result = mcp.click("aura_3")  # aura_3 = Dec button
    result.check("Count changed 2→1", "2 -> 1" in action_result, action_result)

    state = mcp.state("count")
    result.check("Count is 1 after Dec", "count: 1" in state, state)

    return result


def main():
    print("=" * 60)
    print("Plan 370 Phase 2: Desktop MCP Interaction Tests")
    print("=" * 60)

    # Create temp project
    tmpdir = tempfile.mkdtemp(prefix="auto_vm_test_")
    os.makedirs(os.path.join(tmpdir, "src", "front"))
    with open(os.path.join(tmpdir, "pac.at"), "w") as f:
        f.write('name: "counter"\nversion: "1.0.0"\nscene: "ui"\nrender: "vm"\napi: "vm"\n')
    with open(os.path.join(tmpdir, "src", "front", "app.at"), "w") as f:
        f.write(COUNTER_AT)

    # Start VM mode
    print(f"\nStarting VM mode in {tmpdir}...")
    proc = subprocess.Popen(
        [AUTO_BIN, "run", "-r", "vm"],
        cwd=tmpdir,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

    try:
        print(f"Waiting for MCP server on port {MCP_PORT}...")
        if not wait_for_server():
            print("❌ MCP server did not start within 30 seconds")
            proc.kill()
            sys.exit(1)
        print("✅ MCP server ready")

        # Wait for UI to render (iced needs a few seconds to open window + first frame)
        print("Waiting for UI to render...")
        for i in range(15):
            time.sleep(2)
            try:
                snap = mcp.call("autoui_snapshot")
                if "aura_" in snap or "Counter" in snap:
                    print(f"✅ UI rendered after {(i+1)*2}s")
                    break
            except:
                pass
        else:
            print("⚠ UI may not have rendered, running tests anyway...")

        result = run_tests()

        print(f"\n{'='*60}")
        print(f"Results: {result.passed} passed, {result.failed} failed")
        if result.errors:
            for err in result.errors:
                print(f"  ❌ {err}")
        print(f"{'='*60}")

        sys.exit(0 if result.failed == 0 else 1)
    finally:
        proc.kill()
        proc.wait()
        print("VM process terminated.")


if __name__ == "__main__":
    main()
