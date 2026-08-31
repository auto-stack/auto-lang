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

    def press_sequence(self, keys: list) -> str:
        res = self.call("autoui_press_sequence", {"keys": keys})
        return res.get("content", [{}])[0].get("text", "")

    def screenshot(self, name: str, baseline: bool = True, save_path: str = None) -> str:
        args = {"name": name, "baseline": baseline}
        if save_path:
            args["save_path"] = save_path
        res = self.call("autoui_screenshot", args)
        return res.get("content", [{}])[0].get("text", "")

def main():
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../../"))
    auto_bin = os.path.join(root_dir, "target/debug/auto.exe")
    app_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "../"))
    out_dir = os.path.join(app_dir, "src/front/tests/screenshots")
    os.makedirs(out_dir, exist_ok=True)

    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))

    print(f"[*] Starting VM mode for 011 on MCP port {port}...")
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm"], cwd=app_dir, env=env)

    failures = []

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

        print("[+] VM Ready! Capturing initial screenshot (fit window, no in-app header)...")
        time.sleep(0.5)
        client.screenshot("011_vm_dark_initial")

        # 0. Plan 504: in-app title bar / Settings removed (moved to pac.at + os-config).
        snap = client.snapshot()
        if "Settings" in snap or "ExampleHeader" in snap:
            failures.append("in-app header/settings still present in snapshot")
            print("[-] FAIL: snapshot still contains Settings/ExampleHeader")
        else:
            print("[+] OK: no in-app Settings header (Plan 504)")

        # 1. Decimal Evaluation: 3.5 + 1 = 4.5
        print("[*] Performing calculation: 3.5 + 1 = ...")
        client.press_sequence(["C", "3", ".", "5", "+", "1", "="])
        time.sleep(0.4)
        client.screenshot("011_vm_calc_eval")

        # 2. Scientific Mode: 2 * ( 3 + 4 ) = 14
        print("[*] Switching to Scientific mode and evaluating 2 * ( 3 + 4 ) = ...")
        client.press_sequence(["Scientific", "C", "2", "*", "(", "3", "+", "4", ")", "="])
        time.sleep(0.4)
        client.screenshot("011_vm_scientific_mode")

        # 3. Plan 504: math.pow static dispatch — 2 ^ 10 = 1024 (was in-app loop).
        print("[*] Evaluating 2 ^ 10 = 1024 (math.pow static dispatch)...")
        client.press_sequence(["C", "2", "^", "1", "0", "="])
        time.sleep(0.4)
        snap = client.snapshot()
        if "1024" in snap:
            print("[+] OK: 2 ^ 10 = 1024 (math.pow)")
        else:
            failures.append("2 ^ 10 did not yield 1024")
            print("[-] FAIL: 2 ^ 10 result not 1024 in snapshot")
        client.screenshot("011_vm_pow")

        # Switch back to Basic
        client.press_sequence(["Basic", "C"])
        time.sleep(0.3)

        # Copy baseline screenshots to local out_dir
        tests_screenshots_dir = os.path.join(app_dir, "tests/screenshots")
        if os.path.isdir(tests_screenshots_dir):
            for f in os.listdir(tests_screenshots_dir):
                if f.startswith("011_vm_") and f.endswith(".png"):
                    shutil.copy2(os.path.join(tests_screenshots_dir, f), os.path.join(out_dir, f))

        if failures:
            print(f"[-] {len(failures)} assertion(s) failed: {failures}", file=sys.stderr)
            sys.exit(1)
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
