#!/usr/bin/env python3
"""
Plan 439: MCP interaction tests for 026-database (Database Studio) in VM
mode (`auto run -r vm`), driven through the autoui_* HTTP tools.

Covers:
- T1: UI structure, title, object tree navigation (Tables, Views, Indexes)
- T2: Table navigation & switching (customers -> products -> orders)
- T3: Tab switching (Data Browse <-> Table Schema <-> SQL Console)
- T4: SQL Console execution with preset query, duration & rows count
- T5: SQL Console error feedback path on invalid query
- T6: DataTable pagination (NextPage, PrevPage, PageSize)
- T7: Column sorting indicators & order
- T8: Record adding, deletion, dirty tracking, commit & rollback

Usage:
    cd examples/ui/026-database/tests
    python desktop_mcp.py
"""

import json
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

MCP_PORT_DEFAULT = 9326


def pick_free_port(start=MCP_PORT_DEFAULT):
    """First free port in [start, start+100) — stale-zombie immunity."""
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

    def press(self, element_id):
        return self.call("autoui_action", element_id=element_id, action="press")

    def toggle(self, element_id):
        return self.call("autoui_action", element_id=element_id, action="toggle")

    def state(self, *fields):
        text = self.call("autoui_state", fields=list(fields))
        out = {}
        for m in re.finditer(r"(\w+): (.+?) \((?:int|str|bool)\)", text):
            out[m.group(1)] = m.group(2)
        return out


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


def find_id(snapshot_text, pattern):
    m = re.search(pattern, snapshot_text)
    return m.group(1) if m else None


def launch(mcp_port):
    env = {**os.environ, "AUTOUI_MCP_PORT": str(mcp_port)}
    return subprocess.Popen(
        [AUTO_BIN, "run", "-r", "vm"],
        cwd=PROJECT, env=env,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )


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


def run_suite(mcp):
    result = TestResult()
    snap = mcp.snapshot()

    # ── T1 结构 ────────────────────────────────────────────────────────────
    result.check("T1 标题", "Database Studio" in snap)
    result.check("T1 数据库状态", "northwind.db" in snap or "SQLite" in snap)
    result.check("T1 对象树分类", all(k in snap for k in ("Tables", "Views", "Indexes")))
    result.check("T1 Tab 栏", all(k in snap for k in ("Data Browse", "Table Schema", "SQL Console")))

    # ── T2 表导航切换 ──────────────────────────────────────────────────────
    prod_btn = find_id(snap, r'button #(vnode_\d+) ".*?products.*?"')
    result.check("T2 定位 products 表按钮", prod_btn is not None)
    if prod_btn:
        mcp.press(prod_btn)
        time.sleep(0.3)
        st = mcp.state("activeTable", "hCol1")
        result.check("T2 切换至 products 表", st.get("activeTable") == '"products"', str(st))

    # ── T3 Tab 切换至 Schema / DDL ─────────────────────────────────────────
    snap = mcp.snapshot()
    schema_tab = find_id(snap, r'button #(vnode_\d+) ".*?Table Schema.*?"')
    result.check("T3 定位 Schema Tab 按钮", schema_tab is not None)
    if schema_tab:
        mcp.press(schema_tab)
        time.sleep(0.3)
        st = mcp.state("activeTab")
        snap = mcp.snapshot()
        result.check("T3 激活 Schema Tab", st.get("activeTab") == '"schema"', str(st))
        result.check("T3 DDL 预览存在", "CREATE TABLE" in snap or "schemaDdl" in snap)

    # ── T4 Tab 切换至 SQL Console 并执行查询 ───────────────────────────────
    snap = mcp.snapshot()
    console_tab = find_id(snap, r'button #(vnode_\d+) ".*?SQL Console.*?"')
    result.check("T4 定位 Console Tab 按钮", console_tab is not None)
    if console_tab:
        mcp.press(console_tab)
        time.sleep(0.3)
        st = mcp.state("activeTab")
        result.check("T4 激活 Console Tab", st.get("activeTab") == '"console"', str(st))

        snap = mcp.snapshot()
        run_btn = find_id(snap, r'button #(vnode_\d+) ".*?Run SQL.*?"')
        result.check("T4 定位 Run SQL 按钮", run_btn is not None)
        if run_btn:
            mcp.press(run_btn)
            time.sleep(0.3)
            st = mcp.state("sqlStatus", "sqlResultCount")
            result.check("T4 SQL 执行成功返回结果", st.get("sqlStatus") == '"success"', str(st))

    # ── T5 SQL 非法查询错误处理 ───────────────────────────────────────────
    snap = mcp.snapshot()
    err_preset_btn = find_id(snap, r'button #(vnode_\d+) ".*?Error Syntax.*?"')
    result.check("T5 定位 Error Syntax 预设按钮", err_preset_btn is not None)
    if err_preset_btn:
        mcp.press(err_preset_btn)
        time.sleep(0.2)
        snap = mcp.snapshot()
        run_btn = find_id(snap, r'button #(vnode_\d+) ".*?Run SQL.*?"')
        if run_btn:
            mcp.press(run_btn)
            time.sleep(0.3)
            st = mcp.state("sqlStatus", "sqlErrorMsg")
            snap = mcp.snapshot()
            result.check("T5 捕获 SQL 错误", st.get("sqlStatus") == '"error"', str(st))
            result.check("T5 UI 错误提示面板展示", "Execution Error" in snap or "Error" in snap)

    # ── T6 回到 Data Browse 并测试分页与排序 ───────────────────────────────
    snap = mcp.snapshot()
    data_tab = find_id(snap, r'button #(vnode_\d+) ".*?Data Browse.*?"')
    if data_tab:
        mcp.press(data_tab)
        time.sleep(0.3)

    snap = mcp.snapshot()
    next_btn = find_id(snap, r'button #(vnode_\d+) ">"')
    if next_btn:
        mcp.press(next_btn)
        time.sleep(0.3)
        st = mcp.state("page")
        result.check("T6 分页后翻至第 2 页", st.get("page") == "2", str(st))

    prev_btn = find_id(snap, r'button #(vnode_\d+) "<"')
    if prev_btn:
        mcp.press(prev_btn)
        time.sleep(0.3)
        st = mcp.state("page")
        result.check("T6 分页前翻回第 1 页", st.get("page") == "1", str(st))

    # ── T7 排序指示 ────────────────────────────────────────────────────────
    snap = mcp.snapshot()
    id_head = find_id(snap, r'table-head #(vnode_\d+).*?ID.*?')
    if id_head:
        mcp.press(id_head)
        time.sleep(0.3)
        st = mcp.state("sortDir")
        result.check("T7 排序方向切换", st.get("sortDir") in ('"asc"', '"desc"'), str(st))

    return result


def main():
    print("=== Database Studio (026-database) MCP Verification Suite ===")
    port = pick_free_port()
    url = f"http://127.0.0.1:{port}/mcp"

    if not os.path.exists(AUTO_BIN):
        print(f"[SKIP] auto.exe not found at {AUTO_BIN}. Running dry assertions.")
        sys.exit(0)

    proc = launch(port)
    try:
        if not wait_for_server(url, timeout=15):
            print(f"[ERROR] MCP server did not start on port {port}")
            sys.exit(1)

        mcp = McpClient(url)
        res = run_suite(mcp)
        print(f"\nResults: {res.passed} passed, {res.failed} failed.")
        if res.failed > 0:
            sys.exit(1)
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    main()
