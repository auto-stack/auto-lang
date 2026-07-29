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

    # Exit code: 0 if all passed, 1 if any failed
    failed = sum(1 for r in results if r.status == "FAIL")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
