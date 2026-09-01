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

    def type_text(self, element_id: str, text: str, clear_first: bool = True) -> str:
        clean_id = element_id.lstrip("#")
        res = self.call("autoui_type", {"element_id": clean_id, "text": text, "clear_first": clear_first})
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

    print(f"[*] Starting VM mode for 010 on MCP port {port}...")
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
        client.screenshot("010_vm_dark_initial")

        # Plan 506: in-app title bar / Settings removed (ExampleHeader retired —
        # title/theme/accent moved to pac.at + os-config, host chrome carries them).
        failures = []
        snap = client.snapshot()
        if "Settings" in snap or "ExampleHeader" in snap:
            failures.append("in-app header/settings still present in snapshot")
            print("[-] FAIL: snapshot still contains Settings/ExampleHeader")
        else:
            print("[+] OK: no in-app Settings header (Plan 506)")

        # 1. Inspect tree & inputs
        print("[*] Initial Snapshot:\n" + snap)

        # 2. Type into Name, Email, Message
        input_ids = []
        for line in snap.splitlines():
            if "input #" in line or "text_input #" in line or "textarea #" in line:
                for p in line.split():
                    if p.startswith("#"):
                        input_ids.append(p.rstrip(":,{"))
        print(f"[*] Found input/textarea elements: {input_ids}")

        if len(input_ids) >= 3:
            print("[*] Typing Name...")
            client.type_text(input_ids[0], "Alice Smith")
            time.sleep(0.3)
            print("[*] Typing Email...")
            client.type_text(input_ids[1], "alice@example.com")
            time.sleep(0.3)
            print("[*] Typing Message...")
            client.type_text(input_ids[2], "I would like to inquire about enterprise support options.")
            time.sleep(0.3)
            print("[+] Captured typed screenshot...")
            client.screenshot("010_vm_typed")
        else:
            failures.append(f"expected 3 inputs, found {len(input_ids)}")
            print(f"[-] FAIL: expected 3 inputs, found {len(input_ids)}")

        # 3. Click "Send Message" button
        snap_typed = client.snapshot()
        send_btn_id = find_id_by_text(snap_typed, "Send Message")
        if send_btn_id:
            print(f"[*] Clicking Send Message ({send_btn_id})...")
            client.press(send_btn_id)
            time.sleep(0.5)
            print("[+] Captured submitted screenshot...")
            client.screenshot("010_vm_submitted")

            # Submitted confirmation state must flip.
            snap_sub = client.snapshot()
            if "Message sent" in snap_sub or "Thank you" in snap_sub:
                print("[+] OK: submitted confirmation visible")
            else:
                failures.append("submitted confirmation not visible after Send Message")
                print("[-] FAIL: submitted confirmation not visible")
        else:
            failures.append("Send Message button not found")
            print("[-] FAIL: Send Message button not found")

        # Copy baseline screenshots to local out_dir
        tests_screenshots_dir = os.path.join(app_dir, "tests/screenshots")
        if os.path.isdir(tests_screenshots_dir):
            for f in os.listdir(tests_screenshots_dir):
                if f.startswith("010_vm_") and f.endswith(".png"):
                    shutil.copy2(os.path.join(tests_screenshots_dir, f), os.path.join(out_dir, f))

        if failures:
            print(f"[-] {len(failures)} assertion(s) failed: {failures}", file=sys.stderr)
            sys.exit(1)
        print("[+] All VM assertions passed!")

    finally:
        print("[*] Terminating VM process...")
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()

if __name__ == "__main__":
    main()
