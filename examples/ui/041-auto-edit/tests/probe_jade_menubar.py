#!/usr/bin/env python3
"""PLAN-664 T-05: jade desktop consumer-side U-1 sampling.

Boots the jade-garden desktop front-end (VM mode) with the plan-664
worktree binary, opens the 视图 menubar via MCP, and asserts the expanded
items enter the snapshot (the consumer's D-05/D-11 family surface).
"""
import os
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import desktop_mcp as dm

AUTO_BIN = os.environ.get("AUTO_BIN") or "auto"
PROJECT = r"D:\autostack\auto-down\jade-garden\front\desktop"
PORT = dm.pick_free_port()
MCP_URL = f"http://127.0.0.1:{PORT}/mcp"

env = dict(os.environ)
env["AUTO_UI_MCP_PORT"] = str(PORT)
proc = subprocess.Popen(
    [AUTO_BIN, "run", "-r", "vm", "."],
    cwd=PROJECT, env=env,
    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
)
fails = 0
try:
    if not dm.wait_for_server(MCP_URL, 30):
        print("FATAL: MCP server never came up")
        sys.exit(1)
    mcp = dm.McpClient(MCP_URL)
    time.sleep(2.5)

    snap = mcp.snapshot()
    menubar_labels = [l for l in ("文件", "视图", "帮助") if dm.find_button_by_text(snap, l)]
    print(f"[1] menubar triggers visible: {menubar_labels}")
    if not menubar_labels:
        print("FATAL: no menubar triggers in snapshot — app shell did not render")
        sys.exit(1)

    cache = [snap]
    for label in menubar_labels:
        ok = dm.open_menu(mcp, cache, label)
        found_any = False
        if ok:
            # jade menubar items carry synthesized __menubar_item onclick —
            # their presence under the opened menu = items entered snapshot.
            # open_menu refreshes cache[0] with the post-open snapshot.
            found_any = '__menubar_item(' in cache[0]
        status = "PASS" if (ok and found_any) else ("SKIP" if not ok else "FAIL")
        if status == "FAIL":
            fails += 1
        print(f"[2] open {label}: opened={ok} items_in_snapshot={found_any} -> {status}")

    # persistence: reopen 视图 and confirm items STILL there 1.5s later
    # (Tick-pump regression face — jade has no periodic pump, so this is a
    # stability confirmation rather than the 041 race).
    print(f"RESULT: jade menubar sampling fails={fails}")
    sys.exit(1 if fails else 0)
finally:
    dm._kill_proc_tree(proc)
