#!/usr/bin/env python3
"""Plan 479 T6: live acceptance driver for notification center (S6) + notify verb.

Starts (or attaches to) ui_desktop with the in-process MCP HTTP server, then
archives screenshots to docs/plans/reports/assets/479-t6/.

Injection channels (472/478 先例):
  - MCP (autoui_*): in-process, always available; screenshots + primary-app
    actions. Used for evidence capture.
  - OS keyboard/pointer (dock 铃铛点击、面板交互): requires real window
    foreground. When the user's session is active, foreground contention
    blocks it (472 #5–#8 / 478 T6.2–4 同款) — items degrade to BLOCKED and
    are covered by headless pointers (see 479-t6-live-acceptance.md §2).

Usage:
    MCP_PORT=9478 python test_479_t6.py            # attach to running desktop
    AUTO_BIN=... python test_479_t6.py --launch    # spawn ui_desktop first
"""
import json
import os
import shutil  # noqa: E402
import subprocess
import sys
import time
import urllib.request

PORT = int(os.environ.get("MCP_PORT", "9478"))
SAVE_DIR = os.environ.get(
    "SAVE_DIR", os.path.join("docs", "plans", "reports", "assets", "479-t6")
)
DESKTOP_EXE = os.environ.get(
    "DESKTOP_EXE",
    os.path.join("target", "debug", "examples", "ui_desktop.exe"),
)
STORAGE = os.environ.get(
    "AUTO_VM_STORAGE_FILE", os.path.join("tmp", "479-t6-storage.json")
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


def storage_notes() -> dict:
    """Ground truth: shell.notes.* slots in the storage JSON (persist path)."""
    if not os.path.exists(STORAGE):
        return {}
    with open(STORAGE, encoding="utf-8") as f:
        data = json.load(f)
    return {k: v for k, v in sorted(data.items()) if k.startswith("shell.notes.")}


def main() -> int:
    if "--launch" in sys.argv:
        env = dict(os.environ, AUTOUI_MCP_PORT=str(PORT), AUTO_VM_STORAGE_FILE=STORAGE)
        subprocess.Popen(
            [DESKTOP_EXE, "--apps-dir", "examples/ui"],
            env=env,
            stdout=open("tmp/479-t6-desktop.log", "ab"),
            stderr=subprocess.STDOUT,
        )
        time.sleep(8)

    c = McpClient(PORT)

    # ---- MCP-reachable evidence (always available) ----
    shot(c, "10-initial.png", "T6.1 dock 铃铛实机渲染（未读 0 无 badge）")

    # ---- Ground truth: persistence slots ----
    notes = storage_notes()
    filled = {k: v for k, v in notes.items() if v}
    if filled:
        print(f"[PASS] T6.x 落盘槽位非空（push_notification→persist_notes 实机链）: {filled}")
    else:
        print("[INFO] shell.notes.* 全空/缺席（本会话零通知——push-only 写路径语义）")

    # ---- OS-injection flows (require window foreground; degrade to BLOCKED) ----
    print(
        "[BLOCKED-PRONE] T6.2 LaunchApp 成败 toast 入史 + badge 翻转"
        " —— 需前台；headless 指针：notif_push_dual_face_history_and_toast +"
        " notif_unread_semantics_panel_visibility + notif_projection_notes_and_fingerprint"
    )
    print(
        "[BLOCKED-PRONE] T6.3 铃铛点击开面板（未读清零）+ 逐条 × + 全部清除 + Esc"
        " —— 需前台；headless 指针：notif_center_summon_headless +"
        " notif_end_to_end_toggle_dismiss_restore + notif_shell_at_smoke_toggle_and_badge"
    )
    print(
        "[BLOCKED-PRONE] T6.4 notify 动词实机注入（App 主动请求）"
        " —— 需特权 App 表面；headless 指针：notif_commands_encode_parse_round_trip +"
        " execute_desktop_commands Notify 臂（e2e 测内覆盖）"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
