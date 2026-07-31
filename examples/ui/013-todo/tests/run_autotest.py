#!/usr/bin/env python3
"""
Plan 371 Task 16/20: CLI runner for .autotest suites.

Usage:
    cd examples/ui/013-todo/tests
    python run_autotest.py 013-todo.autotest --mode vm

Prerequisites:
    - auto built with ui-iced: cargo build --features ui-iced --bin auto
    - App running: cd examples/ui/013-todo && auto run -r vm
    - MCP server on localhost:9247

Visual regression (Plan 371 Task 20):
    python run_autotest.py --screenshot-baseline   # capture baselines
    python run_autotest.py --screenshot-diff       # compare against baselines
"""

import sys
import os

# Add the tests directory to path so we can import the autotest package
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from autotest import run_suite


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Run .autotest scenarios via MCP")
    parser.add_argument("autotest", nargs="?", default="013-todo.autotest",
                        help="Path to .autotest file (default: 013-todo.autotest)")
    parser.add_argument("--mode", default="vm", choices=["vm", "rust"],
                        help="Rendering mode (controls skip_if behavior)")
    parser.add_argument("--url", default="http://localhost:9247/mcp",
                        help="MCP server URL")
    parser.add_argument("--screenshot-baseline", action="store_true",
                        help="Save baseline screenshots for each scenario (Task 20)")
    parser.add_argument("--screenshot-diff", action="store_true",
                        help="Compare screenshots against baseline after each scenario (Task 20)")
    args = parser.parse_args()

    # Resolve path relative to this script's directory
    test_dir = os.path.dirname(os.path.abspath(__file__))
    path = args.autotest
    if not os.path.isabs(path):
        path = os.path.join(test_dir, path)

    if not os.path.exists(path):
        print(f"Error: {path} not found")
        sys.exit(1)

    # Plan 371 Task 20: choose screenshot mode (diff takes precedence if both set).
    screenshot = None
    if args.screenshot_diff:
        screenshot = "diff"
    elif args.screenshot_baseline:
        screenshot = "baseline"

    results, screenshot_results = run_suite(
        path, mode=args.mode, url=args.url, screenshot=screenshot,
    )

    # Plan 371 Task 20: report screenshot verdicts and let diff mismatches fail.
    screenshot_failed = 0
    if screenshot_results:
        print(f"{'─'*60}")
        print(f"Screenshots ({screenshot}):")
        for sid, verdict in screenshot_results:
            short = verdict.replace("\n", " ")[:100]
            if screenshot == "diff":
                if verdict.startswith("Screenshot matches"):
                    print(f"  ✅ {sid}: {short}")
                elif verdict.startswith("Screenshot DIFFERS"):
                    print(f"  ⚠️  {sid}: {short}")
                    screenshot_failed += 1
                else:
                    print(f"  ❌ {sid}: {short}")
                    screenshot_failed += 1
            else:  # baseline
                print(f"  📸 {sid}: {short}")
        print(f"{'─'*60}\n")

    # Exit code: 0 if all scenarios passed AND no screenshot diff failed.
    failed = sum(1 for r in results if r.status == "FAIL")
    sys.exit(1 if (failed or screenshot_failed) else 0)


if __name__ == "__main__":
    main()
