"""
bps-gallery VM-arm click E2E gate (PLAN-685 T-03, G2).

Drives a bps-gallery VM instance (`auto run -r vm`) over the embedded MCP
server and asserts the sidebar card click chain end to end:

    card onclick `.Select(item.id)`  (hop 1, payload-encoded)
      -> BP internal `.Select(id) -> { on_select(id) }`  (hop 2, in-body
         callback re-emit — the PLAN-685 G1 defect site: the stripped
         arg snapshot used to degrade the handler PARAM to its name
         literal, selected_id ended as "id")
      -> App `.SelectBp(id) -> { .selected_id = id }`

Per-card assertions: selected_id == the clicked bp id AND the detail area
actually renders that bp's Spec/Gotchas body (snapshot text probe).

Pre-click state probe doubles as the PLAN-685 T-04 first-start race
detector: the race's broken signature is selected_id == "id" (the same
degenerate literal) with zero clicks. `--runs N` launches N fresh
instances and reports the race count per run (T-04 frequency evidence).

Usage:
    python test_bp_gallery_click.py \
        --auto-bin D:/autostack/.wt/lang-685/auto-lang/target/debug/auto.exe \
        --app-dir D:/autostack/.wt/lang-685/auto-lang/examples/bps-gallery \
        --runs 2

The script kills ONLY the process it spawned (by PID).
"""

import argparse
import json
import os
import re
import socket
import subprocess
import sys
import time
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_vm_mcp import AutoUiMcpClient, pick_free_port  # noqa: E402

# Cards to click: (kind, name) — ≥3 bp ids across ≥2 kinds (T-03 contract).
# Resolve existence against blueprints/ so the gate fails loudly if a bp
# is renamed (blueprint drift = gate drift, by design).
DEFAULT_CARDS = [
    ("dashboard", "overview"),
    ("navigation", "sidebar-nav"),
    ("layout", "gallery-shell"),
]

CARD_BTN_RE = re.compile(r'button #(\S+) "([^"\n]+)\n([^"]*)"')


def discover_bps(app_dir: str) -> dict:
    """bp id -> {spec heading, gotchas marker} from blueprints/<kind>/<name>/."""
    bps = {}
    root = os.path.normpath(os.path.join(app_dir, "..", "..", "blueprints"))
    if not os.path.isdir(root):
        return bps
    for kind in sorted(os.listdir(root)):
        kind_dir = os.path.join(root, kind)
        if not os.path.isdir(kind_dir):
            continue
        for name in sorted(os.listdir(kind_dir)):
            bp_dir = os.path.join(kind_dir, name)
            spec_path = os.path.join(bp_dir, "spec.md")
            gotcha_path = os.path.join(bp_dir, "gotchas.md")
            if not os.path.isfile(spec_path):
                continue
            with open(spec_path, encoding="utf-8") as f:
                spec = f.read()
            # frontmatter-stripped first heading = distinctive body probe
            body = spec.split("+++", 2)[-1]
            heading = next(
                (ln.lstrip("# ").strip() for ln in body.splitlines() if ln.startswith("# ")),
                None,
            )
            gotchas_marker = None
            if os.path.isfile(gotcha_path):
                with open(gotcha_path, encoding="utf-8") as f:
                    gotchas_marker = next(
                        (ln.lstrip("# ").strip() for ln in f if ln.startswith("# ")),
                        None,
                    )
            bps[f"{kind}/{name}"] = {"spec_heading": heading, "gotchas_heading": gotchas_marker}
    return bps


def parse_snapshot_cards(snapshot: str) -> dict:
    """sidebar card button text first line (bp id) -> vnode id."""
    cards = {}
    for vnode_id, first_line, _rest in CARD_BTN_RE.findall(snapshot):
        if re.fullmatch(r"[a-z0-9-]+/[a-z0-9-]+", first_line):
            cards.setdefault(first_line, vnode_id)
    return cards


def read_state(client: AutoUiMcpClient, field: str) -> str:
    text = client.state([field])
    m = re.search(rf'{re.escape(field)}:\s+"?(.*?)"?\s*\(', text)
    return m.group(1) if m else ""


