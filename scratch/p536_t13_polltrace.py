"""
PLAN-536 Phase 2 T13: AUTO_DEBUG_POLLTRACE 仪器化定位跑。
- UI 进程带 AUTO_DEBUG_POLLTRACE=1(引擎 SET_FIELD/门控追踪)
- 发送 → 60s 轮询: 每拍 autoui_state(streaming/messages/poll_window/pre_stream_len)
- 产出: vm 日志 [POLLTRACE] 序列 + 状态时序 + turns.jsonl 落库核对
"""
import json
import os
import re
import socket
import subprocess
import sys
import time
import urllib.request

AUTO_BIN = r"D:/autostack/.wt/lang-536/auto-lang/target/debug/auto.exe"
MUSK_ROOT = r"D:/autostack/.wt/lang-536/auto-musk"
MUSK_BIN = r"D:/autostack/auto-musk/backend/target/debug/musk.exe"
BACKEND_PORT = 9268
EV_DIR = r"D:/autostack/.wt/lang-536/auto-lang/scratch/p536_t13_evidence"
SEND_TEXT = "536P2 T13 polltrace 定位"


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
        with urllib.request.urlopen(r, timeout=20) as resp:
            res = json.loads(resp.read().decode())
            if "error" in res:
                raise RuntimeError(f"{tool}: {res['error']}")
            return res.get("result", {})

    def text(self, tool: str, args: dict = None) -> str:
        res = self.call(tool, args)
        return res.get("content", [{}])[0].get("text", "")


def main():
    os.makedirs(EV_DIR, exist_ok=True)
    ui_log = open(EV_DIR + r"\vm_ui.log", "ab")
    be_log = open(EV_DIR + r"\backend.log", "ab")

    be = subprocess.Popen(
        [MUSK_BIN, "serve", "--addr", f"127.0.0.1:{BACKEND_PORT}",
         "--workdir", r"D:\autostack\auto-musk\tmp\musk-demo"],
        stdout=be_log, stderr=be_log)
    time.sleep(2.5)
    if be.poll() is not None:
        print("[-] backend exited early"); sys.exit(4)

    mcp_port = pick_free_port()
    env = dict(os.environ,
               AUTOUI_MCP_PORT=str(mcp_port),
               AUTO_BACKEND=f"http://127.0.0.1:{BACKEND_PORT}",
               AUTO_VM_MERGE="0",
               AUTO_DEBUG_POLLTRACE="1",
               RUST_MIN_STACK="16777216")
    ui = subprocess.Popen([AUTO_BIN, "run", "--render=vm", "--no-merge"],
                          cwd=MUSK_ROOT, env=env, stdout=ui_log, stderr=ui_log)
    print(f"[*] backend :{BACKEND_PORT}  UI mcp:{mcp_port}")
    c = Client(mcp_port)
    t0 = time.time()
    snap = ""
    while time.time() - t0 < 60:
        if ui.poll() is not None:
            print(f"[-] UI exited early code={ui.returncode}"); sys.exit(4)
        try:
            snap = c.text("autoui_snapshot")
            if snap and "tree:" in snap:
                break
        except Exception:
            pass
        time.sleep(0.5)
    else:
        print("[-] UI not ready"); sys.exit(1)
    print("[+] ready")

    def qstate():
        s = c.text("autoui_state", {"fields": ["streaming", "messages", "poll_window", "pre_stream_len"]})
        def grab(name):
            m = re.search(name + r": ([^\n]*)", s)
            return m.group(1)[:40] if m else "?"
        return f"streaming={grab('streaming')} msgs_vmrefs={s.count('<vmref>')} poll_window={grab('poll_window')} pre_len={grab('pre_stream_len')}"

    lines = snap.splitlines()
    comp = send_btn = None
    for idx, line in enumerate(lines):
        if "textarea" in line and "#" in line:
            if "构建" in "\n".join(lines[idx:idx + 6]):
                m = re.search(r"#(vnode_\d+)", line); comp = m.group(1)
                for l2 in lines[idx + 1:idx + 40]:
                    if "button" in l2 and "#" in l2:
                        m2 = re.search(r"#(vnode_\d+)", l2); send_btn = m2.group(1); break
                break
    print(f"[*] composer={comp} send={send_btn}")
    print("[before]", qstate())

    c.text("autoui_type", {"element_id": comp, "text": SEND_TEXT, "clear_first": True})
    time.sleep(2.5)
    print("[typed ]", qstate())
    c.text("autoui_action", {"element_id": send_btn, "action": "press"})
    print("[+] sent; polling 60s with state samples")
    for i in range(20):
        time.sleep(3)
        try:
            print(f"t+{3*(i+1):>3}s", qstate())
            s = c.text("autoui_snapshot")
            open(EV_DIR + rf"\poll_{i}.txt", "w", encoding="utf-8").write(s)
        except Exception as e:
            print(f"[!] poll {i} err: {e}")
    open(EV_DIR + r"\state_final.txt", "w", encoding="utf-8").write(c.text("autoui_state"))
    ui.terminate()
    be.terminate()
    print("[+] done — grep vm log for [POLLTRACE]")


if __name__ == "__main__":
    main()
