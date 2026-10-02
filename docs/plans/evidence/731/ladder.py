#!/usr/bin/env python
"""PLAN-731 T-00/T-05: 键入+滚动两形阶梯谱（S5 分段口径）。

725 ladder.py 扩展（同负载同协议——examples/ui/041-auto-edit + MCP 驱动）：
- [P725-FRAME] 行新增 S5 三子段（s5_layout/s5_shaping/s5_draw——731 T-00
  根包装探针+编辑器整形打点）；残差口径退役——present 侧 wgpu flush 以
  total−segsum−s5sum 单列（gpu_residual，仅配对帧）。
- 滚动形：autoui_action scroll（__mcp_scroll → iced scroll_to 绝对偏移）
  递增步进——视口推进→新暴露行整形负载（024 滚动段同机理）。
- [P725-ARMS] 臂钻取聚合（ce_draw_block/ce_gutter/ce_fold_scan/
  ce_text_runs）按相位归账（滚动突发跨帧累积，flush 行归其突发末帧相位）。

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
    r"s5_layout_us=(\d+) s5_shaping_us=(\d+) s5_draw_us=(\d+) "
    r"builds=(\d+) dirty=(-?\d+)"
)
ARMS_RE = re.compile(r"\[P725-ARMS\] (.+)$")

SIZES = {
    # (档名, 目标字节, 行模板)——行形与 725/下游 stage_frame fixture 同源。
    "5kb": (5 * 1024, "line {i:05d} frame drive payload content\n"),
    "100kb": (100 * 1024, "line {i:05d} frame drive payload content\n"),
    "1mb": (1024 * 1024, "line {i:05d} frame drive payload content padding padding padding padding\n"),
}

SCROLL_STEPS = 25      # 每档滚动步数
SCROLL_STEP_PX = 900   # 每步视口推进（≈45 行——新暴露行整形负载）
SCROLL_INTERVAL = 0.2  # 步间隔（>024 实录 ~108ms 帧时——不排队堆积）


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

    def find_scrollable(self, textarea_eid: str) -> str | None:
        """定位包含编辑器的 Scrollable（autoui_find kind=scrollable，子树含
        textarea id 者优先，否则首个）。"""
        r = self.call("autoui_find", {"kind": "scrollable", "limit": 10})
        txt = r.get("content", [{}])[0].get("text", "")
        cands = re.findall(r"vnode_\w+", txt)
        if not cands:
            return None
        # 含 textarea 的子树优先（autoui_find 输出祖先链 Atom 子树）。
        lines = txt.splitlines()
        chosen = None
        for i, line in enumerate(lines):
            if textarea_eid.lstrip("#") in line and "vnode_" in line:
                m = re.search(r"vnode_\w+", line)
                if m:
                    chosen = m.group(0)
                    break
        return chosen or cands[0]

    def scroll_to(self, eid: str, y: float) -> None:
        self.call("autoui_action", {"element_id": eid, "action": "scroll",
                                    "value": y})

    def fixture_open(self, path: str) -> None:
        """写 auto_open_path + 触发 App.Tick（ConsumeOpen 装载链）。"""
        r = self.call("autoui_fixture", {
            "schema_version": 1,
            "state": {"auto_open_path": path},
            "trigger": {"widget": "App", "event": "Tick"},
        })
        txt = r.get("content", [{}])[0].get("text", "")
        # 装载校核放宽：fixture 响应文本可为空（装载成效由后续帧谱随尺寸
        # 缩放验证）；只在明确 error 时告警。
        if "error" in txt.lower():
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


def parse_frame_line(line):
    m = FRAME_RE.search(line)
    if not m:
        return None
    g = m.groups()
    return {
        "begin": int(g[0]), "present": int(g[1]),
        "s1_us": int(g[2]), "s2_us": int(g[3]),
        "s3a_us": int(g[4]), "s3b_us": int(g[5]), "s4_us": int(g[6]),
        "s5_layout_us": int(g[7]), "s5_shaping_us": int(g[8]),
        "s5_draw_us": int(g[9]),
        "builds": int(g[10]), "dirty": int(g[11]),
    }


def parse_arms_line(line):
    m = ARMS_RE.search(line)
    if not m:
        return None
    arms = {}
    for part in m.group(1).split():
        name, _, val = part.partition("=")
        us, _, n = val.partition("/")
        try:
            arms[name] = (int(us), int(n))
        except ValueError:
            pass
    return arms


def pct(sorted_vals, q):
    if not sorted_vals:
        return None
    idx = min(len(sorted_vals) - 1, int(len(sorted_vals) * q))
    return sorted_vals[idx]


def summarize(frames, arms, phase, size_name, label, phase_seconds=None):
    """相位谱：typed（dirty=1 且 s2>0）/ scroll 帧与孤儿 fall-through 帧。
    S5 三子段 + gpu 残差（配对帧）+ 臂聚合 + 泵率谱（T-04：distinct
    present/秒 + fall-through 孤儿计数——泵配对竞争下的实际呈现节奏）。"""
    rows = []
    if phase == "type":
        members = [f for f in frames if f["dirty"] == 1 and f["s2_us"] > 0]
    else:
        # 滚动相位：泵配对行（present>0）+ 孤儿（present=-1）都算——以
        # present>0 行计帧率面，孤儿行只入成本谱。
        members = [f for f in frames if f["present"] > 0 or f["s5_draw_us"] > 0]
    for f in members:
        total = (f["present"] - f["begin"]) if f["present"] > 0 else None
        seg = f["s1_us"] + f["s2_us"] + f["s3a_us"] + f["s3b_us"] + f["s4_us"]
        s5 = f["s5_layout_us"] + f["s5_shaping_us"] + f["s5_draw_us"]
        rows.append({
            "type": f"{phase}_frame", "label": label, "size": size_name,
            "present": f["present"],
            "total_ms": round(total / 1000.0, 2) if total is not None else None,
            "seg_sum_ms": round(seg / 1000.0, 2),
            "s5_ms": round(s5 / 1000.0, 2),
            "gpu_residual_ms": round((total * 1000 - seg - s5) / 1e6, 2)
                                if total is not None else None,
            "s1_ms": round(f["s1_us"] / 1000.0, 3),
            "s2_ms": round(f["s2_us"] / 1000.0, 3),
            "s3a_ms": round(f["s3a_us"] / 1000.0, 3),
            "s3b_ms": round(f["s3b_us"] / 1000.0, 3),
            "s4_ms": round(f["s4_us"] / 1000.0, 3),
            "s5_layout_ms": round(f["s5_layout_us"] / 1000.0, 3),
            "s5_shaping_ms": round(f["s5_shaping_us"] / 1000.0, 3),
            "s5_draw_ms": round(f["s5_draw_us"] / 1000.0, 3),
            "builds": f["builds"], "dirty": f["dirty"],
        })
    tot = sorted(r["total_ms"] for r in rows if r["total_ms"] is not None)
    s5s = sorted(r["s5_ms"] for r in rows)
    presents = sum(1 for f in frames if f["present"] > 0)
    orphans = sum(1 for f in frames if f["present"] == -1
                  and (f["s4_us"] > 0 or f["s5_draw_us"] > 0 or f["s3b_us"] > 0))
    summary = {
        "type": "summary", "phase": phase, "label": label, "size": size_name,
        "frames": len(members),
        "present_paired": sum(1 for r in rows if r["total_ms"] is not None),
        "total_p50_ms": pct(tot, 0.5), "total_p95_ms": pct(tot, 0.95),
        "s5_p50_ms": pct(s5s, 0.5), "s5_p95_ms": pct(s5s, 0.95),
        # T-04 泵率谱：distinct present 行数 / 相位墙钟秒（None=相位时长
        # 未记录——泵计数仍有效，率值缺省）。
        "pump_events": presents,
        "pump_per_sec": round(presents / phase_seconds, 2) if phase_seconds else None,
        "orphan_fallthrough_frames": orphans,
        "arms": {k: {"us": v[0], "n": v[1]} for k, v in arms.items()},
    }
    if members:
        n = len(members)
        for k in ("s1_ms", "s2_ms", "s3a_ms", "s3b_ms", "s4_ms",
                  "s5_layout_ms", "s5_shaping_ms", "s5_draw_ms"):
            summary[f"{k}_mean"] = round(
                sum(r[k] for r in rows) / n, 3)
    return rows, summary


def run_size(label, size_name, target_bytes, line_tpl, auto_bin, keys, out_path):
    fx = make_fixture(size_name, target_bytes, line_tpl)
    port = pick_free_port()
    ad = tempfile.mkdtemp(prefix=f"p731-ladder-{size_name}-")
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
        scrollable = mcp.find_scrollable(eid)
        print(f"[*] {size_name}: textarea={eid} scrollable={scrollable}")
        # 装载 fixture 文件（新 tab 激活）+ settle
        mcp.fixture_open(fx)
        time.sleep(4.0)
        def mark():
            with open(log_path, encoding="utf-8", errors="replace") as f:
                return sum(1 for _ in f)
        n0 = mark(); t0w = time.time()
        # ── 键入相位（60ms 节拍，下游协议） ──
        for _ in range(keys):
            try:
                mcp.type_text(eid, "x")
            except Exception as e:
                print(f"[-] type 派发败: {e}")
            time.sleep(0.06)
        time.sleep(2.0)
        n1 = mark(); t1w = time.time()
        # ── 滚动相位（scroll_to 递增步进） ──
        if scrollable:
            for i in range(1, SCROLL_STEPS + 1):
                try:
                    mcp.scroll_to(scrollable, i * SCROLL_STEP_PX)
                except Exception as e:
                    print(f"[-] scroll 派发败: {e}")
                time.sleep(SCROLL_INTERVAL)
            time.sleep(2.0)
        n2 = mark(); t2w = time.time()
        # ── 换行连发相位（022/024 下游协议形——行数增长/gutter 位宽/全窗
        #    reshape 嫌疑面的复现臂） ──
        for _ in range(keys):
            try:
                mcp.type_text(eid, "z\n")
            except Exception as e:
                print(f"[-] typenl 派发败: {e}")
            time.sleep(0.06)
        time.sleep(2.0)
        n3 = mark(); t3w = time.time()
        with open(log_path, encoding="utf-8", errors="replace") as f:
            lines = f.readlines()
        print(f"[*] {size_name}: log lines={len(lines)} type=[{n0}:{n1}] scroll=[{n1}:{n2}] typenl=[{n2}:{n3}]")
        phase_seconds = {"type": t1w - t0w, "scroll": t2w - t1w, "typenl": t3w - t2w}
        phases = {"type": lines[n0:n1], "scroll": lines[n1:n2],
                  "typenl": lines[n2:n3]}
        for phase, plines in phases.items():
            frames, arms_acc = [], {}
            for line in plines:
                f = parse_frame_line(line)
                if f:
                    frames.append(f)
                    continue
                a = parse_arms_line(line)
                if a:
                    for k, (us, n) in a.items():
                        prev = arms_acc.get(k, (0, 0))
                        arms_acc[k] = (prev[0] + us, prev[1] + n)
            rows, summary = summarize(frames, arms_acc, phase, size_name, label,
                                      phase_seconds.get(phase))
            with open(out_path, "a", encoding="utf-8") as f:
                for r in rows + [summary]:
                    f.write(json.dumps(r, ensure_ascii=False) + "\n")
            s5_means = ", ".join(
                "{}={}".format(k, summary.get(k + "_mean"))
                for k in ("s5_layout_ms", "s5_shaping_ms", "s5_draw_ms"))
            print(f"[+] {size_name}/{phase}: frames={summary['frames']} "
                  f"paired={summary['present_paired']} "
                  f"totalP50={summary['total_p50_ms']} P95={summary['total_p95_ms']} "
                  f"s5P50={summary['s5_p50_ms']} P95={summary['s5_p95_ms']} "
                  f"means=({s5_means}) "
                  f"arms={summary['arms']}")
        return True
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
                            "scroll_steps": SCROLL_STEPS,
                            "scroll_step_px": SCROLL_STEP_PX,
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
