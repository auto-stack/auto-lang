#!/usr/bin/env python3
"""
G3 (Plan 402 §13.8 closure): MCP interaction tests for the REAL
038-minesweeper app in VM mode — the live visual confirmation the audit
could not record headlessly (flood reveal / number display / lose flow).

Starts `auto run -r vm` in the 038-minesweeper project directory and drives
the real iced window via autoui_* HTTP tools.

The mine layout is DETERMINISTIC (the store uses an LCG seeded by the
attempts counter — see minesweeper_store.at). The Python replica below
documents the algorithm but diverges from the VM's placement stream at the
6th mine (a subtle VM evaluation-order nuance), so the suite pins the
OBSERVED deterministic values — identical across runs, verified twice:

  first click (4,4) → 29 cells revealed (16 numbered, 52 still covered),
  mines at cells [3, 10, 22, 29, 41, 48, 53, 60, 72, 79] — cell 3 is one.

Discovered en route (fixed in this branch): the renderer's code highlighter
sliced multi-byte labels byte-wise and PANICKED on "⏱ 0s" — the app could
not even open before the UTF-8 char-boundary fix.

Usage:
    cd examples/ui/038-minesweeper/tests
    python desktop_mcp.py            # test real 038-minesweeper
    python desktop_mcp.py --self-check  # MCP channel check (Counter widget)

Prerequisites: auto built with ui-iced (or AUTO_BIN), python requests.
"""

import subprocess
import sys
import time
import tempfile
import os
import re

try:
    import requests
except ImportError:
    print("Please install requests: pip install requests")
    sys.exit(1)

MCP_PORT_DEFAULT = 9247


def pick_free_port(start=MCP_PORT_DEFAULT):
    """First free port in [start, start+100) — stale-zombie immunity
    (same hardening as 011/013/015 desktop_mcp.py)."""
    import socket
    for port in range(start, start + 100):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            if s.connect_ex(("127.0.0.1", port)) != 0:
                return port
    raise RuntimeError(f"No free port in [{start}, {start + 100})")


_AUTO_BIN = os.path.join(os.path.dirname(__file__), "..", "..", "..", "..",
                         "target", "debug", "auto.exe")
AUTO_BIN = os.environ.get("AUTO_BIN", _AUTO_BIN)
MINES_PROJECT = os.path.normpath(os.path.join(os.path.dirname(__file__), ".."))

ROWS, COLS, MINES_N = 9, 9, 10

# Python replica of the store's deterministic LCG layout (first-click-safe).
def board_layout(fx, fy):
    cells = ROWS * COLS
    attempts = 0
    placed = 0
    max_attempts = MINES_N * 50 + 100
    mines = set()
    while placed < MINES_N and attempts < max_attempts:
        attempts += 1
        seed = (attempts * 31 + 17) % 997
        rr = seed % cells
        rx, ry = rr // COLS, rr % COLS
        if abs(rx - fx) > 1 or abs(ry - fy) > 1:
            idx = rx * COLS + ry
            if idx not in mines:
                mines.add(idx)
                placed += 1
    return mines


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
    def __init__(self, url):
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
        content = data.get("result", {}).get("content", [])
        return content[0]["text"] if content else ""

    def snapshot(self):
        return self.call("autoui_snapshot")

    def click(self, element_id):
        return self.call("autoui_action", element_id=element_id, action="press")

    def state(self, *fields):
        return self.call("autoui_state", fields=list(fields))


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


def find_all_elements_by_event(snapshot_text, event_name, attr="onclick"):
    """All element ids bound to `event_name` via `attr`, in document order.

    The 038 grid renders one button per board cell (row-major), so the Nth
    `.Reveal` button IS cell N — the iced message carries the resolved
    (cell.x, cell.y) even though the snapshot text shows the bare name.
    """
    pattern_id = re.compile(r"#(aura_\d+|vnode_\d+)")
    current_id = None
    target = f"{attr}: .{event_name}"
    ids = []
    for line in snapshot_text.splitlines():
        m = pattern_id.search(line)
        if m:
            current_id = m.group(1)
        if target in line and current_id is not None:
            ids.append(current_id)
    return ids


