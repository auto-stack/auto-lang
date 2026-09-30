#!/usr/bin/env python3
"""Verify every widgets-gallery route in the live VM renderer.

The route inventory is derived from src/front/app.at and the page files. The
driver follows real sidebar buttons, checks the route state and title, verifies
that every preview-card has a populated Auto preview, and saves scroll-position
screenshots under src/front/tests/screenshots (normally gitignored).
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

from test_vm_mcp import AutoUiMcpClient, pick_free_port


VM_INTERACTIONS = {
    "/alertdialog": ("Show Dialog", "Are you absolutely sure?"),
    "/dialog": ("Open Dialog", "Edit Profile"),
    "/sheet": ("Open", "Profile content goes here."),
    "/drawer": ("Open Drawer", "Edit Profile"),
    "/popover": ("Open Popover", "Set the dimensions for the layer."),
    "/dropdownmenu": ("Open", "Profile"),
    "/datepicker": ("Pick a date", "Su"),
    "/kitchen-sink": ("Open dialog", "Dialog content sample"),
}


def parse_front(front: Path) -> tuple[list[dict], dict[str, str]]:
    app = (front / "app.at").read_text(encoding="utf-8")
    routes = re.findall(r'"(/[^"\n]*)"\s*->\s*use\s+([a-zA-Z0-9_-]+)', app)
    labels: dict[str, str] = {}
    for match in re.finditer(r"sidebar_menu_button\s*\(([^)]*)\)\s*\{", app):
        props = match.group(1)
        route_match = re.search(r'to\s*:\s*"([^"]+)"', props)
        if not route_match:
            continue
        start = match.end()
        depth = 1
        i = start
        in_string = False
        escaped = False
        while i < len(app) and depth:
            ch = app[i]
            if in_string:
                if escaped:
                    escaped = False
                elif ch == "\\":
                    escaped = True
                elif ch == '"':
                    in_string = False
            elif ch == '"':
                in_string = True
            elif ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
            i += 1
        body = app[start : i - 1]
        text_match = re.search(r'\btext\s+"([^"]+)"', body)
        if not text_match:
            text_match = re.search(r'\btext\s*:\s*"([^"]+)"', props)
        if text_match:
            labels[route_match.group(1)] = text_match.group(1)

    entries: list[dict] = []
    seen: set[str] = set()
    for route, module in routes:
        page_path = front / "pages" / f"{module}.at"
        source = page_path.read_text(encoding="utf-8") if page_path.exists() else ""
        title_match = re.search(r'\bh1\s+"([^"]+)"', source)
        if not title_match:
            title_match = re.search(r'\bh1\s*\([^)]*\btext\s*:\s*"([^"]+)"', source)
        entries.append(
            {
                "route": route,
                "module": module,
                "title": title_match.group(1) if title_match else "",
                "preview_cards": len(re.findall(r"\bpreview-card\s*(?:\(|\{)", source)),
                "page_file": page_path.name,
                "routed": True,
            }
        )
        seen.add(module)

    # Special schema-generated showcase pages are part of the user-visible
    # gallery contract even if a source route was accidentally omitted.
    special = front / "pages" / "kitchen-sink.at"
    if special.exists():
        source = special.read_text(encoding="utf-8")
        title_match = re.search(r'\bh1\s+"([^"]+)"', source)
        if not title_match:
            title_match = re.search(r'\bh1\s*\([^)]*\btext\s*:\s*"([^"]+)"', source)
        entry = {
            "route": "/kitchen-sink",
            "module": "kitchen-sink",
            "title": title_match.group(1) if title_match else "Kitchen Sink",
            "preview_cards": len(re.findall(r"\bpreview-card\s*(?:\(|\{)", source)),
            "schema_examples": re.findall(r'\bh2\s+"([^"]+)"', source),
            "page_file": special.name,
            "routed": "/kitchen-sink" in {route for route, _ in routes},
        }
        existing = next((item for item in entries if item["module"] == "kitchen-sink"), None)
        if existing:
            existing.update(entry)
        else:
            entries.append(entry)
    return entries, labels


def parse_buttons(snapshot: str) -> dict[str, list[str]]:
    result: dict[str, list[str]] = {}
    for ident, label in re.findall(r'button\s+(#vnode_\d+)\s+"([^"]*)"', snapshot):
        result.setdefault(label, []).append(ident)
    return result


def exact_text_node(snapshot: str, text: str) -> bool:
    escaped = re.escape(text)
    return re.search(rf'^\s+text\s+#vnode_\d+\s+"{escaped}"(?:\s|$)', snapshot, re.M) is not None


def current_route(client: AutoUiMcpClient) -> str | None:
    state = client.state(["__current_route"])
    match = re.search(r'__current_route:\s+"([^"]+)"', state)
    return match.group(1) if match else None


def main_scrollable(snapshot: str, title: str) -> str | None:
    """Find the app's content scrollable by its proximity to the page heading."""
    lines = snapshot.splitlines()
    title_pattern = re.compile(rf'^\s*text\s+#vnode_\d+\s+"{re.escape(title)}"(?:\s|$)')
    title_indices = [i for i, line in enumerate(lines) if title_pattern.match(line)]
    if not title_indices:
        return None
    candidates: list[tuple[int, str]] = []
    for i, line in enumerate(lines):
        match = re.match(r'^\s*scrollable\s+#(vnode_\d+)', line)
        if not match:
            continue
        next_heading = next((heading for heading in title_indices if heading > i), None)
        if next_heading is not None:
            candidates.append((next_heading - i, match.group(1)))
    # VTree text may include multi-line code samples whose indentation looks
    # like widget nodes; choose the content pane nearest its first heading.
    return min(candidates)[1] if candidates else None


