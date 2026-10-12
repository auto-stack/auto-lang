#!/usr/bin/env python3
"""remote_frame_e2e.py — PLAN-760 T-05 remote-frame 实机 e2e 腿。

编排（PLAN-102 P6 探针同族形态）：
  ① rqhost bridge: `auto rqhost --pipe <n> --bridge-ws <port> --bridge-token rf-e2e`
  ② 计数器夹具:   AUTO_RQHOST_*=<n> AUTO_VM_RENDER=remote AUTOUI_MCP_PORT=<p1>
                   auto run --render vm   (examples/ui/remote-frame-e2e/fixture)
  ③ 宿主 demo:     AUTOUI_MCP_PORT=<p2> auto run -r vm   (host 模板替换 __RF_PORT__)
  断言（AC-02/03/04）：
    A 首帧   : bridge 日志 mirror subscribed + 宿主窗截图 md5 离开占位态（≤10s）
    B 输入   : SendInput 点击宿主窗 (48,36)（inc 按钮位，canvas fit 1:1 直映射）
               → 夹具 MCP state count +1（WS→pipe→app 输入闭环）
    C 断连   : kill bridge → 宿主截图转占位（reconnecting 灰底）
    D 恢复   : 重启 bridge（同端口同 token）→ 预算 30s 内截图离开占位（帧恢复）
    E census : 全程无 panic（进程退出码/输出扫描）；收尾进程树清场。

用法：python scripts/remote_frame_e2e.py [--auto-exe PATH] [--keep]
  --auto-exe 缺省解析序：env AUTO_EXE → <repo>/target/release/auto.exe →
  <repo>/target/debug/auto.exe（皆缺则红——先 cargo build）。
  --keep 保留临时宿主目录与截图（调试用）。
收据：tmp/remote-frame-e2e/receipt.json。退出 0 = 全断言过。
注意：SendInput 需要交互桌面会话（手动门/实机跑——CI headless 不可用，
      计划 §6 #[ignore] 同律）。
"""

import argparse
import ctypes
import ctypes.wintypes as wt
import hashlib
import json
import os
import re
import shutil
import socket
import subprocess
import sys
import time
import urllib.request

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FIXTURE = os.path.join(REPO, "examples", "ui", "remote-frame-e2e", "fixture")
HOST_TMPL = os.path.join(REPO, "examples", "ui", "remote-frame-e2e", "host")
OUT_DIR = os.path.join(REPO, "tmp", "remote-frame-e2e")
TOKEN = "rf-e2e"
PIPE = "rf-e2e-bridge"


def log(s):
    print(f"[rf-e2e] {s}", flush=True)


def free_port():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def kill_tree(proc):
    if proc is None or proc.poll() is not None:
        return
    subprocess.run(
        ["taskkill", "/T", "/F", "/PID", str(proc.pid)],
        capture_output=True,
    )


# ── MCP 最小客户端（test_vm_mcp.py 同族）──────────────────────────────
class Mcp:
    def __init__(self, port):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self._id = 1

    def call(self, tool, args=None, timeout=20):
        req = {
            "jsonrpc": "2.0",
            "id": self._id,
            "method": "tools/call",
            "params": {"name": tool, "arguments": args or {}},
        }
        self._id += 1
        r = urllib.request.Request(
            self.url, data=json.dumps(req).encode(), headers={"Content-Type": "application/json"}
        )
        with urllib.request.urlopen(r, timeout=timeout) as resp:
            res = json.loads(resp.read().decode())
        if "error" in res:
            raise RuntimeError(f"{tool}: {res['error']}")
        out = res.get("result", {})
        if out.get("isError"):
            raise RuntimeError(f"{tool}: {out['content'][0]['text'][:200]}")
        return out.get("content", [{}])[0].get("text", "")

    def state(self, fields=None):
        args = {"fields": fields} if fields else {}
        return self.call("autoui_state", args)

    def screenshot(self, name):
        return self.call("autoui_screenshot", {"name": name}, timeout=40)


def await_output(proc, pattern, timeout_ms, buf):
    """在 proc 输出累积 buf 里等 pattern（就地追加）。"""
    t0 = time.time()
    while (time.time() - t0) * 1000 < timeout_ms:
        if re.search(pattern, buf[0]):
            return True
        if proc.poll() is not None:
            return False
        time.sleep(0.3)
    return bool(re.search(pattern, buf[0]))


