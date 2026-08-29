import os
import sys
import time
import socket
import subprocess
import json
import shutil
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

def find_id_by_text(snapshot_str: str, text: str) -> str:
    for line in snapshot_str.splitlines():
        if text in line:
            parts = line.split()
            for part in parts:
                if part.startswith("#"):
                    return part.rstrip(":,{")
    return None

def main():
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../../"))
    auto_bin = os.path.join(root_dir, "target/debug/auto.exe")
    app_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "../"))
    out_dir = os.path.join(app_dir, "src/front/tests/screenshots")
    os.makedirs(out_dir, exist_ok=True)
    
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))

    print(f"[*] Starting VM mode for 008 on MCP port {port}...")
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
            print("[-] Timeout waiting for AutoUI MCP server.", file=sys.stderr)
            sys.exit(1)

        print("[+] VM Ready! Capturing Initial Dark screenshot...")
        client.screenshot("008_vm_dark_initial")

        # 1. Open Settings
        snap = client.snapshot()
        settings_btn_id = find_id_by_text(snap, "⚙ Settings")
        print(f"[*] Settings button id: {settings_btn_id}")
        if settings_btn_id:
            client.press(settings_btn_id)
            time.sleep(0.4)
            client.screenshot("008_vm_settings_open")

        # 2. Select Coral Accent
        snap = client.snapshot()
        color_btn_ids = []
        for line in snap.splitlines():
            if 'button #' in line and '""' in line:
                for p in line.split():
                    if p.startswith("#"):
                        color_btn_ids.append(p.rstrip(":,{"))
        print(f"[*] Found color buttons: {color_btn_ids}")
        if len(color_btn_ids) >= 2:
            coral_id = color_btn_ids[1]
            print(f"[*] Clicking coral accent ({coral_id})...")
            client.press(coral_id)
            time.sleep(0.4)
            client.screenshot("008_vm_coral_accent")

        # 3. Theme Light
        snap = client.snapshot()
        light_btn_id = find_id_by_text(snap, "☀️ Light")
        print(f"[*] Light button id: {light_btn_id}")
        if light_btn_id:
            client.press(light_btn_id)
            time.sleep(0.4)
            client.screenshot("008_vm_light_mode")

        # 4. Theme Dark
        snap = client.snapshot()
        dark_btn_id = find_id_by_text(snap, "🌙 Dark")
        print(f"[*] Dark button id: {dark_btn_id}")
        if dark_btn_id:
            client.press(dark_btn_id)
            time.sleep(0.4)
            client.screenshot("008_vm_back_to_dark")

        # Copy screenshots
        for f in os.listdir("."):
            if f.startswith("008_vm_") and f.endswith(".png"):
                shutil.copy(f, os.path.join(out_dir, f))
                os.remove(f)

        print("[+] All VM screenshots captured successfully!")

    finally:
        print("[*] Terminating VM process...")
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except Exception:
            proc.kill()

if __name__ == "__main__":
    main()
