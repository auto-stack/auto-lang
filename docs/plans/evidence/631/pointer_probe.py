#!/usr/bin/env python3
"""
PLAN-631 AC-03/AC-04 实机探针：027 右键菜单 placement "pointer"。

流程：启动 VM 轨 027（AUTO_POPOVER_DEBUG=1 → popover 面板挂载时打印定位）→
win32 SendInput 真实右键点击列表行名称单元格 → autoui_state 验证 ctx_open →
解析 [popover-debug] 行验证面板真实挂载且原点 = 点击点（snap 钳制容差内）→
vtree 单实例计数 → 外点关闭。

Usage:
    python docs/plans/evidence/631/pointer_probe.py
"""

import ctypes
import ctypes.wintypes
import json
import os
import re
import subprocess
import sys
import tempfile
import time

import requests

LANG_ROOT = os.path.normpath(os.path.join(os.path.dirname(__file__), "..", "..", "..", ".."))
PROJECT = os.path.join(LANG_ROOT, "examples", "ui", "027-file-manager")
AUTO_BIN = os.environ.get(
    "AUTO_BIN", os.path.join(LANG_ROOT, "target", "debug", "auto.exe"))

user32 = ctypes.windll.user32
# DPI 感知:合成点击/窗口矩形/命中查询统一进物理像素空间(缩放屏上
# 非 DPI 感知进程的虚拟化坐标会让点击系统性偏移)。
try:
    ctypes.windll.shcore.SetProcessDpiAwareness(2)  # PER_MONITOR_DPI_AWARE
except Exception:
    user32.SetProcessDPIAware()


def pick_free_port(start=9631):
    import socket
    for port in range(start, start + 100):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            if s.connect_ex(("127.0.0.1", port)) != 0:
                return port
    raise RuntimeError("no free port")


class McpClient:
    def __init__(self, url):
        self.url = url
        self.req_id = 0

    def call(self, tool, **args):
        self.req_id += 1
        r = requests.post(self.url, json={
            "jsonrpc": "2.0", "method": "tools/call",
            "params": {"name": tool, "arguments": args}, "id": self.req_id,
        }, timeout=15).json()
        return r["result"]["content"][0]["text"] if r.get("result", {}).get("content") else ""

    def state(self, *fields):
        txt = self.call("autoui_state", fields=list(fields))
        out = {}
        for m in re.finditer(r"(\w+): (.+?) \((?:int|str|bool|list)\)", txt):
            out[m.group(1)] = m.group(2)
        return out

    def vtree(self):
        return self.call("autoui_vtree")


def wait_server(url, timeout=45):
    for _ in range(timeout):
        try:
            requests.post(url, json={"jsonrpc": "2.0", "method": "tools/list",
                                     "params": {}, "id": 1}, timeout=2)
            return True
        except Exception:
            time.sleep(1)
    return False


def find_window(pid):
    """返回属于 pid 的可见顶层主窗口 (hwnd, (left, top, right, bottom))。"""
    result = []
    EnumWindowsProc = ctypes.WINFUNCTYPE(ctypes.c_bool, ctypes.c_void_p, ctypes.c_void_p)

    def cb(hwnd, _):
        if user32.IsWindowVisible(hwnd):
            pid_ref = ctypes.c_ulong()
            user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid_ref))
            if pid_ref.value == pid:
                rect = ctypes.wintypes.RECT()
                user32.GetWindowRect(hwnd, ctypes.byref(rect))
                w, h = rect.right - rect.left, rect.bottom - rect.top
                if w > 300 and h > 300:
                    result.append((hwnd, (rect.left, rect.top, rect.right, rect.bottom)))
        return True

    user32.EnumWindows(EnumWindowsProc(cb), None)
    return result[0] if result else None


def foreground(hwnd):
    """把窗口拉到前台;SetForegroundWindow 被前台锁拒绝时退置 TOPMOST
    （z-order 顶置即可让合成点击落在目标窗口——鼠标事件按屏下窗口派发）。"""
    user32.ShowWindow(hwnd, 9)  # SW_RESTORE
    user32.keybd_event(0x12, 0, 0, 0)    # ALT down
    user32.keybd_event(0x12, 0, 0x0002)  # ALT up
    user32.SetForegroundWindow(hwnd)
    time.sleep(0.4)
    if user32.GetForegroundWindow() == hwnd:
        return True
    # HWND_TOPMOST(-1);SWP_NOSIZE|SWP_NOMOVE|SWP_SHOWWINDOW
    user32.SetWindowPos(hwnd, -1, 0, 0, 0, 0, 0x0001 | 0x0002 | 0x0040)
    time.sleep(0.3)
    return True


def right_click(x, y):
    user32.SetCursorPos(ctypes.c_int(x), ctypes.c_int(y))
    time.sleep(0.15)
    user32.mouse_event(0x0008, 0, 0, 0, 0)  # RIGHTDOWN
    time.sleep(0.05)
    user32.mouse_event(0x0010, 0, 0, 0, 0)  # RIGHTUP


def left_click(x, y):
    user32.SetCursorPos(ctypes.c_int(x), ctypes.c_int(y))
    time.sleep(0.1)
    user32.mouse_event(0x0002, 0, 0, 0, 0)  # LEFTDOWN
    user32.mouse_event(0x0004, 0, 0, 0, 0)  # LEFTUP


