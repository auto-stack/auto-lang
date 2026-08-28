"""U3 wedge corpus 探针(产品级判据):一次 boot 测一形态。

判定:press 模式按钮 → 等待 → autoui_screenshot。
  成功返回 PNG 路径 = ALIVE(事件循环可服务截图通道)
  "timed out" 错误   = FROZEN(U3 症状:截图通道 10s 超时)
"""
import json, os, re, socket, subprocess, sys, time, urllib.request

APP = r"D:\autostack\auto-lang\crates\auto-lang\test\ui\plan446_u3_text_wedge"
AUTO = r"D:\autostack\auto-lang\target\debug\auto.exe"

def pick_free_port():
    with socket.socket() as s:
        s.bind(('127.0.0.1', 0)); return s.getsockname()[1]

def mcp(port, name, args=None, timeout=60):
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": name, "arguments": args or {}}}
    r = urllib.request.Request(f"http://127.0.0.1:{port}/mcp", data=json.dumps(req).encode(),
                               headers={'Content-Type': 'application/json'})
    with urllib.request.urlopen(r, timeout=timeout) as resp:
        return json.loads(resp.read().decode())

def text_of(resp):
    try: return resp["result"]["content"][0]["text"]
    except Exception: return json.dumps(resp)[:200]

def find_button(port, label):
    txt = text_of(mcp(port, "autoui_snapshot", {"include_state": False}))
    for m in re.finditer(r'button #(vnode_\d+) "([^"]*)"', txt, re.S):
        if label in [s.strip() for s in m.group(2).split("\n")]:
            return m.group(1)
    return None

def run(mode, hold_secs=5):
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))
    proc = subprocess.Popen([AUTO, "run", "-r", "vm", "src/front/app.at"], cwd=APP, env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        for _ in range(60):
            st = text_of(mcp(port, "autoui_state", timeout=10))
            if "No state available" not in st:
                break
            time.sleep(0.5)
        btn = find_button(port, mode)
        if not btn:
            print(f"[{mode}] BUTTON NOT FOUND")
            return "NO-BUTTON"
        mcp(port, "autoui_action", {"element_id": btn, "action": "press"})
        time.sleep(hold_secs)
        t = time.time()
        r = text_of(mcp(port, "autoui_screenshot", timeout=30))
        dt = time.time() - t
        if "timed out" in r:
            print(f"[{mode}] FROZEN (screenshot {dt:.1f}s -> {r[:60]!r})")
            return "FROZEN"
        print(f"[{mode}] ALIVE (screenshot {dt:.1f}s ok)")
        return "ALIVE"
    finally:
        proc.kill()

if __name__ == "__main__":
    modes = sys.argv[1:] or ["ok", "ta_655k", "ta_1m3", "ta_1m3_field", "text_1m3", "ta_breakable", "text_unbreakable"]
    results = {}
    for m in modes:
        results[m] = run(m)
    print("VERDICT:", json.dumps(results))
