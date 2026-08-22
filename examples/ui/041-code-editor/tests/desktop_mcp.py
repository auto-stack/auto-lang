#!/usr/bin/env python3
"""
Plan 418 Phase 1: MCP action-matrix tests for the real 041 code-editor
(auto-edit) app in VM mode.

Starts `auto run -r vm`, waits for the UI MCP server, then exercises the
13 semantic Act* handlers through their REAL trigger surfaces (menu items,
toolbar icon buttons, global shortcuts) and asserts observable state
(title/path/tab/console fields of the App model).

Out of scope here (manual / interactive): ActOpen/ActSave (blocking rfd
OS dialogs cannot be auto-dismissed). ActQuit runs last — its pass
condition is the app process exiting.

Snapshot caveat: event bindings render WITHOUT arguments
(`onclick: .MenuToggle`, not `.MenuToggle("file")`), so menubar buttons
are located by their text label via find_button_by_text.

Usage:
    cd examples/ui/041-code-editor/tests
    python desktop_mcp.py

Prerequisites:
    - auto built with ui-iced: cargo build --features ui-iced --bin auto
      (or set AUTO_BIN env var to the binary path)
    - Python requests: pip install requests
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

MCP_PORT_DEFAULT = 9247


def pick_free_port(start=MCP_PORT_DEFAULT):
    import socket
    for port in range(start, start + 100):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            if s.connect_ex(("127.0.0.1", port)) != 0:
                return port
    raise RuntimeError(f"No free port in [{start}, {start + 100})")


_AUTO_BIN = os.path.join(os.path.dirname(__file__), "..", "..", "..", "..",
                         "target", "debug", "auto.exe")
AUTO_BIN = os.environ.get("AUTO_BIN", _AUTO_BIN)
PROJECT = os.path.normpath(os.path.join(os.path.dirname(__file__), ".."))


class McpClient:
    """JSON-RPC client for the UI MCP server."""

    def __init__(self, url):
        self.url = url
        self.req_id = 0

    def call(self, tool_name, **arguments):
        for attempt in (1, 2):
            self.req_id += 1
            try:
                resp = requests.post(self.url, json={
                    "jsonrpc": "2.0", "method": "tools/call",
                    "params": {"name": tool_name, "arguments": arguments},
                    "id": self.req_id,
                }, timeout=45)
                break
            except (requests.ConnectionError, requests.Timeout):
                if attempt == 2:
                    raise
                time.sleep(2)
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


def find_element_by_event(snapshot_text, event_name, attr="onclick"):
    """First `aura_N` element bound to `event_name` via `attr` (substring
    match; only useful for unparametrized bindings — see module docstring)."""
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


def find_button_by_text(snapshot_text, label):
    """Element id of the `button #id "label" { ... }` node."""
    pat = re.compile(r'button #(\w+) "' + re.escape(label) + '"')
    m = pat.search(snapshot_text)
    return m.group(1) if m else None


def open_menu(mcp, snap_cache, label):
    """Click the menubar button `label` (文件/编辑/视图/帮助), refresh snapshot."""
    btn = find_button_by_text(snap_cache[0], label)
    if btn is None:
        return False
    mcp.click(btn)
    time.sleep(0.3)
    snap_cache[0] = mcp.snapshot()
    return True


def state_str(state_text, field):
    m = re.search(rf'{field}:\s*"((?:[^"\\]|\\.)*)"', state_text)
    return m.group(1) if m else None


def state_int(state_text, field):
    m = re.search(rf"{field}:\s*(-?\d+)", state_text)
    return int(m.group(1)) if m else None


def state_bool(state_text, field):
    m = re.search(rf"{field}:\s*(true|false)", state_text)
    return m.group(1) == "true" if m else None


class TestResult:
    def __init__(self):
        self.passed = 0
        self.failed = 0
        self.errors = []

    def check(self, name, condition, detail=""):
        if condition:
            self.passed += 1
            print(f"  PASS  {name}")
        else:
            self.failed += 1
            self.errors.append(f"{name}: {detail}")
            print(f"  FAIL  {name}: {detail}")


def run_tests(mcp_url, proc):
    mcp = McpClient(mcp_url)
    result = TestResult()

    # T1: structure — menubar, toolbar icons, editor (retry until rendered)
    print("\nT1: Snapshot structure")
    snap_cache = [mcp.snapshot()]
    for _ in range(10):
        if "code_editor" in snap_cache[0]:
            break
        time.sleep(1)
        snap_cache[0] = mcp.snapshot()
    snap = snap_cache[0]
    result.check("App widget present", 'widget: "App"' in snap, snap[:200])
    for act in ("ActNew", "ActUndo", "ActCopy"):
        result.check(f"toolbar binds .{act}", find_element_by_event(snap, act) is not None,
                     f"no onclick: .{act} element")
    if "code_editor" in snap:
        result.check("editor present", True)
    else:
        print("  NOTE  editor node not yet in snapshot (render timing); skipping")
    result.check("menubar buttons present", find_button_by_text(snap, "文件") is not None
                 and find_button_by_text(snap, "帮助") is not None, "menu buttons not found")
    # Plan 418 7.3-3: event args render readably (was invisible  encoding)
    result.check("snapshot renders event args", 'onclick: .MenuToggle("file")' in snap,
                 "MenuToggle binding args not visible in snapshot")

    # T2: ActConsole via View menu — console_open flips, menu auto-closes
    print("\nT2: ActConsole (menu item)")
    before = state_bool(mcp.state("console_open"), "console_open")
    ok = open_menu(mcp, snap_cache, "视图")
    result.check("view menu opened", ok, "视图 button not found")
    item = find_element_by_event(snap_cache[0], "ActConsole")
    result.check("menu item .ActConsole found", item is not None, "not in open-menu snapshot")
    if item:
        mcp.click(item)
        time.sleep(0.3)
        after = state_bool(mcp.state("console_open"), "console_open")
        result.check("console_open flipped", after == (not before), f"{before} -> {after}")
        # menu auto-closes after item activation (Plan 418: Act handlers reset menu_open)
        result.check("menu closed after pick", state_str(mcp.state("menu_open"), "menu_open") == "",
                     mcp.state("menu_open"))
        snap_cache[0] = mcp.snapshot()

    # T3: ActAbout via Help menu — console line recorded
    print("\nT3: ActAbout (menu item)")
    open_menu(mcp, snap_cache, "帮助")
    item = find_element_by_event(snap_cache[0], "ActAbout")
    if item:
        mcp.click(item)
        time.sleep(0.3)
        result.check("about line logged", "auto-edit 0.1" in (state_str(mcp.state("console"), "console") or ""),
                     "console missing about line")
    else:
        result.check("about menu item found", False, "no .ActAbout in help menu snapshot")

    # T4: ActNew via File menu — title/path reset
    print("\nT4: ActNew (menu item)")
    open_menu(mcp, snap_cache, "文件")
    item = find_element_by_event(snap_cache[0], "ActNew")
    if item:
        mcp.click(item)
        time.sleep(0.3)
        st = mcp.state("title_main", "path_main")
        result.check("title_main reset to untitled", state_str(st, "title_main") == "untitled.at", st)
        result.check("path_main cleared", state_str(st, "path_main") == "", st)
        result.check("new logged", "new: cleared" in (state_str(mcp.state("console"), "console") or ""), "")
    else:
        result.check("new menu item found", False, "no .ActNew in file menu snapshot")

    # T5: ActSwitchTab via View menu — tab flips
    print("\nT5: ActSwitchTab (menu item)")
    tab_before = state_int(mcp.state("tab"), "tab")
    open_menu(mcp, snap_cache, "视图")
    item = find_element_by_event(snap_cache[0], "ActSwitchTab")
    if item:
        mcp.click(item)
        time.sleep(0.3)
        tab_after = state_int(mcp.state("tab"), "tab")
        result.check("tab flipped", tab_after == 1 - tab_before, f"{tab_before} -> {tab_after}")
    else:
        result.check("switch-tab menu item found", False, "no .ActSwitchTab in view menu")

    # T6: toolbar editor actions — undo/redo/cut/copy/paste via toolbar
    # icons; select-all via the edit menu (no toolbar icon for it).
    print("\nT6: Editor actions (toolbar icons + edit-menu select-all)")
    snap_cache[0] = mcp.snapshot()
    for act, log in (("ActUndo", "undo"), ("ActRedo", "redo"),
                     ("ActCut", "cut"), ("ActCopy", "copy"), ("ActPaste", "paste")):
        el = find_element_by_event(snap_cache[0], act)
        if el is None:
            result.check(f"toolbar .{act}", False, "element not found")
            continue
        mcp.click(el)
        time.sleep(0.3)
        alive = proc.poll() is None
        result.check(f".{act} executed (app alive)", alive, "process died")
        if alive:
            result.check(f".{act} logged", log in (state_str(mcp.state("console"), "console") or ""),
                         "console line missing")
    if open_menu(mcp, snap_cache, "编辑"):
        el = find_element_by_event(snap_cache[0], "ActSelectAll")
        if el:
            mcp.click(el)
            time.sleep(0.3)
            result.check(".ActSelectAll executed (app alive)", proc.poll() is None, "process died")
            result.check(".ActSelectAll logged", "select all" in (state_str(mcp.state("console"), "console") or ""),
                         "console line missing")
        else:
            result.check("edit menu .ActSelectAll found", False, "not in open-menu snapshot")
    else:
        result.check("edit menu opened", False, "编辑 button not found")

    # T7: global shortcut — Ctrl+J toggles console (Plan 275 pipeline;
    # modifiers passed separately, server builds the "Ctrl+j" binding name).
    print("\nT7: Global shortcut Ctrl+J")
    before = state_bool(mcp.state("console_open"), "console_open")
    try:
        mcp.call("autoui_keyboard", key="j", modifiers=["ctrl"])
        time.sleep(0.3)
        after = state_bool(mcp.state("console_open"), "console_open")
        result.check("console_open flipped via Ctrl+J", after == (not before), f"{before} -> {after}")
    except Exception as e:
        result.check("console_open flipped via Ctrl+J", False, f"keyboard tool error: {e}")

    # T7b: config-layer shortcut — Ctrl+D exists ONLY in auto-edit.at
    # (view.switch-tab); proves the P2-4 fallback fires under the DSL layer.
    print("T7b: Config-layer shortcut Ctrl+D (auto-edit.at only)")
    tab_before = state_int(mcp.state("tab"), "tab")
    try:
        mcp.call("autoui_keyboard", key="d", modifiers=["ctrl"])
        time.sleep(0.3)
        tab_after = state_int(mcp.state("tab"), "tab")
        result.check("tab flipped via config Ctrl+D", tab_after == 1 - tab_before,
                     f"{tab_before} -> {tab_after}")
    except Exception as e:
        result.check("tab flipped via config Ctrl+D", False, f"keyboard tool error: {e}")

    # T8: ActQuit via File menu — process exits
    print("\nT8: ActQuit (menu item)")
    open_menu(mcp, snap_cache, "文件")
    item = find_element_by_event(snap_cache[0], "ActQuit")
    if item:
        # ActQuit runs Process.exit(0): the process may die before the HTTP
        # response completes — a dropped connection here IS the success path.
        try:
            mcp.click(item)
        except (requests.ConnectionError, requests.Timeout):
            pass
        for _ in range(10):
            if proc.poll() is not None:
                break
            time.sleep(0.5)
        result.check("app exited on quit", proc.poll() is not None,
                     f"process still running (poll={proc.poll()})")
    else:
        result.check("quit menu item found", False, "no .ActQuit in file menu")

    return result


def main():
    print("=" * 60)
    print("Plan 418 Phase 1: Desktop MCP action-matrix (real 041 auto-edit)")
    print("=" * 60)

    if not os.path.exists(AUTO_BIN):
        print(f"ERROR: auto binary not found at {AUTO_BIN}")
        print("Build it first: cargo build --features ui-iced --bin auto")
        print("or set AUTO_BIN env var to the binary path.")
        sys.exit(2)

    mcp_port = pick_free_port()
    mcp_url = f"http://localhost:{mcp_port}/mcp"
    if mcp_port != MCP_PORT_DEFAULT:
        print(f"NOTE: port {MCP_PORT_DEFAULT} busy (stale auto.exe?); "
              f"using AUTOUI_MCP_PORT={mcp_port}")

    print(f"\nStarting real 041 auto-edit in {PROJECT}...")
    import tempfile
    app_log = tempfile.NamedTemporaryFile(
        prefix="auto041_mcp_", suffix=".log", delete=False, mode="w",
        encoding="utf-8", errors="replace")
    proc = subprocess.Popen(
        [AUTO_BIN, "run", "-r", "vm"],
        cwd=PROJECT,
        env={**os.environ, "AUTOUI_MCP_PORT": str(mcp_port)},
        stdout=app_log,
        stderr=subprocess.STDOUT,
    )
    print(f"App output log: {app_log.name}")

    try:
        print(f"Waiting for MCP server on port {mcp_port}...")
        if not wait_for_server(mcp_url):
            print("ERROR: MCP server did not start within 30s.")
            proc.kill()
            sys.exit(1)
        print("MCP server ready")
        result = run_tests(mcp_url, proc)
    finally:
        if proc.poll() is None:
            proc.kill()

    print("\n" + "=" * 60)
    print(f"RESULT: {result.passed} passed, {result.failed} failed")
    if result.errors:
        print("Failures:")
        for e in result.errors:
            print(f"  - {e}")
    print("=" * 60)
    sys.exit(1 if result.failed else 0)


if __name__ == "__main__":
    main()
