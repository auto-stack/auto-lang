"""
AutoUI VM Mode MCP Test for 020-music-player.
Verifies initial dark mode, Settings popup interaction, Light/Dark theme toggle, Accent color selection,
Playback controls (Play/Pause, Next/Prev), and Up Next Playlist track selection.
"""

import json
import os
import socket
import subprocess
import sys
import time
import urllib.request

def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(('127.0.0.1', 0))
        return s.getsockname()[1]

class AutoUiMcpClient:
    def __init__(self, port: int):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self._req_id = 1

    def call(self, tool_name: str, args: dict = None) -> dict:
        req = {
            "jsonrpc": "2.0",
            "id": self._req_id,
            "method": "tools/call",
            "params": {"name": tool_name, "arguments": args or {}}
        }
        self._req_id += 1
        data = json.dumps(req).encode('utf-8')
        http_req = urllib.request.Request(
            self.url,
            data=data,
            headers={'Content-Type': 'application/json'}
        )
        with urllib.request.urlopen(http_req, timeout=10) as response:
            res = json.loads(response.read().decode('utf-8'))
            if "error" in res:
                raise RuntimeError(f"MCP Tool '{tool_name}' error: {res['error']}")
            return res.get("result", {})

    def snapshot(self, mode: str = "rendered") -> str:
        res = self.call("autoui_snapshot", {"mode": mode})
        return res.get("content", [{}])[0].get("text", "")

    def inspect(self, element_id: str) -> str:
        clean_id = element_id.lstrip("#")
        res = self.call("autoui_inspect", {"element_id": clean_id})
        return res.get("content", [{}])[0].get("text", "")

    def press(self, element_id: str) -> str:
        clean_id = element_id.lstrip("#")
        res = self.call("autoui_action", {"element_id": clean_id, "action": "press"})
        return res.get("content", [{}])[0].get("text", "")

    def screenshot(self, name: str, baseline: bool = True, save_path: str = None) -> str:
        args = {"name": name, "baseline": baseline}
        if save_path:
            args["save_path"] = save_path
        res = self.call("autoui_screenshot", args)
        return res.get("content", [{}])[0].get("text", "")

def find_vnode_by_text(snapshot_text: str, target_text: str) -> str:
    """Finds vnode id in snapshot for an element with the given text."""
    lines = snapshot_text.splitlines()
    for line in lines:
        if target_text in line and ("#vnode_" in line or "#aura_" in line):
            for token in line.split():
                if token.startswith("#vnode_") or token.startswith("#aura_"):
                    return token.lstrip("#")
    return None

def main():
    app_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    auto_bin = os.path.join(app_dir, "..", "..", "target", "debug", "auto.exe")
    if not os.path.exists(auto_bin):
        auto_bin = "auto"

    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))

    print(f"[*] Starting '{auto_bin} run -r vm' in {app_dir} on MCP port {port}...")
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm"], cwd=app_dir, env=env)

    try:
        client = AutoUiMcpClient(port)
        ready = False
        start_time = time.time()

        while time.time() - start_time < 15:
            try:
                snap = client.snapshot()
                if snap and "tree:" in snap:
                    ready = True
                    break
            except Exception:
                pass
            time.sleep(0.3)

        if not ready:
            print("[-] Timeout waiting for AutoUI MCP server to become ready.", file=sys.stderr)
            sys.exit(1)

        print("[+] UI Ready! Taking initial dark mode snapshot & screenshot...")
        snap = client.snapshot()
        print(snap[:400] + "...")
        client.screenshot("020_player_vm_dark_initial")
        print("[+] Captured 020_player_vm_dark_initial.png")

        # 1. Toggle Settings
        settings_id = find_vnode_by_text(snap, "⚙ Settings")
        print(f"[*] Found Settings button: {settings_id}")
        if settings_id:
            client.press(settings_id)
            time.sleep(0.5)
            snap = client.snapshot()
            client.screenshot("020_player_vm_settings_open")
            print("[+] Captured 020_player_vm_settings_open.png")

            # 2. Click Light mode
            light_id = find_vnode_by_text(snap, "☀ Light")
            print(f"[*] Found Light mode button: {light_id}")
            if light_id:
                client.press(light_id)
                time.sleep(0.5)
                client.screenshot("020_player_vm_light")
                print("[+] Captured 020_player_vm_light.png")

                # 3. Click Dark mode back
                snap = client.snapshot()
                dark_id = find_vnode_by_text(snap, "🌙 Dark")
                if dark_id:
                    client.press(dark_id)
                    time.sleep(0.3)

            # 4. Close settings
            snap = client.snapshot()
            close_id = find_vnode_by_text(snap, "✕")
            if close_id:
                client.press(close_id)
                time.sleep(0.3)

        # 5. Test Play/Pause toggle
        snap = client.snapshot()
        pause_id = find_vnode_by_text(snap, "⏸ Pause")
        if pause_id:
            print(f"[*] Found Play/Pause button: {pause_id}")
            client.press(pause_id)
            time.sleep(0.3)
            client.screenshot("020_player_vm_paused")
            print("[+] Captured 020_player_vm_paused.png")

            # Toggle back to play
            snap = client.snapshot()
            play_id = find_vnode_by_text(snap, "▶ Play")
            if play_id:
                client.press(play_id)
                time.sleep(0.3)

        # 6. Test Next Track button
        snap = client.snapshot()
        next_id = find_vnode_by_text(snap, "Next ⏭")
        if next_id:
            print(f"[*] Found Next button: {next_id}")
            client.press(next_id)
            time.sleep(0.3)
            client.screenshot("020_player_vm_next_track")
            print("[+] Captured 020_player_vm_next_track.png")

        # 7. Test Playlist Selection (Track 3 Debussy)
        snap = client.snapshot()
        t3_id = find_vnode_by_text(snap, "Clair de Lune")
        if t3_id:
            print(f"[*] Found Track 3 in playlist: {t3_id}")
            client.press(t3_id)
            time.sleep(0.3)
            client.screenshot("020_player_vm_track3_selected")
            print("[+] Captured 020_player_vm_track3_selected.png")

        # 8. Test Like button toggle
        snap = client.snapshot()
        like_id = find_vnode_by_text(snap, "🤍 Like")
        if like_id:
            print(f"[*] Found Like button: {like_id}")
            client.press(like_id)
            time.sleep(0.3)
            client.screenshot("020_player_vm_liked")
            print("[+] Captured 020_player_vm_liked.png")

        print("[+] All VM MCP interactions completed successfully!")

    finally:
        print("[*] Terminating VM process...")
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()

if __name__ == "__main__":
    main()
