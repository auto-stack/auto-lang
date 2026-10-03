import json, urllib.request, re, time

def call(n, a=None, timeout=60):
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": n, "arguments": a or {}}}
    r = urllib.request.Request(
        "http://127.0.0.1:4288/mcp",
        data=json.dumps(req).encode(),
        headers={"Content-Type": "application/json"},
    )
    res = json.loads(urllib.request.urlopen(r, timeout=timeout).read().decode())
    return res.get("result", {}).get("content", [{}])[0].get("text", "")

s = call("autoui_snapshot", {"mode": "rendered"})
print("len", len(s))
print("route", call("autoui_state", {"fields": ["__current_route"]})[:150])
print("has Layout group", "Layout" in s)
print("btn count", len(re.findall(r"button #", s)))
# find openSidebar
for m in re.finditer(r"onClick: \.(\w+)", s):
    pass
print("handlers", set(re.findall(r"onClick: \.(\w+)", s)))
# hamburger / openSidebar node
m = re.search(r"#(vnode_\d+)[^\n]*\n[^\n]*onClick: \.openSidebar", s)
if not m:
    m = re.search(r"(vnode_\d+)[^\n]{0,80}openSidebar", s)
print("openSidebar node", m.group(1) if m else None)
# try press openSidebar
if m:
    try:
        print("press open", call("autoui_action", {"element_id": m.group(1), "action": "press"}, 30)[:300])
    except Exception as e:
        print("press err", e)
    time.sleep(0.4)
    s = call("autoui_snapshot", {"mode": "rendered"})
    print("after open len", len(s), "Layout" in s, "Column" in s)

# screenshot
print("shot", call("autoui_screenshot", {"name": "vm_current", "baseline": True})[:200])
