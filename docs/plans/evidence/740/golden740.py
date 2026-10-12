#!/usr/bin/env python
"""PLAN-731 T-00/T-05: 视觉 golden 三形（键入/滚动/resize）——frozen ① 门。

复用 autoui_screenshot 内建 baseline/diff 通道（Plan 371 Task 20）：
- baseline 跑（改前）：--mode baseline → 存 tests/screensshots/731/<form>.png。
- 对照跑（改后）：  --mode diff    → 逐形 diff，全 matches=绿门（阈值 0.5%）。

三形（041-auto-edit，VM 轨，fixture 装载同 ladder）：
- type   : 装载 100KB fixture → 键入固定文本 → 截图。
- scroll : 续同一进程 → scroll_to 固定偏移（3600px）→ 截图。
- resize : 第二次启动 AUTO_VM_WINDOW=1000x640 → 同装载+键入 → 截图。

用法：
  python golden.py --mode baseline
  python golden.py --mode diff
"""

import argparse
import hashlib
import json
import os
import re
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))
APP_DIR = os.path.join(REPO, "examples", "ui", "041-auto-edit")
FIXTURE = os.path.join(HERE, "fixtures", "ladder-100kb.txt")
DEFAULT_BIN = os.path.join(REPO, "target", "release", "auto.exe")
# 基线落 app CWD 相对路径（VM 轨进程内 chdir src/front 后为
# src/front/tests/screenshots/——同启动形态下确定）。
GOLDEN_DIR = os.path.join(APP_DIR, "src", "front", "tests", "screenshots")
FORMS = ("type", "scroll", "resize")

TYPED = "golden typed line one\ngolden typed line two"
SCROLL_OFFSET = 3600.0


def wait_converged(mcp, timeout=40.0, gap=0.6):
    """键入逐字符异步排干的收敛门：snapshot 全文哈希连续两次相等（间隔
    gap 秒）才返回——截图不抢跑（基线自洽实录：settle 2.5s 仍截到中途态，
    状态栏行数 2450 vs 终态 2452）。"""
    deadline = time.time() + timeout
    last = None
    while time.time() < deadline:
        try:
            txt = mcp.snapshot()
        except Exception:
            time.sleep(0.4)
            continue
        h = hashlib.md5(txt.encode("utf-8", "ignore")).hexdigest()
        if last is not None and h == last[0] and time.time() - last[1] >= gap:
            return True
        last = (h, time.time())
        time.sleep(gap)
    print("[-] 收敛超时（快照仍在变化）——按当前态继续")
    return False


def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class Mcp:
    def __init__(self, port: int):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self._id = 1

    def call(self, tool: str, args: dict, timeout: int = 60) -> dict:
        body = json.dumps({
            "jsonrpc": "2.0", "id": self._id, "method": "tools/call",
            "params": {"name": tool, "arguments": args},
        }).encode()
        self._id += 1
        req = urllib.request.Request(self.url, data=body,
                                     headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            out = json.loads(resp.read().decode())
        if "error" in out:
            raise RuntimeError(f"{tool}: {out['error']}")
        return out.get("result", {})

    def snapshot(self) -> str:
        r = self.call("autoui_snapshot", {})
        return r.get("content", [{}])[0].get("text", "")

    def type_text(self, eid: str, text: str) -> None:
        self.call("autoui_type", {"element_id": eid.lstrip("#"),
                                  "text": text, "clear_first": False})

    def scroll_to(self, eid: str, y: float) -> None:
        self.call("autoui_action", {"element_id": eid, "action": "scroll",
                                    "value": y})

    def find_scrollable(self, textarea_eid: str):
        r = self.call("autoui_find", {"kind": "scrollable", "limit": 10})
        txt = r.get("content", [{}])[0].get("text", "")
        cands = re.findall(r"vnode_\w+", txt)
        if not cands:
            return None
        for line in txt.splitlines():
            if textarea_eid.lstrip("#") in line and "vnode_" in line:
                m = re.search(r"vnode_\w+", line)
                if m:
                    return m.group(0)
        return cands[0]

    def fixture_open(self, path: str) -> None:
        self.call("autoui_fixture", {
            "schema_version": 1,
            "state": {"auto_open_path": path},
            "trigger": {"widget": "App", "event": "Tick"},
        })

    def screenshot(self, name: str, mode: str, threshold: float) -> str:
        if mode == "baseline":
            r = self.call("autoui_screenshot", {"name": name, "baseline": True})
        else:
            r = self.call("autoui_screenshot",
                          {"name": name, "diff": True, "threshold": threshold})
        return r.get("content", [{}])[0].get("text", "")


def run_app(auto_bin, window: str | None):
    port = pick_free_port()
    ad = tempfile.mkdtemp(prefix="p740-golden-")
    env = dict(os.environ,
               AUTO_BENCH="1",
               AUTOUI_MCP_PORT=str(port),
               AUTOUI_TEST_FIXTURES="1",
               AUTOUI_HOT_RELOAD="0",
               APPDATA=ad,
               AUTO_PROJECT_DIR=APP_DIR)
    if window:
        env["AUTO_VM_WINDOW"] = window
    log_path = os.path.join(ad, "app.log")
    log_f = open(log_path, "w", encoding="utf-8")
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm"], cwd=APP_DIR, env=env,
                            stdout=log_f, stderr=subprocess.STDOUT)
    return proc, port, log_f, ad


