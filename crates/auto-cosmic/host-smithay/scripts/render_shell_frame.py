#!/usr/bin/env python3
"""Plan 509 Step 5 driver: shell first-frame PNG via the production chain.

Boots the real iced desktop host (`ui_desktop` example, Plan 505 acceptance
channel) in the current environment (WSL2/WSLg for Stage 1), waits for the
desktop shell surface (dock + wallpaper) to settle, then requests a baseline
screenshot through the in-process MCP server. The produced PNG is moved to
`--out` (default: repo `docs/plans/reports/assets/509/shell-first-frame.png`)
for the smithay host to import as its Stage-1 static texture.

Usage (from repo root, inside WSL):
    python3 crates/auto-cosmic/host-smithay/scripts/render_shell_frame.py \
        [--desktop-exe <path>] [--out <png>]
"""

import argparse
import os
import shutil
import sys
import time

REPO_ROOT = os.path.normpath(
    os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")
)
ACCEPTANCE = os.path.join(
    REPO_ROOT, ".agents", "skills", "autoui-verifier", "scripts", "acceptance_channel.py"
)
sys.path.insert(0, os.path.dirname(ACCEPTANCE))

import acceptance_channel  # noqa: E402  (shared 505 machinery)

SHOT_NAME = "509-shell-first-frame"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--desktop-exe",
        default=os.path.join("target-509", "debug", "examples", "ui_desktop"),
        help="ui_desktop binary (build first: cargo build -p auto-lang "
        "--features ui-iced --example ui_desktop)",
    )
    parser.add_argument(
        "--out",
        default=os.path.join(
            REPO_ROOT, "docs", "plans", "reports", "assets", "509", "shell-first-frame.png"
        ),
    )
    args = parser.parse_args()

    # DesktopSession reads the module-level DESKTOP_EXE at spawn time.
    acceptance_channel.DESKTOP_EXE = os.path.abspath(args.desktop_exe)
    storage = os.path.join(REPO_ROOT, "tmp", "509-render-driver-storage.json")

    session = acceptance_channel.DesktopSession(
        out_dir=os.path.dirname(args.out), storage_file=storage
    )
    try:
        # Shell surface is booted when autoui_check passes; give the first
        # frames a moment to land (clock toast / wallpaper writer settle).
        time.sleep(2.0)
        out = session.mcp.text("autoui_screenshot", {"name": SHOT_NAME, "baseline": True})
        print(f"[509-driver] autoui_screenshot -> {out.strip()}")
    finally:
        session.proc.terminate()
        try:
            session.proc.wait(timeout=5)
        except Exception:
            session.proc.kill()

    produced = os.path.join(REPO_ROOT, "tests", "screenshots", f"{SHOT_NAME}.png")
    if not os.path.isfile(produced):
        print(f"[509-driver] FAIL: {produced} not produced", file=sys.stderr)
        return 1
    os.makedirs(os.path.dirname(args.out), exist_ok=True)
    shutil.move(produced, args.out)
    print(f"[509-driver] shell first frame -> {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
