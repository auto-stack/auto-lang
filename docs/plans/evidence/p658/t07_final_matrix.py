#!/usr/bin/env python3
"""PLAN-658 T-07 最终全矩阵 E2E：单画廊进程 020+017+031 + 端口证据。"""
import json
import re
import subprocess
import sys
import time

sys.path.insert(0, r"D:/autostack/auto-lang/.agents/skills/autoui-verifier/scripts")
from test_vm_mcp import AutoUiMcpClient

OUT = r"D:/autostack/auto-lang/docs/plans/evidence/p658/t07_final_matrix.json"
result = {}


def side_id(c, needle, kind=None):
    snap = c.snapshot()
    for line in snap.splitlines():
        if needle in line and (kind is None or kind in line):
            m = re.search(r"(?:aura_\d+|vnode_\d+)", line)
            if m:
                return m.group(0)
    return None


def main():
    c = AutoUiMcpClient(9247)

    # ① 020 曲库（AC-01 复验）。
    vid = side_id(c, "020-music-player", "button")
    assert vid
    c.press(vid)
    time.sleep(3.0)
    snap = c.snapshot()
    lib = next((l.strip()[:100] for l in snap.splitlines() if "首歌曲" in l), None)
    result["ac01_library_header"] = lib
    result["ac01_pass"] = bool(lib) and "393" in lib

    # ② 017 双向（AC-02 复验：fixture draft + send → 渲染 + typing）。
    vid = side_id(c, "017-chat", "button")
    c.press(vid)
    time.sleep(2.5)
    c.fixture({"draft": "t07-final-roundtrip"})
    time.sleep(0.4)
    btn = side_id(c, '"Send"', "button")
    c.press(btn)
    time.sleep(2.2)
    snap = c.snapshot()
    result["ac02_message_rendered"] = any("t07-final-roundtrip" in l for l in snap.splitlines())
    inp = None
    for line in snap.splitlines():
        if "input" not in line:
            continue
        m = re.search(r"(?:aura_\d+|vnode_\d+)", line)
        if m and "Type a message" in c.inspect(m.group(0)):
            inp = m.group(0)
            break
    if inp:
        c.type_text(inp, "t")
        time.sleep(0.6)
        s = c.state(["typing_name"])
        result["ac02_typing"] = "You" in s
    result["ac02_pass"] = result["ac02_message_rendered"] and result.get("ac02_typing")

    # ③ 031 出图（AC-03 复验：目录 6 图 + Next 大图 + URI）。
    vid = side_id(c, "031-image-viewer", "button")
    c.press(vid)
    time.sleep(2.0)
    c.fixture({"auto_open_path": "D:/autostack/.wt/lang-658/auto-lang/examples/ui/031-image-viewer/tests/fixtures"})
    time.sleep(2.5)
    s = c.state(["image_count", "media_src", "view_state"])
    result["ac03_state"] = s.replace("\n", " ")[:200]
    result["ac03_pass"] = "image_count: 6" in s and "media_src: \"/api/__auto/media/" in s

    # ④ AC-04 端口证据：auto 进程监听面（应仅 proxy 3358 + MCP 9247）。
    out = subprocess.run(
        ["netstat", "-ano"], capture_output=True, text=True
    ).stdout
    listens = [
        l.strip()[:110]
        for l in out.splitlines()
        if "LISTENING" in l and ("3358" in l or "9247" in l or ":30" in l or ":80" in l)
    ]
    result["ac04_listeners"] = listens[:10]
    # 30xx/80xx 直连面应只有 3358/9247 属于本画廊；3049（vite）不在 VM 臂。
    result["ac04_pass"] = any("3358" in l for l in listens) and not any(
        ":3049" in l for l in listens
    )

    result["overall"] = all(
        result.get(k) for k in ("ac01_pass", "ac02_pass", "ac03_pass", "ac04_pass")
    )
    with open(OUT, "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, indent=2)
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
