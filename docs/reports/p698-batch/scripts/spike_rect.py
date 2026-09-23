"""PLAN-698 T-03a spike verdict probe: render-face via select_rect + screenshot retry."""
import json
import os
import re
import subprocess
import sys
import time
import urllib.request

AUTO = r"D:/autostack/.wt/lang-698/auto-lang/target/debug/auto.exe"
APP = r"D:/autostack/.wt/lang-698/auto-os/widgets-gallery"
PORT = 18802
SAVE = r"D:/autostack/.wt/lang-698/scratch-w1"
LOG = open(SAVE + "/spike_rect_app.log", "w", encoding="utf-8")


class Mcp:
    def __init__(self, port):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self.req_id = 0

    def call(self, tool, args=None, timeout=12):
        self.req_id += 1
        req = json.dumps({"jsonrpc": "2.0", "id": self.req_id,
                          "method": "tools/call",
                          "params": {"name": tool, "arguments": args or {}}}).encode()
        r = urllib.request.Request(self.url, data=req,
                                   headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(r, timeout=timeout) as resp:
            res = json.loads(resp.read().decode("utf-8"))
        if "error" in res:
            raise RuntimeError(res["error"])
        return res.get("result", {})

    def text(self, tool, args=None, timeout=12):
        return self.call(tool, args, timeout).get("content", [{}])[0].get("text", "")


def wait_ui(mcp, seconds=90):
    deadline = time.time() + seconds
    while time.time() < deadline:
        try:
            snap = mcp.text("autoui_snapshot", {"mode": "rendered"}, timeout=5)
            if "No UI available yet" not in snap:
                return True
        except Exception:
            pass
        time.sleep(2)
    return False


def find_button(snap, label):
    for line in snap.splitlines():
        m = re.match(r'\s*button #(vnode_\d+) "(.*)"', line)
        if m and m.group(2) == label:
            return m.group(1)
    return None


def main():
    env = dict(os.environ, AUTOUI_MCP_PORT=str(PORT))
    proc = subprocess.Popen([AUTO, "run"], cwd=APP, env=env,
                            stdout=LOG, stderr=subprocess.STDOUT)
    try:
        mcp = Mcp(PORT)
        if not wait_ui(mcp):
            print("RESULT: no UI")
            return 2
        snap = mcp.text("autoui_snapshot", {"mode": "rendered"}, timeout=20)
        nav = find_button(snap, "Menubar")
        mcp.text("autoui_action", {"element_id": nav, "action": "press"}, timeout=15)
        time.sleep(2)
        snap = mcp.text("autoui_snapshot", {"mode": "rendered"}, timeout=20)

        trig = find_button(snap, "File")
        mcp.text("autoui_action", {"element_id": trig, "action": "press"}, timeout=15)
        time.sleep(1.5)
        snap = mcp.text("autoui_snapshot", {"mode": "rendered"}, timeout=20)
        sub = find_button(snap, "Share")
        # bounds of the Share trigger from the snapshot line context is not
        # available; use select_rect with the full window then filter.
        mcp.text("autoui_action", {"element_id": sub, "action": "press"}, timeout=15)
        time.sleep(1.5)

        # render-face probe: rect over the whole window
        res = mcp.call("autoui_select_rect",
                       {"x": 0.0, "y": 0.0, "w": 100000.0, "h": 100000.0, "format": "json"}, timeout=15)
        txt = res.get("content", [{}])[0].get("text", "")
        open(SAVE + "/spike_rect_open.json", "w", encoding="utf-8").write(txt)
        try:
            env_json = json.loads(txt)
            nodes = env_json.get("nodes", [])
            hits = []
            for n in nodes:
                body = json.dumps(n)
                if "Email Link" in body or ("Twitter" in body and "Share" not in n.get("id", "")):
                    hits.append({k: n.get(k) for k in ("id", "kind", "bounds", "x", "y", "w", "h", "rect") if k in n})
            print("[select_rect] nodes:", len(nodes))
            print("[submenu items in RENDER surface]:", json.dumps(hits, ensure_ascii=False)[:800])
            # Share trigger bounds for geometry comparison
            for n in nodes:
                if "Share" in json.dumps(n):
                    print("[Share trigger node]:", json.dumps({k: n.get(k) for k in ("id", "bounds", "x", "y", "w", "h", "rect") if k in n})[:400])
                    break
        except json.JSONDecodeError:
            print("[select_rect] non-JSON:", txt[:200])

        # screenshot retry (channel behavior note)
        for attempt in (1, 2):
            t0 = time.time()
            try:
                txt = mcp.text("autoui_screenshot",
                               {"name": f"spike_rect_shot{attempt}", "baseline": False,
                                "save_path": SAVE + f"/spike_rect_shot{attempt}.png"}, timeout=25)
                print(f"[shot {attempt}] {time.time()-t0:.1f}s -> {txt[:100]}")
            except Exception as e:
                print(f"[shot {attempt}] exception after {time.time()-t0:.1f}s: {type(e).__name__}")
        print("RESULT: probe complete; process alive =", proc.poll() is None)
        return 0
    finally:
        proc.terminate()
        LOG.close()


if __name__ == "__main__":
    sys.exit(main())
