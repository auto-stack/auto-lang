#!/usr/bin/env python
"""PLAN-735 T-02: 装载 1MB fixture 后的 type 驱动周期分解探针。

复现 ladder 键入臂条件（fixture 装载 + 60ms sleep），逐调用计时 +
逐次 sleep 实测——把 ~95ms 周期分解为 call/sleep/其他。
"""
import json
import os
import re
import socket
import subprocess
import tempfile
import time
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))
APP_DIR = os.path.join(REPO, "examples", "ui", "041-auto-edit")
FIXTURES = os.path.join(HERE, "fixtures")
DEFAULT_BIN = os.path.join(REPO, "target", "release", "auto.exe")


def pick_free_port():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class Mcp:
    def __init__(self, port):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self._id = 1

    def call(self, tool, args, timeout=30):
        body = json.dumps({"jsonrpc": "2.0", "id": self._id,
                           "method": "tools/call",
                           "params": {"name": tool, "arguments": args}}).encode()
        self._id += 1
        req = urllib.request.Request(self.url, data=body,
                                     headers={"Content-Type": "application/json"})
        t0 = time.perf_counter()
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            out = json.loads(resp.read().decode())
        return (time.perf_counter() - t0) * 1000, out.get("result", {})


def main():
    # 1MB fixture（同 ladder 模板）
    os.makedirs(FIXTURES, exist_ok=True)
    fx = os.path.join(FIXTURES, "ladder-1mb.txt")
    if not os.path.exists(fx) or os.path.getsize(fx) < 1024 * 1024 * 9 // 10:
        with open(fx, "w", encoding="utf-8", newline="") as f:
            i = written = 0
            tpl = "line {i:05d} frame drive payload content padding padding padding padding\n"
            while written < 1024 * 1024:
                line = tpl.format(i=i)
                f.write(line)
                written += len(line.encode())
                i += 1
    port = pick_free_port()
    ad = tempfile.mkdtemp(prefix="p735-type-period-")
    env = dict(os.environ, AUTO_BENCH="1", AUTO_FRAME_BENCH="1",
               AUTO_SCHED_DIAG="1", AUTOUI_MCP_PORT=str(port),
               AUTOUI_TEST_FIXTURES="1", AUTOUI_HOT_RELOAD="0",
               APPDATA=ad, AUTO_PROJECT_DIR=APP_DIR)
    log_path = os.path.join(ad, "app.log")
    log_f = open(log_path, "w", encoding="utf-8")
    proc = subprocess.Popen([DEFAULT_BIN, "run", "-r", "vm"], cwd=APP_DIR,
                            env=env, stdout=log_f, stderr=subprocess.STDOUT)
    try:
        mcp = Mcp(port)
        eid = None
        for _ in range(30):
            try:
                _, r = mcp.call("autoui_snapshot", {})
                m2 = re.search(r"textarea #(\w+)", r.get("content", [{}])[0].get("text", ""))
                if m2:
                    eid = m2.group(1)
                    break
            except Exception:
                pass
            time.sleep(1.0)
        _, r = mcp.call("autoui_find", {"kind": "scrollable", "limit": 10})
        sc = (re.findall(r"vnode_\w+", r.get("content", [{}])[0].get("text", "")) or [None])[0]
        mcp.call("autoui_fixture", {"schema_version": 1,
                                    "state": {"auto_open_path": fx},
                                    "trigger": {"widget": "App", "event": "Tick"}})
        time.sleep(4.0)
        print(f"[i] eid={eid} scrollable={sc} fixture={os.path.getsize(fx)}B")

        # 30 键 60ms 节拍（ladder 同协议）——逐调用计时 + sleep 实测
        rows = []
        for i in range(30):
            t0 = time.perf_counter()
            dt_call, _ = mcp.call("autoui_type", {"element_id": eid, "text": "x", "clear_first": False})
            t1 = time.perf_counter()
            time.sleep(0.06)
            t2 = time.perf_counter()
            rows.append((t0, dt_call, (t2 - t1) * 1000))
        base = rows[0][0]
        print("[i] call_ms sleep_ms period_ms")
        periods = []
        for j, (t0, c, s) in enumerate(rows):
            period = (t0 - base) if j == 0 else (rows[j][0] - rows[j-1][0])
            period = period * 1000
            periods.append(period)
            print(f"  {j:02d} call={c:6.2f} sleep={s:6.2f} period={period:7.1f}")
        ps = sorted(periods[1:])
        print(f"[+] period p50={ps[len(ps)//2]:.1f}ms mean={sum(ps)/len(ps):.1f}ms")
        print(f"[*] log={log_path}")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
        log_f.close()


if __name__ == "__main__":
    main()
