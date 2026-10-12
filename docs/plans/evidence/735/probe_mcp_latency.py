#!/usr/bin/env python
"""PLAN-735 T-02: MCP 调用时延微探针（每调用墙钟——无 sleep 背靠背）。

定位 ~65-73ms/调用 的来源面：autoui_type vs autoui_snapshot vs
autoui_action(scroll) 三面各 N 次背靠背计时。
"""
import json
import os
import re
import socket
import statistics
import subprocess
import sys
import tempfile
import time
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))
APP_DIR = os.path.join(REPO, "examples", "ui", "041-auto-edit")
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
        dt = (time.perf_counter() - t0) * 1000
        if "error" in out:
            raise RuntimeError(f"{tool}: {out['error']}")
        return dt, out.get("result", {})


def main():
    port = pick_free_port()
    ad = tempfile.mkdtemp(prefix="p735-mcp-latency-")
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
                dt, r = mcp.call("autoui_snapshot", {})
                m2 = re.search(r"textarea #(\w+)", r.get("content", [{}])[0].get("text", ""))
                if m2:
                    eid = m2.group(1)
                    print(f"[i] textarea={eid} (snapshot dt={dt:.1f}ms)")
                    break
            except Exception:
                pass
            time.sleep(1.0)
        if not eid:
            print("[-] no textarea")
            return
        scrollable = None
        dt, r = mcp.call("autoui_find", {"kind": "scrollable", "limit": 10})
        txt = r.get("content", [{}])[0].get("text", "")
        cands = re.findall(r"vnode_\w+", txt)
        scrollable = cands[0] if cands else None

        for name, fn in [
            ("type x", lambda i: mcp.call("autoui_type", {"element_id": eid, "text": "x", "clear_first": False})),
            ("snapshot", lambda i: mcp.call("autoui_snapshot", {})),
            ("scroll", lambda i: mcp.call("autoui_action", {"element_id": scrollable, "action": "scroll", "value": i * 20})),
            ("heartbeat", lambda i: mcp.call("autoui_heartbeat", {})),
        ]:
            dts = []
            for i in range(15):
                try:
                    dt, _ = fn(i)
                    dts.append(dt)
                except Exception as e:
                    print(f"[-] {name} #{i}: {e}")
                time.sleep(0.02)
            if dts:
                print(f"[+] {name}: n={len(dts)} mean={statistics.mean(dts):.1f}ms "
                      f"p50={sorted(dts)[len(dts)//2]:.1f}ms min={min(dts):.1f} max={max(dts):.1f}")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
        log_f.close()
        print(f"[*] log={log_path}")


if __name__ == "__main__":
    main()
