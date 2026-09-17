#!/usr/bin/env python3
"""
PLAN-631 T-01: 027-file-manager 列表交互重建剖析驱动器。

启动 VM 轨 027（P631_PROFILE=1 → renderer 逐重建帧吐 [P631-PROFILE] 行），
经 autoui_* MCP 通道驱动 N 次列表行选中，收集重建剖析行并输出统计。

Usage:
    python docs/plans/evidence/631/profile_driver.py [--clicks 15] [--label debug]
Requires the profiled build: target/{debug|release}/auto.exe（AUTO_BIN 可覆盖）。
"""

import argparse
import json
import os
import re
import statistics
import subprocess
import sys
import tempfile
import time

try:
    import requests
except ImportError:
    print("Please install requests: pip install requests")
    sys.exit(1)

LANG_ROOT = os.path.normpath(os.path.join(os.path.dirname(__file__), "..", "..", "..", ".."))
PROJECT = os.path.join(LANG_ROOT, "examples", "ui", "027-file-manager")


def find_auto_bin(profile):
    if "AUTO_BIN" in os.environ and os.path.exists(os.environ["AUTO_BIN"]):
        return os.environ["AUTO_BIN"]
    cand = os.path.join(LANG_ROOT, "target", profile, "auto.exe")
    return cand


def pick_free_port(start=9531):
    import socket
    for port in range(start, start + 100):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            if s.connect_ex(("127.0.0.1", port)) != 0:
                return port
    raise RuntimeError(f"No free port in [{start}, {start+100})")


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


def wait_for_server(url, timeout=60):
    for _ in range(timeout):
        try:
            requests.post(url, json={"jsonrpc": "2.0", "method": "tools/list",
                                     "params": {}, "id": 1}, timeout=2)
            return True
        except (requests.ConnectionError, requests.Timeout):
            time.sleep(1)
    return False


PROFILE_RE = re.compile(
    r"\[P631-PROFILE\] rebuild builder_ms=([\d.]+) render_ms=([\d.]+) "
    r"style_parse_calls=(\d+) style_parse_ms=([\d.]+)"
)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--clicks", type=int, default=15)
    ap.add_argument("--profile", default="debug", choices=["debug", "release"])
    ap.add_argument("--label", default="")
    args = ap.parse_args()

    auto_bin = find_auto_bin(args.profile)
    if not os.path.exists(auto_bin):
        print(f"auto.exe not found at {auto_bin}")
        sys.exit(1)

    port = pick_free_port()
    storage = os.path.join(tempfile.gettempdir(), f"p631_profile_{port}.json")
    if os.path.exists(storage):
        os.remove(storage)
    log_path = os.path.join(tempfile.gettempdir(), f"p631_profile_{port}.log")
    env = {**os.environ,
           "AUTOUI_MCP_PORT": str(port),
           "AUTO_VM_STORAGE_FILE": storage,
           "P631_PROFILE": "1"}
    log_file = open(log_path, "w", encoding="utf-8")
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm"], cwd=PROJECT, env=env,
                            stdout=log_file, stderr=log_file)
    url = f"http://127.0.0.1:{port}/mcp"
    try:
        if not wait_for_server(url):
            print("MCP server did not start; log:")
            print(open(log_path, encoding="utf-8").read()[-2000:])
            sys.exit(1)
        mcp = McpClient(url)
        time.sleep(1.5)  # 首帧 + 启动期重建沉淀（不计入统计窗口）

        snap = mcp.snapshot()
        # 列表行按钮(缩放数据集的生成名);退回任意 button。
        row_ids = re.findall(r'button #(aura_\d+|vnode_\d+)[^"]*"((?:quarterly_report|meeting_minutes|budget_draft|asset_|archive_|script_)\d+)', snap)
        row_ids = [rid for rid, _ in row_ids]
        if len(row_ids) < 2:
            row_ids = re.findall(r'button #(aura_\d+|vnode_\d+)', snap)
        if len(row_ids) < 2:
            print("no clickable rows found in snapshot")
            sys.exit(1)
        # 取两个不同行交替选中（同行重选在 VM 层可能被去重）。
        a, b = row_ids[0], row_ids[1]
        for i in range(args.clicks):
            target = a if i % 2 == 0 else b
            mcp.press(target)
            time.sleep(0.25)
        time.sleep(1.0)  # 尾帧沉淀
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
        log_file.close()

    lines = PROFILE_RE.findall(open(log_path, encoding="utf-8", errors="replace").read())
    rows = [(float(b), float(r), int(c), float(pm)) for b, r, c, pm in lines]
    if not rows:
        print("no [P631-PROFILE] lines found")
        sys.exit(1)

    builder = [r[0] for r in rows]
    render = [r[1] for r in rows]
    calls = [r[2] for r in rows]
    parse_ms = [r[3] for r in rows]
    total = [r[0] + r[1] for r in rows]

    def stats(v):
        v = sorted(v)
        return {
            "n": len(v),
            "median": round(statistics.median(v), 2),
            "p90": round(v[int(len(v) * 0.9) - 1 if len(v) > 1 else 0], 2),
            "max": round(max(v), 2),
        }

    summary = {
        "label": args.label or args.profile,
        "profile": args.profile,
        "frames": len(rows),
        "builder_ms": stats(builder),
        "render_ms": stats(render),
        "rebuild_total_ms": stats(total),
        "style_parse_calls": stats(calls),
        "style_parse_ms": stats(parse_ms),
        "style_parse_ms_median_share_of_builder_pct": round(
            100 * statistics.median(parse_ms) / max(statistics.median(builder), 0.01), 1),
    }
    print(json.dumps(summary, indent=2, ensure_ascii=False))
    out = os.path.join(os.path.dirname(__file__), f"profile_{summary['label']}.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump(summary, f, indent=2, ensure_ascii=False)
    print(f"saved -> {out}")


if __name__ == "__main__":
    main()
