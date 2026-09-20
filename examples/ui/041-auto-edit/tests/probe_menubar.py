#!/usr/bin/env python3
"""PLAN-664 U-1 probe: where does the menubar-open chain break?

Starts the 041 app in VM mode, clicks the 视图 menubar trigger via
autoui_action, and dumps: (a) the click response itself, (b) the
pre/post snapshot diff around the trigger, (c) whether any popover
panel/items appear anywhere in the post-click snapshot.
"""
import os
import re
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import desktop_mcp as dm

AUTO_BIN = os.environ.get("AUTO_BIN") or "auto"
PROJECT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
PORT = dm.pick_free_port()
MCP_URL = f"http://127.0.0.1:{PORT}/mcp"

env = dict(os.environ)
env["AUTO_UI_MCP_PORT"] = str(PORT)
env["AUTO_OPEN_PATH"] = ""
env["AUTO_DEBUG_MENUBAR"] = "1"
err_file = open(os.path.join(os.environ.get("TEMP", "/tmp"), "p664_probe_stderr.log"), "w", encoding="utf-8", errors="replace")
proc = subprocess.Popen(
    [AUTO_BIN, "run", "-r", "vm", "."],
    cwd=PROJECT, env=env,
    stdout=subprocess.DEVNULL, stderr=err_file,
)
try:
    if not dm.wait_for_server(MCP_URL, 30):
        print("FATAL: MCP server never came up")
        sys.exit(1)
    mcp = dm.McpClient(MCP_URL)
    time.sleep(1.5)

    pre = mcp.snapshot()
    btn = dm.find_button_by_text(pre, "视图")
    print(f"[1] trigger button found: {btn is not None} ({btn})")
    if btn is None:
        sys.exit(1)

    # Raw click response — desktop_mcp.open_menu swallows this; we need it.
    resp = mcp.click(btn)
    print(f"[2] click response:\n{resp}")

    time.sleep(1.5)
    post = mcp.snapshot()

    # Did anything change at all?
    pre_lines, post_lines = set(pre.splitlines()), set(post.splitlines())
    added = [l for l in post.splitlines() if l not in pre_lines]
    removed = [l for l in pre.splitlines() if l not in post_lines]
    print(f"[3] snapshot diff after click: +{len(added)} / -{len(removed)} lines")
    for l in added[:25]:
        print(f"    + {l}")
    for l in removed[:10]:
        print(f"    - {l}")

    # Any popover/menu-item markers in the post snapshot?
    for needle in ("切换 Console", "Popover", "popover", "menubar"):
        hits = [l for l in post.splitlines() if needle in l]
        print(f"[4] '{needle}' in post snapshot: {len(hits)} lines")
        for l in hits[:6]:
            print(f"      {l}")

    # Menu open state visible? autoui_state for anything menu-ish
    st = mcp.state()
    print("[5] state tail:\n" + "\n".join(st.splitlines()[-8:]))

    # [6] Context around the 视图 trigger in the POST snapshot — is there a
    # popover panel (Column) sibling after it? Any children at all?
    lines = post.splitlines()
    for i, l in enumerate(lines):
        if '"视图"' in l:
            lo, hi = max(0, i - 3), min(len(lines), i + 40)
            print(f"[6] post-snapshot context lines {lo}..{hi}:")
            for j in range(lo, hi):
                print(f"      {j:4d} {lines[j]}")
            break
finally:
    dm._kill_proc_tree(proc)
    err_file.close()
    log = os.path.join(os.environ.get("TEMP", "/tmp"), "p664_probe_stderr.log")
    with open(log, "r", encoding="utf-8", errors="replace") as f:
        dbg = [l for l in f.read().splitlines() if "P664-DBG" in l]
    print(f"[7] {len(dbg)} debug lines from app stderr:")
    for l in dbg[:40]:
        print(f"      {l}")
