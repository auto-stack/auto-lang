import json, urllib.request, re, time

def call(n, a=None):
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": n, "arguments": a or {}}}
    r = urllib.request.Request(
        "http://127.0.0.1:4288/mcp",
        data=json.dumps(req).encode(),
        headers={"Content-Type": "application/json"},
    )
    return json.loads(urllib.request.urlopen(r, timeout=20).read().decode()
                     ).get("result", {}).get("content", [{}])[0].get("text", "")

def snap():
    return call("autoui_snapshot", {"mode": "rendered"})

home = snap()
m = re.search(r'button #(vnode_\d+) "AreaChart"', home)
print("target", m.group(1) if m else None)
if not m:
    raise SystemExit(1)
vid = m.group(1)
print("inspect:", call("autoui_inspect", {"element_id": vid})[:600])
print("--- press ---")
print(call("autoui_action", {"element_id": vid, "action": "press"})[:400])
time.sleep(0.6)
t2 = snap()
print("len", len(t2), "changed", t2 != home)
# unique texts in content area - look for headings after Outlet-like section
# print unique-ish long text nodes that differ from home
hset = set(re.findall(r'text #vnode_\d+ "([^"]{6,60})"', home))
tset = set(re.findall(r'text #vnode_\d+ "([^"]{6,60})"', t2))
print("new texts:", list(tset - hset)[:20])
print("gone texts:", list(hset - tset)[:10])
open(r"D:/autostack/auto-lang/.tmp-vm-after-areachart.txt", "w", encoding="utf-8").write(t2)
