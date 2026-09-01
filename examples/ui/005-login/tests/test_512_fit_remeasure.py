"""Plan 512 T2 实证之二：005-login 校验错误行增/缩 → fit 窗口双向跟随
（G1 改案后的第二实证腿——016 切月恒 42 格高度不变被证伪，见计划判定表节）。

断言：空表单点 Sign In → email/password 两条错误行出现，窗口物理高度
增加 >24px（滞回 8px 三倍）；两个输入框各键入非空值（EmailChanged /
PasswordChanged 即时清错误行）→ 窗口回落到基线 ±16px。
"""
import ctypes
import ctypes.wintypes
import os
import subprocess
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../011-calculator/tests"))
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


def find_id_by_text(snapshot_str: str, text: str):
    """快照中 input 的 placeholder 在元素行下一行——命中文本行后向上回溯
    最近的 `#id` 令牌。"""
    lines = snapshot_str.splitlines()
    for i, line in enumerate(lines):
        if text in line:
            for j in range(i, max(i - 4, -1), -1):
                for part in lines[j].split():
                    if part.startswith("#"):
                        return part.rstrip(":,{")
    return None


def main():
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../../"))
    auto_bin = os.path.join(root_dir, "target/debug/auto.exe")
    app_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "../"))
    out_dir = os.path.join(app_dir, "src/front/tests/screenshots")
    os.makedirs(out_dir, exist_ok=True)

    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))
    print(f"[*] Starting 005 VM (fit remeasure probe #2) on MCP port {port}...")
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
            last = wait_rect(proc.pid, "Login", timeout=2)
            if last and last != rect0:
                if rect0 is not None:
                    break  # 尺寸发生过变化 = 首测 shrink 已生效
            rect0 = last
            time.sleep(0.5)
        base = wait_rect(proc.pid, "Login")
        if not base:
            failures.append("window rect not found")
            print("[-] FAIL: cannot locate Login window")
            raise SystemExit(1)
        print(f"[+] fit 首测后窗口: {base[0]}x{base[1]}")

        print("[*] Submit empty form (expect two error rows, window grows)...")
        client.press_sequence(["Sign In"])
        grown = None
        start = time.time()
        while time.time() - start < 10:
            r = wait_rect(proc.pid, "Login", timeout=2)
            if r and r[1] > base[1] + 24:
                grown = r
                break
            time.sleep(0.5)
        if grown:
            print(f"[+] OK: 校验错误行出现后窗口增高 {base[1]} -> {grown[1]} (+{grown[1]-base[1]}px)")
        else:
            failures.append("window did not grow after validation errors")
            print("[-] FAIL: window height did not grow on validation errors")
        client.screenshot("512_005_errors_grown", save_path=os.path.join(out_dir, "512_005_errors_grown.png"))

        print("[*] Type into both inputs (errors clear, expect window shrinks back)...")
        snap = client.snapshot()
        email_id = find_id_by_text(snap, "you@example.com")
        pass_id = find_id_by_text(snap, "Enter your password")
        if not email_id or not pass_id:
            failures.append(f"input ids not found in snapshot (email={email_id}, password={pass_id})")
            print("[-] FAIL: cannot locate input element ids")
            raise SystemExit(1)
        client.call("autoui_type", {"element_id": email_id.lstrip("#"), "text": "a@b.com", "clear_first": True})
        client.call("autoui_type", {"element_id": pass_id.lstrip("#"), "text": "secret", "clear_first": True})
        shrunk = None
        start = time.time()
        while time.time() - start < 10:
            r = wait_rect(proc.pid, "Login", timeout=2)
            if r and abs(r[1] - base[1]) <= 16:
                shrunk = r
                break
            time.sleep(0.5)
        if shrunk:
            print(f"[+] OK: 错误行清除后窗口回缩到 {shrunk[0]}x{shrunk[1]}（基线 {base[0]}x{base[1]} ±16）")
        else:
            failures.append("window did not shrink back after errors cleared")
            print("[-] FAIL: window height did not shrink back")
        client.screenshot("512_005_errors_cleared", save_path=os.path.join(out_dir, "512_005_errors_cleared.png"))

        if failures:
            print(f"[-] {len(failures)} assertion(s) failed: {failures}", file=sys.stderr)
            sys.exit(1)
        print("[+] Plan 512 005 fit remeasure probe PASS")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    main()
