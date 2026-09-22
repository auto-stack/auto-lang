"""Retake one grid cell with section-state verification.

Usage: python shoot_one.py <port> <bp_id> <section_label> <out_png>
"""
import json
import os
import re
import shutil
import sys
import time
import urllib.request

PORT, BP_ID, LABEL, OUT_PNG = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
URL = f"http://127.0.0.1:{PORT}/mcp"
BTN_RE = re.compile(r'button #(\S+) "((?:[^"\\]|\\.)*)"')
SECT_KEY = {"Spec": "spec", "Reference": "reference", "Gotchas": "gotchas"}


def call(tool, args):
    req = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                      "params": {"name": tool, "arguments": args}}).encode()
    r = urllib.request.Request(URL, data=req, headers={"Content-Type": "application/json"})
    return json.load(urllib.request.urlopen(r, timeout=15)).get("result", {}).get(
        "content", [{}])[0].get("text", "")


def find_button(text, label, exact=False):
    for m in BTN_RE.finditer(text):
        vid, txt = m.group(1), m.group(2).split("\n")[0]
        if (exact and txt == label) or (not exact and txt.startswith(label)):
            return vid
    return None


def field(state_text, name):
    m = re.search(rf'{name}:\s*"(.*?)"', state_text)
    return m.group(1) if m else ""


t = call("autoui_snapshot", {"mode": "rendered"})
card = find_button(t, BP_ID)
assert card, f"card {BP_ID} not found"
for _ in range(4):
    call("autoui_action", {"element_id": card, "action": "press"})
    time.sleep(1.0)
    if BP_ID in call("autoui_state", {"fields": ["selected_id"]}):
        break
for _ in range(4):
    tab = find_button(call("autoui_snapshot", {"mode": "rendered"}), LABEL, exact=True)
    assert tab, f"tab {LABEL} not found"
    call("autoui_action", {"element_id": tab, "action": "press"})
    time.sleep(1.0)
    if field(call("autoui_state", {"fields": ["active_section"]}), "active_section") == SECT_KEY[LABEL]:
        break
sect = field(call("autoui_state", {"fields": ["active_section"]}), "active_section")
sel = field(call("autoui_state", {"fields": ["selected_id"]}), "selected_id")
print(f"state: selected={sel} section={sect}")
assert sect == SECT_KEY[LABEL] and sel == BP_ID, "state did not settle"
for attempt in range(5):
    res = call("autoui_screenshot", {"name": f"retake_{attempt}", "baseline": False})
    m = re.search(r"saved to:\s*(.+)", res)
    if m:
        shutil.copyfile(m.group(1).strip(), OUT_PNG)
        if os.path.getsize(OUT_PNG) > 10000:
            print(f"saved {OUT_PNG} ({os.path.getsize(OUT_PNG)}B)")
            break
    time.sleep(2.0)
