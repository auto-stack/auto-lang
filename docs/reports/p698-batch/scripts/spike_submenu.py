"""PLAN-698 T-03a spike: nested floating submenu — render + snapshot + screenshot gate (single run)."""
import json
import os
import re
import subprocess
import sys
import time
import urllib.request

AUTO = r"D:/autostack/.wt/lang-698/auto-lang/target/debug/auto.exe"
APP = r"D:/autostack/.wt/lang-698/auto-os/widgets-gallery"
PORT = 18801
SAVE = r"D:/autostack/.wt/lang-698/scratch-w1"
LOG = open(SAVE + "/spike_app.log", "w", encoding="utf-8")


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


def wait_mcp(mcp, seconds=90):
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


def ids_matching(snap, *needles):
    """BUTTON ids whose accessible text contains a needle."""
    out = []
    for line in snap.splitlines():
        m = re.match(r'\s*button #(vnode_\d+) "(.*)"', line)
        if m and any(n.lower() in m.group(2).lower() for n in needles):
            out.append((m.group(1), m.group(2)))
    return out


def shot(mcp, name):
    t0 = time.time()
    try:
        txt = mcp.text("autoui_screenshot",
                       {"name": name, "baseline": False,
                        "save_path": SAVE + "/" + name + ".png"}, timeout=12)
        print(f"[SHOT OK] {name} in {time.time()-t0:.1f}s -> {txt[:120]}")
        return True
    except Exception as e:
        print(f"[SHOT HANG/TIMEOUT] {name} after {time.time()-t0:.1f}s: {type(e).__name__}")
        return False


def main():
    env = dict(os.environ, AUTOUI_MCP_PORT=str(PORT))
    proc = subprocess.Popen([AUTO, "run"], cwd=APP, env=env,
                            stdout=LOG, stderr=subprocess.STDOUT)
    try:
        mcp = Mcp(PORT)
        if not wait_mcp(mcp):
            print("RESULT: app never answered MCP")
            return 2
        print("[+] MCP alive")
        snap = mcp.text("autoui_snapshot", {"mode": "rendered"}, timeout=20)
        open(SAVE + "/spike_home.txt", "w", encoding="utf-8").write(snap)

        # 1. navigate to the menubar page via sidebar nav
        nav = ids_matching(snap, "menubar")
        print("[nav candidates]", nav[:8])
        if not nav:
            print("RESULT: no menubar nav id found; see spike_home.txt")
            return 3
        nav_id = nav[0][0]
        print(mcp.text("autoui_action", {"element_id": nav_id, "action": "press"}, timeout=15)[:200])
        time.sleep(2)

        snap = mcp.text("autoui_snapshot", {"mode": "rendered"}, timeout=20)
        open(SAVE + "/spike_menubar_page.txt", "w", encoding="utf-8").write(snap)
        trigs = [t for t in ids_matching(snap, "File", "Edit", "View", "Profiles") if t[1] in ("File", "Edit", "View", "Profiles")]
        print("[trigger ids]", trigs[:10])
        if not trigs:
            pass
            print("[fallback text ids]", trigs[:6])
        if not trigs:
            print("RESULT: no menubar trigger found on page")
            return 4

        # 2. open the first menu (control shot — known good per P695)
        print(mcp.text("autoui_action", {"element_id": trigs[0][0], "action": "press"}, timeout=15)[:200])
        time.sleep(1.5)
        ok1 = shot(mcp, "spike_menu_open")

        # 3. find submenu trigger (inside open panel) and open it — the 695 hang site
        snap = mcp.text("autoui_snapshot", {"mode": "rendered"}, timeout=20)
        open(SAVE + "/spike_menu_open.txt", "w", encoding="utf-8").write(snap)
        subs = ids_matching(snap, "Share")
        print("[submenu candidates]", subs[:10])
        if not subs:
            print("RESULT: submenu trigger not visible in snapshot")
            return 5
        print(mcp.text("autoui_action", {"element_id": subs[0][0], "action": "press"}, timeout=15)[:200])
        time.sleep(1.5)

        snap2 = mcp.text("autoui_snapshot", {"mode": "rendered"}, timeout=20)
        open(SAVE + "/spike_submenu_open.txt", "w", encoding="utf-8").write(snap2)
        ok2 = shot(mcp, "spike_submenu_open")

        print(f"RESULT: control_shot={'OK' if ok1 else 'HANG'} submenu_shot={'OK' if ok2 else 'HANG'}")
        return 0
    finally:
        proc.terminate()
        LOG.close()


if __name__ == "__main__":
    sys.exit(main())