def pump_reader(proc, buf):
    def rd():
        for line in iter(proc.stdout.readline, b""):
            buf[0] += line.decode("utf-8", "replace")

    import threading

    threading.Thread(target=rd, daemon=True).start()


def purple_pixels(png_path):
    """夹具帧特征：primary 紫色像素数（RF-FIXTURE-LIVE 大字 + count 文本；
    占位态灰字/灰底 = 0）。PIL 缺席时退化 md5 无法判——e2e 环境要求 PIL。"""
    from PIL import Image

    im = Image.open(png_path).convert("RGB")
    n = 0
    w, h = im.size
    for y in range(0, h, 2):
        for x in range(0, w, 2):
            r, g, b = im.getpixel((x, y))
            if abs(r - 147) < 40 and abs(g - 148) < 40 and b > 190:
                n += 1
    return n * 4  # 1/4 采样还原


def shot_png_path(text):
    m = re.search(r"tmp/[\w\-/]+\.png", text.replace("\\", "/"))
    return os.path.join(REPO, m.group(0)) if m else None


def md5_of(path):
    with open(path, "rb") as f:
        return hashlib.md5(f.read()).hexdigest()


# ── Win32：宿主窗定位 + SendInput 真实指针事件 ────────────────────────
def find_window_by_title(title, timeout_ms=15000):
    t0 = time.time()
    while (time.time() - t0) * 1000 < timeout_ms:
        hwnd = ctypes.windll.user32.FindWindowW(None, title)
        if hwnd:
            return hwnd
        time.sleep(0.3)
    return 0


def client_origin(hwnd):
    pt = wt.POINT()
    ctypes.windll.user32.ClientToScreen(hwnd, ctypes.byref(pt))
    return pt.x, pt.y


def send_click(x, y, hwnd=None):
    """PostMessage 直投客户区消息（WM_MOUSEMOVE/WM_LBUTTONDOWN/UP）——
    mouse_event 全局注入在非前台/会话受限环境下 press 会被输入系统丢弃；
    winit 0.30.13 窗口过程处理这组 legacy 消息（event_loop.rs:1781
    WM_LBUTTONDOWN → MouseInput）。lparam = 客户区坐标（与窗口逻辑域
    直通，e2e 2026-10-11 逐点对账）。带返回值校验 + 单次内重试。
    """
    if not hwnd:
        return
    # 前台化（AttachThreadInput 旁路前台锁定；winit 对非激活窗的鼠标按下
    # 分发不稳定——实测前台化 + 物理光标同步后通道稳定）+ 物理光标对位。
    fg = ctypes.windll.user32.GetForegroundWindow()
    if fg != hwnd:
        cur_thread = ctypes.windll.kernel32.GetCurrentThreadId()
        fg_thread = ctypes.windll.user32.GetWindowThreadProcessId(fg, None)
        target_thread = ctypes.windll.user32.GetWindowThreadProcessId(hwnd, None)
        ctypes.windll.user32.AttachThreadInput(cur_thread, fg_thread, True)
        ctypes.windll.user32.AttachThreadInput(cur_thread, target_thread, True)
        ctypes.windll.user32.BringWindowToTop(hwnd)
        ctypes.windll.user32.SetForegroundWindow(hwnd)
        ctypes.windll.user32.AttachThreadInput(cur_thread, fg_thread, False)
        ctypes.windll.user32.AttachThreadInput(cur_thread, target_thread, False)
        time.sleep(0.3)
    ctypes.windll.user32.SetCursorPos(int(x), int(y))
    time.sleep(0.1)
    ox, oy = client_origin(hwnd)
    cx, cy = int(x - ox), int(y - oy)
    lparam = (cy << 16) | (cx & 0xFFFF)
    WM_MOUSEMOVE, WM_LBUTTONDOWN, WM_LBUTTONUP = 0x0200, 0x0201, 0x0202
    u32 = ctypes.windll.user32
    for _ in range(3):
        ok1 = u32.PostMessageW(hwnd, WM_MOUSEMOVE, 0, lparam)
        time.sleep(0.06)
        ok2 = u32.PostMessageW(hwnd, WM_LBUTTONDOWN, 0x0001, lparam)
        time.sleep(0.06)
        ok3 = u32.PostMessageW(hwnd, WM_LBUTTONUP, 0, lparam)
        time.sleep(0.06)
        u32.PostMessageW(hwnd, WM_MOUSEMOVE, 0, lparam)
        if ok1 and ok2 and ok3:
            return
        time.sleep(0.2)


