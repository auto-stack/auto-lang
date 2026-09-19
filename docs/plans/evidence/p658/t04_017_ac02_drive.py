#!/usr/bin/env python3
"""PLAN-658 T-04 AC-02 驱动：画廊内嵌 017 双向收发 + 流式到达取证。

前置：画廊已在 run（proxy 3358、MCP 9247）。本脚本全新定位节点：
①选 017 → 种子消息渲染（client 模块经 proxy 拉数据）；
②输入消息（oninput → set_typing → SSE Typing 帧）→ 发送（POST → SSE
NewMessage 帧）→ Tick 排水 → 新消息渲染 + typing 指示器时序取样。
"""
import re
import sys
import time

sys.path.insert(0, r"D:/autostack/auto-lang/.agents/skills/autoui-verifier/scripts")
from test_vm_mcp import AutoUiMcpClient

MSG = "p658-live-sse-roundtrip"
SHOT = r"D:/autostack/auto-lang/docs/plans/evidence/p658/t04_017_bidirectional.png"


def find(c, needle, kind=None):
    snap = c.snapshot()
    for line in snap.splitlines():
        if needle not in line:
            continue
        if kind and kind not in line:
            continue
        m = re.search(r"(?:aura_\d+|vnode_\d+)", line)
        if m:
            return m.group(0), line.strip()[:110]
    return None, None


def main():
    c = AutoUiMcpClient(9247)
    # ① 选中 017。
    vid, line = find(c, "017-chat", "button")
    assert vid, "sidebar 017 entry not found"
    print("select:", vid, line)
    c.press(vid)
    time.sleep(2.5)

    seeded, _ = find(c, "Hey! How is the project going?")
    print("seeded message rendered:", bool(seeded))
    assert seeded, "seeded messages not rendered"

    sse = c.state(["__p658_sse"])
    print("sse handle:", sse.replace("\n", " "))
    assert ": -1" not in sse, "SSE not opened"

    # ② 输入 → typing 事件流 → 发送 → NewMessage 事件流。
    # composer 输入框在 snapshot 里不带 placeholder 文本——inspect 判别。
    snap = c.snapshot()
    inp = None
    for line in snap.splitlines():
        if "input" not in line:
            continue
        m = re.search(r"(?:aura_\d+|vnode_\d+)", line)
        if not m:
            continue
        if "Type a message" in c.inspect(m.group(0)):
            inp = m.group(0)
            break
    btn, _ = find(c, '"Send"', "button")
    assert inp and btn, f"composer/send not found: {inp} {btn}"

    samples = []
    print(c.type_text(inp, MSG))
    time.sleep(0.4)
    samples.append(("after-input", c.state(["typing_name"]).replace("\n", " ")))
    time.sleep(0.8)
    r = c.press(btn)
    print("send ok:", "ok" in r)
    for i in range(6):
        time.sleep(0.5)
        samples.append((f"t+{(i+1)*0.5:.1f}", c.state(["typing_name"]).replace("\n", " ")))

    time.sleep(1.0)
    snap = c.snapshot()
    hits = [l.strip()[:110] for l in snap.splitlines() if MSG in l]
    print("new message rendered:", len(hits))
    for h in hits[:3]:
        print(" ", h)
    for t, s in samples:
        print("typing", t, s)

    c.screenshot("t04_017", baseline=False, save_path=SHOT)
    print("screenshot:", SHOT)
    assert hits, "sent message did not render (SSE roundtrip failed)"
    print("AC-02 PASS")


if __name__ == "__main__":
    main()
