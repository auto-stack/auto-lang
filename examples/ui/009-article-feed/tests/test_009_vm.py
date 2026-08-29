import os
import sys
import time
import socket
import subprocess
import json
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
    
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))

    print(f"[*] Starting VM mode for 009 on MCP port {port}...")
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
        client.screenshot("009_vm_dark_initial")

        # 1. Find Settings button
        snap = client.snapshot()
        print("[*] Snapshot:")
        print(snap)
        settings_id = find_id_by_text(snap, "⚙ Settings")
        print(f"[*] Settings button ID: {settings_id}")
        if settings_id:
            client.press(settings_id)
            time.sleep(0.5)
            print("[+] Settings opened. Capturing Settings Open screenshot...")
            client.screenshot("009_vm_settings_open")

            snap2 = client.snapshot()
            print("[*] Settings open snapshot:")
            print(snap2)

            # 2. Click Coral Accent (2nd accent button)
            # Find the ghost button lines in snapshot
            coral_id = find_id_by_text(snap2, "coral")
            if not coral_id:
                # Find all buttons in snap2
                button_ids = []
                for line in snap2.splitlines():
                    if "button #" in line:
                        for p in line.split():
                            if p.startswith("#"):
                                button_ids.append(p.rstrip(":,{"))
                print(f"[*] Found buttons: {button_ids}")
                # Button list typically: [Settings, Light, Dark, indigo, coral, ocean, sage, amber, ReadMore1, ReadMore2, ReadMore3]
                # If 11 buttons, button_ids[4] is coral!
                if len(button_ids) >= 5:
                    coral_id = button_ids[4] # coral
            if coral_id:
                print(f"[*] Clicking coral accent ({coral_id})...")
                client.press(coral_id)
                time.sleep(0.5)
                client.screenshot("009_vm_coral_accent")

            # 3. Click Light Mode button
            light_id = find_id_by_text(snap2, "Light")
            if light_id:
                print(f"[*] Clicking Light mode ({light_id})...")
                client.press(light_id)
                time.sleep(0.5)
                client.screenshot("009_vm_light_mode")

            # 4. Click Dark Mode button
            snap3 = client.snapshot()
            dark_id = find_id_by_text(snap3, "Dark")
            if dark_id:
                print(f"[*] Clicking Dark mode ({dark_id})...")
                client.press(dark_id)
                time.sleep(0.5)
                client.screenshot("009_vm_back_to_dark")

        print("[+] All VM screenshots captured successfully!")

    finally:
        print("[*] Terminating VM process...")
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()

if __name__ == "__main__":
    main()
