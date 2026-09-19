"""
p656-scroll-pane VM(MCP) 验证驱动 —— PLAN-656 AC-16 实机腿。

验证链（每步断言经 autoui_state 读状态，不经源码推断）：
1. 布局面：snapshot 含 scroll-pane 节点（普通 y / x / hidden / managed）。
2. ordinary controller 全链：press to-end → press probe → state.probe > 0
   （native scroll_to_end → intent 队列 → update 排空 → iced scroll_to →
   on_scroll 测量 → 注册表快照 → scroll_state() 读出）。
3. managed controller 全链：press m-to-end → press m-probe →
   state.mprobe > 9_000_000（10M 逻辑 extent 驱动 range）。
4. on-scroll 观察：对观察 pane 发 MCP scroll → state.py > 0（8 位置实参派发）。
5. 截图归档。
"""
import json, os, socket, subprocess, sys, time, urllib.request

# --archive：通过后把快照/截图刷入 tests/（AC-16 归档物刷新档，review F-R2）。
ARCHIVE = "--archive" in sys.argv

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class Client:
    def __init__(self, port):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self._id = 1

    def call(self, name, args=None):
        req = {"jsonrpc": "2.0", "id": self._id, "method": "tools/call",
               "params": {"name": name, "arguments": args or {}}}
        self._id += 1
        r = urllib.request.Request(self.url, data=json.dumps(req).encode(),
                                   headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(r, timeout=10) as resp:
            res = json.loads(resp.read().decode())
            if "error" in res:
                raise RuntimeError(f"{name}: {res['error']}")
            return res.get("result", {})

    def text(self, name, args=None):
        return self.call(name, args).get("content", [{}])[0].get("text", "")

    def state(self, *fields):
        args = {"fields": list(fields)} if fields else {}
        return self.text("autoui_state", args)


def _id_of(line: str):
    for tok in line.replace("#", " ").split():
        if tok.startswith("aura_") or tok.startswith("vnode_"):
            return tok
    return None


def find_button_id(snap: str, label: str):
    for line in snap.splitlines():
        if f'"{label}"' in line and "button" in line:
            return _id_of(line)
    return None


def find_scroll_id(snap: str, needle: str):
    # 观察 pane：含 obs-1 文本的 scrollable 子树——VM 轨 scrollable 节点行
    # 形如 `scrollable #vnode_... {`；取 obs-1 文本行之前最近的 scrollable。
    lines = snap.splitlines()
    for i, line in enumerate(lines):
        if f'"{needle}"' in line:
            for j in range(i - 1, -1, -1):
                if "scrollable" in lines[j]:
                    return _id_of(lines[j])
    return None


def main():
    app_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    auto_bin = os.environ.get("AUTO_BIN", "auto")
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))
    print(f"[*] auto run -r vm @ {app_dir} (mcp:{port})")
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm"], cwd=app_dir, env=env)
    fails = []
    try:
        c = Client(port)
        snap = ""
        t0 = time.time()
        while time.time() - t0 < 25:
            try:
                snap = c.text("autoui_snapshot")
                if "tree:" in snap:
                    break
            except Exception:
                pass
            time.sleep(0.3)
        if "tree:" not in snap:
            print("[-] VM MCP not ready"); return 1
        print("[+] snapshot ready (%d chars)" % len(snap))

        # 1. 布局面
        for needle, why in [("y-row-01", "ordinary y pane"), ("x-cell-01", "ordinary x pane"),
                            ("managed_content key=cap", "managed pane placeholder")]:
            ok = needle in snap
            print(("[+] " if ok else "[-] ") + f"layout: {why}: {'OK' if ok else 'MISSING'}")
            if not ok:
                fails.append(why)

        def press(label):
            bid = find_button_id(snap, label)
            if not bid:
                fails.append(f"button not found: {label}"); return False
            c.call("autoui_action", {"element_id": bid, "action": "press"})
            time.sleep(0.5)
            # refresh snapshot for subsequent lookups
            nonlocal_snap[0] = c.text("autoui_snapshot")
            return True

        nonlocal_snap = [snap]
        snap = snap  # keep local for helper closures via nonlocal_snap

        def s():
            return nonlocal_snap[0]

        def get_state(*f):
            return c.state(*f)

        # 2. ordinary controller 全链（bind 预热与 intent 排空由 2s 心跳
        # 节拍驱动——update 早退使 tail 块只在心跳等长路径执行；等待窗
        # 覆盖预热+排空两拍）。
        time.sleep(2.6)
        st = get_state("probe")
        print(f"[*] initial state: {st[:120]}")
        press("probe")
        st0 = get_state("probe")
        press("to-end"); time.sleep(2.6); press("probe")
        st1 = get_state("probe")
        print(f"[*] probe after to-end: {st1[:160]}")
        v1 = extract_float(st1, "probe")
        if v1 is None or v1 <= 0:
            fails.append(f"ordinary controller chain: probe={st1[:80]}")
        else:
            print(f"[+] ordinary controller chain: probe={v1:.0f} (>0)")

        # 3. managed controller 全链（10M 逻辑 extent）。managed wrapper 的
        # 物化同步使 offset 多帧收敛（读回实证 360→180→…→9999776 族中间值，
        # on_scroll 回声会把中间值短暂写进注册表投影）——单次读会撞上收敛
        # 窗口（实测 1/8 概率读到 120），轮询直至稳定或超时。
        press("m-to-end")
        vm_ = None
        for _ in range(6):
            time.sleep(1.2)
            press("m-probe")
            stm = get_state("mprobe")
            vm_ = extract_float(stm, "mprobe")
            if vm_ is not None and vm_ >= 9_000_000:
                break
        print(f"[*] mprobe after m-to-end: {stm[:160]}")
        if vm_ is None or vm_ < 9_000_000:
            fails.append(f"managed controller chain: mprobe={stm[:80]}")
        else:
            print(f"[+] managed chain: mprobe={vm_:.0f} (>=9M, logical extent drives range)")

        # 4. on-scroll 观察（MCP scroll 普通滚动 → 8 实参派发）
        sid = find_scroll_id(s(), "obs-1")
        if sid:
            c.call("autoui_action", {"element_id": sid, "action": "scroll", "value": 60})
            time.sleep(0.5)
            sto = get_state("oy", "py", "vy")
            py = extract_float(sto, "py")
            oy = extract_float(sto, "oy")
            print(f"[*] on-scroll state: {sto[:160]}")
            if py is None or py <= 0 or oy is None or oy <= 0:
                fails.append(f"on-scroll observe: {sto[:80]}")
            else:
                print(f"[+] on-scroll observe: oy={oy:.0f} py={py:.3f}")
        else:
            fails.append("obs pane not found for scroll action")

        # 5. 截图归档（--archive：快照+截图刷入 tests/，AC-16 归档物保持
        # 交付态布局——review F-R2；默认仅打印不落盘，避免日常探针脏化归档）。
        try:
            if ARCHIVE:
                snap_now = c.text("autoui_snapshot")
                with open(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                       "vm_snapshot.txt"), "w", encoding="utf-8") as f:
                    f.write(snap_now)
                print("[*] snapshot archived -> tests/vm_snapshot.txt")
            res = c.text("autoui_screenshot", {"name": "p656_vm_review"})
            print(f"[*] screenshot: {res[:160]}")
            if ARCHIVE:
                import re as _re, shutil as _shutil
                m = _re.search(r"([A-Za-z]:\\\\[^\"]+?autoui-screenshot-\d+\.png|[A-Za-z]:\\[^\"]+?autoui-screenshot-\d+\.png)", res)
                if m:
                    src_path = m.group(1).replace("\\\\", "\\")
                    if os.path.exists(src_path):
                        _shutil.copyfile(src_path, os.path.join(
                            os.path.dirname(os.path.abspath(__file__)), "vm_review.png"))
                        print("[*] screenshot archived -> tests/vm_review.png")
                    else:
                        print(f"[!] archive source missing: {src_path}")
                else:
                    print("[!] screenshot path not parsed from result")
        except Exception as e:
            print(f"[!] screenshot failed: {e}")

        print("=" * 50)
        if fails:
            print("[-] FAILURES: " + "; ".join(fails))
            return 2
        print("[+] ALL P656 VM CHECKS PASSED")
        return 0
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()


def extract_float(state_text: str, field: str):
    import re
    m = re.search(rf'"{field}"\s*:\s*([-\d.eE+]+)', state_text)
    if not m:
        m = re.search(rf'{field}[^=\d-]*([-\d.eE+]+)', state_text)
    return float(m.group(1)) if m else None


if __name__ == "__main__":
    sys.exit(main())