def empty_auto_previews(snapshot: str) -> list[str]:
    lines = snapshot.splitlines()
    empty: list[str] = []

    def indentation(index: int) -> int:
        return len(lines[index]) - len(lines[index].lstrip())

    def previous_sibling_block(index: int, level: int) -> int | None:
        """Return the opening line of the immediately preceding VTree sibling."""
        j = index - 1
        while j >= 0 and not lines[j].strip():
            j -= 1
        if j < 0 or lines[j].strip() != "}" or indentation(j) != level:
            return None
        depth = 1
        j -= 1
        while j >= 0:
            stripped = lines[j].strip()
            if stripped == "}":
                depth += 1
            elif stripped.endswith("{"):
                depth -= 1
                if depth == 0:
                    return j
            j -= 1
        return None

    for i, line in enumerate(lines):
        match = re.match(r'^(\s*)button\s+(#vnode_\d+)\s+"Auto"\s*$', line)
        if not match:
            continue
        button_indent = indentation(i)
        # The empty container immediately after the Auto button is its tab
        # underline. The sample pane is the previous sibling of the toolbar
        # row containing Auto, inside the preview-card wrapper.
        ancestors: list[tuple[int, int]] = []
        for j in range(i - 1, -1, -1):
            row = re.match(r'^\s*row\s+#vnode_\d+\s+\{\s*$', lines[j])
            if not row:
                continue
            level = indentation(j)
            if level >= button_indent:
                continue
            if any(lines[k].strip() == "}" and indentation(k) == level for k in range(j + 1, i)):
                continue
            ancestors.append((level, j))

        for level, row_index in ancestors:
            sibling_index = previous_sibling_block(row_index, level)
            if sibling_index is None:
                continue
            container = re.match(r'^\s*container\s+(#vnode_\d+)\s+\{\s*$', lines[sibling_index])
            if not container:
                continue
            content_index = sibling_index + 1
            while content_index < row_index and not lines[content_index].strip():
                content_index += 1
            # The preview area may wrap the sample in a Column. Empty means
            # the wrapper itself closes immediately with no child node.
            if content_index < row_index and lines[content_index].strip() == "}":
                empty.append(container.group(1))
            break
    return empty


