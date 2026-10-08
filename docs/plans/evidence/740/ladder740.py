#!/usr/bin/env python
"""PLAN-740 T-00: 交付节奏三轴谱 + 驱动率阶梯跟随性探针。

ladder735.py 同负载协议扩展（041-auto-edit VM 轨）：
- AUTO_SCHED_DIAG 扩轴解析（740 新轴）：`mcp_poll`（16ms tokio tick 真实
  触发节奏 gap + got 消费标记——action 消费轴）与 `update_end`（update 轮
  完成/request_redraw 发出代理轴）。既有轴（update 入口/redraw_deliver
  到达/frame_msg enqueue/[P725-FRAME] draw_end_ms 呈现真值）照旧。
- 驱动率阶梯（--rungs）：041 例上按目标 Hz 档驱动 type/scroll 相位。
  驱动侧逐调用计时（时延自适应 sleep=period-latency，有效驱动率实测），
  交付侧取 distinct draw_end 交付率；脚本钟→app 对数钟线性映射对齐
  （窗口=首调用起至末调用+300ms 宽限），产出跟随性谱
  （AC-02 判据材料：交付率==有效驱动率 ±10% 至 ≥54Hz 档）。
- 驱动机制勘定（T-00 内建）：单发 call（autoui_type 单字符 /
  autoui_action scroll）——735 probe_type_period 实测单发时延 ~2-20ms
  （62.5ms 为含 60ms sleep 的周期），sleep→0 单发驱动理论可达 ≥54Hz。
  若某档实测有效驱动率显著低于目标档（时延地板），谱面如实记录实际
  达到率（不冒领），作为 batch-drive 内部通道候选的勘定依据。

用法：
  python ladder740.py --label p740-pre --out ladder-p740-pre.jsonl
    （735 协议复刻：三档 size × type/scroll/typenl，60ms 节拍——零回退对照）
  python ladder740.py --label p740-pre --out ladder-p740-pre.jsonl --rungs "10,16,34,54"
    （阶梯跟随性谱：5kb 单档 × 各 rung type/scroll 相位）
"""

import argparse
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

FRAME_RE = re.compile(
    r"\[P725-FRAME\] begin=(-?\d+) present=(-?\d+) s1_payload_us=(\d+) "
    r"s2_vm_us=(\d+) s3a_mcp_us=(\d+) s3b_build_us=(\d+) s4_element_us=(\d+) "
    r"s5_layout_us=(\d+) s5_shaping_us=(\d+) s5_draw_us=(\d+) "
    r"builds=(\d+) dirty=(-?\d+)(?: draw_end_ms=(-?\d+))?"
)
SD_REDRAW_DELIVER_RE = re.compile(r"\[SCHED-DIAG\] redraw_deliver t=(\d+)ms")
SD_UPDATE_RE = re.compile(
    r"\[SCHED-DIAG\] update t=(\d+)ms app=\S+ widget=\S* event=(\S*)")
SD_UPDATE_END_RE = re.compile(r"\[SCHED-DIAG\] update_end t=(\d+)ms")
# PLAN-740 新轴：16ms tick 真实触发节奏 + 消费标记。
SD_MCP_POLL_RE = re.compile(
    r"\[SCHED-DIAG\] mcp_poll t=(\d+)ms gap=(-?\d+)ms got=(\d)")
ARMS_RE = re.compile(r"\[P725-ARMS\] (.+)$")
SD_ENQUEUE_RE = re.compile(r"\[SCHED-DIAG\] frame_msg enqueue t=(\d+)ms")
SD_ARM_RE = re.compile(r"\[SCHED-DIAG\] arm_frame t=(\d+)ms")
SD_PUMP_ENTER_RE = re.compile(
    r"\[SCHED-DIAG\] frame_pump enter t=(\d+)ms queued_init=(-?\d+) "
    r"cpu_cont=(-?\d+) parked_total=(-?\d+) write_q=(-?\d+)"
)
SD_SUB_FRAME_RE = re.compile(r"\[SCHED-DIAG\] sub_frame t=(\d+)ms")
SD_SUB_PARKED_RE = re.compile(r"\[SCHED-DIAG\] sub_parked t=(\d+)ms")

