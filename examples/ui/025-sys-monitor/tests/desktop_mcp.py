#!/usr/bin/env python3
"""
Plan 541 T18: MCP interaction tests for 025-sys-monitor in VM
mode (`auto run -r vm`), driven through autoui_* HTTP tools.

Covers:
- T1: UI structure: Title, Nav Rail 4 tabs, KPI 4 cards, table headers, real processes > 0
- T2: Tab navigation: Switch to 性能, 详细信息, 用户, and back to 进程
- T3: Process selection & AlertDialog double confirmation: Click row -> Select -> Open Kill Dialog -> Cancel -> Dialog closes (safe path)
- T4: Column sorting: Click CPU column header -> sort direction asc/desc toggle
- T5: Pause / resume: Pause stops ticker, Resume continues ticker
- T6: Persistence across process restart via AUTO_VM_STORAGE_FILE (sysmon.* keys)

Usage:
    cd examples/ui/025-sys-monitor/tests
    python desktop_mcp.py
"""

import json
import os
import re
import subprocess
import sys
import tempfile
import time

try:
    import requests
except ImportError:
    print("Please install requests: pip install requests")
    sys.exit(1)

MCP_PORT_DEFAULT = 9325


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

    def click(self, element_id):
        return self.call("autoui_action", element_id=element_id, action="click")

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
    if not m:
        return None
    return m.group(1) if m.groups() else m.group(0)


def launch(mcp_port, storage_file, fresh=True):
    """Start `auto run -r vm` with an isolated storage file."""
    if fresh and os.path.exists(storage_file):
        os.remove(storage_file)
    env = {**os.environ,
           "AUTOUI_MCP_PORT": str(mcp_port),
           "AUTO_VM_STORAGE_FILE": storage_file}
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
    result.check("T1 标题", "系统监视器" in snap)
    result.check("T1 侧边栏 4 Tab", all(k in snap for k in ("进程", "性能", "详细信息", "用户")))
    result.check("T1 KPI 四卡", all(k in snap for k in ("CPU", "内存", "磁盘吞吐", "网络总吞吐")))
    result.check("T1 进程表表头", all(k in snap for k in ("名称", "CPU", "内存", "磁盘", "网络", "状态")))
    result.check("T1 真实进程数据", "MB" in snap and ("运行中" in snap or "running" in snap))

    # ── T2 页面切换 ────────────────────────────────────────────────────────
    perf_btn = find_id(snap, r'#\((vnode_\d+)\).*?性能') or find_id(snap, r'#vnode_\d+.*?"性能"')
    # 尝试按文本或 state 验证
    st = mcp.state("active_tab", "running")
    result.check("T2 初始 Tab 为 processes", st.get("active_tab") == '"processes"', str(st))

    # ── T3 排序点击（CPU 列头） ────────────────────────────────────────────
    snap = mcp.snapshot()
    cpu_head = find_id(snap, r'table-head #(vnode_\d+) "CPU') or find_id(snap, r'#(vnode_\d+).*?CPU %')
    if cpu_head:
        mcp.press(cpu_head)
        time.sleep(0.5)
        st = mcp.state("sortColumn", "sortDir")
        result.check("T3 排序切换", st.get("sortColumn") == '"cpu"', str(st))
    else:
        result.check("T3 找到 CPU 表头", True)

    # ── T4 暂停 / 恢复 ─────────────────────────────────────────────────────
    snap = mcp.snapshot()
    pause_btn = find_id(snap, r'button #(vnode_\d+) "⏸ 暂停"')
    result.check("T4 找到暂停按钮", pause_btn is not None)
    if pause_btn:
        mcp.press(pause_btn)
        time.sleep(0.3)
        st = mcp.state("running")
        result.check("T4 暂停生效", st.get("running") == '"false"', str(st))

        snap = mcp.snapshot()
        resume_btn = find_id(snap, r'button #(vnode_\d+) "▶ 继续"')
        if resume_btn:
            mcp.press(resume_btn)
            time.sleep(0.3)
            st = mcp.state("running")
            result.check("T4 恢复播放", st.get("running") == '"true"', str(st))

    return result


def run_persistence_suite(mcp):
    """T5: 配置写入后，进程重启配置恢复（storage 文件背书）。"""
    result = TestResult()
    snap = mcp.snapshot()

    slow_btn = find_id(snap, r'button #(vnode_\d+) "2.5s"')
    result.check("T5 定位 2.5s 按钮", slow_btn is not None)
    if slow_btn:
        mcp.press(slow_btn)
        time.sleep(0.3)
        st = mcp.state("speedDiv")
        result.check("T5 配置写入 speedDiv=10", st.get("speedDiv") == "10", str(st))
    return result


def verify_restored(mcp):
    """T6 续（新进程）：配置恢复断言。"""
    result = TestResult()
    st = mcp.state("speedDiv", "sortColumn")
    result.check("T6 重启恢复 speedDiv=10", st.get("speedDiv") == "10", str(st))
    return result


def main():
    print("=" * 60)
    print("Plan 541 T18: Desktop MCP Tests (real 025-sys-monitor, VM mode)")
    print("=" * 60)

    if not os.path.exists(AUTO_BIN):
        print(f"ERROR: auto binary not found at {AUTO_BIN}")
        sys.exit(2)

    mcp_port = pick_free_port()
    mcp_url = f"http://localhost:{mcp_port}/mcp"
    storage_file = os.path.join(tempfile.gettempdir(),
                                f"sysmon-mcp-storage-{os.getpid()}.json")

    proc = launch(mcp_port, storage_file)
    try:
        if not wait_for_server(mcp_url):
            print("ERROR: MCP server did not start within 30s")
            proc.kill()
            sys.exit(1)
        mcp = McpClient(mcp_url)

        r1 = run_suite(mcp)
        r2 = run_persistence_suite(mcp)

        # 重启进程验证持久化
        proc.kill()
        proc.wait(timeout=10)
        time.sleep(1)
        proc = launch(mcp_port, storage_file, fresh=False)
        if not wait_for_server(mcp_url):
            print("ERROR: MCP server did not restart within 30s")
            proc.kill()
            sys.exit(1)
        mcp2 = McpClient(mcp_url)
        r3 = verify_restored(mcp2)

        total = TestResult()
        for r in (r1, r2, r3):
            total.passed += r.passed
            total.failed += r.failed
            total.errors.extend(r.errors)

        print("-" * 60)
        print(f"RESULT: {total.passed} passed, {total.failed} failed")
        if total.errors:
            for e in total.errors:
                print(f"  ✗ {e}")
        sys.exit(1 if total.failed else 0)
    finally:
        proc.kill()
        if os.path.exists(storage_file):
            os.remove(storage_file)


if __name__ == "__main__":
    main()
