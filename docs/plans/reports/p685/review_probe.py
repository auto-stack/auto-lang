"""PLAN-685 review: AC-07 tab switching + AC-08 header runtime probe."""
import json
import re
import sys
import time
import urllib.request

PORT = sys.argv[1] if len(sys.argv) > 1 else "9778"
URL = f"http://127.0.0.1:{PORT}/mcp"
BTN = re.compile(r'button #(\S+) "((?:[^"\\]|\\.)*)"')


def call(tool, args):
    req = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                      "params": {"name": tool, "arguments": args}}).encode()
    r = urllib.request.Request(URL, data=req, headers={"Content-Type": "application/json"})
    return json.load(urllib.request.urlopen(r, timeout=15)).get("result", {}).get(
        "content", [{}])[0].get("text", "")


def snap():
    return call("autoui_snapshot", {"mode": "rendered"})


def find(label, exact=False):
    for m in BTN.finditer(snap()):
        vid, txt = m.group(1), m.group(2).split("\n")[0]
        if (exact and txt == label) or (not exact and txt.startswith(label)):
            return vid
    return None


def press(eid):
    call("autoui_action", {"element_id": eid, "action": "press"})


def field(name):
    t = call("autoui_state", {"fields": [name]})
    m = re.search(rf'{name}:\s*"(.*?)"', t)
    return m.group(1) if m else "?"


for _ in range(120):
    try:
        if "tree:" in snap():
            break
    except Exception:
        pass
    time.sleep(0.3)
time.sleep(3)

print("initial:", field("selected_id"), field("active_section"))
s0 = snap()
print("AC-07a spec-only at start: Intent-present=%s gotchas-absent=%s" %
      ("Intent" in s0, "Gotchas —" not in s0))
for label, marker, old in [("Reference", "Source of", "Intent"),
                           ("Gotchas", "Gotchas —", "Source of"),
                           ("Spec", "Intent", "Source of")]:
    tab = find(label, exact=True)
    press(tab)
    time.sleep(1.0)
    s = snap()
    print("AC-07b tab %s: state=%s marker=%s old-absent=%s" %
          (label, field("active_section"), marker in s, old not in s))
card = find("data-display/row-list")
press(card)
time.sleep(1.5)
print("AC-07c after bp switch: selected=%s section=%s (Q-4 keep)" %
      (field("selected_id"), field("active_section")))
s = snap()
print("AC-08 header: kind=%s name=%s prev=%s next=%s badge=%s" %
      ("data-display" in s, "row-list" in s, '"‹"' in s, '"›"' in s,
       "1 variant" in s))
