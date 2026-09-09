#!/usr/bin/env python3
"""os-003 T3 秒表接 Tick —— VM 轨手测自动化（Start→走表→计圈→停止）。"""
import json, subprocess, time, urllib.request, os, sys

APP = r"D:\autostack\.wt\os-003\auto-lang\examples\ui\012-stopwatch"
AUTO = r"D:\autostack\auto-lang\target\debug\auto.exe"

proc = subprocess.Popen([AUTO, "run", "-r", "vm"], cwd=APP)

def call(tool, args=None):
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": tool, "arguments": args or {}}}
    r = urllib.request.Request("http://127.0.0.1:9247/mcp",
                               data=json.dumps(req).encode(),
                               headers={"Content-Type": "application/json"})
    res = json.loads(urllib.request.urlopen(r, timeout=10).read())
    c = res.get("result", {}).get("content", [{}])
    return c[0].get("text", "") if c else str(res.get("error"))

def state(*fields):
    t = call("autoui_state", {"fields": list(fields)})
    out = {}
    for l in t.splitlines():
        if ":" not in l:
            continue
        k, v = l.split(":", 1)
        v = v.strip()
        for suf in (" (str)", " (int)", " (float)", " (bool)"):
            if v.endswith(suf):
                v = v[: -len(suf)]
                break
        out[k.strip()] = v
    return out

try:
    s0 = None
    for _ in range(45):
        time.sleep(1)
        try:
            s0 = state("elapsed", "sw_on", "tab"); break
        except Exception:
            pass
    assert s0, "MCP 未就绪"
    print("boot:", s0)
    call("autoui_press_sequence", {"keys": ["开始"]})
    time.sleep(2.2)
    s1 = state("elapsed", "sw_on", "time_display", "ms_display")
    print("running:", s1)
    assert s1.get("sw_on") == '"true"', "sw_on 应为 true"
    elapsed = int(s1.get("elapsed", "0").strip('" '))
    assert elapsed >= 1750, f"2.2s 应累计 ≥1750ms（250ms tick 容差），实际 {elapsed}"
    call("autoui_press_sequence", {"keys": ["计圈"]})
    time.sleep(0.4)
    call("autoui_press_sequence", {"keys": ["停止"]})
    time.sleep(0.6)
    s2 = state("elapsed", "lap1", "sw_on")
    print("stopped:", s2)
    lap1 = s2.get("lap1", "").strip('"')
    assert ":" in lap1 and "." in lap1, f"lap1 应为 MM:SS.cc 形态，实际 {lap1}"
    assert s2.get("sw_on") == '"false"'
    # 停止后 elapsed 冻结（再等 1s 不变）
    time.sleep(1.0)
    s3 = state("elapsed")
    assert int(s3.get("elapsed", "0").strip('" ')) == int(s2.get("elapsed", "0").strip('" ')), "停止后应冻结"
    print("T3 VM 轨冒烟：PASS（走表/计圈/暂停冻结全过）")
finally:
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except Exception:
        proc.kill()