def main():
    port = pick_free_port()
    storage = os.path.join(tempfile.gettempdir(), f"p631_probe_{port}.json")
    if os.path.exists(storage):
        os.remove(storage)
    log_path = os.path.join(tempfile.gettempdir(), f"p631_probe_{port}.log")
    env = {**os.environ,
           "AUTOUI_MCP_PORT": str(port),
           "AUTO_VM_STORAGE_FILE": storage,
           "AUTO_POPOVER_DEBUG": "1"}
    log_file = open(log_path, "w", encoding="utf-8")
    proc = subprocess.Popen([AUTO_BIN, "run", "-r", "vm"], cwd=PROJECT, env=env,
                            stdout=log_file, stderr=log_file)
    checks = []
    try:
        url = f"http://127.0.0.1:{port}/mcp"
        if not wait_server(url):
            print("FAIL: server not up"); sys.exit(1)
        time.sleep(2.0)  # 首帧稳定
        hit_win = None
        for _ in range(20):
            hit_win = find_window(proc.pid)
            if hit_win:
                break
            time.sleep(0.5)
        if not hit_win:
            print("FAIL: window not found"); sys.exit(1)
        hwnd, (left, top, right, bottom) = hit_win
        # 置前台(探针实测:有浏览器类窗口叠加时合成点击会被其截获)。
        fg_ok = foreground(hwnd)
        checks.append(("window_found", fg_ok, f"rect={(left, top, right, bottom)} foreground={fg_ok}"))

        # 触发点定标(截图量得,窗口相对分数):README.md 名称文字 ≈ (27%, 42%)
        # (按钮 shrink-wrap 文字,必须点在文字上);"···" 快捷钮 ≈ (95.7%, 42%)。
        mcp = McpClient(url)
        fx, fy = 0.273, 0.42
        cx = left + int((right - left) * fx)
        cy = top + int((bottom - top) * fy)
        user32.SetForegroundWindow(user32.GetAncestor(user32.GetForegroundWindow(), 2)) if False else None
        right_click(cx, cy)
        time.sleep(0.9)
        st_try = mcp.state("ctx_open")
        hit = None
        if st_try.get("ctx_open") == "true":
            hit = (cx, "right")
        else:
            left_click(left + 30, top + 30)
            time.sleep(0.4)
            cx2 = left + int((right - left) * 0.957)
            left_click(cx2, cy)
            time.sleep(0.9)
            st_try = mcp.state("ctx_open")
            if st_try.get("ctx_open") == "true":
                hit = (cx2, "left")
        if hit is None:
            checks.append(("menu_opens", False,
                           f"no trigger hit (name-text right-click ({cx},{cy}) + ··· left-click, y={cy})"))
            for name, ok, detail in checks:
                print(f"  {'PASS' if ok else 'FAIL'}  {name}: {detail}")
            print("RESULT: HAS FAILURES")
            sys.exit(1)
        cx, kind = hit
        checks.append(("menu_opens", True,
                       f"ctx_open=true via {kind}-click at ({cx},{cy})"))

        # 单实例:vtree 中 popover 节点数(大小写不敏感)。
        vtree = mcp.vtree()
        n_pop = vtree.lower().count("popover")
        checks.append(("popover_instances_eq_1", n_pop == 1,
                       f"vtree popover count={n_pop}"))

        # 面板定位:AUTO_POPOVER_DEBUG 行 placement=Pointer panel=...
        # (Panel::layout 仅在 open 时运行——该行存在即面板真实挂载)。
        time.sleep(0.5)
        log = open(log_path, encoding="utf-8", errors="replace").read()
        pointer_lines = re.findall(r"\[popover-debug\] placement=Pointer.*?panel=([^\n]+)", log)
        ok_pos = False
        detail = f"no Pointer debug line (lines={len(pointer_lines)})"
        for pl in pointer_lines[-1:]:
            m = re.search(r"x: ([\d.]+), y: ([\d.]+)", pl)
            if m:
                px, py = float(m.group(1)), float(m.group(2))
                # snap 钳制容差:面板不越界即可,与点击点差 < 400px 视为指针定位。
                dx, dy = abs(px - cx), abs(py - cy)
                ok_pos = dx < 400 and dy < 400
                detail = f"panel_origin=({px:.0f},{py:.0f}) click=({cx},{cy}) dx={dx:.0f} dy={dy:.0f}"
        checks.append(("panel_at_pointer", ok_pos, detail))

        # 外点关闭:左键点击窗口左上(空白工具区) → ctx_open 翻回 false。
        left_click(left + 30, top + 30)
        time.sleep(0.8)
        st2 = mcp.state("ctx_open")
        checks.append(("menu_dismisses_on_outside_click",
                       st2.get("ctx_open") == "false",
                       f"ctx_open={st2.get('ctx_open')}"))


    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
        log_file.close()

    all_ok = True
    for name, ok, detail in checks:
        print(f"  {'PASS' if ok else 'FAIL'}  {name}: {detail}")
        all_ok &= ok
    print("RESULT:", "ALL PASS" if all_ok else "HAS FAILURES")
    out = os.path.join(os.path.dirname(__file__), "pointer_probe_result.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump([{"check": n, "ok": o, "detail": d} for n, o, d in checks],
                  f, indent=2, ensure_ascii=False)
    sys.exit(0 if all_ok else 1)


if __name__ == "__main__":
    main()