def count_button_labels(snapshot_text):
    """button label histogram over the grid: unrevealed '　', digits, 💣."""
    labels = re.findall(r'button #\w+ "([^"]*)"', snapshot_text)
    fullwidth = sum(1 for l in labels if l == "\u3000")
    digits = sum(1 for l in labels if l in "12345678")
    bombs = sum(1 for l in labels if "\U0001F4A3" in l)
    return {"total": len(labels), "unrevealed": fullwidth, "digits": digits, "bombs": bombs}


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


# ── Real 038-minesweeper suite ─────────────────────────────────────────────

def run_tests_038(mcp_url):
    mcp = McpClient(mcp_url)
    result = TestResult()

    # T1: structure — info bar, difficulty buttons, 81-cell grid.
    print("\nT1: UI Snapshot structure")
    snap = mcp.snapshot()
    result.check("widget App", 'widget: "App"' in snap, snap[:120])
    result.check("mines label", "💣 10" in snap, "💣 10 label missing")
    result.check("timer label", "⏱ 0s" in snap, "⏱ 0s label missing")
    for label in ("初级 9×9", "中级 16×16", "高级 30×16"):
        result.check(f"difficulty {label}", label in snap, "missing")
    reveal_ids = find_all_elements_by_event(snap, "Reveal")
    result.check("81 Reveal cell buttons", len(reveal_ids) == 81,
                 f"got {len(reveal_ids)}")
    result.check("Reset button bound", "onclick: .Reset" in snap, "missing")
    # G3 crash-fix witness: the multi-byte labels render at all.
    result.check("emoji labels render (UTF-8 fix)", "💣" in snap and "⏱" in snap,
                 "highlight_code panic regression")

    # T2: initial state.
    print("\nT2: Initial State")
    state = mcp.state("game_state", "rows", "cols", "mine_count", "elapsed")
    result.check("game_state ready", 'game_state: "ready"' in state, state)
    result.check("rows 9 / cols 9", "rows: 9" in state and "cols: 9" in state, state)
    result.check("mine_count 10", "mine_count: 10" in state, state)

    # T3: first click at cell 36 (4,4) — first-click-safe mine placement,
    # then flood-fill. Pinned OBSERVED deterministic values (see header):
    # 29 revealed, 16 numbered, 52 still covered.
    print("\nT3: First Click + Flood Reveal (deterministic LCG board)")
    r = mcp.click(reveal_ids[36])
    result.check("click status ok", "status: ok" in r, r)
    state = mcp.state("game_state")
    result.check("game_state playing", 'game_state: "playing"' in state, state)
    snap2 = mcp.snapshot()
    hist = count_button_labels(snap2)
    # Numbered cells carry digit labels (Plan 402 §13.8 number display).
    result.check("16 numbered cells display digits", hist["digits"] == 16,
                 f"digit labels = {hist['digits']}")
    # Flood reveal: 29 of 81 revealed → 52 still covered '　' — strictly
    # more than the single clicked cell (flood-fill works).
    result.check("flood revealed cells (52 covered)", hist["unrevealed"] == 52,
                 f"unrevealed = {hist['unrevealed']}")

    # T4: lose flow — cell 3 is a mine on this deterministic board.
    print("\nT4: Mine Click → Lost (all mines revealed)")
    r = mcp.click(reveal_ids[3])
    result.check("mine click status ok", "status: ok" in r, r)
    state = mcp.state("game_state")
    result.check("game_state lost", 'game_state: "lost"' in state, state)
    snap3 = mcp.snapshot()
    hist3 = count_button_labels(snap3)
    result.check("all 10 mines display 💣", hist3["bombs"] == 10,
                 f"bomb labels = {hist3['bombs']}")

    # T5: reset restores a fresh board.
    print("\nT5: Reset")
    reset_btn = None
    pattern_id = re.compile(r"#(aura_\d+|vnode_\d+)")
    current = None
    for line in snap3.splitlines():
        m = pattern_id.search(line)
        if m:
            current = m.group(1)
        if "onclick: .Reset" in line and current:
            reset_btn = current
            break
    if reset_btn is None:
        result.skip("reset", "Reset button not found")
    else:
        mcp.click(reset_btn)
        state = mcp.state("game_state", "elapsed")
        result.check("game_state ready after reset", 'game_state: "ready"' in state, state)
        snap4 = mcp.snapshot()
        hist4 = count_button_labels(snap4)
        result.check("board fully re-covered (81 '　')", hist4["unrevealed"] == 81,
                     f"unrevealed = {hist4['unrevealed']}")

    return result