SIZES = {
    "5kb": (5 * 1024, "line {i:05d} frame drive payload content\n"),
    "100kb": (100 * 1024, "line {i:05d} frame drive payload content\n"),
    "1mb": (1024 * 1024, "line {i:05d} frame drive payload content padding padding padding padding\n"),
}

SCROLL_STEPS = 25
SCROLL_STEP_PX = 900
SCROLL_INTERVAL = 0.2
# 阶梯 rung 相位宽度：目标 ~3s 驱动窗（+2s settle 同 735 协议）。
RUNG_PHASE_S = 3.0
RUNG_SETTLE_S = 2.0
RUNG_GRACE_S = 0.3


def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class Mcp:
    def __init__(self, port: int):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self._id = 1

    def call(self, tool: str, args: dict, timeout: int = 30):
        """返回 (result, t_start_perf, latency_s)——驱动侧逐调用计时。"""
        body = json.dumps({
            "jsonrpc": "2.0", "id": self._id, "method": "tools/call",
            "params": {"name": tool, "arguments": args},
        }).encode()
        self._id += 1
        req = urllib.request.Request(self.url, data=body,
                                     headers={"Content-Type": "application/json"})
        t0 = time.perf_counter()
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            out = json.loads(resp.read().decode())
        lat = time.perf_counter() - t0
        if "error" in out:
            raise RuntimeError(f"{tool}: {out['error']}")
        return out.get("result", {}), t0, lat

    def snapshot(self) -> str:
        r, _, _ = self.call("autoui_snapshot", {})
        return r.get("content", [{}])[0].get("text", "")

    def type_text(self, eid: str, text: str):
        return self.call("autoui_type", {"element_id": eid.lstrip("#"),
                                         "text": text, "clear_first": False})

    def find_scrollable(self, textarea_eid: str) -> str | None:
        r, _, _ = self.call("autoui_find", {"kind": "scrollable", "limit": 10})
        txt = r.get("content", [{}])[0].get("text", "")
        cands = re.findall(r"vnode_\w+", txt)
        if not cands:
            return None
        lines = txt.splitlines()
        chosen = None
        for i, line in enumerate(lines):
            if textarea_eid.lstrip("#") in line and "vnode_" in line:
                m = re.search(r"vnode_\w+", line)
                if m:
                    chosen = m.group(0)
                    break
        return chosen or cands[0]

    def scroll_to(self, eid: str, y: float):
        return self.call("autoui_action", {"element_id": eid, "action": "scroll",
                                           "value": y})

    def fixture_open(self, path: str) -> None:
        r, _, _ = self.call("autoui_fixture", {
            "schema_version": 1,
            "state": {"auto_open_path": path},
            "trigger": {"widget": "App", "event": "Tick"},
        })
        txt = r.get("content", [{}])[0].get("text", "")
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
    if m:
        g = m.groups()
        return {
            "begin": int(g[0]), "present": int(g[1]),
            "s1_us": int(g[2]), "s2_us": int(g[3]),
            "s3a_us": int(g[4]), "s3b_us": int(g[5]), "s4_us": int(g[6]),
            "s5_layout_us": int(g[7]), "s5_shaping_us": int(g[8]),
            "s5_draw_us": int(g[9]),
            "builds": int(g[10]), "dirty": int(g[11]),
            "draw_end_ms": int(g[12]) if g[12] is not None else -1,
        }
    return None


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


