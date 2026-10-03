#!/usr/bin/env python3
"""PLAN-706 follow-up: walk widgets-gallery VM pages via AutoUI MCP.

Usage:
  python .tmp-vm-gallery-walk.py --auto-bin D:/autostack/auto-lang/target/debug/auto.exe \
    --app-dir D:/autostack/auto-os/widgets-gallery
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

# Reuse the verifier client
sys.path.insert(0, r"D:/autostack/auto-lang/.agents/skills/autoui-verifier/scripts")
from test_vm_mcp import AutoUiMcpClient, pick_free_port  # type: ignore


ROUTES = [
    "row", "col", "center", "flex", "alignment", "absolute", "spacing", "sizing",
    "scroll", "position", "responsive", "grid", "grid-span",
    "button", "calendar", "checkbox", "code-editor", "combobox", "datepicker",
    "form", "input", "label", "radiogroup", "select", "slider", "switch",
    "textarea", "toggle", "togglegroup",
    "line-chart", "bar-chart", "area-chart", "donut-chart", "flow-diagram",
    "aspectratio", "avatar", "badge", "card", "carousel", "datatable", "skeleton",
    "table", "treeview", "filetree",
    "alert", "progress", "sonner", "toast", "tooltip",
    "breadcrumb", "command", "menubar", "navigationmenu", "pagination",
    "scrollarea", "separator", "sidebar", "tabs",
    "accordion", "alertdialog", "collapsible", "contextmenu", "dialog", "drawer",
    "dropdownmenu", "hovercard", "popover", "sheet",
    "kitchen-sink",
]


def parse_snapshot(snap: str) -> dict:
    """Extract useful signals from an autoui_snapshot dump."""
    # Common empty/placeholder markers
    empty_markers = [
        "Empty", "placeholder", "unimplemented", "TODO", "not supported",
        "unsupported", "未实现", "占位",
    ]
    lines = snap.splitlines()
    text = "\n".join(lines)
    # Count element kinds
    kinds = re.findall(r"\bkind:(\w+)", text)
    if not kinds:
        kinds = re.findall(r"\((\w+)\s+#", text)
    # Button/text presence
    n_button = len(re.findall(r"\bButton\b|\bbutton\b", text))
    n_text = len(re.findall(r"\bText\b|\btext\b", text))
    n_svg = len(re.findall(r"\bSvg\b|\bsvg\b|\bPath\b|\bpath\b", text))
    # Empty-ish: very short snapshot or only chrome
    empty_hits = [m for m in empty_markers if m.lower() in text.lower()]
    return {
        "len": len(snap),
        "lines": len(lines),
        "kinds_sample": kinds[:20],
        "n_kinds": len(kinds),
        "n_button": n_button,
        "n_text": n_text,
        "n_svg": n_svg,
        "empty_hits": empty_hits,
        "head": snap[:400].replace("\n", " | "),
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--auto-bin", default=r"D:/autostack/auto-lang/target/debug/auto.exe")
    ap.add_argument("--app-dir", default=r"D:/autostack/auto-os/widgets-gallery")
    ap.add_argument("--out", default=r"D:/autostack/auto-lang/.tmp-vm-walk.json")
    ap.add_argument("--timeout", type=int, default=45)
    ap.add_argument("--shot-every", type=int, default=0, help="screenshot every N pages (0=off)")
    args = ap.parse_args()

    app_dir = os.path.abspath(args.app_dir)
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))
    print(f"[*] Starting VM MCP on port {port} in {app_dir}", flush=True)
    proc = subprocess.Popen(
        [args.auto_bin, "run", "-r", "vm"],
        cwd=app_dir,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )

    results = []
    try:
        client = AutoUiMcpClient(port)
        ready = False
        t0 = time.time()
        while time.time() - t0 < args.timeout:
            try:
                snap = client.snapshot()
                if snap and ("tree:" in snap or "vnode" in snap or len(snap) > 80):
                    ready = True
                    break
            except Exception as e:
                pass
            time.sleep(0.4)
        if not ready:
            print("[-] MCP not ready", file=sys.stderr)
            return 1

        home = client.snapshot()
        home_info = parse_snapshot(home)
        results.append({"route": "(home)", **home_info})
        print(f"[+] home snap len={home_info['len']} kinds={home_info['n_kinds']}", flush=True)
        Path(args.out + ".home.txt").write_text(home, encoding="utf-8")

        # Find clickable sidebar links by scanning snapshot for button/link-like ids
        # Heuristic: look for vnode/aura ids near route names
        for i, route in enumerate(ROUTES):
            t = time.time()
            try:
                # Try pressing a node whose text contains the route title-ish name
                # Fall back: snapshot and look for href-like or press by name search
                snap_before = client.snapshot()
                # Search for a pressable containing the route token
                token = route.replace("-", " ")
                # Prefer exact-ish label match in snapshot lines
                target_id = None
                for line in snap_before.splitlines():
                    low = line.lower()
                    if route.lower() in low or token in low:
                        m = re.search(r"#(vnode_\d+|aura_\d+)", line)
                        if m:
                            target_id = m.group(1)
                            break
                        m2 = re.search(r"id[=:]\s*\"?(\d+)\"?", line)
                        if m2:
                            target_id = m2.group(1)
                            break
                nav_ok = False
                if target_id:
                    try:
                        client.press(target_id if target_id.startswith("#") else f"#{target_id}")
                        nav_ok = True
                    except Exception as e:
                        nav_err = str(e)
                time.sleep(0.25)
                snap = client.snapshot()
                info = parse_snapshot(snap)
                # Title-ish first 120 chars of non-empty content
                rec = {
                    "route": route,
                    "nav_id": target_id,
                    "nav_ok": nav_ok,
                    "dt": round(time.time() - t, 2),
                    **info,
                }
                results.append(rec)
                flag = ""
                if info["len"] < 200 or info["empty_hits"]:
                    flag = " ** SUSPECT **"
                print(
                    f"[{i+1}/{len(ROUTES)}] {route:16s} len={info['len']:5d} "
                    f"btn={info['n_button']:3d} svg={info['n_svg']:2d} "
                    f"empty={info['empty_hits']}{flag}",
                    flush=True,
                )
                if args.shot_every and (i % args.shot_every == 0):
                    try:
                        client.screenshot(f"vm_walk_{route}")
                    except Exception:
                        pass
            except Exception as e:
                results.append({"route": route, "error": str(e)})
                print(f"[{i+1}] {route} ERROR {e}", flush=True)

    finally:
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except Exception:
            proc.kill()

    Path(args.out).write_text(json.dumps(results, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"[*] wrote {args.out} ({len(results)} rows)", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
