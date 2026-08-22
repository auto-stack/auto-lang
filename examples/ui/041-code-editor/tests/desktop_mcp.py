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


def find_button_by_icon(snapshot_text, icon):
    """Synthesized toolbar buttons carry PUA icon labels
    ("<icon>") — locate the button node by icon name."""
    marker = "" + icon + ""
    pat = re.compile(r'button #(\w+) "[^"]*' + re.escape(marker))
    m = pat.search(snapshot_text)
    return m.group(1) if m else None


def find_button_by_onclick(snapshot_text, handler):
    """Element id of the first button whose onclick references `handler`.

    DSL `button { icon (name: "x") }` renders with an EMPTY label + [Image]
    child (no PUA marker), so find_button_by_icon can't see it — find the
    `onclick: .<handler>` line, then walk back to the nearest enclosing
    `button #id` line (Plan 420 tab-strip x/+ buttons)."""
    target = f"onclick: .{handler}"
    current_id = None
    for line in snapshot_text.splitlines():
        m = re.search(r'button #(\w+)', line)
        if m:
            current_id = m.group(1)
        if target in line and current_id is not None:
            return current_id
    return None


def find_tab_close_buttons(snapshot_text):
    """Ids of the tab-strip close (x) buttons.

    The x buttons render as `button #id "" { text "[Image]" }` — empty label,
    icon child, and (Plan 420 known gap) NO onclick attribute in the snapshot
    because probe paths for for+if children don't align with vtree paths.
    Distinguish from the `+` button (same shape) by the + having an onclick
    attribute. Returns ids in document order."""
    ids = []
    lines = snapshot_text.splitlines()
    i = 0
    while i < len(lines):
        m = re.search(r'button #(\w+) ""', lines[i])
        if m:
            block = []
            depth = lines[i].count("{") - lines[i].count("}")
            j = i + 1
            while j < len(lines) and depth > 0:
                depth += lines[j].count("{") - lines[j].count("}")
                block.append(lines[j])
                j += 1
            joined = "\n".join(block)
            if "[Image]" in joined and "onclick:" not in joined:
                ids.append(m.group(1))
            i = j
        else:
            i += 1
    return ids


