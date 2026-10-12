#!/usr/bin/env python
"""PLAN-725 T-00/T-05: 键入→present 阶梯谱（上游基准件）。

负载：examples/ui/041-auto-edit（现役编辑器例——menubar/toolbar/树/tab/
  code_editor/状态栏全 chrome，与下游帧档工作负载同形）。文件装载走
  MCP fixture 通道（AUTOUI_TEST_FIXTURES=1 写 auto_open_path 状态 + 触发
  App.Tick → ConsumeOpen 装载）——本例无 AUTO_OPEN_PATH 读者（那是下游
  auto-edit 仓 .at 侧 Env.get 臂），fixture 是零改例的等价驱动。
驱动：MCP autoui_type 单字符连发（60ms 节拍，下游 stage_frame 同协议）。
读回：stderr [P725-FRAME] 行（S1..S4 分段+孤儿帧——frame_segments 探针，
  AUTO_FRAME_BENCH 同门）。
谱面：文档尺寸阶梯 5KB/100KB/1MB × VM 轨；每档 30 键；产物 JSONL。
  dirty 帧（键入帧）取 total=present-begin 与分段占比；present=-1 孤儿行
  = 无泵呈现的 fall-through 帧成本（put-then-take 勘定面）。

用法：
  python ladder.py --label baseline --out ladder-baseline.jsonl
  python ladder.py --label after    --out ladder-after.jsonl
（--auto-bin 默认取本 worktree 构建；复审复跑=同命令同负载。）
"""

import argparse
import json
import os
import re
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))
APP_DIR = os.path.join(REPO, "examples", "ui", "041-auto-edit")
FIXTURES = os.path.join(HERE, "fixtures")
DEFAULT_BIN = os.path.join(REPO, "target", "debug", "auto.exe")

FRAME_RE = re.compile(
    r"\[P725-FRAME\] begin=(-?\d+) present=(-?\d+) s1_payload_us=(\d+) "
    r"s2_vm_us=(\d+) s3a_mcp_us=(\d+) s3b_build_us=(\d+) s4_element_us=(\d+) "
    r"builds=(\d+) dirty=(-?\d+)"
)

SIZES = {
    # (档名, 目标字节, 行模板)——行形与下游 stage_frame fixture 同源。
    "5kb": (5 * 1024, "line {i:05d} frame drive payload content\n"),
    "100kb": (100 * 1024, "line {i:05d} frame drive payload content\n"),
    "1mb": (1024 * 1024, "line {i:05d} frame drive payload content padding padding padding padding\n"),
}


