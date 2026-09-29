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

    def handler_widget(self, app: str, widget: str, handler: str, arg: str = None):
        """Plan 559 W7: (app, widget) sub-component targeting — the inject
        layer dispatches into the named DynamicComponent's handler context
        (namespaced call_handler_for, the onclick pipeline)."""
        payload = {"action": "handler", "app": app, "widget": widget, "handler": handler}
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
            # synthetic-input-cannot-reach-winner blocker bypassed by the
            # channel's MCP injection arm, per 505 acceptance-channel report).
            s.bus("launch\u001f011-calculator")
            s.settle(8)
            shots.append(s.shot("p515-01-calculator-launched"))
        elif name == "p559":
            # Plan 559 T8 (W7): picker click-through e2e. The W6 drop-in
            # fixture module (widgets-declared, no view_name) renders in the
            # GENERIC ConfigEditor; W7 (app, widget) targeting drives the
            # WallpaperPicker's Pick directly; the write lands in the daemon
            # config store (config.at) and the desktop host hot-applies it
            # (551-10/11 wallpaper hot-apply 对照).
            import json as _json
            import urllib.request as _ur

            def _cfg(mid):
                with _ur.urlopen(f"http://127.0.0.1:17701/api/config/{mid}") as r:
                    return _json.load(r)["value"]

            s.handler("shell", "OpenSettingsPanel")
            s.settle(8)  # os-config spawn + vm merged load settle
            s.handler("settings", "SelectModule", "p559-fixture")
            s.settle(3)
            shots.append(s.shot("p559-01-fixture-editor-picker"))
            # Idempotent baseline: reset the fixture to aqua via the daemon
            # before the Pick, so the plum write-through is provable on every
            # re-run (execution-time first run proved aqua→plum change; a
            # persisted plum state would otherwise make before==after).
            _reset = _ur.Request(
                "http://127.0.0.1:17701/api/config/p559-fixture",
                data=_json.dumps({
                    "value": {
                        "cfg_wallpaper": "C:/Users/zhaop/.config/autoos/p559-wallpapers\\aqua.png",
                        "cfg_wallpapers_dir": "C:/Users/zhaop/.config/autoos/p559-wallpapers",
                    }
                }).encode(),
                method="PUT",
                headers={"Content-Type": "application/json"},
            )
            with _ur.urlopen(_reset) as r:
                r.read()
            s.settle(2)
            before = _cfg("p559-fixture").get("cfg_wallpaper", "")
            s.handler_widget(
                "settings", "WallpaperPicker", "Pick",
                "C:/Users/zhaop/.config/autoos/p559-wallpapers\\plum.png",
            )
            s.settle(5)  # fresh GET → editField → PUT → host 400ms poll → apply
            after = _cfg("p559-fixture").get("cfg_wallpaper", "")
            assert "aqua.png" in before, f"p559: baseline reset failed (before={before!r})"
            assert after != before and "plum.png" in after, (
                f"p559: Pick must write through to config.at (before={before!r} after={after!r})"
            )
            shots.append(s.shot("p559-02-pick-hot-apply"))
        elif name == "p709":
            # PLAN-709 T-09：win_rect 验收通道实机腿（AC-10）。bus 动词
            # 确定性摆窗（ToDesk 等远程输入层吞合成输入环境下的唯一摆窗
            # 面），截图块像素对比断言几何生效 + no-op 容错。截图偶发
            # "window size is zero"（冷启动/远程会话窗口期）→ 全拍重试。
            def _shot_retry(name_, tries=6):
                import time as _t
                for attempt in range(tries):
                    try:
                        return s.shot(name_)
                    except RuntimeError as e:
                        if attempt == tries - 1 or "not produced" not in str(e):
                            raise
                        _t.sleep(2.0)

            s.bus("launch011-calculator")
            s.settle(8)
            shots.append(_shot_retry("p709-01-calculator-launched"))
            # 聚焦窗 wid 自 __wm_meta（"free	<N>"）读取——伪窗不占真实
            # wid 段，launch 窗即焦点窗。
            st = s.mcp.text("autoui_state", {"fields": ["__wm_meta"]})
            import re as _re
            m = _re.search('__wm_meta:[ ]*"free[^0-9]*([0-9]+)"', st)
            assert m, f"p709: __wm_meta 不可读: {st[:200]}"
            wid = m.group(1)
            before_png = os.path.join(out_dir, "p709-01-calculator-launched.png")
            before = _png_block_mean(before_png, 940, 380, 420, 320)
            s.bus(f"win_rect	{wid}	940,380,420,320")
            s.settle(4)
            shots.append(_shot_retry("p709-02-after-winrect"))
            after_png = os.path.join(out_dir, "p709-02-after-winrect.png")
            after = _png_block_mean(after_png, 940, 380, 420, 320)
            assert before != after, (
                f"p709: win_rect 后新位块像素应变化（before={before} after={after}）"
            )
            # no-op 容错：未知 wid（999）与坏参不炸桌面（再摆一次可校验通道仍活）。
            s.bus("win_rect	999	1,1,10,10")
            s.bus("win_rect	bad	1,1,10,10")
            s.settle(2)
            s.bus(f"win_rect	{wid}	500,300,380,300")
            s.settle(4)
            shots.append(_shot_retry("p709-03-reposition-after-noops"))
            moved2 = _png_block_mean(
                os.path.join(out_dir, "p709-03-reposition-after-noops.png"),
                500, 300, 380, 300,
            )
            assert moved2 != after, (
                f"p709: no-op 后通道仍活（二次摆窗生效 {moved2} vs {after}）"
            )
        else:
            raise SystemExit(f"unknown scenario: {name}")
        print(f"[{name}] PASS — {len(shots)} shot(s):")
        for p in shots:
            print(f"  {os.path.relpath(p, REPO_ROOT)}")
        return 0
    finally:
        s.close()


