"""Plan 446 批五 U1 探针：事件冻结（P0）复现。
启动 U1 corpus 的 VM 应用（真实 iced 窗口 + MCP），驱动侧栏两次 press，
断言 active_id 逐步翻转。现场症状：循环构建后 press 被接受但 active_id
冻结（全局死导航）。
"""
import json, os, re, socket, subprocess, sys, time, urllib.request

APP = r"D:\autostack\auto-lang\crates\auto-lang\test\ui\plan446_u1_event_freeze"
AUTO = r"D:\autostack\auto-lang\target\debug\auto.exe"

BUTTON_RE = re.compile(r'button #(\S+) "([^"]+)"')
FIELD_RE = re.compile(r'^\s+(active_id|current): "(.*?)" \(str\)', re.M)

def pick_free_port():
    with socket.socket() as s:
        s.bind(('127.0.0.1', 0)); return s.getsockname()[1]

def mcp(port, name, args=None):
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": name, "arguments": args or {}}}
    r = urllib.request.Request(f"http://127.0.0.1:{port}/mcp",
                               data=json.dumps(req).encode(),
                               headers={'Content-Type': 'application/json'})
    with urllib.request.urlopen(r, timeout=15) as resp:
        return json.loads(resp.read().decode())

def wait_state(port, timeout=30):
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            if "No state available" not in json.dumps(mcp(port, "autoui_state")):
                return True
        except Exception:
            pass
        time.sleep(0.5)
    return False

def state(port):
    txt = mcp(port, "autoui_state")["result"]["content"][0]["text"]
    return dict(FIELD_RE.findall(txt))

def buttons(port):
    txt = mcp(port, "autoui_snapshot")["result"]["content"][0]["text"]
    return BUTTON_RE.findall(txt)

def press(port, element_id):
    return mcp(port, "autoui_action", {"element_id": element_id, "action": "press"})

def main():
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))
    proc = subprocess.Popen([AUTO, "run", "-r", "vm", "src/front/app.at"], cwd=APP, env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        if not wait_state(port):
            print("BOOT-FAIL: state never became available"); return 2
        print("boot ok; initial state:", state(port))
        btns = buttons(port)
        print("buttons:", [b[1] for b in btns])
        roles = next((nid for nid, label in btns if label == "Roles"), None)
        if roles is None:
            print("SNAPSHOT-NO-ROLES"); return 3
        press(port, roles); time.sleep(1.0)
        s1 = state(port)
        print("after press#1 (Roles):", s1)
        btns2 = buttons(port)
        daemon = next((nid for nid, label in btns2 if label == "Daemon"), None)
        press(port, daemon); time.sleep(1.0)
        s2 = state(port)
        print("after press#2 (Daemon):", s2)
        ok = s1.get("active_id") == "roles" and s2.get("active_id") == "daemon"
        print("U1-VERDICT:",
              "NO-FREEZE (active_id updates through presses)" if ok
              else "FREEZE REPRODUCED")
        return 0 if ok else 1
    finally:
        proc.kill()

if __name__ == "__main__":
    sys.exit(main())
