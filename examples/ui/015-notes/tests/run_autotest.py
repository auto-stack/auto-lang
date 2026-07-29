#!/usr/bin/env python3
"""
Plan 371 Task 16: CLI runner for .autotest suites.

Usage:
    cd examples/ui/015-notes/tests
    python run_autotest.py 015-notes.autotest --mode vm

Prerequisites:
    - auto built with ui-iced: cargo build --features ui-iced --bin auto
    - App running: cd examples/ui/015-notes && auto run -r vm
    - MCP server on localhost:9247
"""

import sys
import os

# Add the tests directory to path so we can import the autotest package
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from autotest import run_suite


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Run .autotest scenarios via MCP")
    parser.add_argument("autotest", nargs="?", default="015-notes.autotest",
                        help="Path to .autotest file (default: 015-notes.autotest)")
    parser.add_argument("--mode", default="vm", choices=["vm", "rust"],
                        help="Rendering mode (controls skip_if behavior)")
    parser.add_argument("--url", default="http://localhost:9247/mcp",
                        help="MCP server URL")
    parser.add_argument("--screenshot-baseline", action="store_true",
                        help="Save baseline screenshots for each scenario (Task 10)")
    parser.add_argument("--screenshot-diff", action="store_true",
                        help="Compare screenshots against baseline after each scenario (Task 10)")
    args = parser.parse_args()

    # Resolve path relative to this script's directory
    test_dir = os.path.dirname(os.path.abspath(__file__))
    path = args.autotest
    if not os.path.isabs(path):
        path = os.path.join(test_dir, path)

    if not os.path.exists(path):
        print(f"Error: {path} not found")
        sys.exit(1)

    results = run_suite(path, mode=args.mode, url=args.url)

    # Task 10: screenshot baseline / diff
    if args.screenshot_baseline or args.screenshot_diff:
        import hashlib
        baseline_dir = os.path.join(os.path.dirname(path), "screenshots", args.mode)
        os.makedirs(baseline_dir, exist_ok=True)

        adapter = None
        for r in results:
            baseline_path = os.path.join(baseline_dir, f"{r.sid}.txt")
            # Capture current screenshot via MCP
            try:
                import requests as req
                resp = req.post(args.url, json={
                    "jsonrpc": "2.0", "method": "tools/call",
                    "params": {"name": "autoui_screenshot", "arguments": {}},
                    "id": 999,
                }, timeout=15)
                data = resp.json()
                screenshot_path = data.get("result", {}).get("content", [{}])[0].get("text", "")
                # Hash the screenshot file for comparison
                if screenshot_path and os.path.exists(screenshot_path.replace("\\\\?\\", "")):
                    real_path = screenshot_path.replace("\\\\?\\", "").strip()
                    with open(real_path, "rb") as f:
                        file_hash = hashlib.md5(f.read()).hexdigest()
                else:
                    file_hash = "no-screenshot"
            except Exception as e:
                file_hash = f"error:{e}"

            if args.screenshot_baseline:
                with open(baseline_path, "w") as f:
                    f.write(file_hash)
                print(f"  📸 Basline saved: {r.sid} → {file_hash[:12]}")
            elif args.screenshot_diff:
                if os.path.exists(baseline_path):
                    with open(baseline_path) as f:
                        baseline_hash = f.read().strip()
                    if baseline_hash == file_hash:
                        print(f"  ✅ Screenshot match: {r.sid}")
                    else:
                        print(f"  ⚠️  Screenshot CHANGED: {r.sid} (baseline={baseline_hash[:12]} vs now={file_hash[:12]})")
                else:
                    print(f"  ❓ No baseline for {r.sid} — run with --screenshot-baseline first")

    # Exit code: 0 if all passed, 1 if any failed
    failed = sum(1 for r in results if r.status == "FAIL")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
