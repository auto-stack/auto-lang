"""
PLAN-536 T1 实机探针:timer 写 state → 画布是否重绘(像素级证据)。

跑 test/ui/plan051_timer 语料(App LocalTick 40ms + TickerStore 门控 PollTick),
视图文本 f"local=${.local_count} poll=${.store.poll_count}" 随计数推进。
两次截图对比像素差:变化 → 画布重绘正常;恒零 → 题1 实机复现。
"""
import json
import os
import socket
import struct
import subprocess
import sys
import time
import urllib.request
import zlib

AUTO_BIN = r"D:/autostack/auto-lang/target/debug/auto.exe"
APP_DIR = sys.argv[1] if len(sys.argv) > 1 else \
    r"D:/autostack/.wt/lang-536/auto-lang/test/ui/plan051_timer"
OUT_DIR = r"D:/autostack/.wt/lang-536/auto-lang/scratch/p536_t1_live"


def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class Client:
    def __init__(self, port: int):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self._req_id = 1

    def call(self, tool: str, args: dict = None) -> dict:
        req = {"jsonrpc": "2.0", "id": self._req_id,
               "method": "tools/call",
               "params": {"name": tool, "arguments": args or {}}}
        self._req_id += 1
        r = urllib.request.Request(self.url, data=json.dumps(req).encode(),
                                   headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(r, timeout=15) as resp:
            res = json.loads(resp.read().decode())
            if "error" in res:
                raise RuntimeError(f"{tool}: {res['error']}")
            return res.get("result", {})

    def text(self, tool: str, args: dict = None) -> str:
        res = self.call(tool, args)
        return res.get("content", [{}])[0].get("text", "")

    def screenshot_path(self, name: str) -> str:
        msg = self.text("autoui_screenshot", {"name": name, "baseline": False})
        print(f"    screenshot {name}: {msg}")
        marker = "saved to:"
        if marker in msg:
            return msg.split(marker, 1)[1].strip()


def png_pixels(path: str):
    """Minimal PNG decoder (RGBA, 8-bit) → (width, height, bytes)."""
    with open(path, "rb") as f:
        data = f.read()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "not a png"
    pos, idat, w, h, bitd, ctype = 8, b"", 0, 0, 0, 0
    while pos < len(data):
        ln = struct.unpack(">I", data[pos:pos+4])[0]
        typ = data[pos+4:pos+8]
        chunk = data[pos+8:pos+8+ln]
        if typ == b"IHDR":
            w, h, bitd, ctype = struct.unpack(">IIBB", chunk[:10])
        elif typ == b"IDAT":
            idat += chunk
        pos += 12 + ln
    assert bitd == 8 and ctype == 6, f"expect RGBA8, got bitd={bitd} ctype={ctype}"
    raw = zlib.decompress(idat)
    stride = w * 4
    out = bytearray(w * h * 4)
    prev = bytearray(stride)
    p = 0
    for y in range(h):
        f0 = raw[p]; p += 1
        line = bytearray(raw[p:p+stride]); p += stride
        if f0 == 1:
            for i in range(4, stride):
                line[i] = (line[i] + line[i-4]) & 0xFF
        elif f0 == 2:
            for i in range(stride):
                line[i] = (line[i] + prev[i]) & 0xFF
        elif f0 == 3:
            for i in range(stride):
                a = line[i-4] if i >= 4 else 0
                line[i] = (line[i] + ((a + prev[i]) >> 1)) & 0xFF
        elif f0 == 4:
            for i in range(stride):
                a = line[i-4] if i >= 4 else 0
                b = prev[i]
                c = prev[i-4] if i >= 4 else 0
                pa, pb, pc = abs(b-c), abs(a-c), abs(a+b-2*c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + pr) & 0xFF
        out[y*stride:(y+1)*stride] = line
        prev = line
    return w, h, bytes(out)


def pct_diff(a, b) -> float:
    (w1, h1, px1), (w2, h2, px2) = a, b
    if (w1, h1) != (w2, h2):
        return 100.0
    n = len(px1)
    diff = sum(1 for i in range(0, n, 16) if px1[i:i+4] != px2[i:i+4])
    total = n // 16
    return 100.0 * diff / max(total, 1)


def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))
    print(f"[*] launch auto run -r vm (mcp:{port}) in {APP_DIR}")
    proc = subprocess.Popen([AUTO_BIN, "run", "-r", "vm"], cwd=APP_DIR, env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT)
    try:
        c = Client(port)
        t0 = time.time()
        ready = False
        while time.time() - t0 < 30:
            try:
                snap = c.text("autoui_snapshot")
                if snap and "tree:" in snap:
                    ready = True
                    break
            except Exception:
                pass
            time.sleep(0.3)
        if not ready:
            print("[-] app not ready in 30s"); sys.exit(1)
        print("[+] ready, state read:")
        print("    " + c.text("autoui_state").replace("\n", "\n    ")[:600])

        p1 = c.screenshot_path("p536_t1_s0")
        if not p1:
            print("[-] no screenshot path"); sys.exit(1)
        # 落盘到固定名,防运行目录污染
        img1 = png_pixels(p1)

        time.sleep(2.5)  # ≥50 拍(40ms LocalTick)
        st = c.text("autoui_state")
        print(f"[+] after 2.5s state: {st.strip()[:400]}")
        p2 = c.screenshot_path("p536_t1_s1")
        img2 = png_pixels(p2)

        d = pct_diff(img1, img2)
        print(f"[*] pixel diff over 2.5s: {d:.3f}%")
        verdict = "CANVAS REPAINTS (timer 失效链通)" if d > 0.05 else \
                  "CANVAS FROZEN (题1 实机复现)" if d == 0.0 else f"marginal {d:.3f}%"
        print(f"[verdict] {verdict}")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    main()