def one_run(auto_bin: str, app_dir: str, cards: list, timeout: int) -> dict:
    """One fresh instance: race probe + per-card click assertions."""
    report = {"race_signature": None, "clicked": [], "ok": False}
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm"], cwd=app_dir, env=env)
    try:
        client = AutoUiMcpClient(port)
        ready = False
        deadline = time.time() + timeout
        while time.time() < deadline:
            try:
                snap = client.snapshot()
                if snap and "tree:" in snap:
                    ready = True
                    break
            except Exception:
                pass
            time.sleep(0.3)
        if not ready:
            report["error"] = "MCP not ready before timeout"
            return report

        # T-04 first-start race probe (zero clicks): healthy first frame has
        # selected_id == "" (the detail falls back to the first item). ANY
        # non-empty value at startup is a degenerate write — "id" exactly is
        # the G1 param-name signature; other values = broken start variant.
        initial_sel = read_state(client, "selected_id")
        report["initial_selected_id"] = initial_sel
        if initial_sel != "":
            report["race_signature"] = (
                f'selected_id == {initial_sel!r} at first frame, zero clicks'
            )

        snapshot = client.snapshot()
        elems = parse_snapshot_cards(snapshot)
        for bp_id in cards:
            vnode = elems.get(bp_id)
            if not vnode:
                report["error"] = f"sidebar card for {bp_id} not found in snapshot"
                return report
            receipt = client.press(vnode)  # harness returns text already
            sel = read_state(client, "selected_id")
            if sel != bp_id:
                report["error"] = (
                    f"press {bp_id}: selected_id == {sel!r} (receipt: {receipt[:200]})"
                )
                return report
            detail = client.snapshot()
            probes = [bp_id]
            heading = discover_cache.get(bp_id, {}).get("spec_heading")
            if heading:
                probes.append(heading)
            missing = [p for p in probes if p not in detail]
            if missing:
                report["error"] = f"press {bp_id}: detail missing {missing}"
                return report
            report["clicked"].append(bp_id)
        report["ok"] = True
        return report
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()


def main():
    global discover_cache
    parser = argparse.ArgumentParser(description="bps-gallery VM click E2E gate (PLAN-685)")
    parser.add_argument("--auto-bin", default="auto", help="Path to auto executable")
    parser.add_argument(
        "--app-dir",
        default=os.path.join("examples", "bps-gallery"),
        help="bps-gallery app directory",
    )
    parser.add_argument("--runs", type=int, default=1, help="Fresh-instance runs")
    parser.add_argument("--timeout", type=int, default=20, help="Seconds to wait for UI startup")
    args = parser.parse_args()

    app_dir = os.path.abspath(args.app_dir)
    all_bps = discover_bps(app_dir)
    discover_cache = all_bps
    cards = []
    for kind, name in DEFAULT_CARDS:
        bp_id = f"{kind}/{name}"
        if bp_id not in all_bps:
            print(f"[-] blueprint {bp_id} missing under blueprints/ — rename drift", file=sys.stderr)
            sys.exit(2)
        cards.append(bp_id)
    kinds_covered = {bp.split("/")[0] for bp in cards}
    if len(kinds_covered) < 2:
        print("[-] contract: cards must span >=2 kinds", file=sys.stderr)
        sys.exit(2)

    races = 0
    for i in range(args.runs):
        print(f"[*] run {i + 1}/{args.runs}: launching fresh VM instance...")
        report = one_run(args.auto_bin, app_dir, cards, args.timeout)
        if report.get("race_signature"):
            races += 1
            print(f"[!] FIRST-START RACE: {report['race_signature']}")
        if not report.get("ok"):
            print(f"[-] run {i + 1} FAILED: {report.get('error')}", file=sys.stderr)
            sys.exit(3)
        print(f"[+] run {i + 1} ok: clicked {report['clicked']} "
              f"(initial selected_id={report['initial_selected_id']!r})")

    print(f"[+] PASS: {args.runs} run(s) x {len(cards)} cards, kinds={sorted(kinds_covered)}, "
          f"first-start race {races}/{args.runs}")


if __name__ == "__main__":
    main()