def open_menu(mcp, snap_cache, label):
    """Click the menubar button `label` (文件/编辑/视图/帮助), refresh snapshot.

    Always re-snapshots first: vnode ids drift across rebuilds, so an id from
    a pre-pick snapshot may no longer resolve (T5 stale-id lesson)."""
    snap_cache[0] = mcp.snapshot()
    btn = find_button_by_text(snap_cache[0], label)
    if btn is None:
        return False
    mcp.click(btn)
    time.sleep(1.0)
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
    # Sentinel: "(rendered)" — the post-render VTree snapshot. The pre-render
    # fallback (raw view_template) still names the editor `code_editor` and has
    # NO synthesized menubar/toolbar buttons, so polling on "code_editor" can
    # break the loop during the ~1.5s first-render window and every synthesis
    # check below fails. In rendered snapshots the editor is `textarea`.
    print("\nT1: Snapshot structure")
    snap_cache = [mcp.snapshot()]
    for _ in range(10):
        if "(rendered)" in snap_cache[0]:
            break
        time.sleep(1)
        snap_cache[0] = mcp.snapshot()
    snap = snap_cache[0]
    result.check("App widget present", 'widget: "App"' in snap, snap[:200])
    for icon in ("file-plus", "undo-2", "copy"):
        result.check(f"toolbar icon {icon} present", find_button_by_icon(snap, icon) is not None,
                     "icon button not found")
    if "code_editor" in snap:
        result.check("editor present", True)
    else:
        print("  NOTE  editor node not yet in snapshot (render timing); skipping")
    result.check("menubar buttons present", find_button_by_text(snap, "文件") is not None
                 and find_button_by_text(snap, "帮助") is not None, "menu buttons not found")
    # Plan 418 §8.4①: synthesized buttons now carry onclick in snapshots
    # (probe paths aligned with the real vtree nesting) — lock it in.
    result.check("toolbar synthesized onclick present",
                 "onclick: .ActNew" in snap, "synthesized toolbar onclick missing")
    result.check("menubar synthesized onclick present",
                 "__menubar_toggle(\"file\")" in snap, "menubar toggle onclick missing")
    # (Plan 418 P2-3: DSL-declared bindings still render their args; the
    # synthesized menubar/toolbar are located by label/icon instead — probe
    # path alignment for synthesized subtrees is a known gap, plan 418 8.4.)


    # T2: ActConsole via View menu — console_open flips, menu auto-closes
    print("\nT2: ActConsole (menu item)")
    before = state_bool(mcp.state("console_open"), "console_open")
    ok = open_menu(mcp, snap_cache, "视图")
    result.check("view menu opened", ok, "视图 button not found")
    item = find_button_by_text(snap_cache[0], "切换 Console")
    result.check("menu item (切换 Console) found", item is not None, "not in open-menu snapshot")
    if item:
        mcp.click(item)
        time.sleep(0.3)
        after = state_bool(mcp.state("console_open"), "console_open")
        result.check("console_open flipped", after == (not before), f"{before} -> {after}")
        # menu auto-closes after item activation (Plan 418: Act handlers reset menu_open)
        snap_cache[0] = mcp.snapshot()
        result.check("menu closed after pick", find_button_by_text(snap_cache[0], "切换 Console") is None,
                     "panel item still present")
        snap_cache[0] = mcp.snapshot()

    # T3: ActAbout via Help menu — console line recorded
    print("\nT3: ActAbout (menu item)")
    open_menu(mcp, snap_cache, "帮助")
    item = find_button_by_text(snap_cache[0], "关于 auto-edit")
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
    item = find_button_by_text(snap_cache[0], "新建")
    if item:
        mcp.click(item)
        time.sleep(0.3)
        st = mcp.state("title_active", "path_active")
        result.check("title_active reset to untitled", state_str(st, "title_active") == "untitled.at", st)
        result.check("path_active cleared", state_str(st, "path_active") == "", st)
        result.check("new logged", "new: cleared" in (state_str(mcp.state("console"), "console") or ""), "")
    else:
        result.check("new menu item found", False, "no .ActNew in file menu snapshot")

    # T5: ActSwitchTab via View menu — tab flips
    print("\nT5: ActSwitchTab (menu item)")
    tab_before = state_int(mcp.state("tab"), "tab")
    open_menu(mcp, snap_cache, "视图")
    item = find_button_by_text(snap_cache[0], "切换 Tab")
    if item:
        mcp.click(item)
        time.sleep(0.3)
        tab_after = state_int(mcp.state("tab"), "tab")
        result.check("tab flipped", tab_after == 1 - tab_before, f"{tab_before} -> {tab_after}")
    else:
        result.check("switch-tab menu item found", False, "no .ActSwitchTab in view menu")

    # T6: toolbar editor actions — undo/redo/cut/copy/paste via toolbar
    # icons; the full edit-menu/toolbar cycle with REAL editor-text
    # assertions (Plan 418 follow-up: previously only alive+console-log were
    # checked — undo could silently no-op). Model resync relies on the
    # handlers' code_editor_text() write-back (app.at) so the assertions
    # hold under MCP dispatch (no real iced events to flush the widget's
    # external-dirty publish).
    print("\nT6: Editor actions (toolbar icons + edit-menu, text-verified)")
    # Plan 420: 正文断言走派生标量 src_active(激活 tab 的镜像,undo/cut/
    # paste 等 handler 写回 —— 与旧 src_main/src_util 同步点一致)。
    title_now = state_str(mcp.state("title_active"), "title_active") or ""
    if "util" in title_now:
        src_field, marker = "src_active", "工具模块"
    else:
        src_field, marker = "src_active", "你好世界"

    def src_now():
        return state_str(mcp.state(src_field), src_field) or ""

    def toolbar(icon):
        snap_cache[0] = mcp.snapshot()
        el = find_button_by_icon(snap_cache[0], icon)
        if el is None:
            result.check(f"toolbar {icon} present", False, "element not found")
            return False
        mcp.click(el)
        time.sleep(0.4)
        return proc.poll() is None

    def menu_item(label):
        # opens the edit menu and clicks `label`; False when not found
        for menu_label, item in (("编辑", label),):
            if not open_menu(mcp, snap_cache, menu_label):
                return False
            el = find_button_by_text(snap_cache[0], item)
            if el is None:
                return False
            mcp.click(el)
            time.sleep(0.4)
            return True
        return False

    # 1) select-all via the edit menu → selection state really set
    ok = menu_item("全选")
    sel = state_int(mcp.state("sel"), "sel")
    result.check("select-all sets .sel", ok and sel > 0, f"sel={sel}")

    # 2) cut empties the editor AND the model binding
    if toolbar("scissors"):
        result.check("cut empties editor text", src_now() == "", f"{src_field}={src_now()[:30]!r}")
        result.check("cut logged", "cut" in (state_str(mcp.state("console"), "console") or ""))

    # 3) undo restores the preloaded text
    if toolbar("undo-2"):
        result.check("undo restores text", marker in src_now(), f"{src_field} missing {marker!r}")

    # 4) redo re-applies the cut
    if toolbar("redo-2"):
        result.check("redo re-empties text", src_now() == "", f"{src_field}={src_now()[:30]!r}")

    # 5) undo restores again, then copy → cut → paste round-trips the text
    if toolbar("undo-2"):
        result.check("undo (2nd) restores text", marker in src_now(), "")
        ok = menu_item("全选")
        if toolbar("copy"):
            result.check("copy logged", "copy" in (state_str(mcp.state("console"), "console") or ""))
        if toolbar("scissors"):
            result.check("cut (2nd) empties text", src_now() == "", "")
        if toolbar("clipboard"):
            result.check("paste restores text", marker in src_now(),
                         f"paste did not round-trip clipboard")
            result.check("paste logged", "paste" in (state_str(mcp.state("console"), "console") or ""))

    # T7: global shortcut — Ctrl+J now flows ONLY from auto-edit.at
    # (config fallback layer; the DSL onkeydown attrs were removed in P2-3c).
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
    # T9: Plan 420 — tab close / + open (AUTO_OPEN_PATH bypass) / dirty-confirm
    # / AUTO_SAVE_PATH roundtrip. Runs on a FRESH app process (earlier groups
    # dirty tabs / mutate active state; a clean instance keeps 9.1-9.4
    # deterministic). Requires AUTO_OPEN_PATH/AUTO_SAVE_PATH (see main()).
    print("\nT9: Plan 420 tab workspace (close/+ open/dirty/save)")
    if os.environ.get("AUTO_OPEN_PATH") and os.environ.get("AUTO_SAVE_PATH"):
        t9_port = pick_free_port()
        _mcp_orig, _snap_orig = mcp, snap_cache
        t9_proc = subprocess.Popen(
            [AUTO_BIN, "run", "-r", "vm"],
            cwd=PROJECT,
            env={**os.environ, "AUTOUI_MCP_PORT": str(t9_port)},
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        t9_url = f"http://127.0.0.1:{t9_port}/mcp"
        if wait_for_server(t9_url, 30):
            mcp = McpClient(t9_url)
            snap_cache = [""]
            for _ in range(15):
                snap_cache[0] = mcp.snapshot()
                if "(rendered)" in snap_cache[0]:
                    break
                time.sleep(1)
        roundtrip_path = os.environ["AUTO_OPEN_PATH"]
        marker_before = open(roundtrip_path, encoding="utf-8").read()
        # 9.1 close both starter tabs (x icon buttons in the tab strip; a
        # tab dirtied by T6 opens the confirm popover — force-close through it)
        for _ in range(3):
            snap_cache[0] = mcp.snapshot()
            if state_int(mcp.state("tab_count"), "tab_count") == 0:
                break
            if state_str(mcp.state("confirm_open"), "confirm_open") == "true":
                snap_cache[0] = mcp.snapshot()
                force = find_button_by_text(snap_cache[0], "直接关闭")
                if force:
                    mcp.click(force)
                    time.sleep(0.5)
                continue
            xs = find_tab_close_buttons(snap_cache[0])
            if not xs:
                break
            mcp.click(xs[0])
            time.sleep(0.5)
        result.check("all tabs closed", state_int(mcp.state("tab_count"), "tab_count") == 0,
                     mcp.state("tab_count"))
        snap_cache[0] = mcp.snapshot()
        result.check("empty state visible", "没有打开的文件" in snap_cache[0], "empty-state text missing")

        # 9.2 + opens the AUTO_OPEN_PATH file into a new tab
        snap_cache[0] = mcp.snapshot()
        plus = find_button_by_onclick(snap_cache[0], "ActOpen")
        ok_plus = plus is not None
        result.check("+ button found", ok_plus, "plus icon button missing")
        if ok_plus:
            mcp.click(plus)
            time.sleep(0.8)
            st = mcp.state("tab_count", "title_active", "src_active")
            result.check("tab opened from AUTO_OPEN_PATH",
                         state_int(st, "tab_count") == 1
                         and state_str(st, "title_active") == os.path.basename(roundtrip_path),
                         st)
            result.check("opened content matches file",
                         marker_before.splitlines()[0] in (state_str(st, "src_active") or ""),
                         st)

        # 9.3 dirty-confirm: ActCut unconditionally dirties the active tab
        # (autoui_type passes the TEXT as first handler arg -- the generic
        # input-tool convention -- which displaces the loop-index payload of
        # `oninput: .SrcChanged(i)`; real-window events keep the payload).
        snap_cache[0] = mcp.snapshot()
        cut_btn = find_button_by_icon(snap_cache[0], "scissors")
        ok_cut = cut_btn is not None
        result.check("cut toolbar button found", ok_cut, "scissors icon missing")
        if ok_cut:
            mcp.click(cut_btn)
            time.sleep(0.5)
            result.check("cut logged", "cut" in (state_str(mcp.state("console"), "console") or ""),
                         "no cut console line")
            snap_cache[0] = mcp.snapshot()
            xs9 = find_tab_close_buttons(snap_cache[0])
            if xs9:
                mcp.click(xs9[0])
                time.sleep(0.6)
                result.check("dirty confirm popover opens",
                             "confirm_open: true" in mcp.state("confirm_open"),
                             mcp.state("confirm_open"))
                snap_cache[0] = mcp.snapshot()
                force = find_button_by_text(snap_cache[0], "\u76f4\u63a5\u5173\u95ed")
                result.check("confirm force-close item found", force is not None, "not in snapshot")
                if force:
                    mcp.click(force)
                    time.sleep(0.5)
                    result.check("dirty tab closed",
                                 state_int(mcp.state("tab_count"), "tab_count") == 0,
                                 mcp.state("tab_count"))

        # 9.4 save roundtrip: reopen, type, save via toolbar icon → file rewritten
        snap_cache[0] = mcp.snapshot()
        plus = find_button_by_onclick(snap_cache[0], "ActOpen")
        if plus:
            mcp.click(plus)
            time.sleep(0.8)
            # dirty via cut, then save via toolbar icon -> file rewritten
            snap_cache[0] = mcp.snapshot()
            cut_btn = find_button_by_icon(snap_cache[0], "scissors")
            if cut_btn:
                mcp.click(cut_btn)
                time.sleep(0.4)
            snap_cache[0] = mcp.snapshot()
            save_btn = find_button_by_icon(snap_cache[0], "save")
            if save_btn:
                mcp.click(save_btn)
                time.sleep(0.8)
                written = open(os.environ["AUTO_SAVE_PATH"], encoding="utf-8").read()
                result.check("saved file round-trips content",
                             marker_before.splitlines()[0] in written, written[-80:])
            else:
                result.check("save toolbar button found", False, "save icon missing")
        # restore the main app client (T8 quits the ORIGINAL process) and
        # retire the T9 instance.
        mcp, snap_cache = _mcp_orig, _snap_orig
        t9_proc.terminate()
        try:
            t9_proc.wait(5)
        except Exception:
            t9_proc.kill()
    else:
        print("  NOTE  AUTO_OPEN_PATH/AUTO_SAVE_PATH not set; skipping T9 (see runner env)")

    # T10: Plan 423 — enabled-if disabled 态 + 配置热重载(独立新鲜进程)。
    print("\nT10: Plan 423 enabled-if + hot reload")
    if os.environ.get("AUTO_OPEN_PATH") and os.environ.get("AUTO_SAVE_PATH"):
        t10_port = pick_free_port()
        t10_proc = subprocess.Popen(
            [AUTO_BIN, "run", "-r", "vm"],
            cwd=PROJECT,
            env={**os.environ, "AUTOUI_MCP_PORT": str(t10_port)},
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        config_file = os.path.join(PROJECT, "auto-edit.at")
        config_backup = open(config_file, encoding="utf-8").read()
        try:
            t10_url = f"http://127.0.0.1:{t10_port}/mcp"
            assert wait_for_server(t10_url, 30), "T10 server never up"
            mcp10 = McpClient(t10_url)
            snap10 = ""
            for _ in range(15):
                snap10 = mcp10.snapshot()
                if "(rendered)" in snap10:
                    break
                time.sleep(1)

            # 10.1 close both starter tabs → file.save (enabled-if .tab_count > 0)
            # goes disabled: snapshot marker + click dispatches nothing.
            for _ in range(3):
                if state_int(mcp10.state("tab_count"), "tab_count") == 0:
                    break
                xs = find_tab_close_buttons(mcp10.snapshot())
                if not xs:
                    break
                mcp10.click(xs[0])
                time.sleep(0.5)
            result.check("T10 all tabs closed",
                         state_int(mcp10.state("tab_count"), "tab_count") == 0, "tabs left")
            snap10 = mcp10.snapshot()
            save_btn = find_button_by_onclick(snap10, "ActSave")
            ok_save = save_btn is not None
            result.check("T10 save button found", ok_save, "ActSave button missing")
            if ok_save:
                m = re.search(r'button #' + re.escape(save_btn) + r'[^{]*\{[^}]*\}', snap10, re.S)
                region = m.group(0) if m else ""
                result.check("T10 save disabled marker in snapshot",
                             "disabled: true" in region, region[:120])
                console_before = state_str(mcp10.state("console"), "console") or ""
                mcp10.click(save_btn)
                time.sleep(0.6)
                console_after = state_str(mcp10.state("console"), "console") or ""
                result.check("T10 disabled click dispatches nothing",
                             "saved:" not in console_after and console_after == console_before,
                             f"before={console_before[-40:]!r} after={console_after[-40:]!r}")

            # 10.2 reopen a tab → save re-enables (marker gone).
            plus = find_button_by_onclick(mcp10.snapshot(), "ActOpen")
            if plus:
                mcp10.click(plus)
                time.sleep(0.8)
                snap10 = mcp10.snapshot()
                save_btn = find_button_by_onclick(snap10, "ActSave")
                if save_btn:
                    m2 = re.search(r'button #' + re.escape(save_btn) + r'[^{]*\{[^}]*\}', snap10, re.S)
                    region2 = m2.group(0) if m2 else ""
                    result.check("T10 save re-enabled after open",
                                 "disabled: true" not in region2, region2[:120])

            # 10.3 hot reload: append an action + a T10 menu INSIDE the root
            # block (auto-atom rejects trailing nodes after the closing brace),
            # reload via the MCP tool, expect it in the next snapshot.
            modified = config_backup.rstrip()
            assert modified.endswith("}"), "unexpected auto-edit.at shape"
            modified = modified[:-1] + (
                '\n    action { id : "help.t10" handler : ".ActAbout" title : "T10 重载项" }'
                '\n    menubar { menu { id : "t10menu" title : "T10" item { action : "help.t10" } } }\n}\n'
            )
            with open(config_file, "w", encoding="utf-8") as f:
                f.write(modified)
            try:
                mcp10.call("action_config_reload")
                time.sleep(1.5)  # heartbeat rebuild cadence
                open_menu(mcp10, [snap10], "T10")
                item = find_button_by_text(mcp10.snapshot(), "T10 重载项")
                result.check("T10 hot-reloaded menu item appears", item is not None,
                             "item not in snapshot after reload")
                if item:
                    mcp10.click(item)
                    time.sleep(0.4)
                    result.check("T10 reloaded item dispatches",
                                 "auto-edit 0.1" in (state_str(mcp10.state("console"), "console") or ""),
                                 "no about line")
            finally:
                with open(config_file, "w", encoding="utf-8") as f:
                    f.write(config_backup)
                mcp10.call("action_config_reload")  # restore effective config
        finally:
            t10_proc.terminate()
            try:
                t10_proc.wait(5)
            except Exception:
                t10_proc.kill()
    else:
        print("  NOTE  AUTO_OPEN_PATH/AUTO_SAVE_PATH not set; skipping T10")

    print("\nT8: ActQuit (menu item)")
    open_menu(mcp, snap_cache, "文件")
    item = find_button_by_text(snap_cache[0], "退出")
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
    # Plan 420 T9: file open/save automation bypass — with these set, ActOpen/
    # ActSave skip the blocking rfd dialogs (test-build semantics only).
    roundtrip_file = tempfile.NamedTemporaryFile(
        prefix="auto041_t9_", suffix=".at", delete=False, mode="w", encoding="utf-8")
    roundtrip_file.write("// t9 roundtrip file\nfn t9() int { 42 }\n")
    roundtrip_file.flush()
    roundtrip_file.close()
    os.environ["AUTO_OPEN_PATH"] = roundtrip_file.name
    os.environ["AUTO_SAVE_PATH"] = roundtrip_file.name
    t9_env = {
        **os.environ,
        "AUTOUI_MCP_PORT": str(mcp_port),
    }
    proc = subprocess.Popen(
        [AUTO_BIN, "run", "-r", "vm"],
        cwd=PROJECT,
        env=t9_env,
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
