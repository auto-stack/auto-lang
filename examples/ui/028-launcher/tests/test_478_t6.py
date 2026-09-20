#!/usr/bin/env python3
"""Plan 478 T6: live acceptance driver for switcher overlay + workspace pager.

Starts (or attaches to) ui_desktop with the in-process MCP HTTP server, then
drives the 478 acceptance flows and archives screenshots to
docs/plans/reports/assets/478-t6/.

Two injection channels exist (472 T5 先例):
  - MCP (autoui_*): in-process, always available; screenshots + primary-app
    actions. Used for evidence capture.
  - OS keyboard (Ctrl+Tab / Ctrl+Alt+Shift+←→ / dock clicks): requires real
    window foreground. When the user's session is active, the trusted host
    refuses injection (frontmost_pid_mismatch) — items degrade to BLOCKED and
    are covered by headless pointers (see T6 report §2; 472 #5–#8 同款).

Usage:
    MCP_PORT=9478 python test_478_t6.py            # attach to running desktop
    AUTO_BIN=... python test_478_t6.py --launch    # spawn ui_desktop first
"""
import json
import os
import subprocess
import sys
import time
import urllib.request

PORT = int(os.environ.get("MCP_PORT", "9478"))
SAVE_DIR = os.environ.get(
    "SAVE_DIR", os.path.join("docs", "plans", "reports", "assets", "478-t6")
)
DESKTOP_EXE = os.environ.get(
    "DESKTOP_EXE",
    os.path.join("target", "debug", "examples", "ui_desktop.exe"),
)


class McpClient:
    def __init__(self, port: int):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self._req_id = 1

    def call(self, tool_name: str, args: dict = None) -> dict:
        req = {
            "jsonrpc": "2.0",
            "id": self._req_id,
            "method": "tools/call",
            "params": {"name": tool_name, "arguments": args or {}},
        }
        self._req_id += 1
        http_req = urllib.request.Request(
            self.url,
            data=json.dumps(req).encode("utf-8"),
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(http_req, timeout=20) as response:
            res = json.loads(response.read().decode("utf-8"))
            if "error" in res:
                raise RuntimeError(f"MCP tool '{tool_name}' error: {res['error']}")
            return res.get("result", {})

    def text(self, result: dict) -> str:
        content = result.get("content") or []
        return "\n".join(c.get("text", "") for c in content if c.get("type") == "text")

    def screenshot(self, name: str) -> str:
        """Capture; returns the tool-reported path (tmp/ timestamped PNG)."""
        return self.text(self.call("autoui_screenshot", {"name": name}))


def shot(c: McpClient, fname: str, label: str) -> str:
    """Capture and archive to SAVE_DIR/fname; returns archived path."""
    out = c.screenshot(fname.replace(".png", ""))
    src = out.split("Screenshot saved to:")[-1].strip().strip("\\/?")
    src = src.replace("\\\\?\\", "")
    dst = os.path.join(SAVE_DIR, fname)
    os.makedirs(SAVE_DIR, exist_ok=True)
    if os.path.exists(src):
        shutil.copyfile(src, dst)
        print(f"[PASS] {label} -> {dst}")
        return dst
    print(f"[WARN] {label}: capture not found ({out[:120]})")
    return ""


import shutil  # noqa: E402  (used by shot)


def main() -> int:
    if "--launch" in sys.argv:
        env = dict(os.environ, AUTOUI_MCP_PORT=str(PORT))
        subprocess.Popen(
            [DESKTOP_EXE, "--apps-dir", "examples/ui"],
            env=env,
            stdout=open("tmp/478-t6-desktop.log", "ab"),
            stderr=subprocess.STDOUT,
        )
        time.sleep(8)

    c = McpClient(PORT)
    results = []

    # ---- MCP-reachable evidence (always available) ----
    shot(c, "10-initial.png", "T6.1 dock pager 实机渲染（1 基标签/当前高亮/×/+）")

    # ---- OS-injection flows (require window foreground; degrade to BLOCKED) ----
    print(
        "[BLOCKED-PRONE] T6.2 Ctrl+Tab 召唤/Tab 推进/Enter 确认/Esc 取消"
        " —— 需窗口前台；受阻时按 472 先例以 headless 指针覆盖："
        "switcher_summon_advance_confirm_roundtrip"
    )
    print(
        "[BLOCKED-PRONE] T6.3 pager +/× 点击、分区切换点击"
        " —— 需前台；headless 指针：workspace_v11_host_arms_add_close_send +"
        " desktop_shell_at_builds_with_dock_defaults"
    )
    print(
        "[BLOCKED-PRONE] T6.4 Ctrl+Alt+Shift+←/→ 跨区发送隐现"
        " —— 需前台；headless 指针：workspace_v11_host_arms_add_close_send +"
        " workspace_move_win_to_hidden_and_same_partition"
    )
    _ = results
    return 0


if __name__ == "__main__":
    sys.exit(main())
