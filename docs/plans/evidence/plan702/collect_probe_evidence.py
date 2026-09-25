"""PLAN-702 T-06: AC-01/AC-02 evidence collector for the 032 probe app.

Flow:
  1. Start a local HTTP server that answers POST /slow after ~65s (and /fast
     immediately, unused by the app but handy for preflight).
  2. Launch `auto run -r vm` on examples/ui/032-handler-async-probe with an
     MCP port; wait for the MCP endpoint.
  3. Press "Start 65s request" (Load handler parks; old driver = UI frozen +
     30s timeout crash of the handler).
  4. During the wait window: press Bump repeatedly, read state (bumps must
     advance immediately), confirm ticks advance (AppTick alive) -> AC-01.
  5. Wait for result to land -> AC-02 (model value == server body).
  6. Dump stderr log assertions: zero "[VM-HANDLER] ... timed out",
     "[VM-PARKED] parked" and "resumed to completion" present.

Usage:
  python collect_probe_evidence.py --auto-bin <path-to-auto.exe> \
      --out-dir docs/plans/evidence/plan702
"""

import argparse
import json
import os
import socket
import subprocess
import sys
import threading
import time
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

SLOW_SECONDS = 65
SLOW_BODY = '{"ok":true,"slow":702}'

class SlowHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0) or 0)
        if length:
            self.rfile.read(length)
        if self.path.startswith("/slow"):
            print(f"[slow-server] /slow hit, sleeping {SLOW_SECONDS}s", flush=True)
            time.sleep(SLOW_SECONDS)
            body = SLOW_BODY.encode()
        else:
            body = b'{"ok":true,"fast":1}'
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *a):
        pass


def pick_free_port():
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getnameinfo(s.getsockname(), socket.NI_NUMERICHOST)[1] if False else s.getsockname()[1]


def mcp_ready(port, timeout_s=120):
    url = f"http://127.0.0.1:{port}/mcp"
    deadline = time.time() + timeout_s
    req_body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}).encode()
    while time.time() < deadline:
        try:
            req = urllib.request.Request(url, data=req_body, headers={"Content-Type": "application/json"})
            with urllib.request.urlopen(req, timeout=3):
                return True
        except Exception:
            time.sleep(1.0)
    return False


