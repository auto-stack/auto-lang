"""PLAN-685 review: Q-4 keep-section check under a NON-default section."""
import json
import re
import time
import urllib.request

URL = "http://127.0.0.1:9778/mcp"
BTN = re.compile(r'button #(\S+) "((?:[^"\\]|\\.)*)"')


def call(tool, args):
    req = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                      "params": {"name": tool, "arguments": args}}).encode()
    r = urllib.request.Request(URL, data=req, headers={"Content-Type": "application/json"})
    return json.load(urllib.request.urlopen(r, timeout=15)).get("result", {}).get(
        "content", [{}])[0].get("text", "")


def find(label, exact=False):
    for m in BTN.finditer(call("autoui_snapshot", {"mode": "rendered"})):
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


press(find("Gotchas", exact=True))
time.sleep(1.0)
print("section after Gotchas tab:", field("active_section"))
press(find("form/login"))
time.sleep(1.5)
print("after bp switch to form/login: selected=%s section=%s (expect keep=gotchas)" %
      (field("selected_id"), field("active_section")))
press(find("dashboard", exact=True))
time.sleep(1.5)
print("after kind breadcrumb switch: selected=%s kind-filter active=%s section=%s" %
      (field("selected_id"), field("active_kind"), field("active_section")))
