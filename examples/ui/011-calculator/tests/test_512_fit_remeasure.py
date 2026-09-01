"""Plan 512 T2：fit 动态重测实证——011-calculator Scientific/Basic 切换，
OS 窗口尺寸双向跟随（P504-2 原始复现面：首测一次性 → 切 Scientific 底部
"=" 键裁剪；本期机制 = 内容尺寸变化 → 重测 → 窗口跟随，滞回 8px）。

断言：切 Scientific 后窗口物理高度增加 >24px（实测量测 +44px 物理：
391.8→435.8 逻辑；起草假设的 +50px 偏高被实证修正，阈值为滞回 8px
的三倍，足以排除抖动）；切回 Basic 后回落到初始高度 ±16px（滞回阈值
两倍容差）。
"""
import ctypes
import ctypes.wintypes
import os
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(__file__))
from test_011_vm import AutoUiMcpClient, pick_free_port  # noqa: E402

user32 = ctypes.windll.user32


def find_window_rect(pid: int, title_part: str):
    """按进程 pid + 标题子串找顶层窗口，返回 (w, h) 物理像素。"""
    result = []

    @ctypes.WINFUNCTYPE(ctypes.c_bool, ctypes.c_void_p, ctypes.c_void_p)
    def enum_cb(hwnd, _lp):
        if not user32.IsWindowVisible(hwnd):
            return True
        proc_id = ctypes.c_ulong(0)
        user32.GetWindowThreadProcessId(hwnd, ctypes.byref(proc_id))
        if proc_id.value != pid:
            return True
        buf = ctypes.create_unicode_buffer(256)
        user32.GetWindowTextW(hwnd, buf, 256)
        if title_part.lower() in buf.value.lower():
            rect = ctypes.wintypes.RECT()
            user32.GetWindowRect(hwnd, ctypes.byref(rect))
            result.append((rect.right - rect.left, rect.bottom - rect.top))
        return True

    user32.EnumWindows(enum_cb, 0)
    return result[0] if result else None


def wait_rect(pid, title, timeout=10):
    start = time.time()
    while time.time() - start < timeout:
        r = find_window_rect(pid, title)
        if r:
            return r
        time.sleep(0.3)
    return None


def main():
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../../"))
    auto_bin = os.path.join(root_dir, "target/debug/auto.exe")
    app_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "../"))
    out_dir = os.path.join(app_dir, "src/front/tests/screenshots")
    os.makedirs(out_dir, exist_ok=True)

    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))
    print(f"[*] Starting 011 VM (fit remeasure probe) on MCP port {port}...")
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm"], cwd=app_dir, env=env)
    failures = []

    try:
        client = AutoUiMcpClient(port)
        ready = False
        start = time.time()
        while time.time() - start < 20:
            try:
                snap = client.snapshot()
                if snap and "tree:" in snap:
                    ready = True
                    break
            except Exception:
                pass
            time.sleep(0.3)
        if not ready:
            print("[-] Timeout waiting for MCP", file=sys.stderr)
            sys.exit(1)

        # 等首测收缩完成（ServiceTick 节拍 + 测量回执）。
        rect0 = None
        start = time.time()
        last = None
        while time.time() - start < 15:
            last = wait_rect(proc.pid, "Calculator", timeout=2)
            if last and last != rect0:
                if rect0 is not None:
                    break  # 尺寸发生过变化 = 首测 shrink 已生效
            rect0 = last
            time.sleep(0.5)
        base = wait_rect(proc.pid, "Calculator")
        if not base:
            failures.append("window rect not found")
            print("[-] FAIL: cannot locate Calculator window")
        else:
            print(f"[+] fit 首测后窗口: {base[0]}x{base[1]}")

        print("[*] Switching to Scientific (expect window grows)...")
        client.press_sequence(["Scientific"])
        grown = None
        start = time.time()
        while time.time() - start < 10:
            r = wait_rect(proc.pid, "Calculator", timeout=2)
            if r and base and r[1] > base[1] + 24:
                grown = r
                break
            time.sleep(0.5)
        if grown:
            print(f"[+] OK: Scientific 后窗口增高 {base[1]} -> {grown[1]} (+{grown[1]-base[1]}px)")
        else:
            failures.append("window did not grow after Scientific switch")
            print("[-] FAIL: window height did not grow (P504-2 still present?)")
        client.screenshot("512_011_scientific_grown", save_path=os.path.join(out_dir, "512_011_scientific_grown.png"))

        print("[*] Switching back to Basic (expect window shrinks back)...")
        client.press_sequence(["Basic"])
        shrunk = None
        start = time.time()
        while time.time() - start < 10:
            r = wait_rect(proc.pid, "Calculator", timeout=2)
            if r and base and abs(r[1] - base[1]) <= 16:
                shrunk = r
                break
            time.sleep(0.5)
        if shrunk:
            print(f"[+] OK: Basic 后窗口回缩到 {shrunk[0]}x{shrunk[1]}（基线 {base[0]}x{base[1]} ±16）")
        else:
            failures.append("window did not shrink back after Basic switch")
            print("[-] FAIL: window height did not shrink back")

        if failures:
            print(f"[-] {len(failures)} assertion(s) failed: {failures}", file=sys.stderr)
            sys.exit(1)
        print("[+] Plan 512 011 fit remeasure probe PASS")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    main()
