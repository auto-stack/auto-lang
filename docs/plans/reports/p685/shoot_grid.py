"""PLAN-685 T-06: bps-gallery screenshot grid driver (3 bp x 3 section).

Usage: python shoot_grid.py <port> <out_dir> <suffix>
Drives an already-running VM/Vue MCP instance; saves <suffix>.png per cell.
"""
import json
import os
import re
import shutil
import sys
import time
import urllib.request

PORT = sys.argv[1] if len(sys.argv) > 1 else "9775"
OUT = sys.argv[2] if len(sys.argv) > 2 else "."
SUFFIX = sys.argv[3] if len(sys.argv) > 3 else "vm"
URL = f"http://127.0.0.1:{PORT}/mcp"

BTN_RE = re.compile(r'button #(\S+) "((?:[^"\\]|\\.)*)"')


def call(tool, args):
    req = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                      "params": {"name": tool, "arguments": args}}).encode()
    r = urllib.request.Request(URL, data=req, headers={"Content-Type": "application/json"})
    return json.load(urllib.request.urlopen(r, timeout=15)).get("result", {}).get(
        "content", [{}])[0].get("text", "")


def snap():
    return call("autoui_snapshot", {"mode": "rendered"})


def press(eid):
    return call("autoui_action", {"element_id": eid, "action": "press"})


def find_button(text, label, exact=False):
    for m in BTN_RE.finditer(text):
        vid, txt = m.group(1), m.group(2).split("\n")[0]
        if (exact and txt == label) or (not exact and txt.startswith(label)):
            return vid
    return None


def shot(name):
    p = os.path.join(OUT, f"{name}.png")
    for attempt in range(5):
        res = call("autoui_screenshot", {"name": name, "baseline": False})
        # receipt: "Screenshot saved to: <path>" — save_path prop ignored by
        # this build; the file lands under the app's tmp/ dir.
        m = re.search(r"saved to:\s*(.+)", res)
        size = 0
        if m:
            src = m.group(1).strip()
            try:
                shutil.copyfile(src, p)
                size = os.path.getsize(p)
            except OSError:
                size = 0
        if size > 10000:  # zero-size window hazard: retry
            print(f"  shot {name}: {size}B")
            return True
        time.sleep(2.0)
    print(f"  shot {name}: FAILED ({size}B)")
    return False


def wait_ready():
    for _ in range(140):
        try:
            if "tree:" in snap():
                return
        except Exception:
            pass
        time.sleep(0.3)
    raise SystemExit("MCP not ready")


def main():
    os.makedirs(OUT, exist_ok=True)
    wait_ready()
    time.sleep(6)  # window settle (first-30s hazard)
    shot(f"{SUFFIX}_00_initial")
    bps = [("dashboard/overview", "overview"),
           ("data-display/row-list", "row-list"),
           ("data-display/master-detail", "master-detail")]
    sections = [("Spec", "spec"), ("Reference", "reference"), ("Gotchas", "gotchas")]
    for bp_id, short in bps:
        card = None
        for _ in range(3):
            t = snap()
            card = find_button(t, bp_id)
            if card:
                break
            time.sleep(1)
        if not card:
            print(f"!! card not found for {bp_id}")
            continue
        for _ in range(3):
            press(card)
            time.sleep(1.0)
            if f'"{bp_id}"' in call("autoui_state", {"fields": ["selected_id"]}):
                break
        print(f"== {bp_id} selected ==")
        for label, sect in sections:
            for _ in range(3):
                t = snap()
                tab = find_button(t, label, exact=True)
                if tab:
                    press(tab)
                    time.sleep(1.0)
                    break
                time.sleep(0.6)
            shot(f"{SUFFIX}_{short}_{sect}")


if __name__ == "__main__":
    main()
