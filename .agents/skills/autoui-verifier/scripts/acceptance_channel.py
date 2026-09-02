#!/usr/bin/env python3
"""Plan 505 C: acceptance-channel driver (实机验收通道统一入口).

Boots the real iced desktop host (ui_desktop) in acceptance mode
(AUTOUI_ACCEPTANCE=1) and drives desktop-surface interactions through the
in-process MCP injection tool `autoui_desktop` — the CUA pixel-identity-guard
bypass documented in docs/plans/reports/505-acceptance-channel.md.

Scenarios:
  drill  channel smoke: gear → settings panel mounts → screenshot proves it
  p487   P487-1: gear open panel + dock position hot-switch + Esc self-hide
  p496   P496-1: wallpaper writer (settings SaveWallpaper) + desktop icons
  p501   P501-2: gear → settings → system section → open os-config (launch arm)
  p515   P504-3: real-launch e2e — DesktopBus launch record → 011-calculator

Usage:
    python acceptance_channel.py --scenario drill [--out-dir <dir>]
Prerequisites:
    cargo build --features ui-iced --example ui_desktop
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

REPO_ROOT = os.path.normpath(os.path.join(os.path.dirname(__file__), "..", "..", "..", ".."))
DESKTOP_EXE = os.environ.get(
    "DESKTOP_EXE",
    os.path.join(REPO_ROOT, "target", "debug", "examples", "ui_desktop.exe"),
)


def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class Mcp:
    """Minimal JSON-RPC client for the embedded AutoUI MCP server."""

    def __init__(self, port: int):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self._id = 1

    def call(self, tool: str, args: dict = None) -> dict:
        req = {
            "jsonrpc": "2.0",
            "id": self._id,
            "method": "tools/call",
            "params": {"name": tool, "arguments": args or {}},
        }
        self._id += 1
        http = urllib.request.Request(
            self.url, data=json.dumps(req).encode("utf-8"),
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(http, timeout=15) as resp:
            body = json.loads(resp.read().decode("utf-8"))
        if "error" in body:
            raise RuntimeError(f"{tool}: {body['error']}")
        return body["result"]

    def text(self, tool: str, args: dict = None) -> str:
        result = self.call(tool, args)
        # {content: [{type: text, text: ...}]}
        parts = result.get("content") or []
        return "\n".join(p.get("text", "") for p in parts if p.get("type") == "text")


class DesktopSession:
    """Acceptance-mode desktop host process + MCP client."""

    def __init__(self, out_dir: str, storage_file: str):
        self.port = pick_free_port()
        self.storage = os.path.abspath(storage_file)
        self.out_dir = out_dir
        env = dict(os.environ)
        env["AUTOUI_ACCEPTANCE"] = "1"
        env["AUTOUI_MCP_PORT"] = str(self.port)
        env["AUTO_VM_STORAGE_FILE"] = self.storage
        self.proc = subprocess.Popen(
            [DESKTOP_EXE, "--apps-dir", os.path.join(REPO_ROOT, "examples", "ui")],
            cwd=REPO_ROOT,
            env=env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        self.mcp = Mcp(self.port)
        self._wait_ready()

    def _wait_ready(self, timeout_s: float = 30.0):
        deadline = time.time() + timeout_s
        while time.time() < deadline:
            if self.proc.poll() is not None:
                raise RuntimeError("ui_desktop exited during boot")
            try:
                self.mcp.call("autoui_check")
                return
            except (urllib.error.URLError, ConnectionError, OSError, RuntimeError):
                time.sleep(0.3)
        raise RuntimeError(f"MCP server never became ready on port {self.port}")

    def bus(self, verb: str):
        out = self.mcp.text("autoui_desktop", {"action": "bus", "verb": verb})
        self.settle()
        return out

    def handler(self, app: str, handler: str, arg: str = None):
        payload = {"action": "handler", "app": app, "handler": handler}
        if arg is not None:
            payload["arg"] = arg
        out = self.mcp.text("autoui_desktop", payload)
        self.settle()
        return out

    def settle(self, ticks: int = 3):
        """ServiceTick cadence is ≤400ms; a few ticks cover inject → drain →
        render → screenshot-request round trip."""
        time.sleep(0.5 * ticks)

    def shot(self, name: str) -> str:
        # autoui_screenshot with baseline=true writes tests/screenshots/<name>.png
        # (CWD-relative; host runs with cwd=REPO_ROOT) — then we move it into the
        # scenario evidence dir.
        out = self.mcp.text("autoui_screenshot", {"name": name, "baseline": True})
        produced = os.path.join(REPO_ROOT, "tests", "screenshots", f"{name}.png")
        deadline = time.time() + 3
        while not os.path.isfile(produced) and time.time() < deadline:
            time.sleep(0.2)
        if not os.path.isfile(produced) or os.path.getsize(produced) < 1024:
            raise RuntimeError(f"screenshot not produced for {name}: {out[:200]}")
        dest = os.path.join(self.out_dir, f"{name}.png")
        os.replace(produced, dest)
        return dest

    def close(self):
        try:
            self.proc.terminate()
            self.proc.wait(timeout=5)
        except Exception:
            self.proc.kill()


def write_storage(path: str, entries: dict):
    with open(path, "w", encoding="utf-8") as f:
        json.dump(entries, f)


def run_scenario(name: str, out_dir: str):
    os.makedirs(out_dir, exist_ok=True)
    storage = os.path.join(out_dir, f"{name}-storage.json")
    if os.path.exists(storage):
        os.remove(storage)
    if not os.path.exists(storage):
        write_storage(storage, {})
    s = DesktopSession(out_dir, storage)
    shots = []
    try:
        if name == "drill":
            s.handler("shell", "OpenSettingsPanel")
            shots.append(s.shot("drill-01-settings-panel"))
        elif name == "p487":
            s.handler("shell", "OpenSettingsPanel")
            shots.append(s.shot("p487-01-gear-panel-open"))
            s.handler("settings", "Nav", "dock")
            s.handler("settings", "PickPosition", "top")
            shots.append(s.shot("p487-02-dock-hot-switch-top"))
            s.handler("settings", "Escape")
            shots.append(s.shot("p487-03-esc-panel-hidden"))
        elif name == "p496":
            s.handler("shell", "OpenSettingsPanel")
            s.handler("settings", "Nav", "appearance")
            s.handler("settings", "DraftWallpaper", "#1e3a5f")
            s.handler("settings", "SaveWallpaper")
            shots.append(s.shot("p496-01-wallpaper-writer-applied"))
            s.handler("settings", "Escape")
            s.handler("desktop", "ActivateApp", "011-calculator")
            shots.append(s.shot("p496-02-icon-activate-calculator"))
        elif name == "p501":
            s.handler("shell", "OpenSettingsPanel")
            s.handler("settings", "Nav", "system")
            shots.append(s.shot("p501-01-system-section-osconfig-badge"))
            s.handler("settings", "OpenSystemSettings")
            s.settle(4)
            shots.append(s.shot("p501-02-osconfig-launched"))
        elif name == "p515":
            # Plan 515 G4 C3 (P504-3): real-launch e2e — DesktopBus `launch`
            # record (same drain/execute arm as real shell.at writes; the
            # synthetic-input-cannot-reach-winit blocker bypassed by the
            # channel's MCP injection arm, per 505 acceptance-channel report).
            s.bus("launch\u001f011-calculator")
            s.settle(8)
            shots.append(s.shot("p515-01-calculator-launched"))
        else:
            raise SystemExit(f"unknown scenario: {name}")
        print(f"[{name}] PASS — {len(shots)} shot(s):")
        for p in shots:
            print(f"  {os.path.relpath(p, REPO_ROOT)}")
        return 0
    finally:
        s.close()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--scenario", required=True,
                    choices=["drill", "p487", "p496", "p501", "p515"])
    ap.add_argument("--out-dir", default=None)
    args = ap.parse_args()
    if not os.path.isfile(DESKTOP_EXE):
        raise SystemExit(
            f"desktop host not built: {DESKTOP_EXE}\n"
            "  build once: cargo build --features ui-iced --example ui_desktop"
        )
    out_dir = args.out_dir or os.path.join(
        REPO_ROOT, "docs", "plans", "reports", "assets", "505", args.scenario
    )
    sys.exit(run_scenario(args.scenario, out_dir))


if __name__ == "__main__":
    main()
