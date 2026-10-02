#!/usr/bin/env python
"""PLAN-735 T-01/T-02: raw app.log 四轴分析（呈现/通知/送达/更新分离谱）。

用法：python analyze_base.py <app.log> <type_start> <type_end> <scroll_end> <typenl_end>
（行号窗口来自 ladder735 运行输出 type=[a:b] scroll=[b:c] typenl=[c:d]）
"""
import re
import sys

FRAME_RE = re.compile(
    r"\[P725-FRAME\] begin=(-?\d+) present=(-?\d+).*draw_end_ms=(-?\d+)")
DELIVER_RE = re.compile(r"\[SCHED-DIAG\] redraw_deliver t=(\d+)ms")
ENQUEUE_RE = re.compile(r"\[SCHED-DIAG\] frame_msg enqueue t=(\d+)ms")
UPDATE_RE = re.compile(r"\[SCHED-DIAG\] update t=(\d+)ms app=\S+ widget=\S* event=(\S+)")
ARM_RE = re.compile(r"\[SCHED-DIAG\] arm_frame t=(\d+)ms")
PUMP_ENTER_RE = re.compile(
    r"\[SCHED-DIAG\] frame_pump enter t=(\d+)ms queued_init=(-?\d+) cpu_cont=(-?\d+) parked_total=(-?\d+) write_q=(-?\d+)")

def pct(v, q):
    v = sorted(v)
    return v[min(len(v) - 1, int(len(v) * q))] if v else None

def main():
    path = sys.argv[1]
    windows = {
        "type": (int(sys.argv[2]), int(sys.argv[3])),
        "scroll": (int(sys.argv[3]), int(sys.argv[4])),
        "typenl": (int(sys.argv[4]), int(sys.argv[5])),
    }
    with open(path, encoding="utf-8", errors="replace") as f:
        lines = f.readlines()
    print(f"log lines={len(lines)}")
    for phase, (a, b) in windows.items():
        frames, delivers, enqueues, updates, arms, enters = [], [], [], [], [], []
        for line in lines[a:b]:
            m = FRAME_RE.search(line)
            if m:
                frames.append((int(m.group(1)), int(m.group(2)), int(m.group(3))))
                continue
            m = DELIVER_RE.search(line)
            if m:
                delivers.append(int(m.group(1)))
                continue
            m = ENQUEUE_RE.search(line)
            if m:
                enqueues.append(int(m.group(1)))
                continue
            m = UPDATE_RE.search(line)
            if m:
                updates.append((int(m.group(1)), m.group(2)))
                continue
            m = ARM_RE.search(line)
            if m:
                arms.append(int(m.group(1)))
                continue
            m = PUMP_ENTER_RE.search(line)
            if m:
                enters.append(tuple(int(x) for x in m.groups()))
        begins = sorted({f[0] for f in frames if f[0] > 0})
        draw_ends = sorted({f[2] for f in frames if f[2] > 0})
        presents = sorted({f[1] for f in frames if f[1] > 0})
        dur = max(draw_ends + delivers + [t for t, _ in updates] + [1]) - \
              min([t for t, _ in updates] + draw_ends[:1] or [0])
        # 呈现节奏：draw_end 相邻差
        gaps = [b2 - b1 for b1, b2 in zip(draw_ends, draw_ends[1:])]
        active_gaps = [g for g in gaps if g < 500]
        print(f"\n== {phase} == frames_rows={len(frames)} draws={len(draw_ends)} "
              f"paired={len(presents)} delivers={len(delivers)} enq={len(enqueues)} "
              f"updates={len(updates)} arms={len(arms)} pump_enters={len(enters)}")
        print(f"   draw_end span: {draw_ends[0] if draw_ends else '-'}..{draw_ends[-1] if draw_ends else '-'} "
              f"({(draw_ends[-1]-draw_ends[0])/1000:.2f}s)" if draw_ends else "   no draws")
        if active_gaps:
            print(f"   draw→draw gap P50={pct(active_gaps,0.5)}ms P95={pct(active_gaps,0.95)}ms "
                  f"(n={len(active_gaps)} active, {len(gaps)-len(active_gaps)} idle>500ms)")
        if delivers:
            dgaps = [b2 - b1 for b1, b2 in zip(delivers, delivers[1:])]
            adg = [g for g in dgaps if g < 500]
            print(f"   deliver gap P50={pct(adg,0.5)}ms P95={pct(adg,0.95)}ms "
                  f"(n={len(adg)} active)")
        src = [t for t, e in updates if "Src" in e or "mcp" in e.lower()]
        print(f"   driven updates (Src/mcp): {len(src)}")
        if enters:
            print(f"   counters(enter[0..3]): {enters[:3]}")
        # 配对率断言面：draw 帧中拿到 present>0 行的比例
        # （行级配对——present>0 的行数 / draw_end>0 的行数）
        drawn_rows = [f for f in frames if f[2] > 0]
        paired_rows = [f for f in drawn_rows if f[1] > 0]
        if drawn_rows:
            print(f"   row-pairing: {len(paired_rows)}/{len(drawn_rows)} "
                  f"({100*len(paired_rows)/len(drawn_rows):.0f}%)")

if __name__ == "__main__":
    main()
