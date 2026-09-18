"""PLAN-655 AC-03: gallery VM embedded traversal — nav to 008/009/016, screenshot.

Reuses AutoUiMcpClient from the autoui-verifier skill. Gallery demos resolve
from AUTO_GALLERY_APPS (lang-655 group sibling auto-lang worktree corpus).
"""
import importlib.util
import os
import re
import subprocess
import sys
import time

SKILL = r"D:\autostack\.wt\lang-655\auto-lang\.agents\skills\autoui-verifier\scripts\test_vm_mcp.py"
GALLERY = r"D:\autostack\.wt\lang-655\auto-os\ui-gallery"
AUTO = r"D:\autostack\.wt\lang-655\auto-lang\target\debug\auto.exe"
APPS = r"D:\autostack\.wt\lang-655\auto-lang\examples\ui"

spec = importlib.util.spec_from_file_location("tvm", SKILL)
tvm = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tvm)

port = tvm.pick_free_port()
env = dict(os.environ, AUTOUI_MCP_PORT=str(port), AUTO_GALLERY_APPS=APPS)
proc = subprocess.Popen([AUTO, "run", "-r", "vm"], cwd=GALLERY, env=env)
vid = re.compile(r"#(vnode_\d+)")


def find_vnode(snap: str, label: str) -> str | None:
    for line in snap.splitlines():
        if label in line:
            m = vid.search(line)
            if m:
                return m.group(1)
    return None


try:
    client = tvm.AutoUiMcpClient(port)
    start = time.time()
    while time.time() - start < 90:
        try:
            snap = client.snapshot()
            if snap and "tree:" in snap:
                break
        except Exception:
            pass
        time.sleep(0.3)
    print("[+] gallery ready")

    for entry, shot in [("008-pricing-table", "p655_gallery_008"),
                        ("009-article-feed", "p655_gallery_009"),
                        ("016-calendar", "p655_gallery_016")]:
        snap = client.snapshot()
        node = find_vnode(snap, entry)
        if not node:
            print(f"[-] sidebar entry {entry} not found in snapshot")
            continue
        print(f"[*] pressing {entry} ({node})...")
        client.press(node)
        time.sleep(4)  # embedded demo compile + swap
        snap2 = client.snapshot()
        for needle in ["Single Developer", "$39", "Buy Now", "Contact Us"]:
            if needle in snap2:
                print(f"    [+] vtree contains: {needle}")
        res = client.screenshot(shot)
        print(f"    [+] screenshot: {res}")
        if entry == "008-pricing-table":
            # scroll reachability approximation: PageDown into the embedded
            # frame scroll, then capture again (待澄清项的自动化近似;人工滚轮
            # 复验仍归 review/实机会话)。
            client.keyboard("PageDown")
            time.sleep(1)
            client.keyboard("PageDown")
            time.sleep(1)
            res = client.screenshot("p655_gallery_008_scrolled")
            print(f"    [+] scrolled screenshot: {res}")
            snap3 = client.snapshot()
            for needle in ["Single Developer", "Buy Now"]:
                print(f"    [+] after scroll vtree contains {needle}: {needle in snap3}")
finally:
    proc.terminate()
    try:
        proc.wait(timeout=3)
    except subprocess.TimeoutExpired:
        proc.kill()
print("[+] done")