def _png_block_mean(path: str, x: int, y: int, w: int, h: int) -> tuple:
    """Minimal PNG reader (8-bit RGB/RGBA, non-interlaced) → block mean color.

    Plan 709 T-09 win_rect acceptance leg: geometry assertion without PIL —
    the window-screenshot PNGs from the desktop host are plain 8-bit RGBA.
    Coordinates are logical (iced) pixels; the host screenshot bakes its own
    scale_factor, so we sample the CENTER block with margins — scale-agnostic.
    """
    import struct as _struct
    import zlib as _zlib

    with open(path, "rb") as f:
        data = f.read()
    assert data[:8] == bytes([0x89]) + b"PNG" + bytes([13, 10, 26, 10]), f"not a png: {path}"
    pos, width, height, bitd, color, interlace = 8, 0, 0, 0, 0, 0
    idat = bytearray()
    while pos < len(data):
        (length,) = _struct.unpack(">I", data[pos:pos + 4])
        ctype = data[pos + 4:pos + 8]
        chunk = data[pos + 8:pos + 8 + length]
        pos += 12 + length
        if ctype == b"IHDR":
            width, height, bitd, color, _c, _f, interlace = _struct.unpack(
                ">IIBBBBB", chunk)
            assert bitd == 8 and interlace == 0, "unsupported png variant"
        elif ctype == b"IDAT":
            idat.extend(chunk)
        elif ctype == b"IEND":
            break
    channels = {0: 1, 2: 3, 6: 4}[color]
    raw = _zlib.decompress(bytes(idat))
    stride = width * channels
    # defilter (paeth etc.) — only standard five filters.
    out = bytearray(height * stride)
    prev = bytearray(stride)
    pos = 0
    for row in range(height):
        ft = raw[pos]
        pos += 1
        line = bytearray(raw[pos:pos + stride])
        pos += stride
        bpp = channels
        if ft == 1:
            for i in range(bpp, stride):
                line[i] = (line[i] + line[i - bpp]) & 0xFF
        elif ft == 2:
            for i in range(stride):
                line[i] = (line[i] + prev[i]) & 0xFF
        elif ft == 3:
            for i in range(stride):
                left = line[i - bpp] if i >= bpp else 0
                line[i] = (line[i] + ((left + prev[i]) >> 1)) & 0xFF
        elif ft == 4:
            for i in range(stride):
                a = line[i - bpp] if i >= bpp else 0
                b = prev[i]
                c = prev[i - bpp] if i >= bpp else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + pr) & 0xFF
        out[row * stride:(row + 1) * stride] = line
        prev = line
    # 逻辑像素 → 截图像素（screenshot 自带 scale，按宽高比近似即可：
    # 采样中心块留边 25%，尺度误差被均值吸收）。
    sx = width / 1280.0  # 桌面逻辑宽缺省 1280；比例探针容差足够。
    sy = height / 800.0
    cx0 = int((x + w * 0.25) * sx)
    cy0 = int((y + h * 0.25) * sy)
    cx1 = int((x + w * 0.75) * sx)
    cy1 = int((y + h * 0.75) * sy)
    cx0, cy0 = max(0, cx0), max(0, cy0)
    cx1, cy1 = min(width, cx1), min(height, cy1)
    rs = gs = bs = n = 0
    for yy in range(cy0, cy1, 4):
        base = yy * stride
        for xx in range(cx0, cx1, 4):
            o = base + xx * channels
            rs += out[o]
            gs += out[o + 1]
            bs += out[o + 2]
            n += 1
    assert n > 0, f"empty block at {x},{y},{w},{h} (png {width}x{height})"
    return (rs // n, gs // n, bs // n)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--scenario", required=True,
                    choices=["drill", "p487", "p496", "p501", "p515", "p559", "p709"])
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