def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class Mcp:
    def __init__(self, port: int):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self._id = 1

    def call(self, tool: str, args: dict, timeout: int = 30) -> dict:
        body = json.dumps({
            "jsonrpc": "2.0", "id": self._id, "method": "tools/call",
            "params": {"name": tool, "arguments": args},
        }).encode()
        self._id += 1
        req = urllib.request.Request(self.url, data=body,
                                     headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            out = json.loads(resp.read().decode())
        if "error" in out:
            raise RuntimeError(f"{tool}: {out['error']}")
        return out.get("result", {})

    def snapshot(self) -> str:
        r = self.call("autoui_snapshot", {})
        return r.get("content", [{}])[0].get("text", "")

    def type_text(self, eid: str, text: str) -> None:
        self.call("autoui_type", {"element_id": eid.lstrip("#"),
                                  "text": text, "clear_first": False})

    def fixture_open(self, path: str) -> None:
        """写 auto_open_path + 触发 App.Tick（ConsumeOpen 装载链）。"""
        r = self.call("autoui_fixture", {
            "schema_version": 1,
            "state": {"auto_open_path": path},
            "trigger": {"widget": "App", "event": "Tick"},
        })
        txt = r.get("content", [{}])[0].get("text", "")
        if "error" in txt.lower() or "applied" not in txt.lower():
            print(f"[-] fixture 装载异常: {txt[:200]}")


def make_fixture(name: str, target_bytes: int, line_tpl: str) -> str:
    os.makedirs(FIXTURES, exist_ok=True)
    path = os.path.join(FIXTURES, f"ladder-{name}.txt")
    if os.path.exists(path) and os.path.getsize(path) >= target_bytes * 9 // 10:
        return path
    with open(path, "w", encoding="utf-8", newline="") as f:
        i = 0
        written = 0
        while written < target_bytes:
            line = line_tpl.format(i=i)
            f.write(line)
            written += len(line.encode("utf-8"))
            i += 1
    return path


def parse_frames(log_path: str):
    frames = []
    with open(log_path, encoding="utf-8", errors="replace") as f:
        for line in f:
            m = FRAME_RE.search(line)
            if m:
                g = m.groups()
                frames.append({
                    "begin": int(g[0]), "present": int(g[1]),
                    "s1_us": int(g[2]), "s2_us": int(g[3]),
                    "s3a_us": int(g[4]), "s3b_us": int(g[5]),
                    "s4_us": int(g[6]),
                    "builds": int(g[7]), "dirty": int(g[8]),
                })
    return frames


def pct(sorted_vals, q):
    if not sorted_vals:
        return None
    idx = min(len(sorted_vals) - 1, int(len(sorted_vals) * q))
    return sorted_vals[idx]


def summarize(frames, keys, size_name, label):
    """dirty 键入帧谱（dirty=1 且 s2>0——真派发帧）+ 孤儿 fall-through 帧谱。
    present 配对（泵消费序 bounds 回路竞争）不稳定——total 仅在配对帧上
    报告，配对率入 summary（分段谱不受影响：全 dirty 帧计入）。"""
    typed = [f for f in frames if f["dirty"] == 1 and f["s2_us"] > 0]
    paired = [f for f in typed if f["present"] > 0]
    orphans = [f for f in frames if f["present"] == -1 and f["s4_us"] > 0]
    rows = []
    for f in typed:
        total = (f["present"] - f["begin"]) if f["present"] > 0 else None
        seg = f["s1_us"] + f["s2_us"] + f["s3a_us"] + f["s3b_us"] + f["s4_us"]
        rows.append({
            "type": "typed_frame", "label": label, "size": size_name,
            "total_ms": round(total / 1000.0, 2) if total is not None else None,
            "seg_sum_ms": round(seg / 1000.0, 2),
            "s5_residual_ms": round((total * 1000 - seg) / 1e6, 2)
                              if total is not None else None,
            "s1_ms": round(f["s1_us"] / 1000.0, 3),
            "s2_ms": round(f["s2_us"] / 1000.0, 3),
            "s3a_ms": round(f["s3a_us"] / 1000.0, 3),
            "s3b_ms": round(f["s3b_us"] / 1000.0, 3),
            "s4_ms": round(f["s4_us"] / 1000.0, 3),
            "builds": f["builds"],
        })
    for f in orphans:
        rows.append({
            "type": "orphan_fallthrough", "label": label, "size": size_name,
            "total_ms": None,
            "s3b_ms": round(f["s3b_us"] / 1000.0, 3),
            "s4_ms": round(f["s4_us"] / 1000.0, 3),
            "builds": f["builds"],
        })
    tot = sorted(r["total_ms"] for r in rows
                 if r["type"] == "typed_frame" and r["total_ms"] is not None)
    segsums = sorted(r["seg_sum_ms"] for r in rows if r["type"] == "typed_frame")
    summary = {
        "type": "summary", "label": label, "size": size_name,
        "typed_frames": len(typed), "keys_sent": keys,
        "present_paired": len(paired),
        "total_p50_ms": pct(tot, 0.5), "total_p95_ms": pct(tot, 0.95),
        "segsum_p50_ms": pct(segsums, 0.5), "segsum_p95_ms": pct(segsums, 0.95),
        "orphan_frames": len(orphans),
        "orphan_s4_p50_ms": pct(sorted(r["s4_ms"] for r in rows
                                       if r["type"] == "orphan_fallthrough"), 0.5),
    }
    if typed:
        n = len(typed)
        segs = {k: sum(r[k] for r in rows if r["type"] == "typed_frame") / n
                for k in ("s1_ms", "s2_ms", "s3a_ms", "s3b_ms", "s4_ms")}
        summary["seg_mean_ms"] = {k: round(v, 2) for k, v in segs.items()}
    return rows, summary


def run_size(label, size_name, target_bytes, line_tpl, auto_bin, keys, out_path):
    fx = make_fixture(size_name, target_bytes, line_tpl)
    port = pick_free_port()
    ad = tempfile.mkdtemp(prefix=f"p725-ladder-{size_name}-")
    env = dict(os.environ,
               AUTO_BENCH="1", AUTO_FRAME_BENCH="1",
               AUTOUI_MCP_PORT=str(port),
               AUTOUI_TEST_FIXTURES="1",
               AUTOUI_HOT_RELOAD="0",
               APPDATA=ad,
               AUTO_PROJECT_DIR=APP_DIR)
    log_path = os.path.join(ad, "app.log")
    print(f"[*] {size_name}: fixture={os.path.getsize(fx)}B port={port} log={log_path}")
    log_f = open(log_path, "w", encoding="utf-8")
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm"], cwd=APP_DIR, env=env,
                            stdout=log_f, stderr=subprocess.STDOUT)
    try:
        mcp = Mcp(port)
        eid = None
        for _ in range(30):
            try:
                snap = mcp.snapshot()
                m2 = re.search(r"textarea #(\w+)", snap)
                if m2:
                    eid = m2.group(1)
                    break
            except Exception:
                pass
            time.sleep(1.0)
        if not eid:
            print(f"[-] {size_name}: textarea 未定位")
            return None
        # 装载 fixture 文件（新 tab 激活）+ settle
        mcp.fixture_open(fx)
        time.sleep(4.0)
        # 稳态截断：丢弃装载余波，取标记行位
        n0 = 0
        with open(log_path, encoding="utf-8", errors="replace") as f:
            n0 = sum(1 for _ in f)
        # 键入连发（60ms 节拍，下游协议）
        for _ in range(keys):
            try:
                mcp.type_text(eid, "x")
            except Exception as e:
                print(f"[-] type 派发败: {e}")
            time.sleep(0.06)
        time.sleep(2.0)  # 尾帧落账
        with open(log_path, encoding="utf-8", errors="replace") as f:
            lines = f.readlines()
        print(f"[*] {size_name}: log lines={len(lines)} marker_n0={n0}")
        tail = lines[n0:]
        tail_frames = []
        for line in tail:
            m3 = FRAME_RE.search(line)
            if m3:
                g = m3.groups()
                tail_frames.append({
                    "begin": int(g[0]), "present": int(g[1]),
                    "s1_us": int(g[2]), "s2_us": int(g[3]),
                    "s3a_us": int(g[4]), "s3b_us": int(g[5]),
                    "s4_us": int(g[6]),
                    "builds": int(g[7]), "dirty": int(g[8]),
                })
        rows, summary = summarize(tail_frames, keys, size_name, label)
        with open(out_path, "a", encoding="utf-8") as f:
            for r in rows + [summary]:
                f.write(json.dumps(r, ensure_ascii=False) + "\n")
        print(f"[+] {size_name}: typed={summary['typed_frames']} "
              f"paired={summary['present_paired']} "
              f"totalP50={summary['total_p50_ms']} P95={summary['total_p95_ms']} "
              f"segsumP50={summary['segsum_p50_ms']} P95={summary['segsum_p95_ms']} "
              f"orphan={summary['orphan_frames']} "
              f"segs={summary.get('seg_mean_ms')}")
        return summary
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
        log_f.close()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--label", required=True, help="baseline | after")
    ap.add_argument("--out", required=True, help="输出 JSONL 路径")
    ap.add_argument("--auto-bin", default=DEFAULT_BIN)
    ap.add_argument("--keys", type=int, default=30)
    ap.add_argument("--sizes", default="5kb,100kb,1mb")
    args = ap.parse_args()

    with open(args.out, "w", encoding="utf-8") as f:
        f.write(json.dumps({"type": "meta", "label": args.label,
                            "auto_bin": args.auto_bin,
                            "keys": args.keys,
                            "app": "examples/ui/041-auto-edit"}) + "\n")
    for size in args.sizes.split(","):
        size = size.strip().lower()
        if size not in SIZES:
            print(f"[-] 未知档 {size}")
            continue
        tb, tpl = SIZES[size]
        run_size(args.label, size, tb, tpl, args.auto_bin, args.keys, args.out)
    print(f"[*] 谱落 {args.out}")


if __name__ == "__main__":
    main()
