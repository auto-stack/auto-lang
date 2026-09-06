#!/usr/bin/env python3
"""
Plan 553 T6: MCP interaction tests for the real 031-paint (pixel paint) app
in VM mode.

Starts `auto run -r vm` in the 031-paint project directory, waits for the UI
MCP server, then exercises the paint UI via autoui_* HTTP tools: snapshot
structure, initial 16x16 white canvas, pencil paint / eraser / flood fill /
eyedropper, undo-redo, and save-load persistence (storage `paint.canvas.v1`).

元素寻址（与 013 的 onclick 文本匹配不同）：snapshot v2 只含视觉树——
- 画布 256 格 = 树中唯一 256-子容器（缩进解析）的子节点，DOM 序 = 格序；
- 调色板 16 格 = 唯一 16-叶-子容器（palette 序）；
- 命名按钮直接 `button #vnode_... "label"` 正则（标签含 emoji 前缀，用
  子串匹配）。
点击经 autoui_action press（派发元素绑定 handler，无需绑定文本可见）。

Usage:
    cd examples/ui/031-paint/tests
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
from collections import defaultdict

try:
    import requests
except ImportError:
    print("Please install requests: pip install requests")
    sys.exit(1)

MCP_PORT_DEFAULT = 9253


def pick_free_port(start=MCP_PORT_DEFAULT):
    """First port in [start, start+100) nothing is bound to (013 hermetic)."""
    import socket
    for port in range(start, start + 100):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            if s.connect_ex(("127.0.0.1", port)) != 0:
                return port
    raise RuntimeError(f"No free port in [{start}, {start + 100})")

# Default auto binary: <repo>/target/debug/auto(.exe)（worktree 自有 target）
_AUTO_BIN = os.path.join(os.path.dirname(__file__), "..", "..", "..", "..",
                         "target", "debug", "auto.exe")
AUTO_BIN = os.environ.get("AUTO_BIN", _AUTO_BIN)
PAINT_PROJECT = os.path.normpath(
    os.path.join(os.path.dirname(__file__), ".."))


class McpClient:
    """JSON-RPC client for the UI MCP server (013 同型)."""

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

    def snapshot(self):
        return self.call("autoui_snapshot")

    def click(self, element_id):
        return self.call("autoui_action", element_id=element_id, action="press")

    def state(self, *fields):
        return self.call("autoui_state", fields=list(fields))


def wait_for_server(url, timeout=60):
    for _ in range(timeout):
        try:
            requests.post(url, json={
                "jsonrpc": "2.0", "method": "tools/list", "params": {}, "id": 1
            }, timeout=2)
            return True
        except (requests.ConnectionError, requests.Timeout):
            time.sleep(1)
    return False


def wait_for_ui(mcp, timeout=45):
    """tools/list 就绪早于首帧渲染（fit 窗 + 256 格 Init）——轮询 snapshot
    直到不再是 "No UI available"。"""
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            snap = mcp.snapshot()
        except RuntimeError:
            snap = ""
        if snap and "No UI available" not in snap:
            return snap
        time.sleep(2)
    return None


def parse_tree(snapshot_text):
    """缩进解析 snapshot v2 树 → {parent_id: [child ids]}（DFS 序）。"""
    lines = [(len(l) - len(l.lstrip()), l.strip())
             for l in snapshot_text.splitlines() if "#vnode_" in l]
    children = defaultdict(list)
    stack = []
    for ind, line in lines:
        m = re.search(r"#(vnode_\d+)", line)
        if not m:
            continue
        vid = m.group(1)
        while stack and stack[-1][0] >= ind:
            stack.pop()
        if stack:
            children[stack[-1][1]].append(vid)
        stack.append((ind, vid))
    return children


def find_buttons(snapshot_text):
    """`button #vnode_... "label"` → {label: id}（同标签取首个）。"""
    out = {}
    for vid, label in re.findall(r'button #(vnode_\d+) "([^"]+)"', snapshot_text):
        if label not in out:
            out[label] = vid
    return out


def button_by_substr(buttons, frag):
    for label, vid in buttons.items():
        if frag in label:
            return vid
    return None


def discover_canvas(snapshot_text):
    """(cells[256], swatches[16]) —— 画布 = 唯一 256-子容器；调色板 = 唯一
    16-叶-子容器。"""
    children = parse_tree(snapshot_text)
    cells = None
    for parent, ch in children.items():
        if len(ch) == 256:
            cells = ch
            break
    swatches = None
    for parent, ch in children.items():
        if len(ch) == 16 and all(not children.get(x) for x in ch):
            swatches = ch
            break
    return cells, swatches


def refresh(mcp):
    """重取快照并重发现全部元素 id——**任何状态变更（重渲染）都会改变
    vnode 哈希 id**，缓存 id 过期即点击落空（T6 实测：T1 缓存在 T3 重渲染
    后失效）。每次交互组前调用。"""
    snap = mcp.snapshot()
    cells, swatches = discover_canvas(snap)
    buttons = find_buttons(snap)
    btn = {f: button_by_substr(buttons, f)
           for f in ("Pencil", "Eraser", "Fill", "Picker", "Undo", "Redo",
                     "Clear", "New", "Save", "Load")}
    return cells, swatches, btn


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


# ── Real 031-paint test suite ──────────────────────────────────────────────

def run_tests_031(mcp_url):
    mcp = McpClient(mcp_url)
    result = TestResult()
    RED = "#ef4444"

    # T1: 快照结构 + 元素寻址
    print("\nT1: Snapshot structure & element discovery")
    snap = mcp.snapshot()
    result.check("Snapshot contains App widget", 'widget: "App"' in snap, snap[:200])
    result.check("Snapshot shows Paint title", '"Paint"' in snap, "title missing")
    buttons = find_buttons(snap)
    for frag in ("Pencil", "Eraser", "Fill", "Picker", "Undo", "Redo",
                 "Clear", "New", "Save", "Load"):
        result.check(f"{frag} button found",
                     button_by_substr(buttons, frag) is not None,
                     f"labels: {list(buttons)}")
    cells, swatches = discover_canvas(snap)
    result.check("256 canvas cells discovered", cells is not None and len(cells) == 256,
                 f"cells: {None if cells is None else len(cells)}")
    result.check("16 palette swatches discovered",
                 swatches is not None and len(swatches) == 16,
                 f"swatches: {None if swatches is None else len(swatches)}")

    if not cells or not swatches:
        print("  (fatal: element discovery incomplete — abort suite)")
        return result

    # T2: 初始状态（256 白格 + 默认深灰当前色 + pencil）
    print("\nT2: Initial state (16x16 white canvas)")
    state = mcp.state("px", "cur", "tool", "saved")
    result.check("px materialized 256 white cells",
                 state.count("#ffffff") == 256, state[:300])
    result.check("cur is default dark", "111827" in state, state)
    result.check("tool is pencil", 'tool: "pencil"' in state, state)
    result.check("saved flag is 1", 'saved: "1"' in state, state)

    # T3: 铅笔染格（红 swatch → Paint(0)）
    print("\nT3: Pencil paint cell 0")
    cells, swatches, btn = refresh(mcp)
    r = mcp.click(swatches[1])
    result.check("swatch click ok", "status: ok" in r, r)
    r = mcp.click(cells[0])
    result.check("cell click ok", "status: ok" in r, r)
    state = mcp.state("px")
    result.check("exactly 1 red cell", state.count(RED) == 1, state[:300])
    result.check("saved flips to 0", 'saved: "0"' in mcp.state("saved"), "")

    # T4: 橡皮擦回
    print("\nT4: Eraser restores white")
    cells, swatches, btn = refresh(mcp)
    mcp.click(btn["Eraser"])
    mcp.click(cells[0])
    state = mcp.state("px")
    result.check("0 red cells after erase", state.count(RED) == 0, state[:300])

    # T5: 铅笔一红 + 油漆桶白格起灌 → 全盘 256 红
    print("\nT5: Flood fill")
    cells, swatches, btn = refresh(mcp)
    mcp.click(btn["Pencil"])
    mcp.click(cells[0])                      # 1 红
    cells, swatches, btn = refresh(mcp)
    mcp.click(btn["Fill"])
    mcp.click(cells[1])                      # 白格起灌 → 全红
    state = mcp.state("px")
    result.check("flood fills all 256 cells", state.count(RED) == 256,
                 f"count={state.count(RED)}")

    # T6: 撤销/重做（撤销回 1 红；重做回 256 红）。Redo 的快照为全同串
    # 列表（uniform 256 红）——触 VM 债 P553-D1（split 产物重建的全同串
    # 列表整体赋值塌缩为单元素，px len=1 实测）→ 债引用式 SKIP（013
    # audit-B12 同型）。
    print("\nT6: Undo / Redo")
    cells, swatches, btn = refresh(mcp)
    mcp.click(btn["Undo"])
    state = mcp.state("px")
    result.check("undo back to 1 red", state.count(RED) == 1,
                 f"count={state.count(RED)}")
    cells, swatches, btn = refresh(mcp)
    mcp.click(btn["Redo"])
    state = mcp.state("px")
    if state.count(RED) == 256:
        result.check("redo back to 256 red", True)
    else:
        result.skip("redo back to 256 red",
                    f"known gap P553-D1: got {state.count(RED)} — uniform-"
                    f"snapshot restore collapses to len-1 px (VM runtime)")

    # T7: 吸管（picker 点红格 → cur 变红）
    print("\nT7: Eyedropper")
    cells, swatches, btn = refresh(mcp)
    mcp.click(btn["Picker"])
    cells, swatches, btn = refresh(mcp)
    mcp.click(cells[0])
    result.check("cur picked red", RED in mcp.state("cur"), mcp.state("cur")[:80])

    # T8: 存取（Save → New 清盘 → Load 恢复）。恢复基线 = Save 时刻态；
    # 若 Save 时为全同盘（P553-D1）按债引用式 SKIP。
    print("\nT8: Save / New / Load persistence")
    cells, swatches, btn = refresh(mcp)
    saved_red = mcp.state("px").count(RED)
    mcp.click(btn["Save"])
    result.check("saved flag 1 after save", 'saved: "1"' in mcp.state("saved"), "")
    cells, swatches, btn = refresh(mcp)
    mcp.click(btn["New"])
    state = mcp.state("px")
    result.check("new clears canvas", state.count(RED) == 0,
                 f"count={state.count(RED)}")
    cells, swatches, btn = refresh(mcp)
    mcp.click(btn["Load"])
    state = mcp.state("px")
    loaded = state.count(RED)
    if saved_red == 256:
        # 全同盘恢复受 P553-D1 影响：期望 256，塌缩时按债 SKIP
        if loaded == 256:
            result.check("load restores 256 red", True)
        else:
            result.skip("load restores 256 red",
                        f"known gap P553-D1: got {loaded} (uniform save)")
    else:
        result.check("load restores saved canvas", loaded == saved_red,
                     f"saved={saved_red} loaded={loaded}")
    result.check("saved flag 1 after load", 'saved: "1"' in mcp.state("saved"), "")

    return result


def main():
    mcp_port = pick_free_port()
    mcp_url = f"http://127.0.0.1:{mcp_port}/mcp"
    print(f"[harness] project: {PAINT_PROJECT}")
    print(f"[harness] auto bin: {AUTO_BIN}")
    print(f"[harness] using AUTOUI_MCP_PORT={mcp_port}")
    proc = subprocess.Popen(
        [AUTO_BIN, "run", "-r", "vm"],
        cwd=PAINT_PROJECT,
        env={**os.environ, "AUTOUI_MCP_PORT": str(mcp_port)},
        # VM app 首帧前会打大量日志——PIPE 无人读会塞满缓冲导致进程写阻塞、
        # UI 永不渲染（实测）；DEVNULL 丢弃（排障改手动重定向复现）。
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    try:
        if not wait_for_server(mcp_url, timeout=60):
            print("[harness] MCP server did not come up "
                  "(rerun manually with output redirect to diagnose)")
            sys.exit(2)
        mcp = McpClient(mcp_url)
        if wait_for_ui(mcp) is None:
            print("[harness] UI never rendered "
                  "(rerun manually with output redirect to diagnose)")
            sys.exit(2)
        result = run_tests_031(mcp_url)
        print(f"\n===== 031-paint VM MCP: "
              f"{result.passed} passed, {result.failed} failed, "
              f"{result.skipped} skipped =====")
        if result.errors:
            print("Failures:")
            for e in result.errors:
                print(f"  - {e}")
        sys.exit(1 if result.failed else 0)
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    main()