def locate_editor(mcp):
    for _ in range(30):
        try:
            snap = mcp.snapshot()
            m2 = re.search(r"textarea #(\w+)", snap)
            if m2:
                return m2.group(1)
        except Exception:
            pass
        time.sleep(1.0)
    return None


def phase_set(mcp, name, mode, threshold):
    wait_converged(mcp)
    txt = mcp.screenshot(f"740-{name}", mode, threshold)
    print(f"[*] golden {name}: {txt[:160]}")
    return txt


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--mode", choices=["baseline", "diff"], required=True)
    ap.add_argument("--auto-bin", default=DEFAULT_BIN)
    ap.add_argument("--threshold", type=float, default=0.005)
    args = ap.parse_args()

    if not os.path.exists(FIXTURE):
        print(f"[-] fixture 缺失（先跑 ladder.py 生成）: {FIXTURE}")
        sys.exit(2)

    results = []

    # ── 形 1+2：标准窗（type + scroll） ──
    proc, port, log_f, ad = run_app(args.auto_bin, None)
    try:
        mcp = Mcp(port)
        eid = locate_editor(mcp)
        assert eid, "textarea 未定位"
        mcp.fixture_open(FIXTURE)
        time.sleep(4.0)
        mcp.type_text(eid, TYPED)
        time.sleep(1.0)
        results.append(("type", phase_set(mcp, "type", args.mode, args.threshold)))
        scroller = mcp.find_scrollable(eid)
        assert scroller, "scrollable 未定位"
        mcp.scroll_to(scroller, SCROLL_OFFSET)
        results.append(("scroll", phase_set(mcp, "scroll", args.mode, args.threshold)))
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
        log_f.close()

    # ── 形 3：resize（第二启动 1000x640） ──
    proc, port, log_f, ad = run_app(args.auto_bin, "1000x640")
    try:
        mcp = Mcp(port)
        eid = locate_editor(mcp)
        assert eid, "textarea 未定位 (resize)"
        mcp.fixture_open(FIXTURE)
        time.sleep(4.0)
        mcp.type_text(eid, TYPED)
        time.sleep(1.0)
        results.append(("resize", phase_set(mcp, "resize", args.mode, args.threshold)))
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
        log_f.close()

    if args.mode == "diff":
        bad = [(n, t) for n, t in results if "DIFFERS" in t.upper()]
        if bad:
            print(f"[-] golden 红: {bad}")
            sys.exit(1)
        print("[+] golden 三形全绿 (matches)")
    else:
        print(f"[+] baseline 已存 {GOLDEN_DIR}/740-{{{','.join(FORMS)}}}.png")


if __name__ == "__main__":
    main()
