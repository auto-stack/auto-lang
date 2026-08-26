"""
AutoUI VM Mode MCP Test Runner.
Drives an AutoUI application running in VM mode (`auto run -r vm`) via embedded MCP Server.
"""

import argparse
import json
import os
import socket
import subprocess
import sys
import time
import urllib.request
import urllib.error

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
        res = self.call("autoui_inspect", {"element_id": element_id})
        return res.get("content", [{}])[0].get("text", "")

    def type_text(self, element_id: str, text: str, clear_first: bool = True) -> str:
        res = self.call("autoui_type", {"element_id": element_id, "text": text, "clear_first": clear_first})
        return res.get("content", [{}])[0].get("text", "")

    def press(self, element_id: str) -> str:
        res = self.call("autoui_press", {"element_id": element_id})
        return res.get("content", [{}])[0].get("text", "")

    def toggle(self, element_id: str) -> str:
        res = self.call("autoui_toggle", {"element_id": element_id})
        return res.get("content", [{}])[0].get("text", "")

    def keyboard(self, key: str, modifiers: list = None) -> str:
        args = {"key": key}
        if modifiers:
            args["modifiers"] = modifiers
        res = self.call("autoui_keyboard", args)
        return res.get("content", [{}])[0].get("text", "")

    def screenshot(self, name: str, baseline: bool = True, save_path: str = None) -> str:
        args = {"name": name, "baseline": baseline}
        if save_path:
            args["save_path"] = save_path
        res = self.call("autoui_screenshot", args)
        return res.get("content", [{}])[0].get("text", "")

def main():
    parser = argparse.ArgumentParser(description="AutoUI VM Mode MCP Test Driver")
    parser.add_argument("--auto-bin", default="auto", help="Path to auto executable")
    parser.add_argument("--app-dir", required=True, help="Directory containing the AutoUI app (e.g. examples/ui/003-converter)")
    parser.add_argument("--initial-screenshot", default="vm_initial", help="Name for initial screenshot")
    parser.add_argument("--save-dir", default=None, help="Directory to copy screenshots to")
    parser.add_argument("--timeout", type=int, default=15, help="Seconds to wait for UI startup")
    args = parser.parse_args()

    app_dir = os.path.abspath(args.app_dir)
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))

    print(f"[*] Starting '{args.auto_bin} run -r vm' in {app_dir} on MCP port {port}...")
    proc = subprocess.Popen([args.auto_bin, "run", "-r", "vm"], cwd=app_dir, env=env)

    try:
        client = AutoUiMcpClient(port)
        ready = False
        start_time = time.time()

        while time.time() - start_time < args.timeout:
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

        print("[+] UI Ready! Capturing snapshot...")
        snap = client.snapshot()
        print(snap[:500] + ("..." if len(snap) > 500 else ""))

        if args.initial_screenshot:
            print(f"[*] Saving initial screenshot '{args.initial_screenshot}'...")
            res = client.screenshot(args.initial_screenshot)
            print(f"[+] {res}")

    finally:
        print("[*] Terminating VM process...")
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()

if __name__ == "__main__":
    main()