def mcp_call(port, tool, args=None, timeout=30):
    req_body = json.dumps({"jsonrpc": "2.0", "id": 99, "method": "tools/call",
                           "params": {"name": tool, "arguments": args or {}}}).encode()
    req = urllib.request.Request(f"http://127.0.0.1:{port}/mcp", data=req_body,
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        res = json.loads(resp.read().decode())
    if "error" in res:
        raise RuntimeError(f"MCP {tool} error: {res['error']}")
    content = res.get("result", {}).get("content", [{}])
    return content[0].get("text", "")


def state_fields(port):
    text = mcp_call(port, "autoui_state")
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return {"_raw": text}


def press_button(port, snapshot_text, label):
    """Find the element id whose label matches, then autoui_action press."""
    # snapshot text is a rendered tree; buttons appear as their label with an id
    import re
    for line in snapshot_text.splitlines():
        if label in line:
            m = re.search(r"#([A-Za-z0-9_:\-]+)", line)
            if m:
                return m.group(1)
    return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--auto-bin", required=True)
    ap.add_argument("--app-dir", default=os.path.join("examples", "ui", "032-handler-async-probe"))
    ap.add_argument("--out-dir", default="docs/plans/evidence/plan702")
    ap.add_argument("--wait-s", type=int, default=SLOW_SECONDS, help="slow endpoint delay")
    args = ap.parse_args()
    globals()["SLOW_SECONDS"] = args.wait_s

    os.makedirs(args.out_dir, exist_ok=True)
    log_path = os.path.join(args.out_dir, "probe_vm_stderr.log")
    report_path = os.path.join(args.out_dir, "probe_ac01_ac02_report.json")

    # 1. slow server
    server = ThreadingHTTPServer(("127.0.0.1", 7072), SlowHandler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    print("[*] slow server on :7072")

    # 2. launch app
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))
    log_f = open(log_path, "w", encoding="utf-8", errors="replace")
    proc = subprocess.Popen([args.auto_bin, "run", "-r", "vm"], cwd=args.app_dir,
                            env=env, stdout=log_f, stderr=subprocess.STDOUT)
    print(f"[*] app pid={proc.pid} mcp={port}")
    try:
        if not mcp_ready(port):
            print("[-] MCP never became ready", flush=True)
            return 2
        print("[+] MCP ready")
        time.sleep(2.0)  # let first frames render

        snap = mcp_call(port, "autoui_snapshot", {"mode": "rendered"})
        with open(os.path.join(args.out_dir, "probe_snapshot_initial.txt"), "w", encoding="utf-8") as f:
            f.write(snap)
        load_id = press_button(port, snap, "Start 65s request")
        bump_id = press_button(port, snap, "Bump")
        print(f"[*] load_id={load_id} bump_id={bump_id}")
        if not load_id or not bump_id:
            print("[-] buttons not found in snapshot")
            return 2

        st0 = state_fields(port)
        print(f"[*] initial state: {st0}")

        # 3. fire the parking handler
        t_fire = time.time()
        mcp_call(port, "autoui_action", {"element_id": load_id, "action": "press"})
        print("[+] Load pressed — handler should park and RETURN immediately")

        # 4. AC-01 window: interactivity during the wait
        samples = []
        bumps_seq = []
        ticks_seq = []
        wait_s = args.wait_s
        for i in range(wait_s):  # one probe per second
            time.sleep(1.0)
            mcp_call(port, "autoui_action", {"element_id": bump_id, "action": "press"})
            st = state_fields(port)
            bumps = st.get("bumps")
            ticks = st.get("ticks")
            bumps_seq.append(bumps)
            ticks_seq.append(ticks)
            samples.append({"t": round(time.time() - t_fire, 1), "bumps": bumps, "ticks": ticks})
            if i in (4, 30, wait_s - 5):
                print(f"[*] t+{i}s bumps={bumps} ticks={ticks}")

        # 5. AC-02: result lands after slow response
        final = None
        deadline = time.time() + 30
        while time.time() < deadline:
            st = state_fields(port)
            r = st.get("result")
            if r and r != "(idle)":
                final = st
                break
            time.sleep(1.0)
        t_done = time.time() - t_fire
        print(f"[*] final state after {t_done:.1f}s: {final}")

        # 6. log assertions
        with open(log_path, encoding="utf-8", errors="replace") as f:
            log_text = f.read()
        timeouts = log_text.count("timed out in call_fn_by_name")
        parked = "parked (event Load" in log_text
        resumed = "resumed to completion" in log_text

        bumps_ok = all(isinstance(b, int) for b in bumps_seq if b is not None) and \
            len(set(b for b in bumps_seq if b is not None)) >= wait_s - 3
        ticks_ok = any(isinstance(t, int) and t > 0 for t in ticks_seq)

        report = {
            "ac01_interactivity": {
                "bumps_sequence": bumps_seq,
                "ticks_sequence": ticks_seq[:10] + ["..."] + ticks_seq[-5:],
                "bumps_advanced_every_probe": bumps_ok,
                "ticks_alive": ticks_ok,
            },
            "ac02_result": final,
            "log_assertions": {
                "busy_wait_timeouts": timeouts,
                "parked_logged": parked,
                "resumed_logged": resumed,
            },
            "wall_clock_to_result_s": round(t_done, 1),
        }
        with open(report_path, "w", encoding="utf-8") as f:
            json.dump(report, f, indent=2, ensure_ascii=False)
        print(json.dumps(report, indent=2, ensure_ascii=False))

        ok = bumps_ok and ticks_ok and timeouts == 0 and parked and resumed and final is not None
        print("[+] AC-01/AC-02 PASS" if ok else "[-] AC-01/AC-02 FAIL")
        return 0 if ok else 1
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
        log_f.close()
        server.shutdown()


if __name__ == "__main__":
    sys.exit(main())
