import json, urllib.request, re, time

def call(n, a=None, timeout=60):
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": n, "arguments": a or {}}}
    r = urllib.request.Request(
        "http://127.0.0.1:9247/mcp", data=json.dumps(req).encode(),
        headers={"Content-Type": "application/json"})
    return json.loads(urllib.request.urlopen(r, timeout=timeout).read().decode()
                     ).get("result", {}).get("content", [{}])[0].get("text", "")

def snap():
    return call("autoui_snapshot", {"mode": "rendered"})

def route():
    t = call("autoui_state", {"fields": ["__current_route"]})
    m = re.search(r'__current_route: "([^"]*)"', t)
    return m.group(1) if m else ""

def goto(label, want):
    t0 = time.time()
    s = snap()
    m = re.search(r'button #(vnode_\d+) "%s"' % re.escape(label), s)
    if not m:
        return None, time.time() - t0
    press_t0 = time.time()
    call("autoui_action", {"element_id": m.group(1), "action": "press"})
    press_dt = time.time() - press_t0
    for _ in range(80):
        time.sleep(0.2)
        if route() == want:
            return {"press_s": round(press_dt, 2), "total_s": round(time.time() - t0, 2)}, time.time() - t0
    return {"press_s": round(press_dt, 2), "total_s": round(time.time() - t0, 2), "route": route()}, time.time() - t0

pages = [
    ("Home", "/"),
    ("Row", "/row"),
    ("Column", "/col"),
    ("Button", "/button"),
    ("AreaChart", "/area-chart"),
    ("DataTable", "/datatable"),
    ("FileTree", "/filetree"),
    ("Command", "/command"),
    ("Sidebar", "/sidebar"),
    ("Carousel", "/carousel"),
]
for label, want in pages:
    info, _ = goto(label, want)
    print(f"{label:12s} {info}", flush=True)
    time.sleep(0.3)
