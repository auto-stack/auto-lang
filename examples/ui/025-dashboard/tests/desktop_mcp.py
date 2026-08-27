#!/usr/bin/env python3
"""
Plan 438 M2: MCP interaction tests for the REAL 025-dashboard app in VM
mode (`auto run -r vm`), driven through the autoui_* HTTP tools.

Covers the M1 browser assertions on the desktop track (KPI ticks, sort
flip, chart toggles, pause/resume) plus the M2 additions (process table
rendering, config persistence across a process restart).

Storage isolation: AUTO_VM_STORAGE_FILE points at a throwaway file
(deleted before each launch) so persistence assertions are deterministic
and no state leaks between runs. Without it the VM storage shim is
per-project under the temp dir (cwd-hashed) — see SPEC.md.

Usage:
    cd examples/ui/025-dashboard/tests
    python desktop_mcp.py

Prerequisites: auto built with ui-iced (or AUTO_BIN), python requests.
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

MCP_PORT_DEFAULT = 9317


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

PROC_NAMES = {"auto-edit", "chrome", "rust-analyzer", "node", "ash-server",
              "musk-gui", "mcp-hub", "code-helper"}


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


def launch(mcp_port, storage_file, fresh=True):
    """Start `auto run -r vm` with an isolated storage file.

    fresh=True clears the file first (deterministic initial state);
    fresh=False keeps it — the persistence round-trip relies on this."""
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
    result.check("T1 KPI 四卡", all(k in snap for k in ("CPU", "内存", "网络", "进程")))
    result.check("T1 三曲线卡", all(k in snap for k in ("CPU 使用率", "内存占用", "网络吞吐")))
    result.check("T1 三 checkbox", snap.count("checkbox") == 3,
                 f"checkbox={snap.count('checkbox')}")
    result.check("T1 svg 三图", snap.count("[Image]") == 3,
                 f"images={snap.count('[Image]')}")
    result.check("T1 表头四列", all(k in snap for k in ("名称", "CPU % ↓", "内存", "状态")))
    names = [n for n in re.findall(r'text #vnode_\d+ "([a-z-]+)"', snap)
             if n in PROC_NAMES]
    result.check("T1 进程 8 行", len(names) == 8, f"rows={len(names)}")

    # ── T2 KPI 随 Tick 变化 ────────────────────────────────────────────────
    s1 = mcp.state("cpu", "netT", "procCount")
    time.sleep(2.6)  # 有效刷新 1s（默认档）
    s2 = mcp.state("cpu", "netT", "procCount")
    result.check("T2 KPI 随 Tick 变化",
                 s1.get("cpu") != s2.get("cpu") or s1.get("netT") != s2.get("netT"),
                 f"{s1} -> {s2}")

    # ── T3 排序点击（CPU 列头） ────────────────────────────────────────────
    snap = mcp.snapshot()
    cpu_btn = find_id(snap, r'button #(vnode_\d+) "CPU % ↓"')
    result.check("T3 找到 CPU 列头按钮", cpu_btn is not None)
    if cpu_btn:
        mcp.press(cpu_btn)
        time.sleep(0.5)
        st = mcp.state("sortColumn", "sortDir", "cpuH")
        snap = mcp.snapshot()
        names = [n for n in re.findall(r'text #vnode_\d+ "([a-z-]+)"', snap)
                 if n in PROC_NAMES]
        result.check("T3 排序翻转 asc", st.get("sortDir") == '"asc"', str(st))
        result.check("T3 列头 ↑", "CPU % ↑" in snap)
        result.check("T3 行序重排", len(names) == 8 and names != sorted(names, reverse=True),
                     str(names))

    # ── T4 曲线开关（独立隐藏） ────────────────────────────────────────────
    snap = mcp.snapshot()
    first_cb = find_id(snap, r"checkbox #(vnode_\d+)")
    result.check("T4 找到 checkbox", first_cb is not None)
    if first_cb:
        mcp.toggle(first_cb)
        time.sleep(0.5)
        st = mcp.state("showCpu", "showMem", "showNet")
        snap = mcp.snapshot()
        hidden = [k for k, v in (("CPU 使用率", "showCpu"),
                                 ("内存占用", "showMem"), ("网络吞吐", "showNet"))
                  if k not in snap]
        result.check("T4 独立隐藏一张卡",
                     len(hidden) == 1 and st.get(hidden[0] == "CPU 使用率" and "showCpu"
                                                 or {"内存占用": "showMem",
                                                     "网络吞吐": "showNet"}[hidden[0]]) == "false",
                     f"hidden={hidden} state={st}")
        result.check("T4 其余卡存活", snap.count("checkbox") == 2)
        # 恢复（为 T6 持久化断言保持已知态）
        snap = mcp.snapshot()
        first_cb = find_id(snap, r"checkbox #(vnode_\d+)")
        if first_cb:
            mcp.toggle(first_cb)

    # ── T5 暂停 / 恢复 ─────────────────────────────────────────────────────
    snap = mcp.snapshot()
    pause_btn = find_id(snap, r'button #(vnode_\d+) "⏸ 暂停"')
    result.check("T5 找到暂停按钮", pause_btn is not None)
    if pause_btn:
        mcp.press(pause_btn)
        time.sleep(0.3)
        st = mcp.state("running", "cpu")
        result.check("T5 暂停生效", st.get("running") == '"false"', str(st))
        a = mcp.state("cpu")
        time.sleep(2.2)
        b = mcp.state("cpu")
        result.check("T5 冻结", a.get("cpu") == b.get("cpu"), f"{a} -> {b}")
        snap = mcp.snapshot()
        resume_btn = find_id(snap, r'button #(vnode_\d+) "▶ 继续"')
        if resume_btn:
            mcp.press(resume_btn)
            time.sleep(0.3)
            st = mcp.state("running")
            result.check("T5 恢复播放", st.get("running") == '"true"', str(st))

    return result


def run_persistence_suite(mcp):
    """T6: 配置写入后，进程重启配置恢复（storage 文件背书）。"""
    result = TestResult()
    snap = mcp.snapshot()

    slow_btn = find_id(snap, r'button #(vnode_\d+) "2.5s"')
    mem_btn = find_id(snap, r'button #(vnode_\d+) "内存"')
    first_cb = find_id(snap, r"checkbox #(vnode_\d+)")
    result.check("T6 定位控件", all(x for x in (slow_btn, mem_btn, first_cb)))
    if not all(x for x in (slow_btn, mem_btn, first_cb)):
        return result

    mcp.press(slow_btn)
    mcp.toggle(first_cb)
    time.sleep(0.3)
    mcp.press(mem_btn)
    time.sleep(0.5)
    st = mcp.state("speedDiv", "sortColumn", "showCpu")
    result.check("T6 配置写入", st.get("speedDiv") == "10"
                 and st.get("sortColumn") == '"mem"'
                 and st.get("showCpu") == "false", str(st))
    return result


def verify_restored(mcp):
    """T6 续（新进程）：配置恢复断言。"""
    result = TestResult()
    st = mcp.state("speedDiv", "sortColumn", "sortDir", "showCpu", "showMem")
    result.check("T6 重启恢复 speedDiv=10", st.get("speedDiv") == "10", str(st))
    result.check("T6 重启恢复 sort=mem desc",
                 st.get("sortColumn") == '"mem"' and st.get("sortDir") == '"desc"', str(st))
    result.check("T6 重启恢复 showCpu=false", st.get("showCpu") == "false", str(st))
    snap = mcp.snapshot()
    result.check("T6 CPU 卡保持隐藏", "CPU 使用率" not in snap)
    result.check("T6 内存列头 ↓", "内存 ↓" in snap)
    return result


def main():
    print("=" * 60)
    print("Plan 438 M2: Desktop MCP Tests (real 025-dashboard, VM mode)")
    print("=" * 60)

    if not os.path.exists(AUTO_BIN):
        print(f"ERROR: auto binary not found at {AUTO_BIN}")
        sys.exit(2)

    mcp_port = pick_free_port()
    mcp_url = f"http://localhost:{mcp_port}/mcp"
    storage_file = os.path.join(tempfile.gettempdir(),
                                f"dash-mcp-storage-{os.getpid()}.json")

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
        r3 = verify_restored(mcp)

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