def invisible_schema_examples(snapshot: str, names: list[str]) -> list[str]:
    """Find schema rows with no visible text or component leaf in the VTree."""
    lines = snapshot.splitlines()
    heading_indices: dict[str, int] = {}
    for name in names:
        pattern = re.compile(rf'^\s*text\s+#vnode_\d+\s+"{re.escape(name)}"(?:\s|$)')
        index = next((i for i, line in enumerate(lines) if pattern.match(line)), None)
        if index is not None:
            heading_indices[name] = index

    visual_leaf = re.compile(
        r'^\s*(?:text|button|input|checkbox|switch|radio|slider|progress|image|icon|calendar|select|textarea|canvas)\s+#vnode_\d+'
    )
    invisible: list[str] = []
    present = [(name, index) for name, index in heading_indices.items()]
    present.sort(key=lambda item: item[1])
    for offset, (name, index) in enumerate(present):
        end = present[offset + 1][1] if offset + 1 < len(present) else len(lines)
        sample = lines[index + 1 : end]
        # Divider and Separator are painted as native geometry by the VM
        # renderer, so their VTree subtree contains structural containers
        # rather than a named leaf node or text. Their visible lines are
        # verified in the gallery screenshots.
        native_geometry = name in {"divider", "separator"} and any(
            re.match(r"^\s*(?:container|col)\s+#vnode_", line) for line in sample
        )
        if not native_geometry and not any(visual_leaf.match(line) for line in sample):
            invisible.append(name)
    return invisible


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--auto-bin", required=True)
    parser.add_argument("--app-dir", required=True)
    parser.add_argument("--theme", choices=("light", "dark"), required=True)
    parser.add_argument("--output", help="Write JSON results to this path")
    parser.add_argument("--timeout", type=int, default=90)
    parser.add_argument("--max-scroll-frames", type=int, default=32)
    parser.add_argument("--checks-only", action="store_true", help="Skip screenshots while triaging routes")
    parser.add_argument("--interactions", action="store_true", help="Exercise representative interactive examples")
    parser.add_argument("--dump-tree-dir", help="Write one raw rendered tree per checked route")
    parser.add_argument("--only-route", action="append", help="Scan a route; may be repeated")
    args = parser.parse_args()

    app_dir = Path(args.app_dir).resolve()
    front = app_dir / "src" / "front"
    entries, source_labels = parse_front(front)
    if args.only_route:
        wanted = set(args.only_route)
        entries = [entry for entry in entries if entry["route"] in wanted]
    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port))
    command = [args.auto_bin, "run", "-r", "vm", "--theme", args.theme]
    proc = subprocess.Popen(command, cwd=app_dir, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    client = AutoUiMcpClient(port)
    results: list[dict] = []
    screenshot_dir = front / "tests" / "screenshots"
    screenshot_dir.mkdir(parents=True, exist_ok=True)

    try:
        snapshot = ""
        deadline = time.monotonic() + args.timeout
        while time.monotonic() < deadline:
            try:
                snapshot = client.snapshot()
                home_label = source_labels.get("/")
                controls_ready = not home_label or home_label in parse_buttons(snapshot)
                if "tree:" in snapshot and "PRE-RENDER FALLBACK" not in snapshot and controls_ready:
                    break
            except Exception:
                pass
            time.sleep(0.25)
        if "tree:" not in snapshot:
            raise RuntimeError("VM MCP did not produce a rendered snapshot before timeout")

        def capture(route: str, suffix: str) -> str:
            slug = "home" if route == "/" else route.strip("/").replace("/", "-")
            name = f"p717_vm_{args.theme}_{slug}_{suffix}"
            path = screenshot_dir / f"{name}.png"
            try:
                path.unlink()
            except FileNotFoundError:
                pass
            last_message = ""
            for attempt in range(3):
                try:
                    last_message = client.screenshot(name, baseline=True)
                except Exception as exc:
                    last_message = f"Error: {exc}"
                if path.exists() and "Error:" not in last_message:
                    return str(path)
                time.sleep(0.4 * (attempt + 1))
            record["screenshot_capture_response"] = last_message
            return ""

        for entry in entries:
            route = entry["route"]
            label = source_labels.get(route)
            buttons = parse_buttons(snapshot)
            record = {**entry, "theme": args.theme, "nav_label": label, "screenshots": [], "failures": []}
            if not entry["routed"]:
                record["failures"].append("missing source route")
                results.append(record)
                continue
            if not label or label not in buttons:
                record["failures"].append("missing live sidebar button")
                results.append(record)
                continue
            navigation_ok = False
            for attempt in range(3):
                buttons = parse_buttons(snapshot)
                if not buttons.get(label):
                    break
                action = client.press(buttons[label][0])
                route_deadline = time.monotonic() + 3
                while time.monotonic() < route_deadline:
                    if current_route(client) == route:
                        navigation_ok = True
                        break
                    time.sleep(0.1)
                if navigation_ok:
                    break
                snapshot = client.snapshot()
            snapshot = client.snapshot()
            if args.dump_tree_dir:
                tree_dir = Path(args.dump_tree_dir)
                tree_dir.mkdir(parents=True, exist_ok=True)
                tree_slug = "home" if route == "/" else route.strip("/").replace("/", "-")
                (tree_dir / f"{tree_slug}.txt").write_text(snapshot, encoding="utf-8")
            if not navigation_ok:
                record["failures"].append("navigation did not reach the expected route")
            bounded_snapshot = client.call(
                "autoui_snapshot", {"mode": "rendered", "include_bounds": True}
            ).get("content", [{}])[0].get("text", "")
            if args.dump_tree_dir:
                (Path(args.dump_tree_dir) / f"{tree_slug}.bounds.txt").write_text(bounded_snapshot, encoding="utf-8")
            record["title_visible"] = exact_text_node(snapshot, entry["title"])
            if not record["title_visible"]:
                record["failures"].append("expected page title is missing from rendered text nodes")

            auto_count = len(re.findall(r'button\s+#vnode_\d+\s+"Auto"', snapshot))
            record["live_preview_cards"] = auto_count
            if auto_count != entry["preview_cards"]:
                record["failures"].append(
                    f"preview-card count mismatch: source={entry['preview_cards']} live={auto_count}"
                )
            empty = empty_auto_previews(snapshot)
            record["empty_preview_containers"] = empty
            if empty:
                record["failures"].append(f"empty Auto preview container(s): {', '.join(empty)}")

            if entry["module"] == "kitchen-sink":
                names = entry["schema_examples"]
                record["live_schema_examples"] = sum(
                    exact_text_node(snapshot, name) for name in names
                )
                record["invisible_schema_examples"] = invisible_schema_examples(snapshot, names)
                record["visible_schema_examples"] = len(names) - len(record["invisible_schema_examples"])
                if record["live_schema_examples"] != len(names):
                    record["failures"].append(
                        f"schema heading count mismatch: source={len(names)} live={record['live_schema_examples']}"
                    )
                if record["invisible_schema_examples"]:
                    record["failures"].append(
                        "schema examples without visible content: "
                        + ", ".join(record["invisible_schema_examples"])
                    )

            content_scroll_id = main_scrollable(bounded_snapshot, entry["title"])
            if not content_scroll_id:
                record["failures"].append("main page scrollable is missing")
                results.append(record)
                continue

            if args.checks_only:
                results.append(record)
                continue

            reset = client.call(
                "autoui_action",
                {"element_id": content_scroll_id, "action": "scroll", "value": 0},
            )
            reset_message = reset.get("content", [{}])[0].get("text", "")
            if reset.get("isError") or "status: ok" not in reset_message:
                record["failures"].append(f"could not reset main scroll: {reset_message[:140]}")

            # Capture each successive viewport until the native scroll extent
            # stops changing. A repeated screenshot marks the true bottom.
            previous_hash = None
            step = 640
            for frame in range(args.max_scroll_frames):
                offset = frame * step
                if frame:
                    response = client.call(
                        "autoui_action",
                        {"element_id": content_scroll_id, "action": "scroll", "value": offset},
                    )
                    message = response.get("content", [{}])[0].get("text", "")
                    if response.get("isError") or "status: ok" not in message:
                        record["failures"].append(f"scroll failed at y={offset}: {message[:140]}")
                        break
                    snapshot = client.snapshot()
                shot = capture(route, f"y{offset:04d}")
                if not shot:
                    detail = record.get("screenshot_capture_response", "no response")
                    record["failures"].append(f"screenshot capture failed at y={offset}: {detail}")
                    break
                record["screenshots"].append(shot)
                try:
                    import hashlib
                    from PIL import Image

                    with Image.open(shot) as image:
                        digest = hashlib.sha256(image.convert("RGBA").tobytes()).hexdigest()
                except OSError:
                    digest = None
                if frame > 0 and digest and digest == previous_hash:
                    record["screenshots"].pop()
                    try:
                        Path(shot).unlink()
                    except OSError:
                        pass
                    break
                previous_hash = digest

            if args.interactions and route in VM_INTERACTIONS:
                button_label, expected_text = VM_INTERACTIONS[route]
                button_id = parse_buttons(snapshot).get(button_label, [None])[0]
                if not button_id:
                    record["failures"].append(f"interaction trigger is missing: {button_label}")
                else:
                    action_result = client.press(button_id)
                    interaction_snapshot = ""
                    interaction_deadline = time.monotonic() + 4
                    while time.monotonic() < interaction_deadline:
                        interaction_snapshot = client.snapshot()
                        if exact_text_node(interaction_snapshot, expected_text):
                            break
                        time.sleep(0.1)
                    if not exact_text_node(interaction_snapshot, expected_text):
                        record["interaction_snapshot_tail"] = "\n".join(interaction_snapshot.splitlines()[-80:])
                        debug_shot = capture(route, "interaction-debug")
                        if debug_shot:
                            record["interaction_debug_screenshot"] = debug_shot
                        record["failures"].append(
                            f"interaction did not show {expected_text!r}: {action_result[:180]}"
                        )
                    else:
                        record["interaction_text_visible"] = expected_text
                        shot = capture(route, "interaction")
                        if not shot:
                            detail = record.get("screenshot_capture_response", "no response")
                            record["failures"].append(f"interaction screenshot failed: {detail}")
                        else:
                            record["interaction_screenshot"] = shot
            results.append(record)

        summary = {
            "backend": "vm",
            "theme": args.theme,
            "route_count": len(entries),
            "pass_count": sum(not row["failures"] for row in results),
            "fail_count": sum(bool(row["failures"]) for row in results),
            "results": results,
        }
        if args.output:
            output = Path(args.output)
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")
        print(json.dumps({k: v for k, v in summary.items() if k != "results"}, ensure_ascii=False))
        for row in results:
            if row["failures"]:
                print(f"FAIL {row['route']} ({row['title']}): {'; '.join(row['failures'])}")
        return 0 if summary["fail_count"] == 0 else 1
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    raise SystemExit(main())