def parse_sched_diag(plines):
    """相位内 SCHED-DIAG 汇总（PLAN-740 扩轴：mcp_poll gap 谱 + update_end）。"""
    enqueues, arms_cnt, pump_enters = [], 0, []
    sub_frame_events, sub_parked_events = 0, 0
    delivers, updates, update_ends = [], [], []
    polls, polls_got, got_pairs = [], [], []
    for line in plines:
        m = SD_ENQUEUE_RE.search(line)
        if m:
            enqueues.append(int(m.group(1)))
            continue
        m = SD_REDRAW_DELIVER_RE.search(line)
        if m:
            delivers.append(int(m.group(1)))
            continue
        m = SD_UPDATE_RE.search(line)
        if m:
            updates.append({"t": int(m.group(1)), "event": m.group(2)})
            continue
        m = SD_UPDATE_END_RE.search(line)
        if m:
            update_ends.append(int(m.group(1)))
            continue
        m = SD_MCP_POLL_RE.search(line)
        if m:
            gap, got = int(m.group(2)), int(m.group(3))
            polls.append(gap)
            got_pairs.append((gap, got))
            if got:
                polls_got.append(int(m.group(1)))
            continue
        m = SD_ARM_RE.search(line)
        if m:
            arms_cnt += 1
            continue
        m = SD_PUMP_ENTER_RE.search(line)
        if m:
            pump_enters.append({
                "t": int(m.group(1)), "queued_init": int(m.group(2)),
                "cpu_cont": int(m.group(3)), "parked_total": int(m.group(4)),
                "write_q": int(m.group(5)),
            })
            continue
        if SD_SUB_FRAME_RE.search(line):
            sub_frame_events += 1
        if SD_SUB_PARKED_RE.search(line):
            sub_parked_events += 1
    gaps_sorted = sorted(g for g in polls if g >= 0)
    got_gaps = sorted(g for (g, got) in got_pairs if g >= 0 and got)
    summary = {
        "type": "sched_diag",
        "enqueue_events": len(enqueues),
        "redraw_deliver_events": len(delivers),
        "update_events": len(updates),
        "update_end_events": len(update_ends),
        "redraw_deliver_ts": delivers,
        "update_end_ts": update_ends,
        "update_ts_events": updates,
        "arm_frame_events": arms_cnt,
        "pump_enter_events": len(pump_enters),
        "sub_frame_assemblies": sub_frame_events,
        "sub_parked_assemblies": sub_parked_events,
        "counter_samples": pump_enters[:20],
        # PLAN-740 轴谱：tick 真实触发节奏（全部 tick 的 gap 分布）与
        # 消费拍（got=1 的触发时刻序列）。
        "mcp_poll_ticks": len(polls),
        "mcp_poll_gap_p50_ms": pct(gaps_sorted, 0.5),
        "mcp_poll_gap_p95_ms": pct(gaps_sorted, 0.95),
        "mcp_poll_gap_max_ms": gaps_sorted[-1] if gaps_sorted else None,
        "mcp_poll_got_gap_p50_ms": pct(got_gaps, 0.5),
        "mcp_poll_got_gap_p95_ms": pct(got_gaps, 0.95),
        "mcp_poll_got_ts": polls_got,
    }
    return summary


