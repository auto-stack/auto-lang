#!/usr/bin/env python
"""PLAN-740: 阶梯谱定责分析器。

从 ladder740.py 产出的 JSONL 提取交付节奏四轴定责表：
- 轴1 消费/tick 触发（mcp_poll）：tick gap 分布（真实触发节奏）+ got=1
  消费率（相邻消费时刻差分布）。
- 轴2 发出（update_end）：update 轮完成节奏 + emit→到达时延
  （update_end_ts → 最近 redraw_deliver_ts）。
- 轴3 到达（redraw_deliver）：到达节奏 + 到达→draw_end 时延。
- 轴4 交付（draw_end）：distinct draw_end 间隔分布 + 交付率 vs
  有效驱动率跟随比。

用法：
  python analyze740.py ladder-p740-pre.jsonl [--rungs-only]
"""

import argparse
import json
import sys


def pct(vals, q):
    if not vals:
        return None
    vals = sorted(vals)
    return vals[min(len(vals) - 1, int(len(vals) * q))]


def gaps(ts, lo=-10**9, hi=10**9):
    sel = [t for t in ts if lo <= t <= hi]
    return [b - a for a, b in zip(sel, sel[1:])]


def fmt(x, nd=1):
    return "-" if x is None else f"{x:.{nd}f}"


def analyze(path, rungs_only=False):
    meta = None
    sched_rows = []
    rung_rows = []
    summary_rows = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                r = json.loads(line)
            except json.JSONDecodeError:
                continue
            t = r.get("type")
            if t == "meta":
                meta = r
            elif t == "sched_diag":
                sched_rows.append(r)
            elif t == "rung_phase":
                rung_rows.append(r)
            elif t == "summary":
                summary_rows.append(r)

    if meta:
        print(f"# {path}\n  label={meta.get('label')} bin={meta.get('auto_bin')}"
              f" rungs={meta.get('rungs')} rung_size={meta.get('rung_size')}")

    if rung_rows:
        print("\n## 阶梯跟随性谱（rung_phase 行）")
        print("| 档.相位 | 驱动Hz(有效) | 时延p50/p95 ms | 交付数/驱动数 | "
              "follow(counts) | 交付Hz(对齐) | draw_end gap p50/p95 ms |")
        print("|---|---|---|---|---|---|---|")
        for r in rung_rows:
            print(f"| {r['size']}.{r['phase']} | {fmt(r['drive_hz_effective'])} "
                  f"| {fmt(r['drive_lat_p50_ms'])}/{fmt(r['drive_lat_p95_ms'])} "
                  f"| {r.get('deliver_aligned_count')}/{r['drive_calls']} "
                  f"| {fmt(r.get('follow_ratio_counts'), 2)} "
                  f"| {fmt(r.get('deliver_hz_aligned'))} "
                  f"| {fmt(r.get('draw_end_gap_p50_ms'))}/"
                  f"{fmt(r.get('draw_end_gap_p95_ms'))} |")

    if not rungs_only:
        print("\n## 相位四轴节奏（sched_diag 行）")
        print("| 相位 | tick数 | tick gap p50/p95/max | got消费拍 | "
              "got间隔 p50/p95 | update数 | update_end数 | deliver数 |")
        print("|---|---|---|---|---|---|---|---|")
        for r in sched_rows:
            got_ts = r.get("mcp_poll_got_ts", [])
            got_g = gaps(got_ts)
            print(f"| {r['size']}.{r.get('phase')} | {r['mcp_poll_ticks']} "
                  f"| {fmt(r['mcp_poll_gap_p50_ms'], 0)}/"
                  f"{fmt(r['mcp_poll_gap_p95_ms'], 0)}/"
                  f"{fmt(r['mcp_poll_gap_max_ms'], 0)} "
                  f"| {len(got_ts)} "
                  f"| {fmt(pct(got_g, 0.5), 0)}/{fmt(pct(got_g, 0.95), 0)} "
                  f"| {r['update_events']} | {r['update_end_events']} "
                  f"| {r['redraw_deliver_events']} |")

        print("\n## 时延链（update_end→deliver→draw_end，sched_diag 行内对读）")
        for r in sched_rows:
            ue = r.get("update_end_ts", [])
            dv = r.get("redraw_deliver_ts", [])
            # emit→deliver：每个 deliver 找其后最近的 update_end（滞后量）
            ed = []
            for d in dv:
                prevs = [u for u in ue if u <= d]
                if prevs:
                    ed.append(d - prevs[-1])
            de = r.get("update_ts_events", [])
            print(f"| {r['size']}.{r.get('phase')} "
                  f"| emit→deliver p50/p95: {fmt(pct(ed, 0.5))}/"
                  f"{fmt(pct(ed, 0.95))} ms |")

        print("\n## 735 协议 summary（零回退对照面）")
        print("| 档.相位 | frames | paired | 配对率 | total p50/p95 | s5 p50/p95 | "
              "pump/s | draw_end/s |")
        print("|---|---|---|---|---|---|---|---|")
        for r in summary_rows:
            rate = (r["present_paired"] / r["frames"] * 100) if r["frames"] else 0
            print(f"| {r['size']}.{r['phase']} | {r['frames']} "
                  f"| {r['present_paired']} | {rate:.0f}% "
                  f"| {fmt(r['total_p50_ms'])}/{fmt(r['total_p95_ms'])} "
                  f"| {fmt(r['s5_p50_ms'], 2)}/{fmt(r['s5_p95_ms'], 2)} "
                  f"| {fmt(r['pump_per_sec'], 2)} "
                  f"| {fmt(r['draw_end_per_sec'], 2)} |")


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("jsonl")
    ap.add_argument("--rungs-only", action="store_true")
    args = ap.parse_args()
    analyze(args.jsonl, args.rungs_only)
