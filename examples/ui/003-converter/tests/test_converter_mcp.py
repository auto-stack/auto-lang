import subprocess
import sys
import time
import os
import re
import socket
import struct
import requests

def pick_free_port(start=9247):
    for port in range(start, start + 100):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            if s.connect_ex(("127.0.0.1", port)) != 0:
                return port
    raise RuntimeError("No free port")

AUTO_BIN = os.path.normpath(
    os.path.join(os.path.dirname(__file__), "..", "..", "..", "..", "target", "debug", "auto.exe")
)
CONVERTER_PROJECT = os.path.normpath(
    os.path.join(os.path.dirname(__file__), "..")
)

class McpClient:
    def __init__(self, url):
        self.url = url
        self.req_id = 0

    def call(self, tool_name, **arguments):
        self.req_id += 1
        resp = requests.post(self.url, json={
            "jsonrpc": "2.0", "method": "tools/call",
            "params": {"name": tool_name, "arguments": arguments},
            "id": self.req_id,
        }, timeout=15)
        data = resp.json()
        if "error" in data:
            raise RuntimeError(f"MCP error: {data['error']}")
        content = data.get("result", {}).get("content", [])
        return content[0]["text"] if content else ""

    def snapshot(self):
        return self.call("autoui_snapshot")

    def state(self, *fields):
        return self.call("autoui_state", fields=list(fields) if fields else [])

    def vtree(self):
        return self.call("autoui_vtree")

    def find(self, **kwargs):
        return self.call("autoui_find", **kwargs)

    def type_text(self, element_id, text, clear_first=True):
        return self.call("autoui_type", element_id=element_id, text=text, clear_first=clear_first)

    def screenshot(self, name="", baseline=False, diff=False):
        return self.call("autoui_screenshot", name=name, baseline=baseline, diff=diff)

def main():
    mcp_port = pick_free_port()
    mcp_url = f"http://127.0.0.1:{mcp_port}/mcp"
    env = {**os.environ, "AUTOUI_MCP_PORT": str(mcp_port)}

    print(f"Starting auto run -r vm on port {mcp_port} in {CONVERTER_PROJECT}...")
    proc = subprocess.Popen(
        [AUTO_BIN, "run", "-r", "vm"],
        cwd=CONVERTER_PROJECT,
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

    client = McpClient(mcp_url)

    connected = False
    for _ in range(40):
        time.sleep(0.5)
        try:
            r = requests.post(mcp_url, json={"jsonrpc": "2.0", "method": "tools/list", "params": {}, "id": 1}, timeout=2)
            if r.status_code == 200:
                connected = True
                print("Connected to AutoUI MCP Server!")
                break
        except Exception:
            pass

    if not connected:
        proc.kill()
        print("Failed to connect to AutoUI MCP server.")
        sys.exit(1)

    try:
        # Wait for the iced window to render its first frame
        print("Waiting for UI to render...")
        rendered = False
        snap = ""
        for i in range(25):
            time.sleep(1)
            try:
                snap = client.snapshot()
                if "aura_" in snap or "Temperature" in snap:
                    print(f"UI rendered after {i + 1}s!")
                    rendered = True
                    break
            except Exception as e:
                print(f"Snapshot poll: {e}")

        print("\n--- Snapshot ---")
        print(snap[:500] + "...")

        print("\n--- Initial State ---")
        state = client.state()
        print(state)

        print("\n--- Live VTree ---")
        vtree = client.vtree()
        print(vtree[:500] + "...")

        print("\n--- Finding Input Elements ---")
        inputs = client.find(kind="input")
        print(inputs)

        # Take initial screenshot
        print("\n--- Capturing Initial Screenshot (VM) ---")
        ss1 = client.screenshot(name="converter_vm_initial", baseline=True)
        print(ss1)

        # Plan 506: window:"fit" — the independent VM window must shrink to
        # content size (far below the 1293x836 default). The screenshot PNG's
        # pixel size IS the window size (iced window capture).
        def png_size(path):
            with open(path, "rb") as fh:
                head = fh.read(26)
            w, h = struct.unpack(">II", head[16:24])
            return w, h

        shot_path = os.path.join(CONVERTER_PROJECT, "src", "front", "tests", "screenshots", "converter_vm_initial.png")
        if os.path.isfile(shot_path):
            w, h = png_size(shot_path)
            print(f"--- Fit window size: {w}x{h} ---")
            if w >= 900 or h >= 900:
                print(f"FAIL: fit window not shrunk ({w}x{h} vs default 1293x836)")
                sys.exit(1)
            print("OK: fit window shrunk to content size (Plan 506)")
        else:
            print("FAIL: initial screenshot not found for fit assertion")
            sys.exit(1)

        # Find input element IDs from snapshot
        # Plan 512 S7：快照 id 双 scheme 兼容——首帧前回退路径出 aura_N
        # （源模板），渲染完成后出 vnode_N（实况树）；何时切换取决于首帧
        # 时序（机器负载敏感），两种 id autoui_type 均受理。
        inputs_matched = re.findall(r'input\s+#((?:aura|vnode)_\d+)', snap)
        print(f"Matched input IDs: {inputs_matched}")

        if len(inputs_matched) >= 2:
            celsius_id = inputs_matched[0]      # aura_6
            fahrenheit_id = inputs_matched[1]   # aura_9

            print(f"\n--- Testing Decimal Conversion: Typing '323' into Fahrenheit ({fahrenheit_id}) ---")
            res = client.type_text(fahrenheit_id, "323", clear_first=True)
            print(f"Type result: {res}")
            time.sleep(1)

            # Check snapshot after typing
            snap2 = client.snapshot()
            print(f"\nSnapshot after typing 323 into F:\n{snap2}")

            print("\n--- Capturing Converted Screenshot (VM) ---")
            ss2 = client.screenshot(name="converter_vm_decimal", baseline=True)
            print(ss2)
        else:
            print("Could not match 2 input IDs from snapshot!")

    finally:
        print("Stopping VM process...")
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except Exception:
            proc.kill()

if __name__ == "__main__":
    main()
