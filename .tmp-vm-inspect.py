import json, urllib.request, re

def call(n, a=None):
    req = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
           "params": {"name": n, "arguments": a or {}}}
    r = urllib.request.Request(
        "http://127.0.0.1:4288/mcp",
        data=json.dumps(req).encode(),
        headers={"Content-Type": "application/json"},
    )
    return json.loads(urllib.request.urlopen(r, timeout=15).read().decode()
                     ).get("result", {}).get("content", [{}])[0].get("text", "")

t = open(r"D:/autostack/auto-lang/.tmp-vm-home.txt", encoding="utf-8").read()
print("=== keyword hits ===")
for kw in ["outlet", "Outlet", "NavTo", "onClick", "menu_button", "sidebar_menu",
           "active", "route", ".Go", "Link", "href"]:
    hits = [ln[:150] for ln in t.splitlines() if kw in ln][:3]
    print(f"{kw}: {hits}")

print("\n=== AreaChart button ===")
m = re.search(r'button #vnode_(\d+) "AreaChart"', t)
print("id", m.group(1) if m else None)
if m:
    print(call("autoui_inspect", {"element_id": m.group(1)})[:1200])

print("\n=== try press AreaChart ===")
if m:
    print(call("autoui_action", {"element_id": m.group(1), "action": "press"})[:400])
    import time
    time.sleep(0.5)
    t2 = call("autoui_snapshot", {"mode": "rendered"})
    print("new len", len(t2), "same as home", t2 == t)
    # look for AreaChart page content
    for kw in ["AreaChart", "area-chart", "Overlay", "SVG", "svg", "Preview", "preview"]:
        print(f"  {kw}: {t2.count(kw)}")
    # save
    open(r"D:/autostack/auto-lang/.tmp-vm-after-areachart.txt", "w", encoding="utf-8").write(t2)