def summarize(frames, arms, phase, size_name, label, phase_seconds=None):
    rows = []
    if phase.startswith("type"):
        members = [f for f in frames if f["dirty"] == 1 and f["s2_us"] > 0]
    else:
        members = [f for f in frames if f["present"] > 0 or f["s5_draw_us"] > 0]
    for f in members:
        total = (f["present"] - f["begin"]) if f["present"] > 0 else None
        seg = f["s1_us"] + f["s2_us"] + f["s3a_us"] + f["s3b_us"] + f["s4_us"]
        s5 = f["s5_layout_us"] + f["s5_shaping_us"] + f["s5_draw_us"]
        rows.append({
            "type": f"{phase}_frame", "label": label, "size": size_name,
            "present": f["present"], "draw_end_ms": f.get("draw_end_ms", -1),
            "total_ms": round(total / 1000.0, 2) if total is not None else None,
            "seg_sum_ms": round(seg / 1000.0, 2),
            "s5_ms": round(s5 / 1000.0, 2),
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
    # 呈现真相通道（735 同款）：distinct draw_end 集合=交付率分子。
    draw_ends = sorted({f["draw_end_ms"] for f in frames
                        if f.get("draw_end_ms", -1) > 0})
    present_ts = sorted({f["present"] for f in frames if f["present"] > 0})
    summary = {
        "type": "summary", "phase": phase, "label": label, "size": size_name,
        "frames": len(members),
        "present_paired": sum(1 for r in rows if r["total_ms"] is not None),
        "total_p50_ms": pct(tot, 0.5), "total_p95_ms": pct(tot, 0.95),
        "s5_p50_ms": pct(s5s, 0.5), "s5_p95_ms": pct(s5s, 0.95),
        "pump_events": presents,
        "pump_per_sec": round(presents / phase_seconds, 2) if phase_seconds else None,
        "orphan_fallthrough_frames": orphans,
        "draw_end_frames": len(draw_ends),
        "draw_end_per_sec": round(len(draw_ends) / phase_seconds, 2)
                            if phase_seconds else None,
        "notify_per_sec": round(len(present_ts) / phase_seconds, 2)
                          if phase_seconds else None,
        "draw_end_ts": draw_ends,
        "arms": {k: {"us": v[0], "n": v[1]} for k, v in arms.items()},
    }
    if members:
        n = len(members)
        for k in ("s1_ms", "s2_ms", "s3a_ms", "s3b_ms", "s4_ms",
                  "s5_layout_ms", "s5_shaping_ms", "s5_draw_ms"):
            summary[f"{k}_mean"] = round(
                sum(r[k] for r in rows) / n, 3)
    return rows, summary


def write_rows(out_path, rows):
    with open(out_path, "a", encoding="utf-8") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")


def launch_app(auto_bin, size_name, fixture_path):
    """启动 041 例 VM 轨（AUTO_SCHED_DIAG 三轴全开），等待 textarea 就绪。

    返回 (proc, mcp, eid, scrollable, log_path, log_f, ad)。
    """
    port = pick_free_port()
    ad = tempfile.mkdtemp(prefix=f"p740-{size_name}-")
    env = dict(os.environ,
               AUTO_BENCH="1", AUTO_FRAME_BENCH="1",
               AUTO_SCHED_DIAG="1",
               AUTOUI_MCP_PORT=str(port),
               AUTOUI_TEST_FIXTURES="1",
               AUTOUI_HOT_RELOAD="0",
               APPDATA=ad,
               AUTO_PROJECT_DIR=APP_DIR)
    log_path = os.path.join(ad, "app.log")
    print(f"[*] {size_name}: fixture={os.path.getsize(fixture_path)}B "
          f"port={port} log={log_path}")
    log_f = open(log_path, "w", encoding="utf-8")
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm"], cwd=APP_DIR, env=env,
                            stdout=log_f, stderr=subprocess.STDOUT)
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
        proc.terminate()
        log_f.close()
        return None
    scrollable = mcp.find_scrollable(eid)
    print(f"[*] {size_name}: textarea={eid} scrollable={scrollable}")
    mcp.fixture_open(fixture_path)
    time.sleep(4.0)
    return proc, mcp, eid, scrollable, log_path, log_f, ad


def mark(log_path):
    with open(log_path, encoding="utf-8", errors="replace") as f:
        return sum(1 for _ in f)


def last_log_t(log_path):
    """取日志末行的 SCHED-DIAG/FRAME t 值（app 对数钟读数，脚本钟映射锚）。"""
    try:
        with open(log_path, encoding="utf-8", errors="replace") as f:
            for line in reversed(f.readlines()[-200:]):
                m = re.search(r"t=(\d+)ms", line)
                if m:
                    return int(m.group(1))
    except Exception:
        pass
    return None


def read_log(log_path, n0, n1):
    with open(log_path, encoding="utf-8", errors="replace") as f:
        return f.readlines()[n0:n1]


def drive_phase(mcp, eid, scrollable, phase, hz_period, steps):
    """以目标周期驱动一个相位（时延自适应 sleep）。

    返回 (calls, t0w, t1w)：calls=[(t_start_perf, latency_s)]。
    """
    calls = []
    last_lat = 0.0
    t0w = time.time()
    for i in range(steps):
        sleep_for = max(0.0, hz_period - last_lat) if hz_period else 0.0
        if sleep_for > 0:
            time.sleep(sleep_for)
        try:
            if phase == "type":
                _, ts, lat = mcp.type_text(eid, "x")
            else:
                _, ts, lat = mcp.scroll_to(scrollable, (i + 1) * SCROLL_STEP_PX)
            calls.append((ts, lat))
            last_lat = lat
        except Exception as e:
            print(f"[-] {phase} 派发败: {e}")
    t1w = time.time()
    return calls, t0w, t1w


def rung_summary_row(label, size_name, hz, phase, calls, plines, frames):
    """阶梯 rung 相位跟随性谱行（脚本钟→对数钟映射对齐）。"""
    lats = sorted(c[1] * 1000 for c in calls)
    if len(calls) >= 2:
        drive_span_s = calls[-1][0] - calls[0][0]
    else:
        drive_span_s = 0.0
    drive_hz = (len(calls) - 1) / drive_span_s if drive_span_s > 0 else None
    # 对数钟映射锚：映射精度依赖 mark 时刻的末行 t 值——由调用方传
    # t0_log/t0w；此处仅组装帧侧数据。
    draw_ends = sorted({f["draw_end_ms"] for f in frames
                        if f.get("draw_end_ms", -1) > 0})
    delivers = sorted({int(m.group(1)) for m in
                       (SD_REDRAW_DELIVER_RE.search(l) for l in plines) if m})
    return {
        "type": "rung_phase", "label": label, "size": size_name,
        "rung_hz": hz, "phase": phase,
        "drive_calls": len(calls),
        "drive_lat_p50_ms": pct(lats, 0.5), "drive_lat_p95_ms": pct(lats, 0.95),
        "drive_span_s": round(drive_span_s, 3),
        "drive_hz_effective": round(drive_hz, 2) if drive_hz else None,
        "draw_end_count": len(draw_ends),
        "draw_end_ts": draw_ends,
        "deliver_count": len(delivers),
        "deliver_ts": delivers,
    }


def finalize_rung_follow(out_path, row, t0_log, calls):
    """补算跟随性：驱动窗映射到对数钟后数交付事件（+grace 宽限）。

    线性映射：log_t(call) = t0_log + (call_start_perf - first_call_perf)×1000
    （perf_counter 差即真实墙钟差；t0_log 锚=首调用前一刻日志末行 t 值）。
    主判据=follow_ratio_counts（交付数/驱动数，±10% 判据材料），
    Hz 比为参考（受 grace 稀释）。
    """
    if t0_log is None or not calls or row["drive_hz_effective"] is None:
        return
    span_s = row["drive_span_s"]
    first_perf = calls[0][0]
    t_first_log = t0_log
    t_last_log = t0_log + (calls[-1][0] - first_perf) * 1000
    grace_ms = RUNG_GRACE_S * 1000
    de = [t for t in row["draw_end_ts"]
          if t_first_log - 30 <= t <= t_last_log + grace_ms]
    dv = [t for t in row["deliver_ts"]
          if t_first_log - 30 <= t <= t_last_log + grace_ms]
    row["follow_window_log_ms"] = [round(t_first_log),
                                   round(t_last_log + grace_ms)]
    row["deliver_aligned_count"] = len(de)
    row["redeliver_aligned_count"] = len(dv)
    window_s = span_s + RUNG_GRACE_S
    deliver_hz = len(de) / window_s if window_s > 0 else None
    dvhz = len(dv) / window_s if window_s > 0 else None
    row["deliver_hz_aligned"] = round(deliver_hz, 2) if deliver_hz else None
    row["redeliver_hz_aligned"] = round(dvhz, 2) if dvhz else None
    if row["drive_hz_effective"]:
        row["follow_ratio_counts"] = round(
            len(de) / row["drive_calls"], 2) if row["drive_calls"] else None
        row["follow_ratio_draw_end"] = round(
            deliver_hz / row["drive_hz_effective"], 2) if deliver_hz else None
    gaps = [b - a for a, b in zip(de, de[1:])]
    row["draw_end_gap_p50_ms"] = pct(sorted(gaps), 0.5)
    row["draw_end_gap_p95_ms"] = pct(sorted(gaps), 0.95)
    write_rows(out_path, [row])


def run_size(label, size_name, target_bytes, line_tpl, auto_bin, keys, out_path,
             rungs=None):
    fx = make_fixture(size_name, target_bytes, line_tpl)
    launched = launch_app(auto_bin, size_name, fx)
    if not launched:
        print(f"[-] {size_name}: textarea 未定位")
        return None
    proc, mcp, eid, scrollable, log_path, log_f, ad = launched
    try:
        if rungs:
            for hz in rungs:
                period = 1.0 / hz
                steps = max(15, int(RUNG_PHASE_S * hz))
                for phase in ("type", "scroll"):
                    n0 = mark(log_path)
                    t0_log = last_log_t(log_path)
                    calls, _, _ = drive_phase(mcp, eid, scrollable, phase,
                                              period, steps)
                    time.sleep(RUNG_SETTLE_S)
                    n1 = mark(log_path)
                    plines = read_log(log_path, n0, n1)
                    frames = [f for f in (parse_frame_line(l) for l in plines)
                              if f]
                    _, summary = summarize(frames, {}, f"{phase}@{hz:g}",
                                           size_name, label,
                                           phase_seconds=RUNG_PHASE_S
                                           + RUNG_SETTLE_S)
                    sd = parse_sched_diag(plines)
                    sd["phase"] = f"{phase}@{hz:g}"
                    sd["label"] = label
                    sd["size"] = size_name
                    row = rung_summary_row(label, size_name, hz, phase,
                                           calls, plines, frames)
                    finalize_rung_follow(out_path, row, t0_log, calls)
                    write_rows(out_path, rows_for_phase(frames, summary, sd))
                    print(f"[+] {size_name}/{phase}@{hz:g}: "
                          f"drive={row['drive_hz_effective']}Hz "
                          f"(lat p50={row['drive_lat_p50_ms']}ms) "
                          f"deliver_aligned={row.get('deliver_aligned_count')}"
                          f"/{row['drive_calls']} "
                          f"follow={row.get('follow_ratio_counts')} "
                          f"de_gap_p50={row.get('draw_end_gap_p50_ms')}ms "
                          f"poll_gap_p50={sd['mcp_poll_gap_p50_ms']}/"
                          f"p95={sd['mcp_poll_gap_p95_ms']}ms "
                          f"got={len(sd['mcp_poll_got_ts'])}")
        else:
            # 735 协议复刻：type / scroll / typenl 三相位。
            def mark_():
                return mark(log_path)
            n0 = mark_(); t0w = time.time()
            for _ in range(keys):
                try:
                    mcp.type_text(eid, "x")
                except Exception as e:
                    print(f"[-] type 派发败: {e}")
                time.sleep(0.06)
            time.sleep(2.0)
            n1 = mark_(); t1w = time.time()
            if scrollable:
                for i in range(1, SCROLL_STEPS + 1):
                    try:
                        mcp.scroll_to(scrollable, i * SCROLL_STEP_PX)
                    except Exception as e:
                        print(f"[-] scroll 派发败: {e}")
                    time.sleep(SCROLL_INTERVAL)
                time.sleep(2.0)
            n2 = mark_(); t2w = time.time()
            for _ in range(keys):
                try:
                    mcp.type_text(eid, "z\n")
                except Exception as e:
                    print(f"[-] typenl 派发败: {e}")
                time.sleep(0.06)
            time.sleep(2.0)
            n3 = mark_(); t3w = time.time()
            print(f"[*] {size_name}: type=[{n0}:{n1}] scroll=[{n1}:{n2}] "
                  f"typenl=[{n2}:{n3}]")
            phase_seconds = {"type": t1w - t0w, "scroll": t2w - t1w,
                             "typenl": t3w - t2w}
            phases = {"type": (n0, n1), "scroll": (n1, n2),
                      "typenl": (n2, n3)}
            for phase, (a, b) in phases.items():
                plines = read_log(log_path, a, b)
                frames, arms_acc = [], {}
                for line in plines:
                    f = parse_frame_line(line)
                    if f:
                        frames.append(f)
                        continue
                    a2 = parse_arms_line(line)
                    if a2:
                        for k, (us, n) in a2.items():
                            prev = arms_acc.get(k, (0, 0))
                            arms_acc[k] = (prev[0] + us, prev[1] + n)
                rows, summary = summarize(frames, arms_acc, phase, size_name,
                                          label, phase_seconds.get(phase))
                sd = parse_sched_diag(plines)
                sd["phase"] = phase
                sd["label"] = label
                sd["size"] = size_name
                write_rows(out_path, rows + [summary, sd])
                s5_means = ", ".join(
                    "{}={}".format(k, summary.get(k + "_mean"))
                    for k in ("s5_layout_ms", "s5_shaping_ms", "s5_draw_ms"))
                print(f"[+] {size_name}/{phase}: frames={summary['frames']} "
                      f"paired={summary['present_paired']} "
                      f"totalP50={summary['total_p50_ms']} "
                      f"P95={summary['total_p95_ms']} "
                      f"s5P50={summary['s5_p50_ms']} "
                      f"P95={summary['s5_p95_ms']} "
                      f"pump/s={summary['pump_per_sec']} "
                      f"draw_end/s={summary['draw_end_per_sec']} "
                      f"poll_gap_p50={sd['mcp_poll_gap_p50_ms']}/"
                      f"p95={sd['mcp_poll_gap_p95_ms']}ms "
                      f"got={len(sd['mcp_poll_got_ts'])} "
                      f"means=({s5_means})")
        return True
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
        log_f.close()


def rows_for_phase(frames, summary, sd):
    """rung 相位行集：帧行 + 汇总 + sched_diag（follow 行由 finalize 单写）。"""
    members = [f for f in frames if f["present"] > 0 or f["s5_draw_us"] > 0
               or (f["dirty"] == 1 and f["s2_us"] > 0)]
    rows = [{
        "type": "rung_frame", "present": f["present"],
        "draw_end_ms": f.get("draw_end_ms", -1), "dirty": f["dirty"],
        "builds": f["builds"],
    } for f in members]
    return rows + [summary, sd]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--label", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--auto-bin", default=DEFAULT_BIN)
    ap.add_argument("--keys", type=int, default=30)
    ap.add_argument("--sizes", default="5kb,100kb,1mb")
    ap.add_argument("--rungs", default=None,
                    help='阶梯跟随性谱，如 "10,16,34,54"')
    ap.add_argument("--rung-size", default="5kb",
                    help="阶梯跟随性谱所用 fixture 档（默认 5kb）")
    args = ap.parse_args()

    with open(args.out, "w", encoding="utf-8") as f:
        f.write(json.dumps({"type": "meta", "label": args.label,
                            "auto_bin": args.auto_bin,
                            "keys": args.keys,
                            "scroll_steps": SCROLL_STEPS,
                            "scroll_step_px": SCROLL_STEP_PX,
                            "rungs": args.rungs,
                            "rung_size": args.rung_size,
                            "sched_diag": True,
                            "axes": ["mcp_poll", "update_end",
                                     "redraw_deliver", "draw_end_ms"],
                            "app": "examples/ui/041-auto-edit"}) + "\n")
    if args.rungs:
        rungs = [float(h) for h in args.rungs.split(",")]
        tb, tpl = SIZES[args.rung_size.strip().lower()]
        run_size(args.label, args.rung_size.strip().lower(), tb, tpl,
                 args.auto_bin, args.keys, args.out, rungs=rungs)
    else:
        for size in args.sizes.split(","):
            size = size.strip().lower()
            if size not in SIZES:
                print(f"[-] 未知档 {size}")
                continue
            tb, tpl = SIZES[size]
            run_size(args.label, size, tb, tpl, args.auto_bin, args.keys,
                     args.out)
    print(f"[*] 谱落 {args.out}")


if __name__ == "__main__":
    main()