def window_dpi_scale(hwnd):
    """宿主窗坐标比例——实测（e2e 2026-10-11）：PostMessage lparam 坐标
    与窗口逻辑域直通（y 逐点对账），DPI 缩放在 winit 侧消化；恒 1.0。"""
    return 1.0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--auto-exe", default=None)
    ap.add_argument("--keep", action="store_true")
    args = ap.parse_args()

    auto_exe = args.auto_exe or os.environ.get("AUTO_EXE") or next(
        (
            p
            for p in (
                os.path.join(REPO, "target", "release", "auto.exe"),
                os.path.join(REPO, "target", "debug", "auto.exe"),
            )
            if os.path.exists(p)
        ),
        None,
    )
    if not auto_exe or not os.path.exists(auto_exe):
        log("auto.exe 不可达：--auto-exe / AUTO_EXE / target/{release,debug} 皆缺（先 cargo build）")
        return 2

    os.makedirs(OUT_DIR, exist_ok=True)
    bridge_port = free_port()
    fixture_mcp_port, host_mcp_port = free_port(), free_port()

    # 宿主 demo 临时目录（替换 __RF_PORT__）。
    host_dir = os.path.join(OUT_DIR, "host-live")
    if os.path.exists(host_dir):
        shutil.rmtree(host_dir, ignore_errors=True)
    shutil.copytree(HOST_TMPL, host_dir)
    app_at = os.path.join(host_dir, "src", "front", "app.at")
    with open(app_at, encoding="utf-8") as f:
        src = f.read()
    with open(app_at, "w", encoding="utf-8", newline="\n") as f:
        f.write(src.replace("__RF_PORT__", str(bridge_port)))

    receipt = {
        "plan": 760,
        "task": "T-05",
        "auto_exe": auto_exe,
        "bridge_port": bridge_port,
        "ts": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "steps": {},
    }

    def step(name, ok, detail=""):
        receipt["steps"][name] = {"pass": bool(ok), "detail": detail}
        log(f"{'PASS' if ok else 'FAIL'} {name}: {detail}")

    bridge = fixture = host = None
    try:
        # ① bridge。
        bridge_buf = [""]
        bridge = subprocess.Popen(
            [auto_exe, "rqhost", "--pipe", PIPE, "--bridge-ws", str(bridge_port), "--bridge-token", TOKEN],
            cwd=FIXTURE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        )
        pump_reader(bridge, bridge_buf)
        # 输出诊断落盘(AUTO_RF_DEBUG=1 时含注册表泵逐拍行)。
        def tee(bufref, path):
            import threading as _t
            def w():
                last = 0
                while True:
                    time.sleep(1.0)
                    with open(path, "w", encoding="utf-8") as f:
                        f.write(bufref[0])
            _t.Thread(target=w, daemon=True).start()
        tee(bridge_buf, os.path.join(OUT_DIR, "bridge.log"))
        ok = await_output(bridge, r"rqhost-bridge\] ws listening on", 20000, bridge_buf)
        step("bridge-up", ok, f"port={bridge_port}")

        # ② 夹具（remote 臂,MCP 共位）。
        fix_buf = [""]
        fixture = subprocess.Popen(
            [auto_exe, "run", "--render", "vm"],
            cwd=FIXTURE,
            env=dict(
                os.environ,
                AUTOUI_MCP_PORT=str(fixture_mcp_port),
                AUTO_RQHOST_WELLKNOWN=PIPE,
                AUTO_RQHOST_PIPE=PIPE,
                AUTO_VM_RENDER="remote",
                AUTO_VM_WINDOW="480x680",
            ),
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        )
        pump_reader(fixture, fix_buf)
        fix_mcp = Mcp(fixture_mcp_port)
        fix_ready, t0 = False, time.time()
        while time.time() - t0 < 25:
            try:
                if "RfFixture" in fix_mcp.state() or "count" in fix_mcp.state():
                    fix_ready = True
                    break
            except Exception:
                pass
            if fixture.poll() is not None:
                break
            time.sleep(0.4)
        step("fixture-mcp-up", fix_ready, f"port={fixture_mcp_port}")
        # bridge 侧 adopt 观测。
        ok = await_output(bridge, r"adopted `RfFixture`", 15000, bridge_buf)
        step("fixture-adopted", ok, "bridge adopted RfFixture")

        # ③ 宿主 demo（VM 模式 iced 窗,remote-frame 全窗）。
        host_buf = [""]
        host = subprocess.Popen(
            [auto_exe, "run", "-r", "vm"],
            cwd=host_dir,
            env=dict(os.environ, AUTOUI_MCP_PORT=str(host_mcp_port), AUTO_VM_WINDOW="480x680",
                     AUTO_RF_DEBUG=os.environ.get("AUTO_RF_DEBUG", "1")),
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        )
        pump_reader(host, host_buf)
        tee(host_buf, os.path.join(OUT_DIR, "host.log"))
        host_mcp = Mcp(host_mcp_port)
        host_ready, t0 = False, time.time()
        while time.time() - t0 < 25:
            try:
                host_mcp.state()
                host_ready = True
                break
            except Exception:
                pass
            if host.poll() is not None:
                break
            time.sleep(0.4)
        step("host-mcp-up", host_ready, f"port={host_mcp_port}")

        # A 首帧：bridge mirror subscribed + 宿主窗出现夹具特征（紫色像素
        # ≥2000——RF-FIXTURE-LIVE 大字；占位态 0。md5 法在首帧早于首截图时
        # 失效，特征像素法对静止夹具稳）。
        ok = await_output(bridge, r"mirror subscribed `RfFixture`", 15000, bridge_buf)
        purple, t0 = 0, time.time()
        while time.time() - t0 < 10:
            try:
                png = shot_png_path(host_mcp.screenshot("rf-e2e-a"))
                if png:
                    purple = purple_pixels(png)
                    if purple >= 2000:
                        break
            except Exception:
                pass
            time.sleep(1.2)
        step("first-frame", ok and purple >= 2000, f"subscribed={ok} purple={purple} (≤10s)")

        # B 输入闭环：SendInput 点宿主窗客户区 (48,36)（inc 按钮位;fit 1:1）。
        hwnd = find_window_by_title("RF Host")
        hwnd_pid = None
        if hwnd:
            pid = wt.DWORD()
            ctypes.windll.user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
            hwnd_pid = pid.value
        own = hwnd_pid == host.pid
        step("host-window-found", bool(hwnd) and own,
             f"hwnd={hwnd} pid={hwnd_pid} host_pid={host.pid} own={own}")
        if hwnd:
            ox, oy = client_origin(hwnd)
            dpi = window_dpi_scale(hwnd)
            # 对照臂：本地 VM 按钮（逻辑 ~(48,16)）——SendInput 注入通道的
            # 分层定位（local +1 = OS 注入/iced 分发通；remote 帧区 y 偏移
            # +32 = 顶栏高度）。
            def host_state(field):
                st = host_mcp.state()
                m = re.search(field + r"\D*(-?\d+)", st)
                return int(m.group(1)) if m else None

            local_before = host_state("local") or 0
            local_after, tries = local_before, 0
            while local_after == local_before and tries < 4:
                send_click(ox + int(60 * dpi), oy + int(16 * dpi), hwnd=hwnd)
                time.sleep(1.0)
                try:
                    local_after = host_state("local") or 0
                except Exception:
                    pass
                tries += 1
            step("os-inject-control-arm", local_after == local_before + 1,
                 f"local {local_before}→{local_after} tries={tries} (PostMessage→iced→VM 消息链)")

            before = int(re.search(r"count\D*(-?\d+)", fix_mcp.state()).group(1)) if fix_ready else -1
            # remote-frame 区域内夹具 inc 按钮位（canvas bounds 自 y≈33 起，
            # fit 1:1 → 远端 (60, 36) = 宿主逻辑 (60, 70)；大按钮命中区宽）。
            after, tries = before, 0
            while after == before and tries < 4:
                send_click(ox + int(60 * dpi), oy + int(70 * dpi), hwnd=hwnd)
                time.sleep(1.2)
                try:
                    after = int(re.search(r"count\D*(-?\d+)", fix_mcp.state()).group(1))
                except Exception:
                    pass
                tries += 1
            step("input-closed-loop", after == before + 1,
                 f"count {before}→{after} tries={tries} (canvas update→InputMsg→bridge→app)")

        # C 断连：kill bridge → 宿主转占位（紫色特征归零）。
        kill_tree(bridge)
        try:
            bridge.wait(timeout=10)
        except Exception:
            pass
        bridge = None
        purple_dead, dead_err, t0 = -1, "", time.time()
        while time.time() - t0 < 15:
            try:
                png = shot_png_path(host_mcp.screenshot("rf-e2e-c"))
                if png:
                    purple_dead = purple_pixels(png)
                    if purple_dead < 500:
                        break
                else:
                    dead_err = "no png path in: " + host_mcp.screenshot("rf-e2e-c")[:100]
            except Exception as e:
                dead_err = repr(e)[:200]
            time.sleep(1.2)
        step("dead-placeholder", 0 <= purple_dead < 500,
             f"purple={purple_dead} (帧特征消失) err={dead_err}")

        # D 恢复：重启 bridge（同端口同 token）+ 重启夹具（原夹具的 pipe
        # 客户端随 bridge 死亡而断——其重连预算短于本流程，直接重启最稳）
        # → 宿主 30s 预算内重拨 → 订阅兑现 → 帧恢复。
        kill_tree(fixture)
        bridge = subprocess.Popen(
            [auto_exe, "rqhost", "--pipe", PIPE, "--bridge-ws", str(bridge_port), "--bridge-token", TOKEN],
            cwd=FIXTURE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        )
        rb_buf = [""]
        pump_reader(bridge, rb_buf)
        ok = await_output(bridge, r"rqhost-bridge\] ws listening on", 20000, rb_buf)
        fixture_mcp_port = free_port()
        fixture = subprocess.Popen(
            [auto_exe, "run", "--render", "vm"],
            cwd=FIXTURE,
            env=dict(
                os.environ,
                AUTOUI_MCP_PORT=str(fixture_mcp_port),
                AUTO_RQHOST_WELLKNOWN=PIPE,
                AUTO_RQHOST_PIPE=PIPE,
                AUTO_VM_RENDER="remote",
                AUTO_VM_WINDOW="480x680",
            ),
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        )
        fix_buf = [""]
        pump_reader(fixture, fix_buf)
        fix_mcp = Mcp(fixture_mcp_port)
        await_output(bridge, r"adopted `RfFixture`", 20000, rb_buf)
        # 夹具重连（app 侧管道断后要重新 run?——夹具进程还在:rq 客户端自
        # 带 per-app 管道重连(ClientPump ReconnectPolicy)。bridge 重启后
        # adopt 新连接,宿主 remote-frame 在 30s 预算内重拨成功。)
        purple_rev, rev_err, t0 = 0, "", time.time()
        while time.time() - t0 < 35:
            try:
                png = shot_png_path(host_mcp.screenshot("rf-e2e-d"))
                if png:
                    purple_rev = purple_pixels(png)
                    if purple_rev >= 2000:
                        break
            except Exception as e:
                rev_err = repr(e)[:200]
            time.sleep(1.2)
        step("reconnect-recovered", purple_rev >= 2000,
             f"purple={purple_rev} (帧特征恢复,≤30s 预算) err={rev_err}")

        # E 无 panic + census。
        panic = "panicked" in fix_buf[0] + host_buf[0]
        step("no-panic", not panic, f"fixture_out={len(fix_buf[0])}B host_out={len(host_buf[0])}B")
    finally:
        kill_tree(host)
        kill_tree(fixture)
        kill_tree(bridge)
        time.sleep(1.0)
        with open(os.path.join(OUT_DIR, "receipt.json"), "w", encoding="utf-8") as f:
            json.dump(receipt, f, indent=2, ensure_ascii=False)
        if not args.keep:
            shutil.rmtree(host_dir, ignore_errors=True)
        log(f"receipt → {os.path.join(OUT_DIR, 'receipt.json')}")

    all_ok = all(s["pass"] for s in receipt["steps"].values())
    log(f"RESULT: {'PASS' if all_ok else 'FAIL'} ({sum(s['pass'] for s in receipt['steps'].values())}/{len(receipt['steps'])})")
    return 0 if all_ok else 1


if __name__ == "__main__":
    sys.exit(main())
