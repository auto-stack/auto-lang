#!/usr/bin/env python3
"""
Screenshot comparison script for a3ui-replica vs original a2ui demo.
Captures full-page screenshots of corresponding pages and generates
side-by-side comparisons plus a markdown analysis report.
"""

import os
import sys
from pathlib import Path
from playwright.sync_api import sync_playwright

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
BASE_DIR = Path(__file__).parent.parent  # examples/a3ui-replica
OUTPUT_DIR = BASE_DIR / "analysis" / "screenshots"
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

LOCAL_BASE = "http://localhost:3458"
REMOTE_BASE = "https://a2ui-composer.ag-ui.com"

PAGES = [
    {"route": "/",            "name": "create",        "desc": "Home / Create Page"},
    {"route": "/gallery",      "name": "gallery",       "desc": "Gallery Page"},
    {"route": "/basic-catalog", "name": "basic_catalog", "desc": "Basic Components Catalog"},
    {"route": "/custom-catalog", "name": "custom_catalog", "desc": "Custom Components Catalog"},
    {"route": "/icons",        "name": "icons",         "desc": "Icons Page"},
    {"route": "/theater",      "name": "theater",       "desc": "Theater Page"},
]

VIEWPORT = {"width": 1440, "height": 900}

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def capture_page(browser, url: str, out_path: Path):
    page = browser.new_page(viewport=VIEWPORT)
    try:
        page.goto(url, wait_until="networkidle", timeout=60000)
        # Give extra time for animations / lazy content
        page.wait_for_timeout(1500)
        page.screenshot(path=str(out_path), full_page=True)
        print(f"  [OK] {out_path.name}")
    except Exception as e:
        print(f"  [ERR] {out_path.name}: {e}")
        # Try to capture anyway
        try:
            page.screenshot(path=str(out_path), full_page=True)
            print(f"  [OK] {out_path.name} (partial)")
        except:
            pass
    finally:
        page.close()

def create_side_by_side(left_path: Path, right_path: Path, out_path: Path):
    from PIL import Image
    left = Image.open(left_path)
    right = Image.open(right_path)
    # Ensure same height for clean comparison
    max_h = max(left.height, right.height)
    if left.height < max_h:
        left = left.crop((0, 0, left.width, max_h))
    if right.height < max_h:
        right = right.crop((0, 0, right.width, max_h))
    total_w = left.width + right.width
    combined = Image.new("RGB", (total_w, max_h), (255, 255, 255))
    combined.paste(left, (0, 0))
    combined.paste(right, (left.width, 0))
    combined.save(out_path)
    print(f"  [OK] comparison -> {out_path.name}")

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    print("=" * 60)
    print("a3ui-replica vs original a2ui -- screenshot comparison")
    print("=" * 60)

    with sync_playwright() as p:
        browser = p.chromium.launch()

        local_shots = []
        remote_shots = []

        for pg in PAGES:
            name = pg["name"]
            route = pg["route"]
            print(f"\n>> {pg['desc']} ({route})")

            local_url = f"{LOCAL_BASE}{route}"
            remote_url = f"{REMOTE_BASE}{route}"

            local_path = OUTPUT_DIR / f"local_{name}.png"
            remote_path = OUTPUT_DIR / f"remote_{name}.png"

            capture_page(browser, local_url, local_path)
            capture_page(browser, remote_url, remote_path)

            local_shots.append(local_path)
            remote_shots.append(remote_path)

            compare_path = OUTPUT_DIR / f"compare_{name}.png"
            create_side_by_side(local_path, remote_path, compare_path)

        browser.close()

    print(f"\n{'=' * 60}")
    print(f"All screenshots saved to: {OUTPUT_DIR}")
    print("=" * 60)

if __name__ == "__main__":
    main()
