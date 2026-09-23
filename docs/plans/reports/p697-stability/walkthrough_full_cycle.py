"""PLAN-697 T-02 全环走查：music-player 拉起→播放交互→关闭→宿主存活录证。

前置：宿主已起（repro_host_exit.launch_host），MCP 7712。
证据落本目录：t2-01..05 截图 + 终端输出转录。
"""
import json
import subprocess
import sys
import time
import urllib.request

PORT = 7712
SHOT = r"D:\autostack\.wt\lang-697\auto-lang\docs\plans\reports\p697-stability"


def call(tool, args):
    req = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                      "params": {"name": tool, "arguments": args}}).encode()
    rq = urllib.request.Request(f"http://127.0.0.1:{PORT}/mcp", data=req,
                                headers={"Content-Type": "application/json"})
    res = json.loads(urllib.request.urlopen(rq, timeout=20).read().decode())
    if "error" in res:
        raise RuntimeError(res["error"])
    return res.get("result", {}).get("content", [{}])[0].get("text", "")


def shot(name):
    out = call("autoui_screenshot", {"save_path": rf"{SHOT}\{name}.png"})
    print(f"[shot] {name}: {str(out)[:100]}")


def find_line(snap, needle):
    for line in snap.splitlines():
        if needle.lower() in line.lower():
            return line.strip()
    return None


def eid(line):
    return line.split()[0].lstrip("#").rstrip(":").strip()


def main():
    # ① 拉起 music-player。
    print("[1] bus launch 020-music-player")
    print("   ", call("autoui_desktop", {"action": "bus",
                                       "verb": "launch\t020-music-player"}))
    time.sleep(6)
    snap = call("autoui_snapshot", {"mode": "rendered"})
    landed = "AutoMusic" in snap or "音乐" in snap or "player" in snap.lower()
    print(f"[check] music-player 窗落地: {landed} (snapshot {len(snap.splitlines())} lines)")
    shot("t2-01-launched")
    open(rf"{SHOT}\t2-snapshot-after-launch.txt", "w", encoding="utf-8").write(snap)

    # ② 行击曲库首行（SelectIndex→播放态→current_url 挂 stream）。
    print("[2] click first track row")
    target = None
    for needle in ("粉雪", ".mp3", ".flac", "01 ", "track"):
        target = find_line(snap, needle)
        if target:
            break
    print("    row:", target)
    if target:
        print("[press]", call("autoui_press", {"element_id": eid(target)})[:80])
    time.sleep(4)
    shot("t2-02-after-row-click")

    # ③ 播放态观察 20s（ontimeupdate/onplaystate 事件流在跑）。
    print("[3] observe playing state 20s")
    time.sleep(20)
    snap2 = call("autoui_snapshot", {"mode": "rendered"})
    open(rf"{SHOT}\t2-snapshot-playing.txt", "w", encoding="utf-8").write(snap2)
    playing_line = find_line(snap2, "is_playing") or find_line(snap2, "暂停") \
        or find_line(snap2, "pause")
    print("[check] playing state marker:", playing_line)
    shot("t2-03-playing-20s")

    # ④ 关闭 music-player 窗（bus close\t<wid>——wid 从 shell 投影/state 取）。
    print("[4] close music-player window")
    st = call("autoui_state", {})
    open(rf"{SHOT}\t2-state-before-close.txt", "w", encoding="utf-8").write(st)
    wid = None
    for line in st.splitlines():
        if "020-music-player" in line and ("wid" in line.lower() or "win" in line.lower()):
            print("    state line:", line.strip()[:160])
    # 快照/状态里直接抓 16 进制或十进制 wid 形态。
    import re
    m = re.search(r"020-music-player[^\n]*?wid[^\d]*(\d+)", st, re.I)
    if m:
        wid = m.group(1)
    if wid:
        print("[close]", call("autoui_desktop", {"action": "bus",
                                                 "verb": f"close\t{wid}"})[:80])
    else:
        # 兜底：找窗标题栏 × 按钮（AURA 快照内 close 图标位）。
        x = find_line(snap2, "close") or find_line(snap2, "×")
        print("    fallback × element:", x)
        if x:
            print("[press]", call("autoui_press", {"element_id": eid(x)})[:80])
    time.sleep(4)
    shot("t2-04-after-close")

    # ⑤ 宿主存活终验。
    alive = subprocess.run(["tasklist", "/FI", "IMAGENAME eq ui_desktop.exe",
                            "/FO", "CSV", "/NH"], capture_output=True,
                           text=True).stdout.strip()
    print("[5] host alive check:", alive.splitlines()[0] if alive else "DEAD")
    snap3 = call("autoui_snapshot", {"mode": "rendered"})
    gone = not ("AutoMusic" in snap3 or "音乐" in snap3)
    print(f"[check] player 窗已撤: {gone}; 宿主快照仍响应 ({len(snap3.splitlines())} lines)")
    shot("t2-05-final-desktop")


if __name__ == "__main__":
    main()
