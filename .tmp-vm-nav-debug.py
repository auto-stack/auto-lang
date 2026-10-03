import json, urllib.request, re, time

def call(n, a=None):
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": n, "arguments": a or {}}}
    r = urllib.request.Request(
        "http://127.0.0.1:4288/mcp",
        data=json.dumps(req).encode(),
        headers={"Content-Type": "application/json"},
    )
    res = json.loads(urllib.request.urlopen(r, timeout=20).read().decode())
    return res.get("result", {}).get("content", [{}])[0].get("text", "")

# state?
print("=== state ===")
try:
    print(call("autoui_state", {"fields": ["__current_route", "searchQuery"]})[:500])
except Exception as e:
    print("state err", e)

print("\n=== inspect Column button on current page ===")
s = call("autoui_snapshot", {"mode": "rendered"})
print("snap len", len(s), "route texts sample:")
# find all buttons with navigate
navs = re.findall(r'button #(vnode_\d+) "([^"]+)"[\s\S]{0,200}?__navigate\S*?(/[a-z0-9\-]*)', s)
print("navs found", len(navs))
for a,b,c in navs[:8]:
    print(" ", a, b, "->", c)

# try navigate via state write?
print("\n=== try fixture / state write route ===")
for args in [
    {"state": {"__current_route": "/column"}},
    {"fields": ["__current_route"]},
]:
    try:
        print("call", args, "=>", call("autoui_state", args)[:200])
    except Exception as e:
        print("err", e)

# press Column with inspect first
m = re.search(r'button #(vnode_\d+) "Column"', s)
print("\nColumn btn", m.group(1) if m else None)
if m:
    print("inspect", call("autoui_inspect", {"element_id": m.group(1)})[:400])
    print("press", call("autoui_action", {"element_id": m.group(1), "action": "press"})[:500])
    time.sleep(0.5)
    s2 = call("autoui_snapshot", {"mode": "rendered"})
    texts = re.findall(r'text #vnode_\d+ "([^"]{6,40})"', s2)
    print("after press texts sample", texts[10:25])
    print("route state", call("autoui_state", {"fields": ["__current_route"]})[:200])