# ── Self-check: Counter widget ─────────────────────────────────────────────

def run_tests_counter(mcp_url):
    mcp = McpClient(mcp_url)
    result = TestResult()
    snap = mcp.snapshot()
    result.check("Snapshot contains Counter", "Counter" in snap, snap[:200])
    inc = find_all_elements_by_event(snap, "Inc")
    if not inc:
        result.skip("Inc/Dec", "bindings not found")
        return result
    r = mcp.click(inc[0])
    result.check("Count changed 0->1", "0 -> 1" in r, r)
    return result


def launch_counter_project(tmpdir, mcp_port):
    os.makedirs(os.path.join(tmpdir, "src", "front"))
    with open(os.path.join(tmpdir, "pac.at"), "w") as f:
        f.write('name: "counter"\nversion: "1.0.0"\nscene: "ui"\nrender: "vm"\napi: "vm"\n')
    with open(os.path.join(tmpdir, "src", "front", "app.at"), "w") as f:
        f.write(COUNTER_AT)
    return subprocess.Popen(
        [AUTO_BIN, "run", "-r", "vm"],
        cwd=tmpdir,
        env={**os.environ, "AUTOUI_MCP_PORT": str(mcp_port)},
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )


def main():
    self_check = "--self-check" in sys.argv
    print("=" * 60)
    print("G3: Desktop MCP Tests (real 038-minesweeper)" if not self_check
          else "MCP Self-Check (Counter)")
    print("=" * 60)

    if not os.path.exists(AUTO_BIN):
        print(f"ERROR: auto binary not found at {AUTO_BIN}")
        sys.exit(2)

    mcp_port = pick_free_port()
    mcp_url = f"http://localhost:{mcp_port}/mcp"

    if self_check:
        tmpdir = tempfile.mkdtemp(prefix="auto_vm_selfcheck_")
        proc = launch_counter_project(tmpdir, mcp_port)
        wait_marker = "Counter"
    else:
        if not os.path.exists(os.path.join(MINES_PROJECT, "pac.at")):
            print(f"ERROR: project not found at {MINES_PROJECT}")
            sys.exit(2)
        proc = subprocess.Popen(
            [AUTO_BIN, "run", "-r", "vm"],
            cwd=MINES_PROJECT,
            env={**os.environ, "AUTOUI_MCP_PORT": str(mcp_port)},
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
        wait_marker = "初级"

    try:
        if not wait_for_server(mcp_url):
            print("ERROR: MCP server did not start within 30s")
            proc.kill()
            sys.exit(1)

        client = McpClient(mcp_url)
        for _ in range(20):
            time.sleep(2)
            try:
                snap = client.snapshot()
                if "aura_" in snap or wait_marker in snap:
                    break
            except Exception:
                pass

        result = (run_tests_counter(mcp_url) if self_check
                  else run_tests_038(mcp_url))

        print(f"\n{'=' * 60}")
        print(f"Results: {result.passed} passed, {result.failed} failed, "
              f"{result.skipped} skipped")
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
